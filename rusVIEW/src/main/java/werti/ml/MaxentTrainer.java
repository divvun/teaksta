package werti.ml;

///////////////////////////////////////////////////////////////////////////////
//Copyright (C) 2001 Chieu Hai Leong and Jason Baldridge
//
//This library is free software; you can redistribute it and/or
//modify it under the terms of the GNU Lesser General Public
//License as published by the Free Software Foundation; either
//version 2.1 of the License, or (at your option) any later version.
//
//This library is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU Lesser General Public License for more details.
//
//You should have received a copy of the GNU Lesser General Public
//License along with this program; if not, write to the Free Software
//Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.
//////////////////////////////////////////////////////////////////////////////

import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.StringReader;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;

import opennlp.maxent.GIS;
import opennlp.maxent.GISModel;
import opennlp.maxent.io.GISModelWriter;
import opennlp.maxent.io.SuffixSensitiveGISModelWriter;

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

/**
 * Main class which calls the GIS procedure after building the EventStream from
 * the data.
 * 
 * Usage (from the deployed WEB-INF directory, which has all the dependencies in
 * one place):
 * 
 * java -cp "./classes:./lib/*" org.werti.ml.MaxentTrainer \
 * /path/to/desc/operators/MLTrainerPipe.xml \ trainingdata.txt model ncoll.txt
 * count.txt
 * 
 * Look for CUSTOMIZE below to see where to make changes to train a different
 * model.
 * 
 * @author Chieu Hai Leong and Jason Baldridge
 * @author Adriane Boyd
 */
public class MaxentTrainer {

	// some parameters if you want to play around with the smoothing option
	// for model training. This can improve model accuracy, though training
	// will potentially take longer and use more memory. Model size will also
	// be larger. Initial testing indicates improvements for models built on
	// small data sets and few outcomes, but performance degradation for those
	// with large data sets and lots of outcomes.
	public static boolean USE_SMOOTHING = false;
	public static double SMOOTHING_OBSERVATION = 0.1;

	private static void usage() {
		System.err
				.println("java MaxentTrainer descriptorFile dataFile modelFile countFileName noncountFileName bothFileName");
		System.exit(1);
	}

	/**
	 * Trains on plain text in provided file and writes to model.
	 * 
	 * @throws Exception
	 */
	@SuppressWarnings("unchecked")
	public static void main(String[] args) throws Exception {
		String descriptorFileName = null, dataFileName = null, modelFileName = null, countFileName = null, noncountFileName = null, bothFileName = null;
		int ai = 0;

		try {
			descriptorFileName = new String(args[ai++]);
			dataFileName = new String(args[ai++]);
			modelFileName = new String(args[ai++]);
			countFileName = new String(args[ai++]);
			noncountFileName = new String(args[ai++]);
			bothFileName = new String(args[ai++]);
		} catch (Exception e) {
			usage();
		}
		
		Set<String> countNouns = new TreeSet<String>();
		BufferedReader bufferedFile = new BufferedReader(new FileReader(
				new File(countFileName)));
		String line;
		while ((line = bufferedFile.readLine()) != null) {
			countNouns.add(line);
		}
		
		Set<String> noncountNouns = new TreeSet<String>();
		bufferedFile = new BufferedReader(new FileReader(
				new File(noncountFileName)));
		while ((line = bufferedFile.readLine()) != null) {
			noncountNouns.add(line);
		}

		Set<String> bothNouns = new TreeSet<String>();
		bufferedFile = new BufferedReader(new FileReader(
				new File(bothFileName)));
		while ((line = bufferedFile.readLine()) != null) {
			bothNouns.add(line);
		}

		File descriptor = new File(descriptorFileName);
		XMLInputSource xmlInput = new XMLInputSource(descriptor);
		AnalysisEngineDescription description = UIMAFramework.getXMLParser()
				.parseAnalysisEngineDescription(xmlInput);

		// read descriptor from disk and initialize a new annotator
		AnalysisEngine ae = UIMAFramework.produceAnalysisEngine(description);

		bufferedFile = new BufferedReader(
				new FileReader(new File(dataFileName)));
		String trainingInput = ""; // input to be processed by UIMA
		String trainingString = ""; // input to be processed by maxent

		// assume we have a large file with one doc per line for training
		int outputCounter = 1;
		while ((line = bufferedFile.readLine()) != null) {
			System.err.println("Processing line " + outputCounter++ + " "
					+ line.length());

			// skip empty lines because UIMA can't handle them
			if (line.trim().length() == 0) {
				continue;
			}

			trainingInput = "<e>" + line + "</e>";

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

			// CUSTOMIZE: choose the right feature extractor
			FeatureExtractor fe = new NounCountabilityFeatureExtractor();

			// loop through the doc by sentence
			final AnnotationIndex sentIndex = cas
					.getAnnotationIndex(SentenceAnnotation.type);
			final AnnotationIndex tokenIndex = cas
					.getAnnotationIndex(Token.type);

			final Iterator<SentenceAnnotation> sit = sentIndex.iterator();

			while (sit.hasNext()) {
				final Iterator<Token> tit = tokenIndex.subiterator(sit.next());
				List<Token> tokenList = new ArrayList<Token>();

				while (tit.hasNext()) {
					Token t = tit.next();
					tokenList.add(t);
				}
				List<String> featuresList = fe.extract(tokenList, false, ",", "^NNS?$");

				assert tokenList.size() == featuresList.size();

				// CUSTOMIZE: filter the results as needed for the activity
				int i = 0;
				for (int j = 0; j < tokenList.size(); j++) {
					Token t = tokenList.get(j);
					if (t.getTag().matches("^NNS?$")) {
						String trainingLine = "";
						trainingLine += featuresList.get(i);
						trainingString += featuresList.get(i);

						if (bothNouns.contains(t.getLemma())) {
							trainingString += " BOTH";
							trainingLine += "BOTH";
						} else if (countNouns.contains(t.getLemma())) {
							trainingString += " COUNT";
							trainingLine += "COUNT";
						} else if (noncountNouns.contains(t.getLemma())) {
							trainingString += " NONCOUNT";
							trainingLine += "NONCOUNT";
						} else {
							trainingString += " UNKNOWN";
							trainingLine += "UNKNOWN";
						}

						trainingString += "\n";
						System.out.println(trainingLine);
						
						i++;
					}
				}
			}
		}

		// TODO: add an option that prints this to a file
		/*
		 * System.out.println("--------------------------------------------");
		 * System.out.println(trainingString);
		 * System.out.println("--------------------------------------------");
		 */

		try {
			StringReader datasr = new StringReader(trainingString);
			GIS.SMOOTHING_OBSERVATION = SMOOTHING_OBSERVATION;
			GISModel model = GIS.trainModel(null, USE_SMOOTHING);

			File outputFile = new File(modelFileName);
			GISModelWriter writer = new SuffixSensitiveGISModelWriter(model,
					outputFile);
			writer.persist();
		} catch (Exception e) {
			System.out.print("Unable to create model due to exception: ");
			System.out.println(e);
			e.printStackTrace();
		}
	}
}
