//! The output from the CG3 analysis in [`crate::pipeline::vislcg3`] is used to
//! enhance spans corresponding to the tags specified by the topic and the
//! activity that was chosen by the user. In this case the topic is North Sámi
//! nouns in plural form; the patterns in [`Vislcg3NounPlEnhancer::process`]
//! extract the correct tokens for enhancement.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo, Eduard Schaf.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::LazyLock;
use std::time::Instant;

use anyhow::{Result, bail};
use regex::Regex;
use tracing::info;

use crate::morpho::MorphoPipeline;
use crate::types::{Document, Enhancement};
use crate::util::constants;
use crate::util::jstring::{
    char_index_of, java_string_hash, java_trim, split_ws, string_tokenizer,
};
use crate::util::{cas_utils, enhancer_utils};

/// Separates one token's generator input (and, in the generator output, one
/// token's generated forms) from the next.
const MARKER: &str = "ñôŃßĘńŠē";

/// A Java exception raised inside the `process` try block that the block does
/// not catch — it catches only IOException and InterruptedException, so an
/// out-of-range substring, a non-numeric `Word` line or a missing map entry
/// leaves `process` instead of being swallowed.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct Uncaught(String);

/// The regex form of the last entry of [`Vislcg3NounPlEnhancer::TAGS_TBR`],
/// which `remove_tags` treats as a pattern rather than a literal.
static TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(Vislcg3NounPlEnhancer::TAGS_TBR[Vislcg3NounPlEnhancer::TAGS_TBR.len() - 1])
        .expect("tags_tbr trailing pattern")
});

/// `Integer.parseInt`: a non-numeric field raises NumberFormatException, which
/// the callers do not catch.
fn parse_offset(field: &str) -> Result<usize> {
    field
        .parse::<usize>()
        .map_err(|_| Uncaught(format!("For input string: \"{}\"", field)).into())
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer]
pub struct Vislcg3NounPlEnhancer {
    n_pl_tags: Option<Vec<String>>,
}

impl Default for Vislcg3NounPlEnhancer {
    fn default() -> Self {
        Vislcg3NounPlEnhancer { n_pl_tags: None }
    }
}

impl Vislcg3NounPlEnhancer {
    pub const CHUNK_BEGIN_SUFFIX: &'static str = "-B";
    pub const CHUNK_INSIDE_SUFFIX: &'static str = "-I";
    pub const LOOKUP_LOC: &'static str = constants::LOOKUP_LOC;
    pub const LOOKUP_FLAGS: &'static str = constants::LOOKUP_FLAGS;
    pub const INVERTED_FST: &'static str = constants::INVERTED_FST;
    pub const FST: &'static str = constants::AN_FST;

    /// List of tags to be removed from analyses because these are not present
    /// in generator-norm. The last entry is a regex matching all possible tags
    /// of the type `<xxx_xxx>`; every earlier entry is a literal.
    pub const TAGS_TBR: [&'static str; 12] = [
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
        r"\+<([a-zA-Z]*_*)*>",
    ];

    /// Stands in for the enclosing instance captured by the Java inner classes:
    /// `Word` and `SpanTag` fold the enclosing enhancer's identity into their
    /// equality and hash, so instances only match when built by the same
    /// enhancer.
    fn outer_id(&self) -> usize {
        self as *const Self as usize
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.initialize-fn]
    pub fn initialize(&mut self, n_pl_tags: Option<&str>) -> Result<()> {
        info!("Noun Pl tags {:?}", self.n_pl_tags);
        let param = match n_pl_tags {
            Some(p) => p,
            None => bail!("NPlTags configuration parameter is not set"),
        };
        self.n_pl_tags = Some(param.split(',').map(str::to_string).collect());
        Ok(())
    }

