package werti.uima.enhancer;
import java.util.Arrays;
import java.util.List;

import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.UnsupportedEncodingException;
import java.io.Writer;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.StringTokenizer;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.apache.log4j.Logger;
import org.apache.uima.UimaContext;
import org.apache.uima.analysis_component.JCasAnnotator_ImplBase;
import org.apache.uima.analysis_engine.AnalysisEngineProcessException;
import org.apache.uima.cas.FSIterator;
import org.apache.uima.jcas.JCas;
import org.apache.uima.resource.ResourceInitializationException;

import werti.server.WERTiServlet;
import werti.uima.types.Enhancement;
import werti.uima.types.annot.CGReading;
import werti.uima.types.annot.CGToken;
import werti.util.CasUtils;
import werti.util.EnhancerUtils;
import werti.util.StringListIterable;

import werti.util.Constants;
import static java.lang.System.out;

/**
 * The output from the CG3 analysis from {@link werti.ae.Vislcg3Annotator}
 * is being used to enhance spans corresponding to the tags specified by the topic
 * and the activity that was chosen by the user.
 * In this case the topic is North Sámi nouns in plural form, use the patterns
 * in the method process() to extract the correct tokens for enhancement.
 *
 * @author Niels Ott?
 * @author Adriane Boyd
 * @author Heli Uibo
 * @author Eduard Schaf
 *
 */

