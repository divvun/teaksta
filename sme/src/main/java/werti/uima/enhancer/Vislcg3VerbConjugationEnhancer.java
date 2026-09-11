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

import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.LogManager;
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
import org.apache.commons.lang.StringUtils;

/**
 * The output from the CG3 analysis from {@link werti.ae.Vislcg3Annotator}
 * is being used to enhance spans corresponding to the tags specified by the topic
 * and the activity that was chosen by the user.
 * In this case the topic is North Sámi verbs in finite forms, use the patterns
 * in the method process() to extract the correct tokens for enhancement.
 *
 * @author Niels Ott?
 * @author Adriane Boyd
 * @author Heli Uibo
 * @author Eduard Schaf
 *
 */

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer]
public class Vislcg3VerbConjugationEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log = LogManager.GetLogger(Vislcg3VerbConjugationEnhancer.class);

	private List<String> FinVerbTags;
	private static String CHUNK_BEGIN_SUFFIX = "-B";
	private static String CHUNK_INSIDE_SUFFIX = "-I";
	private final String lookupLoc = Constants.lookup_Loc;
  private final String lookupFlags = Constants.lookup_Flags;
	private final String invertedFST = Constants.inverted_FST;
	private final String FST = Constants.an_FST;

	//list of tags to be removed from analyses because these are not present in generator-norm
	String[] tags_tbr = {
		"+Err/Orth",
		"+Err/Orth-a-á",
		"+Err/Orth-nom-gen",
		"+Err/Orth-nom-acc",
		"+Err/CmpSub",
		"+Err/MissingSpace",
		"+Err/MissingHyph",
		"+Err/Hyph",
		"+Err/SpaceCmp",
		"+Err/Spellrelax",
		"+Allegro",
		//this is a regex to find all possible tags of the type: <xxx_xxx>
		"\\+<([a-zA-Z]*+_*+)*+>"
	};

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.initialize-fn]
	@Override
	public void initialize(UimaContext context)
		throws ResourceInitializationException {
			//Commenting next ln to reduce output in catalina.out
      //log.info("VerbConjugation tags {}", FinVerbTags);
			super.initialize(context);
			FinVerbTags = Arrays.asList(((String)context.getConfigParameterValue("finverbTags")).split(","));
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+7]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.process-fn+7]
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
    // stop processing if the client has requested it
    if (!CasUtils.isValid(cas)) {
 			return;
    }

		String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
		log.info("Starting VerbConjugation enhancement {}.", enhancement_type);

		long generatingDistractorsTotalTime = 0;
		final long startTime = System.currentTimeMillis();

    Pattern posPattern = Pattern.compile("V\\+");
		//Pattern number_casePattern = Pattern.compile("Ind\\+Prs|Ind\\+Prt|Imprt|Cond\\+Prs|Pot\\+Prs|Neg\\+Ind");
    Pattern number_casePattern = Pattern.compile("Sg1|Sg2|Sg3|Du1|Du2|Du3|Pl1|Pl2|Pl3");

		Map<String, MutableInt> classCounts = new HashMap<String, MutableInt>();

    FSIterator cgTokenIter = cas.getAnnotationIndex(CGToken.type).iterator();

		// get timestamp in milliseconds and use it in the names of the temporary files in order to avoid conflicts between simultaneous users

		long timestamp = System.currentTimeMillis();

		//String cg3GeneratorInputFileLoc = "./output/cg3GeneratorInput"+timestamp+".tmp";
		//String cg3GeneratorOutputFileLoc = "./output/cg3GeneratorOutput"+timestamp+".tmp";
		String cg3GeneratorInputFileLoc = Constants.cg3GeneratorInputFile_Loc;
		String cg3GeneratorOutputFileLoc = Constants.cg3GeneratorOutputFile_Loc;

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
			    //log.info("This is the lemma ={}", reading.getHead());
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
					    //log.info("This reading can be considered={}", currentReadingString);
							// the lemma is the first element of the reading string
							lemma = reading_str.split("\\+")[0];
						}
					}
		    }
		    if(isValidReading){
					//Commenting next ln to reduce output in catalina.out
					//log.info("This reading will be used={}", reading_str);
					String distractors = "";
					String lemma_and_analyses = "";

					// id's with the "+" symbol have to be escaped, thats why we use a "-" instead
					String spanReadingString = reading_str.replace("+", "-");
					// The "<" and ">"symbols also cause problems because these are the tag opening / closing symbol.
					spanReadingString = spanReadingString.replace("<" ,"x");
					spanReadingString = spanReadingString.replace(">" ,"y");

					MutableInt idCount = classCounts.get(spanReadingString);
					if (idCount == null) {
				    classCounts.put(spanReadingString, new MutableInt());
					}
					else {
					  idCount.increment();
					}
					// create a word with begin and end of the current CGToken
					Word word = new Word(cgt.getBegin(), cgt.getEnd());

					String spanTagStart = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + spanReadingString, classCounts.get(spanReadingString).value) + "\" class=\"wertiviewtoken  wertiviewVerbConjugation\">";  // was: wertiviewhit

          SpanTag spanTag = new SpanTag(spanTagStart);

          spanTag.addAttribute("lemma", lemma);
					wordToSpanMap.put(word, spanTag);

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
						    //log.info("Enhancement={}",e); // testing
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

		    log.info("Distractor generation pipeline: {}", generationPipeline[2]);

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
    log.info("Finished VerbConjugation enhancement.");
    final long endTime = System.currentTimeMillis();

    log.info("Total execution time: {} seconds.", (endTime - startTime)*0.001);
    log.info("Generating the distractforms takes in total: {} seconds.", generatingDistractorsTotalTime * 0.001);
 	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2]
	private String removeTags(String input_str) {
		for (int h=0; h<tags_tbr.length; h++) {
			if (h<tags_tbr.length-1) {
				if (input_str.contains(tags_tbr[h])) {
					input_str = input_str.replace(tags_tbr[h],"");
				}
			} else {
				Pattern myPattern = Pattern.compile(tags_tbr[h]);
				Matcher myMatcher = myPattern.matcher(input_str);
				if (myMatcher.find()) {
					String mytag = myMatcher.group(0);
					input_str = input_str.replace(mytag,"");
				}
			}
		}
		return input_str;
	}

    /*
    * Create all relevant morphological forms of the current token
    * It is the input for the distractor generation
    */
 	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2]
 	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-morphological-forms-fn+2]
 	private String writeMorphologicalForms(String reading_str) {

		String[] distract_forms_ind = {
			"V+Ind+Prs+ConNeg",
			"V+Ind+Prt+ConNeg",
			"V+Actio+Ess",
			"V+Ind+Prt+Sg1",
			"V+Ind+Prt+Sg2",
			"V+Ind+Prt+Sg3",
			"V+Ind+Prs+Sg1",
			"V+Ind+Prs+Sg2",
			"V+Ind+Prs+Sg3",
			"V+Ind+Prt+Du1",
			"V+Ind+Prt+Du2",
			"V+Ind+Prt+Du3",
			"V+Ind+Prs+Du1",
			"V+Ind+Prs+Du2",
			"V+Ind+Prs+Du3"
		};

		String[] distract_forms_imprt = {
			"V+Imprt+Sg1",
			"V+Imprt+Sg2",
			"V+Imprt+Sg3",
			"V+Imprt+Du1",
			"V+Imprt+Du2",
			"V+Imprt+Du3",
			"V+Imprt+Pl1",
			"V+Imprt+Pl2",
			"V+Imprt+Pl3"
		};

		String[] distract_forms_cond = {
			"V+Cond+Prs+Sg1",
			"V+Cond+Prs+Sg2",
			"V+Cond+Prs+Sg3",
			"V+Cond+Prs+Du1",
			"V+Cond+Prs+Du2",
			"V+Cond+Prs+Du3",
			"V+Cond+Prs+Pl1",
			"V+Cond+Prs+Pl2",
			"V+Cond+Prs+Pl3"
		};

		String[] distract_forms_pot = {
			"V+Pot+Prs+Sg1",
			"V+Pot+Prs+Sg2",
			"V+Pot+Prs+Sg3",
			"V+Pot+Prs+Du1",
			"V+Pot+Prs+Du2",
			"V+Pot+Prs+Du3",
			"V+Pot+Prs+Pl1",
			"V+Pot+Prs+Pl2",
			"V+Pot+Prs+Pl3"
		};

		String[] distract_forms_neg = {
			"V+Neg+Ind+Sg1",
			"V+Neg+Ind+Sg2",
			"V+Neg+Ind+Sg3",
			"V+Neg+Ind+Du1",
			"V+Neg+Ind+Du2",
			"V+Neg+Ind+Du3",
			"V+Neg+Ind+Pl1",
			"V+Neg+Ind+Pl2",
			"V+Neg+Ind+Pl3"
		};

		String[] distract_forms = {"", "", "", "", ""};

		String[] tag = new String[20];
		int i = 0;

		StringTokenizer tok = new StringTokenizer(reading_str,"+");
		while (tok.hasMoreTokens()) {
   		tag[i] = tok.nextToken();
   		i++;
		}

		String lemma = tag[0];
		String pos = tag[1];
		String trans = tag[2];
		String mood = tag[3];
		String tense = tag[4];
		String person = tag[5];
		/*
		//check parts
		log.info("reading_str={}", reading_str);
		if (!(tense.equals("Prs")||tense.equals("Prt"))) {
			person = tense;
			tense = "";
		}
		//log.info("lemma={}, pos={}, trans={}, mood={}, tense={}, person={}, lemma, pos, trans, mood, tense, person);
		*/

		String generationInput = "";

		if (mood.equals("Ind")) {
			for (int j=0; j < distract_forms_ind.length; j++) {
		    generationInput += lemma + "+" + distract_forms_ind[j] + "\n";
		 	}
		}
		if (mood.equals("Imprt")) {
			for (int j=0; j < distract_forms_imprt.length; j++) {
		    generationInput += lemma + "+" + distract_forms_imprt[j] + "\n";
		 	}
		}
		if (mood.equals("Cond")) {
			for (int j=0; j < distract_forms_cond.length; j++) {
		    generationInput += lemma + "+" + distract_forms_cond[j] + "\n";
		 	}
		}
		if (mood.equals("Pot")) {
			for (int j=0; j < distract_forms_pot.length; j++) {
		    generationInput += lemma + "+" + distract_forms_pot[j] + "\n";
		 	}
		}
		if (mood.equals("Neg")) {
			for (int j=0; j < distract_forms_neg.length; j++) {
		    generationInput += lemma + "+" + distract_forms_neg[j] + "\n";
		 	}
		}

		//log.info("generation input:{}", generationInput);

		//add reading_str as last element in generationInput which will be used as correct_answer
		//remove @ only if it is in reading_str (otherwise get "String index out of range" error)
		if (reading_str.indexOf("@")>0) {
			generationInput += reading_str.substring(0, reading_str.indexOf("@")-1)+"\n";
		} else {
			generationInput += reading_str+"\n";
		}

		//if generationInput contains tags_tbr, remove it
		generationInput = removeTags(generationInput);

		return generationInput;
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.write-lemma-and-analyses-fn+2]
	private String writeLemmaAndAnalyses(String reading_str) {

		String lemma_str = reading_str.substring(0, reading_str.indexOf("+"));
		String an_tmp = reading_str.substring(reading_str.indexOf("+")+1, reading_str.length()-1);
		String analyses_str = an_tmp.replace("+<sme>", "");
		//remove @ only if it is in analyses_str (otherwise get "String index out of range" error)
		if (analyses_str.indexOf("@")>0) {
			analyses_str = analyses_str.substring(0, analyses_str.indexOf("@")-1);
		}
		String lem_and_an = lemma_str + "+" + analyses_str + "\n";

		//if analyses contains tags_tbr, remove it
		lem_and_an = removeTags(lem_and_an);

		return lem_and_an;
  }

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3]
	private void generateSpanTagWithDistractors(JCas cas, String cg3GeneratorOutputFileLoc, Map<Word, SpanTag> wordToSpanMap){
		try {
			BufferedReader cg3GeneratorOutputReader = new BufferedReader(new InputStreamReader(new FileInputStream(cg3GeneratorOutputFileLoc), "UTF8"));

			String generatorOutput = "";

			Word currentWord = new Word();
			String distractforms = "";
			String[] splitted_go = {""};

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
						//log.info("spantag before adding distractors:{}", spanTag);
						spanTag.addAttribute("distractors", distractforms);
						spanTag.addAttribute("answer", splitted_go[splitted_go.length-1]);
						// make new enhancement, pass it to the cas
						Enhancement e = new Enhancement(cas);
						e.setRelevant(true);
						e.setBegin(begin);
						e.setEnd(end);
						e.setEnhanceStart(spanTag.getSpanTagStart());

						e.setEnhanceEnd(spanTag.getSpanTagEnd());
						// update CAS
						cas.addFsToIndexes(e);
						//log.info("Enhancement={}",e); // testing
			   	}
				}
				// the marker (ñôŃßĘńŠē) was found, begin to process the generator output, create distractors
				else if(line.contains("ñôŃßĘńŠē")){
					StringTokenizer tok = new StringTokenizer(generatorOutput);
					splitted_go = generatorOutput.split("\\s");
					generatorOutput = "";
					String word = "";
					distractforms = "";
					// the distractorsSet's purpose is to filter out duplicates
					HashSet<String> distractorsSet = new HashSet<String>();
					while (tok.hasMoreTokens()) {
				 		word = tok.nextToken();
						//log.info("ifst output:{}", word);
						// forms that could not be generated are excluded, as well as input strings of the iFST
						if (!word.contains("+") && !word.contains("-") && distractorsSet.add(word)) {
			     		distractforms += word + " ";
			 			}
			 			else{
			     		//log.info("Word that was excluded = {}",  word);
				 		}
			   	}
					// remove the whitespace at the end
					distractforms = distractforms.trim();
					// exclude the distractor if its only one, you need at least 2 distractors for mc
					if(distractorsSet.size() < 2) {
				 		distractforms = "";
			   	}
			   	else {
				 		//log.info("This are the chosen distractforms={}", distractforms);
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

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3]
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
						//log.info("spantag before adding forms:{}", spanTag);
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
						//log.info("Enhancement={}",e); // testing
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
				 		//log.info("ifst output:{}", word);
				 		// forms that could not be generated are excluded, as well as input strings of the iFST
				 		if (!word.contains("+") && !word.contains("-") && possible_formsSet.add(word)) {
				     	possible_forms += word + " ";
				 		}
				 		else{
			     		//log.info("Word that was excluded = {}",  word);
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
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word]
	public class Word {
    private int begin;
    private int end;
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.word-fn+2]
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.word.to-string-fn+2]
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
    private Vislcg3VerbConjugationEnhancer getOuterType() {
			return Vislcg3VerbConjugationEnhancer.this;
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
	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag]
	public class SpanTag{
    private String spanTagStart;
    private String spanTagEnd;
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.span-tag-fn+2]
    public SpanTag(String spanTagStart) {
			this.spanTagStart = spanTagStart;
			this.spanTagEnd = "</span>";
  	}
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-start-fn+2]
    public String getSpanTagStart() {
			return spanTagStart;
    }
    public void setSpanTagStart(String spanTagStart) {
			this.spanTagStart = spanTagStart;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.add-attribute-fn+3]
    public void addAttribute(String attributeName, String attributeValue) {
			this.spanTagStart = this.spanTagStart.replace(">", attributeName + "=\"" + attributeValue + "\">");
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.get-span-tag-end-fn+2]
    public String getSpanTagEnd() {
			return spanTagEnd;
    }
    public void setSpanTagEnd(String spanTagEnd) {
			this.spanTagEnd = spanTagEnd;
    }


    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.span-tag.to-string-fn+2]
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
    private Vislcg3VerbConjugationEnhancer getOuterType() {
			return Vislcg3VerbConjugationEnhancer.this;
    }

	}

}
