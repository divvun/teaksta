package werti.server;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.net.URL;
import java.util.Arrays;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.servlet.ServletConfig;
import javax.servlet.ServletException;
import javax.servlet.http.HttpServlet;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;

import org.apache.commons.lang.StringEscapeUtils;
import org.apache.log4j.Logger;
import org.apache.uima.jcas.JCas;
import org.apache.uima.util.JCasPool;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;

import werti.WERTiContext;
import werti.WERTiContext.WERTiContextException;
import werti.util.ActivitiesSessionLoader;
import werti.util.CasUtils;
import werti.util.DummyException;
import werti.util.HTMLEnhancer;
import werti.util.HTMLUtils;
import werti.util.JSONEnhancer;
import werti.util.PageHandler;
import werti.util.Pair;
import werti.util.PostRequest;
import werti.util.PracticeHandler;

import com.google.gson.Gson;

/**
 * The server side implementation of the WERTi service.
 *
 * This is where the work is coordinated. The rough outline of the procedure is as follows:
 *
 * <ul>
 * <li>We take a request via the doGet() (web form) or doPost() (add-on) methods.</li>
 * <li>For web form requests, the HTML source of the URL is fetched and spans of text are
 * identified.</li>
 * <li>The document is processed in UIMA, invoking the processors for the current
 * topic.</li>
 * <li>Afterwards, we take the resulting CAS and insert enhancement annotations
 * (<tt>WERTi</tt>-<tt>&lt;span&gt;</tt>s) according to the target annotations from the
 * postprocessor.</li>
 *
 * @author Aleksandar Dimitrov
 * @author Adriane Boyd
 */
public class WERTiServlet extends HttpServlet {
	private static final Logger log =
		Logger.getLogger(WERTiServlet.class);

	public static WERTiContext context;
	
	// maximum amount of of ms to wait for a web-page to load
	private static final int MAX_WAIT = 1000 * 10; // 10 seconds

	public static final long serialVersionUID = 10;
	
	public static final Set<String> supportedVersions = new HashSet<String>(Arrays.asList("0.10", "0.11", "0.12", "0.12.1", "0.12.2"));

	private Processors processors;
	
	public static String enhancement_type; // colorize, click, mc or practice

	// map from enhancement IDs to pairs of an IP address and a JCas
	// (using Longs, we have IDs for 
	// 25,269,512 million years at 1,000 enhancements a day, though 
	// JavaScript's number type only allows IDs for 24 million million years)
	private HashMap<Long,Pair<String,JCas>> enhIdJcasMap;
	// next enhancement ID (is currently reset every time the server is 
	// restarted, but that will change in the user-accounts branch)
	private long nextEnhId;

	@Override
	public void init(ServletConfig config) throws ServletException {
		super.init(config);
		log.warn("Initializing servlet.");
		// initialise servletcontext
		try {
			WERTiContext.init(config);
		} catch (WERTiContextException wce) {
			log.fatal("Context failed to initialize.");
			log.fatal(wce);
		}
		enhIdJcasMap = new HashMap<Long,Pair<String,JCas>>();
		nextEnhId = 0;
		enhancement_type = "";
	}

	@Override
	public void destroy() {
		// no-op
	}

