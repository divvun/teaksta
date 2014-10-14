package werti.uima.ae.filter;

import java.util.List;

import werti.uima.types.annot.Token;

/**
 * The PassiveSentenceFilter decides whether it is worthwhile to parse a 
 * sentence in anticipation of performing an active/passive transformation.
 * 
 * @author Adriane Boyd
 *
 */
public class AlwaysFilter implements Filter {
	public AlwaysFilter() { }
	
	/**
	 * 
	 * @param tokenlist	list of tokens from cas
	 * @return whether this sentence should be parsed for use in the activity
	 */
	@Override
	public boolean filter(List<Token> tl) {
		return true;
	}
}
