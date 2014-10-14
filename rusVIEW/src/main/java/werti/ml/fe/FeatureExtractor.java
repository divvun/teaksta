package werti.ml.fe;

import java.util.List;
import werti.uima.types.annot.Token;

public interface FeatureExtractor {
	public abstract List<String> extract(List<Token> tokenlist, boolean sparse, String sep, String posFilter);
}
