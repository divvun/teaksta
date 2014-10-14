package werti.uima.ae;

import java.util.ArrayList;
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

import de.sfb833.a4.RFTagger.RFTagger;
import de.sfb833.a4.RFTagger.tagsetconv.ConverterFactory;
import de.sfb833.a4.RFTagger.tagsetconv.NoSuchTagsetException;
import de.sfb833.a4.RFTagger.tagsetconv.TagsetConverter;

import werti.WERTiContext;
import werti.WERTiContext.WERTiContextException;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

/**
 * Annotator wrapper for OpenNLP tagger.
 * 
 * Depends on {@link Token} and {@link SentenceAnnotation} annotation from 
 * {@link OpenNlpTokenizer} and {@link OpenNlpSentenceDetector}.
 * 
 * @author Adriane Boyd
 */
public class RFTaggerAnnotator extends JCasAnnotator_ImplBase {
	private static Map<String, RFTagger> taggers;
	private static Map<String, TagsetConverter> converters;
	private static final Logger log = Logger.getLogger(RFTaggerAnnotator.class);
	
	@Override
	public void initialize(UimaContext aContext)
			throws ResourceInitializationException {
		super.initialize(aContext);
		
		try {
			taggers = new HashMap<String, RFTagger>();
			taggers.put("de", WERTiContext.request("RFTagger", RFTagger.class, "de"));
		} catch (WERTiContextException wce) {
			throw new ResourceInitializationException(wce);
		}
		
		try {
			converters = new HashMap<String, TagsetConverter>();
			converters.put("de", ConverterFactory.getConverter("stts"));
		} catch (NoSuchTagsetException e) {
			throw new ResourceInitializationException(e);
		}
	}
	
	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		log.debug("Starting tag annotation");
		
		final AnnotationIndex sentIndex = jcas.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);
				
		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();
		
		final String lang = jcas.getDocumentLanguage();
		
		RFTagger tagger;
		TagsetConverter conv;
				
		if (taggers.containsKey(lang)) {
			tagger = taggers.get(lang);
		} else {
			log.error("No tagger for language: " + lang);
			throw new AnalysisEngineProcessException();
		}
		
		if (converters.containsKey(lang)) {
			conv = converters.get(lang);
		} else {
			log.error("No converter for language: " + lang);
			throw new AnalysisEngineProcessException();
		}
		
		while (sit.hasNext()) {
			List<Token> tokenlist = new ArrayList<Token>();
			List<String> tokens = new ArrayList<String>();
			
			final Iterator<Token> tit = tokenIndex.subiterator(sit.next());
			
			while (tit.hasNext()) {
				final Token t = tit.next();
				tokenlist.add(t);
				tokens.add(t.getCoveredText());
			}
		
			List<String> tags = tagger.getTags(tokens);
			
			for (int i = 0; i < tags.size(); i++) {
				tokenlist.get(i).setDetailedtag(tags.get(i));
				tokenlist.get(i).setTag(conv.rftag2tag(tags.get(i)));
			}
		}
		
		log.debug("Finished tag annotation");
	}
}
