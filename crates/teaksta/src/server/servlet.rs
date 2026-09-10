//! The server side implementation of the WERTi service.
//!
//! This is where the work is coordinated. The rough outline of the procedure
//! is as follows:
//!
//! - We take a request via the `handle_get` (web form) or `handle_post`
//!   (add-on) methods.
//! - For web form requests, the HTML source of the URL is fetched and spans of
//!   text are identified.
//! - The document is processed in the pipeline, invoking the pre- and
//!   postprocessors for the current topic.
//! - Afterwards, we take the resulting CAS and insert enhancement annotations
//!   (`WERTi`-`<span>`s) according to the target annotations from the
//!   postprocessor.
//!
//! Authors: Aleksandar Dimitrov, Adriane Boyd
//!
//! The servlet container types this class is written against — request,
//! response, config and context — have no poem counterpart with the same
//! shape, so the minimum surface each one is used through is modelled locally
//! at the bottom of this module. The session half of the request is the one
//! `ActivitiesSessionLoader` already models, and is threaded through as its
//! own parameter so that the activity registry it hands out can be mutated
//! while the rest of the request is still being read.

use std::collections::{BTreeMap, HashSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use poem::http::StatusCode;
use poem::http::header::{CONTENT_TYPE, LOCATION};
use poem::{Body, IntoResponse, Response};
use regex::Regex;
use reqwest::Url;
use scraper::Html;
use tracing::{debug, error, info, warn};

use crate::server::activities::Activities;
use crate::server::activities::ActivitiesSessionLoader;
use crate::server::activities::HttpServletRequest as SessionRequest;
use crate::server::activity_configuration::{
    ActivityConfiguration, CLIENT_PREFIX, POST_PREFIX, PRE_PREFIX,
};
use crate::server::processors::Processors;
use crate::util::html_enhancer::HtmlEnhancer;
use crate::util::html_utils;
use crate::util::json_enhancer::JsonEnhancer;
use crate::util::page_handler::PageHandler;
use crate::util::post_request::PostRequest;
use crate::util::practice_handler::PracticeHandler;

pub const OUTPUTFILE_LOC: &str = "InputLog.txt";

/// maximum amount of of ms to wait for a web-page to load
const MAX_WAIT: i32 = 1000 * 20; // 20 seconds, was: 10

pub const SERIAL_VERSION_UID: i64 = 10;

pub static SUPPORTED_VERSIONS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["0.10"]));

/// The consumer is constructed on first use and then shared by every request,
/// exactly as the static field it stands in for.
pub static OPENID_CONSUMER: RwLock<Option<OpenIdConsumer>> = RwLock::new(None);

/// colorize, click, mc or cloze
///
/// Written on every request and read by the enhancers, so concurrent requests
/// see each other's value.
pub static ENHANCEMENT_TYPE: RwLock<Option<String>> = RwLock::new(None);

/// Reader for [`ENHANCEMENT_TYPE`] that mirrors a plain static-field read:
/// the unset field reads as the empty string rather than propagating a
/// failure.
pub fn enhancement_type() -> String {
    ENHANCEMENT_TYPE
        .read()
        .ok()
        .and_then(|value| value.clone())
        .unwrap_or_default()
}

