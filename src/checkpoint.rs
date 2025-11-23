//! Model checkpoint management for persistence and recovery.
//!
//! Provides save/load functionality for trained model parameters and state.

use crate::error::{LlmError, Result};
use bincode::{Decode, Encode};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Checkpoint for saving model state.
#[derive(Serialize, Deserialize, Clone, Encode, Decode)]
pub struct Checkpoint {
    /// Model version/epoch
    pub epoch: usize,
    /// Training loss at checkpoint
    pub loss: f32,
    /// Model parameters (serialized)
    pub parameters: Vec<Vec<f32>>,
    /// Metadata about the checkpoint
    pub metadata: CheckpointMetadata,
}

/// Metadata for a checkpoint.
#[derive(Serialize, Deserialize, Clone, Debug, Encode, Decode)]
pub struct CheckpointMetadata {
    /// Timestamp of checkpoint creation
    pub created_at: String,
    /// Model configuration
    pub config: String,
    /// Training step
    pub step: usize,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new(epoch: usize, loss: f32, config: &str) -> Self {
        Self {
            epoch,
            loss,
            parameters: Vec::new(),
            metadata: CheckpointMetadata {
                created_at: chrono::Local::now().to_rfc3339(),
                config: config.to_string(),
                step: epoch,
            },
        }
    }

    /// Add a parameter matrix to the checkpoint.
    pub fn add_parameter(&mut self, matrix: &Array2<f32>) {
        self.parameters.push(matrix.iter().copied().collect());
    }

    /// Save checkpoint to file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let serialized =
            bincode::encode_to_vec(self, bincode::config::standard()).map_err(|e| {
                LlmError::serialization(format!("Failed to serialize checkpoint: {}", e))
            })?;
        std::fs::write(path, serialized).map_err(LlmError::IoError)?;
        tracing::info!("Checkpoint saved to {:?}", path);
        Ok(())
    }

    /// Load checkpoint from file.
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path).map_err(LlmError::IoError)?;
        let (checkpoint, _) =
            bincode::decode_from_slice::<Self, _>(&data, bincode::config::standard()).map_err(
                |e| LlmError::serialization(format!("Failed to deserialize checkpoint: {}", e)),
            )?;
        tracing::info!("Checkpoint loaded from {:?}", path);
        Ok(checkpoint)
    }
}

/// Checkpoint manager for handling multiple checkpoints.
pub struct CheckpointManager {
    checkpoint_dir: std::path::PathBuf,
    keep_best: bool,
    max_checkpoints: usize,
}

impl CheckpointManager {
    /// Create a new checkpoint manager.
    pub fn new(checkpoint_dir: &Path, keep_best: bool, max_checkpoints: usize) -> Result<Self> {
        std::fs::create_dir_all(checkpoint_dir).map_err(LlmError::IoError)?;
        Ok(Self {
            checkpoint_dir: checkpoint_dir.to_path_buf(),
            keep_best,
            max_checkpoints,
        })
    }

    /// Save a checkpoint with automatic cleanup.
    pub fn save(&self, checkpoint: &Checkpoint) -> Result<()> {
        let filename = format!("checkpoint_epoch_{:04}.bin", checkpoint.epoch);
        let path = self.checkpoint_dir.join(&filename);
        checkpoint.save(&path)?;

        if self.keep_best {
            self.cleanup_old_checkpoints()?;
        }
        Ok(())
    }

    /// Load the best checkpoint.
    pub fn load_best(&self) -> Result<Checkpoint> {
        let mut checkpoints = self.list_checkpoints()?;
        checkpoints.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if let Some((path, _)) = checkpoints.first() {
            Checkpoint::load(path)
        } else {
            Err(LlmError::training("No checkpoints found".to_string()))
        }
    }

    /// List all available checkpoints with their losses.
    fn list_checkpoints(&self) -> Result<Vec<(std::path::PathBuf, f32)>> {
        let mut checkpoints = Vec::new();

        for entry in std::fs::read_dir(&self.checkpoint_dir).map_err(LlmError::IoError)? {
            let entry = entry.map_err(LlmError::IoError)?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "bin") {
                if let Ok(checkpoint) = Checkpoint::load(&path) {
                    checkpoints.push((path, checkpoint.loss));
                }
            }
        }

        Ok(checkpoints)
    }

    /// Remove old checkpoints keeping only the best ones.
    fn cleanup_old_checkpoints(&self) -> Result<()> {
        let mut checkpoints = self.list_checkpoints()?;
        checkpoints.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        while checkpoints.len() > self.max_checkpoints {
            if let Some((path, _)) = checkpoints.pop() {
                std::fs::remove_file(&path).map_err(LlmError::IoError)?;
                tracing::debug!("Removed old checkpoint: {:?}", path);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = Checkpoint::new(0, 1.5, "test_config");
        assert_eq!(checkpoint.epoch, 0);
        assert_eq!(checkpoint.loss, 1.5);
    }
}
