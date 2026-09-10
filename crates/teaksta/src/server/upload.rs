//! The upload endpoint behind the web form, plus the context listener that
//! creates the upload directories at web-application startup.
//!
//! The servlet container types this file is written against have no poem
//! counterpart with the same shape, so the minimum surface each one is used
//! through is modelled locally at the bottom of this module: the context with
//! its init parameters *and* its mutable attribute map (the listener publishes
//! into it, the servlet reads back out of it), the already-collected multipart
//! request, and the commons-fileupload pair that walks it. The response is the
//! one [`crate::server::servlet`] already models, so both servlets produce a
//! poem response the same way.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Result, anyhow};
use poem::web::Multipart;
use rand::Rng;
use rand::distr::Alphanumeric;
use scraper::{Html, Node};

use crate::server::servlet::HttpServletResponse;

pub const SERIAL_VERSION_UID: i64 = 15;

/// `new ServletException(message)` — the cause-less form.
fn servlet_exception(message: &str) -> anyhow::Error {
    anyhow!(message.to_string())
}

/// `e.getCause()` concatenated into a string: the cause's own rendering, or
/// the four characters `null` when the exception carries no cause.
fn cause_str(e: &anyhow::Error) -> String {
    match e.chain().nth(1) {
        Some(cause) => cause.to_string(),
        None => "null".to_string(),
    }
}

/// Java renders a null reference as the four characters `null` when it is
/// concatenated into a string.
fn null_str(value: Option<&str>) -> &str {
    value.unwrap_or("null")
}

/// `RandomStringUtils.randomAlphanumeric(count)`: `count` characters drawn
/// from `[A-Za-z0-9]`.
fn random_alphanumeric(count: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(count)
        .map(char::from)
        .collect()
}

/// `new File(pathname)`: the constructor normalises the path string, so runs
/// of separators collapse to one and a trailing separator is dropped.
fn new_file(pathname: &str) -> PathBuf {
    let mut normalised = String::with_capacity(pathname.len());
    let mut last_was_separator = false;

    for c in pathname.chars() {
        if c == MAIN_SEPARATOR {
            if last_was_separator {
                continue;
            }
            last_was_separator = true;
        } else {
            last_was_separator = false;
        }
        normalised.push(c);
    }

    while normalised.len() > 1 && normalised.ends_with(MAIN_SEPARATOR) {
        normalised.pop();
    }

    PathBuf::from(normalised)
}

/// `File#getAbsolutePath()`: an already-absolute path is handed back as it
/// stands, a relative one is resolved against the working directory.
fn get_absolute_path(file: &Path) -> String {
    if file.is_absolute() {
        return file.to_string_lossy().into_owned();
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(file).to_string_lossy().into_owned(),
        Err(_) => file.to_string_lossy().into_owned(),
    }
}

