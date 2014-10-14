package werti.uima.ae.filter;

import java.util.List;

import werti.uima.ae.PhrasalVerbAnnotator;
import werti.uima.types.annot.Token;

/**
 * The PhrasalVerbFilter checks whether it is worthwhile to parse a
 * sentence that potentially contains a phrasal verb.  Currently
 * unused.
 * 
 * @author Adriane Boyd
 *
 */
public class PhrasalVerbFilter implements Filter {
	private String particleRegex;

	public PhrasalVerbFilter() {
		particleRegex = "";
		for (String p : PhrasalVerbAnnotator.particleList) {
			particleRegex += p + "|";
		}
		particleRegex.substring(0, particleRegex.length() - 2);
		particleRegex = "(" + particleRegex + ")";
		System.out.println(particleRegex);
	}
	
	/**
	 * 
	 * @param tokenlist	list of tokens from cas
	 * @return whether this sentence should be parsed
	 */
	@Override
	public boolean filter(List<Token> tokenlist) {
		// if the sentence is too long, skip
		if (tokenlist.size() > 80) {
			return false;
		}
		
		String lemmaString = "";
		boolean containsPhrasalVerb = false;
		
		for(Token t : tokenlist) {
			if (t.getTag().matches("^V.*") && PhrasalVerbAnnotator.verbList.contains(t.getLemma())) {
				containsPhrasalVerb = true;
			}
			lemmaString += t.getLemma() + " ";
		}
		
		// if there's one possible verb and one possible particle,
		// this sentence makes it past the filter
		if (containsPhrasalVerb) {
			if (lemmaString.matches(".*" + particleRegex + ".*")) {
				return true;
			}
		}

		// didn't have both a verb and a particle
		return true;
	}
}
