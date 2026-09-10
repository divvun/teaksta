//! Everything the five "big" CG3 topic enhancers hold in common.
//!
//! `Vislcg3NounEnhancer`, `Vislcg3NounPlEnhancer`,
//! `Vislcg3VerbConjugationEnhancer`, `Vislcg3ConNegEnhancer` and
//! `Vislcg3InfiniteVerbEnhancer` are copy-paste siblings in the Java
//! source: each declares its own `removeTags`, its own pair of
//! generator-output readers and its own enhancement pass, all textually
//! identical. Here they are one implementation carrying every sibling's rule
//! ids; the value types those passes work on are in
//! [`crate::enhancer::cg_span`].
//!
//! What genuinely differs between the topics — the span class, the reading
//! patterns, the preposition-hint rules only the singular-noun topic has,
//! which log lines each Java class kept, and whether an unchecked exception
//! escapes `process` — is named by [`TopicSpec`] and supplied per topic.
//! The two generator inputs differ per topic in shape as well as in data,
//! so [`run`] takes them as closures rather than as configuration.
//!
//! Authors: Niels Ott, Adriane Boyd, Heli Uibo, Eduard Schaf.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::time::Instant;

use anyhow::{Result, anyhow, bail};
use regex::Regex;
use tracing::{debug, info};

pub use crate::enhancer::cg_span::{HINT_CLASS, SpanTag, TOKEN_CLASS, Word};
use crate::morpho::MorphoPipeline;
use crate::server::api::Mode;
use crate::types::{CgToken, Document, Enhancement};
use crate::util::{cas_utils, enhancer_utils};

/// Separates one token's generator input (and, in the generator output, one
/// token's generated forms) from the next.
pub const MARKER: &str = "ñôŃßĘńŠē";

/// List of tags to be removed from analyses because these are not present
/// in generator-norm. The last entry is a regex matching all possible tags
/// of the type `<xxx_xxx>`; every earlier entry is a literal.
pub const TAGS_TBR: [&str; 12] = [
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

/// The chunk-tag suffixes four of the five Java classes declare and none of
/// them reads; the enhancers select on morphological readings instead.
pub const CHUNK_BEGIN_SUFFIX: &str = "-B";
pub const CHUNK_INSIDE_SUFFIX: &str = "-I";

/// The regex form of the last entry of [`TAGS_TBR`], which [`remove_tags`]
/// treats as a pattern rather than a literal.
static TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(TAGS_TBR[TAGS_TBR.len() - 1]).expect("tags_tbr trailing pattern"));

/// The literal entries of [`TAGS_TBR`], longest first. `+Err/Orth` is a
/// prefix of its four `+Err/Orth-*` siblings, so the order is what keeps a
/// shorter entry from consuming a longer one and stranding its suffix.
static TAGS_TBR_LITERALS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut literals: Vec<&'static str> = TAGS_TBR[..TAGS_TBR.len() - 1].to_vec();
    literals.sort_by_key(|tag| std::cmp::Reverse(tag.len()));
    literals
});

/// A failure of the generator seam. This is the only step covered by the
/// `catch (IOException | InterruptedException)` that wraps the Java body:
/// it is printed and abandons the rest of the run, whatever the topic's
/// [`Unchecked`] policy says.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct GeneratorFailure(String);

/// What a topic does with an exception Java would not have caught — an
/// out-of-range substring, a non-numeric `Word` record, a span tag missing
/// from the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unchecked {
    /// The try block catches only the two checked exceptions, so anything
    /// else leaves `process`.
    Propagate,
    /// The Java handler is a bare `catch (Exception e)`, so nothing escapes.
    Swallow,
}

/// Which of the shared log statements the topic's Java class kept. The five
/// classes were edited apart over time and no longer log the same things.
#[derive(Debug, Clone, Copy)]
pub struct Trace {
    /// `"spantag before adding distractors:{}"`, before the mc attributes go in.
    pub span_tag: bool,
    /// `"Enhancement={}"`, after the mc enhancement reaches the CAS.
    pub enhancement: bool,
    /// `"possibleforms= {}"`, before the cloze attribute goes in.
    pub possible_forms: bool,
}