	/* (non-Javadoc)
     * @see javax.servlet.http.HttpServlet#doGet(javax.servlet.http.HttpServletRequest, javax.servlet.http.HttpServletResponse)
     */
	@Override
	protected void doGet(HttpServletRequest req, HttpServletResponse resp)
	throws ServletException, IOException {
		
		req.setCharacterEncoding("UTF-8");
		resp.setCharacterEncoding("UTF-8");

		long startTime = System.currentTimeMillis();
		String clientIpAddress = req.getRemoteAddr();
		
		String url = req.getParameter("url");
		// accept url-s without http://
		if (!url.startsWith("http://")) {
			url = "http://" + url;
		}
		
		String lang = req.getParameter("language");
		String topic = req.getParameter("activity");
		String activity = req.getParameter("client.enhancement");	
		enhancement_type = activity;
		//log.info("enhancement type="+enhancement_type); // testing		
		log.info(getLogMsgForWebapp("received request", clientIpAddress, startTime, lang, topic, activity, url));
		
		if (lang == null) {
			lang = "en";
		}

		ActivityConfiguration config = loadActivitiesAndProcessors(req, topic);

		// merge config with request parameters
		mergeConfigParams(config, req);

		// check if language/topic combination is available
		if (!processors.hasProcessor(lang, topic)) {
			throw new ServletException("The selected language/topic/activity combination is not currently available.");
		}
		
		URL u = new URL(url);
		Document htmlDoc;
		try {
			htmlDoc = Jsoup.parse(u, MAX_WAIT);
		} catch (IOException ioe) {
			throw new ServletException("Webpage retrieval failed.");
		}

		HTMLUtils.markTextNodes(htmlDoc, htmlDoc.body());

		// TODO: potentially modify jsoup to return unescaped text so that this hack 
		//       can be removed
		String htmlString = spansToETags(htmlDoc.html(), HTMLUtils.className, false);

		PageHandler ph = new PageHandler(processors, topic, htmlString, lang);
		JCas cas = null;
		HTMLEnhancer ge = null;
		String result = "";
		try{
			log.debug("Requesting a CAS from the pool");
			cas = checkoutJCas(topic, lang);
			// if no JCas becomes available within the timeout, cas is null
			if (cas == null) {
				log.info(getLogMsgForWebapp("time out when checking out a CAS", clientIpAddress, startTime, lang, topic, activity, url));
				throw new ServletException("The server is too busy right now. Please try again in a few minutes.");
			}
			log.debug("Received a CAS from the pool");

			cas = ph.process(cas);

			ge = new HTMLEnhancer(cas);
			result = ge.enhance(topic, u.toString(), req, config, getServletContext().getServletContextName());
		}
		finally {
			// remove all references to objects that have references to the cas
			ge = null;
			
			// release the cas
			if (cas != null) {
				JCasPool jCasPool = WERTiContext.jCasPoolMap.get(topic).get(lang);
				jCasPool.releaseJCas(cas);
			}
			// INVARIANT: if cas is null, no JCas was checked out from the pool
		}
		
		log.info(getLogMsgForWebapp("finished enhancement", clientIpAddress, startTime, lang, topic, activity, url));

		try { // to write to the response stream
			resp.setContentType("text/html");
			final PrintWriter out = resp.getWriter();
			out.write(result);
			out.close();
		} catch (IOException ioe) {
			log.error(getLogMsgForWebapp("Error writing to response stream", clientIpAddress, startTime, lang, topic, activity, url));
			throw new ServletException("", ioe);
		}
	}
	
