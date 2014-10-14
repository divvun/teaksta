package werti.uima.enhancer;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import org.apache.commons.lang.StringEscapeUtils;
import org.apache.log4j.Logger;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIndex;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;

import werti.uima.types.Enhancement;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;
import werti.util.EnhancerUtils;

/**
 * Highlights Konjunktiv II forms.
 *
 * @author Adriane Boyd
 * @version 0.1
 */

public class KonjunktivEnhancer extends JCasAnnotator_ImplBase {
	private static final Logger log =
		Logger.getLogger(KonjunktivEnhancer.class);
	
	/**
	 * Inserts a plain-text parse after each sentence.
	 * 
	 * @param JCas cas
	 */
	@Override
	@SuppressWarnings("unchecked")
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		log.debug("Starting enhancement");

		final String finclass = "wertiviewkonjfin";
		final String infinclass = "wertiviewkonjinfin";
		
		final String konj2class = "wertiviewkonj2";
		
		final String typeauxwclass = "wertiviewkonjauxwuerden";
		final String typeauxhclass = "wertiviewkonjauxhaben";
		final String typemainclass = "wertiviewkonjmain";
		
		final String conjregclass = "wertiviewkonjreg";
		final String conjirregclass = "wertiviewkonjirreg";
		
		final FSIndex sentIndex = cas.getAnnotationIndex(SentenceAnnotation.type);
		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();
		final AnnotationIndex tokenIndex = cas.getAnnotationIndex(Token.type);

		SentenceAnnotation s;

		int id = 1;

		while (sit.hasNext()) {
			s = sit.next();
			Iterator<Token> tit = tokenIndex.subiterator(s);
			
			List<Token> enhancedTokens = new ArrayList<Token>();

			// find finite subjunctive verbs
			while (tit.hasNext()) {
				Token t = tit.next();
				
				final String tag = t.getDetailedtag();
				if (tag == null) {
					log.debug("Encountered token with NULL tag");
				} else if (tag.matches(".*\\.Past\\.Subj$")) {
					final Enhancement e = new Enhancement(cas);
					e.setBegin(t.getBegin());
					e.setEnd(t.getEnd());
					e.setRelevant(true);
					
					String type = typemainclass;
					if (tag.matches(".*Haben.*") || tag.matches(".*Sein.*")) {
						type = typeauxhclass;
					} else if (tag.matches(".*Aux.*")) {
						type = typeauxwclass;
					}
					
					String lemma = t.getLemma();
					
					String conj = "";
					if (lemma.endsWith("n")) {
						String finiteStem = t.getCoveredText().toLowerCase().replaceAll("(te|test|ten|tet|e|est|et|en)$", "");
						if (lemma.startsWith(finiteStem) && !tag.matches(".*Mod.*")) {
							conj = conjregclass;
						} else {
							conj = conjirregclass;
						}
					}					
					
					e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span", id) 
							+ "\" class=\"wertiviewtoken " + finclass + " " + konj2class + " " + type + " " + conj + "\" title=\"" + StringEscapeUtils.escapeHtml(lemma) + "\">");
					e.setEnhanceEnd("</span>");
					e.addToIndexes();
					
					enhancedTokens.add(t);

					id++;
					
					// mark dependent infinite verbs and separable prefixes
					List<Token> infiniteVerbs = getAuxDependents(tokenIndex, t, s);
					for (Token t2 : infiniteVerbs) {
						final Enhancement e2 = new Enhancement(cas);
						e2.setBegin(t2.getBegin());
						e2.setEnd(t2.getEnd());
						e2.setRelevant(true);
						
						e2.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span", id) 
								+ "\" class=\"wertiviewtoken " + infinclass + " " + konj2class + " " + type + "\">");
						e2.setEnhanceEnd("</span>");
						e2.addToIndexes();
						
						enhancedTokens.add(t2);
						
						id++;
					}
				} else {
					if (!enhancedTokens.contains(t)) {
						final Enhancement e = new Enhancement(cas);
						e.setBegin(t.getBegin());
						e.setEnd(t.getEnd());

						e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span", id) 
								+ "\" class=\"wertiviewtoken\">");
						e.setEnhanceEnd("</span>");

						e.addToIndexes();
						id++;
					}
				}
			}
		}
		log.debug("Finished enhancement");
	}
	
	@SuppressWarnings("unchecked")
	private List<Token> getAuxDependents(AnnotationIndex tokenIndex, Token t, SentenceAnnotation s) {
		final Iterator<Token> tit = tokenIndex.subiterator(s);
		List<Token> infiniteVerbs = new ArrayList<Token>();
		int maltAuxId = t.getMaltdepid();
		
		while (tit.hasNext()) {
			Token t2 = tit.next();

			if (t2.getMaltdephead() == maltAuxId && t2.getMaltdeprel().matches("(AUX|AVZ)")) {
				infiniteVerbs.add(t2);
				infiniteVerbs.addAll(getAuxDependents(tokenIndex, t2, s));
			}
		}
		
		return infiniteVerbs;
	}
}
