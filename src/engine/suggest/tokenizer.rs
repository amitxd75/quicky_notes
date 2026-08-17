//! Sentence and word boundary tokenization engine for statistical language learning.

/// Breaks a raw text block into discrete sentences delimited by punctuation and newlines.
///
/// Ensures N-gram statistical transitions never cross unrelated sentence boundaries.
pub fn split_into_sentences(text: &str) -> Vec<&str> {
    text.split(['.', '!', '?', '\n', ';'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Extracts valid words and identifier tokens from a sentence string, supporting contractions like "I'd", "I've", "don't".
pub fn extract_words_from_sentence(sentence: &str) -> Vec<&str> {
    sentence
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '\'' && c != '’')
        .map(|w| w.trim_matches(['\'', '’', '-', '_', ' ']))
        .filter(|w| w.len() >= 2)
        .collect()
}

/// Extracts the last 1 or 2 words preceding the cursor in the context buffer.
///
/// Invariant: Respects sentence boundaries so it doesn't pull context from previous sentences.
pub fn extract_preceding_words(context_before: &str) -> (Option<String>, Option<String>) {
    // If the context ends with a sentence delimiter or newline, context is reset
    let trimmed = context_before.trim_end();
    if trimmed.is_empty() {
        return (None, None);
    }

    // Only inspect the current active sentence
    let last_sentence = trimmed
        .rsplit(['.', '!', '?', '\n', ';'])
        .next()
        .unwrap_or(trimmed);

    let tokens: Vec<&str> = last_sentence
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '\'' && c != '’')
        .map(|w| w.trim_matches(['\'', '’', '-', '_', ' ']))
        .filter(|w| !w.is_empty())
        .collect();

    let len = tokens.len();
    if len == 0 {
        (None, None)
    } else if len == 1 {
        (None, Some(tokens[0].to_lowercase()))
    } else {
        (
            Some(tokens[len - 2].to_lowercase()),
            Some(tokens[len - 1].to_lowercase()),
        )
    }
}

/// Preserves the casing of a suggested suffix based on the user's input prefix casing.
#[inline]
pub fn format_suffix_casing(prefix: &str, suffix: &str) -> String {
    if prefix
        .chars()
        .all(|c| c.is_uppercase() || c == '\'' || c == '’')
    {
        suffix.to_uppercase()
    } else {
        suffix.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_sentences() {
        let text = "Hello world! This is note #1.\nLet's test sentence splitting; works great?";
        let sentences = split_into_sentences(text);
        assert_eq!(sentences.len(), 4);
        assert_eq!(sentences[0], "Hello world");
        assert_eq!(sentences[1], "This is note #1");
        assert_eq!(sentences[2], "Let's test sentence splitting");
        assert_eq!(sentences[3], "works great");
    }

    #[test]
    fn test_extract_words_from_sentence() {
        let sentence = "let mut my_variable = 42-count;";
        let words = extract_words_from_sentence(sentence);
        assert_eq!(words, vec!["let", "mut", "my_variable", "42-count"]);
    }

    #[test]
    fn test_extract_preceding_words_resets_on_sentence_boundary() {
        assert_eq!(
            extract_preceding_words("my phone went "),
            (Some("phone".to_string()), Some("went".to_string()))
        );
        assert_eq!(
            extract_preceding_words("First sentence. Now went "),
            (Some("now".to_string()), Some("went".to_string()))
        );
        assert_eq!(
            extract_preceding_words("Single "),
            (None, Some("single".to_string()))
        );
    }

    #[test]
    fn test_contractions_tokenization_and_casing() {
        let sentence = "I've been thinking that I'd like to go, but don't know if we'll make it.";
        let words = extract_words_from_sentence(sentence);
        assert!(words.contains(&"I've"));
        assert!(words.contains(&"I'd"));
        assert!(words.contains(&"don't"));
        assert!(words.contains(&"we'll"));

        assert_eq!(format_suffix_casing("I'", "ve"), "VE");
        assert_eq!(format_suffix_casing("don'", "t"), "t");
    }
}