/// Java renders a null reference as the four characters `null` when it is
/// concatenated into a string or handed to the logger; every request
/// parameter that reaches a message here is nullable.
fn null_str(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

/// `new ServletException(message)` — the cause-less form, which discards
/// whatever was caught.
fn servlet_exception(message: &str) -> anyhow::Error {
    anyhow!(message.to_string())
}

/// `new ServletException(message, cause)`. The messages passed at the call
/// sites are empty strings; they are kept verbatim.
fn servlet_exception_cause(message: &str, cause: anyhow::Error) -> anyhow::Error {
    cause.context(message.to_string())
}

// [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet]
#[derive(Default)]
pub struct WertiServlet {
    processors: Option<Processors>,
    servlet_config: Option<ServletConfig>,
}

impl WertiServlet {
    pub fn new() -> Self {
        WertiServlet::default()
    }

    /// `HttpServlet#getServletContext()`, which reads through the config
    /// stashed by `init`.
    fn get_servlet_context(&self) -> Option<&ServletContext> {
        self.servlet_config
            .as_ref()
            .map(|config| config.get_servlet_context())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.init-fn]
    pub fn init(&mut self, config: ServletConfig) -> Result<()> {
        // `super.init(config)`: stash the config so `get_servlet_context`
        // works later.
        self.servlet_config = Some(config.clone());
        warn!("Initializing servlet.");
        // initialise servletcontext
        match crate::context::init(&config) {
            Ok(()) => {}
            Err(wce) => {
                error!("Context failed to initialize.");
                error!("{}", wce);
            }
        }

        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.destroy-fn]
    pub fn destroy(&mut self) {
        // no-op
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn]
    pub fn handle_get(
        &mut self,
        req: &mut HttpServletRequest,
        session_request: &mut SessionRequest,
        resp: &mut HttpServletResponse,
    ) -> Result<()> {
        req.set_character_encoding("UTF-8");
        resp.set_character_encoding("UTF-8");

        resp.set_content_type("text/html");

        // `req.getSession(true)`: the session is created on demand.
        let wait_page_absent = {
            let session = session_request.get_session();
            if session.get_attribute("waitPage").is_none() {
                session.set_attribute("waitPage", Box::new(true));
                true
            } else {
                session.remove_attribute("waitPage");
                false
            }
        };

        if wait_page_absent {
            let written = (|| -> Result<()> {
                let mut out = resp.get_writer()?;

                out.println("<html><head>");
                out.println("<title>Vuorddes...</title>");
                out.println("<meta http-equiv=\"Refresh\" content=\"0\">");
                out.println("</head><body>");
                out.println("<br><br><br>");
                out.println("<center><h1 style='color:#144ea6;'>Prográmma lea bargame.<br>");
                out.println("Vuorddes...</h1></center>");
                out.println("<center><img src='images/ajax-loader.gif' />");
                out.close();
                Ok(())
            })();

            if let Err(ioe) = written {
                error!("Failed to write to temporary wait file");
                return Err(servlet_exception_cause("", ioe));
            }

            return Ok(());
        }

        let start_time = Instant::now();
        debug!("received GET request");

        // OpenID verification request
        if req.get_parameter("openid_return") == Some("true") {
            let mut consumer = OPENID_CONSUMER
                .write()
                .map_err(|_| anyhow!("WERTiServlet.openidConsumer lock poisoned"))?;
            if consumer.is_none() {
                let openid_return_to_url = Self::get_open_id_return_to_url(req);
                *consumer = Some(OpenIdConsumer::new(openid_return_to_url));
            }

            let verified = match consumer.as_ref() {
                Some(consumer) => consumer.verify_response(req)?,
                None => return Err(anyhow!("NullPointerException: openidConsumer")),
            };
            match verified {
                Some(verified) => resp.send_redirect(&format!(
                    "{}/openid/return.jsp?openid.identity={}",
                    Self::get_servlet_base_url(req),
                    verified
                )),
                None => resp.send_redirect(&format!(
                    "{}/openid/verification-failed.jsp",
                    Self::get_servlet_base_url(req)
                )),
            }

            return Ok(());
        }

        let url = req.get_parameter("url");
        // accept url-s without http://
        info!("url:{}", null_str(url));
        let mut url = url
            .ok_or_else(|| anyhow!("NullPointerException: request parameter \"url\""))?
            .to_string();
        if !url.starts_with("file:/") && !url.contains("http") {
            url = format!("http://{}", url);
        }

        let activity = req.get_parameter("activity").map(str::to_string);
        let enhancement = req.get_parameter("client.enhancement").map(str::to_string);
        *ENHANCEMENT_TYPE
            .write()
            .map_err(|_| anyhow!("WERTiServlet.enhancement_type lock poisoned"))? =
            enhancement.clone();
        let lang = req.get_parameter("language").map(str::to_string);
        let lang = match lang {
            Some(lang) => lang,
            None => "en".to_string(),
        };

        let mut config =
            self.load_activities_and_processors(session_request, activity.as_deref())?;
        info!(
            "config:{}",
            match config.as_deref() {
                Some(config) => config.to_string(),
                None => "null".to_string(),
            }
        );
        // merge config with request parameters
        self.merge_config_params(config.as_deref_mut(), req)?;

        let u = Url::parse(&url).map_err(|e| anyhow!("MalformedURLException: {}: {}", url, e))?;
        info!("URL again:{}", u);
        let html_doc = (|| -> Result<Html> {
            if !url.starts_with("file:/") {
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_millis(MAX_WAIT as u64))
                    .build()?;
                let page = client.get(u.clone()).send()?.text()?;
                Ok(Html::parse_document(&page))
            } else {
                let myinput = url
                    .get(7..)
                    .ok_or_else(|| anyhow!("StringIndexOutOfBoundsException: {}", url))?;
                let page = std::fs::read_to_string(myinput)?;
                Ok(Html::parse_document(&page))
            }
        })();
        let mut html_doc = match html_doc {
            Ok(html_doc) => html_doc,
            Err(_ioe) => return Err(servlet_exception("Webpage retrieval failed.")),
        };

        // `htmlDoc.body()`
        let body = html_doc
            .tree
            .nodes()
            .find(|node| {
                node.value()
                    .as_element()
                    .is_some_and(|element| element.name() == "body")
            })
            .map(|node| node.id())
            .ok_or_else(|| anyhow!("NullPointerException: document has no <body>"))?;
        html_utils::mark_text_nodes(&mut html_doc, body)?;

        let html_string = self.spans_to_e_tags(&html_doc, html_utils::CLASS_NAME, false)?;

        let anl_dir_path = self
            .get_servlet_context()
            .and_then(|context| context.get_init_parameter("files_anl_dir"))
            .map(str::to_string);
        let anl_dir_path = anl_dir_path
            .ok_or_else(|| anyhow!("NullPointerException: init parameter \"files_anl_dir\""))?;
        let activity = activity
            .ok_or_else(|| anyhow!("NullPointerException: request parameter \"activity\""))?;
        let processors = self
            .processors
            .as_ref()
            .ok_or_else(|| anyhow!("NullPointerException: processors"))?;
        let ph = PageHandler::new(
            processors,
            &activity,
            &url.replace("/", "-"),
            &anl_dir_path,
            &html_string,
            &lang,
        );
        let cas = ph.process()?;

        if let Some(cas) = cas {
            let ge = HtmlEnhancer::new(&cas);
            let result = ge.enhance(
                &activity,
                &u.to_string(),
                req,
                config
                    .as_deref()
                    .ok_or_else(|| anyhow!("NullPointerException: config"))?,
                null_str(
                    self.get_servlet_context()
                        .and_then(|context| context.get_servlet_context_name()),
                ),
            )?;

            info!(
                "Web ({}): {},  {}, {}, {}, {}",
                start_time.elapsed().as_millis(),
                null_str(req.get_parameter("language")),
                activity,
                null_str(req.get_parameter("client.enhancement")),
                url,
                cas.language
            );

            // Write the url, topic and enhancement type into the file as well:
            // the true will append the new data
            let mut outputfile = OpenOptions::new()
                .create(true)
                .append(true)
                .open(OUTPUTFILE_LOC)?;
            let written = outputfile.write_all(
                format!(
                    "Topic: {}, exercise type: {}, URL: {}\n",
                    activity,
                    null_str(enhancement.as_deref()),
                    url
                )
                .as_bytes(),
            );
            drop(outputfile);
            written?;

            let written = (|| -> Result<()> {
                let mut out = resp.get_writer()?;

                out.write(&result);
                out.close();
                Ok(())
            })();
            if let Err(ioe) = written {
                error!("Failed to write to temporary result file");
                return Err(servlet_exception_cause("", ioe));
            }

            Ok(())
        } else {
            Err(servlet_exception(
                "The selected language/topic/activity combination is not currently available.",
            ))
        }
    }

    /// Annotate according to the topic/activity/text provided in a JSON
    /// PostRequestObject.
    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-post-fn]
    pub fn handle_post(
        &mut self,
        req: &HttpServletRequest,
        session_request: &mut SessionRequest,
        resp: &mut HttpServletResponse,
    ) -> Result<()> {
        if req.get_parameter("word").is_some() {
            info!("LogServlet received POST request");
            let message: String;
            let correct: String;

            // true -> the new data will be appended to the end of the file,
            // instead of overwriting the file
            let mut outputfile = OpenOptions::new()
                .create(true)
                .append(true)
                .open(OUTPUTFILE_LOC)?;

            let extype = req.get_parameter("extype");
            let word = req.get_parameter("word");
            let facit = req.get_parameter("facit");
            if full_match(
                req.get_parameter("correct").ok_or_else(|| {
                    anyhow!("NullPointerException: request parameter \"correct\"")
                })?,
                "1",
            )? {
                correct = "yes".to_string();
            } else {
                correct = "no".to_string();
            }
            let target_correct = req.get_parameter("correctly_clicked");
            let target_total = req.get_parameter("total_clicked");
            if full_match(
                extype
                    .ok_or_else(|| anyhow!("NullPointerException: request parameter \"extype\""))?,
                "click",
            )? {
                message = format!(
                    "The clicked word: {}. Correct: {}. The user has clicked correctly {} words out of {}.",
                    null_str(word),
                    correct,
                    null_str(target_correct),
                    null_str(target_total)
                );
            } else {
                message = format!(
                    "User's answer: {}. Facit: {}. Correct: {}. The user has written/chosen correctly {} words out of {}.",
                    null_str(word),
                    null_str(facit),
                    correct,
                    null_str(target_correct),
                    null_str(target_total)
                );
            }

            let written = outputfile.write_all(format!("{}\n", message).as_bytes());
            drop(outputfile);
            written?;

            Ok(())
        } else if req.get_parameter("nr_of_exercises").is_some() {
            info!("LogServlet received POST request");

            let mut outputfile = OpenOptions::new()
                .create(true)
                .append(true)
                .open(OUTPUTFILE_LOC)?;

            let nr_of_exercises = req.get_parameter("nr_of_exercises");

            let message = format!(
                "Number of exercises on the page: {}.",
                null_str(nr_of_exercises)
            );
            let written = outputfile.write_all(format!("{}\n", message).as_bytes());
            drop(outputfile);
            written?;

            Ok(())
        } else {
            let start_time = Instant::now();
            debug!("received POST request");

            // read request in as string
            let mut request_string = String::new();
            let reader = req.get_reader();
            for line in reader.lines() {
                request_string += line;
            }

            // parse this string into an object
            let request_info: PostRequest = serde_json::from_str(&request_string)?;

            // check if this version is supported
            if !request_info
                .version
                .as_deref()
                .is_some_and(|version| SUPPORTED_VERSIONS.contains(version))
            {
                resp.send_error(490);
                info!(
                    "Add-on, version conflict ({}): {}, {}, {}",
                    start_time.elapsed().as_millis(),
                    null_str(request_info.topic.as_deref()),
                    null_str(request_info.activity.as_deref()),
                    null_str(request_info.url.as_deref())
                );
                return Ok(());
            }

            // if this is an OpenID authentication request, handle it separately
            if full_match(
                request_info
                    .r#type
                    .as_deref()
                    .ok_or_else(|| anyhow!("NullPointerException: requestInfo.type"))?,
                "openid-authentication",
            )? {
                let user_supplied_identifier = request_info.url.as_deref();
                debug!(
                    "requestInfo.document: {}",
                    null_str(request_info.document.as_deref())
                );
                let mut consumer = OPENID_CONSUMER
                    .write()
                    .map_err(|_| anyhow!("WERTiServlet.openidConsumer lock poisoned"))?;
                if consumer.is_none() {
                    let openid_return_to_url = Self::get_open_id_return_to_url(req);
                    *consumer = Some(OpenIdConsumer::new(openid_return_to_url));
                }
                match consumer.as_ref() {
                    Some(consumer) => {
                        consumer.auth_request(
                            user_supplied_identifier.unwrap_or(""),
                            req,
                            session_request,
                            resp,
                        )?;
                    }
                    None => return Err(anyhow!("NullPointerException: openidConsumer")),
                }
                return Ok(());
            }

            let lang = match request_info.language.as_deref() {
                Some(lang) => lang.to_string(),
                None => "en".to_string(),
            };

            let mut config = self
                .load_activities_and_processors(session_request, request_info.topic.as_deref())?;

            // check if the requested topic exists
            let config = match config.as_deref_mut() {
                Some(config) => config,
                None => {
                    resp.send_error(491);
                    info!(
                        "Add-on, topic doesn't exist ({}): {}, {}, {}, {}",
                        start_time.elapsed().as_millis(),
                        lang,
                        null_str(request_info.topic.as_deref()),
                        null_str(request_info.activity.as_deref()),
                        null_str(request_info.url.as_deref())
                    );
                    return Ok(());
                }
            };

            // check if the language-topic combination exists
            if config.get_pre_desc(&lang).is_none() || config.get_post_desc(&lang).is_none() {
                resp.send_error(492);
                info!(
                    "Add-on, topic doesn't exist for language ({}): {}, {}, {}, {}",
                    start_time.elapsed().as_millis(),
                    lang,
                    null_str(request_info.topic.as_deref()),
                    null_str(request_info.activity.as_deref()),
                    null_str(request_info.url.as_deref())
                );
                return Ok(());
            }

            // set enhancement type
            config.set_client_value(
                &lang,
                "enhancement",
                request_info.activity.as_deref().unwrap_or(""),
            );

            // extract the wertiview spans from the document
            let doc = Html::parse_document(request_info.document.as_deref().ok_or_else(|| {
                anyhow!("IllegalArgumentException: String input must not be null")
            })?);
            let html_string = self.spans_to_e_tags(&doc, "wertiview", true)?;

            let result: String;

            // handling each type of request
            if full_match(
                request_info
                    .r#type
                    .as_deref()
                    .ok_or_else(|| anyhow!("NullPointerException: requestInfo.type"))?,
                "practice",
            )? {
                let ph = PracticeHandler::new(&request_info);
                result = ph.process();
            } else {
                // should be a "page" request
                let anl_dir_path = self
                    .get_servlet_context()
                    .and_then(|context| context.get_init_parameter("files_anl_dir"))
                    .map(str::to_string)
                    .ok_or_else(|| {
                        anyhow!("NullPointerException: init parameter \"files_anl_dir\"")
                    })?;
                let processors = self
                    .processors
                    .as_ref()
                    .ok_or_else(|| anyhow!("NullPointerException: processors"))?;
                let ph = PageHandler::new(
                    processors,
                    request_info
                        .topic
                        .as_deref()
                        .ok_or_else(|| anyhow!("NullPointerException: requestInfo.topic"))?,
                    &request_info
                        .url
                        .as_deref()
                        .ok_or_else(|| anyhow!("NullPointerException: requestInfo.url"))?
                        .replace("/", "-"),
                    &anl_dir_path,
                    &html_string,
                    &lang,
                );
                let cas = ph.process()?;

                let cas = cas.ok_or_else(|| anyhow!("NullPointerException: cas"))?;
                let pe = JsonEnhancer::new(
                    &cas,
                    request_info
                        .activity
                        .as_deref()
                        .ok_or_else(|| anyhow!("NullPointerException: requestInfo.activity"))?,
                );
                result = pe.enhance()?;

                info!(
                    "Add-on ({}): {}, {}, {}, {}, {}",
                    start_time.elapsed().as_millis(),
                    null_str(request_info.language.as_deref()),
                    null_str(request_info.topic.as_deref()),
                    null_str(request_info.activity.as_deref()),
                    null_str(request_info.url.as_deref()),
                    cas.language
                );
            }

            // to write to the response stream
            let written = (|| -> Result<()> {
                resp.set_content_type("text/plain");
                let mut out = resp.get_writer()?;
                out.write(&result);
                out.close();
                Ok(())
            })();
            if let Err(ioe) = written {
                error!("Error writing to response stream");
                return Err(servlet_exception_cause("", ioe));
            }

            Ok(())
        }
    }

    /// load the activity.xml files for all topics. Initialize the pre- and
    /// postprocessors for each.
    ///
    /// The registry is handed out by the session loader as a live borrow, so
    /// the processor load runs before the topic lookup rather than after it;
    /// the configuration object the lookup returns is the same one the load
    /// mutates either way.
    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-activities-and-processors-fn]
    fn load_activities_and_processors<'a>(
        &mut self,
        session_request: &'a mut SessionRequest,
        topic_name: Option<&str>,
    ) -> Result<Option<&'a mut ActivityConfiguration>> {
        // load activities from/into session
        let acts = ActivitiesSessionLoader::create_activities_in_session(session_request)?;
        let topic_name = topic_name
            .ok_or_else(|| anyhow!("NullPointerException: topic name is not comparable"))?;

        // load processors if necessary
        self.load_processors(acts)?;

        let config = acts.get_activity(topic_name);

        Ok(config)
    }

    /// replace all `<span class="wertiview">` tags with `<e>` tags. Copy the
    /// wertiview IDs if there are any. Turn the HTML character entities inside
    /// the spans/e-tags into unicode characters.
    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.spans-to-e-tags-fn]
    fn spans_to_e_tags(&self, doc: &Html, class_name: &str, have_ids: bool) -> Result<String> {
        // find all added spans using the class name and replace everything
        // inside the <e> tokens with unescaped unicode characters
        let subject = doc.html();
        let mut html_string = subject.clone();
        let pattern = format!(
            "<span class=\"[^\"]*{}[^\"]*\"( wertiviewid=\"([^\"]*)\")?>(.*?)</span>",
            class_name
        );
        let enhance_patt = Regex::new(&format!("(?s){}", pattern))?;
        // The rewrite pass below re-compiles the pattern from its source text,
        // which carries no flags, so `.` there stops at a line break while the
        // match loop's `.` does not: a span holding a newline is unescaped but
        // never turned into an `<e>` tag.
        let enhance_patt_unflagged = Regex::new(&pattern)?;
        // The matcher runs over the string as it was before the loop began,
        // so the replacements below do not shift what it sees.
        for captures in enhance_patt.captures_iter(&subject) {
            if let Some(group3) = captures.get(3) {
                html_string = html_string.replace(
                    group3.as_str(),
                    &html_escape::decode_html_entities(group3.as_str()),
                );
            }
        }

        // replace these spans with <e> tags for use in normal pipeline
        if have_ids {
            html_string = enhance_patt_unflagged
                .replace_all(&html_string, "<e id=\"${2}\">${3}</e>")
                .into_owned();
        } else {
            html_string = enhance_patt_unflagged
                .replace_all(&html_string, "<e>${3}</e>")
                .into_owned();
        }

        Ok(html_string)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.merge-config-params-fn]
    fn merge_config_params(
        &self,
        mut config: Option<&mut ActivityConfiguration>,
        req: &HttpServletRequest,
    ) -> Result<()> {
        let param_names: Vec<String> = req.get_parameter_names().cloned().collect();
        // No "en" fallback: a request without a `language` parameter keys every
        // lookup on a language that cannot exist, so every set fails.
        let lang = req.get_parameter("language").unwrap_or("").to_string();
        for key in param_names {
            let worked;
            let mut is_config_param = false;
            let value = req.get_parameter(&key).unwrap_or("").to_string();

            if key.starts_with(CLIENT_PREFIX) {
                let config = config
                    .as_deref_mut()
                    .ok_or_else(|| anyhow!("NullPointerException: config"))?;
                worked = config.set_client_value(
                    &lang,
                    key.get(CLIENT_PREFIX.len() + 1..)
                        .ok_or_else(|| anyhow!("StringIndexOutOfBoundsException: {}", key))?,
                    &value,
                );
                is_config_param = true;
            } else if key.starts_with(PRE_PREFIX) {
                let config = config
                    .as_deref_mut()
                    .ok_or_else(|| anyhow!("NullPointerException: config"))?;
                worked = config.set_server_pre_value(
                    &lang,
                    key.get(PRE_PREFIX.len() + 1..)
                        .ok_or_else(|| anyhow!("StringIndexOutOfBoundsException: {}", key))?,
                    &value,
                );
                is_config_param = true;
            } else if key.starts_with(POST_PREFIX) {
                let config = config
                    .as_deref_mut()
                    .ok_or_else(|| anyhow!("NullPointerException: config"))?;
                worked = config.set_server_post_value(
                    &lang,
                    key.get(POST_PREFIX.len() + 1..)
                        .ok_or_else(|| anyhow!("StringIndexOutOfBoundsException: {}", key))?,
                    &value,
                );
                is_config_param = true;
            } else {
                worked = false;
            }

            if is_config_param {
                if worked {
                    debug!("Successfully set config param: {} to: {}", key, value);
                } else {
                    debug!("Access denied for config param: {}", key);
                }
            }
        }

        Ok(())
    }

    /// If the processors haven't been loaded, load them.
    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.load-processors-fn]
    fn load_processors(&mut self, acts: &mut Activities) -> Result<()> {
        if self.processors.is_none() {
            let start_time = Instant::now();
            self.processors = Some(Processors::new(acts)?);
            info!(
                "Loaded all UIMA processors ({})",
                start_time.elapsed().as_millis()
            );
        }

        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-servlet-base-url-fn]
    fn get_servlet_base_url(req: &HttpServletRequest) -> String {
        let base_url = format!(
            "{}://{}:{}{}",
            req.get_scheme(),
            req.get_server_name(),
            req.get_server_port(),
            req.get_context_path()
        );
        base_url
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.get-open-id-return-to-url-fn]
    fn get_open_id_return_to_url(req: &HttpServletRequest) -> String {
        let base_url = Self::get_servlet_base_url(req);
        let openid_return_to_url = format!("{}/VIEW?openid_return=true", base_url);
        openid_return_to_url
    }
}

