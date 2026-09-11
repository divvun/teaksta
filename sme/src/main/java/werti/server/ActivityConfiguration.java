package werti.server;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.util.HashMap;
import java.util.Map;
import java.util.Properties;
import java.util.Set;

import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import javax.xml.xpath.XPath;
import javax.xml.xpath.XPathConstants;
import javax.xml.xpath.XPathExpressionException;
import javax.xml.xpath.XPathFactory;

import org.w3c.dom.Document;
import org.w3c.dom.NamedNodeMap;
import org.w3c.dom.Node;
import org.w3c.dom.NodeList;
import org.xml.sax.SAXException;

/**
 * <p>A class encapsulating the configuration of a WERTi activity.</p>
 * 
 * <p>This includes the names of the UIMA descriptor files for  the processing pipeline(s)
 * as well as client-side configuration (passed to the browser as JavaScript) and
 * server-side configuration (fed into the UIMA annotators as options).
 * 
 * The descriptor files as read in from the XML file are expected to be class path
 * expressions. If they can be found in the classpath, proper {@link URL}s will be returned.
 * </p>
 * 
 * <p>The setters might refuse operation on purpose since configuration entries
 * may be not overridable (aka read-only)</p>
 * 
 * @author Niels Ott
 * @version $Id: ActivityConfiguration.java 1071 2011-06-01 15:39:42Z adriane@SFS.UNI-TUEBINGEN.DE $
 */
// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration+1]
public class ActivityConfiguration {
	
	/**
	 * The configuration prefix for client settings
	 */
	public static final String CLIENT_PREFIX = "client";
	/**
	 * The configuration prefix for server settings - preprocessor part
	 */
	public static final String PRE_PREFIX = "pre";
	/**
	 * The configuration prefix for server settings - postprocessor part
	 */
	public static final String POST_PREFIX = "post";
	/**
	 * The configuration prefix for language settings
	 */
	public static final String LANG_PREFIX = "lang";
	/**
	 * Placeholder for activity base directory
	 */
	public static final String ACT_PLACEHOLDER = "_ACT_";
	