/// The preposition-hint pass, which only the singular-noun topic runs: it
/// links a noun span back to the preposition that governs it, and drops a
/// token outright when any of its readings is an unlikely part of speech.
#[derive(Debug, Clone, Copy)]
pub struct HintRules {
    /// Marks a reading as a hint in its own right.
    pub hint: &'static str,
    /// The tags allowed between a hint and the noun it governs.
    pub valid_hint: &'static str,
    /// Note that the whole token is excluded when this matches any reading,
    /// even when another reading is valid.
    pub exclude: fn(&str) -> bool,
}

/// Everything one topic changes about the shared enhancement pass.
#[derive(Debug, Clone, Copy)]
pub struct TopicSpec {
    /// Names the topic in the start and finish log lines.
    pub label: &'static str,
    /// The second CSS class on every enhanced span.
    pub span_class: &'static str,
    /// Part-of-speech pattern a reading must match.
    pub pos: &'static str,
    /// Number, case, mood or non-finite pattern a reading must also match.
    pub selector: &'static str,
    /// Set only for the topic that runs the preposition-hint pass.
    pub hints: Option<HintRules>,
    /// Whether the mc generator input is built from the reading with its
    /// `+<sme>` language tag already removed.
    pub strip_lang_tag: bool,
    /// `"This reading will be used={}"`, kept by one topic only.
    pub log_chosen_reading: bool,
    pub unchecked: Unchecked,
    pub trace: Trace,
}

/// `String.substring(0, s.indexOf(marker))`, including the
/// `StringIndexOutOfBoundsException` Java raises when the marker is absent
/// and `indexOf` therefore returns -1.
pub fn substring_to_index_of(s: &str, marker: &str) -> Result<String> {
    match s.find(marker) {
        Some(i) => Ok(s[..i].to_string()),
        None => bail!("begin 0, end -1, length {}", s.chars().count()),
    }
}

/// `substring(0, indexOf("+"))` and `substring(indexOf("+") + 1, length() -
/// 1)`: the lemma, and the analyses cut one character short of the end. The
/// upper bound is unconditional, so the last analysis tag always loses its
/// last character.
pub fn split_lemma_dropping_last(reading_str: &str) -> Result<(String, String)> {
    let chars: Vec<char> = reading_str.chars().collect();
    let length = chars.len();
    let plus = match chars.iter().position(|c| *c == '+') {
        Some(index) => index,
        // substring(0, -1) when there is no "+"
        None => bail!("begin 0, end -1, length {}", length),
    };
    if plus + 1 > length - 1 {
        bail!("begin {}, end {}, length {}", plus + 1, length - 1, length);
    }
    let lemma_str: String = chars[..plus].iter().collect();
    let an_tmp: String = chars[plus + 1..length - 1].iter().collect();
    Ok((lemma_str, an_tmp))
}

/// The cloze line the topics assemble from a split reading: drop the
/// language tag, cut the analyses at their syntactic tag when there is one,
/// rejoin them to the lemma on a `+`, and strip the tags the generator will
/// not accept.
pub fn cloze_line(lemma_str: &str, an_tmp: &str) -> String {
    let mut analyses_str = an_tmp.replace("+<sme>", "");
    // the syntactic tag and the separator in front of it are cut away, and
    // only when the reading carries one
    if let Some(cut) = cut_before_syntactic_tag(&analyses_str) {
        analyses_str.truncate(cut);
    }
    // if analyses contains tags_tbr, remove it
    remove_tags(&format!("{}+{}\n", lemma_str, analyses_str))
}

/// The correct-answer line every mc builder appends last: the reading with
/// its syntactic tag cut away, or the whole reading when it carries none.
/// Removing the `@` is conditional because `substring(0, -2)` would
/// otherwise raise "String index out of range".
pub fn correct_answer_line(reading_str: &str) -> String {
    match cut_before_syntactic_tag(reading_str) {
        Some(cut) => reading_str[..cut].to_string() + "\n",
        None => reading_str.to_string() + "\n",
    }
}