/// `String#matches` anchors the pattern at both ends of the subject.
fn full_match(subject: &str, pattern: &str) -> Result<bool> {
    let regex = Regex::new(&format!("^(?:{})$", pattern))?;
    Ok(regex.is_match(subject))
}

/// The openid4java consumer the servlet drives. Discovery, association and
/// response verification are all provider round-trips with no counterpart on
/// this platform, so the two entry points report the missing capability rather
/// than silently answering "not verified" — which would send every caller to
/// the verification-failed page.
///
/// The original also holds a back-reference to the servlet, used only to reach
/// a JSP request dispatcher for the >2048-byte form-redirect path.
pub struct OpenIdConsumer {
    pub return_to_url: String,
}

impl OpenIdConsumer {
    pub fn new(return_to_url: String) -> Self {
        OpenIdConsumer { return_to_url }
    }

    /// placing the authentication request
    pub fn auth_request(
        &self,
        _user_supplied_identifier: &str,
        _http_req: &HttpServletRequest,
        _session_request: &mut SessionRequest,
        _http_resp: &mut HttpServletResponse,
    ) -> Result<Option<String>> {
        bail!("openid provider discovery is not available")
    }

    /// processing the authentication response
    pub fn verify_response(&self, _http_req: &HttpServletRequest) -> Result<Option<String>> {
        bail!("openid response verification is not available")
    }
}

