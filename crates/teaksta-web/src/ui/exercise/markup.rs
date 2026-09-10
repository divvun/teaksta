//! Reading one enhanced page into the blocks and tokens an exercise renders.
//!
//! The enhancer hands back a whole HTML page in which the word forms a topic
//! matched are wrapped in a span carrying the class `teaksta-token` plus, on
//! a hit, a class naming the topic. Everything about that markup is read
//! through element and attribute structure: the class attribute's spacing is
//! not part of the contract, so membership of the class list decides, never
//! the attribute's text.
//!
//! The page is taken apart into blocks of inline runs so an exercise can put
//! its own controls where the tokens were. A run between two tokens is the
//! page's own inline markup, kept verbatim; an inline element split by a
//! token is closed and re-opened around it, which reads identically because
//! only inline elements are ever split.

/// The class the enhancer puts on every span a learner can work on.
pub const TOKEN_CLASS: &str = "teaksta-token";

/// The prefix of the class naming the topic a hit belongs to, so the
/// `Substantive` activity marks its hits `teaksta-Substantive`.
pub const TOPIC_PREFIX: &str = "teaksta-";

/// The class the generic token enhancer marks a hit with when the topic
/// contributes no class of its own.
pub const GENERIC_HIT_CLASS: &str = "teaksta-hit";

/// Elements whose content never reaches the learner.
const DROPPED_ELEMENTS: &[&str] = &["script", "style", "noscript", "template"];

/// Elements that end the run of text they interrupt.
const BLOCK_ELEMENTS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
    "ul",
];

/// Elements that hold nothing, so they never open an inline context.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// How one block of the enhanced page reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockKind {
    #[default]
    Paragraph,
    Heading,
    Item,
    Quote,
}

/// One run inside a block: either the page's own markup or a token the
/// exercise replaces with a control of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Html(String),
    Token(usize),
}

/// One block of the enhanced page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub pieces: Vec<Piece>,
}

/// One enhanced span: the word form as it stands in the page, plus whatever
/// the topic's enhancer attached to it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenSpan {
    pub id: String,
    pub classes: Vec<String>,
    pub text: String,
    pub lemma: Option<String>,
    pub distractors: Vec<String>,
    pub answer: Vec<String>,
    pub possible_forms: Vec<String>,
}

impl TokenSpan {
    /// Whether this token is one of the topic's hits rather than a plain word
    /// the learner is offered alongside them.
    pub fn is_hit(&self, topic: &str) -> bool {
        let named = format!("{TOPIC_PREFIX}{topic}");
        !topic.is_empty()
            && self
                .classes
                .iter()
                .any(|class| class == &named || class == GENERIC_HIT_CLASS)
    }

    /// Every form that counts as the right answer, lowercased. Parallel forms
    /// are why this is a list: a topic can generate several forms the learner
    /// could not tell apart from the text, and each of them is right. The
    /// form standing in the page is always among them, because it is the one
    /// the enhancer took the exercise from.
    pub fn accepted_forms(&self) -> Vec<String> {
        let mut forms: Vec<String> = Vec::new();
        let candidates = self
            .possible_forms
            .iter()
            .chain(self.answer.iter())
            .chain(std::iter::once(&self.text));

        for candidate in candidates {
            let form = candidate.to_lowercase();
            if !form.is_empty() && !forms.contains(&form) {
                forms.push(form);
            }
        }

        forms
    }

    /// Whether a written or chosen form is one of the accepted ones, which
    /// ignores case exactly as the legacy engine did.
    pub fn accepts(&self, guess: &str) -> bool {
        let guess = guess.trim().to_lowercase();
        !guess.is_empty() && self.accepted_forms().contains(&guess)
    }

    /// The forms to show when a learner gives up on a token, separated the way
    /// the legacy hint separated them.
    pub fn hint(&self) -> String {
        if !self.possible_forms.is_empty() {
            self.possible_forms.join("/")
        } else if !self.answer.is_empty() {
            self.answer.join("/")
        } else {
            self.text.clone()
        }
    }

    fn read(attributes: &[(String, String)], inner: &str) -> Self {
        let value = |key: &str| {
            attributes
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value.as_str())
                .filter(|value| !value.is_empty())
        };