/// Where a reading has to be cut to lose its syntactic tag and the separator
/// in front of it. `None` when the reading carries no tag, or opens with one
/// and so has no separator to drop.
fn cut_before_syntactic_tag(reading: &str) -> Option<usize> {
    let at = reading.find('@')?;
    reading[..at].char_indices().next_back().map(|(cut, _)| cut)
}

/// The mc generator input a fixed-table topic builds: one row per distractor
/// analysis on the reading's own lemma, then the correct answer, with the
/// tags the generator will not accept stripped from the whole block.
pub fn lemma_distractors(reading_str: &str, distract_forms: &[&str]) -> Result<String> {
    // Get lemma from the reading
    let lemma = match reading_str.find('+') {
        Some(index) => &reading_str[..index],
        // substring(0, -1) when there is no "+"
        None => bail!("begin 0, end -1, length {}", reading_str.chars().count()),
    };
    let mut generation_input = String::new();
    // Assign distractorforms from the array
    for form in distract_forms {
        generation_input = generation_input + lemma + "+" + form + "\n";
    }
    // add reading_str as last element in generationInput which will be used
    // as correct_answer
    generation_input += &correct_answer_line(reading_str);
    // if generationInput contains tags_tbr, remove it
    Ok(remove_tags(&generation_input))
}

// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn+2]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.remove-tags-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.remove-tags-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.remove-tags-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.remove-tags-fn+2]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.remove-tags-fn+2]
pub fn remove_tags(input_str: &str) -> String {
    let mut input_str = input_str.to_string();
    for tag in TAGS_TBR_LITERALS.iter() {
        input_str = input_str.replace(tag, "");
    }
    // every `<...>` tag goes, not only the first spelling to match
    TAG_REGEX.replace_all(&input_str, "").into_owned()
}

/// The reading a token was accepted on, plus the hint tag the same pass
/// picked up when the topic looks for one.
#[derive(Default)]
struct Selection {
    valid: bool,
    reading: String,
    lemma: String,
    hint_tag: String,
}

/// The compiled form of a topic's reading patterns.
struct Matcher {
    pos: Regex,
    selector: Regex,
    hint: Option<Regex>,
    valid_hint: Option<Regex>,
    exclude: Option<fn(&str) -> bool>,
}

impl Matcher {
    fn new(spec: &TopicSpec) -> Result<Self> {
        let pos = Regex::new(spec.pos)?;
        let selector = Regex::new(spec.selector)?;
        let (hint, valid_hint, exclude) = match spec.hints {
            Some(rules) => (
                Some(Regex::new(rules.hint)?),
                Some(Regex::new(rules.valid_hint)?),
                Some(rules.exclude),
            ),
            None => (None, None, None),
        };
        Ok(Matcher {
            pos,
            selector,
            hint,
            valid_hint,
            exclude,
        })
    }

    /// Select from all readings the first occurrence that is matching pos
    /// and number. A topic with hint rules also decides here whether a hint
    /// carried over from an earlier token is still valid, whether this token
    /// is itself a hint, and whether one unlikely reading disqualifies the
    /// whole token.
    fn select(&self, cgt: &CgToken, hint_is_valid: &mut bool) -> Selection {
        let mut found = Selection::default();
        for reading in &cgt.readings {
            let mut current = String::new();
            for rtag in reading {
                current = current + "+" + rtag;
            }
            let on_topic = self.pos.is_match(&current) && self.selector.is_match(&current);

            // an invalid hint doesn't match the valid hint pattern and also
            // the pos and number hint patterns
            if let Some(valid_hint) = &self.valid_hint
                && *hint_is_valid
                && !valid_hint.is_match(&current)
                && !on_topic
            {
                *hint_is_valid = false;
            }
            // determine if the current tag is a hint; remove the first "+"
            // and quotes and replace "+" with a "-"
            if let Some(hint) = &self.hint
                && found.hint_tag.is_empty()
                && hint.is_match(&current)
            {
                found.hint_tag = current[1..].replace('"', "").replace('+', "-");
                *hint_is_valid = true;
            }
            // don't consider readings that match the exclude pattern, to
            // filter out unlikely readings (e.g. "и" is a CC in almost all
            // cases, the probability that it is a N is very low)
            if let Some(exclude) = self.exclude
                && exclude(&current)
            {
                found.valid = false;
                break;
            }
            if !found.valid && on_topic {
                found.valid = true;
                // remove the first "+" and quotes
                found.reading = current[1..].replace('"', "");
                // the lemma is the first element of the reading string
                found.lemma = found.reading.split('+').next().unwrap_or("").to_string();
            }
        }
        found
    }
}

