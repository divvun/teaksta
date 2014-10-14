package werti.uima.ae.trans;

import java.util.List;

import werti.util.Functional.Function;

public class Sentence {
	public final Span[] spans;

	public Sentence(List<Span> spans_) {
		spans = new Span[spans_.size()];
		{ int i = 0; for(final Span s:spans_) { spans[i++] = s; } }
	}

	public void applyToSubspans(final Function<Span,Span> f) {
		for (int i = 1; i < spans.length; i++) {
			spans[i] = f.apply(spans[i]);
		}
	}
}
