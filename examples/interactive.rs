//! Example: Interactive LLM with best practices.
//!
//! This example demonstrates:
//! - Configuration management
//! - Error handling
//! - Logging
//! - Checkpoint management
//! - Metrics tracking

use llm::{
    Checkpoint, CheckpointManager, Config, Dataset, DatasetType, EMBEDDING_DIM, Embeddings,
    HIDDEN_DIM, LLM, Metrics, Result, Vocab, init_logging, output_projection::OutputProjection,
    transformer::TransformerBlock,
};
use std::io::Write;
use std::path::Path;
use tracing::info;

fn main() -> Result<()> {
    // Initialize logging
    init_logging("info")
        .map_err(|e| llm::LlmError::Other(format!("Logging init failed: {}", e)))?;
    info!("Starting RustGPT Example");

    // Load configuration
    let config = if Path::new("config.toml").exists() {
        Config::from_toml(Path::new("config.toml"))?
    } else {
        info!("Using default configuration");
        Config::default()
    };
    config.validate()?;

    // Setup checkpoint manager
    let _checkpoint_mgr = CheckpointManager::new(
        Path::new(&config.output.checkpoint_dir),
        true,
        5, // keep 5 best checkpoints
    )?;
    info!("Checkpoint manager ready");

    // Load dataset
    let dataset = Dataset::new(
        &config.data.pretraining_data,
        &config.data.chat_training_data,
        if config.data.format == "csv" {
            DatasetType::CSV
        } else {
            DatasetType::JSON
        },
    )?;
    dataset.validate()?;
    info!("Dataset loaded: {} samples", dataset.total_samples());

    // Build vocabulary
    let mut vocab_set = std::collections::HashSet::new();
    Vocab::process_text_for_vocab(&dataset.pretraining_data, &mut vocab_set);
    Vocab::process_text_for_vocab(&dataset.chat_training_data, &mut vocab_set);
    let mut words: Vec<String> = vocab_set.into_iter().collect();
    words.sort();
    let words_refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
    let vocab = Vocab::new(words_refs);
    info!("Vocabulary ready: {} tokens", vocab.size());

    // Initialize model
    let embeddings = Embeddings::new(vocab.clone());
    let transformer_1 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let transformer_2 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let transformer_3 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let output_proj = OutputProjection::new(EMBEDDING_DIM, vocab.size());

    let mut llm = LLM::new(
        vocab.clone(),
        vec![
            Box::new(embeddings),
            Box::new(transformer_1),
            Box::new(transformer_2),
            Box::new(transformer_3),
            Box::new(output_proj),
        ],
    );
    info!("Model initialized: {}", llm.network_description());
    info!("Total parameters: {}", llm.total_parameters());

    // Initialize metrics
    let mut metrics = Metrics::new(100);

    // Interactive mode
    println!("\n=== RustGPT Interactive Mode ===");
    println!("Type 'help' for commands, 'exit' to quit\n");

    let mut input = String::new();
    loop {
        input.clear();
        print!("rustgpt> ");
        std::io::stdout().flush()?;

        match std::io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => {
                info!("EOF or error, shutting down");
                break;
            }
            Ok(_) => {}
        }

        let command = input.trim();
        match command {
            "exit" | "quit" => {
                info!("User requested exit");
                println!("Goodbye!");
                break;
            }
            "help" => {
                println!("Commands:");
                println!("  prompt <text>    - Generate response to prompt");
                println!("  metrics          - Show training metrics");
                println!("  config           - Show current configuration");
                println!("  save <path>      - Save checkpoint");
                println!("  load <path>      - Load checkpoint");
                println!("  exit             - Quit");
            }
            cmd if cmd.starts_with("prompt ") => {
                let prompt = &cmd[7..];
                info!("Generating prediction for: {}", prompt);
                let response = llm.predict(prompt);
                println!("Response: {}\n", response);

                // Record metrics (simulated)
                metrics.record_loss(0.5);
                metrics.record_gradient_norm(0.02);
            }
            "metrics" => {
                println!("Metrics:");
                println!("  Avg Loss: {:.4}", metrics.avg_loss());
                println!("  Avg Gradient Norm: {:.4}", metrics.avg_gradient_norm());
                println!();
            }
            "config" => {
                println!("Configuration:");
                println!("  Embedding Dim: {}", config.model.embedding_dim);
                println!("  Hidden Dim: {}", config.model.hidden_dim);
                println!("  Max Seq Len: {}", config.model.max_seq_len);
                println!("  Learning Rate: {}", config.training.pretraining_lr);
                println!();
            }
            cmd if cmd.starts_with("save ") => {
                let path = &cmd[5..];
                match std::path::Path::new(path).parent() {
                    Some(parent) => {
                        std::fs::create_dir_all(parent)?;
                    }
                    None => {}
                }
                let checkpoint = Checkpoint::new(1, metrics.avg_loss(), "example");
                checkpoint.save(Path::new(path))?;
                println!("Checkpoint saved to {}", path);
                info!("Checkpoint saved");
            }
            cmd if cmd.starts_with("load ") => {
                let path = &cmd[5..];
                match Checkpoint::load(Path::new(path)) {
                    Ok(checkpoint) => {
                        println!("Loaded checkpoint from epoch {}", checkpoint.epoch);
                        info!("Checkpoint loaded successfully");
                    }
                    Err(e) => println!("Error loading checkpoint: {}", e),
                }
            }
            _ if !command.is_empty() => {
                println!("Unknown command. Type 'help' for available commands.");
            }
            _ => {} // Empty input
        }
    }

    info!("RustGPT shutdown complete");
    Ok(())
}
