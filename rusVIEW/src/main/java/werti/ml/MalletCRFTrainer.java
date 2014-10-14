/* Copyright (C) 2003 University of Pennsylvania.
  This file is part of "MALLET" (MAchine Learning for LanguagE Toolkit).
http://www.cs.umass.edu/~mccallum/mallet
This software is provided under the terms of the Common Public License,
version 1.0, as published by http://www.opensource.org. For further
information, see the file `LICENSE' included with this distribution. */

package werti.ml;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.FileReader;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Reader;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Random;
import java.util.regex.Pattern;

import org.apache.log4j.Logger;
import org.apache.uima.UIMAFramework;
import org.apache.uima.analysis_engine.AnalysisEngine;
import org.apache.uima.analysis_engine.AnalysisEngineDescription;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;
import org.apache.uima.util.XMLInputSource;
import werti.ml.fe.FeatureExtractor;
import werti.ml.fe.NounCountabilityFeatureExtractor;
import werti.uima.types.annot.SentenceAnnotation;
import werti.uima.types.annot.Token;

import cc.mallet.fst.CRF;
import cc.mallet.fst.CRFTrainerByLabelLikelihood;
import cc.mallet.fst.MaxLatticeDefault;
import cc.mallet.fst.MultiSegmentationEvaluator;
import cc.mallet.fst.NoopTransducerTrainer;
import cc.mallet.fst.SimpleTagger;
import cc.mallet.fst.TokenAccuracyEvaluator;
import cc.mallet.fst.Transducer;
import cc.mallet.fst.TransducerEvaluator;
import cc.mallet.fst.TransducerTrainer;
import cc.mallet.pipe.Pipe;
import cc.mallet.pipe.iterator.LineGroupIterator;
import cc.mallet.types.Alphabet;
import cc.mallet.types.FeatureVector;
import cc.mallet.types.FeatureVectorSequence;
import cc.mallet.types.Instance;
import cc.mallet.types.InstanceList;
import cc.mallet.types.LabelAlphabet;
import cc.mallet.types.LabelSequence;
import cc.mallet.types.Sequence;
import cc.mallet.util.CommandOption;

/**
 * This class provides a wrapper for training a CRF-based tagger.  It is adapted
 * from SimpleTagger from Mallet.
 * 
 * It expects to get as input a file containing plain text and does all the
 * necessary feature extraction.
 * 
 * Usage (from the deployed WEB-INF directory, which has all the dependencies 
 * in one place):
 * 
 * java -cp "./classes:./lib/*" werti.ml.MalletCRFTrainer --train true \ 
 *   --descriptor /path/to/desc/operators/MLTrainerPipe.xml \ 
 *   --model-file /path/to/mallet.model /path/to/trainingdata.txt
 *  
 * See also: http://mallet.cs.umass.edu/sequences.php
 * 
 * @author Fernando Pereira <a
 *         href="mailto:pereira@cis.upenn.edu">pereira@cis.upenn.edu</a>
 * @author Adriane Boyd
 */
public class MalletCRFTrainer {
	
	private static final Logger logger = Logger.getLogger(MalletCRFTrainer.class);

	private MalletCRFTrainer() {
	}

	/**
	 * Converts an external encoding of a sequence of elements with binary
	 * features to a {@link FeatureVectorSequence}. If target processing is on
	 * (training or labeled test data), it extracts element labels from the
	 * external encoding to create a target {@link LabelSequence}. Two external
	 * encodings are supported:
	 * <ol>
	 * <li>A {@link String} containing lines of whitespace-separated tokens.</li>
	 * <li>a {@link String}<code>[][]</code>.</li>
	 * </ol>
	 * 
	 * Both represent rows of tokens. When target processing is on, the last
	 * token in each row is the label of the sequence element represented by
	 * this row. All other tokens in the row, or all tokens in the row if not
	 * target processing, are the names of features that are on for the sequence
	 * element described by the row.
	 * 
	 */
	public static class SimpleTaggerSentence2FeatureVectorSequence extends Pipe {
		/**
		 * Creates a new <code>SimpleTaggerSentence2FeatureVectorSequence</code>
		 * instance.
		 */
		public SimpleTaggerSentence2FeatureVectorSequence() {
			super(new Alphabet(), new LabelAlphabet());
		}

