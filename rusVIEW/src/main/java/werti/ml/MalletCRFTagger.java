/* Copyright (C) 2003 University of Pennsylvania.
  This file is part of "MALLET" (MAchine Learning for LanguagE Toolkit).
http://www.cs.umass.edu/~mccallum/mallet
This software is provided under the terms of the Common Public License,
version 1.0, as published by http://www.opensource.org. For further
information, see the file `LICENSE' included with this distribution. */

package werti.ml;

import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Pattern;

import org.apache.log4j.Logger;

import cc.mallet.fst.CRF;
import cc.mallet.fst.MaxLatticeDefault;
import cc.mallet.fst.Transducer;
import cc.mallet.pipe.Pipe;
import cc.mallet.types.InstanceList;
import cc.mallet.types.Sequence;

/**
 * This class provides a wrapper for applying a CRF-based tagger to a sequence
 * of tokens. It is a highly simplified version of SimpleTagger from Mallet,
 * with all file handling and options removed.
 * 
 * It expects to get a CRF model file and input data as a String with one line
 * per token, each line containing a list of space-separated features.
 * 
 * See: http://mallet.cs.umass.edu/sequences.php
 * 
 * @author Fernando Pereira <a
 *         href="mailto:pereira@cis.upenn.edu">pereira@cis.upenn.edu</a>
 * @author Adriane Boyd
 * @version 1.0
 */
public class MalletCRFTagger {
	
	private static final Logger logger = Logger.getLogger(MalletCRFTagger.class);

	private Pipe p = null;
	private CRF crf = null;

	// TODO: add try/catch with appropriate error codes
	public MalletCRFTagger(String modelFile)
			throws FileNotFoundException, IOException, ClassNotFoundException {
		ObjectInputStream s = new ObjectInputStream(new FileInputStream(
				modelFile));
		crf = (CRF) s.readObject();
		s.close();
		p = crf.getInputPipe();
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
	public Sequence[] apply(Transducer model, Sequence input, int k) {
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

	/**
	 * Wrapper to run a generic CRF-based tagger.
	 * 
	 * @exception Exception
	 *                if an error occurs
	 */
	public List<String> tag(String testString) {
		InstanceList testData = null;
		List<String> tags = new ArrayList<String>();

		this.p.setTargetProcessing(false);
		testData = new InstanceList(this.p);
		testData.addThruPipe(new LineGroupIteratorFromString(testString,
				Pattern.compile("^\\s*$"), true));

		for (int i = 0; i < testData.size(); i++) {
			Sequence input = (Sequence) testData.get(i).getData();
			Sequence[] outputs = apply(this.crf, input, 1);
			int k = outputs.length;
			boolean error = false;
			for (int a = 0; a < k; a++) {
				if (outputs[a].size() != input.size()) {
					System.err.println("Failed to decode input sequence " + i
							+ ", answer " + a);
					error = true;
				}
			}
			if (!error) {
				for (int j = 0; j < input.size(); j++) {
					tags.add(outputs[0].get(j).toString());
				}
			}
		}

		return tags;
	}
}