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

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer]
public class Vislcg3NounPlEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		LogManager.GetLogger(Vislcg3NounPlEnhancer.class);

	private List<String> NPlTags;
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

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
	@Override
	public void initialize(UimaContext context)
		throws ResourceInitializationException {
      log.info("Noun Pl tags {}", NPlTags);
			super.initialize(context);
			NPlTags = Arrays.asList(((String)context.getConfigParameterValue("NPlTags")).split(","));
		}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn+2]
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
    // stop processing if the client has requested it
    if (!CasUtils.isValid(cas)) {
		  return;
    }

		String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
		log.info("Starting Noun Pl enhancement {}.", enhancement_type);

		long generatingDistractorsTotalTime = 0;
		final long startTime = System.currentTimeMillis();

    Pattern posPattern = Pattern.compile("N\\+");
    //Pattern number_casePattern = Pattern.compile("Pl\\+Nom|Pl\\+Acc|Pl\\+Gen|Pl\\+Ill|Pl\\+Loc|Pl\\+Com|Ess");
		//Since the analyses can be the following: N+(Subclass)+(Semclass)+Number+Case(+Possessivesuffix)(+Clitic), all types of optional tags are added so that
		//for example the reading čáhci+N+<sme>+Sem/Plc_Substnc_Wthr+Sg+Nom that was skipped is now considered valid
		String nom_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Nom(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String acc_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Acc(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String gen_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Gen(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String ill_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Ill(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String loc_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Loc(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String com_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Com(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String ess_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Pl\\+Ess(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?";
		String pattern_to_compile = nom_regex+acc_regex+gen_regex+ill_regex+loc_regex+com_regex+ess_regex;
		Pattern number_casePattern = Pattern.compile(pattern_to_compile);

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

					String spanTagStart = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + spanReadingString, classCounts.get(spanReadingString).value) + "\" class=\"wertiviewtoken  wertiviewSubstantivePlural\">";  // was: wertiviewhit

          SpanTag spanTag = new SpanTag(spanTagStart);

          spanTag.addAttribute("lemma", lemma);
					wordToSpanMap.put(word, spanTag);

			if (isMcActivity) {
			    // generate the distractors, with lemma, number and case and save in a file
				String analyses_str = reading_str.replace("+<sme>", "");
				distractors = writeMorphologicalForms(analyses_str);
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
        log.info("Finished Noun Pl enhancement.");
        final long endTime = System.currentTimeMillis();

        log.info("Total execution time: {} seconds.", (endTime - startTime)*0.001);
        log.info("Generating the distractforms takes in total: {} seconds.", generatingDistractorsTotalTime * 0.001);
     }

		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2]
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
     // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn+2]
     // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn+2]
     private String writeMorphologicalForms(String reading_str) {

	 	 	String[] distractFormsCase = {
				"+Nom",
				"+Acc",
				"+Gen",
				"+Ill",
				"+Loc",
				"+Com",
				"+Ess"
			};

	 		String generationInput = "";
			//log.info("reading string:{}", reading_str);
			String reading_str_input = reading_str.substring(0, reading_str.indexOf("@")-1);

	 		for(String aCase: distractFormsCase){
	    	if(reading_str.contains(aCase)){
		 			// remove the case marker and the syntactic tag from the reading
		 			reading_str = reading_str.substring(0,reading_str.indexOf(aCase));
		 			//log.info("reading string without case and syntax tag:{}", reading_str);
		 			// Assign distractorforms from the array
		 			for(String elem: distractFormsCase) {
		     		generationInput += reading_str + elem + "\n";
		 			}
		 			break;
	     	}
	 		}

			//add reading_str as last element in generationInput which will be used as correct_answer
			generationInput += reading_str_input+"\n";

			//if generationInput contains tags_tbr, remove it
			generationInput = removeTags(generationInput);

	 		//log.info("generation input:{}", generationInput);
	 		return generationInput;
   	}

		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn+2]
		private String writeLemmaAndAnalyses(String reading_str) {
			String lemma_str = reading_str.substring(0, reading_str.indexOf("+"));
			String an_tmp = reading_str.substring(reading_str.indexOf("+")+1, reading_str.length());
			String analyses_str = an_tmp.replace("+<sme>", "");
			analyses_str = analyses_str.substring(0, analyses_str.indexOf("@")-1);
			String lem_and_an = lemma_str + "+" + analyses_str + "\n";

			//if analyses contains tags_tbr, remove it
			lem_and_an = removeTags(lem_and_an);

			return lem_and_an;
	  }

     // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+2]
     // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+2]
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
						log.info("spantag before adding distractors:{}", spanTag);
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
						log.info("Enhancement={}",e); // testing
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
			 			else {
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
		 		else {
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

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+2]
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
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int]
public class MutableInt {
    int value = 1; // note that we start at 1 since we're counting
    /**
     * Increment the mutable int by one.
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
    public void increment () {
	++value;
    }
    /**
     * Get the value of the mutable int.
     * @return the mutable int value.
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
    public int  get () {
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
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word]
public class Word {
    private int begin;
    private int end;
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
    public Word(int begin, int end) {
	this.begin = begin;
	this.end = end;
    }
    public Word() {
	this.begin = 0;
	this.end = 0;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
    public int getBegin() {
	return begin;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
    public void setBegin(int begin) {
	this.begin = begin;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
    public int getEnd() {
	return end;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
    public void setEnd(int end) {
	this.end = end;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
    @Override
    public String toString() {
	 return "Word " + begin + " " + end + "\n";
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
    @Override
    public int hashCode() {
	final int prime = 31;
	int result = 1;
	result = prime * result + getOuterType().hashCode();
	result = prime * result + begin;
	result = prime * result + end;
	return result;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
    private Vislcg3NounPlEnhancer getOuterType() {
	return Vislcg3NounPlEnhancer.this;
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
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag]
public class SpanTag{
    private String spanTagStart;
    private String spanTagEnd;
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
    public SpanTag(String spanTagStart) {
	this.spanTagStart = spanTagStart;
	this.spanTagEnd = "</span>";
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
    public String getSpanTagStart() {
	return spanTagStart;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
    public void setSpanTagStart(String spanTagStart) {
	this.spanTagStart = spanTagStart;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn+2]
    public void addAttribute(String attributeName, String attributeValue) {
	this.spanTagStart = this.spanTagStart.replace(">", attributeName + "=\"" + attributeValue + "\">");
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
    public String getSpanTagEnd() {
	return spanTagEnd;
    }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
    public void setSpanTagEnd(String spanTagEnd) {
	this.spanTagEnd = spanTagEnd;
    }


    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
    @Override
	 public String toString() {
	     return "SpanTag [spanTagStart=" + spanTagStart + ", spanTagEnd=" + spanTagEnd + "]";
         }
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
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
     // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
     // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
    private Vislcg3NounPlEnhancer getOuterType() {
	return Vislcg3NounPlEnhancer.this;
    }

}

}
