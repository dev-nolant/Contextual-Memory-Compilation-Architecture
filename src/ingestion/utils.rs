// Copyright (c) 2026 Nolan Taft



pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .trim()
        .to_string()
}


pub fn extract_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|s| s.len() >= 2)
        .map(|s| s.to_lowercase())
        .collect()
}


pub fn contains_any(text: &str, patterns: &[&str]) -> bool {
    let text_lower = text.to_lowercase();
    patterns.iter().any(|pattern| text_lower.contains(pattern))
}


pub fn extract_after_pattern(text: &str, pattern: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    if let Some(pos) = text_lower.find(pattern) {
        Some(text_lower[pos + pattern.len()..].trim().to_string())
    } else {
        None
    }
}


pub fn extract_before_pattern(text: &str, pattern: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    if let Some(pos) = text_lower.find(pattern) {
        Some(text_lower[..pos].trim().to_string())
    } else {
        None
    }
}


pub fn extract_between_patterns(text: &str, start_pattern: &str, end_pattern: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    if let Some(start_pos) = text_lower.find(start_pattern) {
        let after_start = &text_lower[start_pos + start_pattern.len()..];
        if let Some(end_pos) = after_start.find(end_pattern) {
            Some(after_start[..end_pos].trim().to_string())
        } else {
            None
        }
    } else {
        None
    }
}
