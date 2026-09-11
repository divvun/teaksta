use anyhow::{Result, anyhow};

use crate::types::{Spanned, index_order};

pub mod flow;
pub mod relevance;
pub mod sentences;
pub mod tokenizer;
pub mod vislcg3;

/// The document text with everything outside the given spans blanked to
/// spaces, at byte-for-byte identical offsets. The tokeniser and the sentence
/// detector both run over a masked buffer rather than over the real text —
/// over the relevant stretches and over the tokens respectively — so what
/// they find sits at offsets the document can be annotated with directly.
fn mask_to_spans<T: Spanned>(text: &str, spans: &[T]) -> Result<String> {
    let mut rtext = vec![b' '; text.len()];

    for position in index_order(spans) {
        let (begin, end) = (spans[position].begin(), spans[position].end());
        let covered = text
            .get(begin..end)
            .ok_or_else(|| anyhow!("span {begin}..{end} is not within the document"))?;
        rtext[begin..end].copy_from_slice(covered.as_bytes());
    }

    Ok(String::from_utf8(rtext)?)
}
