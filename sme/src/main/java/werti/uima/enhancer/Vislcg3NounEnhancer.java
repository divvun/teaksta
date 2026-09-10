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
 * In this case the topic is North Sámi nouns in singular form, use the patterns
 * in the method process() to extract the correct tokens for enhancement.
 *
 * @author Niels Ott
 * @author Adriane Boyd
 * @author Heli Uibo
 * @author Eduard Schaf
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer]
public class Vislcg3NounEnhancer extends JCasAnnotator_ImplBase {

	private static final Logger log =
		LogManager.GetLogger(Vislcg3NounEnhancer.class);

	private String enhancement_type = WERTiServlet.enhancement_type; // colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
	private List<String> NTags;
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

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.initialize-fn]
	@Override
	public void initialize(UimaContext context)
			throws ResourceInitializationException {
		super.initialize(context);
		NTags = Arrays.asList(((String)context.getConfigParameterValue("NTags")).split(","));
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+4]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.process-fn+4]
	@Override
	public void process(JCas cas) throws AnalysisEngineProcessException {
		// stop processing if the client has requested it
		if (!CasUtils.isValid(cas)) {
			return;
		}
		// colorize, click, mc or cloze - chosen by the user and sent to the servlet as a request parameter
		String enhancement_type = WERTiServlet.enhancement_type;
		log.info("Starting Noun Sg enhancement {}.", enhancement_type);

		long generatingDistractorsTotalTime = 0;

		final long startTime = System.currentTimeMillis();

		Pattern posPattern = Pattern.compile("N\\+");
		//Pattern numberPattern = Pattern.compile("Sg\\+Nom|Sg\\+Acc|Sg\\+Gen|Sg\\+Ill|Sg\\+Loc|Sg\\+Com|Ess");
		//Since the analyses can be the following: N+(Subclass)+(Semclass)+Number+Case(+Possessivesuffix)(+Clitic), all types of optional tags are added so that
		//for example the reading čáhci+N+<sme>+Sem/Plc_Substnc_Wthr+Sg+Nom that was skipped is now considered valid
		//Sg+Pl
		String nom_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Nom(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String acc_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Acc(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String gen_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Gen(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String ill_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Ill(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String loc_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Loc(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String com_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Com(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String ess_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?Sg|Pl\\+Ess(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?|";
		String attr_regex = "([a-zA-Z]*+[0-9]*+\\+)?(Sem/([a-zA-Z]*+_*+)*+\\+)?\\+Attr(\\+<([a-zA-Z]*+_*+)*+>)?(\\+[a-zA-Z]*+[0-9])?(\\+[a-zA-Z]*+)?(\\+Foc/[a-zA-Z]*+)?(\\+[a-zA-Z]*+)?";
		String pattern_to_compile = nom_regex+acc_regex+gen_regex+ill_regex+loc_regex+com_regex+ess_regex+attr_regex;
		Pattern numberPattern = Pattern.compile(pattern_to_compile);

		// Note that the whole token will be excluded, even when one reading is valid
		// exclude tokens with readings that are one of the following
		//Pattern excludePattern = Pattern.compile("V\\+|Pl|A\\+(?!.*Pred)|Det|Pr$|Pron|Pcle|Adv|Interj|CC|CS");
		//Some readings were excluded although valid, for example +Ta+N+<sme>+Prop+Sem/Plc+Sg+Gen
		//and also isit+N+Sem/Hum+Sg+Com+<compl_subj>+@Pron<
		//Pattern excludePattern = Pattern.compile("V\\+|Pl\\+|A\\+(?!.*Pred)|Det|Pr$|Pron\\+|Pcle|Adv|Interj|CC|CS");
		//Sg+Pl
		Pattern excludePattern = Pattern.compile("V\\+|A\\+(?!.*Pred)|Det|Pr$|Pron\\+|Pcle|Adv|Interj|CC|CS|ACR\\+Dyn");
		// patterns for the hints
		Pattern hintPattern = Pattern.compile("Pr$");
		// the following tags are allowed between hint and noun
		Pattern validHintPattern = Pattern.compile("A\\+|Det|Adv");

		Map<String, MutableInt> classCounts = new HashMap<String, MutableInt>();

		FSIterator cgTokenIter = cas.getAnnotationIndex(CGToken.type).iterator();

		// get timestamp in milliseconds and use it in the names of the temporary files in order to avoid conflicts between simultaneous users
		long timestamp = System.currentTimeMillis();

		//String cg3GeneratorInputFileLoc = "./output/cg3GeneratorInput"+timestamp+".tmp";
		//String cg3GeneratorOutputFileLoc = "./output/cg3GeneratorOutput"+timestamp+".tmp";
		String cg3GeneratorInputFileLoc = Constants.cg3GeneratorInputFile_Loc;
		String cg3GeneratorOutputFileLoc = Constants.cg3GeneratorOutputFile_Loc;

		//create temporary files for saving cg3 input and output
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

		String hintID = "";

		int hintDistance = 0;

		boolean isValidHint = false;


		try {
			Writer cg3GeneratorInputWriter = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(cg3GeneratorInputFileLoc), "UTF-8"));
			Writer cg3GeneratorInputWriterCloze = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(cg3GeneratorInputFileLoc), "UTF-8"));

			// go through tokens
			while (cgTokenIter.hasNext()) {

				CGToken cgt = (CGToken) cgTokenIter.next();

				String hintTag = "";

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

					// determine if a hint is still valid
					if(isValidHint){
						Matcher validHintMatcher = validHintPattern.matcher(currentReadingString);
						Matcher posMatcher = posPattern.matcher(currentReadingString);
						Matcher numberMatcher = numberPattern.matcher(currentReadingString);
						// an invalid hint doesn't match the valid hint pattern
						// and also the pos and number hint patterns
						if(!validHintMatcher.find() &&
								!(posMatcher.find() &&
								numberMatcher.find())){
							isValidHint = false;
							//log.info("This tag breaks the connection between hint and noun ={}", currentReadingString);
						}
					}

					// determine if the current tag is a hint
					if(hintTag.isEmpty()){
						Matcher hintMatcher = hintPattern.matcher(currentReadingString);
						if(hintMatcher.find()){
							// remove the first "+" and quotes and replace "+" with a "-"
							hintTag = currentReadingString.substring(1).replace("\"", "").replace("+", "-");
							//log.info(feedbackWord);
							isValidHint = true;
						}
					}
					//log.info("The current reading string is={}",  currentReadingString);
					// don't consider readings that match the exclude pattern, to filter out unlikely readings
					// (e.g. "и" is a CC in almost all cases, the probability that it is a N is very low)
					Matcher excludeMatcher = excludePattern.matcher(currentReadingString);
					if(excludeMatcher.find()){
						//log.info("This reading won't be considered={}",  currentReadingString);
						isValidReading = false;
						break;
					}
					if(!isValidReading){
						Matcher posMatcher = posPattern.matcher(currentReadingString);
						Matcher numberMatcher = numberPattern.matcher(currentReadingString);
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

					String spanTagStart = "<span id=\"" + EnhancerUtils.get_id("WERTi-span-" + spanReadingString, classCounts.get(spanReadingString).value) +
							"\" class=\"wertiviewtoken  wertiviewSubstantive\">";  // was: wertiviewhit

					SpanTag spanTag = new SpanTag(spanTagStart);

					spanTag.addAttribute("lemma", lemma);

					// only add the hint ID if the distance is allowed and the hint still valid
					// distance = 1 would allow no tokens in between
					if(!hintID.isEmpty() &&
							hintDistance < 4 &&
							isValidHint){
						spanTag.addAttribute("hintid", hintID);
					}

					// reset the validity of a hint
					isValidHint = false;

					wordToSpanMap.put(word, spanTag);

					if (isMcActivity) {
						// generate the distractors, with lemma, gender, animacy, number and case (needed for the form generator)
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
						//log.info("This is the cgt={} B={} E={}", cgt.getCoveredText(), word.getBegin(), word.getEnd());
						// make new enhancement, pass it to the cas
						Enhancement e = new Enhancement(cas);
						e.setRelevant(true);
						e.setBegin(word.getBegin());
						e.setEnd(word.getEnd());
						e.setEnhanceStart(spanTag.getSpanTagStart());
						e.setEnhanceEnd(spanTag.getSpanTagEnd());
						// update CAS
						cas.addFsToIndexes(e);
						//log.info("Enhancement={}", e); // testing
					}
				}
				else {
					if(!hintTag.isEmpty()){
						hintDistance = 0;
						// create a word with begin and end of the current CGToken
						Word word = new Word(cgt.getBegin(), cgt.getEnd());

						MutableInt idCount = classCounts.get(hintTag);
						if (idCount == null) {
							classCounts.put(hintTag, new MutableInt());
						}
						else {
							idCount.increment();
						}

						hintID = EnhancerUtils.get_id("WERTi-span-" + hintTag, classCounts.get(hintTag).value);

						String spanTagStart = "<span id=\"" + hintID +
								"\" class=\"wertiviewhinttag\">";

						SpanTag spanTag = new SpanTag(spanTagStart);

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
				hintDistance++;
			}

			cg3GeneratorInputWriter.close();
			cg3GeneratorInputWriterCloze.close();

			if(isMcActivity){
				// generate distractors only when the activity is "mc" (multiple choice)

				// this was the previous version which required to install hfst on the computer

//				String[] generationPipeline = {
//						"/bin/sh",
//						"-c",
//						"/bin/cat " + cg3GeneratorInputFileLoc +
//						" | " + hfstOptLookupLoc + " " + lookupFlags + " " + invertedOptHfstLoc +
//						" | " + "cut -f1-2"+ // get rid of the weight
//						" > " + cg3GeneratorOutputFileLoc};

				// the newer version is using hfst-ol.jar to load the .ohfst files (ol = optimized lookup)

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
			//cg3GeneratorInputFile.delete();
			//cg3GeneratorOutputFile.delete();

		} catch (IOException e) {
			e.printStackTrace();
		} catch (InterruptedException e) {
			e.printStackTrace();
		}

		log.info("Finished Noun Sg enhancement.");
		final long endTime = System.currentTimeMillis();

		log.info("Total execution time: {} seconds.", (endTime - startTime)*0.001);
		log.info("Generating the distractforms takes in total: {} seconds.", generatingDistractorsTotalTime * 0.001);
	}

	// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2]
	// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2]
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-morphological-forms-fn+2]
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
				String[] distractors_Sg_Nom = {"+Sg+Acc", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"};
				String[] distractors_Sg_Acc = {"+Sg+Nom", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"};
				String[] distractors_Sg_Gen = {"+Sg+Nom", "+Sg+Ill", "+Sg+Loc", "+Sg+Com"};
				String[] distractors_Sg_Ill = {"+Sg+Nom", "+Sg+Com", "+Ess", "+Sg+Acc"};
				String[] distractors_Sg_Loc = {"+Sg+Nom", "+Sg+Com", "+Ess", "+Sg+Acc"};
				String[] distractors_Sg_Com = {"+Sg+Nom", "+Sg+Ill", "+Ess", "+Sg+Acc"};
				String[] distractors_Ess = {"+Pl+Nom", "+Sg+Ill", "+Sg+Loc", "+Pl+Acc"};
				String[] distractors_Pl_Nom = {"+Pl+Acc", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"};
				String[] distractors_Pl_Acc = {"+Pl+Nom", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"};
				String[] distractors_Pl_Gen = {"+Pl+Nom", "+Pl+Ill", "+Pl+Loc", "+Pl+Com"};
				String[] distractors_Pl_Ill = {"+Pl+Nom", "+Pl+Com", "+Ess", "+Pl+Acc"};
				String[] distractors_Pl_Loc = {"+Pl+Nom", "+Pl+Ill", "+Ess", "+Pl+Acc"};
				String[] distractors_Pl_Com = {"+Pl+Nom", "+Pl+Ill", "+Ess", "+Sg+Acc"};
				/*
				String[] distractors_Nom = {"+Acc", "+Ill", "+Loc", "+Com"};
				String[] distractors_Acc = {"+Nom", "+Ill", "+Loc", "+Com"};
				String[] distractors_Gen = {"+Nom", "+Ill", "+Loc", "+Com"};
				String[] distractors_Ill = {"+Nom", "+Com", "+Ess", "+Acc"};
				String[] distractors_Sg_Loc = {"+Nom", "+Com", "+Ess", "+Acc"};
				String[] distractors_Pl_Loc = {"+Nom", "+Ill", "+Ess", "+Acc"};
				String[] distractors_Com = {"+Nom", "+Ill", "+Ess", "+Sg+Acc"};
				String[] distractors_Ess = {"+Pl+Nom", "+Sg+Ill", "+Sg+Loc", "+Pl+Acc"};
				*/

        String generationInput = "";
				String reading_str_input = "";
				//log.info("reading string:{}", reading_str);
				//remove @ only if it is in reading_str (otherwise get "String index out of range" error)
				if (reading_str.indexOf("@")>0) {
					reading_str_input = reading_str.substring(0, reading_str.indexOf("@")-1);
				} else {
					reading_str_input = reading_str;
				}

				String reading_str2 = reading_str;
				String generationInput2 = "";
				//log.info("reading_str2:{}", reading_str2);

				if(reading_str2.contains("+Sg")){
					//check if the strings contains Sg+Nom instead of only Nom
					//because in case of NomAg: oahpaheaddji+N+NomAg+Sem/Hum+Sg+Gen+@ADVL>
					//indexOf returns -1, and substring(0,-1) returns an error
					if(reading_str2.contains("+Sg+Nom")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Sg+Nom"));
						for(String elem: distractors_Sg_Nom) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Acc")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Sg+Acc"));
						for(String elem: distractors_Sg_Acc) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Gen")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Sg+Gen"));
						for(String elem: distractors_Sg_Gen) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Ill")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Sg+Ill"));
						for(String elem: distractors_Sg_Ill) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if (reading_str2.contains("+Loc")) {
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Sg+Loc"));
						for(String elem: distractors_Sg_Loc) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Com")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Sg+Com"));
						for(String elem: distractors_Sg_Com) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
				}

				if(reading_str2.contains("+Ess")){
					reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Ess"));
					for(String elem: distractors_Ess) {
						generationInput2 += reading_str2 + elem + "\n";
					}
				}

				if(reading_str2.contains("+Pl")){
					//same comment as for Sg
					if(reading_str2.contains("+Pl+Nom")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Pl+Nom"));
						for(String elem: distractors_Pl_Nom) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Acc")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Pl+Acc"));
						for(String elem: distractors_Pl_Acc) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Gen")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Pl+Gen"));
						for(String elem: distractors_Pl_Gen) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Ill")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Pl+Ill"));
						for(String elem: distractors_Pl_Ill) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if (reading_str2.contains("+Loc")) {
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Pl+Loc"));
						for(String elem: distractors_Pl_Loc) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
					if(reading_str2.contains("+Com")){
						reading_str2 = reading_str2.substring(0,reading_str2.indexOf("+Pl+Com"));
						for(String elem: distractors_Pl_Com) {
							generationInput2 += reading_str2 + elem + "\n";
						}
					}
				}

				log.info("generationInput2={}", generationInput2);

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
				generationInput2 += reading_str_input+"\n";

				//if generationInput contains tags_tbr, remove it
				generationInput = removeTags(generationInput);
				generationInput2 = removeTags(generationInput2);

        //log.info("generation input:{}", generationInput);
				return generationInput2;
    }

		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.write-lemma-and-analyses-fn+2]
		private String writeLemmaAndAnalyses(String reading_str) {

			String lemma_str = reading_str.substring(0, reading_str.indexOf("+"));
			String an_tmp = reading_str.substring(reading_str.indexOf("+")+1, reading_str.length());
			String analyses_str = an_tmp.replace("+<sme>", "");
			//remove @ only if it is in analyses_str (otherwise get "String index out of range" error)
			if (analyses_str.indexOf("@")>0) {
				analyses_str = analyses_str.substring(0, analyses_str.indexOf("@")-1);
			}
			String lem_and_an = lemma_str + "+" + analyses_str + "\n";

			//if analyses contains tags_tbr, remove it
			lem_and_an = removeTags(lem_and_an);

			if (lem_and_an.contains("+Sg+Acc")) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Sg+Acc"));
				lem_and_an += temp_str+"+Pl+Acc" + "\n";
			}
			if (lem_and_an.contains("+Sg+Gen")) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Sg+Gen"));
				lem_and_an += temp_str+"+Pl+Gen" + "\n";
			}
			if (lem_and_an.contains("+Sg+Ill")) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Sg+Ill"));
				lem_and_an += temp_str+"+Pl+Ill" + "\n";
			}
			if (lem_and_an.contains("+Sg+Com")) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Sg+Com"));
				lem_and_an += temp_str+"+Pl+Com" + "\n";
			}
			if (lem_and_an.contains("+Sg+Loc")) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Sg+Loc"));
				lem_and_an += temp_str+"+Pl+Loc" + "\n";
			}
			if ((lem_and_an.contains("+Pl+Acc")) && !(lem_and_an.contains("+Sg+Acc"))) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Pl+Acc"));
				lem_and_an += temp_str+"+Sg+Acc" + "\n";
			}
			if ((lem_and_an.contains("+Pl+Gen")) && !(lem_and_an.contains("+Sg+Gen"))) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Pl+Gen"));
				lem_and_an += temp_str+"+Sg+Gen" + "\n";
			}
			if ((lem_and_an.contains("+Pl+Ill")) && !(lem_and_an.contains("+Sg+Ill"))) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Pl+Ill"));
				lem_and_an += temp_str+"+Sg+Ill" + "\n";
			}
			if ((lem_and_an.contains("+Pl+Com")) && !(lem_and_an.contains("+Sg+Com"))) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Pl+Com"));
				lem_and_an += temp_str+"+Sg+Com" + "\n";
			}
			if ((lem_and_an.contains("+Pl+Loc")) && !(lem_and_an.contains("+Sg+Loc"))) {
				String temp_str = lem_and_an.substring(0,lem_and_an.indexOf("+Pl+Loc"));
				lem_and_an += temp_str+"+Sg+Loc" + "\n";
			}

			return lem_and_an;
	  }

    /*
     * The output file from the generator is used to create distractors and is placed into the right place in the span tag.
     * Afterwards an enhancement with the span tag is created and passed to the cas.
     */
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3]
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
						else{
							//log.info("Word that was excluded = {}", word);
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

		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3]
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
							log.info("possibleforms= {}", possible_forms);
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word]
    public class Word {
    	private int begin;
    	private int end;
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.word-fn+2]
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
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.word.to-string-fn+2]
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
		private Vislcg3NounEnhancer getOuterType() {
			return Vislcg3NounEnhancer.this;
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
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag]
    public class SpanTag{
    	private String spanTagStart;
    	private String spanTagEnd;
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.span-tag-fn+2]
		public SpanTag(String spanTagStart) {
			this.spanTagStart = spanTagStart;
			this.spanTagEnd = "</span>";
		}
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-start-fn+2]
		public String getSpanTagStart() {
			return spanTagStart;
		}
		public void setSpanTagStart(String spanTagStart) {
			this.spanTagStart = spanTagStart;
		}
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+3]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.add-attribute-fn+3]
		public void addAttribute(String attributeName, String attributeValue) {
			this.spanTagStart = this.spanTagStart.replace(">", attributeName + "=\"" + attributeValue + "\">");
		}
		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.get-span-tag-end-fn+2]
		public String getSpanTagEnd() {
			return spanTagEnd;
		}
		public void setSpanTagEnd(String spanTagEnd) {
			this.spanTagEnd = spanTagEnd;
		}


		// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn+2]
		// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.span-tag.to-string-fn+2]
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
		private Vislcg3NounEnhancer getOuterType() {
			return Vislcg3NounEnhancer.this;
		}

    }

}
