//! Dataset loading and management utilities.
//!
//! Supports loading training data from JSON and CSV formats with comprehensive
//! error handling and data validation.

use crate::error::{LlmError, Result};
use csv::ReaderBuilder;
use std::fs;
use std::path::Path;

/// Dataset container for pre-training and instruction-tuning data.
#[derive(Debug, Clone)]
pub struct Dataset {
    /// Pre-training examples (factual statements)
    pub pretraining_data: Vec<String>,
    /// Instruction tuning examples (conversational)
    pub chat_training_data: Vec<String>,
}

/// Supported data formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum DatasetType {
    /// JSON format
    JSON,
    /// CSV format
    CSV,
}

impl Dataset {
    /// Create a new dataset by loading from files.
    ///
    /// # Arguments
    /// * `pretraining_data_path` - Path to pre-training data file
    /// * `chat_training_data_path` - Path to chat training data file
    /// * `type_of_data` - Format of the data files
    ///
    /// # Errors
    /// Returns an error if files cannot be read or parsed.
    pub fn new(
        pretraining_data_path: impl AsRef<Path>,
        chat_training_data_path: impl AsRef<Path>,
        type_of_data: DatasetType,
    ) -> Result<Self> {
        let pretraining_data: Vec<String>;
        let chat_training_data: Vec<String>;

        match type_of_data {
            DatasetType::CSV => {
                pretraining_data = get_data_from_csv(pretraining_data_path)?;
                chat_training_data = get_data_from_csv(chat_training_data_path)?;
            }
            DatasetType::JSON => {
                pretraining_data = get_data_from_json(pretraining_data_path)?;
                chat_training_data = get_data_from_json(chat_training_data_path)?;
            }
        }

        // Validate data is not empty
        if pretraining_data.is_empty() && chat_training_data.is_empty() {
            return Err(LlmError::DataLoadError(
                "Both datasets are empty".to_string(),
            ));
        }

        tracing::info!(
            "Dataset loaded: {} pre-training samples, {} chat samples",
            pretraining_data.len(),
            chat_training_data.len()
        );

        Ok(Dataset {
            pretraining_data,
            chat_training_data,
        })
    }

    /// Get the total number of training samples.
    pub fn total_samples(&self) -> usize {
        self.pretraining_data.len() + self.chat_training_data.len()
    }

    /// Validate dataset integrity.
    pub fn validate(&self) -> Result<()> {
        if self.pretraining_data.is_empty() && self.chat_training_data.is_empty() {
            return Err(LlmError::DataLoadError(
                "Dataset contains no samples".to_string(),
            ));
        }

        // Check for empty strings
        let empty_count = self
            .pretraining_data
            .iter()
            .chain(self.chat_training_data.iter())
            .filter(|s| s.trim().is_empty())
            .count();

        if empty_count > 0 {
            tracing::warn!("Dataset contains {} empty strings", empty_count);
        }

        Ok(())
    }
}

/// Load data from a JSON file.
fn get_data_from_json(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let path = path.as_ref();
    let data_json = fs::read_to_string(path)
        .map_err(|e| LlmError::DataLoadError(format!("Failed to read JSON file: {}", e)))?;

    let data: Vec<String> = serde_json::from_str(&data_json)
        .map_err(|e| LlmError::DataLoadError(format!("Failed to parse JSON: {}", e)))?;

    tracing::debug!("Loaded {} samples from JSON file", data.len());
    Ok(data)
}

/// Load data from a CSV file.
fn get_data_from_csv(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let path = path.as_ref();
    let file = fs::File::open(path)
        .map_err(|e| LlmError::DataLoadError(format!("Failed to open CSV file: {}", e)))?;

    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);
    let mut data = Vec::new();

    for result in rdr.records() {
        let record = result
            .map_err(|e| LlmError::DataLoadError(format!("Failed to read CSV record: {}", e)))?;
        data.push(record.iter().collect::<Vec<_>>().join(","));
    }

    tracing::debug!("Loaded {} samples from CSV file", data.len());
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_total_samples() {
        let dataset = Dataset {
            pretraining_data: vec!["test1".to_string()],
            chat_training_data: vec!["test2".to_string(), "test3".to_string()],
        };
        assert_eq!(dataset.total_samples(), 3);
    }

    #[test]
    fn test_dataset_validation() {
        let dataset = Dataset {
            pretraining_data: vec!["test1".to_string()],
            chat_training_data: vec!["test2".to_string()],
        };
        assert!(dataset.validate().is_ok());

        let empty_dataset = Dataset {
            pretraining_data: vec![],
            chat_training_data: vec![],
        };
        assert!(empty_dataset.validate().is_err());
    }
}