/// Minimal stand-in for `javax.servlet.ServletContext`, carrying only what the
/// servlet reaches through it: the `context-param` entries declared in
/// `web.xml` and the deployment's display name.
#[derive(Debug, Clone, Default)]
pub struct ServletContext {
    pub init_parameters: BTreeMap<String, String>,
    pub servlet_context_name: Option<String>,
}

impl ServletContext {
    pub fn get_init_parameter(&self, name: &str) -> Option<&str> {
        self.init_parameters.get(name).map(String::as_str)
    }

    pub fn get_servlet_context_name(&self) -> Option<&str> {
        self.servlet_context_name.as_deref()
    }
}

/// Minimal stand-in for `javax.servlet.ServletConfig`: the servlet only ever
/// reaches the context through it.
#[derive(Debug, Clone, Default)]
pub struct ServletConfig {
    pub servlet_context: ServletContext,
}

impl ServletConfig {
    pub fn get_servlet_context(&self) -> &ServletContext {
        &self.servlet_context
    }
}

/// Minimal stand-in for `javax.servlet.http.HttpServletRequest`, carrying the
/// request half the servlet reads. The session half is
/// [`crate::server::activities::HttpServletRequest`], threaded alongside.
#[derive(Debug, Clone, Default)]
pub struct HttpServletRequest {
    /// Parameter order is the container's business; a sorted map gives the
    /// parameter-name walk in `merge_config_params` a stable order.
    pub parameters: BTreeMap<String, String>,
    pub body: String,
    pub scheme: String,
    pub server_name: String,
    pub server_port: u16,
    pub context_path: String,
    pub request_url: String,
    pub query_string: Option<String>,
    pub character_encoding: Option<String>,
}