        Self {
            id: value("id").unwrap_or_default().to_string(),
            classes: value("class")
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_string)
                .collect(),
            text: strip_tags(inner),
            lemma: value("lemma").map(str::to_string),
            distractors: split_forms(value("distractors")),
            answer: split_forms(value("answer")),
            possible_forms: split_forms(value("possibleforms")),
        }
    }
}

/// One enhanced page, ready to render.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Markup {
    blocks: Vec<Block>,
    tokens: Vec<TokenSpan>,
}

impl Markup {
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn tokens(&self) -> &[TokenSpan] {
        &self.tokens
    }

    pub fn token(&self, index: usize) -> Option<&TokenSpan> {
        self.tokens.get(index)
    }

    /// How many tokens belong to the topic, which is how many a learner has to
    /// find or fill in.
    pub fn hits(&self, topic: &str) -> usize {
        self.tokens.iter().filter(|it| it.is_hit(topic)).count()
    }
}

/// Read an enhanced page. Only the body is kept: the head carries the fetched
/// page's own title and base URL, neither of which this client shows.
pub fn parse(page: &str) -> Markup {
    let mut reader = Reader::default();
    let mut rest = body_of(page);

    while let Some(at) = rest.find('<') {
        reader.push_text(&rest[..at]);
        let tail = &rest[at..];

        let Some((tag, after)) = read_tag(tail) else {
            reader.push_text("<");
            rest = &tail[1..];
            continue;
        };

        match tag {
            Lexed::Skip => rest = after,
            Lexed::Close { name } => {
                reader.close(&name);
                rest = after;
            }
            Lexed::Open {
                name,
                attributes,
                raw,
                closed,
            } => {
                if DROPPED_ELEMENTS.contains(&name.as_str()) && !closed {
                    rest = take_element(after, &name).1;
                } else if name == "span" && is_token(&attributes) {
                    let (inner, beyond) = take_element(after, "span");
                    reader.push_token(TokenSpan::read(&attributes, inner));
                    rest = beyond;
                } else {
                    reader.open(&name, raw, closed);
                    rest = after;
                }
            }
        }
    }

    reader.push_text(rest);
    reader.finish()
}

/// Whether a span's class list makes it an exercisable token. Membership of
/// the list decides, so any spacing the enhancer writes reads the same.
fn is_token(attributes: &[(String, String)]) -> bool {
    attributes
        .iter()
        .filter(|(name, _)| name == "class")
        .any(|(_, value)| value.split_whitespace().any(|class| class == TOKEN_CLASS))
}

