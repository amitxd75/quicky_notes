//! Custom Statistical Markov N-Gram Language Model for predictive sentence & phrase completion.
//!
//! # Model Details
//! - **Bigram Model**: Computes conditional probabilities P(wi | wi-1).
//! - **Trigram Model**: Computes conditional probabilities P(wi | wi-2, wi-1).
//! - **Capacity Bounds**: Caps top transitions per word to prevent unbounded memory growth in long-running processes:
//!   - Max 16 Bigrams per token.
//!   - Max 8 Trigrams per token pair.

use super::tokenizer::format_suffix_casing;
use std::collections::HashMap;

/// Maximum number of bigram candidate completions tracked per word.
const MAX_BIGRAMS_PER_WORD: usize = 16;

/// Maximum number of trigram candidate completions tracked per word pair.
const MAX_TRIGRAMS_PER_PAIR: usize = 8;

/// Statistical Markov Language Model managing Bigram and Trigram transition distributions.
#[derive(Default, Debug, Clone)]
pub struct MarkovLanguageModel {
    /// Bigram transition table: `w_{-1} -> [(w_0, frequency_count)]`.
    bigrams: HashMap<String, Vec<(String, u32)>>,
    /// Trigram transition table: `(w_{-2}, w_{-1}) -> [(w_0, frequency_count)]`.
    trigrams: HashMap<(String, String), Vec<(String, u32)>>,
}

impl MarkovLanguageModel {
    /// Constructs a new, empty `MarkovLanguageModel`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Learns a Bigram transition `(w1 -> w2)` with a specified weight increment.
    ///
    /// # Invariants
    /// - Normalizes both tokens to lowercase.
    /// - Automatically truncates candidates beyond [`MAX_BIGRAMS_PER_WORD`].
    pub fn learn_bigram(&mut self, w1: &str, w2: &str, weight_inc: u32) {
        let clean1 = w1.trim().to_lowercase();
        let clean2 = w2.trim().to_lowercase();
        if clean1.is_empty() || clean2.is_empty() || clean1 == clean2 {
            return;
        }

        let entry = self.bigrams.entry(clean1).or_default();
        if let Some(pos) = entry.iter().position(|(w, _)| w == &clean2) {
            entry[pos].1 = entry[pos].1.saturating_add(weight_inc);
        } else {
            entry.push((clean2, weight_inc));
        }

        // Keep transitions sorted descending by probability
        entry.sort_by_key(|b| std::cmp::Reverse(b.1));
        if entry.len() > MAX_BIGRAMS_PER_WORD {
            entry.truncate(MAX_BIGRAMS_PER_WORD);
        }
    }

    /// Returns total number of active bigram transitions tracked.
    #[inline]
    pub fn bigrams_count(&self) -> usize {
        self.bigrams.values().map(|v| v.len()).sum()
    }

    /// Returns total number of active trigram transitions tracked.
    #[inline]
    pub fn trigrams_count(&self) -> usize {
        self.trigrams.values().map(|v| v.len()).sum()
    }

    /// Learns a Trigram transition `((w1, w2) -> w3)` with a specified weight increment.
    ///
    /// # Invariants
    /// - Normalizes all tokens to lowercase.
    /// - Automatically truncates candidates beyond [`MAX_TRIGRAMS_PER_PAIR`].
    pub fn learn_trigram(&mut self, w1: &str, w2: &str, w3: &str, weight_inc: u32) {
        let clean1 = w1.trim().to_lowercase();
        let clean2 = w2.trim().to_lowercase();
        let clean3 = w3.trim().to_lowercase();
        if clean1.is_empty() || clean2.is_empty() || clean3.is_empty() {
            return;
        }

        let entry = self.trigrams.entry((clean1, clean2)).or_default();
        if let Some(pos) = entry.iter().position(|(w, _)| w == &clean3) {
            entry[pos].1 = entry[pos].1.saturating_add(weight_inc);
        } else {
            entry.push((clean3, weight_inc));
        }

        entry.sort_by_key(|b| std::cmp::Reverse(b.1));
        if entry.len() > MAX_TRIGRAMS_PER_PAIR {
            entry.truncate(MAX_TRIGRAMS_PER_PAIR);
        }
    }

    /// Trains the language model on an extracted sequence of words from a single sentence.
    ///
    /// Applies higher initial weight to trigram transitions (6) than bigram transitions (4).
    pub fn train_sentence_tokens(&mut self, words: &[&str]) {
        for window in words.windows(2) {
            self.learn_bigram(window[0], window[1], 4);
        }
        for window in words.windows(3) {
            self.learn_trigram(window[0], window[1], window[2], 6);
        }
    }

    /// Predicts the highest probability suffix for `prefix` given preceding context words `(w_{-2}, w_{-1})`.
    ///
    /// # Search Hierarchy
    /// 1. Evaluates Trigram transition $(w_{-2}, w_{-1}) \rightarrow \text{prefix}\dots$
    /// 2. Falls back to Bigram transition $w_{-1} \rightarrow \text{prefix}\dots$
    pub fn predict_suffix(
        &self,
        w_minus_2: Option<&str>,
        w_minus_1: Option<&str>,
        prefix: &str,
    ) -> Option<String> {
        let clean_prefix = prefix.trim();
        if clean_prefix.is_empty() {
            return None;
        }

        let lower_prefix = clean_prefix.to_lowercase();

        // 1. Level 1: Trigram Transition Match
        if let (Some(w2), Some(w1)) = (w_minus_2, w_minus_1)
            && let Some(candidates) = self.trigrams.get(&(w2.to_string(), w1.to_string()))
        {
            for (next_word, _) in candidates {
                if next_word.starts_with(&lower_prefix) && next_word.len() > lower_prefix.len() {
                    let suffix = &next_word[lower_prefix.len()..];
                    return Some(format_suffix_casing(clean_prefix, suffix));
                }
            }
        }

        // 2. Level 2: Bigram Transition Match
        if let Some(w1) = w_minus_1
            && let Some(candidates) = self.bigrams.get(w1)
        {
            for (next_word, _) in candidates {
                if next_word.starts_with(&lower_prefix) && next_word.len() > lower_prefix.len() {
                    let suffix = &next_word[lower_prefix.len()..];
                    return Some(format_suffix_casing(clean_prefix, suffix));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markov_model_bigram_prediction() {
        let mut model = MarkovLanguageModel::new();
        model.train_sentence_tokens(&["my", "phone", "went", "offline", "yesterday"]);

        // Context "went" + prefix 'o' -> predicts "offline"
        let res = model.predict_suffix(Some("phone"), Some("went"), "o");
        assert_eq!(res, Some("ffline".to_string()));

        // Trigram prediction
        let res_tri = model.predict_suffix(Some("went"), Some("offline"), "y");
        assert_eq!(res_tri, Some("esterday".to_string()));
    }

    #[test]
    fn test_markov_model_frequency_reordering() {
        let mut model = MarkovLanguageModel::new();
        model.learn_bigram("cargo", "run", 2);
        model.learn_bigram("cargo", "build", 10);

        let res = model.predict_suffix(None, Some("cargo"), "");
        assert_eq!(res, None);

        let res_b = model.predict_suffix(None, Some("cargo"), "b");
        assert_eq!(res_b, Some("uild".to_string()));

        let res_r = model.predict_suffix(None, Some("cargo"), "r");
        assert_eq!(res_r, Some("un".to_string()));
    }
}