		/**
		 * Parses a string representing a sequence of rows of tokens into an
		 * array of arrays of tokens.
		 * 
		 * @param sentence
		 *            a <code>String</code>
		 * @return the corresponding array of arrays of tokens.
		 */
		private String[][] parseSentence(String sentence) {
			String[] lines = sentence.split("\n");
			String[][] tokens = new String[lines.length][];
			for (int i = 0; i < lines.length; i++)
				tokens[i] = lines[i].split(" ");
			return tokens;
		}

		@Override
		public Instance pipe(Instance carrier) {
			Object inputData = carrier.getData();
			Alphabet features = getDataAlphabet();
			LabelAlphabet labels;
			LabelSequence target = null;
			String[][] tokens;
			if (inputData instanceof String)
				tokens = parseSentence((String) inputData);
			else if (inputData instanceof String[][])
				tokens = (String[][]) inputData;
			else
				throw new IllegalArgumentException(
						"Not a String or String[][]; got " + inputData);
			FeatureVector[] fvs = new FeatureVector[tokens.length];
			if (isTargetProcessing()) {
				labels = (LabelAlphabet) getTargetAlphabet();
				target = new LabelSequence(labels, tokens.length);
			}
			for (int l = 0; l < tokens.length; l++) {
				int nFeatures;
				if (isTargetProcessing()) {
					if (tokens[l].length < 1)
						throw new IllegalStateException(
								"Missing label at line " + l + " instance "
										+ carrier.getName());
					nFeatures = tokens[l].length - 1;
					target.add(tokens[l][nFeatures]);
				} else
					nFeatures = tokens[l].length;
				int featureIndices[] = new int[nFeatures];
				for (int f = 0; f < nFeatures; f++)
					featureIndices[f] = features.lookupIndex(tokens[l][f]);
				fvs[l] = new FeatureVector(features, featureIndices);
			}
			carrier.setData(new FeatureVectorSequence(fvs));
			if (isTargetProcessing())
				carrier.setTarget(target);
			else
				carrier.setTarget(new LabelSequence(getTargetAlphabet()));
			return carrier;
		}
	}

	/**
	 * Create and train a CRF model from the given training data, optionally
	 * testing it on the given test data.
	 * 
	 * @param training
	 *            training data
	 * @param testing
	 *            test data (possibly <code>null</code>)
	 * @param eval
	 *            accuracy evaluator (possibly <code>null</code>)
	 * @param orders
	 *            label Markov orders (main and backoff)
	 * @param defaultLabel
	 *            default label
	 * @param forbidden
	 *            regular expression specifying impossible label transitions
	 *            <em>current</em><code>,</code><em>next</em> (<code>null</code>
	 *            indicates no forbidden transitions)
	 * @param allowed
	 *            regular expression specifying allowed label transitions (
	 *            <code>null</code> indicates everything is allowed that is not
	 *            forbidden)
	 * @param connected
	 *            whether to include even transitions not occurring in the
	 *            training data.
	 * @param iterations
	 *            number of training iterations
	 * @param var
	 *            Gaussian prior variance
	 * @return the trained model
	 */
	public static CRF train(InstanceList training, InstanceList testing,
			TransducerEvaluator eval, int[] orders, String defaultLabel,
			String forbidden, String allowed, boolean connected,
			int iterations, double var, CRF crf) {
		Pattern forbiddenPat = Pattern.compile(forbidden);
		Pattern allowedPat = Pattern.compile(allowed);
		if (crf == null) {
			crf = new CRF(training.getPipe(), (Pipe) null);
			String startName = crf.addOrderNStates(training, orders, null,
					defaultLabel, forbiddenPat, allowedPat, connected);
			for (int i = 0; i < crf.numStates(); i++)
				crf.getState(i).setInitialWeight(Transducer.IMPOSSIBLE_WEIGHT);
			crf.getState(startName).setInitialWeight(0.0);
		}
		CRFTrainerByLabelLikelihood crft = new CRFTrainerByLabelLikelihood(crf);
		crft.setGaussianPriorVariance(var);
		boolean converged;
		for (int i = 1; i <= iterations; i++) {
			converged = crft.train(training, 1);
			if (i % 1 == 0 && eval != null) // Change the 1 to higher
				// integer to evaluate less
				// often
				eval.evaluate(crft);
			if (converged)
				break;
		}

		return crf;
	}