impl HttpServletRequest {
    pub fn get_parameter(&self, name: &str) -> Option<&str> {
        self.parameters.get(name).map(String::as_str)
    }

    pub fn get_parameter_names(&self) -> impl Iterator<Item = &String> {
        self.parameters.keys()
    }

    pub fn get_reader(&self) -> &str {
        &self.body
    }

    pub fn set_character_encoding(&mut self, encoding: &str) {
        self.character_encoding = Some(encoding.to_string());
    }

    pub fn get_scheme(&self) -> &str {
        &self.scheme
    }

    pub fn get_server_name(&self) -> &str {
        &self.server_name
    }

    pub fn get_server_port(&self) -> u16 {
        self.server_port
    }

    pub fn get_context_path(&self) -> &str {
        &self.context_path
    }

    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }

    pub fn get_query_string(&self) -> Option<&str> {
        self.query_string.as_deref()
    }
}

/// Minimal stand-in for `javax.servlet.http.HttpServletResponse`. The body is
/// accumulated in memory and converted to a poem response once the handler
/// returns, so a handler that fails part-way discards what it had written —
/// which is what a container does for a `ServletException` raised before the
/// response is committed.
#[derive(Debug, Clone)]
pub struct HttpServletResponse {
    pub status: u16,
    pub character_encoding: Option<String>,
    pub content_type: Option<String>,
    pub redirect: Option<String>,
    pub body: String,
    pub writer_closed: bool,
}

