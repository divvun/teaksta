package werti.util;

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;

/**
 * Methods needed for processing practice response.
 * 
 * @author Adriane Boyd
 *
 */
public class PracticeHandler {
	private static final Logger log =
		LogManager.GetLogger(PracticeHandler.class);
	
	private PostRequest requestInfo;
	
	public PracticeHandler(PostRequest aRequestInfo) {
		requestInfo = aRequestInfo;
	}
	
	public String process() {
		return "";
	}
}
