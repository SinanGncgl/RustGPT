//! Comprehensive error handling for the LLM framework.
//!
//! Provides typed error variants for different failure modes across the system,
//! enabling better error recovery and diagnostics.

use std::fmt;
use thiserror::Error;

/// Result type alias for LLM operations.
pub type Result<T> = std::result::Result<T, LlmError>;

/// Comprehensive error types for LLM operations.
#[derive(Error, Debug)]
pub enum LlmError {
    /// Vocabulary-related errors
    #[error("Vocabulary error: {0}")]
    VocabularyError(String),

    /// File I/O errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Data loading errors
    #[error("Data loading error: {0}")]
    DataLoadError(String),

    /// Model architecture errors
    #[error("Model architecture error: {0}")]
    ArchitectureError(String),

    /// Training errors
    #[error("Training error: {0}")]
    TrainingError(String),

    /// Shape/dimension mismatch errors
    #[error("Shape mismatch: expected {expected}, got {actual}")]
    ShapeMismatch { expected: String, actual: String },

    /// Token encoding/decoding errors
    #[error("Token error: {0}")]
    TokenError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Generic errors
    #[error("{0}")]
    Other(String),
}

impl LlmError {
    /// Create a vocabulary error.
    pub fn vocabulary(msg: impl Into<String>) -> Self {
        LlmError::VocabularyError(msg.into())
    }

    /// Create a serialization error.
    pub fn serialization(msg: impl Into<String>) -> Self {
        LlmError::SerializationError(msg.into())
    }

    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        LlmError::ConfigError(msg.into())
    }

    /// Create a data loading error.
    pub fn data_load(msg: impl Into<String>) -> Self {
        LlmError::DataLoadError(msg.into())
    }

    /// Create an architecture error.
    pub fn architecture(msg: impl Into<String>) -> Self {
        LlmError::ArchitectureError(msg.into())
    }

    /// Create a training error.
    pub fn training(msg: impl Into<String>) -> Self {
        LlmError::TrainingError(msg.into())
    }

    /// Create a shape mismatch error.
    pub fn shape_mismatch(expected: impl fmt::Display, actual: impl fmt::Display) -> Self {
        LlmError::ShapeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create a token error.
    pub fn token(msg: impl Into<String>) -> Self {
        LlmError::TokenError(msg.into())
    }

    /// Create a validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        LlmError::ValidationError(msg.into())
    }
}

/// Extension trait for additional error context operations.
pub trait Context<T> {
    /// Add context to an error.
    fn context(self, msg: impl Into<String>) -> Result<T>;
}

impl<T, E: Into<LlmError>> Context<T> for std::result::Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            LlmError::Other(format!("{}: {}", msg.into(), error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = LlmError::vocabulary("test vocab");
        assert_eq!(err.to_string(), "Vocabulary error: test vocab");
    }

    #[test]
    fn test_shape_mismatch() {
        let err = LlmError::shape_mismatch("(10, 20)", "(5, 10)");
        assert!(err.to_string().contains("Shape mismatch"));
    }
}
