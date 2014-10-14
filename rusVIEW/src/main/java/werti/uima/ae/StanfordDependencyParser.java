package werti.uima.ae;

import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;

import org.apache.log4j.Logger;

import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;

import werti.uima.ae.filter.Filter;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;
import edu.stanford.nlp.ling.TaggedWord;
import edu.stanford.nlp.parser.lexparser.LexicalizedParser;
import edu.stanford.nlp.process.PTBEscapingProcessor;
import edu.stanford.nlp.trees.GrammaticalStructure;
import edu.stanford.nlp.trees.GrammaticalStructureFactory;
import edu.stanford.nlp.trees.PennTreebankLanguagePack;
import edu.stanford.nlp.trees.Tree;
import edu.stanford.nlp.trees.TreebankLanguagePack;
import edu.stanford.nlp.trees.TypedDependency;

public class StanfordDependencyParser extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(StanfordDependencyParser.class);
	
	private TreebankLanguagePack tlp = new PennTreebankLanguagePack();
	private GrammaticalStructureFactory gsf = tlp.grammaticalStructureFactory();
	
	private static Map<String, LexicalizedParser> parsers;	
	
	private Filter filter;
	
	@Override
	public void initialize(UimaContext aContext)
			throws ResourceInitializationException {
		super.initialize(aContext);
		
		parsers = new HashMap<String, LexicalizedParser>();

		// TODO: reenable for passives
		/* try {
			parsers.put("en", WERTiContext.request("LexicalizedParser", LexicalizedParser.class, "en"));
		} catch (WERTiContextException wce) {
			throw new ResourceInitializationException(wce);
		}*/
		
		for (LexicalizedParser lp : parsers.values()) {
			lp.setOptionFlags("-maxLength", "80", "-retainTmpSubcategories");
		}
		
		String parserFilter = (String) aContext.getConfigParameterValue("parserFilter");

		try {
			filter = (Filter) Class.forName(parserFilter).newInstance();
		} catch (InstantiationException e) {
			throw new ResourceInitializationException(e);
		} catch (IllegalAccessException e) {
			throw new ResourceInitializationException(e);
		} catch (ClassNotFoundException e) {
			throw new ResourceInitializationException(e);
		}
	}
	
	
	@SuppressWarnings({ "unchecked", "rawtypes" })
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		log.debug("Starting dependency parse annotation");
		
		final String lang = jcas.getDocumentLanguage();
		
		LexicalizedParser lp;
		
		if (parsers.containsKey(lang)) {
			lp = parsers.get(lang);
		} else {
			log.error("No parser for language: " + lang);
			throw new AnalysisEngineProcessException();
		}
		
		final AnnotationIndex sentIndex = jcas.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();
		
		PTBEscapingProcessor ptbEscaper = new PTBEscapingProcessor();
		
		while (sit.hasNext()) {
			SentenceAnnotation s = sit.next();
			if (s.getParseCandidate() != true) continue;
			List<Token> tokenList = new ArrayList<Token>();
			List<TaggedWord> taggedWordList = new ArrayList<TaggedWord>();

			final Iterator<Token> tit = tokenIndex.subiterator(s);
			
			while (tit.hasNext()) {
				// keep a list of tokens to refer to when inserting
				// the parse into the CAS
				Token t = tit.next();
				tokenList.add(t);
				
				// provide parser with words and tags to improve speed
				TaggedWord tw = new TaggedWord();
				tw.setWord(t.getCoveredText());
				tw.setTag(t.getTag());
				taggedWordList.add(tw);
			}
			
			// escape characters for use with Stanford English PCFG model, 
			// which uses PTB conventions
			ptbEscaper.process(taggedWordList);

			if (taggedWordList.size() > 0 && filter.filter(tokenList)) {
				try {
					// parse the sentence
					final Tree parse = lp.apply(taggedWordList);

					final GrammaticalStructure gs = gsf.newGrammaticalStructure(parse);
					final Collection<TypedDependency> tdl = gs.typedDependencies();
					
					for (final TypedDependency td : tdl) {
						addDependencyInfo(tokenList.get(td.dep().label().index() - 1), td);
					}
					
					for (final TypedDependency td : GrammaticalStructure.getRoots(tdl)) {
						Token t = tokenList.get(td.gov().label().index() - 1);
						t.setDepid(td.gov().label().index());
						t.setDephead(0);
						t.setDeprel("root");
					}

					s.setHasdepparse(true);
				} catch (Exception e) {
					log.warn(e);
				}
			}
		}
		
		log.debug("Finished dependency parse annotation");
	}

	private void addDependencyInfo(final Token t, final TypedDependency td) {
		log.debug("Token: " + t.getCoveredText());
		log.debug("Dependency triple: " + td.dep().label().index() + " " + td.gov().label().index() + " " + td.reln().getShortName());
		t.setDepid(td.dep().label().index());
		t.setDephead(td.gov().label().index());
		t.setDeprel(td.reln().getShortName());
	}
}
