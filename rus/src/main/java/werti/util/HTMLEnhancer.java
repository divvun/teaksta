package werti.util;

import javax.servlet.http.HttpServletRequest;

import org.apache.uima.jcas.JCas;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Document;
import org.jsoup.nodes.Element;
import werti.server.ActivityConfiguration;

/**
 * Produces an HTML document with enhancements from a CAS containing
 * Enhancements.
 * 
 * @author Adriane Boyd
 *
 */
public class HTMLEnhancer {
	private JCas cas;
	
	/**
	 * @param cCas CAS with annotations for the topic
	 */
	public HTMLEnhancer(final JCas cCas) {
		cas = cCas;
	}
	
	/**
	 * Converts an HTML CAS document with Enhancements to an HTML string.
	 * 
	 * @return an HTML string containing enhancements
	 */
    public String enhance(final String activity, final String baseurl, 
    		HttpServletRequest req, ActivityConfiguration config, String servletContextName) {
    	String enhancement = req.getParameter("client.enhancement");
    	String activityCat = activity.toLowerCase();
    	String htmlString = EnhancerUtils.casToEnhanced(cas, enhancement);
    	
    	// replace <e> tags with werti spans
    	// (this should probably be done with a real tree traversal, but it was causing me headaches and
    	// a search and replace is probably sufficient and quicker)
    	htmlString = htmlString.replace("<e>", "<span class=\"werti\">");
    	htmlString = htmlString.replace("</e>", "</span>");

    	Document htmlDoc = Jsoup.parse(htmlString);
    	
    	// add base url
    	Element base = htmlDoc.createElement("base");
    	base.attr("href", baseurl);
    	htmlDoc.head().appendChild(base);

    	// add js libraries
    	String thisUrl = req.getRequestURL().toString();
    	thisUrl = thisUrl.replaceFirst("(?<=" + servletContextName + ").*", "");
    	if (activity.matches("Dets")) {
    		activityCat = "pos";
    	}
    	
    	final String jqueryJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/jquery-1.4.2.min.js"
    		+ "\"></script>";
    	
    	final String werticoreJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/werti.js"
    		+ "\"></script>";
    	
    	final String blurJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/blur.js"
    		+ "\"></script>";
    	
    	final String notificationJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/notification.js"
    		+ "\"></script>";
    	
    	final String werticoreCSS = "<link type=\"text/css\" rel=\"stylesheet\" href=\""
    		+ thisUrl + "/js-lib/werticore.css"
    		+ "\"></link>";

    	final String libJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/lib.js"
    		+ "\"></script>";
    	
    	final String activityJS = "<script type=\"text/javascript\" language=\"javascript\" src=\""
    		+ thisUrl + "/js-lib/" + activityCat + ".js"
    		+ "\"></script>";
    	
    	final String loadJS = "<script type=\"text/javascript\" language=\"javascript\">\n" +
    	"werticore.jQuery(document).ready(function() { werticore.jQuery('body').data('werticore-topic', '" + activity + "');\n" +
    	"var topic = \"" + activityCat + "\";\n" +
    	"var activity = \"" + enhancement + "\";\n" +
		"if (!window['werticore'][topic] || !window['werticore'][topic][activity]) {\n" +
		"    alert(\"The selected activity is not available for this topic.  Please choose a different activity.\");\n" +
		"} else {\n" +
    	"    werticore." + activityCat + "." + enhancement + "();\n" +
    	"}\n" +
    	"});\n" +
    	"</script>\n";
    	
    	htmlDoc.head().append(jqueryJS);
    	htmlDoc.head().append(werticoreJS);
    	htmlDoc.head().append(blurJS);
    	htmlDoc.head().append(notificationJS);
    	htmlDoc.head().append(werticoreCSS);
    	htmlDoc.head().append(libJS);
    	htmlDoc.head().append(activityJS);
    	htmlDoc.head().append(loadJS);
    	
    	htmlDoc.select("span.werticore").select("span").attr("style", EnhancerUtils.addedSpanStyle);
    	
       	return htmlDoc.html();
    }
}
