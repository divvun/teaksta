package werti;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.ObjectInputStream;

import java.util.HashMap;
import java.util.Map;
import java.util.Properties;

import java.util.zip.GZIPInputStream;

import javax.servlet.ServletConfig;
import javax.servlet.ServletContext;

import org.apache.log4j.Logger;
import org.apache.uima.util.JCasPool;

import com.aliasi.hmm.HiddenMarkovModel;

import edu.stanford.nlp.parser.lexparser.LexicalizedParser;

import com.aliasi.hmm.HmmDecoder;

import opennlp.maxent.GISModel;
import opennlp.maxent.io.SuffixSensitiveGISModelReader;
import opennlp.tools.postag.POSDictionary;

import de.sfb833.a4.RFTagger.*;

/**
 * Your random garden variety memory hog.
 *
 * This is a singleton class and manages the main memory-intensive fields
 * of the servlet in a static fashion.
 *
 * The intended use is for this to get constructed during Servlet.init() and
 * henceforth during the lifecycle of the servlet be the go-to place for AEs and
 * other objects to get their resources.
 *
 * @author Aleksandar Dimitrov
 * @version 0.2
 */
public class WERTiContext {

	public static Properties p;
	private static InputStreamFactory byteDispenser;
	public static ServletContext context;

	private static Map<String, Map<String, Model<?>>> models;
	private static final Logger log   = Logger.getLogger(WERTiContext.class);
	private static final String PROPS = "/WERTi.properties";
	
	// assigns a JCasPool to each language-topic combination
	public static Map<String, Map<String, JCasPool>> jCasPoolMap;
	// number of requests that can be processed simultaneously = number of 
	// JCases in the JCasPool = number of AnalysisEngines to keep around
	// (5 is a useful number for realistic usage scenarios of VIEW)
	public static final int NUM_THREADS = 1;
	// the maximum number of milliseconds that a new request should wait before
	// throwing an exception (0 means forever)
	// (I assume that people would give up after no more than 30 seconds)
	public static final int TIMEOUT = 30000;
	
	private static abstract class Model<T> {
		T item; protected abstract T manufacture() throws WERTiContextException ;
		final T request() throws WERTiContextException {
			if (item == null) { item = manufacture(); }
		     return item;
		}
	}

	private static abstract class InputStreamFactory {
		public abstract InputStream requestInputStream(String model);
	}

	private static final String propertiesPath =
		System.getProperty("werti.serverProperties", PROPS);

	private static Properties initProps(final InputStream is) throws WERTiContextException {
		final Properties p = new Properties();
		if (is != null) {
			try { p.load(is); }
			catch (IOException ioe) { throw from_ioe("properties", ioe); }
			return p;
		} else {
			throw new WERTiContextException("Failed to get resource for WERTi.properties");
		}
	}

	// is this function ever used?
	private static void init() throws WERTiContextException {
		byteDispenser = new InputStreamFactory() {
			final String root = System.getenv("PWD");
			@Override
			public InputStream requestInputStream(String location) {
				log.debug("Attempting to load "+root+location);
				final ClassLoader cl = WERTiContext.class.getClassLoader();
				return cl.getResourceAsStream(location);
			}
		};
		commoninit();
	}

	public static void init(final ServletConfig newsc) throws WERTiContextException {
		context = newsc.getServletContext();
		byteDispenser = new InputStreamFactory() {
			@Override
			public InputStream requestInputStream(String location) {
				log.debug("Attempting to load "+context.getRealPath(location)+".");
				final InputStream is = context.getResourceAsStream(location);
				if (is == null) {
					log.fatal("Could not access "
					          +context.getRealPath(location)
					          +" for whatever reason");
				}
				return is;
			}
		};
		commoninit();
	}

