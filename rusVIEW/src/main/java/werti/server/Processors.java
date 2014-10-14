package werti.server;

import java.io.IOException;
import java.net.URL;
import java.util.Map;
import java.util.Properties;
import java.util.Set;
import java.util.TreeMap;

import javax.servlet.ServletException;

import org.apache.log4j.Logger;
import org.apache.uima.UIMAFramework;
import org.apache.uima.analysis_engine.AnalysisEngine;
import org.apache.uima.analysis_engine.AnalysisEngineDescription;
import org.apache.uima.resource.ResourceInitializationException;
import org.apache.uima.resource.metadata.ConfigurationParameterSettings;
import org.apache.uima.util.InvalidXMLException;
import org.apache.uima.util.JCasPool;
import org.apache.uima.util.XMLInputSource;

import werti.WERTiContext;

/**
 * Stores a JCasPool and an AnalysisEngine for each topic and language. The 
 * models etc. for the external tools are not loaded here.
 * 
 * There is no division into pre- and postprocessors any more (as of r1411). 
 * There is only one (aggregate) analysis engine for each topic-language 
 * combination.
 * 
 * @author Adriane Boyd
 */
public class Processors {
	private static final Logger log =
		Logger.getLogger(Processors.class);

	private TreeMap<String, TreeMap<String, AnalysisEngine>> aeMap;
    
	public Processors(Activities activities) throws IOException, ServletException {
		aeMap = new TreeMap<String, TreeMap<String, AnalysisEngine>>();

		// This constructor is called by WERTiServlet.doGet or .doPost upon the
		// very first request after the start of the server. It is NOT called 
		// for every request. This is why the JCasPools are created here.
		// (updated 2012-12-17, r1399)
		WERTiContext.jCasPoolMap = new TreeMap<String, Map<String, JCasPool>>();
		
		for (String activity : activities) {
			ActivityConfiguration config = activities.getActivity(activity);
			
			Map<String, JCasPool> innerJCasPoolMap = new TreeMap<String, JCasPool>();
			WERTiContext.jCasPoolMap.put(activity, innerJCasPoolMap);
			
			Set<String> langs = config.getLanguages();
			
			for (String l : langs) {
				if (!aeMap.containsKey(l)) {
					aeMap.put(l, new TreeMap<String, AnalysisEngine>());
				}
				
				final URL aeDesc = config.getDesc(l);

				try { // to initialize UIMA components
					
					AnalysisEngine ae = initAE(loadDescriptor(aeDesc), config.getServerConfigAsProp(l));
					aeMap.get(l).put(activity, ae);
					
					// create a pool of JCas objects
					JCasPool jCasPool = new JCasPool(WERTiContext.NUM_THREADS, ae);
					innerJCasPoolMap.put(l, jCasPool);

				} catch (InvalidXMLException ixmle) {
					log.fatal("Error initializing XML code. Invalid?", ixmle);
					throw new ServletException("", ixmle);
				} catch (ResourceInitializationException rie) {
					log.fatal("Error initializing resource", rie);
					throw new ServletException("", rie);
				} catch (IOException ioe) {
					log.fatal("Error accessing descriptor file", ioe);
					throw new ServletException("", ioe);
				} catch (NullPointerException npe) {
					log.fatal("Error accessing descriptor files or creating analysis objects", npe);
					throw new ServletException("", npe);
				}
			}
		}
	}
	
	public boolean hasProcessor(String lang, String key) {
		if (aeMap.containsKey(lang)) {
			return aeMap.get(lang).containsKey(key);
		}
		else {
			return false;
		}
	}
	
	public AnalysisEngine getProcessor(String lang, String key) {
		if (aeMap.containsKey(lang)) {
			return aeMap.get(lang).get(key);
		}
		
		return null;
	}
	
	/**
	 * Private helper that auto-converts a string to another type, depending on 
	 * a given type. The fallback strategy is to produce a clone of the string passed to
	 * the method.
	 * @param originalParameter the given type that determines the return type
	 * @param value the value to convert into the new type
	 * @return an object of the same type as originalParameter holding the parsed value.
	 * @throws NumberFormatException if the parsing went wrong.
	 */
	private Object autoConvertParameter(Object originalParameter, String value) {
		
		if ( originalParameter instanceof Boolean) {
			return Boolean.parseBoolean(value);
		}
		if ( originalParameter instanceof Integer) {
			return Integer.parseInt(value);
		}
		if ( originalParameter instanceof Float) {
			return Float.parseFloat(value);
		}
		
		// fallback: return as string
		return new String(value);
		
	}	
	
	/**
	 * Private helper that creates an object holding an analysis engine description from file.
	 * @param descriptor the path to the analysis engine descriptor file.
	 * @return an object holding the AE description.
	 * @throws IOException
	 * @throws InvalidXMLException
	 */
	private AnalysisEngineDescription loadDescriptor(URL descriptor) throws IOException, InvalidXMLException {

		log.debug("Loading AE descriptor from url:  " + descriptor.getPath());
		XMLInputSource xmlInput = new XMLInputSource(descriptor);
		AnalysisEngineDescription description = UIMAFramework.getXMLParser().parseAnalysisEngineDescription(xmlInput);
		return description;
		
	}
	
	/**
	 * Private helper initializing the UIMA pipeline.
	 * @param description AE descriptor file for this language & topic (in desc/)
	 * @throws ResourceInitializationException
	 */
	private AnalysisEngine initAE(AnalysisEngineDescription description, Properties config) throws ResourceInitializationException {

		// read descriptor from disk and initialize a new annotator
		// adjust configuration in the AE description by setting all parameters from config
		ConfigurationParameterSettings settings = description.getAnalysisEngineMetaData().getConfigurationParameterSettings();
		for ( Object k : config.keySet() ) {
			String key = (String)k;
			String value = (String)config.get(key);
			
			// auto-adjust type of the parameter according to the type found in the description
			Object genericTypeValue = autoConvertParameter(settings.getParameterValue(key), value);
			settings.setParameterValue(key, genericTypeValue);
			
			log.debug("Setting AE parameter: " + key + "=" + value);
		}
		
		// produce the annotator from the description
		log.debug("Initializing AE.");
		AnalysisEngine ae = UIMAFramework.produceAnalysisEngine(description, WERTiContext.NUM_THREADS, WERTiContext.TIMEOUT);
		return ae;
		
	}
}
