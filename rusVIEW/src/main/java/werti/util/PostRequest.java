package werti.util;

/**
 * Corresponds to the structure of the JSON AJAX requests 
 * sent by the add-on.
 * 
 * @author Adriane Boyd
 */
public class PostRequest {
	public String type;
	public Long enhId;
	public String url;
	public String language;
	public String topic;
	public String activity;
	public String document;
	public String version;
	
	@Override
	public String toString() {
		StringBuilder sb = new StringBuilder();
		sb.append("PostRequest(");
		sb.append("\n  type = ");
		sb.append(this.type);
		sb.append("\n  enhId = ");
		sb.append(this.enhId);
		sb.append("\n  url = ");
		sb.append(this.url);
		sb.append("\n  language = ");
		sb.append(this.language);
		sb.append("\n  topic = ");
		sb.append(this.topic);
		sb.append("\n  activity = ");
		sb.append(this.activity);
		sb.append("\n  document = ");
		sb.append(this.document);
		sb.append("\n  version = ");
		sb.append(this.version);
		sb.append("\n)");
		return sb.toString();
	}
	
	public String toShortString() {
		StringBuilder sb = new StringBuilder();
		sb.append("type = ");
		sb.append(this.type);
		sb.append(",  enhId = ");
		sb.append(this.enhId);
		sb.append(",  language = ");
		sb.append(this.language);
		sb.append(",  topic = ");
		sb.append(this.topic);
		sb.append(",  activity = ");
		sb.append(this.activity);
		sb.append(",  version = ");
		sb.append(this.version);
		sb.append(",  url = ");
		sb.append(this.url);
		return sb.toString();
	}
}
