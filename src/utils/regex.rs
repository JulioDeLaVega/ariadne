use regex::RegexBuilder;
use crate::utils::ToolError;

pub fn search_text(
    text: &str,
    pattern: &str,
    max_matches: usize,
    max_match_len: usize,
) -> Result<Vec<String>, ToolError> {
    let re = RegexBuilder::new(pattern)
        .dot_matches_new_line(true)
        .build()
        .map_err(|_| ToolError::InvalidInput)?;

    let matches: Vec<String> = re
        .find_iter(text)
        .take(max_matches)
        .map(|m| {
            let s = m.as_str();
            match s.char_indices().nth(max_match_len) {
                Some((byte_idx, _)) => s[..byte_idx].to_string(),
                None => s.to_string(),
            }
        })
        .collect();

    Ok(matches)
}