fn split_forms(value: Option<&str>) -> Vec<String> {
    value
        .map(|value| value.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

/// The body of a page, or the whole of it when there is no body element.
fn body_of(page: &str) -> &str {
    let lowered = page.to_ascii_lowercase();
    let Some(start) = lowered.find("<body") else {
        return page;
    };
    let Some(open) = page[start..].find('>') else {
        return page;
    };

    let from = start + open + 1;
    let to = lowered[from..]
        .rfind("</body")
        .map_or(page.len(), |at| from + at);

    &page[from..to]
}

/// One inline element still open where the page is being read, kept so a
/// token splitting it can close and re-open it.
struct Level {
    name: String,
    open: String,
    html: String,
}

/// Accumulates blocks while the page is walked.
#[derive(Default)]
struct Reader {
    blocks: Vec<Block>,
    tokens: Vec<TokenSpan>,
    kind: BlockKind,
    pieces: Vec<Piece>,
    text: String,
    levels: Vec<Level>,
}

impl Reader {
    fn buffer(&mut self) -> &mut String {
        match self.levels.last_mut() {
            Some(level) => &mut level.html,
            None => &mut self.text,
        }
    }

    fn push_text(&mut self, text: &str) {
        if !text.is_empty() {
            self.buffer().push_str(text);
        }
    }

    fn open(&mut self, name: &str, raw: &str, closed: bool) {
        if BLOCK_ELEMENTS.contains(&name) {
            self.finish_block();
            self.kind = block_kind(name);
            return;
        }
        if closed || VOID_ELEMENTS.contains(&name) {
            self.push_text(raw);
            return;
        }

        self.levels.push(Level {
            name: name.to_string(),
            open: raw.to_string(),
            html: String::new(),
        });
    }

    fn close(&mut self, name: &str) {
        if BLOCK_ELEMENTS.contains(&name) {
            self.finish_block();
            return;
        }

        // A stray end tag is dropped, as a browser drops one.
        let Some(at) = self.levels.iter().rposition(|level| level.name == name) else {
            return;
        };

        while self.levels.len() > at {
            let level = self.levels.pop().expect("a level the search just found");
            let closed = format!("{}{}</{}>", level.open, level.html, level.name);
            self.buffer().push_str(&closed);
        }
    }

    /// The inline run read since the last piece, with every open element
    /// closed around it so the run stands on its own.
    fn take_run(&mut self) -> Option<String> {
        let mut html = std::mem::take(&mut self.text);
        let mut empty = html.is_empty();

        for level in &mut self.levels {
            html.push_str(&level.open);
            empty = empty && level.html.is_empty();
            html.push_str(&level.html);
            level.html.clear();
        }
        for level in self.levels.iter().rev() {
            html.push_str("</");
            html.push_str(&level.name);
            html.push('>');
        }

        (!empty).then_some(html)
    }

    fn push_token(&mut self, token: TokenSpan) {
        if let Some(html) = self.take_run() {
            self.pieces.push(Piece::Html(html));
        }
        self.tokens.push(token);
        self.pieces.push(Piece::Token(self.tokens.len() - 1));
    }

    fn finish_block(&mut self) {
        if let Some(html) = self.take_run() {
            self.pieces.push(Piece::Html(html));
        }
        self.levels.clear();

        let pieces = std::mem::take(&mut self.pieces);
        if !is_blank(&pieces) {
            self.blocks.push(Block {
                kind: self.kind,
                pieces,
            });
        }
        self.kind = BlockKind::Paragraph;
    }

    fn finish(mut self) -> Markup {
        self.finish_block();
        Markup {
            blocks: self.blocks,
            tokens: self.tokens,
        }
    }
}

/// Whether a run of pieces would put nothing on the page.
fn is_blank(pieces: &[Piece]) -> bool {
    pieces.iter().all(|piece| match piece {
        Piece::Token(_) => false,
        Piece::Html(html) => strip_tags(html).is_empty(),
    })
}

fn block_kind(name: &str) -> BlockKind {
    match name {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => BlockKind::Heading,
        "li" | "dd" | "dt" => BlockKind::Item,
        "blockquote" => BlockKind::Quote,
        _ => BlockKind::Paragraph,
    }
}

/// One lexed tag.
enum Lexed<'a> {
    Open {
        name: String,
        attributes: Vec<(String, String)>,
        raw: &'a str,
        closed: bool,
    },
    Close {
        name: String,
    },
    /// A comment, doctype or processing instruction, which carries nothing.
    Skip,
}

/// Read the tag starting at `input`, with whatever follows it.
fn read_tag(input: &str) -> Option<(Lexed<'_>, &str)> {
    let rest = input.strip_prefix('<')?;

    if let Some(body) = rest.strip_prefix("!--") {
        let end = body.find("-->")?;
        return Some((Lexed::Skip, &body[end + 3..]));
    }
    if rest.starts_with('!') || rest.starts_with('?') {
        let end = rest.find('>')?;
        return Some((Lexed::Skip, &rest[end + 1..]));
    }
    if let Some(body) = rest.strip_prefix('/') {
        let end = body.find('>')?;
        let name = body[..end].trim().to_ascii_lowercase();
        return Some((Lexed::Close { name }, &body[end + 1..]));
    }

    let end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/')?;
    let name = rest[..end].to_ascii_lowercase();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }

    let (attributes, closed, used) = read_attributes(&rest[end..])?;
    let consumed = 1 + end + used;

    Some((
        Lexed::Open {
            name,
            attributes,
            raw: &input[..consumed],
            closed,
        },
        &input[consumed..],
    ))
}

/// Read an open tag's attributes, answering them with whether the tag closed
/// itself and how many bytes it took including its `>`.
fn read_attributes(input: &str) -> Option<(Vec<(String, String)>, bool, usize)> {
    let mut attributes = Vec::new();
    let mut at = 0;

    loop {
        at += leading_space(&input[at..]);
        let rest = input.get(at..)?;

        if rest.starts_with("/>") {
            return Some((attributes, true, at + 2));
        }
        if rest.starts_with('>') {
            return Some((attributes, false, at + 1));
        }

        let end = rest.find(|c: char| c.is_whitespace() || c == '=' || c == '>')?;
        if end == 0 {
            return None;
        }
        let name = rest[..end].to_ascii_lowercase();
        at += end;
        at += leading_space(&input[at..]);

        let Some(rest) = input[at..].strip_prefix('=') else {
            attributes.push((name, String::new()));
            continue;
        };
        at += 1 + leading_space(rest);

        let rest = input.get(at..)?;
        let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'');
        let value = match quote {
            Some(quote) => {
                let end = rest[1..].find(quote)?;
                at += 1 + end + 1;
                &rest[1..1 + end]
            }
            None => {
                let end = rest
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(rest.len());
                at += end;
                &rest[..end]
            }
        };

        attributes.push((name, decode_entities(value)));
    }
}

