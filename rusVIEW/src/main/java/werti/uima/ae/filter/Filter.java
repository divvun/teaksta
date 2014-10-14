package werti.uima.ae.filter;

import java.util.List;

import werti.uima.types.annot.Token;

public interface Filter {
	public abstract boolean filter(List<Token> tokenlist);
}