	private String actbaseDir;
	private HashMap<String, URL> preDesc;
	private HashMap<String, URL> postDesc;
	private HashMap<String, HashMap<String, ConfigValue>> clientConfig;
	private HashMap<String, HashMap<String, ConfigValue>> serverPreConfig;
	private HashMap<String, HashMap<String, ConfigValue>> serverPostConfig;
	private final String nl = System.getProperty("line.separator");
	private boolean isEnabled;
	private String name;
	
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
	private HashMap<String,ConfigValue> readXmlConfEntries(Node n) {
		
		HashMap<String,ConfigValue> res = new HashMap<String,ConfigValue>();

		// loop over children and if they're of type "entry", go for it
		NodeList entries = n.getChildNodes();
		for ( int i = 0 ; i < entries.getLength() ; i++ ) {
			Node c = entries.item(i);
			if ( "entry".equals(c.getNodeName()) ) {
				NamedNodeMap att = c.getAttributes();
				String key = att.getNamedItem("key").getNodeValue();					
				String value = att.getNamedItem("value").getNodeValue();
				// replace activity directory placeholder in value
				value = value.replaceAll(ACT_PLACEHOLDER, actbaseDir);
				String o = att.getNamedItem("overridable").getNodeValue();
				boolean overridable = false;;
				if ( o.equalsIgnoreCase("yes") ||  o.equalsIgnoreCase("true") || o.equals("1") ) {
					overridable = true;
				}
				res.put(key, new ConfigValue(value,overridable));
			}
			
		}
		
		return res;
		
		
	}
	
	
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
	private void loadFromXml(InputStream is) throws ParserConfigurationException, SAXException, IOException, XPathExpressionException  {

		DocumentBuilder b = DocumentBuilderFactory.newInstance().newDocumentBuilder();
		Document doc = b.parse(is);
		
		XPath xpath = XPathFactory.newInstance().newXPath();
		
		Node n;
		
		// FIXME: validation with internal DTD or Schema		
		// enter client configuration: find config branch and read it in
		//Node n = (Node)xpath.evaluate("//client-cfg", doc, XPathConstants.NODE);
		//clientConfig = readXmlConfEntries(n);
		
		// languages
		n = (Node)xpath.evaluate("//server-cfg", doc, XPathConstants.NODE);
		NodeList langs = n.getChildNodes();
		
		for (int i = 0 ; i < langs.getLength() ; i++) {
			Node l = langs.item(i);
			if (l.getNodeName().equals("lang")) {
				String lcode = l.getAttributes().getNamedItem("code").getNodeValue();

				// server configuration, pre pipeline: find config branch and read it in
				n = (Node)xpath.evaluate("//server-cfg/lang[@code='" + lcode + "']/pre", doc, XPathConstants.NODE);
				serverPreConfig.put(lcode, readXmlConfEntries(n));
				// construct pipeline URL from class path
				String d = xpath.evaluate("//server-cfg/lang[@code='" + lcode + "']/pre/pipeline/@desc", doc);
				preDesc.put(lcode, getClass().getResource(d));

				// see above for pre pipeline
				n = (Node)xpath.evaluate("//server-cfg/lang[@code='" + lcode + "']/post", doc, XPathConstants.NODE);
				serverPostConfig.put(lcode, readXmlConfEntries(n));
				d = xpath.evaluate("//server-cfg/lang[@code='" + lcode + "']/post/pipeline/@desc", doc);
				postDesc.put(lcode, getClass().getResource(d));
			}
		}
		
		
		
		// check for enabled switch
		String enabled = xpath.evaluate("/activity/@enabled", doc);
		if (enabled != null)  {
			if (enabled.toLowerCase().equals("yes") ||enabled.equals("1") ) {
				isEnabled = true;
			} else {
				isEnabled = Boolean.parseBoolean(enabled);
			}
		}
		
		// retrieve activity name
		name = xpath.evaluate("//meta/name/text()", doc);
		
		
	}
	
	
	
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn+2]
	public ActivityConfiguration(File xmlActivityConfig) throws IOException  {
		try {
			actbaseDir = xmlActivityConfig.getParentFile().getAbsolutePath();
			clientConfig = new HashMap<String, HashMap<String, ConfigValue>>();
			serverPreConfig = new HashMap<String, HashMap<String, ConfigValue>>();
			serverPostConfig = new HashMap<String, HashMap<String, ConfigValue>>();
			preDesc = new HashMap<String, URL>();
			postDesc = new HashMap<String, URL>();
			loadFromXml(new FileInputStream(xmlActivityConfig));
		} catch (Exception e) {
			System.out.println(xmlActivityConfig);
			throw new IOException(e);
		}
	}
	
	/**
	 * Internal data container for config entries.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value]
	public class ConfigValue {
		private String value;
		private boolean readOnly;
		// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
		public String getValue() {
			return value;
		}
		// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
		public boolean isReadOnly() {
			return readOnly;
		}
		// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
		public ConfigValue(String value, boolean readOnly) {
			super();
			this.readOnly = readOnly;
			this.value = value;
		}
		
		// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
		// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
		public String toString() {
			if (readOnly) {
				return value + " (read-only)";
			}
			
			return value + " (overridable)";
		}
		
	}
	
	/**
	 * @return the location of the pre pipeline descriptor or <strong>null</strong> if the 
	 * the descriptor could not be found in the class path.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
	public URL getPreDesc(String lang) {
		if (preDesc.containsKey(lang)) {
			return preDesc.get(lang);
		}
		
		return null;
	}

	/**
	 * @return the location of the post pipeline descriptor or <strong>null</strong> if the 
	 * the descriptor could not be found in the class path.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
	public URL getPostDesc(String lang) {
		if (postDesc.containsKey(lang)) {
			return postDesc.get(lang);
		}
		
		return null;
	}

	/**
	 * Private helper for obtaining config values as strings;
	 * @param key
	 * @param conf
	 * @return
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
	private String getValue(String lang, String key, HashMap<String,HashMap<String,ConfigValue>> conf) {
		if (conf.containsKey(lang) && conf.get(lang).containsKey(key)) {
			return conf.get(lang).get(key).getValue();
		}
		
		return null;		
	}
	
	/**
	 * Obtains a value from the client configuration and returns it.
	 * @param key the key for which the value should be returned.
	 * @return the configuration value for the given key or <strong>null</strong>, if no entry for the key is found.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
	public String getClientValue(String lang, String key) {
		return getValue(lang, key, clientConfig);
	}

	/**
	 * Obtains a value from the server <strong>pre pipeline</strong> configuration and returns it.
	 * @param key the key for which the value should be returned.
	 * @return the configuration value for the given key or <strong>null</strong>, if no entry for the key is found.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
	public String getServerPreValue(String lang, String key) {
		return getValue(lang, key, serverPreConfig);
	}
	
	/**
	 * Obtains a value from the server <strong>post pipeline</strong> configuration and returns it.
	 * @param key the key for which the value should be returned.
	 * @return the configuration value for the given key or <strong>null</strong>, if no entry for the key is found.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
	public String getServerPostValue(String lang, String key) {
		return getValue(lang, key, serverPostConfig);
	}	
	
	
	/**
	 * Private helper for setting a config value.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
	private boolean setValue(String lang, String key, String value, HashMap<String,HashMap<String,ConfigValue>> conf ) {
		if (conf.containsKey(lang) && conf.get(lang).containsKey(key)) {
			ConfigValue v = conf.get(lang).get(key);
			if ( v != null && v.isReadOnly() ) {
				return false;
			}
		
			conf.get(lang).put(key, new ConfigValue(value, false));
			return true;
		}
		
		return false;
	}

	/**
	 * Sets a <strong>server-side pre-pipeline</strong> configuration key-value pair if 
	 * the key is not already marked read-only.
	 * New keys are inserted and marked as not-read-only. 
	 * @param key the key of the pair.
	 * @param value the new value to set for that key.
	 * @return true if this worked out, false if that key happens to be read-only.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
	public boolean setServerPreValue(String lang, String key, String value) {
		return setValue(lang, key, value, serverPreConfig);
	}
	
	/**
	 * Sets a <strong>server-side post-pipeline</strong> configuration key-value pair if 
	 * the key is not already marked read-only.
	 * New keys are inserted and marked as not-read-only. 
	 * @param key the key of the pair.
	 * @param value the new value to set for that key.
	 * @return true if this worked out, false if that key happens to be read-only.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
	public boolean setServerPostValue(String lang, String key, String value) {
		return setValue(lang, key, value, serverPostConfig);
	}	

	/**
	 * Sets a <strong>client-side<strong> configuration key-value pair if 
	 * the key is not already marked read-only.
	 * New keys are inserted and marked as not-read-only. 
	 * @param key the key of the pair.
	 * @param value the new value to set for that key.
	 * @return true if this worked out, false if that key happens to be read-only.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
	public boolean setClientValue(String lang, String key, String value) {
		return setValue(lang, key, value, clientConfig);
	}
	
	/**
	 * Private helper for converting internal config into {@link Properties}.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
	private Properties config2Props(String lang, HashMap<String,HashMap<String,ConfigValue>> conf) {
		Properties res = new Properties();
		if (conf.containsKey(lang)) {
			Map<String, ConfigValue> m = conf.get(lang);
			for ( String key : m.keySet() ) {
				res.put(key, m.get(key).getValue());
			}
		}
		
		return res;
	}
	
	/**
	 * Returns the server configuration <strong>pre pipeline</strong> as a whole in a properties
	 * object that can be used by the UIMA pipeline.
	 * @return the server pre pipeline configuration as one object.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
	public Properties getServerPreConfigAsProp(String lang) {
		return config2Props(lang, serverPreConfig);
	}

	/**
	 * Returns the server configuration <strong>post pipeline</strong> as a whole in a properties
	 * object that can be used by the UIMA pipeline.
	 * @return the server post pipeline configuration as one object.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
	public Properties getServerPostConfigAsProp(String lang) {
		return config2Props(lang, serverPostConfig);
	}

	
	/**
	 * Returns the client configuration as a whole in a properties
	 * object.
	 * @return the client configuration as one object.
	 */
	//public Properties getClientConfigAsProp() {
		//return config2Props(clientConfig);
	//}
	

	/**
	 * @return all keys in the server pre config stored in a set.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
	public Set<String> getServerPreKeys() {
		return serverPreConfig.keySet();
	}

	/**
	 * @return all keys in the server post config stored in a set.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
	public Set<String> getServerPostKeys() {
		return serverPostConfig.keySet();
	}

	/**
	 * @return all keys in the server client config stored in a set.
	 */
	//public Set<String> getClientKeys() {
		//return clientConfig.keySet();
	//}

	/**
	 * @return all language codes that are in the server pre and
	 * server post config
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
	public Set<String> getLanguages() {
		// create a set with the intersection of the languages in the pre
		// and post configs
		Set<String> s = serverPreConfig.keySet();
		s.retainAll(serverPostConfig.keySet());
		
		return s;
	}

	/**
	 * Determines whether this activity is enabled or not.
	 * @return true if this activity is enabled, false otherwise.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
	public boolean isEnabled() {
		return isEnabled;
	}


	/**
	 * @return the nice human-readable name for this activity as stored
	 * in the configuration.
	 */
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
	public String getName() {
		return name;
	}

	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
	public String toString() {
		
		String res ="";
		res += "enabled:" + isEnabled + nl;
		res += "name:" + name + nl;
		res += "client-cfg:" + clientConfig + nl;
		res += "pipeline pre:" + preDesc + nl;
		res += "server-cfg pre:" + serverPreConfig + nl;
		res += "pipeline post:" + postDesc + nl;
		res += "server-cfg post:" + serverPostConfig + nl;
		return res;
		
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn+2]
	public static void main(String[] args) throws IOException {
		
		ActivityConfiguration ac = new ActivityConfiguration(new File(args[0]));
		System.out.println(ac);
		
	}
	

}