public class Vislcg3InfiniteVerbEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log = Logger.getLogger(Vislcg3InfiniteVerbEnhancer.class);

	private List<String> infverbTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
	private final String lookupLoc = Constants.lookup_Loc;
  private final String lookupFlags = Constants.lookup_Flags;
	private final String invertedFST = Constants.inverted_FST;
	private final String FST = Constants.an_FST;

	@Override
	public void initialize(UimaContext context)
		throws ResourceInitializationException {
			//Commenting next ln to reduce output in catalina.out
    	//log.info("InfiniteVerb tags "+infverbTags);
			super.initialize(context);
			infverbTags = Arrays.asList(((String)context.getConfigParameterValue("infiniteverbTags")).split(","));
		}

	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
    if (!CasUtils.isValid(cas)) {
	  	return;
		}

		String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
		log.info("Starting InfiniteVerb enhancement "+enhancement_type+".");

		long generatingDistractorsTotalTime = 0;
		final long startTime = System.currentTimeMillis();

    Pattern posPattern = Pattern.compile("V\\+");
    Pattern number_casePattern = Pattern.compile("PrfPrc|VGen|VAbess|Ger|Actio\\+Ess|Inf|ConNeg");

		Map<String, MutableInt> classCounts = new HashMap<String, MutableInt>();

    FSIterator cgTokenIter = cas.getAnnotationIndex(CGToken.type).iterator();

		// get timestamp in milliseconds and use it in the names of the temporary files in order to avoid conflicts between simultaneous users

		long timestamp = System.currentTimeMillis();

		//String cg3GeneratorInputFileLoc = "./output/cg3GeneratorInput"+timestamp+".tmp";
		//String cg3GeneratorOutputFileLoc = "./output/cg3GeneratorOutput"+timestamp+".tmp";
		String cg3GeneratorInputFileLoc = "./output/cg3GeneratorInput"+".tmp";
		String cg3GeneratorOutputFileLoc = "./output/cg3GeneratorOutput"+".tmp";

		//create temporary files for saving cg3 input and output \

		File cg3GeneratorInputFile = new File(cg3GeneratorInputFileLoc);
		File cg3GeneratorOutputFile = new File(cg3GeneratorOutputFileLoc);

		try {
	    cg3GeneratorInputFile.createNewFile();
	    cg3GeneratorOutputFile.createNewFile();

    } catch (IOException e1) {
	    	e1.printStackTrace();
      }

		Map<Word, SpanTag> wordToSpanMap = new HashMap<Word, SpanTag>();

		boolean isMcActivity = enhancement_type.equals("mc");
    boolean isClozeActivity = enhancement_type.equals("cloze");

		try {
			Writer cg3GeneratorInputWriter = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(cg3GeneratorInputFileLoc), "UTF-8"));
	    Writer cg3GeneratorInputWriterCloze = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(cg3GeneratorInputFileLoc), "UTF-8"));

	    // go through tokens
	    while (cgTokenIter.hasNext()) {

				CGToken cgt = (CGToken) cgTokenIter.next();

				//String hintTag = "";

				boolean isValidReading = false;
				String reading_str = "";
				String lemma = "";
				// select from all readings the first occurrence that is matching pos and number
				for(int i = 0; i < cgt.getReadings().size(); i++){
			    CGReading currentReading = cgt.getReadings(i);
			    //log.info("This is the lemma =" +reading.getHead());
			    StringListIterable readingIterator = new StringListIterable(currentReading);
			    String currentReadingString = "";
			    for (String rtag : readingIterator) {
						currentReadingString += "+" + rtag;
			    }
			    if(!isValidReading){
						Matcher posMatcher = posPattern.matcher(currentReadingString);
						Matcher numberMatcher = number_casePattern.matcher(currentReadingString);
						if(posMatcher.find() && numberMatcher.find()){
					    isValidReading = true;
					    // remove the first "+" and quotes
					    reading_str = currentReadingString.substring(1).replace("\"", "");
					    //log.info("This reading can be considered=" +currentReadingString);
							// the lemma is the first element of the reading string
							lemma = reading_str.split("\\+")[0];
			    	}
					}
		    }
		    if(isValidReading){
					//Commenting next ln to reduce output in catalina.out
					//log.info("This reading will be used=" +reading_str);
					String distractors = "";
					String lemma_and_analyses = "";

					// id's with the "+" symbol have to be escaped, thats why we use a "-" instead
					String spanReadingString = reading_str.replace("+", "-");
					// The "<" and ">"symbols also cause problems because these are the tag opening / closing symbol.
					spanReadingString = spanReadingString.replace("<" ,"x");
					spanReadingString = spanReadingString.replace(">" ,"y");
					//out.println("spanReadingString_xxx="+spanReadingString);

					MutableInt idCount = classCounts.get(spanReadingString);
					if (idCount == null) {
						classCounts.put(spanReadingString, new MutableInt());
					}
					else {
						idCount.increment();
					}
					// create a word with begin and end of the current CGToken
					Word word = new Word(cgt.getBegin(), cgt.getEnd());
					//out.println("word_xxx:"+word);

					String spanTagStart = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + spanReadingString, classCounts.get(spanReadingString).value) + "\" class=\"wertiviewtoken  wertiviewInfiniteVerb\">";  // was: wertiviewhit

          SpanTag spanTag = new SpanTag(spanTagStart);

          spanTag.addAttribute("lemma", lemma);
					wordToSpanMap.put(word, spanTag);
					//out.println("lemma_xxx:"+lemma);
					//out.println("spanTag_xxx:"+spanTag);
					//out.println("spanTagStart_xxx:"+spanTagStart);


					if (isMcActivity) {
			    	// generate the distractors, with lemma, number and case and save in a file
						distractors = writeMorphologicalForms(reading_str);
						cg3GeneratorInputWriter.write(distractors);
						// write the marker that separates the current distractors from others
						cg3GeneratorInputWriter.write("ñôŃßĘńŠē\n");
						// write the word to the file in order to assign the correct distractors to the correct span
						cg3GeneratorInputWriter.write(word.toString());
					} else if (isClozeActivity) {
							// extract lemma and analyses from reading_str and write to file
							lemma_and_analyses = writeLemmaAndAnalyses(reading_str);
							cg3GeneratorInputWriterCloze.write(lemma_and_analyses);
							// write the marker that separates the current lemma+analyses from others
							cg3GeneratorInputWriterCloze.write("ñôŃßĘńŠē\n");
							// write the word to the file in order to assign the correct distractors to the correct span
							cg3GeneratorInputWriterCloze.write(word.toString());
						} else {
						    // make new enhancement, pass it to the cas
						    Enhancement e = new Enhancement(cas);
						    e.setRelevant(true);
						    e.setBegin(word.getBegin());
						    e.setEnd(word.getEnd());
						    e.setEnhanceStart(spanTag.getSpanTagStart());
						    e.setEnhanceEnd(spanTag.getSpanTagEnd());
						    // update CAS
						    cas.addFsToIndexes(e);
						    //log.info("Enhancement="+e); // testing
						}
		    }
			}

			cg3GeneratorInputWriter.close();
			cg3GeneratorInputWriterCloze.close();

			if(isMcActivity){
		  	// generate distractors only when the activity is "mc" (multiple choice)
		    String[] generationPipeline = {
					"/bin/sh",
					"-c",
          "/bin/cat " + cg3GeneratorInputFileLoc +
          " | " + lookupLoc + " " + lookupFlags + " " + invertedFST +
					" > " + cg3GeneratorOutputFileLoc};

		    log.info("Distractor generation pipeline: "+generationPipeline[2]);

		    final long startTimeGenerator = System.currentTimeMillis();

		    Process process = Runtime.getRuntime().exec(generationPipeline);
		    process.waitFor();
		    generateSpanTagWithDistractors(cas, cg3GeneratorOutputFileLoc, wordToSpanMap);

		    final long endTimeGenerator = System.currentTimeMillis();
		    generatingDistractorsTotalTime += (endTimeGenerator - startTimeGenerator);
			}

			if(isClozeActivity){
		  	// generate possible forms from lemma and analyses
		    String[] generationPipeline = {
					"/bin/sh",
					"-c",
          "/bin/cat " + cg3GeneratorInputFileLoc +
          " | " + lookupLoc + " " + lookupFlags + " " + invertedFST +
					" > " + cg3GeneratorOutputFileLoc};

		    final long startTimeGenerator = System.currentTimeMillis();

		    Process process = Runtime.getRuntime().exec(generationPipeline);
		    process.waitFor();
		    generateSpanTagWithPossibleForms(cas, cg3GeneratorOutputFileLoc, wordToSpanMap);

		    final long endTimeGenerator = System.currentTimeMillis();
		    generatingDistractorsTotalTime += (endTimeGenerator - startTimeGenerator);
			}

			// delete the temporary files
			cg3GeneratorInputFile.delete();
			cg3GeneratorOutputFile.delete();

		} catch (IOException e) {
	    	e.printStackTrace();
			} catch (InterruptedException e) {
	    		e.printStackTrace();
				}
    log.info("Finished InfiniteVerb enhancement.");
    final long endTime = System.currentTimeMillis();

    log.info("Total execution time: " + (endTime - startTime)*0.001 + " seconds." );
    log.info("Generating the distractforms takes in total: " + generatingDistractorsTotalTime * 0.001 + " seconds." );
	}

	/*
	* Create all relevant morphological forms of the current token
	* It is the input for the distractor generation
	*/
  private String writeMorphologicalForms(String reading_str) {

		String[] distract_forms = {"V+Ind+Prs+Sg1", "V+Ind+Prs+Sg2", "V+Ind+Prs+Sg3", "V+Ind+Prs+Du1", "V+Ind+Prs+Du2", "V+Ind+Prs+Du3", "V+Ind+Prt+Sg1", "V+Ind+Prt+Sg2", "V+Ind+Prt+Sg3", "V+Ind+Prt+Du1", "V+Ind+Prt+Du2", "V+Ind+Prt+Du3"};

		String str, word, result = "", generationInput = "";

		String lemma = reading_str.substring(0, reading_str.indexOf("+"));

	 	for (int j=0; j < distract_forms.length; j++) {
	    generationInput += lemma + "+" + distract_forms[j] + "\n";

	    //generationInput += lemma + "+v1+" + distract_forms[j] + "\n";
	 	}

		return generationInput;
  }

	private String writeLemmaAndAnalyses(String reading_str) {

		String lemma_str = reading_str.substring(0, reading_str.indexOf("+"));
		String an_tmp = reading_str.substring(reading_str.indexOf("+")+1, reading_str.length()-1);
		String analyses_str = an_tmp.replace("+<sme>", "");
		analyses_str = analyses_str.substring(0, analyses_str.indexOf("@")-1);
		String lem_and_an = lemma_str + "+" + analyses_str + "\n";

		return lem_and_an;
  }

	private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap){
	 	try {
   		BufferedReader cg3GeneratorOutputReader = new BufferedReader(new InputStreamReader(new FileInputStream(cg3GeneratorOutputFileLoc), "UTF8"));

     	String generatorOutput = "";

     	Word currentWord = new Word();
     	String distractforms = "";

   		while (cg3GeneratorOutputReader.ready()) {
		 		String line = cg3GeneratorOutputReader.readLine().trim();
		 		if(line.isEmpty()){
		     	continue;
		 		}
		 		// generator output was processed, all distractors are created
		 		// assign the distractors to the correct span from the wordToSpanMap
		 		else if(line.startsWith("Word")){
		    	// only enhance tokens with more than one distractor form
	     		if(!distractforms.isEmpty()){
			 			String[] lineParts = line.split("\\s");
						int begin = Integer.parseInt(lineParts[1]);
						int end = Integer.parseInt(lineParts[2]);
						currentWord = new Word(begin, end);
						SpanTag spanTag = wordToSpanMap.get(currentWord);
						//Commenting next ln to reduce output in catalina.out
						//log.info("spantag before adding distractors:"+spanTag);
						spanTag.addAttribute("distractors", distractforms);
						// make new enhancement, pass it to the cas
						Enhancement e = new Enhancement(cas);
						e.setRelevant(true);
						e.setBegin(begin);
						e.setEnd(end);
						e.setEnhanceStart(spanTag.getSpanTagStart());

						e.setEnhanceEnd(spanTag.getSpanTagEnd());
						// update CAS
						cas.addFsToIndexes(e);
						//log.info("Enhancement="+e); // testing
	     		}
		 		}
				// the marker (ñôŃßĘńŠē) was found, begin to process the generator output, create distractors
				else if(line.contains("ñôŃßĘńŠē")){
					StringTokenizer tok = new StringTokenizer(generatorOutput);
					generatorOutput = "";
					String word = "";
					distractforms = "";
					// the distractorsSet's purpose is to filter out duplicates
					HashSet<String> distractorsSet = new HashSet<String>();
					while (tok.hasMoreTokens()) {
				 		word = tok.nextToken();
				 		//log.info("ifst output:"+word);
				 		// forms that could not be generated are excluded, as well as input strings of the iFST
				 		if (!word.contains("+") && !word.contains("-") && distractorsSet.add(word)) {
				     	distractforms += word + " ";
				 		}
				 		else{
			     		//log.info("Word that was excluded = " + word);
				 		}
				  }
					// remove the whitespace at the end
					distractforms = distractforms.trim();
					// exclude the distractor if its only one, you need at least 2 distractors for mc
					if(distractorsSet.size() < 2) {
				 		distractforms = "";
			   	}
					else {
				 		//log.info("This are the chosen distractforms="+distractforms);
			   	}
				}
				// the generator output for the current token is not fully extracted from the file yet
				else{
				  generatorOutput += line + " ";
				}
		 	}

	    cg3GeneratorOutputReader.close();

	 	} catch (UnsupportedEncodingException e1) {
     		e1.printStackTrace();
	 		} catch (FileNotFoundException e1) {
	     		e1.printStackTrace();
	 			} catch (IOException e) {
	     			e.printStackTrace();
	 				}
	}

	private void generateSpanTagWithPossibleForms(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap){
	 	try {
   		BufferedReader cg3GeneratorOutputReader = new BufferedReader(new InputStreamReader(new FileInputStream(cg3GeneratorOutputFileLoc), "UTF8"));

     	String generatorOutput = "";

     	Word currentWord = new Word();
     	String possible_forms = "";

   		while (cg3GeneratorOutputReader.ready()) {
		 		String line = cg3GeneratorOutputReader.readLine().trim();
		 		if(line.isEmpty()){
		     	continue;
		 		}
		 		// generator output was processed, all possible forms are created
		 		// assign the possible forms to the correct span from the wordToSpanMap
		 		else if(line.startsWith("Word")){
	     		if(!possible_forms.isEmpty()){
			 			String[] lineParts = line.split("\\s");
						int begin = Integer.parseInt(lineParts[1]);
						int end = Integer.parseInt(lineParts[2]);
						currentWord = new Word(begin, end);
						SpanTag spanTag = wordToSpanMap.get(currentWord);
						//Commenting next ln to reduce output in catalina.out
						//log.info("spantag before adding forms:"+spanTag);
						spanTag.addAttribute("possibleforms", possible_forms);
						// make new enhancement, pass it to the cas
						Enhancement e = new Enhancement(cas);
						e.setRelevant(true);
						e.setBegin(begin);
						e.setEnd(end);
						e.setEnhanceStart(spanTag.getSpanTagStart());
						e.setEnhanceEnd(spanTag.getSpanTagEnd());
						// update CAS
						cas.addFsToIndexes(e);
						//log.info("Enhancement="+e); // testing
	     		}
		 		}
				// the marker (ñôŃßĘńŠē) was found, begin to process the generator output, create distractors
				else if(line.contains("ñôŃßĘńŠē")){
					StringTokenizer tok = new StringTokenizer(generatorOutput);
					generatorOutput = "";
					String word = "";
					possible_forms = "";
					// the distractorsSet's purpose is to filter out duplicates
					HashSet<String> possible_formsSet = new HashSet<String>();
					while (tok.hasMoreTokens()) {
				 		word = tok.nextToken();
				 		//log.info("ifst output:"+word);
				 		// forms that could not be generated are excluded, as well as input strings of the iFST
				 		if (!word.contains("+") && !word.contains("-") && possible_formsSet.add(word)) {
				     	possible_forms += word + " ";
				 		}
				 		else{
			     		//log.info("Word that was excluded = " + word);
				 		}
				  }
					// remove the whitespace at the end
					possible_forms = possible_forms.trim();
				}
				// the generator output for the current token is not fully extracted from the file yet
				else{
				  generatorOutput += line + " ";
				}
		 	}

	    cg3GeneratorOutputReader.close();

	 	} catch (UnsupportedEncodingException e1) {
     		e1.printStackTrace();
	 		} catch (FileNotFoundException e1) {
	     		e1.printStackTrace();
	 			} catch (IOException e) {
	     			e.printStackTrace();
	 				}
	}

	/**
	 * This class represents a mutable integer value, which is especially useful
	 * and fast for counting frequencies inside a map.
	 *
	 * @author Eduard Schaf
	 *
	 */
	public class MutableInt {
  	int value = 1; // note that we start at 1 since we're counting
    /**
     * Increment the mutable int by one.
     */
    public void increment () {
			++value;
    }
    /**
     * Get the value of the mutable int.
     * @return the mutable int value.
     */
    public int get () {
			return value;
    }
	}
	/**
	 * This class represents a word of two integers
	 * which are begin and end. They are used to store
	 * the offsets of a given Token.
	 *
	 * @author Eduard Schaf
	 *
	 */
	public class Word {
  	private int begin;
    private int end;
    public Word(int begin, int end) {
			this.begin = begin;
			this.end = end;
    }
    public Word() {
			this.begin = 0;
			this.end = 0;
    }
    public int getBegin() {
			return begin;
    }
    public void setBegin(int begin) {
			this.begin = begin;
    }
    public int getEnd() {
			return end;
    }
    public void setEnd(int end) {
			this.end = end;
    }
    @Override
    public String toString() {
	 		return "Word " + begin + " " + end + "\n";
    }
    @Override
    public int hashCode() {
			final int prime = 31;
			int result = 1;
			result = prime * result + getOuterType().hashCode();
			result = prime * result + begin;
			result = prime * result + end;
			return result;
    }
    @Override
    public boolean equals(Object obj) {
		  if (this == obj)
	      return true;
		  if (obj == null)
	      return false;
		  if (getClass() != obj.getClass())
	      return false;
		  Word other = (Word) obj;
		  if (!getOuterType().equals(other.getOuterType()))
	      return false;
		  if (begin != other.begin)
	      return false;
		  if (end != other.end)
	      return false;
		  return true;
    }
    private Vislcg3InfiniteVerbEnhancer getOuterType() {
			return Vislcg3InfiniteVerbEnhancer.this;
    }

	}
	/**
	 * This class represents a SpanTag consisting out of
	 * the span start tag with possibility to add attributes to the span tag
	 * and the span end tag. It is the span surrounding the
	 * token that is being enhanced.
	 *
	 * @author Eduard Schaf
	 *
	 */
	public class SpanTag{
    private String spanTagStart;
    private String spanTagEnd;
    public SpanTag(String spanTagStart) {
			this.spanTagStart = spanTagStart;
			this.spanTagEnd = "</span>";
    }
    public String getSpanTagStart() {
			return spanTagStart;
    }
    public void setSpanTagStart(String spanTagStart) {
			this.spanTagStart = spanTagStart;
    }
    public void addAttribute(String attributeName, String attributeValue) {
			this.spanTagStart = this.spanTagStart.replace(">", attributeName + "=\"" + attributeValue + "\">");
    }
    public String getSpanTagEnd() {
			return spanTagEnd;
    }
    public void setSpanTagEnd(String spanTagEnd) {
			this.spanTagEnd = spanTagEnd;
    }


		@Override
	 	public String toString() {
			return "SpanTag [spanTagStart=" + spanTagStart + ", spanTagEnd=" + spanTagEnd + "]";
   	}
    @Override
	 	public int hashCode() {
     	final int prime = 31;
     	int result = 1;
     	result = prime * result + getOuterType().hashCode();
     	result = prime * result
		  + ((spanTagEnd == null) ? 0 : spanTagEnd.hashCode());
		     result = prime * result
		  + ((spanTagStart == null) ? 0 : spanTagStart.hashCode());
		     return result;
   	}
    @Override
    public boolean equals(Object obj) {
	    if (this == obj)
				return true;
	    if (obj == null)
				return false;
	    if (getClass() != obj.getClass())
				return false;
	    SpanTag other = (SpanTag) obj;
	    if (!getOuterType().equals(other.getOuterType()))
				return false;
	    if (spanTagEnd == null) {
				if (other.spanTagEnd != null)
		    	return false;
	    } else if (!spanTagEnd.equals(other.spanTagEnd))
					return false;
	    if (spanTagStart == null) {
				if (other.spanTagStart != null)
			    return false;
	    } else if (!spanTagStart.equals(other.spanTagStart))
					return false;
		  return true;
    }
    private Vislcg3InfiniteVerbEnhancer getOuterType() {
			return Vislcg3InfiniteVerbEnhancer.this;
    }

	}

}
