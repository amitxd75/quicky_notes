//! # Context-Aware Autocomplete and Ghost Writing Engine
//!
//! A hybrid predictive text system uniting an offline unigram dictionary Radix Trie
//! with an adaptive Markov N-gram language model trained directly on user notes.
//!
//! # Submodules
//! - [`tokenizer`]: Sentence boundaries and contraction-aware word extraction.
//! - [`trie`]: Compressed Prefix Radix Trie for 333k frequency-weighted vocabulary words.
//! - [`ngram`]: Statistical Markov Language Model tracking bigram and trigram transitions.

pub mod ngram;
pub mod tokenizer;
pub mod trie;

pub use ngram::MarkovLanguageModel;
pub use tokenizer::{extract_preceding_words, extract_words_from_sentence, split_into_sentences};
pub use trie::RadixTrie;

/// Primary suggestion coordinator combining the unigram Radix Trie and Markov language model.
#[derive(Default, Debug, Clone)]
pub struct SuggestionEngine {
    /// Compressed prefix Radix Trie for base unigram vocabulary (~333k words).
    trie: RadixTrie,
    /// Statistical Markov N-Gram Language Model dynamically trained on sentences.
    markov: MarkovLanguageModel,
}

impl SuggestionEngine {
    /// Constructs a new `SuggestionEngine` initialized with the embedded dictionary.
    pub fn new() -> Self {
        Self {
            trie: RadixTrie::new_with_embedded(),
            markov: MarkovLanguageModel::new(),
        }
    }

    /// Spawns a background worker thread to parse the dictionary and train on existing notes.
    ///
    /// # Invariants
    /// - Non-blocking: returns an `mpsc::Receiver` immediately without freezing the UI thread.
    /// - Guarantees all initial note texts are parsed into sentences and trained before sending.
    pub fn start_async_load(initial_texts: Vec<String>) -> std::sync::mpsc::Receiver<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("quicky-suggest-loader".to_string())
            .spawn(move || {
                let mut engine = Self::new();
                for text in initial_texts {
                    engine.learn_text(&text);
                }
                let _ = tx.send(engine);
            })
            .expect("Failed to spawn suggestion loader thread");
        rx
    }

    /// Returns the total count of unique words loaded in the Radix Trie.
    #[allow(dead_code)]
    #[inline]
    pub fn word_count(&self) -> usize {
        self.trie.word_count
    }

    /// Learns or reinforces a single unigram word in the Trie.
    #[inline]
    pub fn learn_word(&mut self, word: &str) {
        self.trie.learn_word(word);
    }

    /// Learns a Bigram transition `(w1 -> w2)` in the Markov language model.
    #[inline]
    pub fn learn_bigram(&mut self, w1: &str, w2: &str) {
        self.markov.learn_bigram(w1, w2, 5);
    }

    /// Learns a Trigram transition `((w1, w2) -> w3)` in the Markov language model.
    #[inline]
    pub fn learn_trigram(&mut self, w1: &str, w2: &str, w3: &str) {
        self.markov.learn_trigram(w1, w2, w3, 8);
    }

    /// Returns total active (bigram, trigram) transition count.
    #[inline]
    pub fn transition_counts(&self) -> (usize, usize) {
        (self.markov.bigrams_count(), self.markov.trigrams_count())
    }

    /// Resets the statistical Markov model and re-trains from a set of note contents.
    pub fn retrain_from_notes<'a, I>(&mut self, note_contents: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        self.markov = MarkovLanguageModel::new();
        for content in note_contents {
            self.learn_text(content);
        }
    }

    /// Analyzes a text document, breaking it into sentences and training both Trie and Markov models.
    pub fn learn_text(&mut self, text: &str) {
        let sentences = split_into_sentences(text);
        for sentence in sentences {
            let words = extract_words_from_sentence(sentence);
            for w in &words {
                self.trie.learn_word(w);
            }
            self.markov.train_sentence_tokens(&words);
        }
    }

    /// Performs context-aware prediction: suggests the best completion suffix for `prefix` given `context_before`.
    ///
    /// # Resolution Pipeline
    /// 1. **Prefix Length Filter**: Requires at least 2 characters to eliminate distracting single-letter ghost popups.
    /// 2. **Complete Word Guard**: If `clean_prefix` is already an exact complete word (Trie returns `None`),
    ///    no compound suffixes are suggested (e.g. `"is"` will never suggest `"nt"`, `"so"` will never suggest `"ft"`).
    /// 3. **Markov Model (Trigram / Bigram)**: If preceding context exists, checks transition probabilities.
    /// 4. **Radix Trie (Unigrams)**: Fallback to highest-scoring unigram candidate.
    pub fn suggest_with_context(&self, context_before: &str, prefix: &str) -> Option<String> {
        let clean_prefix = prefix.trim();
        if clean_prefix.len() < 2 {
            return None;
        }

        // Query Radix Trie unigrams first (anchored to complete word scores)
        let trie_suffix = self.trie.suggest_suffix(clean_prefix)?;

        let (w_minus_2, w_minus_1) = extract_preceding_words(context_before);

        // 1. Try Markov N-Gram prediction first if preceding context exists
        if let Some(suffix) =
            self.markov
                .predict_suffix(w_minus_2.as_deref(), w_minus_1.as_deref(), clean_prefix)
        {
            return Some(suffix);
        }

        // 2. Fallback to Radix Trie unigram candidate
        Some(trie_suffix)
    }

    /// Convenience standalone unigram suggestion without preceding context.
    #[inline]
    pub fn suggest_suffix(&self, prefix: &str) -> Option<String> {
        self.suggest_with_context("", prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_dictionary_loading() {
        let engine = SuggestionEngine::new();
        assert!(engine.word_count() > 100_000);
        let res = engine.suggest_suffix("someth");
        assert_eq!(res, Some("ing".to_string()));
    }

    #[test]
    fn test_context_aware_sentence_training() {
        let mut engine = SuggestionEngine::new();
        engine.learn_text(
            "My phone went offline during the system update. Please restart your device.",
        );

        // Sentence 1 bigram: "went off" -> "line"
        let res = engine.suggest_with_context("my phone went ", "off");
        assert_eq!(res, Some("line".to_string()));

        // Sentence 2 bigram: "restart you" -> "r"
        let res2 = engine.suggest_with_context("Please restart ", "you");
        assert_eq!(res2, Some("r".to_string()));

        // Complete words must NEVER return ghost suggestions
        assert_eq!(engine.suggest_with_context("why para ", "is"), None);
        assert_eq!(engine.suggest_with_context("why para is ", "so"), None);
    }

    #[test]
    fn test_space_isolated_typing_flow() {
        let mut engine = SuggestionEngine::new();
        engine.learn_text("I write fast clean code.");

        // When user types "fast" (complete word), no suggestion
        assert_eq!(engine.suggest_with_context("I write ", "fast"), None);

        // When user types space after "fast" and types single letter "c", no suggestion (len < 2)
        assert_eq!(engine.suggest_with_context("I write fast ", "c"), None);

        // When user types "cl", it suggests "ean" for "clean" (NOT "fastclean")
        let res = engine.suggest_with_context("I write fast ", "cl");
        assert_eq!(res, Some("ean".to_string()));
    }
}