	/**
	 * Apply a transducer to an input sequence to produce the k highest-scoring
	 * output sequences.
	 * 
	 * @param model
	 *            the <code>Transducer</code>
	 * @param input
	 *            the input sequence
	 * @param k
	 *            the number of answers to return
	 * @return array of the k highest-scoring output sequences
	 */
	public static Sequence[] apply(Transducer model, Sequence input, int k) {
		Sequence[] answers;
		if (k == 1) {
			answers = new Sequence[1];
			answers[0] = model.transduce(input);
		} else {
			MaxLatticeDefault lattice = new MaxLatticeDefault(model, input,
					null, 100000); // cacheSizeOption.value());

			answers = lattice.bestOutputSequences(k).toArray(new Sequence[0]);
		}
		return answers;
	}
	
	private static final CommandOption.Double gaussianVarianceOption = new CommandOption.Double(
			SimpleTagger.class, "gaussian-variance", "DECIMAL", true, 10.0,
			"The gaussian prior variance used for training.", null);

	private static final CommandOption.Boolean trainOption = new CommandOption.Boolean(
			SimpleTagger.class, "train", "true|false", true, false,
			"Whether to train", null);

	private static final CommandOption.String testOption = new CommandOption.String(
			SimpleTagger.class,
			"test",
			"lab or seg=start-1.continue-1,...,start-n.continue-n",
			true,
			null,
			"Test measuring labeling or segmentation (start-i, continue-i) accuracy",
			null);

	private static final CommandOption.File modelOption = new CommandOption.File(
			SimpleTagger.class,
			"model-file",
			"FILENAME",
			true,
			null,
			"The filename for reading (train/run) or saving (train) the model.",
			null);

	private static final CommandOption.Double trainingFractionOption = new CommandOption.Double(
			SimpleTagger.class, "training-proportion", "DECIMAL", true, 0.5,
			"Fraction of data to use for training in a random split.", null);

	private static final CommandOption.Integer randomSeedOption = new CommandOption.Integer(
			SimpleTagger.class,
			"random-seed",
			"INTEGER",
			true,
			0,
			"The random seed for randomly selecting a proportion of the instance list for training",
			null);

	private static final CommandOption.IntegerArray ordersOption = new CommandOption.IntegerArray(
			SimpleTagger.class, "orders", "COMMA-SEP-DECIMALS", true,
			new int[] { 1 }, "List of label Markov orders (main and backoff) ",
			null);

	private static final CommandOption.String forbiddenOption = new CommandOption.String(
			SimpleTagger.class, "forbidden", "REGEXP", true, "\\s",
			"label1,label2 transition forbidden if it matches this", null);

	private static final CommandOption.String allowedOption = new CommandOption.String(
			SimpleTagger.class, "allowed", "REGEXP", true, ".*",
			"label1,label2 transition allowed only if it matches this", null);

	private static final CommandOption.String defaultOption = new CommandOption.String(
			SimpleTagger.class, "default-label", "STRING", true, "O",
			"Label for initial context and uninteresting tokens", null);