/// What the token scan accumulates: the span-id frequency counter, the map
/// tying offsets to the span being built, one generator-input buffer per
/// Java writer, and the running preposition hint.
///
/// Both Java writers targeted the same truncated generator input file, so
/// only the branch that is actually taken contributes content.
#[derive(Default)]
struct Scan {
    class_counts: HashMap<String, i32>,
    word_to_span_map: HashMap<Word, SpanTag>,
    generator_input: String,
    generator_input_cloze: String,
    hint_id: String,
    hint_distance: i32,
    hint_is_valid: bool,
}

/// One `process` call: the topic it runs for and the generator inputs that
/// topic builds.
struct Run<'a> {
    spec: &'a TopicSpec,
    matcher: Matcher,
    mc: bool,
    cloze: bool,
    forms: &'a dyn Fn(&str) -> Result<String>,
    analyses: &'a dyn Fn(&str) -> Result<String>,
}

impl Run<'_> {
    /// The body the Java wraps in its try block: walk the tokens, then hand
    /// whichever generator input the activity called for to the FST and read
    /// the forms back onto the spans.
    fn collect(&self, doc: &mut Document, elapsed: &mut f64) -> Result<()> {
        let cg_tokens = doc.cg_tokens.clone();
        let mut scan = Scan::default();

        // go through tokens
        for cgt in &cg_tokens {
            let found = self.matcher.select(cgt, &mut scan.hint_is_valid);
            if found.valid {
                self.enhance_token(doc, cgt, &found, &mut scan);
            } else if !found.hint_tag.is_empty() {
                scan.hint_distance = 0;
                emit_hint_span(doc, cgt, &found.hint_tag, &mut scan);
            }
            scan.hint_distance += 1;
        }

        if self.mc {
            // generate distractors only when the activity is "mc" (multiple
            // choice)
            let started = Instant::now();
            let output = generate_forms(&scan.generator_input)?;
            let trace = self.spec.trace;
            attach_distractors(doc, trace, &output, &mut scan.word_to_span_map)?;
            *elapsed += started.elapsed().as_secs_f64() * 1000.0;
        }

        if self.cloze {
            // generate possible forms from lemma and analyses; the Java
            // rebuilds the identical shell pipeline here without logging it
            let started = Instant::now();
            let output = generate_forms(&scan.generator_input_cloze)?;
            let trace = self.spec.trace;
            attach_possible_forms(doc, trace, &output, &mut scan.word_to_span_map)?;
            *elapsed += started.elapsed().as_secs_f64() * 1000.0;
        }

        Ok(())
    }

    /// Build the span for an accepted token and either enhance in place or
    /// queue the token for the generator.
    fn enhance_token(&self, doc: &mut Document, cgt: &CgToken, found: &Selection, scan: &mut Scan) {
        if self.spec.log_chosen_reading {
            info!("This reading will be used={}", found.reading);
        }
        // id's with the "+" symbol have to be escaped, thats why we use a
        // "-" instead. The "<" and ">" symbols also cause problems because
        // these are the tag opening / closing symbol.
        let span_reading_string = found
            .reading
            .replace('+', "-")
            .replace('<', "x")
            .replace('>', "y");
        let count = bump(&mut scan.class_counts, &span_reading_string);

        // create a word with begin and end of the current CGToken
        let word = Word::new(cgt.begin, cgt.end);

        let id = enhancer_utils::get_id(&format!("teaksta-span-{}", span_reading_string), count);
        let mut span_tag = SpanTag::new(id, &[TOKEN_CLASS, self.spec.span_class]);
        span_tag.add_attribute("lemma", &found.lemma);

        // only add the hint ID if the distance is allowed and the hint still
        // valid; distance = 1 would allow no tokens in between
        if !scan.hint_id.is_empty() && scan.hint_distance < 4 && scan.hint_is_valid {
            span_tag.add_attribute("hintid", &scan.hint_id);
        }
        // reset the validity of a hint
        scan.hint_is_valid = false;

        scan.word_to_span_map.insert(word, span_tag.clone());
        self.queue_for_generator(doc, &word, &span_tag, found, scan);
    }

    /// mc and cloze both defer the span to the generator, writing one record
    /// per token; every other activity enhances straight away.
    ///
    /// A reading the topic cannot turn into a generator input — one carrying
    /// no syntactic tag, or a case marker without the number the cut needs —
    /// is dropped on its own rather than abandoning the enhancement of the
    /// whole document.
    fn queue_for_generator(
        &self,
        doc: &mut Document,
        word: &Word,
        span_tag: &SpanTag,
        found: &Selection,
        scan: &mut Scan,
    ) {
        if self.mc {
            // generate the distractors, with lemma, gender, animacy, number
            // and case (needed for the form generator)
            let reading = match self.spec.strip_lang_tag {
                true => found.reading.replace("+<sme>", ""),
                false => found.reading.clone(),
            };
            match (self.forms)(&reading) {
                Ok(block) => push_record(&mut scan.generator_input, &block, word),
                Err(e) => debug!("no distractor input for {}: {}", reading, e),
            }
        } else if self.cloze {
            // extract lemma and analyses from the reading
            match (self.analyses)(&found.reading) {
                Ok(block) => push_record(&mut scan.generator_input_cloze, &block, word),
                Err(e) => debug!("no cloze input for {}: {}", found.reading, e),
            }
        } else {
            push_enhancement(doc, word.begin, word.end, span_tag);
        }
    }
}