fn leading_space(input: &str) -> usize {
    input.len() - input.trim_start().len()
}

/// The content of an element whose open tag has just been read, with the page
/// beyond its close tag. Nesting of the same element is counted, so an inner
/// span never ends the outer one.
fn take_element<'a>(input: &'a str, name: &str) -> (&'a str, &'a str) {
    let mut depth = 1usize;
    let mut at = 0usize;

    loop {
        let Some(found) = input[at..].find('<') else {
            return (input, "");
        };
        let start = at + found;

        let Some((tag, after)) = read_tag(&input[start..]) else {
            at = start + 1;
            continue;
        };
        let beyond = input.len() - after.len();

        match tag {
            Lexed::Open {
                name: opened,
                closed: false,
                ..
            } if opened == name => depth += 1,
            Lexed::Close { name: shut } if shut == name => {
                depth -= 1;
                if depth == 0 {
                    return (&input[..start], &input[beyond..]);
                }
            }
            _ => {}
        }

        at = beyond;
    }
}

/// The text an HTML run puts on the page, with runs of space collapsed.
fn strip_tags(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;

    while let Some(at) = rest.find('<') {
        text.push_str(&rest[..at]);
        let tail = &rest[at..];
        match read_tag(tail) {
            Some((_, after)) => rest = after,
            None => {
                text.push('<');
                rest = &tail[1..];
            }
        }
    }
    text.push_str(rest);

    decode_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Decode the entities the enhancer and the pages it reads actually write.
/// Anything else is left alone rather than guessed at.
fn decode_entities(value: &str) -> String {
    if !value.contains('&') {
        return value.to_string();
    }

    let mut out = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];

        let named = [
            ("&amp;", "&"),
            ("&lt;", "<"),
            ("&gt;", ">"),
            ("&quot;", "\""),
            ("&apos;", "'"),
            ("&#39;", "'"),
            ("&nbsp;", " "),
        ]
        .into_iter()
        .find(|(entity, _)| rest.starts_with(entity));

        match named {
            Some((entity, plain)) => {
                out.push_str(plain);
                rest = &rest[entity.len()..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = concat!(
        "<p>Mun oidnen ",
        "<span class=\"teaksta-token teaksta-Substantive\" id=\"a\" lemma=\"viessu\"",
        " possibleforms=\"viesu viesuid\">viesu</span> ikte.</p>"
    );

    #[test]
    fn a_token_is_found_by_class_membership() {
        let markup = parse(TOKEN);

        assert_eq!(markup.tokens().len(), 1);
        assert_eq!(markup.tokens()[0].text, "viesu");
        assert_eq!(markup.tokens()[0].id, "a");
    }

    #[test]
    fn a_hit_is_named_by_its_topic_class() {
        let markup = parse(TOKEN);
        let token = &markup.tokens()[0];

        assert!(token.is_hit("Substantive"));
        assert!(!token.is_hit("VerbConjugation"));
        assert!(!token.is_hit(""));
    }

    #[test]
    fn a_plain_token_belongs_to_no_topic() {
        let markup = parse("<p><span class=\"teaksta-token\" id=\"b\">ikte</span></p>");

        assert!(!markup.tokens()[0].is_hit("Substantive"));
        assert_eq!(markup.hits("Substantive"), 0);
    }

    #[test]
    fn the_generic_hit_class_counts_as_a_hit() {
        let markup = parse("<p><span class=\"teaksta-token teaksta-hit\">go</span></p>");

        assert!(markup.tokens()[0].is_hit("Substantive"));
    }

    #[test]
    fn the_page_around_a_token_is_kept() {
        let markup = parse(TOKEN);
        let block = &markup.blocks()[0];

        assert_eq!(block.kind, BlockKind::Paragraph);
        assert_eq!(block.pieces[0], Piece::Html("Mun oidnen ".to_string()));
        assert_eq!(block.pieces[1], Piece::Token(0));
        assert_eq!(block.pieces[2], Piece::Html(" ikte.".to_string()));
    }

    #[test]
    fn an_inline_element_split_is_reopened() {
        let markup = parse("<p><em>a <span class=\"teaksta-token\">b</span> c</em></p>");
        let pieces = &markup.blocks()[0].pieces;

        assert_eq!(pieces[0], Piece::Html("<em>a </em>".to_string()));
        assert_eq!(pieces[2], Piece::Html("<em> c</em>".to_string()));
    }

    #[test]
    fn blocks_keep_the_page_headings() {
        let markup = parse("<h1>Title</h1><p>Text</p><li>Item</li><blockquote>Q</blockquote>");
        let kinds: Vec<_> = markup.blocks().iter().map(|block| block.kind).collect();

        assert_eq!(
            kinds,
            [
                BlockKind::Heading,
                BlockKind::Paragraph,
                BlockKind::Item,
                BlockKind::Quote
            ]
        );
    }

    #[test]
    fn scripts_and_styles_never_reach_a_block() {
        let markup = parse("<body><script>var a = '<p>x</p>';</script><p>Text</p></body>");

        assert_eq!(markup.blocks().len(), 1);
        assert_eq!(
            markup.blocks()[0].pieces[0],
            Piece::Html("Text".to_string())
        );
    }

    #[test]
    fn only_the_body_of_a_page_is_read() {
        let markup = parse("<html><head><title>Head</title></head><body><p>Body</p></body></html>");

        assert_eq!(markup.blocks().len(), 1);
        assert_eq!(
            markup.blocks()[0].pieces[0],
            Piece::Html("Body".to_string())
        );
    }

    #[test]
    fn attributes_need_no_space_between_them() {
        let markup = parse("<p><span class=\"teaksta-token\"lemma=\"viessu\">viesu</span></p>");

        assert_eq!(markup.tokens()[0].lemma.as_deref(), Some("viessu"));
    }

    #[test]
    fn a_nested_span_never_ends_the_token() {
        let markup = parse("<p><span class=\"teaksta-token\"><span>vie</span>su</span> a</p>");

        assert_eq!(markup.tokens().len(), 1);
        assert_eq!(markup.tokens()[0].text, "viesu");
        assert_eq!(markup.blocks()[0].pieces.len(), 2);
    }

    #[test]
    fn parallel_forms_are_all_accepted() {
        let markup = parse(TOKEN);
        let token = &markup.tokens()[0];

        assert!(token.accepts("viesu"));
        assert!(token.accepts("viesuid"));
        assert!(token.accepts(" VIESUID "));
        assert!(!token.accepts("viessu"));
        assert!(!token.accepts(""));
    }

    #[test]
    fn the_hint_separates_the_parallel_forms() {
        assert_eq!(parse(TOKEN).tokens()[0].hint(), "viesu/viesuid");
    }

    #[test]
    fn the_hint_falls_back_to_the_page_form() {
        let markup = parse("<p><span class=\"teaksta-token\">viesu</span></p>");

        assert_eq!(markup.tokens()[0].hint(), "viesu");
    }

    #[test]
    fn the_answer_attribute_is_accepted_too() {
        let markup = parse("<p><span class=\"teaksta-token\" answer=\"lei leai\">lei</span></p>");

        assert!(markup.tokens()[0].accepts("leai"));
        assert_eq!(markup.tokens()[0].hint(), "lei/leai");
    }

    #[test]
    fn entities_are_decoded_in_text_and_values() {
        let markup = parse("<p><span class=\"teaksta-token\" lemma=\"a&amp;b\">x&amp;y</span></p>");

        assert_eq!(markup.tokens()[0].text, "x&y");
        assert_eq!(markup.tokens()[0].lemma.as_deref(), Some("a&b"));
    }

    #[test]
    fn a_comment_carries_nothing_to_the_page() {
        let markup = parse("<p>a<!-- <span class=\"teaksta-token\">b</span> -->c</p>");

        assert!(markup.tokens().is_empty());
        assert_eq!(markup.blocks()[0].pieces[0], Piece::Html("ac".to_string()));
    }
}
