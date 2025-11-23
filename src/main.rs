//! RustGPT: A transformer-based LLM implementation in pure Rust.
//!
//! This binary trains and runs an interactive LLM with support for configuration,
//! checkpointing, and graceful shutdown.

use clap::Parser;
use indicatif::ProgressBar;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

use llm::{
    Config, Dataset, DatasetType, EMBEDDING_DIM, Embeddings, HIDDEN_DIM, LLM, MAX_SEQ_LEN,
    Result as LlmResult, Vocab, init_logging, output_projection::OutputProjection,
    transformer::TransformerBlock,
};

/// Command-line arguments for the LLM
#[derive(Parser, Debug)]
#[command(name = "RustGPT")]
#[command(about = "A transformer-based LLM in pure Rust", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short = 'c', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Training mode: enable training instead of interactive mode
    #[arg(short = 't', long)]
    train: bool,

    /// Load model from checkpoint
    #[arg(short = 'k', long, value_name = "FILE")]
    checkpoint: Option<PathBuf>,

    /// Logging level (debug, info, warn, error)
    #[arg(short = 'l', long, default_value = "info")]
    log_level: String,

    /// Enable visualization dashboard during training
    #[arg(short = 'v', long)]
    visualize: bool,

    /// Path to pretraining data
    #[arg(long, value_name = "FILE")]
    pretraining_data: Option<PathBuf>,

    /// Path to chat training data
    #[arg(long, value_name = "FILE")]
    chat_training_data: Option<PathBuf>,

    /// Output directory for checkpoints
    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,
}

