//! Compact Prefix Radix Trie for unigram vocabulary storage and high-performance autocomplete.
//!
//! # Architecture & Complexity
//! - **Lookup Time Complexity**: O(L) where L is the character length of the query prefix.
//! - **Memory Layout**: Edge-compressed Radix Trie nodes where common prefixes share contiguous node paths.
//! - **Scoring Model**: Two-tier ranking combining static Google Web 1-Gram frequencies with dynamic user reinforcement:
//!   S(node) = user_weight * 100,000 + base_weight

use super::tokenizer::format_suffix_casing;

/// Magic header bytes identifying the binary packed dictionary asset (`"QNW1"`).
const MAGIC_BYTES: &[u8; 4] = b"QNW1";

/// Maximum recursion depth allowed during candidate suffix traversal to guarantee strict execution bounds.
pub const MAX_SEARCH_DEPTH: usize = 12;

/// Multiplier applied to user typing frequency to prioritize local user vocabulary.
pub const USER_WEIGHT_MULTIPLIER: u64 = 100_000;

/// Embedded binary dictionary containing ~333,000 frequency-ranked English words.
static EMBEDDED_WORDS_BIN: &[u8] = include_bytes!("../../../assets/words.bin");

/// A node within the compressed prefix Radix Trie.
#[derive(Default, Debug, Clone)]
pub struct RadixNode {
    /// Compressed edge substring leading into this node.
    pub edge: String,
    /// Ordered child sub-branches continuing from this node.
    pub children: Vec<RadixNode>,
    /// Whether this node marks the terminal character of a valid vocabulary word.
    pub is_word: bool,
    /// Base frequency ranking from global dictionary (range `0..=65535`).
    pub base_weight: u16,
    /// Dynamic user frequency ranking accumulated from local notes and typing.
    pub user_weight: u32,
}

impl RadixNode {
    /// Constructs a new `RadixNode` with explicit edge label and initial weights.
    ///
    /// # Preconditions
    /// - `edge` should contain valid UTF-8 characters.
    #[inline]
    pub fn new(edge: String, is_word: bool, base_weight: u16, user_weight: u32) -> Self {
        Self {
            edge,
            children: Vec::new(),
            is_word,
            base_weight,
            user_weight,
        }
    }

    /// Computes the composite priority score for this node.
    ///
    /// User-reinforced words receive a multiplier over standard dictionary base weights.
    #[inline]
    pub fn score(&self) -> u64 {
        (self.user_weight as u64) * USER_WEIGHT_MULTIPLIER + (self.base_weight as u64)
    }

    /// Inserts a word and its associated weights into this branch of the Radix Trie.
    ///
    /// Performs in-place edge splitting if a partial prefix match is encountered.
    pub fn insert(
        &mut self,
        word: &str,
        base_weight: u16,
        user_weight: u32,
        count_inc: &mut usize,
    ) {
        if word.is_empty() {
            if !self.is_word {
                self.is_word = true;
                *count_inc += 1;
            }
            self.base_weight = self.base_weight.max(base_weight);
            self.user_weight = self.user_weight.saturating_add(user_weight);
            return;
        }

        let Some(first_char) = word.chars().next() else {
            return;
        };

        let child_idx = self
            .children
            .iter()
            .position(|c| c.edge.starts_with(first_char));

        match child_idx {
            Some(idx) => {
                let child = &mut self.children[idx];
                let common_len = common_prefix_len(&child.edge, word);

                if common_len == child.edge.len() {
                    child.insert(&word[common_len..], base_weight, user_weight, count_inc);
                } else {
                    let split_edge = child.edge[common_len..].to_string();
                    let mut split_node = RadixNode::new(
                        split_edge,
                        child.is_word,
                        child.base_weight,
                        child.user_weight,
                    );
                    split_node.children = std::mem::take(&mut child.children);

                    child.edge.truncate(common_len);
                    child.is_word = false;
                    child.base_weight = 0;
                    child.user_weight = 0;
                    child.children.push(split_node);

                    if common_len == word.len() {
                        child.is_word = true;
                        child.base_weight = base_weight;
                        child.user_weight = user_weight;
                        *count_inc += 1;
                    } else {
                        let new_node = RadixNode::new(
                            word[common_len..].to_string(),
                            true,
                            base_weight,
                            user_weight,
                        );
                        child.children.push(new_node);
                        *count_inc += 1;
                    }
                }
            }
            None => {
                let new_node = RadixNode::new(word.to_string(), true, base_weight, user_weight);
                self.children.push(new_node);
                *count_inc += 1;
            }
        }
    }
}

/// Computes the matching byte prefix length between two UTF-8 string slices.
#[inline]
fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .map(|(c, _)| c.len_utf8())
        .sum()
}

/// Compact Radix Trie data structure managing dictionary words and learned unigram vocabulary.
#[derive(Default, Debug, Clone)]
pub struct RadixTrie {
    /// Root node of the Trie.
    pub root: RadixNode,
    /// Total count of unique terminal words stored in the Trie.
    pub word_count: usize,
}

impl RadixTrie {
    /// Constructs a new `RadixTrie` and loads the embedded binary dictionary asset.
    pub fn new_with_embedded() -> Self {
        let mut trie = Self::default();
        trie.load_binary(EMBEDDED_WORDS_BIN);
        trie
    }