impl Default for HttpServletResponse {
    fn default() -> Self {
        HttpServletResponse {
            status: 200,
            character_encoding: None,
            content_type: None,
            redirect: None,
            body: String::new(),
            writer_closed: false,
        }
    }
}

impl HttpServletResponse {
    pub fn set_character_encoding(&mut self, encoding: &str) {
        self.character_encoding = Some(encoding.to_string());
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        self.content_type = Some(content_type.to_string());
    }

    pub fn send_redirect(&mut self, location: &str) {
        self.status = 302;
        self.redirect = Some(location.to_string());
    }

    pub fn send_error(&mut self, status: u16) {
        self.status = status;
    }

    pub fn get_writer(&mut self) -> Result<PrintWriter<'_>> {
        Ok(PrintWriter {
            out: &mut self.body,
            closed: &mut self.writer_closed,
        })
    }
}

impl IntoResponse for HttpServletResponse {
    fn into_response(self) -> Response {
        let status = match StatusCode::from_u16(self.status) {
            Ok(status) => status,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let mut builder = Response::builder().status(status);

        if let Some(location) = self.redirect.as_deref() {
            builder = builder.header(LOCATION, location);
        }

        // `setContentType` and `setCharacterEncoding` are separate calls that
        // land in one header; a response that sets only the former carries no
        // charset at all.
        if let Some(content_type) = self.content_type.as_deref() {
            let header = match self.character_encoding.as_deref() {
                Some(encoding) => format!("{};charset={}", content_type, encoding),
                None => content_type.to_string(),
            };
            builder = builder.header(CONTENT_TYPE, header);
        }

        builder.body(Body::from(self.body))
    }
}

/// Minimal stand-in for `java.io.PrintWriter` over the response body.
pub struct PrintWriter<'a> {
    out: &'a mut String,
    closed: &'a mut bool,
}

impl PrintWriter<'_> {
    pub fn println(&mut self, line: &str) {
        self.out.push_str(line);
        self.out.push('\n');
    }

    pub fn write(&mut self, text: &str) {
        self.out.push_str(text);
    }

    pub fn close(&mut self) {
        *self.closed = true;
    }
}

#[cfg(test)]
#[path = "servlet_tests.rs"]
mod tests;
