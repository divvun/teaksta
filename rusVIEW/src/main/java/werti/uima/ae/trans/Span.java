package werti.uima.ae.trans;

import java.io.BufferedWriter;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;

import java.util.Arrays;
import java.util.Iterator;
import java.util.List;

import org.apache.log4j.Logger;

import org.jgrapht.ext.DOTExporter;
import org.jgrapht.ext.EdgeNameProvider;
import org.jgrapht.ext.IntegerNameProvider;
import org.jgrapht.ext.VertexNameProvider;

public class Span extends Segment implements Iterable<Segment> {
	private static final Logger log = Logger.getLogger(Span.class);

	private Segment[] segments;
	public final WERTiGraph<Segment,Edge> graph;

	@Override
	public Iterator<Segment> iterator() {
		return new Iterator<Segment>() {
			private int pointer = 0;
			@Override
			public boolean hasNext() { return pointer < segments.length; }
			@Override
			public void    remove()  { throw new UnsupportedOperationException(); }
			@Override
			public Segment next()    { if (hasNext()) { return segments[pointer++]; }
			                           else           { return null; } }
		};
	}

	public Span(final List<? extends Segment> c, WERTiGraph<Segment,Edge> g_) {
		super(c.get(0).begin, c.get(c.size()-1).end);
		setSegments(c);
		this.graph = (g_);
	}

	public void setSegments(List<? extends Segment> l) {
		segments = new Segment[l.size()];
		{ int i = 0; for(final Segment s:l) { segments[i++] = s; } }
	}

	public List<Segment> toList() { return Arrays.asList(segments); }

	@Override
	public String realize() {
		final StringBuffer sb = new StringBuffer();
		sb.append("[");
		for (final Segment s:segments) {
			sb.append(s.toString());
			sb.append(", ");
		}
		sb.deleteCharAt(sb.length()-1);
		sb.deleteCharAt(sb.length()-1);
		sb.append("]");
		return sb.toString();
	}

	public void printGraph(final File f) {
		try {
			final BufferedWriter w = new BufferedWriter(new FileWriter(f));
			dotex.export(w,this.graph);
			log.info("Exported graph for "+this.realize()+" to file "+f.toString()+".");
		} catch (IOException ioe) {
			log.warn("Couldn't print graph for "+this.realize()+" to file "+f.toString()+".");
			log.warn(ioe);
		}
	}

	public String renderAsString() {
		final StringBuffer sb = new StringBuffer();
		for (final Segment s:segments) {
			if (s instanceof Span) sb.append(((Span)s).renderAsString()+" ");
			else                   sb.append(s.toString()+" ");
		}
		sb.deleteCharAt(sb.length()-1);
		return sb.toString();
	}

	private static final DOTExporter<Segment,Edge> dotex = new DOTExporter<Segment,Edge>
		( new IntegerNameProvider<Segment>()
		, new VertexNameProvider<Segment>() {
		      @Override
			public String getVertexName(Segment t) { return t.realize(); }
		  }
		, new EdgeNameProvider<Edge>() {
		      @Override
			public String getEdgeName(Edge e) { return e.label; }
		  }
		);

}
