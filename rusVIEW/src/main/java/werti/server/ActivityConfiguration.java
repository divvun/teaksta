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
 * @version $Id: ActivityConfiguration.java 1661 2014-08-25 16:34:39Z eschaf@SFS.UNI-TUEBINGEN.DE $
 */
public class ActivityConfiguration {

	/**
	 * The configuration prefix for client settings
	 */
	public static final String CLIENT_PREFIX = "client";
	/**
	 * The configuration prefix for server settings
	 */
	public static final String SERVER_PREFIX = "pre";
	/**
	 * The configuration prefix for language settings
	 */
	public static final String LANG_PREFIX = "lang";
	/**
	 * Placeholder for activity base directory
	 */
	public static final String ACT_PLACEHOLDER = "_ACT_";
	
	private String actbaseDir;
	private HashMap<String, URL> aeDesc;
	private HashMap<String, HashMap<String, ConfigValue>> clientConfig;
	private HashMap<String, HashMap<String, ConfigValue>> serverConfig;
	private final String nl = System.getProperty("line.separator");
	private boolean isEnabled;
	private String name;
	
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
				boolean overridable = false;
				if ( o.equalsIgnoreCase("yes") ||  o.equalsIgnoreCase("true") || o.equals("1") ) {
					overridable = true;
				}
				// if something is overridable, it is not read-only
				res.put(key, new ConfigValue(value,(!overridable)));
			}
			
		}
		
		return res;
		
		
	}
	
	
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

				// server configuration, pipeline: find config branch and read it in
				n = (Node)xpath.evaluate("//server-cfg/lang[@code='" + lcode + "']/pre", doc, XPathConstants.NODE);
				serverConfig.put(lcode, readXmlConfEntries(n));
				// construct pipeline URL from class path
				String d = xpath.evaluate("//server-cfg/lang[@code='" + lcode + "']/pre/pipeline/@desc", doc);
				aeDesc.put(lcode, getClass().getResource(d));
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
	
	
	
	public ActivityConfiguration(File xmlActivityConfig) throws IOException  {
		try {
			actbaseDir = xmlActivityConfig.getParentFile().getAbsolutePath();
			clientConfig = new HashMap<String, HashMap<String, ConfigValue>>();
			serverConfig = new HashMap<String, HashMap<String, ConfigValue>>();
			aeDesc = new HashMap<String, URL>();
			loadFromXml(new FileInputStream(xmlActivityConfig));
		} catch (Exception e) {
			System.out.println(xmlActivityConfig);
			throw new IOException(e);
		}
	}
	
	/**
	 * Internal data container for config entries.
	 */
	public class ConfigValue {
		private String value;
		private boolean readOnly;
		public String getValue() {
			return value;
		}
		public boolean isReadOnly() {
			return readOnly;
		}
		public ConfigValue(String value, boolean readOnly) {
			super();
			this.readOnly = readOnly;
			this.value = value;
		}
		
		@Override
		public String toString() {
			if (readOnly) {
				return value + " (read-only)";
			}
			
			return value + " (overridable)";
		}
		
	}
	
	/**
	 * @return the location of the pipeline descriptor or <strong>null</strong> if the 
	 * the descriptor could not be found in the class path.
	 */
	public URL getDesc(String lang) {
		if (aeDesc.containsKey(lang)) {
			return aeDesc.get(lang);
		}
		
		return null;
	}

	/**
	 * Private helper for obtaining config values as strings;
	 * @param key
	 * @param conf
	 * @return
	 */
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
	public String getClientValue(String lang, String key) {
		return getValue(lang, key, clientConfig);
	}

	/**
	 * Obtains a value from the server pipeline configuration and returns it.
	 * @param key the key for which the value should be returned.
	 * @return the configuration value for the given key or <strong>null</strong>, if no entry for the key is found.
	 */
	public String getServerValue(String lang, String key) {
		return getValue(lang, key, serverConfig);
	}
	
	
	/**
	 * Private helper for setting a config value.
	 */
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
	 * Sets a <strong>server-side pipeline</strong> configuration key-value pair if 
	 * the key is not already marked read-only.
	 * New keys are inserted and marked as not-read-only. 
	 * @param key the key of the pair.
	 * @param value the new value to set for that key.
	 * @return true if this worked out, false if that key happens to be read-only.
	 */
	public boolean setServerValue(String lang, String key, String value) {
		return setValue(lang, key, value, serverConfig);
	}
	
	/**
	 * Sets a <strong>client-side<strong> configuration key-value pair if 
	 * the key is not already marked read-only.
	 * New keys are inserted and marked as not-read-only. 
	 * @param key the key of the pair.
	 * @param value the new value to set for that key.
	 * @return true if this worked out, false if that key happens to be read-only.
	 */
	public boolean setClientValue(String lang, String key, String value) {
		return setValue(lang, key, value, clientConfig);
	}
	
	/**
	 * Private helper for converting internal config into {@link Properties}.
	 */
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
	 * Returns the server configuration <strong>pipeline</strong> as a whole in a properties
	 * object that can be used by the UIMA pipeline.
	 * @return the server pipeline configuration as one object.
	 */
	public Properties getServerConfigAsProp(String lang) {
		return config2Props(lang, serverConfig);
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
	 * @return all keys in the server config stored in a set.
	 */
	public Set<String> getServerKeys() {
		return serverConfig.keySet();
	}

	/**
	 * @return all keys in the server client config stored in a set.
	 */
	//public Set<String> getClientKeys() {
		//return clientConfig.keySet();
	//}

	/**
	 * @return all language codes that are in the server config
	 */
	public Set<String> getLanguages() {
		// create a set with the intersection of the languages in the configs
		Set<String> s = serverConfig.keySet();
		return s;
	}

	/**
	 * Determines whether this activity is enabled or not.
	 * @return true if this activity is enabled, false otherwise.
	 */
	public boolean isEnabled() {
		return isEnabled;
	}


	/**
	 * @return the nice human-readable name for this activity as stored
	 * in the configuration.
	 */
	public String getName() {
		return name;
	}

	@Override
	public String toString() {
		
		String res ="";
		res += "enabled:" + isEnabled + nl;
		res += "name:" + name + nl;
		res += "client-cfg:" + clientConfig + nl;
		res += "pipeline:" + aeDesc + nl;
		res += "server-cfg:" + serverConfig + nl;
		return res;
		
	}
	
	public static void main(String[] args) throws IOException {
		
		ActivityConfiguration ac = new ActivityConfiguration(new File(args[0]));
		System.out.println(ac);
		
	}
	

}