	private static final CommandOption.Integer iterationsOption = new CommandOption.Integer(
			SimpleTagger.class, "iterations", "INTEGER", true, 500,
			"Number of training iterations", null);

	private static final CommandOption.Boolean viterbiOutputOption = new CommandOption.Boolean(
			SimpleTagger.class, "viterbi-output", "true|false", true, false,
			"Print Viterbi periodically during training", null);

	private static final CommandOption.Boolean connectedOption = new CommandOption.Boolean(
			SimpleTagger.class, "fully-connected", "true|false", true, true,
			"Include all allowed transitions, even those not in training data",
			null);

	private static final CommandOption.Boolean continueTrainingOption = new CommandOption.Boolean(
			SimpleTagger.class, "continue-training", "true|false", false,
			false, "Continue training from model specified by --model-file",
			null);

	private static final CommandOption.Integer nBestOption = new CommandOption.Integer(
			SimpleTagger.class, "n-best", "INTEGER", true, 1,
			"How many answers to output", null);

	private static final CommandOption.Integer cacheSizeOption = new CommandOption.Integer(
			SimpleTagger.class, "cache-size", "INTEGER", true, 100000,
			"How much state information to memoize in n-best decoding", null);

	private static final CommandOption.Boolean includeInputOption = new CommandOption.Boolean(
			SimpleTagger.class,
			"include-input",
			"true|false",
			true,
			false,
			"Whether to include the input features when printing decoding output",
			null);

	private static final CommandOption.Boolean featureInductionOption = new CommandOption.Boolean(
			SimpleTagger.class, "feature-induction", "true|false", true, false,
			"Whether to perform feature induction during training", null);

	private static final CommandOption.String descriptorOption = new CommandOption.String(
			SimpleTagger.class,
			"descriptor",
			"/path/to/descriptor.xml",
			true,
			null,
			"Provide path to UIMA descriptor for preprocessing",
			null);
	
	private static final CommandOption.List commandOptions = new CommandOption.List(
			"Training, testing and running a generic tagger.",
			new CommandOption[] { gaussianVarianceOption, trainOption,
					iterationsOption, testOption, trainingFractionOption,
					modelOption, randomSeedOption, ordersOption,
					forbiddenOption, allowedOption, defaultOption,
					viterbiOutputOption, connectedOption,
					continueTrainingOption, nBestOption, cacheSizeOption,
					includeInputOption, featureInductionOption, descriptorOption, });

	/**
	 * Test a transducer on the given test data, evaluating accuracy with the
	 * given evaluator
	 * 
	 * @param model
	 *            a <code>Transducer</code>
	 * @param eval
	 *            accuracy evaluator
	 * @param testing
	 *            test data
	 */
	public static void test(TransducerTrainer tt, TransducerEvaluator eval,
			InstanceList testing) {
		eval.evaluateInstanceList(tt, testing, "Testing");
	}