/// `File#getName()`: the last component of the path.
fn get_name(file: &Path) -> String {
    file.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The single-argument `File#setExecutable`/`setReadable`/`setWritable` are
/// owner-only, so each one sets or clears exactly one mode bit. The boolean
/// result is discarded by every caller here.
fn set_permission(file: &Path, bit: u32, enable: bool) -> bool {
    let Ok(metadata) = std::fs::metadata(file) else {
        return false;
    };
    let mut permissions = metadata.permissions();
    let mode = permissions.mode();
    permissions.set_mode(if enable { mode | bit } else { mode & !bit });
    std::fs::set_permissions(file, permissions).is_ok()
}

const OWNER_EXECUTE: u32 = 0o100;
const OWNER_WRITE: u32 = 0o200;
const OWNER_READ: u32 = 0o400;

// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet]
#[derive(Debug, Default)]
pub struct UploadDownloadFileServlet {
    /// The container hands the context to the servlet through its config;
    /// `init` is where it arrives, and `getServletContext()` reads it back.
    servlet_context: Option<ServletContext>,
    uploader: Option<ServletFileUpload>,
}

impl UploadDownloadFileServlet {
    pub fn new() -> Self {
        UploadDownloadFileServlet::default()
    }

    /// `HttpServlet#getServletContext()`. Before `init` there is nothing to
    /// hand back, and every read of a null context raises inside the caller's
    /// try block.
    fn get_servlet_context(&self) -> Result<&ServletContext> {
        self.servlet_context
            .as_ref()
            .ok_or_else(|| anyhow!("NullPointerException: servlet context"))
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn]
    pub fn init(&mut self, servlet_context: ServletContext) -> Result<()> {
        self.servlet_context = Some(servlet_context);
        let mut file_factory = DiskFileItemFactory::new();
        let files_dir = self
            .servlet_context
            .as_ref()
            .and_then(|ctx| ctx.get_attribute("FILES_DIR_FILE"))
            .and_then(Attribute::as_file)
            .map(Path::to_path_buf);
        file_factory.set_repository(files_dir);
        self.uploader = Some(ServletFileUpload::new(file_factory));
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn]
    pub fn handle_post(
        &self,
        request: &HttpServletRequest,
        response: &mut HttpServletResponse,
    ) -> Result<()> {
        if !ServletFileUpload::is_multipart_content(request) {
            return Err(servlet_exception("Content type is not multipart/form-data"));
        }

        response.set_content_type("text/html");
        response.set_character_encoding("UTF-8");

        // The response holds its body in memory, so the writer is a buffer
        // that is handed over once the handler is done; the redirect and the
        // body are separate fields and the order they are set in does not
        // change the response.
        let mut out = String::new();
        out.push_str(
            "<html xmlns='http://www.w3.org/1999/xhtml' xml:lang='en' lang='sme'><head><meta charset='utf-8'></head><body>",
        );

        let uploaded = (|| -> Result<()> {
            let mut act = String::new();
            let mut en = String::new();
            let mut path = String::new();
            let mut captcha;
            let mut file_size_err = false;
            let mut file_type_err = false;
            let mut captcha_err = false;
            let mut verify = false;
            let mut file_uploaded = false;
            let mut save_file = false;
            let mut file_lang_err = false;
            let mut i: usize = 0;
            let mut file_index: usize = 0;
            let uploader = self
                .uploader
                .as_ref()
                .ok_or_else(|| anyhow!("NullPointerException: uploader"))?;
            let file_items_list = uploader.parse_request(request)?;
            for file_item in file_items_list.iter() {
                if file_item.is_form_field() {
                    let name = file_item.get_field_name().to_string();
                    let value = file_item.get_string();
                    println!("name= {}i= {}value= {}", name, i, value);
                    if name == "enhancement_upload" {
                        en = value;
                    } else if name == "activity_up" {
                        act = value;
                    } else if name == "g-recaptcha-response" {
                        captcha = value;
                        let secret_file = new_file(
                            self.get_servlet_context()?
                                .get_init_parameter("secret_key")
                                .ok_or_else(|| {
                                    anyhow!("NullPointerException: init parameter \"secret_key\"")
                                })?,
                        );
                        let secret_reader = BufReader::new(std::fs::File::open(&secret_file)?);
                        // `readLine` hands back null at end of input, and a
                        // null secret is concatenated into the verifier's
                        // request body as the four characters `null`.
                        let secret = match secret_reader.lines().next() {
                            Some(line) => line?,
                            None => "null".to_string(),
                        };
                        verify = crate::util::verify_recaptcha::verify(&captcha, &secret)?;
                    } else if name == "save_options" {
                        if value == "save" {
                            save_file = true;
                        }
                    }
                } else {
                    let _name_f = file_item.get_field_name();
                    let _value_f = file_item.get_string();
                    file_index = i;
                }
                i += 1;
            }
            // With no file part in the request `file_index` is still 0, so
            // this inspects whichever part happens to come first.
            let file_item = file_items_list.get(file_index).ok_or_else(|| {
                anyhow!(
                    "IndexOutOfBoundsException: Index: {}, Size: {}",
                    file_index,
                    file_items_list.len()
                )
            })?;
            if !file_item.get_string().is_empty() {
                file_uploaded = true;
            }
            if file_uploaded {
                if verify {
                    if file_item.get_size() < 5 * (1024 * 1024) {
                        let _file_input_stream = file_item.get_input_stream()?;
                        let rnd_name = random_alphanumeric(10);
                        let file: PathBuf;
                        if save_file {
                            file = new_file(&format!(
                                "{}{}{}",
                                attribute_string(self.get_servlet_context()?, "FILES_PRM_DIR"),
                                MAIN_SEPARATOR,
                                rnd_name
                            ));
                            path = get_absolute_path(&file);
                        } else {
                            file = new_file(&format!(
                                "{}{}{}",
                                attribute_string(self.get_servlet_context()?, "FILES_TMP_DIR"),
                                MAIN_SEPARATOR,
                                rnd_name
                            ));
                            path = get_absolute_path(&file);
                        }
                        file_item.write(&file)?;
                        set_permission(&file, OWNER_EXECUTE, false);
                        set_permission(&file, OWNER_READ, true);
                        set_permission(&file, OWNER_WRITE, false);

                        let text_html = check_meta_data(&file, "text/html")?;
                        let text_xhtml = check_meta_data(&file, "application/xhtml+xml")?;

                        if text_html || text_xhtml {
                            println!("file is html");
                            let doc = jsoup_parse(&file)?;
                            let text_content = document_text(&doc);

                            let file_path = "/tmp/inputLang.txt";
                            let written = (|| -> std::io::Result<BufWriter<std::fs::File>> {
                                let mut writer = BufWriter::new(std::fs::File::create(file_path)?);
                                writer.write_all(text_content.as_bytes())?;
                                Ok(writer)
                            })();
                            let writer = match written {
                                Ok(writer) => Some(writer),
                                Err(_ex) => {
                                    println!("file not written");
                                    None
                                }
                            };
                            // finally: close, swallowing whatever that raises
                            if let Some(mut writer) = writer {
                                let _ = writer.flush();
                            }

                            let _check_lang_res = String::new();
                            // pytextcat needs file -- filePath
                            let shell_command = format!(" pytextcat proc <{}", file_path);
                            let text_check_lang = ["/bin/sh", "-c", shell_command.as_str()];
                            // The process's standard error is piped and never
                            // drained, and its exit status is never read.
                            let mut process = Command::new(text_check_lang[0])
                                .args(&text_check_lang[1..])
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()?;
                            let std_input = BufReader::new(
                                process
                                    .stdout
                                    .take()
                                    .ok_or_else(|| anyhow!("NullPointerException: stdout"))?,
                            );
                            let mut stdout = String::new();

                            // read the output from the command
                            for s in std_input.lines() {
                                stdout.push_str(&s?);
                            }

                            if stdout == "sme" {
                                println!("file is sme");
                            } else {
                                if file.exists() {
                                    let _ = std::fs::remove_file(&file);
                                }
                                println!("file not sme, deleted");
                                path = String::new();
                                file_lang_err = true;
                            }
                        } else {
                            let _ = std::fs::remove_file(&file);
                            path = String::new();
                            file_type_err = true;
                        }
                    } else {
                        path = String::new();
                        file_size_err = true;
                    }
                } else {
                    path = String::new();
                    captcha_err = true;
                }
            } else {
                path = String::new();
            }

            // http://gtoahpa-01.uit.no , http://oahpa.no , http://127.0.0.1:8080
            let host_name = "http://gtoahpa-01.uit.no/konteaksta";

            if path != "" {
                // none of the three interpolated values is URL-encoded
                response.send_redirect(&format!(
                    "{}/WERTiServlet?activity={}&client.enhancement={}&url=file://{}",
                    host_name, act, en, path
                ));
            } else {
                if !file_uploaded {
                    out.push_str("<br><br><br>");
                    out.push_str(
                        "<center><h1 style='color:#144ea6;'>Vajálduhttet sáddet fiilla!</h1></center>",
                    );
                    out.push_str(&format!(
                        "<center><a href={}>Ruovttoluotta</a></center>",
                        host_name
                    ));
                }
                if file_size_err {
                    out.push_str("<br><br><br>");
                    out.push_str(
                        "<center><h1 style='color:#144ea6;'>Fiila lea menddo stuoris! Lobálaš sturrodat: 5MB.</h1></center>",
                    );
                    out.push_str(&format!(
                        "<center><a href={}>Ruovttoluotta</a></center>",
                        host_name
                    ));
                }
                if file_type_err {
                    out.push_str("<br><br><br>");
                    out.push_str(
                        "<center><h1 style='color:#144ea6;'>Fiilla formáhta ii leat html! Lobálaš formáhta: html.</h1></center>",
                    );
                    out.push_str(&format!(
                        "<center><a href={}>Ruovttoluotta</a></center>",
                        host_name
                    ));
                }
                if captcha_err {
                    out.push_str("<br><br><br>");
                    out.push_str(
                        "<center><h1 style='color:#144ea6;'>Vajálduhttet captcha!</h1></center>",
                    );
                    out.push_str(&format!(
                        "<center><a href={}>Ruovttoluotta</a></center>",
                        host_name
                    ));
                }
                if file_lang_err {
                    out.push_str("<br><br><br>");
                    out.push_str(
                        "<center><h1 style='color:#144ea6;'>Fiila ii sisttisdoala davvisámegiela!</h1></center>",
                    );
                    out.push_str(&format!(
                        "<center><a href={}>Ruovttoluotta</a></center>",
                        host_name
                    ));
                }
            }

            Ok(())
        })();

        // The upload failure and the catch-all produce the same output.
        if let Err(e) = uploaded {
            out.push_str(&format!(
                "Exception in uploading file. Cause: {}",
                cause_str(&e)
            ));
        }
        out.push_str("</body></html>");
        response.get_writer()?.write(&out);
        Ok(())
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn]
pub fn check_meta_data(f: &Path, get_content_type: &str) -> Result<bool> {
    // The detector is handed the whole file rather than a stream; a read
    // failure is the `IOException` the outer handler answers with false.
    let is = match std::fs::read(f) {
        Ok(is) => is,
        Err(_e) => {
            // Handle error
            return Ok(false);
        }
    };
    // The file name is a detection hint, under `resourceName`.
    let resource_name = get_name(f);
    let content_type = detect_content_type(&is, &resource_name);
    println!("content_type={}", null_str(content_type.as_deref()));

    match content_type {
        // A substring test, not equality: `text/html` matches a detected
        // `text/html; charset=UTF-8`.
        Some(content_type) => Ok(content_type.contains(get_content_type)),
        // Nothing detected leaves the content type null and the membership
        // test raises past the three handled failures.
        None => Err(anyhow!(
            "NullPointerException: metadata.get(Metadata.CONTENT_TYPE)"
        )),
    }
}

/// Media-type detection over the stored bytes plus the file-name hint, in
/// place of Tika's `AutoDetectParser` and the `Metadata` it fills in. Only
/// the two types the caller tests for are distinguished with any care; the
/// uploaded file is stored under a random name with no extension, so the
/// name hint carries nothing in practice and the verdict rests on the bytes.
fn detect_content_type(content: &[u8], resource_name: &str) -> Option<String> {
    if content.is_empty() {
        return None;
    }

    let head_len = content.len().min(8192);
    let head = String::from_utf8_lossy(&content[..head_len]).to_lowercase();
    let name = resource_name.to_lowercase();

    if head.contains("<?xml") && head.contains("xhtml") {
        return Some("application/xhtml+xml".to_string());
    }
    if head.contains("<!doctype html")
        || head.contains("<html")
        || head.contains("<head")
        || head.contains("<body")
    {
        return Some("text/html; charset=UTF-8".to_string());
    }
    if name.ends_with(".xhtml") {
        return Some("application/xhtml+xml".to_string());
    }
    if name.ends_with(".html") || name.ends_with(".htm") {
        return Some("text/html".to_string());
    }
    if head.starts_with("<?xml") {
        return Some("application/xml".to_string());
    }
    if std::str::from_utf8(content).is_ok() {
        return Some("text/plain; charset=UTF-8".to_string());
    }

    Some("application/octet-stream".to_string())
}

/// `Jsoup.parse(file, "UTF-8")`: the file is read as UTF-8, with malformed
/// input replaced rather than rejected, and parsed as a whole document.
fn jsoup_parse(file: &Path) -> Result<Html> {
    let bytes = std::fs::read(file)?;
    Ok(Html::parse_document(&String::from_utf8_lossy(&bytes)))
}

/// jsoup's `Document#text()`: the normalised text of every text node in
/// document order, with a space inserted at each block boundary and the
/// result trimmed. Script and style bodies are data rather than text and are
/// left out; the content of a whitespace-preserving element is taken raw.
fn document_text(doc: &Html) -> String {
    let mut accum = String::new();

    for node in doc.tree.root().descendants() {
        match node.value() {
            Node::Text(text) => {
                let whole_text: &str = text;
                let parent_name = node
                    .parent()
                    .and_then(|parent| parent.value().as_element().map(|e| e.name().to_string()))
                    .unwrap_or_default();
                if parent_name == "script" || parent_name == "style" {
                    continue;
                }
                if parent_name == "pre" || parent_name == "textarea" {
                    accum.push_str(whole_text);
                } else {
                    let strip_leading = last_char_is_whitespace(&accum);
                    append_normalised_whitespace(&mut accum, whole_text, strip_leading);
                }
            }
            Node::Element(element) => {
                if !accum.is_empty()
                    && (is_block(element.name()) || element.name() == "br")
                    && !last_char_is_whitespace(&accum)
                {
                    accum.push(' ');
                }
            }
            _ => {}
        }
    }

    accum.trim().to_string()
}

/// jsoup's `StringUtil.appendNormalisedWhitespace`: every run of whitespace
/// collapses to a single space, and a leading run is dropped when the text
/// already ends in whitespace.
fn append_normalised_whitespace(accum: &mut String, text: &str, strip_leading: bool) {
    let mut last_was_white = false;
    let mut reached_non_white = false;

    for c in text.chars() {
        if is_whitespace(c) {
            if (strip_leading && !reached_non_white) || last_was_white {
                continue;
            }
            accum.push(' ');
            last_was_white = true;
        } else {
            accum.push(c);
            last_was_white = false;
            reached_non_white = true;
        }
    }
}

fn last_char_is_whitespace(accum: &str) -> bool {
    accum.ends_with(' ')
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\u{000C}' || c == '\r'
}

/// jsoup's `Tag#isBlock()` over the tags it knows as block-level.
fn is_block(name: &str) -> bool {
    matches!(
        name,
        "html"
            | "head"
            | "body"
            | "frameset"
            | "script"
            | "noscript"
            | "style"
            | "meta"
            | "link"
            | "title"
            | "frame"
            | "noframes"
            | "section"
            | "nav"
            | "aside"
            | "hgroup"
            | "header"
            | "footer"
            | "p"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "pre"
            | "div"
            | "blockquote"
            | "hr"
            | "address"
            | "figure"
            | "figcaption"
            | "form"
            | "fieldset"
            | "ins"
            | "del"
            | "dl"
            | "dt"
            | "dd"
            | "li"
            | "table"
            | "caption"
            | "thead"
            | "tfoot"
            | "tbody"
            | "colgroup"
            | "col"
            | "tr"
            | "th"
            | "td"
            | "video"
            | "audio"
            | "canvas"
            | "details"
            | "menu"
            | "plaintext"
            | "template"
            | "article"
            | "main"
            | "svg"
            | "math"
            | "center"
            | "dir"
            | "applet"
            | "marquee"
            | "listing"
    )
}

// [spec:teaksta:def:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener]
#[derive(Debug, Default)]
pub struct FileLocationContextListener;

impl FileLocationContextListener {
    pub fn new() -> Self {
        FileLocationContextListener
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn]
    pub fn context_initialized(
        &self,
        servlet_context_event: &mut ServletContextEvent,
    ) -> Result<()> {
        let ctx = servlet_context_event.get_servlet_context_mut();

        let rnd_name = random_alphanumeric(10);
        println!("rnd_name={}", rnd_name);

        // Directory for files upload
        let relative_path = ctx
            .get_init_parameter("files_dir")
            .ok_or_else(|| anyhow!("NullPointerException: init parameter \"files_dir\""))?
            .to_string();
        let file = new_file(&relative_path);
        if !file.exists() {
            let _ = std::fs::create_dir_all(&file);
        }
        println!("File Directory created to be used for storing files");
        ctx.set_attribute("FILES_DIR_FILE", Attribute::File(file));
        ctx.set_attribute("FILES_DIR", Attribute::Text(relative_path));

        // Directory for files we have permission to store
        let relative_path_prm = ctx
            .get_init_parameter("files_prm_dir")
            .ok_or_else(|| anyhow!("NullPointerException: init parameter \"files_prm_dir\""))?
            .to_string();
        let file_prm = new_file(&relative_path_prm);
        if !file_prm.exists() {
            let _ = std::fs::create_dir_all(&file_prm);
        }
        ctx.set_attribute("FILES_PRM_DIR_FILE", Attribute::File(file_prm));
        ctx.set_attribute("FILES_PRM_DIR", Attribute::Text(relative_path_prm));

        // Directory for files we will delete
        let relative_path_tmp = ctx
            .get_init_parameter("files_tmp_dir")
            .ok_or_else(|| anyhow!("NullPointerException: init parameter \"files_tmp_dir\""))?
            .to_string();
        let file_tmp = new_file(&relative_path_tmp);
        if !file_tmp.exists() {
            let _ = std::fs::create_dir_all(&file_tmp);
        }
        ctx.set_attribute("FILES_TMP_DIR_FILE", Attribute::File(file_tmp));
        ctx.set_attribute("FILES_TMP_DIR", Attribute::Text(relative_path_tmp));

        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-destroyed-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-destroyed-fn]
    pub fn context_destroyed(&self, _servlet_context_event: &ServletContextEvent) {
        //do cleanup if needed
    }
}

/// One entry of the servlet context's attribute map. The listener publishes
/// each directory twice, as a `File` and as the raw path string, and the
/// servlet reads each form back differently: the `File` through a cast, the
/// string through concatenation.
#[derive(Debug, Clone)]
pub enum Attribute {
    File(PathBuf),
    Text(String),
}

impl Attribute {
    /// What Java string concatenation of the attribute yields.
    pub fn to_java_string(&self) -> String {
        match self {
            Attribute::File(file) => file.to_string_lossy().into_owned(),
            Attribute::Text(text) => text.clone(),
        }
    }

    /// `(File) attribute`.
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            Attribute::File(file) => Some(file.as_path()),
            Attribute::Text(_) => None,
        }
    }
}