	@SuppressWarnings("serial")
	private static void commoninit() throws WERTiContextException {
		assert(byteDispenser != null);
		if (p == null) {
			p = initProps(byteDispenser.requestInputStream(propertiesPath));
		}
		
		models = new HashMap<String, Map<String, Model<?>>>();
		Map<String, Model<?>> models_en = new HashMap<String,Model<?>>() {{ 
			put("HmmDecoder",new Model<HmmDecoder>(){
						@Override
						protected HmmDecoder manufacture() throws WERTiContextException {
							final HiddenMarkovModel hmm = readObjectFor(HiddenMarkovModel.class, "lgptagger.en");
							return new HmmDecoder(hmm);
						}
			});
			// TODO: reenable for passives (and ensure thread-safety)
			/*put("LexicalizedParser",new Model<LexicalizedParser>(){
				protected LexicalizedParser manufacture() throws WERTiContextException {
				    final String grammar = p.getProperty("stanfordP.en");
				    final String[] options = { "-maxLength", "80", "-retainTmpSubcategories" };
				    return LexicalizedParser.loadModel(grammar, options);
				}
			});
			put("MaltParserService",new Model<MaltParserService>(){
				protected MaltParserService manufacture() throws WERTiContextException {
					final String model = p.getProperty("maltparser.en");
					final File mpfile = new File(context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("maltparserpath"));
					log.info(String.format("-c %s -m parse -w %s", model, mpfile.getAbsolutePath()));
					try {
						OptionManager.instance().loadOptionDescriptionFile();
						OptionManager.instance().getOptionDescriptions().generateMaps();
						final MaltParserService maltParserService = new MaltParserService();						
						maltParserService.initializeParserModel(String.format("-c %s -m parse -w %s", model, mpfile.getAbsolutePath()));
						
						return maltParserService;
					} catch (MaltChainedException ioe) {
						throw new WERTiContextException
							("Failed to load MaltParser.",ioe);
					}
				}
			});*/
			put("TokenizerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlptokenizer.en");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp)); 
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
							("Failed to load OpenNLP tokenizer.",ioe);
					}
				}
			});
			put("SentenceDetectorME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlpsbd.en");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
							("Failed to load OpenNLP SBD.",ioe);
					}
				}
			});
			put("POSTaggerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlptagger.en");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp)); 
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
							("Failed to load OpenNLP tagger.",ioe);
					}
				}
			});
			put("POSTaggerME_dict",new Model<POSDictionary>() {
				@Override
				protected POSDictionary manufacture() throws WERTiContextException {
					final String tagdictp = makePathForModel("onlptagger-tagdict.en");
					try {
						POSDictionary tagdict = new POSDictionary(context.getRealPath("/") + tagdictp);
						return tagdict;
					} catch (IOException ioe) {
						throw new WERTiContextException
							("Failed to load OpenNLP tagger.",ioe);
					}
				}
			});
			put("ChunkerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlpchunker.en");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
							("Failed to load OpenNLP chunker.",ioe);
					}
				}
			});
			// no model available for English 3.2?
			/*put("TreeTaggerWrapper",new Model<TreeTaggerWrapper<String>>() {
				protected TreeTaggerWrapper<String> manufacture() throws WERTiContextException {
					final String modelPath = makePathForModel("treetagger-model.en");
					final String modelEncoding = p.getProperty("treetagger-encoding.en");
					final String ttPath = p.getProperty("treetagger-path");
					TreeTaggerWrapper<String> tt = new TreeTaggerWrapper<String>();

					try {
						// set the TreeTagger model and encoding
						tt.setModel(modelPath + ":" + modelEncoding);

						// set the TreeTagger path
						DefaultExecutableResolver res = new DefaultExecutableResolver();
						ArrayList<String> paths = new ArrayList<String>();
						paths.add(ttPath);
						res.setAdditionalPaths(paths);
						tt.setExecutableProvider(res);
						return tt;
					} catch (Exception e) {
						throw new WERTiContextException("Failed to load TreeTaggerWrapper.", e);
					}
				}
			});*/
		}};
		Map<String, Model<?>> models_es = new HashMap<String,Model<?>>() {{
			put("TokenizerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlptokenizer.es");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp)); 
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP tokenizer.",ioe);
					}
				}
			});
			put("SentenceDetectorME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlpsbd.es");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP SBD.",ioe);
					}
				}
			});
			put("POSTaggerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlptagger.es");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP tagger.",ioe);
					}
				}
			});
			put("ChunkerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlpchunker.es");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP chunker.",ioe);
					}
				}
			});
			// TODO: figure out why TreeTaggerWrapper crashes after a while when
			// the model has been loaded in WERTiContext so that the model
			// loading can be moved back to an initialization step
			// (also see werti.uima.ae.TreeTaggerAnnotator.process)
			/*put("TreeTaggerWrapper",new Model<TreeTaggerWrapper<String>>() {
				protected TreeTaggerWrapper<String> manufacture() throws WERTiContextException {
					final String modelPath = context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("treetagger-model.es");
					final String modelEncoding = p.getProperty("treetagger-encoding.es");
					final String ttPath = p.getProperty("treetagger-path");
					TreeTaggerWrapper<String> tt = new TreeTaggerWrapper<String>();

					try {
						// set the TreeTagger model and encoding
						tt.setModel(modelPath + ":" + modelEncoding);

						// set the TreeTagger path
						DefaultExecutableResolver res = new DefaultExecutableResolver();
						ArrayList<String> paths = new ArrayList<String>();
						paths.add(ttPath);
						res.setAdditionalPaths(paths);
						tt.setExecutableProvider(res);
						return tt;
					} catch (Exception e) {
						throw new WERTiContextException("Failed to load TreeTaggerWrapper.", e);
					}
				}
			});*/
		}};
		Map<String, Model<?>> models_de = new HashMap<String,Model<?>>() {{
			put("TokenizerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlptokenizer.de");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp)); 
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP tokenizer.",ioe);
					}
				}
			});
			put("SentenceDetectorME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlpsbd.de");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP SBD.",ioe);
					}
				}
			});
			put("POSTaggerME",new Model<GISModel>() {
				@Override
				protected GISModel manufacture() throws WERTiContextException {
					final String mp = makePathForModel("onlptagger.de");
					try {
						final SuffixSensitiveGISModelReader reader = new SuffixSensitiveGISModelReader(new File(context.getRealPath("/") + mp));
						return reader.getModel();
					} catch (IOException ioe) {
						throw new WERTiContextException
						("Failed to load OpenNLP tagger.",ioe);
					}
				}
			});
			// TODO see above
			/*put("TreeTaggerWrapper",new Model<TreeTaggerWrapper<String>>() {
				protected TreeTaggerWrapper<String> manufacture() throws WERTiContextException {
					final String modelPath = context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("treetagger-model.de");
					final String modelEncoding = p.getProperty("treetagger-encoding.de");
					final String ttPath = p.getProperty("treetagger-path");
					TreeTaggerWrapper<String> tt = new TreeTaggerWrapper<String>();

					try {
						// set the TreeTagger model and encoding
						tt.setModel(modelPath + ":" + modelEncoding);

						// set the TreeTagger path
						DefaultExecutableResolver res = new DefaultExecutableResolver();
						ArrayList<String> paths = new ArrayList<String>();
						paths.add(ttPath);
						res.setAdditionalPaths(paths);
						tt.setExecutableProvider(res);
						return tt;
					} catch (Exception e) {
						throw new WERTiContextException("Failed to load TreeTaggerWrapper.", e);
					}
				}
			});*/
			put("RFTagger",new Model<RFTagger>() {
				@Override
				protected RFTagger manufacture() throws WERTiContextException {					
					try {
						// initializing parameter file
						final File modelFile = new File(context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("rftagger.de"));
						// initialize the tagger with defaults
						return new RFTagger(new de.sfb833.a4.RFTagger.Model(modelFile));
					} catch (IOException ioe) {
						throw new WERTiContextException
							("Failed to load RFTagger.",ioe);
					}
				}
			});
			put("LexicalizedParser",new Model<LexicalizedParser>(){
				@Override
				protected LexicalizedParser manufacture() throws WERTiContextException {
					final String grammar = p.getProperty("stanfordP.de");
				    final String[] options = { "-maxLength", "80", "-retainTmpSubcategories" };
				    return LexicalizedParser.loadModel(grammar, options);
				}
			});
			// moved to werti.uima.ae.MaltParser.initialize
			/*put("MaltParserService",new Model<MaltParserService>(){
				protected MaltParserService manufacture() throws WERTiContextException {
					final String model = p.getProperty("maltparser.de");
					final File mpfile = new File(context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("maltparserpath"));
					log.info(String.format("-c %s -m parse -w %s", model, mpfile.getAbsolutePath()));
					try {
						OptionManager.instance().loadOptionDescriptionFile();
						OptionManager.instance().getOptionDescriptions().generateMaps();
						final MaltParserService maltParserService = new MaltParserService();						
						maltParserService.initializeParserModel(String.format("-c %s -m parse -w %s", model, mpfile.getAbsolutePath()));
						
						return maltParserService;
					} catch (MaltChainedException ioe) {
						throw new WERTiContextException
							("Failed to load MaltParser.",ioe);
					}
				}
			});*/
		}};
		
		/*Map<String, Model<?>> models_htmlcontent = new HashMap<String,Model<?>>() {{
			put("Classifier",new Model<Classifier>(){
				protected Classifier manufacture() throws WERTiContextException {
					log.info("Loading HTML classifier model");
					final String modelFileName =  context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("htmlcontent.misc");
					try {
						return (Classifier) SerializationHelper.read(modelFileName);
					}
					catch (Exception e) {
						e.printStackTrace();
						throw new WERTiContextException
							("Failed to load HTML content classifier model " + modelFileName);
					}
				}
			});
		}};
		
		Map<String, Model<?>> models_htmlsmoother = new HashMap<String,Model<?>>() {{
			put("Classifier",new Model<Classifier>(){
				protected Classifier manufacture() throws WERTiContextException {
					log.info("Loading HTML smoother model");
					final String modelFileName =  context.getRealPath("/") + p.getProperty("models.base") + p.getProperty("htmlsmoother.misc");
					try {
						return (Classifier) SerializationHelper.read(modelFileName);
					}
					catch (Exception e) {
						e.printStackTrace();
						throw new WERTiContextException
							("Failed to load HTML content smoother model " + modelFileName);
					}
				}
			});
		}};*/
		
		models.put("en", models_en);
		models.put("es", models_es);
		models.put("de", models_de);
		/*models.put("htmlcontent", models_htmlcontent);
		models.put("htmlsmoother", models_htmlsmoother);*/
	}

	private static <T> T conditionalCast(Class<T> c, Object o) throws WERTiContextException {
		if (c.isInstance(o)) { return c.cast(o); }
		// this shouldn't ever happen.
		else throw new WERTiContextException
			("Can't cast type "+o.getClass()+" to "+c.getName()+".");
	}
	
	public static <T> T request(String modelId, Class<T> c) throws WERTiContextException {
		return request(modelId, c, "en");
	}

	public static <T> T request(String modelId, Class<T> c, String lang) throws WERTiContextException {
		if (byteDispenser == null) {
			log.warn("Initializing local context.");
			init();
		} else { 
			log.debug("Using pre-existing context."); 
		}
		if (models.containsKey(lang)) {
			if (models.get(lang).containsKey(modelId)) {
				final Model<?> m = models.get(lang).get(modelId);
				log.debug("Requested model "+modelId);
				final Object o = m.request();
				return conditionalCast(c,o);
			}
		}

		throw new WERTiContextException("Cannot fulfil request for unknown class " 
				+ c + " for language " + lang + ".");
	}

	/**
	 * This is a super-safe method that shuoldn't leak any dangling references.
	 * Thanks to dmlloyd at ##java.
	 */
	private static Object getResourceObject(InputStream is, boolean zipped)
		throws WERTiContextException {
		if (is == null) {
			throw new WERTiContextException("Can't get object resource for null inputStream");
		}
		try { // to open a connection to the resource
			if (zipped) { is = new GZIPInputStream(is); }
			try { // to connect to the object input stream of the resource
				final ObjectInputStream ois = new ObjectInputStream(is);
				try { // to actually read it in and return it.
					final long t = System.currentTimeMillis();
					final Object o = ois.readObject();
					ois.close();
					is.close();
					log.info("Loading took "+(System.currentTimeMillis()-t)+"ms.");
					return o;
				} catch (ClassNotFoundException cnfe) {
					throw new WERTiContextException
						("The class of the model object is unknown.", cnfe);
				} finally {
					try { ois.close(); }
					catch(IOException ioe) { throw new WERTiContextException(ioe); }
				}
			} finally {
				try { is.close(); }
				catch(IOException ioe) { throw new WERTiContextException(ioe); }
			}
		}
		catch (IOException ioe) { throw new WERTiContextException(ioe); }
		catch (NullPointerException npe) { throw new WERTiContextException(npe); }
	}

	@SuppressWarnings("serial")
	public static class WERTiContextException extends Exception {
		public WERTiContextException(String message) { super(spam(message)); }
		public WERTiContextException(String message, Throwable cause) { 
			super(spam(message), cause);
		}
		public WERTiContextException(Throwable cause) { super(cause); }

	}

	public static WERTiContextException from_ioe(String path) {
		return new WERTiContextException("Could not access "+path);
	}

	public static WERTiContextException from_ioe(String path, Throwable e) {
		return new WERTiContextException("Could not access "+path,e);
	}

	private static String spam(final String message) {
		return "WERTiContext found a problem: "+message;
	}
	private static <T> T readObjectFor(Class<T> c, String t) throws WERTiContextException {
		final String modelPath = makePathForModel(t);
		InputStream is = byteDispenser.requestInputStream(modelPath);
		boolean isZipped = p.getProperty(t+".zipped").equals("yes")
		                 || modelPath.endsWith(".gz");
		final Object o = getResourceObject(is,isZipped);
		return conditionalCast(c,o);
	}
	private static String makePathForModel(String t) {
		final String s = p.getProperty("models.base")+p.getProperty(t);
		log.debug("Loading model from "+s+".");
		return s;
	}
}
