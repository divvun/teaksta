package werti.ml.fe;

import java.util.ArrayList;
import java.util.List;
import werti.uima.types.annot.Token;

/**
 * Extract features (mostly derived from Baldwin and Bond (2003)) for
 * use in noun countability classification.
 * 
 * @author Adriane Boyd
 *
 */
public class PassiveByPhraseFeatureExtractor implements FeatureExtractor {
	
	public PassiveByPhraseFeatureExtractor() {
		// does this need to to anything?
	}
	
	/**
	 * 
	 * @param tokenlist	list of tokens from cas
	 * @param sparse	whether to return sparse features or only
	 * @param sep		string used to separate features
	 * @param tags		regex string to match POS tags to extract
	 * 					features for
	 * @return one string for each token annotated
	 */
	@Override
	public List<String> extract(List<Token> tokenlist, boolean sparse, String sep, String posFilter) {
		List<String> features = new ArrayList<String>();
		
		List<String> tokens = new ArrayList<String>();
		List<String> tags = new ArrayList<String>();
		List<String> chunks = new ArrayList<String>();
		List<String> lemmas = new ArrayList<String>();
	
		for(Token t : tokenlist) {
			tokens.add(t.getCoveredText());
			tags.add(t.getTag());
			chunks.add(t.getChunk());
			lemmas.add(t.getLemma());
		}
		
		for (int i = 0; i < tokenlist.size(); i++) {
			Token t = tokenlist.get(i);
			
			String sparseFeatures = "";
			String allFeatures = "";

			if (!(t.getTag().matches(posFilter))) {
				continue;
			}
			
			boolean endOfChunk = true;
			boolean singular = true;
			Token nextT = null;
			
			if (i + 1 < tokenlist.size()) {
				if (tokenlist.get(i + 1).getChunk().charAt(0) == 'I') {
					endOfChunk = false;
				}
				nextT = tokenlist.get(i + 1);
			}
			
			// noun number (to get better stats)
			if (singular) {
				allFeatures += "SG" + sep;
				sparseFeatures += "NOUNNUM=SG" + sep;
			} else {
				allFeatures += "PL" + sep;
				sparseFeatures += "NOUNNUM=PL" + sep;
			}
			
			// head noun number (if noun == lemma, then singular, otherwise plural)
			// when noun is head noun (at right boundary of noun chunk)
			if (endOfChunk) {
				if (singular) {
					allFeatures += "SG" + sep;
					sparseFeatures += "HEADNOUNNUM=SG" + sep;
				} else {
					allFeatures += "PL" + sep;
					sparseFeatures += "HEADNOUNNUM=PL" + sep;
				}
			} else {
				allFeatures += "-" + sep;
			}
			
			// modifier noun number (if noun == lemma, then singular, otherwise plural)
			// when noun is modifier noun (not at right boundary of noun chunk)
			if (!endOfChunk){ 
				if (singular) {
					allFeatures += "SG" + sep;
					sparseFeatures += "MODNOUNNUM=PL" + sep;
				} else {
					allFeatures += "PL" + sep;
					sparseFeatures += "MODNOUNNUM=PL" + sep;
				}
			} else {
				allFeatures += "-" + sep;
			}
			
			// subject-verb agreement if the next token starts a VP
			if (endOfChunk && nextT != null && nextT.getChunk().equals("B-VP")) {
				String subjNum = singular ? "SG" : "PL";
				String verbNum = "UNK";
				if (nextT.getTag().equals("VBZ")) {
					verbNum = "SG";
				} else if (nextT.getTag().equals("VBP")) {
					verbNum = "PL";
				}

				allFeatures += subjNum + verbNum + sep;
				sparseFeatures += "SUBJVERBAGR=" + subjNum + verbNum + sep;
			} else {
				allFeatures += "-" + sep;
			}
			
			// coordinate noun number
			String coordinateNounNum = "-" + sep + "-" + sep;
			if (endOfChunk && nextT != null && nextT.getTag().equals("CC")) {
				// get last token in next NP chunk
				int j = i + 2;
				Token endOfNextChunkT = null;
				if (j < tokenlist.size()) {
					if (tokenlist.get(j).getChunk().equals("B-NP")) {						
						while (tokenlist.get(j).getChunk().matches("^I.*")) {
							j++;
						}
						endOfNextChunkT = tokenlist.get(j - 1);
					}
				}
				
				if (endOfNextChunkT != null) {	
					String n1num = singular ? "SG" : "PL";
					String n2num = "SG";
					if (endOfNextChunkT.getCoveredText().toLowerCase().equals(endOfNextChunkT.getLemma().toLowerCase())) {
						coordinateNounNum = n1num + sep + n2num + sep;
						sparseFeatures += "COORDNOUNNUM=" + n1num + sep + "COORDNOUNNUM2=" + n2num + sep;
					}
				}
			}
			allFeatures += coordinateNounNum;
			
			// N of N constructions
			String nOfN = "-" + sep;
			// look for "of" to the left
			if (i - 1 >= 0 && tokenlist.get(i - 1).getLemma().toLowerCase().equals("of")) {
				if (i - 2 >= 0) {
					Token n1 = tokenlist.get(i - 2);
					// need to put n1 by lemma into a generic class (11?)
					nOfN = n1.getLemma().toUpperCase() + "_" + t.getLemma().toUpperCase() + sep;
					sparseFeatures += "NOFN=" + n1.getLemma().toUpperCase() + "_" + t.getLemma().toUpperCase() + sep;
				}
			}
			allFeatures += nOfN;
			
			// occurrence in PPs:
			// look for a preposition to the left as the beginning of a PP
			String occurInPP = "-" + sep;
			if (singular) {
				Token prep = null;
				String detPresent = "NODET";
				// if there are parts of this NP to the left, look for a det
				int j = i - 1;
				if (t.getChunk().equals("I-NP")) {
					// look for preceeding determiner
					while (j >= 0 && !tokenlist.get(j).getChunk().equals("B-NP")) {
						// in case the det isn't the first word in the NP
						if (tokenlist.get(j).getTag().equals("DT")) {
							detPresent = "DET";
						}
						j--;
					}
					// when the det is the first word in the NP
					if (j >= 0 && tokenlist.get(j).getTag().matches("DT")) {
						detPresent = "DET";
					}
					j--;
				}
				
				// if this is a PP, then the previous word should be a prep
				if (j >= 0 && tokenlist.get(j).getTag().equals("IN") && tokenlist.get(j).getChunk().equals("B-PP")) {
					prep = tokenlist.get(j);
				}
				
				if (prep != null) {
					occurInPP = prep.getLemma().toUpperCase() + "_" + detPresent + sep;
					sparseFeatures += "PPOCCUR=" + prep.getLemma().toUpperCase() + "_" + detPresent + sep;
				}
			}
			allFeatures += occurInPP;
			
			if (sparse) {
				features.add(sparseFeatures);
			} else {
				features.add(allFeatures);
			}
			
		}
		
		return features;
	}
}