	/**
	 * Command-line wrapper to train, test, or run a generic CRF-based tagger.
	 * 
	 * @param args
	 *            the command line arguments. Options (shell and Java quoting
	 *            should be added as needed):
	 *            <dl>
	 *            <dt><code>--help</code> <em>boolean</em></dt>
	 *            <dd>Print this command line option usage information. Give
	 *            <code>true</code> for longer documentation. Default is
	 *            <code>false</code>.</dd>
	 *            <dt><code>--prefix-code</code> <em>Java-code</em></dt>
	 *            <dd>Java code you want run before any other interpreted code.
	 *            Note that the text is interpreted without modification, so
	 *            unlike some other Java code options, you need to include any
	 *            necessary 'new's. Default is null.</dd>
	 *            <dt><code>--gaussian-variance</code> <em>positive-number</em></dt>
	 *            <dd>The Gaussian prior variance used for training. Default is
	 *            10.0.</dd>
	 *            <dt><code>--train</code> <em>boolean</em></dt>
	 *            <dd>Whether to train. Default is <code>false</code>.</dd>
	 *            <dt><code>--iterations</code> <em>positive-integer</em></dt>
	 *            <dd>Number of training iterations. Default is 500.</dd>
	 *            <dt><code>--test</code> <code>lab</code> or <code>seg=</code>
	 *            <em>start-1</em><code>.</code><em>continue-1</em>
	 *            <code>,</code>...<code>,</code><em>start-n</em><code>.</code>
	 *            <em>continue-n</em></dt>
	 *            <dd>Test measuring labeling or segmentation (<em>start-i</em>,
	 *            <em>continue-i</em>) accuracy. Default is no testing.</dd>
	 *            <dt><code>--training-proportion</code>
	 *            <em>number-between-0-and-1</em></dt>
	 *            <dd>Fraction of data to use for training in a random split.
	 *            Default is 0.5.</dd>
	 *            <dt><code>--model-file</code> <em>filename</em></dt>
	 *            <dd>The filename for reading (train/run) or saving (train) the
	 *            model. Default is null.</dd>
	 *            <dt><code>--random-seed</code> <em>integer</em></dt>
	 *            <dd>The random seed for randomly selecting a proportion of the
	 *            instance list for training Default is 0.</dd>
	 *            <dt><code>--orders</code> <em>comma-separated-integers</em></dt>
	 *            <dd>List of label Markov orders (main and backoff) Default is
	 *            1.</dd>
	 *            <dt><code>--forbidden</code> <em>regular-expression</em></dt>
	 *            <dd>If <em>label-1</em><code>,</code><em>label-2</em> matches
	 *            the expression, the corresponding transition is forbidden.
	 *            Default is <code>\\s</code> (nothing forbidden).</dd>
	 *            <dt><code>--allowed</code> <em>regular-expression</em></dt>
	 *            <dd>If <em>label-1</em><code>,</code><em>label-2</em> does not
	 *            match the expression, the corresponding expression is
	 *            forbidden. Default is <code>.*</code> (everything allowed).</dd>
	 *            <dt><code>--default-label</code> <em>string</em></dt>
	 *            <dd>Label for initial context and uninteresting tokens.
	 *            Default is <code>O</code>.</dd>
	 *            <dt><code>--viterbi-output</code> <em>boolean</em></dt>
	 *            <dd>Print Viterbi periodically during training. Default is
	 *            <code>false</code>.</dd>
	 *            <dt><code>--fully-connected</code> <em>boolean</em></dt>
	 *            <dd>Include all allowed transitions, even those not in
	 *            training data. Default is <code>true</code>.</dd>
	 *            <dt><code>--n-best</code> <em>positive-integer</em></dt>
	 *            <dd>Number of answers to output when applying model. Default
	 *            is 1.</dd>
	 *            <dt><code>--include-input</code> <em>boolean</em></dt>
	 *            <dd>Whether to include input features when printing decoding
	 *            output. Default is <code>false</code>.</dd>
	 *            </dl>
	 *            Remaining arguments:
	 *            <ul>
	 *            <li><em>training-data-file</em> if training</li>
	 *            <li><em>training-and-test-data-file</em>, if training and
	 *            testing with random split</li>
	 *            <li><em>training-data-file</em> <em>test-data-file</em> if
	 *            training and testing from separate files</li>
	 *            <li><em>test-data-file</em> if testing</li>
	 *            <li><em>input-data-file</em> if applying to new data
	 *            (unlabeled)</li>
	 *            </ul>
	 * @exception Exception
	 *                if an error occurs
	 */
	@SuppressWarnings("unchecked")
	public static void main(String[] args) throws Exception {
		Reader trainingFile = null, testFile = null;
		InstanceList trainingData = null, testData = null;
		int restArgs = commandOptions.processOptions(args);
		if (restArgs == args.length) {
			commandOptions.printUsage(true);
			throw new IllegalArgumentException("Missing data file(s)");
		}
		if (trainOption.value) {
			trainingFile = new FileReader(new File(args[restArgs]));
			if (testOption.value != null && restArgs < args.length - 1)
				testFile = new FileReader(new File(args[restArgs + 1]));
		} else
			testFile = new FileReader(new File(args[restArgs]));

		Pipe p = null;
		CRF crf = null;
		TransducerEvaluator eval = null;
		if (continueTrainingOption.value || !trainOption.value) {
			if (modelOption.value == null) {
				commandOptions.printUsage(true);
				throw new IllegalArgumentException("Missing model file option");
			}
			ObjectInputStream s = new ObjectInputStream(new FileInputStream(
					modelOption.value));
			crf = (CRF) s.readObject();
			s.close();
			p = crf.getInputPipe();
		} else {
			p = new SimpleTaggerSentence2FeatureVectorSequence();
			p.getTargetAlphabet().lookupIndex(defaultOption.value);
		}
		
		BufferedReader bufferedFile = new BufferedReader(trainingFile);
		String trainingInput = "<e>";
		String line;
		while ((line = bufferedFile.readLine()) != null) {
			trainingInput += line + "\n";
		}
		trainingInput += "</e>";
		
		File descriptor = new File(descriptorOption.value);
		XMLInputSource xmlInput = new XMLInputSource(descriptor);
		AnalysisEngineDescription description = UIMAFramework.getXMLParser().parseAnalysisEngineDescription(xmlInput);

		// read descriptor from disk and initialize a new annotator
		AnalysisEngine ae = UIMAFramework.produceAnalysisEngine(description);
		
		JCas cas = null;
		try { // to process
			cas = ae.newJCas();
			cas.setDocumentText(trainingInput);
			ae.process(cas);
		} catch (AnalysisEngineProcessException aepe) {
			throw new Exception("Text analysis failed.", aepe);
		} catch (ResourceInitializationException rie) {
			throw new Exception("Text analysis failed.", rie);
		}
		
		// TODO: filter to only train on sentences that contain nouns with known categories
		FeatureExtractor fe = new NounCountabilityFeatureExtractor();
		String trainingString = "";
		
		final AnnotationIndex sentIndex = cas.getAnnotationIndex(SentenceAnnotation.type);
		final AnnotationIndex tokenIndex = cas.getAnnotationIndex(Token.type);

		final Iterator<SentenceAnnotation> sit = sentIndex.iterator();

		while (sit.hasNext()) {

			final Iterator<Token> tit = tokenIndex.subiterator(sit.next());
			List<Token> tokenlist = new ArrayList<Token>();
		
			while (tit.hasNext()) {
				Token t = tit.next();
				tokenlist.add(t);			
			}
			List<String> featuresList = fe.extract(tokenlist, true, " ", "^NN$");
		
			
			for (int i = 0; i < featuresList.size(); i++) {
				trainingString += featuresList.get(i) + "\n";
			}
		}
		
		if (trainOption.value) {
			p.setTargetProcessing(true);
			trainingData = new InstanceList(p);
			trainingData.addThruPipe(new LineGroupIteratorFromString(trainingString,
					Pattern.compile("^\\s*$"), true));
			logger.info("Number of features in training data: "
					+ p.getDataAlphabet().size());
			if (testOption.value != null) {
				if (testFile != null) {
					testData = new InstanceList(p);
					testData.addThruPipe(new LineGroupIterator(testFile,
							Pattern.compile("^\\s*$"), true));
				} else {
					Random r = new Random(randomSeedOption.value);
					InstanceList[] trainingLists = trainingData.split(r,
							new double[] { trainingFractionOption.value,
									1 - trainingFractionOption.value });
					trainingData = trainingLists[0];
					testData = trainingLists[1];
				}
			}
		} else if (testOption.value != null) {
			p.setTargetProcessing(true);
			testData = new InstanceList(p);
			testData.addThruPipe(new LineGroupIterator(testFile, Pattern
					.compile("^\\s*$"), true));
		} else {
			p.setTargetProcessing(false);
			testData = new InstanceList(p);
			testData.addThruPipe(new LineGroupIterator(testFile, Pattern
					.compile("^\\s*$"), true));
		}
		logger.info("Number of predicates: " + p.getDataAlphabet().size());

		if (testOption.value != null) {
			if (testOption.value.startsWith("lab"))
				eval = new TokenAccuracyEvaluator(new InstanceList[] {
						trainingData, testData }, new String[] { "Training",
						"Testing" });
			else if (testOption.value.startsWith("seg=")) {
				String[] pairs = testOption.value.substring(4).split(",");
				if (pairs.length < 1) {
					commandOptions.printUsage(true);
					throw new IllegalArgumentException(
							"Missing segment start/continue labels: "
									+ testOption.value);
				}
				String startTags[] = new String[pairs.length];
				String continueTags[] = new String[pairs.length];
				for (int i = 0; i < pairs.length; i++) {
					String[] pair = pairs[i].split("\\.");
					if (pair.length != 2) {
						commandOptions.printUsage(true);
						throw new IllegalArgumentException(
								"Incorrectly-specified segment start and end labels: "
										+ pairs[i]);
					}
					startTags[i] = pair[0];
					continueTags[i] = pair[1];
				}
				eval = new MultiSegmentationEvaluator(new InstanceList[] {
						trainingData, testData }, new String[] { "Training",
						"Testing" }, startTags, continueTags);
			} else {
				commandOptions.printUsage(true);
				throw new IllegalArgumentException("Invalid test option: "
						+ testOption.value);
			}
		}

		if (p.isTargetProcessing()) {
			Alphabet targets = p.getTargetAlphabet();
			StringBuffer buf = new StringBuffer("Labels:");
			for (int i = 0; i < targets.size(); i++)
				buf.append(" ").append(targets.lookupObject(i).toString());
			logger.info(buf.toString());
		}
		if (trainOption.value) {
			crf = train(trainingData, testData, eval, ordersOption.value,
					defaultOption.value, forbiddenOption.value,
					allowedOption.value, connectedOption.value,
					iterationsOption.value, gaussianVarianceOption.value, crf);
			if (modelOption.value != null) {
				ObjectOutputStream s = new ObjectOutputStream(
						new FileOutputStream(modelOption.value));
				s.writeObject(crf);
				s.close();
			}
		} else {
			if (crf == null) {
				if (modelOption.value == null) {
					commandOptions.printUsage(true);
					throw new IllegalArgumentException(
							"Missing model file option");
				}
				ObjectInputStream s = new ObjectInputStream(
						new FileInputStream(modelOption.value));
				crf = (CRF) s.readObject();
				s.close();
			}
			if (eval != null)
				test(new NoopTransducerTrainer(crf), eval, testData);
			else {
				boolean includeInput = includeInputOption.value();
				for (int i = 0; i < testData.size(); i++) {
					Sequence input = (Sequence) testData.get(i).getData();
					Sequence[] outputs = apply(crf, input, nBestOption.value);
					int k = outputs.length;
					boolean error = false;
					for (int a = 0; a < k; a++) {
						if (outputs[a].size() != input.size()) {
							System.err
									.println("Failed to decode input sequence "
											+ i + ", answer " + a);
							error = true;
						}
					}
					if (!error) {
						for (int j = 0; j < input.size(); j++) {
							StringBuffer buf = new StringBuffer();
							for (int a = 0; a < k; a++)
								buf.append(outputs[a].get(j).toString())
										.append(" ");
							if (includeInput) {
								FeatureVector fv = (FeatureVector) input.get(j);
								buf.append(fv.toString(true));
							}
							System.out.println(buf.toString());
						}
						System.out.println();
					}
				}
			}
		}
	}

}