/// A missing attribute is null, and concatenating null yields `null`.
fn attribute_string(context: &ServletContext, name: &str) -> String {
    match context.get_attribute(name) {
        Some(attribute) => attribute.to_java_string(),
        None => "null".to_string(),
    }
}

/// Stand-in for `javax.servlet.ServletContext` as this module uses it: the
/// `context-param` entries declared in `web.xml`, plus the attribute map the
/// listener writes and the servlet reads.
#[derive(Debug, Clone, Default)]
pub struct ServletContext {
    pub init_parameters: BTreeMap<String, String>,
    pub attributes: BTreeMap<String, Attribute>,
}

impl ServletContext {
    pub fn get_init_parameter(&self, name: &str) -> Option<&str> {
        self.init_parameters.get(name).map(String::as_str)
    }

    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.get(name)
    }

    pub fn set_attribute(&mut self, name: &str, value: Attribute) {
        self.attributes.insert(name.to_string(), value);
    }
}

/// Stand-in for `javax.servlet.ServletContextEvent`: the listener only
/// reaches the context through it.
#[derive(Debug, Clone, Default)]
pub struct ServletContextEvent {
    pub servlet_context: ServletContext,
}

impl ServletContextEvent {
    pub fn get_servlet_context(&self) -> &ServletContext {
        &self.servlet_context
    }

