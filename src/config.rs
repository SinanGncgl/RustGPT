//! Configuration management for the LLM framework.
//!
//! Supports loading from TOML/YAML files and environment variables with builder pattern.

use crate::error::{LlmError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure for the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Model configuration
    pub model: ModelConfig,
    /// Training configuration
    pub training: TrainingConfig,
    /// Data configuration
    pub data: DataConfig,
    /// Output configuration
    pub output: OutputConfig,
}

/// Model-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Embedding dimension (default: 128)
    pub embedding_dim: usize,
    /// Hidden dimension (default: 256)
    pub hidden_dim: usize,
    /// Maximum sequence length (default: 80)
    pub max_seq_len: usize,
    /// Number of transformer blocks (default: 3)
    pub num_blocks: usize,
    /// Vocabulary size (0 = dynamic from data)
    pub vocab_size: usize,
}

/// Training configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Pre-training epochs
    pub pretraining_epochs: usize,
    /// Instruction tuning epochs
    pub finetuning_epochs: usize,
    /// Pre-training learning rate
    pub pretraining_lr: f32,
    /// Instruction tuning learning rate
    pub finetuning_lr: f32,
    /// Gradient clipping threshold
    pub gradient_clip: f32,
    /// Batch size
    pub batch_size: usize,
    /// Enable checkpoint saving
    pub checkpoint_enabled: bool,
    /// Checkpoint interval (epochs)
    pub checkpoint_interval: usize,
}

/// Data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// Path to pretraining data
    pub pretraining_data: String,
    /// Path to chat training data
    pub chat_training_data: String,
    /// Data format: "json" or "csv"
    pub format: String,
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output directory for checkpoints
    pub checkpoint_dir: String,
    /// Logging level: "debug", "info", "warn", "error"
    pub log_level: String,
    /// Enable progress bars
    pub show_progress: bool,
}

impl Default for Config {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Config {
            model: ModelConfig::default(),
            training: TrainingConfig::default(),
            data: DataConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            embedding_dim: 128,
            hidden_dim: 256,
            max_seq_len: 80,
            num_blocks: 3,
            vocab_size: 0,
        }
    }
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            pretraining_epochs: 300,
            finetuning_epochs: 300,
            pretraining_lr: 0.0005,
            finetuning_lr: 0.0001,
            gradient_clip: 5.0,
            batch_size: 32,
            checkpoint_enabled: true,
            checkpoint_interval: 10,
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            pretraining_data: "data/pretraining_data.json".to_string(),
            chat_training_data: "data/chat_training_data.json".to_string(),
            format: "json".to_string(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            checkpoint_dir: "./checkpoints".to_string(),
            log_level: "info".to_string(),
            show_progress: true,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_toml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| LlmError::ConfigError(format!("Failed to read config file: {}", e)))?;
        toml::from_str(&content)
            .map_err(|e| LlmError::ConfigError(format!("Failed to parse TOML config: {}", e)))
    }

    /// Load configuration from a YAML file.
    pub fn from_yaml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| LlmError::ConfigError(format!("Failed to read config file: {}", e)))?;
        serde_yaml::from_str(&content)
            .map_err(|e| LlmError::ConfigError(format!("Failed to parse YAML config: {}", e)))
    }

    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let mut config = Config::default();

        if let Ok(val) = std::env::var("LLM_EMBEDDING_DIM") {
            config.model.embedding_dim = val.parse().map_err(|_| {
                LlmError::ConfigError("Invalid LLM_EMBEDDING_DIM value".to_string())
            })?;
        }

        if let Ok(val) = std::env::var("LLM_HIDDEN_DIM") {
            config.model.hidden_dim = val
                .parse()
                .map_err(|_| LlmError::ConfigError("Invalid LLM_HIDDEN_DIM value".to_string()))?;
        }

        if let Ok(val) = std::env::var("LLM_MAX_SEQ_LEN") {
            config.model.max_seq_len = val
                .parse()
                .map_err(|_| LlmError::ConfigError("Invalid LLM_MAX_SEQ_LEN value".to_string()))?;
        }

        if let Ok(val) = std::env::var("LLM_PRETRAINING_LR") {
            config.training.pretraining_lr = val.parse().map_err(|_| {
                LlmError::ConfigError("Invalid LLM_PRETRAINING_LR value".to_string())
            })?;
        }

        Ok(config)
    }

    /// Save configuration to a TOML file.
    pub fn save_toml(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| LlmError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)
            .map_err(|e| LlmError::ConfigError(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.model.embedding_dim == 0 {
            return Err(LlmError::ConfigError(
                "embedding_dim must be > 0".to_string(),
            ));
        }
        if self.model.hidden_dim == 0 {
            return Err(LlmError::ConfigError("hidden_dim must be > 0".to_string()));
        }
        if self.model.max_seq_len == 0 {
            return Err(LlmError::ConfigError("max_seq_len must be > 0".to_string()));
        }
        if self.training.pretraining_lr <= 0.0 {
            return Err(LlmError::ConfigError(
                "pretraining_lr must be > 0".to_string(),
            ));
        }
        if self.training.finetuning_lr <= 0.0 {
            return Err(LlmError::ConfigError(
                "finetuning_lr must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.model.embedding_dim, 128);
        assert_eq!(config.model.hidden_dim, 256);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        config.model.embedding_dim = 0;
        assert!(config.validate().is_err());
    }
}