    /// Parses binary-packed dictionary bytes into the Radix Trie.
    ///
    /// # Binary Layout Format
    /// - `[0..4]`: Magic bytes (`"QNW1"`).
    /// - `[4..]`: Sequence of `[u8 length][UTF-8 word][u16 little-endian weight]`.
    pub fn load_binary(&mut self, data: &[u8]) {
        if data.len() < 4 || &data[0..4] != MAGIC_BYTES {
            return;
        }

        let mut idx = 4;
        let mut count = 0;
        while idx < data.len() {
            let len = data[idx] as usize;
            idx += 1;

            if idx + len + 2 > data.len() {
                break;
            }

            if let Ok(word) = std::str::from_utf8(&data[idx..idx + len]) {
                let weight_bytes = [data[idx + len], data[idx + len + 1]];
                let base_weight = u16::from_le_bytes(weight_bytes);
                self.root.insert(word, base_weight, 0, &mut count);
            }

            idx += len + 2;
        }
        self.word_count += count;
    }

    /// Learns or reinforces a single unigram word in the Trie.
    ///
    /// Filters out tokens shorter than 2 characters or containing invalid punctuation.
    pub fn learn_word(&mut self, word: &str) {
        let clean = word.trim().to_lowercase();
        if clean.len() >= 2
            && clean
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '\'' || c == '’')
        {
            let mut count = 0;
            self.root.insert(&clean, 0, 1, &mut count);
            self.word_count += count;
        }
    }

    /// Searches the Radix Trie for the highest-scoring completion suffix matching `clean_prefix`.
    ///
    /// # Returns
    /// - `Some(suffix)`: The remaining letters needed to complete the highest-ranked candidate word.
    /// - `None`: If no candidate exists or the prefix already has higher score than any child completion.
    pub fn suggest_suffix(&self, clean_prefix: &str) -> Option<String> {
        let lower_prefix = clean_prefix.to_lowercase();
        let mut curr = &self.root;
        let mut rem_prefix = lower_prefix.as_str();

        while !rem_prefix.is_empty() {
            let first_ch = rem_prefix.chars().next()?;
            let child = curr
                .children
                .iter()
                .find(|c| c.edge.starts_with(first_ch))?;

            let common_len = common_prefix_len(&child.edge, rem_prefix);
            if common_len == rem_prefix.len() {
                let mut best_suffix = None;
                let mut path_buf = child.edge[common_len..].to_string();

                // If this node completes a word:
                // - If path_buf is non-empty (e.g. prefix "he", edge "help" -> path_buf "lp"),
                //   then "help" is the candidate with initial score.
                // - If path_buf is empty (e.g. prefix "is", edge "is" -> path_buf ""),
                //   then "is" is the exact word. We anchor best_score to "is"'s score so children
                //   (like "isn't" or "island") will only be suggested if they rank HIGHER than "is"!
                let mut best_score = if child.is_word {
                    if !path_buf.is_empty() {
                        best_suffix = Some(path_buf.clone());
                    }
                    child.score()
                } else {
                    0
                };

                self.find_best_candidate(
                    child,
                    &mut path_buf,
                    0,
                    &mut best_suffix,
                    &mut best_score,
                );

                let suffix = best_suffix?;
                if suffix.is_empty() {
                    return None;
                }
                return Some(format_suffix_casing(clean_prefix, &suffix));
            } else if common_len == child.edge.len() {
                rem_prefix = &rem_prefix[common_len..];
                curr = child;
            } else {
                return None;
            }
        }

        None
    }

    /// Recursively explores child branches to find the highest-scoring candidate completion.
    fn find_best_candidate(
        &self,
        node: &RadixNode,
        current_path: &mut String,
        depth: usize,
        best_suffix: &mut Option<String>,
        best_score: &mut u64,
    ) {
        if depth > MAX_SEARCH_DEPTH {
            return;
        }

        for child in &node.children {
            let prev_len = current_path.len();
            current_path.push_str(&child.edge);

            if child.is_word {
                let score = child.score();
                if score > *best_score {
                    *best_score = score;
                    *best_suffix = Some(current_path.clone());
                }
            }

            self.find_best_candidate(child, current_path, depth + 1, best_suffix, best_score);
            current_path.truncate(prev_len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radix_trie_embedded_lookup() {
        let trie = RadixTrie::new_with_embedded();
        assert!(trie.word_count > 100_000);
        let res = trie.suggest_suffix("somet");
        assert!(
            res == Some("hing".to_string())
                || res == Some("imes".to_string())
                || res == Some("ime".to_string()),
            "Expected 'hing' or 'imes' or 'ime', got {:?}",
            res
        );

        let res_th = trie.suggest_suffix("someth");
        assert_eq!(res_th, Some("ing".to_string()));
    }

    #[test]
    fn test_radix_trie_user_learning_priority() {
        let mut trie = RadixTrie::new_with_embedded();
        for _ in 0..10 {
            trie.learn_word("supercalifragilistic");
        }
        let res = trie.suggest_suffix("supercali");
        assert_eq!(res, Some("fragilistic".to_string()));
    }

    #[test]
    fn test_exact_complete_word_returns_no_compound_suffix() {
        let trie = RadixTrie::new_with_embedded();
        let res = trie.suggest_suffix("hel");
        assert!(res.is_some());

        // Exact complete words MUST return None (never suggesting compound suffixes)
        assert_eq!(trie.suggest_suffix("hello"), None);
        assert_eq!(trie.suggest_suffix("is"), None);
        assert_eq!(trie.suggest_suffix("the"), None);
    }
}