    pub fn get_servlet_context_mut(&mut self) -> &mut ServletContext {
        &mut self.servlet_context
    }
}

/// Stand-in for `javax.servlet.http.HttpServletRequest` as the upload path
/// reads it: the request method, the content type, and the multipart body
/// already split into parts by the container.
#[derive(Debug, Clone, Default)]
pub struct HttpServletRequest {
    pub method: String,
    pub content_type: Option<String>,
    pub items: Vec<FileItem>,
}

impl HttpServletRequest {
    pub fn get_method(&self) -> &str {
        &self.method
    }

    pub fn get_content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}

/// Collects a poem multipart body into the parsed-part view the request
/// carries. The container does this before the servlet method is entered, so
/// the servlet body itself stays synchronous.
pub async fn collect_multipart(
    content_type: Option<String>,
    mut multipart: Multipart,
) -> Result<HttpServletRequest> {
    let mut items = Vec::new();

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().unwrap_or_default().to_string();
        let file_name = field.file_name().map(str::to_string);
        let part_content_type = field.content_type().map(str::to_string);
        let content = field.bytes().await?;
        items.push(FileItem {
            field_name,
            file_name,
            content_type: part_content_type,
            content,
        });
    }

    Ok(HttpServletRequest {
        method: "POST".to_string(),
        content_type,
        items,
    })
}

