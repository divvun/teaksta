package werti.uima.enhancer;

import java.util.Arrays;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Set;

import org.apache.log4j.Logger;

import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;

import org.apache.uima.analysis_engine.AnalysisEngineProcessException;

import org.apache.uima.cas.FSIndex;

import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;

import werti.uima.types.Enhancement;

import werti.uima.types.annot.Token;
import werti.util.CasUtils;
import werti.util.EnhancerUtils;

/**
 * An enhancement class that puts WERTi-<tt>&lt;span&gt;</tt>s around <em>all</em>
 * tokens and optionally gives them the attribute 'wertiviewhit' when they belong to a given
 * POS.
 *
 * @author Adriane Boyd
 * @version 0.1
 */

public class SerEstarEnhancer extends JCasAnnotator_ImplBase {
	private static final Logger log =
		Logger.getLogger(SerEstarEnhancer.class);

	private static final Set<String> estarTags = new HashSet<String>(Arrays.asList("VEadj", "VEfin", "VEger", "VEinf"));
	private static final Set<String> serTags = new HashSet<String>(Arrays.asList("VSadj", "VSfin", "VSger", "VSinf"));
	
	// including forms because TreeTagger seems to miss some subjunctive forms,
	// TODO: figure out what TreeTagger is actually doing
	private static final Set<String> estarForms = new HashSet<String>(
			Arrays.asList("estar", "estado", "estando", "estoy", "estás",
					"está", "estamos", "estáis", "están", "estuve",
					"estuviste", "estuvo", "estuvimos", "estuvisteis",
					"estuvieron", "estaba", "estabas", "estaba", "estábamos",
					"estabais", "estaban", "estaré", "estarás", "estará",
					"estaremos", "estaréis", "estarán", "estaría", "estarías",
					"estaría", "estaríamos", "estaríais", "estarían", "esté",
					"estés", "esté", "estemos", "estéis", "estén", "estuviera",
					"estuviese", "estuvieras", "estuvieses", "estuviera",
					"estuviese", "estuviéramos", "estuviésemos", "estuvierais",
					"estuvieseis", "estuvieran", "estuviesen", "estuviere",
					"estuvieres", "estuviere", "estuviéremos", "estuviereis",
					"estuvieren"));
	private static final Set<String> serForms = new HashSet<String>(
			Arrays.asList("ser", "sido", "siendo", "soy", "eres", "es",
					"somos", "sois", "son", "fui", "fuiste", "fue", "fuimos",
					"fuisteis", "fueron", "era", "eras", "era", "éramos",
					"erais", "eran", "seré", "serás", "será", "seremos",
					"seréis", "serán", "sería", "serías", "sería", "seríamos",
					"seríais", "serían", "sea", "seas", "sea", "seamos",
					"seáis", "sean", "fuera", "fuese", "fueras", "fueses",
					"fuera", "fuese", "fuéramos", "fuésemos", "fuerais",
					"fueseis", "fueran", "fuesen", "fuere", "fueres", "fuere",
					"fuéremos", "fuereis", "fueren"));
	
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
		super.initialize(context);	
	}

	/**
	 * Iterate over all tokens and put a span around them. If a token matches one of the
	 * ser/estar POS tags, mark with a class.
	 *
	 */
	@Override
	@SuppressWarnings("unchecked")
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		int id = 0;
		log.debug("Starting enhancement");

		final FSIndex textIndex = cas.getAnnotationIndex(Token.type);
		final Iterator<Token> tit = textIndex.iterator();

		Token t;

		while (tit.hasNext()) {
			t = tit.next();
			// enhance all non-punctuation tokens
			if (t.getCoveredText().matches(".*[^\\p{P}].*")) {
				final Enhancement e = new Enhancement(cas);
				e.setBegin(t.getBegin());
				e.setEnd(t.getEnd());

				id++;
				final String hitClass;

				if (t.getTag() == null) {
					log.debug("Encountered token with NULL tag");
					hitClass = null;
				} else if (serTags.contains(t.getTag())) {
					hitClass = "wertiviewSer";
				} else if (estarTags.contains(t.getTag())) {
					hitClass = "wertiviewEstar";
				} else if (t.getTag().matches("^VL.*") && serForms.contains(t.getCoveredText().toLowerCase())) {
					hitClass = "wertiviewSer";
				} else if (t.getTag().matches("^VL.*") && estarForms.contains(t.getCoveredText().toLowerCase())) {
					hitClass = "wertiviewEstar";
				} else {
					hitClass = null;
				}

				if (hitClass != null) {
					e.setRelevant(true);
				} else {
					e.setRelevant(false);
				}
				e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span", id) 
						+ "\" class=\"wertiviewtoken " + hitClass + " wertiview" + t.getTag() + "\">");
				e.setEnhanceEnd("</span>");

				if (log.isTraceEnabled()) {
					log.trace("Enhanced " + t.getCoveredText()
							+ " with tag "
							+ t.getTag()
							+ " with id "
							+ id);
				}
				e.addToIndexes();
			
			}
		}
		log.debug("Finished enhancement");
	}

	
}