/// Run one topic's enhancement pass over `doc`. The body of every
/// `Vislcg3*Enhancer::process`; see [`TopicSpec`] for what each topic
/// changes about it.
pub fn run(
    doc: &mut Document,
    spec: &TopicSpec,
    mode: Mode,
    morphological_forms: &dyn Fn(&str) -> Result<String>,
    lemma_and_analyses: &dyn Fn(&str) -> Result<String>,
) -> Result<()> {
    // stop processing if the client has requested it
    if !cas_utils::is_valid(doc) {
        return Ok(());
    }
    info!("Starting {} enhancement {}.", spec.label, mode.name());

    let mut elapsed_generating: f64 = 0.0;
    let start_time = Instant::now();

    let pass = Run {
        spec,
        matcher: Matcher::new(spec)?,
        mc: mode == Mode::Mc,
        cloze: mode == Mode::Cloze,
        forms: morphological_forms,
        analyses: lemma_and_analyses,
    };

    // Everything the Java body wraps in try { } catch (IOException |
    // InterruptedException) { printStackTrace(); }. The generator seam is
    // the only step that raises one of those two; whether anything else
    // escapes is the topic's own policy.
    if let Err(e) = pass.collect(doc, &mut elapsed_generating) {
        if spec.unchecked == Unchecked::Propagate && !e.is::<GeneratorFailure>() {
            return Err(e);
        }
        eprintln!("{}", e);
    }

    info!("Finished {} enhancement.", spec.label);
    let end_time = start_time.elapsed();

    info!("Total execution time: {} seconds.", end_time.as_secs_f64());
    info!(
        "Generating the distractforms takes in total: {} seconds.",
        elapsed_generating * 0.001
    );
    Ok(())
}