/// One part of the multipart body, in the shape
/// `org.apache.commons.fileupload.FileItem` is used in.
#[derive(Debug, Clone, Default)]
pub struct FileItem {
    pub field_name: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub content: Vec<u8>,
}

impl FileItem {
    /// A part that carries no filename is a plain form field.
    pub fn is_form_field(&self) -> bool {
        self.file_name.is_none()
    }

    pub fn get_field_name(&self) -> &str {
        &self.field_name
    }

    /// `getString()` with no charset decodes the part's bytes with the
    /// platform default charset, materialising the whole part as a string.
    pub fn get_string(&self) -> String {
        String::from_utf8_lossy(&self.content).into_owned()
    }

    pub fn get_size(&self) -> u64 {
        self.content.len() as u64
    }

    pub fn get_input_stream(&self) -> std::io::Result<&[u8]> {
        Ok(&self.content)
    }

    pub fn write(&self, file: &Path) -> std::io::Result<()> {
        std::fs::write(file, &self.content)
    }
}

/// Stand-in for `org.apache.commons.fileupload.disk.DiskFileItemFactory`.
#[derive(Debug, Clone, Default)]
pub struct DiskFileItemFactory {
    /// The spill-over directory for parts too large to hold in memory. A null
    /// repository leaves the system temporary directory in place.
    pub repository: Option<PathBuf>,
}

