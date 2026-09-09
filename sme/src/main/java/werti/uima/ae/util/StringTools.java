package werti.uima.ae.util;

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;

/**
 * String normalization tools.  Currently only used in passive
 * sentence conversions.
 * 
 * @author Adriane Boyd
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools]
public class StringTools {
	private static final Logger log = LogManager.GetLogger(StringTools.class);
	
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.capitalize-first-letter-fn]
	public static String capitalizeFirstLetter(String s) {
		for (int i = 0; i < s.length(); i++) {
			String l = s.substring(i, i + 1);
			if (l.matches("[^\\p{Z}]")) {
				s = s.substring(0, i) + l.toUpperCase() + s.substring(i + 1);
				break;
			}
		}
		
		return s;		
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.uncapitalize-first-letter-fn]
	public static String uncapitalizeFirstLetter(String s) {
		for (int i = 0; i < s.length(); i++) {
			String l = s.substring(i, i + 1);
			if (l.matches("[^\\p{Z}]")) {
				s = s.substring(0, i) + l.toLowerCase() + s.substring(i + 1);
				break;
			}
		}
		
		return s;		
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.util.string-tools.string-tools.fix-punctuation-whitespace-fn]
	public static String fixPunctuationWhitespace(String s) {
		// replace space before periods
		s = s.replaceAll(" \\.", ".");
		
		// replace space before commas
		s = s.replaceAll(" ,", ",");
		
		// replace space before question marks
		s = s.replaceAll(" \\?", "?");
		
		// replace space before exclamation points
		s = s.replaceAll(" !", "!");
		
		// TODO: do something about bracketing punctuation?
		return s;
	}
}