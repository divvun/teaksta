package werti.util;

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;

/**
 * Methods needed for processing practice response.
 * 
 * @author Adriane Boyd
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler]
public class PracticeHandler {
	private static final Logger log =
		LogManager.GetLogger(PracticeHandler.class);
	
	private PostRequest requestInfo;
	
	// [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn]
	public PracticeHandler(PostRequest aRequestInfo) {
		requestInfo = aRequestInfo;
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn]
	public String process() {
		return "";
	}
}