	/**
	 * Annotate according to the topic/activity/text provided in a JSON PostRequestObject.
	 * 
	 * @param req the servlet request
	 * @param resp the servlet response
	 */
	@Override
	protected void doPost(HttpServletRequest req, HttpServletResponse resp)
			throws ServletException, IOException {
		
		req.setCharacterEncoding("UTF-8");
		resp.setCharacterEncoding("UTF-8");
		
		long startTime = System.currentTimeMillis();
		String clientIpAddress = req.getRemoteAddr();
		
		// read request in as string
		// (gson.fromJson() seems unhappy with req.getReader() as its first argument, don't know why)
		String line;
		String requestString = "";
		BufferedReader reader = req.getReader();
		while((line = reader.readLine()) != null) {
			requestString += line;
		}

		// parse this string into an object
		Gson gson = new Gson();
		PostRequest requestInfo = gson.fromJson(requestString, PostRequest.class);

		enhancement_type = requestInfo.activity;
		log.info("The enhancement type is = " + enhancement_type);
		
		
		log.info(getLogMsgForAddon("received request", requestInfo, clientIpAddress, startTime));

		if (requestInfo.language == null) {
			requestInfo.language = "en";
		}
		
		String result = "";
		
		// handling each type of request:
		// - request for an enhancement ID (no data to process yet)
		if (requestInfo.type.equals("ID")) {
			// generate a unique ID for the enhancement (will be linked with 
			// the CAS later)
			result = gson.toJson(new Long(nextEnhId));
			log.debug("assigned enhId: " + nextEnhId);
			nextEnhId += 1;
		}

		// - request to stop a running enhancement
		else if (requestInfo.type.equals("stop")) {
			// find the CAS corresponding to the enhId
			Pair<String,JCas> pair = enhIdJcasMap.get(requestInfo.enhId);
			if (pair != null) {
				String storedClientIpAddress = pair.first;
				JCas cas = pair.second;

				// allow clients to only stop their own requests
				if (cas != null) {
					if (clientIpAddress.equals(storedClientIpAddress)) {
						synchronized(cas) {
							//enhIdJcasMap.remove(requestInfo.enhId);

							if (! CasUtils.hasBeenReset(cas)) {
								// the annotators check regularly if the CAS is still valid
								CasUtils.makeInvalid(cas);
								log.debug("made CAS invalid for enhId: " + requestInfo.enhId);
							}
						}
					}
					else {
						// issue attack warning
						log.warn(getLogMsgForAddon("client " + clientIpAddress + " tried to stop enhancement " + requestInfo.enhId + " of client " + storedClientIpAddress, requestInfo, clientIpAddress, startTime));
					}
				}
			}
			
			// do not tell the client whether stopping was successful
			result = "stopped";
		}
		
		// - logging of user input to mc and practice activities
		else if (requestInfo.type.equals("practice")) {
			if (! checkAvailabilityAndLoad(requestInfo, clientIpAddress, req, resp, startTime)) {
				return;
			}
			
			PracticeHandler ph = new PracticeHandler(requestInfo);
			result = ph.process();
		}
		
		// - actual enhancement request
		else { // should be a "page" request
			if (! checkAvailabilityAndLoad(requestInfo, clientIpAddress, req, resp, startTime)) {
				return;
			}
			
			// extract the wertiview spans from the document		
			log.debug("Starting conversion of HTML to <e>");
			String htmlString = spansToETags(requestInfo.document, "wertiview", true);
			log.debug("Finished conversion of HTML to <e>");
			String topic = requestInfo.topic;
			PageHandler ph = new PageHandler(processors, requestInfo.topic, htmlString, requestInfo.language);
			JCas cas = null;
			JSONEnhancer pe = null;
			boolean casIsValid = true;
			try {
				log.debug("Requesting a CAS from the pool");
				cas = checkoutJCas(topic, requestInfo.language);
				// if no JCas becomes available within the timeout, cas is null
				if (cas == null) {
					resp.sendError(493);
					log.info(getLogMsgForAddon("time out when checking out a CAS", requestInfo, clientIpAddress, startTime));
					return;
				}
				log.debug("Received a CAS from the pool");
				
				if (requestInfo.enhId != null) {
					synchronized(cas) {
						// store the cas in a map, indexed by its enhancement ID
						enhIdJcasMap.put(requestInfo.enhId, 
								new Pair<String,JCas>(clientIpAddress, cas));

						// store the enhancement ID in the cas
						CasUtils.addEnhId(cas, requestInfo.enhId);
					}
				}

				// run the UIMA pipeline
				cas = ph.process(cas);

				// stop processing if the client has requested it
				casIsValid = CasUtils.isValid(cas);
				if (!casIsValid) {
					throw new DummyException("jump to finally block");
				}
				
				pe = new JSONEnhancer(cas, requestInfo.activity);
				result = pe.enhance();
			}
			catch (DummyException e) {
			}
			finally {
				// if the CAS is invalid, don't send the enhancement results 
				// to the client (even though we're actually finished)
				// because the client requested to stop the enhancement
				casIsValid = CasUtils.isValid(cas);

				// remove all references to objects that have references to the cas
				pe = null;
				
				// remove it from the enhId map (fails silently)
				if (requestInfo.enhId != null) {
					enhIdJcasMap.remove(requestInfo.enhId);
				}

				// release the cas
				if (cas != null) {
					JCasPool jCasPool = WERTiContext.jCasPoolMap.get(topic).get(requestInfo.language);
					jCasPool.releaseJCas(cas);
				}
				// INVARIANT: if cas is null, no JCas was checked out from the pool
			}
			
			// if the CAS isn't valid any more, stop
			if (!casIsValid) {
				resp.sendError(494);
				log.info(getLogMsgForAddon("stopped enhancement", requestInfo, clientIpAddress, startTime));
				return;
			}
			// enhancement was successful and was not stopped
			else {
				log.info(getLogMsgForAddon("finished enhancement", requestInfo, clientIpAddress, startTime));
			}
		}

		try { // to write to the response stream
			resp.setContentType("text/plain");
			final PrintWriter out = resp.getWriter();
			out.write(result);
			out.close();
		} catch (IOException ioe) {
			log.error(getLogMsgForAddon("Error writing to response stream", requestInfo, clientIpAddress, startTime));
			throw new ServletException("", ioe);
		}
	}
	