fn main() -> LlmResult<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(&args.log_level)
        .map_err(|e| llm::LlmError::Other(format!("Failed to initialize logging: {}", e)))?;

    info!("RustGPT v{} starting", llm::VERSION);

    // Load or create configuration
    let mut config = if let Some(config_path) = args.config {
        info!("Loading configuration from {:?}", config_path);
        Config::from_toml(&config_path)?
    } else {
        Config::default()
    };

    // Override configuration with CLI arguments
    if let Some(path) = args.pretraining_data {
        config.data.pretraining_data = path.to_string_lossy().to_string();
    }
    if let Some(path) = args.chat_training_data {
        config.data.chat_training_data = path.to_string_lossy().to_string();
    }
    if let Some(path) = args.output {
        config.output.checkpoint_dir = path.to_string_lossy().to_string();
    }

    // Validate configuration
    config.validate()?;
    info!("Configuration loaded and validated");
    info!(
        "Model config: embedding_dim={}, hidden_dim={}, max_seq_len={}",
        config.model.embedding_dim, config.model.hidden_dim, config.model.max_seq_len
    );

    // Load dataset
    info!(
        "Loading dataset from {:?} and {:?}",
        config.data.pretraining_data, config.data.chat_training_data
    );

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
    info!("Dataset loaded: {} total samples", dataset.total_samples());

    // Build vocabulary from dataset
    info!("Building vocabulary...");
    let mut vocab_set = std::collections::HashSet::new();
    Vocab::process_text_for_vocab(&dataset.pretraining_data, &mut vocab_set);
    Vocab::process_text_for_vocab(&dataset.chat_training_data, &mut vocab_set);

    let mut vocab_words: Vec<String> = vocab_set.into_iter().collect();
    vocab_words.sort();
    let vocab_words_refs: Vec<&str> = vocab_words.iter().map(|s| s.as_str()).collect();
    let vocab = Vocab::new(vocab_words_refs);
    info!("Vocabulary built with {} tokens", vocab.size());

    // Create model layers
    info!("Initializing model layers...");
    let transformer_block_1 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let transformer_block_2 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let transformer_block_3 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let output_projection = OutputProjection::new(EMBEDDING_DIM, vocab.words.len());
    let embeddings = Embeddings::new(vocab.clone());

    let mut llm = LLM::new(
        vocab.clone(),
        vec![
            Box::new(embeddings),
            Box::new(transformer_block_1),
            Box::new(transformer_block_2),
            Box::new(transformer_block_3),
            Box::new(output_projection),
        ],
    );

    println!("\n=== MODEL INFORMATION ===");
    println!("Network architecture: {}", llm.network_description());
    println!(
        "Model configuration -> max_seq_len: {}, embedding_dim: {}, hidden_dim: {}",
        MAX_SEQ_LEN, EMBEDDING_DIM, HIDDEN_DIM
    );
    println!("Total parameters: {}", llm.total_parameters());

    let test_input = "User: How do mountains form?";
    println!("\n=== BEFORE TRAINING ===");
    println!("Input: {}", test_input);
    println!("Output: {}", llm.predict(test_input));

    // Training phase
    info!("Starting training phase...");

    // Pre-training
    println!("\n=== PRE-TRAINING MODEL ===");
    info!(
        "Pre-training on {} examples for {} epochs with learning rate {}",
        dataset.pretraining_data.len(),
        config.training.pretraining_epochs,
        config.training.pretraining_lr
    );

    let pretraining_examples: Vec<&str> = dataset
        .pretraining_data
        .iter()
        .map(|s| s.as_str())
        .collect();

    // Use visualization dashboard if -v flag is set, otherwise use progress bar
    if args.visualize {
        llm::training_ui::train_with_dashboard(
            &mut llm,
            pretraining_examples.clone(),
            config.training.pretraining_epochs,
            config.training.pretraining_lr,
            "Pre-training",
        )?;
    } else {
        let pb = ProgressBar::new(config.training.pretraining_epochs as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg}\n[{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
        );
        llm.train_with_progress(
            pretraining_examples.clone(),
            config.training.pretraining_epochs,
            config.training.pretraining_lr,
            Some(&pb),
        );
        pb.finish_with_message("✓ Pre-training complete");
    }

    // Instruction tuning
    println!("\n=== INSTRUCTION TUNING ===");
    let chat_training_examples: Vec<&str> = dataset
        .chat_training_data
        .iter()
        .map(|s| s.as_str())
        .collect();

    info!(
        "Instruction tuning on {} examples for {} epochs with learning rate {}",
        dataset.chat_training_data.len(),
        config.training.finetuning_epochs,
        config.training.finetuning_lr
    );

    if args.visualize {
        llm::training_ui::train_with_dashboard(
            &mut llm,
            chat_training_examples.clone(),
            config.training.finetuning_epochs,
            config.training.finetuning_lr,
            "Instruction Tuning",
        )?;
    } else {
        let pb = ProgressBar::new(config.training.finetuning_epochs as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg}\n[{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
        );
        llm.train_with_progress(
            chat_training_examples.clone(),
            config.training.finetuning_epochs,
            config.training.finetuning_lr,
            Some(&pb),
        );
        pb.finish_with_message("✓ Instruction tuning complete");
    }

    println!("\n=== AFTER TRAINING ===");
    println!("Input: {}", test_input);
    let result = llm.predict(test_input);
    println!("Output: {}", result);
    println!("======================\n");

    info!("Training completed successfully");

    // Interactive mode
    println!("\n--- Interactive Mode ---");
    println!("Type a prompt and press Enter to generate text.");
    println!("Type 'exit' to quit.");
    info!("Entering interactive mode");

    let mut input = String::new();
    loop {
        input.clear();
        print!("\nEnter prompt: ");
        std::io::stdout().flush().unwrap();

        if std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| llm::LlmError::Other(format!("Failed to read input: {}", e)))?
            == 0
        {
            info!("EOF reached, exiting");
            break;
        }

        let trimmed_input = input.trim();
        if trimmed_input.eq_ignore_ascii_case("exit") {
            info!("User requested exit");
            println!("Exiting interactive mode.");
            break;
        }

        if trimmed_input.is_empty() {
            continue;
        }

        let formatted_input = format!("User: {}", trimmed_input);
        info!("Generating prediction for: {}", formatted_input);
        let prediction = llm.predict(&formatted_input);
        println!("Model output: {}", prediction);
    }

    info!("RustGPT shutdown complete");
    Ok(())
}
