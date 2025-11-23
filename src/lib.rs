//! RustGPT: A production-ready transformer-based LLM implementation in pure Rust.
//!
//! This library provides a complete implementation of a transformer-based language model
//! with training, inference, and checkpoint management capabilities.
//!
//! # Features
//! - Self-attention mechanisms with causal masking
//! - Multi-layer transformer architecture
//! - Adam optimizer with gradient clipping
//! - Model checkpointing and persistence
//! - Comprehensive error handling and logging
//! - Configuration management
//! - Training metrics and monitoring
//!
//! # Example
//! ```ignore
//! use llm::config::Config;
//! use llm::logging;
//!
//! // Initialize logging
//! logging::init_logging("info").unwrap();
//!
//! // Load configuration
//! let config = Config::default();
//! config.validate().unwrap();
//! ```

pub mod adam;
pub mod checkpoint;
pub mod config;
pub mod dataset_loader;
pub mod embeddings;
pub mod error;
pub mod feed_forward;
pub mod layer_norm;
pub mod llm;
pub mod logging;
pub mod metrics;
pub mod output_projection;
pub mod self_attention;
pub mod transformer;
pub mod training_ui;
pub mod vocab;
pub mod visualization;

// Re-export key types and functions for easier access
pub use config::Config;
pub use dataset_loader::{Dataset, DatasetType};
pub use embeddings::Embeddings;
pub use error::{LlmError, Result};
pub use llm::{LLM, Layer};
pub use logging::{init_json_logging, init_logging};
pub use metrics::Metrics;
pub use vocab::Vocab;

// Re-export checkpoint management
pub use checkpoint::{Checkpoint, CheckpointManager};

// Re-export visualization
pub use visualization::{TrainingVisualizer, VisualizationConfig};

/// Model configuration constants
pub const MAX_SEQ_LEN: usize = 80;
pub const EMBEDDING_DIM: usize = 128;
pub const HIDDEN_DIM: usize = 256;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