/// The output from the generator is used to create distractors and is
/// placed into the right place in the span tag. Afterwards an enhancement
/// with the span tag is created and passed to the cas.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-distractors-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-distractors-fn+3]
pub fn attach_distractors(
    doc: &mut Document,
    trace: Trace,
    cg3_generator_output: &str,
    word_to_span_map: &mut HashMap<Word, SpanTag>,
) -> Result<()> {
    let mut generator_output = String::new();
    let mut distractforms = String::new();
    let mut answer = String::new();

    for raw_line in cg3_generator_output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        // generator output was processed, all distractors are created;
        // assign the distractors to the correct span from the wordToSpanMap
        else if line.starts_with("Word") {
            // only enhance tokens with more than one distractor form
            if !distractforms.is_empty() {
                let word = word_record(line)?;
                let span_tag = span_for(word_to_span_map, &word)?;
                if trace.span_tag {
                    info!("spantag before adding distractors:{}", span_tag);
                }
                span_tag.add_attribute("distractors", &distractforms);
                span_tag.add_attribute("answer", &answer);
                let e = push_enhancement(doc, word.begin, word.end, span_tag);
                if trace.enhancement {
                    info!("Enhancement={:?}", e);
                }
                // the block belongs to this token alone: a further Word
                // record before the next marker has no forms of its own
                distractforms.clear();
                answer.clear();
            }
        }
        // the marker (ñôŃßĘńŠē) was found, begin to process the generator
        // output, create distractors
        else if line.contains(MARKER) {
            let go = std::mem::take(&mut generator_output);
            answer = go
                .split_whitespace()
                .next_back()
                .unwrap_or_default()
                .to_string();
            let (forms, unique) = collect_forms(&go);
            // exclude the distractor if its only one, you need at least 2
            // distractors for mc
            distractforms = match unique < 2 {
                true => String::new(),
                false => forms,
            };
        }
        // the generator output for the current token is not fully extracted
        // from the stream yet
        else {
            generator_output = generator_output + line + " ";
        }
    }

    Ok(())
}

/// Cloze counterpart of the distractor reader: every generable form is
/// attached to the span, with no answer singled out and no minimum count.
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:def:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-enhancer.vislcg3-noun-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-noun-pl-enhancer.vislcg3-noun-pl-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-verb-conjugation-enhancer.vislcg3-verb-conjugation-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-con-neg-enhancer.vislcg3-con-neg-enhancer.generate-span-tag-with-possible-forms-fn+3]
// [spec:teaksta:sem:sme.src.main.java.werti.uima.enhancer.vislcg3-infinite-verb-enhancer.vislcg3-infinite-verb-enhancer.generate-span-tag-with-possible-forms-fn+3]
pub fn attach_possible_forms(
    doc: &mut Document,
    trace: Trace,
    cg3_generator_output: &str,
    word_to_span_map: &mut HashMap<Word, SpanTag>,
) -> Result<()> {
    let mut generator_output = String::new();
    let mut possible_forms = String::new();

    for raw_line in cg3_generator_output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        // generator output was processed, all possible forms are created;
        // assign the possible forms to the correct span from the
        // wordToSpanMap
        else if line.starts_with("Word") {
            if !possible_forms.is_empty() {
                let word = word_record(line)?;
                let span_tag = span_for(word_to_span_map, &word)?;
                if trace.possible_forms {
                    info!("possibleforms= {}", possible_forms);
                }
                span_tag.add_attribute("possibleforms", &possible_forms);
                push_enhancement(doc, word.begin, word.end, span_tag);
                // the block belongs to this token alone: a further Word
                // record before the next marker has no forms of its own
                possible_forms.clear();
            }
        }
        // the marker (ñôŃßĘńŠē) was found, begin to process the generator
        // output, create the possible forms
        else if line.contains(MARKER) {
            let go = std::mem::take(&mut generator_output);
            possible_forms = collect_forms(&go).0;
        }
        // the generator output for the current token is not fully extracted
        // from the stream yet
        else {
            generator_output = generator_output + line + " ";
        }
    }

    Ok(())
}