	/**
	 * Check if this language, topic, activity, and version are available/ 
	 * supported by the servlet. If they are, load the activities and 
	 * processors (if they haven't been loaded yet).
	 * @throws IOException 
	 * @throws ServletException 
	 */
	private boolean checkAvailabilityAndLoad(PostRequest requestInfo, 
			String clientIpAddress, HttpServletRequest req, 
			HttpServletResponse resp, long startTime) 
					throws IOException, ServletException {

		// check if this version is supported
		if (!supportedVersions.contains(requestInfo.version)) {
			resp.sendError(490);
			log.info(getLogMsgForAddon("version conflict", requestInfo, clientIpAddress, startTime));
			return false;
		}

		log.debug("Starting processor loading");
		ActivityConfiguration config = loadActivitiesAndProcessors(req, requestInfo.topic);
		log.debug("Finished processor loading");
		
		// check if the requested topic exists
		if (config == null) {
			resp.sendError(491);
			log.info(getLogMsgForAddon("topic doesn't exist", requestInfo, clientIpAddress, startTime));
			return false;
		}
		
		// check if the language-topic combination exists
		if (config.getDesc(requestInfo.language) == null) {
			resp.sendError(492);
			log.info(getLogMsgForAddon("topic doesn't exist for language", requestInfo, clientIpAddress, startTime));
			return false;
		}
		
		/* // disallow passives
		if (requestInfo.topic.equals("Passives")) {
			resp.sendError(491);
			return;
		}*/

		// set enhancement type
		config.setClientValue(requestInfo.language, "enhancement", requestInfo.activity);
		
		return true;
	}
	
	/**
	 * load the activity.xml files for all topics. Initialize the processor 
	 * for each.
	 * @param req the HTTP request
	 * @param topicName the topic chosen by the user
	 * @return the configuration of the activity chosen by the user
	 */
	private ActivityConfiguration loadActivitiesAndProcessors(HttpServletRequest req, 
			String topicName) throws IOException, ServletException {

		// load activities from/into session
		Activities acts = ActivitiesSessionLoader.createActivitiesInSession(req);
		ActivityConfiguration config = acts.getActivity(topicName);

		// load processors if necessary
		if (processors == null) {
			long startTime = System.currentTimeMillis();
			processors = new Processors(acts);
			log.info("Loaded all UIMA processors (" + (System.currentTimeMillis() - startTime) + ")");
		}
		
		return config;
	}
	
