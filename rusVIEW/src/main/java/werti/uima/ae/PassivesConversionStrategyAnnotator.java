package werti.uima.ae;

import java.util.Iterator;

import org.apache.log4j.Logger;

import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;

import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;

import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;
import werti.util.CasUtils;

public class PassivesConversionStrategyAnnotator extends JCasAnnotator_ImplBase {
	private static final Logger log = Logger
			.getLogger(PassivesConversionStrategyAnnotator.class);

	@SuppressWarnings("unchecked")
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		
		log.debug("Starting passives pattern recognition.");
		final AnnotationIndex sind = cas
				.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tind = cas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sind.iterator();

		while (sit.hasNext()) {
			final SentenceAnnotation s = sit.next();
			final Iterator<Token> tit = tind.subiterator(s);
			if (auxPassRecognizer.recognize(tit)) {
				s.setPassiveConversionStrategy("auxpass");
				s.setParseCandidate(true);
				log.debug("Found passive sentence: " + s.getCoveredText());
			} else {
				log.debug("Does not appear to be passive :"
						+ s.getCoveredText());
				s.setParseCandidate(false);
			}
		}
		log.debug("Finished passives pattern recognition.");
	}

	private FSA<Token> auxPassRecognizer = new FSA<Token>() {
		final FSAState<Token> start = new FSAState<Token>(false) {
			@Override
			public FSAState<Token> transition(final Token t) {
				if (t.getTag().matches("^VB[DZ]"))
					return readMainVerb;
				else
					return this;
			}
		};
		final FSAState<Token> readMainVerb = new FSAState<Token>(false) {
			@Override
			public FSAState<Token> transition(final Token t) {
				if (t.getTag().equals("VBN"))
					return readBy;
				else
					return this;
			}
		};
		final FSAState<Token> readBy = new FSAState<Token>(false) {
			@Override
			public FSAState<Token> transition(final Token t) {
				if (t.getCoveredText().toLowerCase().equals("by"))
					return bliss;
				else
					return this;
			}
		};

		private FSAState<Token> bliss = new FSAState<Token>(true);
		private FSAState<Token> doom = new FSAState<Token>(false);

		{
			currentState = start;
		} // instance initialization
	};

	private class FSA<T> {
		protected FSAState<T> currentState;

		public void transition(T t) {
			currentState = currentState.transition(t);
		}

		public boolean accepts() {
			return currentState.accepts;
		}

		public boolean recognize(final Iterator<T> tit) {
			while (tit.hasNext()) {
				transition(tit.next());
			}
			return accepts();
		}
	}

	private class FSAState<T> {
		final boolean accepts;

		public FSAState(boolean accepts_) {
			this.accepts = accepts_;
		}

		public FSAState<T> transition(final T t) {
			return this;
		}
	}
}
