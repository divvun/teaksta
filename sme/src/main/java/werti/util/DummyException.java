package werti.util;

/**
 * A DummyException is used to throw an exception in order to move to a 
 * certain point in the code, e.g., a finally block.
 * 
 * @author Marion Zepf
 */
// [spec:teaksta:def:sme.src.main.java.werti.util.dummy-exception.dummy-exception]
public class DummyException extends Exception {

	// [spec:teaksta:def:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn]
	public DummyException() {
		super();
		// Auto-generated constructor stub
	}

	public DummyException(String message) {
		super(message);
		// Auto-generated constructor stub
	}

	public DummyException(Throwable cause) {
		super(cause);
		// Auto-generated constructor stub
	}

	public DummyException(String message, Throwable cause) {
		super(message, cause);
		// Auto-generated constructor stub
	}

}