	/**
	 * replace all <span class="wertiview"> tags with <e> tags. Copy the 
	 * wertiview IDs if there are any. Turn the HTML character entities inside 
	 * the spans/e-tags into unicode characters.
	 * @param doc the result of a Jsoup parse
	 * @param className the name of the class of the relevant spans
	 * @param haveIds whether the wertiview spans have IDs in the wertiviewid attribute
	 * @return the <html> node as a string
	 */
	private String spansToETags(String htmlString, String className, boolean haveIds) {
		// find all added spans using the class name and replace everything inside 
		// the <e> tokens with unescaped unicode characters
		Pattern enhancePatt = Pattern.compile("<span class=\"[^\"]*" + className + "[^\"]*\"( wertiviewid=\"([^\"]*)\")?>(.*?)</span>", Pattern.UNICODE_CASE | Pattern.DOTALL);
		Matcher enhanceMatcher = enhancePatt.matcher(htmlString);
		
		int skew = 0;
		while (enhanceMatcher.find()) {
			String originalText = enhanceMatcher.group(3);
			String escapedText = StringEscapeUtils.unescapeHtml(originalText);
			htmlString = htmlString.substring(0, enhanceMatcher.start(3) + skew) + escapedText + htmlString.substring(enhanceMatcher.end(3) + skew, htmlString.length());
			skew += escapedText.length() - originalText.length();
		}
		

		// replace these spans with <e> tags for use in normal pipeline
		if (haveIds) {
			htmlString = enhanceMatcher.replaceAll("<e id=\"$2\">$3</e>");
		}
		else {
			htmlString = enhanceMatcher.replaceAll("<e>$3</e>");
		}
		
		// Convert back to unescaped unicode characters
		htmlString = StringEscapeUtils.unescapeHtml(htmlString);

		return htmlString;
	}
	
	@SuppressWarnings("unchecked")
	private void mergeConfigParams(ActivityConfiguration config, HttpServletRequest req) {
		Enumeration<String> paramNames = req.getParameterNames();
		String lang = req.getParameter("language");
		while (paramNames.hasMoreElements()) {
			String key = paramNames.nextElement();
			boolean worked = false;
			boolean isConfigParam = false;
			String value = req.getParameter(key);
			
			log.info("Config parameter key="+key+" with value="+value);//testing

			if (key.startsWith(ActivityConfiguration.CLIENT_PREFIX)) {
				worked = config.setClientValue(lang, key.substring(ActivityConfiguration.CLIENT_PREFIX.length()+1), value);
				isConfigParam = true;
			} else if (key.startsWith(ActivityConfiguration.SERVER_PREFIX)) {
				worked = config.setServerValue(lang, key.substring(ActivityConfiguration.SERVER_PREFIX.length()+1), value);
				isConfigParam = true;
			}

			if (isConfigParam) {
				if (worked) {
					log.debug("Successfully set config param: " + key + " to:" + value);
				} else {
					log.debug("Access denied for config param: " + key);
				}
			}
		}
	}
	
	private JCas checkoutJCas(String topic, String lang) {
		// checkout a JCas from the JCasPool
		JCasPool jCasPool = WERTiContext.jCasPoolMap.get(topic).get(lang);
		JCas cas = jCasPool.getJCas(WERTiContext.TIMEOUT);
		return cas;
	}

	/**
	 * helper method. Call logAddonEvent() or logWebappEvent() instead.
	 */
	private String getLogMsg(String component, String msg, 
			String clientIpAddress, long startTime, String details) {

		return component + ", " + msg + " (" + (System.currentTimeMillis() - startTime) + "):  clientIP = " + clientIpAddress + ",  " + details;
	}

	/**
	 * using this method ensures a common log message format for all add-on 
	 * events. The logging level is info.
	 * @param msg description of the event
	 */
	private String getLogMsgForAddon(String msg, 
			PostRequest requestInfo, String clientIpAddress, long startTime) {
		
		return getLogMsg("Add-on", msg, clientIpAddress, startTime, requestInfo.toShortString());
	}

	/**
	 * using this method ensures a common log message format for all webapp 
	 * events. The logging level is info.
	 * @param msg description of the event
	 */
	private String getLogMsgForWebapp(String msg, 
			String clientIpAddress, long startTime, 
			String lang, String topic, String activity, String url) {
		
		String details = "language = " + lang + ",  topic = " + topic + ",  activity = " + activity + ",  url = " + url;
		return getLogMsg("Web", msg, clientIpAddress, startTime, details);
	}
}


