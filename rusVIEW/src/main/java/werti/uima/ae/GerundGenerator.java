package werti.uima.ae;

import java.util.Iterator;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;

import werti.uima.ae.util.Morphg;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

/**
 * 
 * @author adriane
 *
 */
public class GerundGenerator extends JCasAnnotator_ImplBase {
	
	private static final Logger log = Logger.getLogger(GerundGenerator.class);
	
	private String morphgLoc;
	private String morphgVerbstemLoc;
	
	/* (non-Javadoc)
	 * @see org.apache.uima.analysis_component.AnalysisComponent_ImplBase#initialize(org.apache.uima.UimaContext)
	 */
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
		super.initialize(context);
		
		morphgLoc = (String) context.getConfigParameterValue("morphgLoc");
		morphgVerbstemLoc = (String) context.getConfigParameterValue("morphgVerbstemLoc");
	}

	
	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas jcas) throws AnalysisEngineProcessException{
		// stop processing if the client has requested it
		if (!CasUtils.isValid(jcas)) {
			return;
		}
		
		log.debug("Starting gerund generation");
		
		final AnnotationIndex tokenIndex = jcas.getAnnotationIndex(Token.type);
				
		Iterator<Token> tit = tokenIndex.iterator();
		
		String input = ""; 

		while (tit.hasNext()) {
			Token t = tit.next();
			if (t.getTag() != null && t.getTag().matches("gv|vb")) {
				input = input.concat(t.getCoveredText() + "+ing_" + t.getTag() + "\t");
			} else {
				input = input.concat(t.getCoveredText() + "_" + t.getTag() + "\t");
			}
		}

		Morphg mg = new Morphg(morphgLoc, morphgVerbstemLoc);
		
		String output = mg.process(input);

		String[] outputParts = output.split("\\t");

		tit = tokenIndex.iterator();

		int i = 0;
		while (tit.hasNext()) {
			Token t = tit.next();
			if (t.getTag() != null && t.getTag().matches("be|do|gv|vb")) {
				if (t.getTag().matches("gv|vb")) {
					t.setGerund(outputParts[i]);
				} else if (t.getTag().matches("be")) {
					t.setGerund("being");
				} else if (t.getTag().matches("do")) {
					t.setGerund("doing");
				}
			}

			i++;
		}
		
		log.debug("Finished gerund generation");
	}
}