impl DiskFileItemFactory {
    pub fn new() -> Self {
        DiskFileItemFactory::default()
    }

    pub fn set_repository(&mut self, repository: Option<PathBuf>) {
        self.repository = repository;
    }
}

/// Stand-in for `org.apache.commons.fileupload.servlet.ServletFileUpload`.
/// No size limit, no file-count limit and no header encoding is configured on
/// it; the 5 MB cap is applied by hand on each incoming upload.
#[derive(Debug, Clone, Default)]
pub struct ServletFileUpload {
    pub file_item_factory: DiskFileItemFactory,
}

impl ServletFileUpload {
    pub fn new(file_item_factory: DiskFileItemFactory) -> Self {
        ServletFileUpload { file_item_factory }
    }

    pub fn is_multipart_content(request: &HttpServletRequest) -> bool {
        if !request.get_method().eq_ignore_ascii_case("POST") {
            return false;
        }
        match request.get_content_type() {
            Some(content_type) => content_type.to_lowercase().starts_with("multipart/"),
            None => false,
        }
    }

    pub fn parse_request(&self, request: &HttpServletRequest) -> Result<Vec<FileItem>> {
        if !ServletFileUpload::is_multipart_content(request) {
            return Err(anyhow!(
                "InvalidContentTypeException: the request doesn't contain a multipart/form-data or multipart/mixed stream, content type header is {}",
                null_str(request.get_content_type())
            ));
        }
        Ok(request.items.clone())
    }
}

#[cfg(test)]
#[path = "upload_tests.rs"]
mod tests;
