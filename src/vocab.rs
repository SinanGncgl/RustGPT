//! Vocabulary management and tokenization utilities.
//!
//! Provides bidirectional mapping between tokens and their string representations,
//! with support for custom vocabulary building from training data.

use crate::error::{LlmError, Result};
use bincode::Encode;
use std::collections::{HashMap, HashSet};

/// Vocabulary for token encoding/decoding.
#[derive(Clone, Encode, Debug)]
pub struct Vocab {
    /// Mapping from words to token IDs
    pub encode: HashMap<String, usize>,
    /// Mapping from token IDs to words
    pub decode: HashMap<usize, String>,
    /// Ordered list of words
    pub words: Vec<String>,
}

impl Default for Vocab {
    fn default() -> Self {
        Self::new(Self::default_words())
    }
}

impl Vocab {
    /// Create a new vocabulary from a list of words.
    ///
    /// # Arguments
    /// * `words` - List of vocabulary words
    pub fn new(words: Vec<&str>) -> Self {
        if words.is_empty() {
            tracing::warn!("Creating vocabulary with no words");
        }

        let mut encode = HashMap::new();
        let mut decode = HashMap::new();

        for (i, &word) in words.iter().enumerate() {
            encode.insert(word.to_string(), i);
            decode.insert(i, word.to_string());
        }

        tracing::debug!("Vocabulary created with {} words", words.len());

        Vocab {
            encode,
            decode,
            words: words.iter().map(|w| w.to_string()).collect(),
        }
    }

    /// Encode a word to its token ID.
    ///
    /// # Arguments
    /// * `word` - The word to encode
    ///
    /// # Returns
    /// Token ID if the word exists in vocabulary, None otherwise
    pub fn encode(&self, word: &str) -> Option<usize> {
        self.encode.get(word).copied()
    }

    /// Encode a word to its token ID, with error handling.
    pub fn encode_or_error(&self, word: &str) -> Result<usize> {
        self.encode(word)
            .ok_or_else(|| LlmError::token(format!("Unknown token: {}", word)))
    }

    /// Decode a token ID to its word.
    ///
    /// # Arguments
    /// * `token_id` - The token ID to decode
    ///
    /// # Returns
    /// Reference to the word if the token ID exists, None otherwise
    pub fn decode(&self, token_id: usize) -> Option<&String> {
        self.decode.get(&token_id)
    }

    /// Decode a token ID with error handling.
    pub fn decode_or_error(&self, token_id: usize) -> Result<String> {
        self.decode(token_id)
            .cloned()
            .ok_or_else(|| LlmError::token(format!("Unknown token ID: {}", token_id)))
    }

    /// Get vocabulary size.
    pub fn size(&self) -> usize {
        self.words.len()
    }

    /// Check if a word is in the vocabulary.
    pub fn contains(&self, word: &str) -> bool {
        self.encode.contains_key(word)
    }

    /// Get default vocabulary for testing.
    pub fn default_words() -> Vec<&'static str> {
        vec!["hello", "world", "this", "is", "rust", "</s>"]
    }

    /// Process text data to extract vocabulary words.
    ///
    /// # Arguments
    /// * `texts` - Text samples to process
    /// * `vocab_set` - HashSet to accumulate vocabulary words
    pub fn process_text_for_vocab(texts: &[String], vocab_set: &mut HashSet<String>) {
        // Add end of sequence token
        vocab_set.insert("</s>".to_string());

        // Process all training examples for vocabulary
        for text in texts {
            for word in text.split_whitespace() {
                // Handle punctuation by splitting it from words
                let mut current = String::new();
                for c in word.chars() {
                    if c.is_ascii_punctuation() {
                        if !current.is_empty() {
                            vocab_set.insert(current.clone());
                            current.clear();
                        }
                        vocab_set.insert(c.to_string());
                    } else {
                        current.push(c);
                    }
                }
                if !current.is_empty() {
                    vocab_set.insert(current);
                }
            }
        }
    }

    /// Build vocabulary from multiple text samples.
    ///
    /// # Arguments
    /// * `texts` - Text samples to build vocabulary from
    ///
    /// # Returns
    /// A new Vocab instance built from the texts
    pub fn from_texts(texts: &[String]) -> Self {
        let mut vocab_set = HashSet::new();
        Self::process_text_for_vocab(texts, &mut vocab_set);
        let mut words: Vec<String> = vocab_set.into_iter().collect();
        words.sort();
        let words_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        Self::new(words_refs)
    }

    /// Get vocabulary statistics.
    pub fn statistics(&self) -> VocabStats {
        VocabStats {
            total_words: self.words.len(),
            has_eos_token: self.encode.contains_key("</s>"),
            has_unk_token: self.encode.contains_key("<unk>"),
        }
    }
}

/// Vocabulary statistics.
#[derive(Debug, Clone)]
pub struct VocabStats {
    /// Total number of words
    pub total_words: usize,
    /// Whether end-of-sequence token exists
    pub has_eos_token: bool,
    /// Whether unknown token exists
    pub has_unk_token: bool,
}

impl From<Vocab> for String {
    fn from(val: Vocab) -> Self {
        String::from_iter(
            val.words
                .iter()
                .enumerate()
                .map(|(i, str)| format!("({i},{str}),")),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab_encode_decode() {
        let vocab = Vocab::default();
        let token_id = vocab.encode("hello").unwrap();
        assert_eq!(vocab.decode(token_id), Some(&"hello".to_string()));
    }

    #[test]
    fn test_vocab_size() {
        let vocab = Vocab::default();
        assert_eq!(vocab.size(), 6);
    }

    #[test]
    fn test_vocab_contains() {
        let vocab = Vocab::default();
        assert!(vocab.contains("hello"));
        assert!(!vocab.contains("nonexistent"));
    }

    #[test]
    fn test_vocab_from_texts() {
        let texts = vec!["hello world".to_string(), "this is rust".to_string()];
        let vocab = Vocab::from_texts(&texts);
        assert!(vocab.contains("hello"));
        assert!(vocab.contains("world"));
    }
}