/// The distinct surface forms in one closed generator-output block, in
/// first-seen order, with the count the caller needs to apply its minimum.
/// Forms that could not be generated are excluded, as well as input strings
/// of the iFST; the set's purpose is to filter out duplicates.
fn collect_forms(block: &str) -> (String, usize) {
    let mut kept: Vec<&str> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for word in block.split_whitespace() {
        if !word.contains(['+', '-']) && seen.insert(word) {
            kept.push(word);
        }
    }
    (kept.join(" "), seen.len())
}

/// The `Word <begin> <end>` record the generator input carried through, read
/// back with the index and parse failures Java raises on a malformed line.
fn word_record(line: &str) -> Result<Word> {
    let line_parts: Vec<&str> = line.split_whitespace().collect();
    if line_parts.len() < 3 {
        bail!("Index 2 out of bounds for length {}", line_parts.len());
    }
    let begin = parse_offset(line_parts[1])?;
    let end = parse_offset(line_parts[2])?;
    Ok(Word::new(begin, end))
}

/// `Integer.parseInt`: a non-numeric field raises NumberFormatException,
/// which the callers do not catch.
fn parse_offset(field: &str) -> Result<usize> {
    field
        .parse::<usize>()
        .map_err(|_| anyhow!("For input string: \"{}\"", field))
}

/// The Java lookup returns null for an unregistered offset pair and the next
/// call raises a NullPointerException.
fn span_for<'a>(
    word_to_span_map: &'a mut HashMap<Word, SpanTag>,
    word: &Word,
) -> Result<&'a mut SpanTag> {
    match word_to_span_map.get_mut(word) {
        Some(span_tag) => Ok(span_tag),
        None => bail!("no span tag for {}", word),
    }
}

/// One generator-input record: the analyses to generate, the marker that
/// separates this token's block from the next, and the word offsets that
/// assign the correct forms to the correct span.
fn push_record(writer: &mut String, block: &str, word: &Word) {
    writer.push_str(block);
    writer.push_str(MARKER);
    writer.push('\n');
    writer.push_str(&word.to_string());
    writer.push('\n');
}

/// make new enhancement, pass it to the cas
fn push_enhancement(
    doc: &mut Document,
    begin: usize,
    end: usize,
    span_tag: &SpanTag,
) -> Enhancement {
    let e = Enhancement {
        relevant: true,
        begin,
        end,
        enhance_start: span_tag.start_tag(),
        enhance_end: span_tag.end_tag().to_string(),
    };
    // update CAS
    doc.enhancements.push(e.clone());
    e
}

/// The hint span a preposition gets in its own right, and the id the noun
/// that follows will point back at.
fn emit_hint_span(doc: &mut Document, cgt: &CgToken, hint_tag: &str, scan: &mut Scan) {
    // create a word with begin and end of the current CGToken
    let word = Word::new(cgt.begin, cgt.end);
    let count = bump(&mut scan.class_counts, hint_tag);
    scan.hint_id = enhancer_utils::get_id(&format!("teaksta-span-{}", hint_tag), count);
    let span_tag = SpanTag::new(scan.hint_id.clone(), &[HINT_CLASS]);
    push_enhancement(doc, word.begin, word.end, &span_tag);
}

/// Count one sighting of a span id and report the running total: the first
/// stores a fresh counter, which already stands at one, and later ones
/// increment it in place.
fn bump(class_counts: &mut HashMap<String, i32>, key: &str) -> i32 {
    let count = class_counts.entry(key.to_string()).or_insert(0);
    *count += 1;
    *count
}

/// The generator seam. Its failures are the IOException and
/// InterruptedException the Java try block catches.
fn generate_forms(generator_input: &str) -> Result<String> {
    MorphoPipeline::shared()
        .generate(generator_input)
        .map_err(|e| GeneratorFailure(e.to_string()).into())
}

#[cfg(test)]
#[path = "cg_enhancer_tests.rs"]
mod tests;
