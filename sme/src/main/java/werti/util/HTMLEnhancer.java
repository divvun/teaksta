package werti.util;

import javax.servlet.http.HttpServletRequest;
import java.util.HashMap;
import java.io.UnsupportedEncodingException;

import org.apache.uima.jcas.JCas;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;
import org.jsoup.nodes.Element;
import werti.server.ActivityConfiguration;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;

/**
 * Produces an HTML document with enhancements from a CAS containing
 * Enhancements.
 *
 * @author Adriane Boyd
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer]
public class HTMLEnhancer {
        private static final Logger log =
	    LogManager.GetLogger(HTMLEnhancer.class);

	private JCas cas;

	/**
	 * @param cCas CAS with annotations for the topic
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.html-enhancer-fn]
	public HTMLEnhancer(final JCas cCas) {
		cas = cCas;
	}

	/**
	 * Converts an HTML CAS document with Enhancements to an HTML string.
	 *
	 * @return an HTML string containing enhancements
	 */
    // [spec:teaksta:def:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.util.html-enhancer.html-enhancer.enhance-fn]
    public String enhance(final String activity, final String baseurl,
    		HttpServletRequest req, ActivityConfiguration config, String servletContextName) throws UnsupportedEncodingException {
	HashMap<String, String> dict = new HashMap<String, String>(); // translations of topics and activities to North Sámi
	dict.put("SubstantiveSingular", "Substantiivvat ovttaidlogus");
	dict.put("SubstantivePlural", "Substantiivvat máŋggaidlogus");
	dict.put("VerbConjugation", "Finihtta vearbbat");
	dict.put("NegVerbs", "Biehttalanvearbbat");
	dict.put("InfiniteVerbs", "Infinihtta vearbbat");
	dict.put("Conjunctions", "Konjunkšuvnnat");
  dict.put("Substantive", "Substantiivvat");
	dict.put("Subject", "Subjeakta");
	dict.put("Object", "Objeakta");
	dict.put("Adverbial", "Adverbiála");
	dict.put("colorize", "Geahča ivdnejuvvon sániid.");
	dict.put("click", "Coahkkal rivttes sániid!");
	dict.put("mc", "Vállje rivttes sániid!");
	dict.put("cloze", "Čále rivttes sániid!");

	String enhancement = req.getParameter("client.enhancement");
	String activityCat = activity.toLowerCase();

	// get the translations of the topic and the exercise type to sme from the small dictionary
	String activity_sme = dict.get(activity);
	String enhancement_sme = dict.get(enhancement);

	String htmlString = EnhancerUtils.casToEnhanced(cas, enhancement);

	// replace <e> tags with wertiview spans
	// (this should probably be done with a real tree traversal, but it was causing me headaches and
	// a search and replace is probably sufficient and quicker)
	htmlString = htmlString.replace("<e>", "<span class=\"wertiview\">");
	htmlString = htmlString.replace("</e>", "</span>");

	Document htmlDoc = Jsoup.parse(htmlString);

	// add base url
	Element base = htmlDoc.createElement("base");
	base.attr("href", baseurl);
	htmlDoc.head().appendChild(base);

	// Write the chosen topic and activity in North Sámi into the page title. So the user has a short reminder about the exercise.
	String topic_activity = activity_sme + ": " + enhancement_sme;
	String customised_title = new String(topic_activity.getBytes(), "UTF-8"); // encode the title string as utf8
	Element title = htmlDoc.select("title").first();
  if (title == null) {
    log.info("title null");
  } else {
    title.text(customised_title);
  }

    	// add js libraries
    	String thisUrl = req.getRequestURL().toString();
        thisUrl = thisUrl.replace("http", "https");
	thisUrl = thisUrl.replaceFirst("/WERTiServlet","");
	log.info("URL to js-lib:{}", thisUrl);
    	//thisUrl = thisUrl.replaceFirst("(?<=" + servletContextName + ").*", ""); // something went wrong with the url on gtlab, so that js libraries and .css files had wrong paths
    	if (activity.matches("Arts") || activity.matches("Dets")) {
    		activityCat = "pos";
    	}

    	final String jqueryJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/jquery-1.4.2.min.js"
    		+ "\"></script>";

    	final String wertiviewJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/wertiview.js"
    		+ "\"></script>";

    	final String blurJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/blur.js"
    		+ "\"></script>";

    	final String notificationJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/notification.js"
    		+ "\"></script>";

    	final String wertiviewCSS = "<link type=\"text/css\" rel=\"stylesheet\" href=\""
    		+ thisUrl + "/js-lib/wertiview.css"  // was: view.css
    		+ "\"></link>";

    	final String libJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/lib.js"
    		+ "\"></script>";

    	final String activityJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/activity.js"
    		+ "\"></script>";

    	final String topicJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/" + activityCat + ".js"
    		+ "\"></script>";

    	final String loadJS = "<script type=\"text/javascript\" language=\"javascript\">\n" +
    	"wertiview.jQuery(document).ready(function() { wertiview.jQuery('body').data('wertiview-topic', '" + activity + "');\n" +
    	"var topic = \"" + activityCat + "\";\n" +
    	"var activity = \"" + enhancement + "\";\n" +
		"if (!window['wertiview'][topic] || !window['wertiview'][topic][activity]) {\n" +
		"    alert(\"topic \"+topic+\" activity \"+ activity + \"The selected activity is not available for this topic.  Please choose a different activity.\");\n" +
		"} else {\n" +
    	"    wertiview." + activityCat + "." + enhancement + "();\n" +
    	"}\n" +
    	"});\n" +
    	"</script>\n";



    	htmlDoc.head().append(jqueryJS);
    	htmlDoc.head().append(wertiviewJS);
    	htmlDoc.head().append(blurJS);
    	htmlDoc.head().append(notificationJS);
    	htmlDoc.head().append(wertiviewCSS);
    	htmlDoc.head().append(libJS);
    	htmlDoc.head().append(activityJS);
    	htmlDoc.head().append(topicJS);
    	htmlDoc.head().append(loadJS);

      htmlDoc.body().prependElement("p").attr("class", "p_reminder");
      htmlDoc.body().select("p.p_reminder").append("<span class='span_reminder'>"+activity_sme+": "+enhancement_sme+"</span>");

    	htmlDoc.select("span.wertiview").select("span").attr("style", EnhancerUtils.addedSpanStyle);

       	return htmlDoc.html();
    }
}
