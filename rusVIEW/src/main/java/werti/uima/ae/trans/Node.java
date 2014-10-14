package werti.uima.ae.trans;

import werti.uima.types.annot.Token;

public class Node extends Segment {
	public final Token token;
	private final String surface;
	public Node(final Token t) { this(t, t.getCoveredText()); }

	public Node(final Token t, final String overrideSurface) {
		super(t.getBegin(), t.getEnd());
		this.token = t;
		this.surface = overrideSurface;
	}

	@Override
	public String realize() { return this.surface; }
}
