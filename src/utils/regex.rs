use regex::RegexBuilder;
use crate::utils::ToolError;

// pub fn search_text(
//     text: &str,
//     pattern: &str,
//     max_matches: usize,
//     max_match_len: usize,
// ) -> Result<Vec<String>, ToolError> {
//     let re = RegexBuilder::new(pattern)
//         .dot_matches_new_line(true)
//         .build()
//         .map_err(|_| ToolError::InvalidInput)?;

//     let matches: Vec<String> = re
//         .find_iter(text)
//         .take(max_matches)
//         .map(|m| {
//             let s = m.as_str();
//             match s.char_indices().nth(max_match_len) {
//                 Some((byte_idx, _)) => s[..byte_idx].to_string(),
//                 None => s.to_string(),
//             }
//         })
//         .collect();

//     Ok(matches)
// }

pub fn search_text(
    text: &str,
    pattern: &str,
    max_matches: usize,
    max_total_len: usize,
) -> Result<Vec<String>, ToolError> {
    let re = RegexBuilder::new(pattern)
        .dot_matches_new_line(true)
        .build()
        .map_err(|_| ToolError::InvalidInput)?;

    let mut matches: Vec<String> = Vec::new();
    let mut remaining = max_total_len;

    for m in re.find_iter(text).take(max_matches) {
        if remaining == 0 {
            break;
        }

        let s = m.as_str();
        let char_count = s.chars().count();

        if char_count <= remaining {
            matches.push(s.to_string());
            remaining -= char_count;
        } else {
            // truncate this match to fit the remaining budget, then stop
            let byte_idx = s
                .char_indices()
                .nth(remaining)
                .map(|(idx, _)| idx)
                .unwrap_or(s.len());
            matches.push(s[..byte_idx].to_string());
            remaining = 0;
        }
    }

    Ok(matches)
}