    pub fn new(n_pl_tags: Option<&str>) -> Result<Self> {
        let mut this = Self::default();
        this.initialize(n_pl_tags)?;
        Ok(this)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.process-fn]
    pub fn process(&self, doc: &mut Document) -> Result<()> {
        // stop processing if the client has requested it
        if !cas_utils::is_valid(doc) {
            return Ok(());
        }

        // colorize, click, mc or cloze - chosen by the user and sent to the
        // servlet as a request parameter
        let enhancement_type = crate::server::servlet::enhancement_type();
        info!("Starting Noun Pl enhancement {}.", enhancement_type);

        let mut generating_distractors_total_time: f64 = 0.0;
        let start_time = Instant::now();

        let pos_pattern = Regex::new(r"N\+")?;
        // Since the analyses can be the following:
        // N+(Subclass)+(Semclass)+Number+Case(+Possessivesuffix)(+Clitic), all
        // types of optional tags are added so that for example the reading
        // čáhci+N+<sme>+Sem/Plc_Substnc_Wthr+Sg+Nom that was skipped is now
        // considered valid.
        let nom_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Nom(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|";
        let acc_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Acc(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|";
        let gen_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Gen(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|";
        let ill_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Ill(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|";
        let loc_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Loc(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|";
        let com_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Com(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?|";
        let ess_regex = r"([a-zA-Z]*[0-9]*\+)?(Sem/([a-zA-Z]*_*)*\+)?Pl\+Ess(\+<([a-zA-Z]*_*)*>)?(\+[a-zA-Z]*[0-9])?(\+[a-zA-Z]*)?(\+Foc/[a-zA-Z]*)?(\+[a-zA-Z]*)?";
        let pattern_to_compile = format!(
            "{}{}{}{}{}{}{}",
            nom_regex, acc_regex, gen_regex, ill_regex, loc_regex, com_regex, ess_regex
        );
        let number_case_pattern = Regex::new(&pattern_to_compile)?;

        let mut class_counts: HashMap<String, MutableInt> = HashMap::new();

        let cg_tokens = doc.cg_tokens.clone();

        // The timestamped file names that would isolate simultaneous users are
        // commented out in the source; every request shares these two paths.
        let cg3_generator_input_file_loc = constants::CG3_GENERATOR_INPUT_FILE_LOC;
        let cg3_generator_output_file_loc = constants::CG3_GENERATOR_OUTPUT_FILE_LOC;

        let mut word_to_span_map: HashMap<Word, SpanTag> = HashMap::new();

        let is_mc_activity = enhancement_type == "mc";
        let is_cloze_activity = enhancement_type == "cloze";

        let morpho = MorphoPipeline::shared();

        // Everything the Java body wraps in try { } catch (IOException |
        // InterruptedException) { printStackTrace(); }.
        let try_result: Result<()> = (|| -> Result<()> {
            // One buffer per Java writer; both writers targeted the same
            // truncated generator input file, so only the branch that is
            // actually taken contributes content.
            let mut cg3_generator_input_writer = String::new();
            let mut cg3_generator_input_writer_cloze = String::new();

            // go through tokens
            for cgt in &cg_tokens {
                let mut is_valid_reading = false;
                let mut reading_str = String::new();
                let mut lemma = String::new();
                // select from all readings the first occurrence that is
                // matching pos and number
                for i in 0..cgt.readings.len() {
                    let current_reading = &cgt.readings[i];
                    let mut current_reading_string = String::new();
                    for rtag in current_reading {
                        current_reading_string = current_reading_string + "+" + rtag;
                    }
                    if !is_valid_reading
                        && pos_pattern.is_match(&current_reading_string)
                        && number_case_pattern.is_match(&current_reading_string)
                    {
                        is_valid_reading = true;
                        // remove the first "+" and quotes
                        reading_str = current_reading_string[1..].replace('"', "");
                        // the lemma is the first element of the reading string
                        lemma = reading_str.split('+').next().unwrap_or("").to_string();
                    }
                }
                if is_valid_reading {
                    // id's with the "+" symbol have to be escaped, thats why we
                    // use a "-" instead
                    let mut span_reading_string = reading_str.replace('+', "-");
                    // The "<" and ">" symbols also cause problems because these
                    // are the tag opening / closing symbol.
                    span_reading_string = span_reading_string.replace('<', "x");
                    span_reading_string = span_reading_string.replace('>', "y");

                    if !class_counts.contains_key(&span_reading_string) {
                        class_counts.insert(span_reading_string.clone(), MutableInt::default());
                    } else {
                        class_counts
                            .get_mut(&span_reading_string)
                            .expect("present")
                            .increment();
                    }
                    // create a word with begin and end of the current CGToken
                    let word = Word::new(self.outer_id(), cgt.begin, cgt.end);

                    // was: wertiviewhit
                    let span_tag_start = format!(
                        "<span id=\"{}\" class=\"wertiviewtoken  wertiviewSubstantivePlural\">",
                        enhancer_utils::get_id(
                            &format!("WERTi-span-{}", span_reading_string),
                            class_counts[&span_reading_string].value
                        )
                    );

                    let mut span_tag = SpanTag::new(self.outer_id(), span_tag_start);

                    span_tag.add_attribute("lemma", &lemma);
                    word_to_span_map.insert(word.clone(), span_tag.clone());

                    if is_mc_activity {
                        // generate the distractors, with lemma, number and case
                        // and save in a file
                        let analyses_str = reading_str.replace("+<sme>", "");
                        let distractors = self.write_morphological_forms(&analyses_str)?;
                        cg3_generator_input_writer.push_str(&distractors);
                        // write the marker that separates the current
                        // distractors from others
                        cg3_generator_input_writer.push_str(MARKER);
                        cg3_generator_input_writer.push('\n');
                        // write the word to the file in order to assign the
                        // correct distractors to the correct span
                        cg3_generator_input_writer.push_str(&word.to_string());
                    } else if is_cloze_activity {
                        // extract lemma and analyses from reading_str and write
                        // to file
                        let lemma_and_analyses = self.write_lemma_and_analyses(&reading_str)?;
                        cg3_generator_input_writer_cloze.push_str(&lemma_and_analyses);
                        // write the marker that separates the current
                        // lemma+analyses from others
                        cg3_generator_input_writer_cloze.push_str(MARKER);
                        cg3_generator_input_writer_cloze.push('\n');
                        // write the word to the file in order to assign the
                        // correct distractors to the correct span
                        cg3_generator_input_writer_cloze.push_str(&word.to_string());
                    } else {
                        // make new enhancement, pass it to the cas
                        let mut e = Enhancement::default();
                        e.relevant = true;
                        e.begin = word.get_begin();
                        e.end = word.get_end();
                        e.enhance_start = span_tag.get_span_tag_start().to_string();
                        e.enhance_end = span_tag.get_span_tag_end().to_string();
                        // update CAS
                        doc.enhancements.push(e);
                    }
                }
            }

            if is_mc_activity {
                // generate distractors only when the activity is "mc"
                // (multiple choice)
                let generation_pipeline = format!(
                    "/bin/cat {} | {} {} {} > {}",
                    cg3_generator_input_file_loc,
                    Self::LOOKUP_LOC,
                    Self::LOOKUP_FLAGS,
                    Self::INVERTED_FST,
                    cg3_generator_output_file_loc
                );

                info!("Distractor generation pipeline: {}", generation_pipeline);

                let start_time_generator = Instant::now();

                let generator_output = morpho.generate(&cg3_generator_input_writer)?;
                self.generate_span_tag_with_distractors(
                    doc,
                    &generator_output,
                    &mut word_to_span_map,
                )?;

                generating_distractors_total_time +=
                    start_time_generator.elapsed().as_secs_f64() * 1000.0;
            }

            if is_cloze_activity {
                // generate possible forms from lemma and analyses; the Java
                // rebuilds the identical shell pipeline here without logging it
                let start_time_generator = Instant::now();

                let generator_output = morpho.generate(&cg3_generator_input_writer_cloze)?;
                self.generate_span_tag_with_possible_forms(
                    doc,
                    &generator_output,
                    &mut word_to_span_map,
                )?;

                generating_distractors_total_time +=
                    start_time_generator.elapsed().as_secs_f64() * 1000.0;
            }

            Ok(())
        })();

        if let Err(e) = try_result {
            if e.downcast_ref::<Uncaught>().is_some() {
                return Err(e);
            }
            eprintln!("{}", e);
        }

        info!("Finished Noun Pl enhancement.");
        let end_time = start_time.elapsed();

        info!("Total execution time: {} seconds.", end_time.as_secs_f64());
        info!(
            "Generating the distractforms takes in total: {} seconds.",
            generating_distractors_total_time * 0.001
        );
        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn]
    fn remove_tags(&self, input_str: &str) -> String {
        let mut input_str = input_str.to_string();
        for h in 0..Self::TAGS_TBR.len() {
            if h < Self::TAGS_TBR.len() - 1 {
                if input_str.contains(Self::TAGS_TBR[h]) {
                    input_str = input_str.replace(Self::TAGS_TBR[h], "");
                }
            } else if let Some(m) = TAG_REGEX.find(&input_str) {
                let mytag = m.as_str().to_string();
                input_str = input_str.replace(&mytag, "");
            }
        }
        input_str
    }

    /// Create all relevant morphological forms of the current token. It is the
    /// input for the distractor generation.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-morphological-forms-fn]
    fn write_morphological_forms(&self, reading_str: &str) -> Result<String> {
        let distract_forms_case = ["+Nom", "+Acc", "+Gen", "+Ill", "+Loc", "+Com", "+Ess"];

        let mut reading_str = reading_str.to_string();
        let mut generation_input = String::new();
        // substring(0, indexOf("@") - 1): a reading without a syntactic tag
        // gives substring(0, -2), and a reading whose first character is "@"
        // gives substring(0, -1)
        let reading_str_input = match char_index_of(&reading_str, '@') {
            Some(i) if i >= 1 => reading_str.chars().take(i - 1).collect::<String>(),
            Some(_) => bail!(Uncaught(format!(
                "begin 0, end -1, length {}",
                reading_str.chars().count()
            ))),
            None => bail!(Uncaught(format!(
                "begin 0, end -2, length {}",
                reading_str.chars().count()
            ))),
        };

        for a_case in distract_forms_case {
            if reading_str.contains(a_case) {
                // remove the case marker and the syntactic tag from the reading
                let cut = reading_str.find(a_case).expect("contains");
                reading_str = reading_str[..cut].to_string();
                // Assign distractorforms from the array
                for elem in distract_forms_case {
                    generation_input = generation_input + &reading_str + elem + "\n";
                }
                break;
            }
        }

        // add reading_str as last element in generationInput which will be used
        // as correct_answer
        generation_input = generation_input + &reading_str_input + "\n";

        // if generationInput contains tags_tbr, remove it
        generation_input = self.remove_tags(&generation_input);

        Ok(generation_input)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.write-lemma-and-analyses-fn]
    fn write_lemma_and_analyses(&self, reading_str: &str) -> Result<String> {
        // substring(0, indexOf("+")): a reading without a "+" gives
        // substring(0, -1)
        let plus = match reading_str.find('+') {
            Some(i) => i,
            None => bail!(Uncaught(format!(
                "begin 0, end -1, length {}",
                reading_str.chars().count()
            ))),
        };
        let lemma_str = &reading_str[..plus];
        let an_tmp = &reading_str[plus + 1..];
        let mut analyses_str = an_tmp.replace("+<sme>", "");
        // substring(0, indexOf("@") - 1) with the same out-of-range cases as in
        // write_morphological_forms
        analyses_str = match char_index_of(&analyses_str, '@') {
            Some(i) if i >= 1 => analyses_str.chars().take(i - 1).collect::<String>(),
            Some(_) => bail!(Uncaught(format!(
                "begin 0, end -1, length {}",
                analyses_str.chars().count()
            ))),
            None => bail!(Uncaught(format!(
                "begin 0, end -2, length {}",
                analyses_str.chars().count()
            ))),
        };
        let lem_and_an = format!("{}+{}\n", lemma_str, analyses_str);

        // if analyses contains tags_tbr, remove it
        let lem_and_an = self.remove_tags(&lem_and_an);

        Ok(lem_and_an)
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn]
    fn generate_span_tag_with_distractors(
        &self,
        doc: &mut Document,
        cg3_generator_output: &str,
        word_to_span_map: &mut HashMap<Word, SpanTag>,
    ) -> Result<()> {
        let mut generator_output = String::new();

        let mut current_word = Word::empty(self.outer_id());
        let mut distractforms = String::new();
        let mut splitted_go: Vec<String> = vec![String::new()];

        for raw_line in cg3_generator_output.lines() {
            let line = java_trim(raw_line);
            if line.is_empty() {
                continue;
            }
            // generator output was processed, all distractors are created;
            // assign the distractors to the correct span from the wordToSpanMap
            else if line.starts_with("Word") {
                // only enhance tokens with more than one distractor form
                if !distractforms.is_empty() {
                    let line_parts = split_ws(line);
                    if line_parts.len() < 3 {
                        bail!(Uncaught(format!(
                            "Index 2 out of bounds for length {}",
                            line_parts.len()
                        )));
                    }
                    let begin: usize = parse_offset(line_parts[1])?;
                    let end: usize = parse_offset(line_parts[2])?;
                    current_word = Word::new(self.outer_id(), begin, end);
                    let span_tag = match word_to_span_map.get_mut(&current_word) {
                        Some(t) => t,
                        // the Java lookup returns null here and the next call
                        // raises a NullPointerException
                        None => bail!(Uncaught(format!("no span tag for {}", current_word))),
                    };
                    info!("spantag before adding distractors:{}", span_tag);
                    span_tag.add_attribute("distractors", &distractforms);
                    if splitted_go.is_empty() {
                        bail!(Uncaught("Index -1 out of bounds for length 0".to_string()));
                    }
                    span_tag.add_attribute("answer", &splitted_go[splitted_go.len() - 1]);
                    // make new enhancement, pass it to the cas
                    let mut e = Enhancement::default();
                    e.relevant = true;
                    e.begin = begin;
                    e.end = end;
                    e.enhance_start = span_tag.get_span_tag_start().to_string();

                    e.enhance_end = span_tag.get_span_tag_end().to_string();
                    // update CAS
                    doc.enhancements.push(e.clone());
                    info!("Enhancement={:?}", e);
                }
            }
            // the marker (ñôŃßĘńŠē) was found, begin to process the generator
            // output, create distractors
            else if line.contains(MARKER) {
                let go = std::mem::take(&mut generator_output);
                let tok = string_tokenizer(&go);
                splitted_go = split_ws(&go).into_iter().map(str::to_string).collect();
                distractforms = String::new();
                // the distractorsSet's purpose is to filter out duplicates
                let mut distractors_set: HashSet<String> = HashSet::new();
                for word in tok {
                    // forms that could not be generated are excluded, as well
                    // as input strings of the iFST
                    if !word.contains('+')
                        && !word.contains('-')
                        && distractors_set.insert(word.to_string())
                    {
                        distractforms = distractforms + word + " ";
                    }
                }
                // remove the whitespace at the end
                distractforms = java_trim(&distractforms).to_string();
                // exclude the distractor if its only one, you need at least 2
                // distractors for mc
                if distractors_set.len() < 2 {
                    distractforms = String::new();
                }
            }
            // the generator output for the current token is not fully extracted
            // from the stream yet
            else {
                generator_output = generator_output + line + " ";
            }
        }

        Ok(())
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn]
    fn generate_span_tag_with_possible_forms(
        &self,
        doc: &mut Document,
        cg3_generator_output: &str,
        word_to_span_map: &mut HashMap<Word, SpanTag>,
    ) -> Result<()> {
        let mut generator_output = String::new();

        let mut current_word = Word::empty(self.outer_id());
        let mut possible_forms = String::new();

        for raw_line in cg3_generator_output.lines() {
            let line = java_trim(raw_line);
            if line.is_empty() {
                continue;
            }
            // generator output was processed, all possible forms are created;
            // assign the possible forms to the correct span from the
            // wordToSpanMap
            else if line.starts_with("Word") {
                if !possible_forms.is_empty() {
                    let line_parts = split_ws(line);
                    if line_parts.len() < 3 {
                        bail!(Uncaught(format!(
                            "Index 2 out of bounds for length {}",
                            line_parts.len()
                        )));
                    }
                    let begin: usize = parse_offset(line_parts[1])?;
                    let end: usize = parse_offset(line_parts[2])?;
                    current_word = Word::new(self.outer_id(), begin, end);
                    let span_tag = match word_to_span_map.get_mut(&current_word) {
                        Some(t) => t,
                        // the Java lookup returns null here and the next call
                        // raises a NullPointerException
                        None => bail!(Uncaught(format!("no span tag for {}", current_word))),
                    };
                    span_tag.add_attribute("possibleforms", &possible_forms);
                    // make new enhancement, pass it to the cas
                    let mut e = Enhancement::default();
                    e.relevant = true;
                    e.begin = begin;
                    e.end = end;
                    e.enhance_start = span_tag.get_span_tag_start().to_string();
                    e.enhance_end = span_tag.get_span_tag_end().to_string();
                    // update CAS
                    doc.enhancements.push(e);
                }
            }
            // the marker (ñôŃßĘńŠē) was found, begin to process the generator
            // output, create the possible forms
            else if line.contains(MARKER) {
                let go = std::mem::take(&mut generator_output);
                let tok = string_tokenizer(&go);
                possible_forms = String::new();
                // the possible_formsSet's purpose is to filter out duplicates
                let mut possible_forms_set: HashSet<String> = HashSet::new();
                for word in tok {
                    // forms that could not be generated are excluded, as well
                    // as input strings of the iFST
                    if !word.contains('+')
                        && !word.contains('-')
                        && possible_forms_set.insert(word.to_string())
                    {
                        possible_forms = possible_forms + word + " ";
                    }
                }
                // remove the whitespace at the end
                possible_forms = java_trim(&possible_forms).to_string();
            }
            // the generator output for the current token is not fully extracted
            // from the stream yet
            else {
                generator_output = generator_output + line + " ";
            }
        }

        Ok(())
    }
}

/// This class represents a mutable integer value, which is especially useful
/// and fast for counting frequencies inside a map.
///
/// Author: Eduard Schaf.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int]
#[derive(Debug, Clone)]
pub struct MutableInt {
    // note that we start at 1 since we're counting
    value: i32,
}

impl Default for MutableInt {
    fn default() -> Self {
        MutableInt { value: 1 }
    }
}

impl MutableInt {
    /// Increment the mutable int by one.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.increment-fn]
    pub fn increment(&mut self) {
        self.value += 1;
    }

    /// Get the value of the mutable int.
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.mutable-int.get-fn]
    pub fn get(&self) -> i32 {
        self.value
    }
}

/// This class represents a word of two integers which are begin and end. They
/// are used to store the offsets of a given Token.
///
/// Author: Eduard Schaf.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word]
#[derive(Debug, Clone)]
pub struct Word {
    outer: usize,
    begin: usize,
    end: usize,
}

impl Word {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.word-fn]
    pub fn new(outer: usize, begin: usize, end: usize) -> Self {
        Word { outer, begin, end }
    }

    /// The no-argument Java constructor: both offsets zero.
    pub fn empty(outer: usize) -> Self {
        Word {
            outer,
            begin: 0,
            end: 0,
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-begin-fn]
    pub fn get_begin(&self) -> usize {
        self.begin
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-begin-fn]
    pub fn set_begin(&mut self, begin: usize) {
        self.begin = begin;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-end-fn]
    pub fn get_end(&self) -> usize {
        self.end
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.set-end-fn]
    pub fn set_end(&mut self, end: usize) {
        self.end = end;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.hash-code-fn]
    pub fn hash_code(&self) -> i32 {
        let prime: i32 = 31;
        let mut result: i32 = 1;
        result = prime
            .wrapping_mul(result)
            .wrapping_add(self.get_outer_type() as i32);
        result = prime.wrapping_mul(result).wrapping_add(self.begin as i32);
        result = prime.wrapping_mul(result).wrapping_add(self.end as i32);
        result
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.equals-fn]
    pub fn equals(&self, other: &Word) -> bool {
        if self.get_outer_type() != other.get_outer_type() {
            return false;
        }
        if self.begin != other.begin {
            return false;
        }
        if self.end != other.end {
            return false;
        }
        true
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.get-outer-type-fn]
    fn get_outer_type(&self) -> usize {
        self.outer
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.word.to-string-fn]
impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Word {} {}", self.begin, self.end)
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Eq for Word {}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.hash_code());
    }
}

/// This class represents a SpanTag consisting out of the span start tag with
/// possibility to add attributes to the span tag and the span end tag. It is
/// the span surrounding the token that is being enhanced.
///
/// Author: Eduard Schaf.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag]
#[derive(Debug, Clone)]
pub struct SpanTag {
    outer: usize,
    span_tag_start: String,
    span_tag_end: String,
}

impl SpanTag {
    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.span-tag-fn]
    pub fn new(outer: usize, span_tag_start: String) -> Self {
        SpanTag {
            outer,
            span_tag_start,
            span_tag_end: "</span>".to_string(),
        }
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-start-fn]
    pub fn get_span_tag_start(&self) -> &str {
        &self.span_tag_start
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-start-fn]
    pub fn set_span_tag_start(&mut self, span_tag_start: String) {
        self.span_tag_start = span_tag_start;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.add-attribute-fn]
    pub fn add_attribute(&mut self, attribute_name: &str, attribute_value: &str) {
        self.span_tag_start = self
            .span_tag_start
            .replace(">", &format!("{}=\"{}\">", attribute_name, attribute_value));
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-span-tag-end-fn]
    pub fn get_span_tag_end(&self) -> &str {
        &self.span_tag_end
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.set-span-tag-end-fn]
    pub fn set_span_tag_end(&mut self, span_tag_end: String) {
        self.span_tag_end = span_tag_end;
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.hash-code-fn]
    pub fn hash_code(&self) -> i32 {
        let prime: i32 = 31;
        let mut result: i32 = 1;
        result = prime
            .wrapping_mul(result)
            .wrapping_add(self.get_outer_type() as i32);
        result = prime
            .wrapping_mul(result)
            .wrapping_add(java_string_hash(&self.span_tag_end));
        result = prime
            .wrapping_mul(result)
            .wrapping_add(java_string_hash(&self.span_tag_start));
        result
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.equals-fn]
    pub fn equals(&self, other: &SpanTag) -> bool {
        if self.get_outer_type() != other.get_outer_type() {
            return false;
        }
        if self.span_tag_end != other.span_tag_end {
            return false;
        }
        if self.span_tag_start != other.span_tag_start {
            return false;
        }
        true
    }

    // [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
    // [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.get-outer-type-fn]
    fn get_outer_type(&self) -> usize {
        self.outer
    }
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.span-tag.to-string-fn]
impl fmt::Display for SpanTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpanTag [spanTagStart={}, spanTagEnd={}]",
            self.span_tag_start, self.span_tag_end
        )
    }
}

#[cfg(test)]
#[path = "noun_pl_tests.rs"]
mod tests;
