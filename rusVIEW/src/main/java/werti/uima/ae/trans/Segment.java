package werti.uima.ae.trans;

public abstract class Segment {
	public final int begin;
	public final int end;

	public Segment(final int begin_, final int end_) {
		this.begin = begin_;
		this.end   = end_;
	}

	public abstract String realize();
	@Override
	public String toString() { return realize(); }
}
