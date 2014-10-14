package werti.uima.enhancer;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileReader;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;

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
 * Given lists of count, noncount, and both nouns stored in resource
 * files, enhance single nouns according to these lists and use NP
 * chunk information to enhance noun-noun compounds according to the 
 * final noun in the compound.
 * 
 * @author Adriane Boyd
 *
 */
public class NounCountEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		Logger.getLogger(NounCountEnhancer.class);

	private String countFileName, noncountFileName, bothFileName, filter;

	private Set<String> countNouns = new TreeSet<String>();
	private Set<String> noncountNouns = new TreeSet<String>();
	private Set<String> bothNouns = new TreeSet<String>();
	private Set<String> knownNouns = new TreeSet<String>();

	@Override
	public void initialize(UimaContext context)
	throws ResourceInitializationException {
		super.initialize(context);

		countFileName = (String) context.getConfigParameterValue("countFile");
		noncountFileName = (String) context.getConfigParameterValue("noncountFile");
		bothFileName = (String) context.getConfigParameterValue("bothFile");
		filter = (String) context.getConfigParameterValue("posFilter");

		try {
			BufferedReader bufferedFile = new BufferedReader(new FileReader(
					new File(countFileName)));
			String line;
			while ((line = bufferedFile.readLine()) != null) {
				countNouns.add(line);
			}

			bufferedFile = new BufferedReader(new FileReader(
					new File(noncountFileName)));
			while ((line = bufferedFile.readLine()) != null) {
				noncountNouns.add(line);
			}

			bufferedFile = new BufferedReader(new FileReader(
					new File(bothFileName)));
			while ((line = bufferedFile.readLine()) != null) {
				bothNouns.add(line);
			}

			knownNouns.addAll(countNouns);
			knownNouns.addAll(noncountNouns);
			knownNouns.addAll(bothNouns);

		} catch (FileNotFoundException e) {
			throw new ResourceInitializationException();
		} catch (IOException e) {
			throw new ResourceInitializationException();
		}
	}

	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		log.debug("Starting count enhancement");
		int id = 0;

		final FSIndex textIndex = cas.getAnnotationIndex(Token.type);
		final Iterator<Token> tit = textIndex.iterator();

		Token t; // token pointer
		List<Token> chunk = new ArrayList<Token>();
		boolean inNPChunk = false;

		while (tit.hasNext()) {
			t = tit.next();

			// build up NP chunks
			if (t.getChunk() != null && t.getChunk().equals("B-NP")) {
				chunk.clear();
				chunk.add(t);
				inNPChunk = true;
			} else if (inNPChunk && t.getChunk() != null && t.getChunk().equals("I-NP")) {
				chunk.add(t);
			} else if (inNPChunk) {
				id = enhanceNPChunk(chunk, cas, id);
				inNPChunk = false;
			}
		}

		log.debug("Finished count enhancement");
	}

	/**
	 * 
	 * @param chunk a list of tokens in an NP chunk
	 * @param cas the CAS
	 * @param id used to create unique span ids
	 * @return
	 */
	private int enhanceNPChunk(List<Token> chunk, JCas cas, int id) {
		// use classification of final token in chunk to classify all tokens in noun-noun compounds
		Token finalToken = chunk.get(chunk.size() - 1);
		
		String hit = null;
		if (finalToken.getTag().matches("NNS")) {
			hit = "COUNT";
		} else if (knownNouns.contains(finalToken.getCoveredText().toLowerCase())) {
			if (bothNouns.contains(finalToken.getCoveredText().toLowerCase())) {
				hit = "BOTH";
			} else if (countNouns.contains(finalToken.getCoveredText().toLowerCase())) {
				hit = "COUNT";
			} else {
				hit = "NONCOUNT";
			}
		}
		
		// starting at the right hand chunk boundary, enhance the final nouns and 
		// any preceding noun sequence with this class
		for (int i = chunk.size() - 1; i >= 0; i--) {
			Token t = chunk.get(i);

			// enhance all nouns that aren't punctuation
			if (t.getTag() != null && t.getTag().matches(filter) && t.getCoveredText().matches(".*[^\\p{P}].*")) {
				final Enhancement e = new Enhancement(cas);
				e.setBegin(t.getBegin());
				e.setEnd(t.getEnd());

				id++;

				String hitclass = "";
				if (hit != null) {
					hitclass = "wertiview" + hit;
					e.setRelevant(true);
				} else {
					e.setRelevant(false);
				}

				e.setEnhanceStart("<span id=\"" + EnhancerUtils.get_id("WERTi-span", id) 
						+ "\" class=\"wertiviewtoken " + hitclass + "\">");
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

			// if this isn't a noun, stop enhancing within this chunk
			if (t.getTag() != null && !t.getTag().matches(filter)) {
				break;
			}
		}
		
		return id;
	}
}