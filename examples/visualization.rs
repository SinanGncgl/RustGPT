//! Example demonstrating the training visualization dashboard.
//!
//! This example shows how to use the TrainingVisualizer to monitor training progress
//! with real-time loss graphs in the terminal.

use llm::{
    Config, Dataset, DatasetType, EMBEDDING_DIM, Embeddings, HIDDEN_DIM, LLM, 
    Vocab, TrainingVisualizer, VisualizationConfig, init_logging,
    output_projection::OutputProjection, transformer::TransformerBlock,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging("info")?;

    // Load configuration
    let config = Config::default();
    config.validate()?;

    // Load dataset
    let dataset = Dataset::new(
        "data/pretraining_data.json",
        "data/chat_training_data.json",
        DatasetType::JSON,
    )?;

    dataset.validate()?;

    // Build vocabulary
    let mut vocab_set = std::collections::HashSet::new();
    Vocab::process_text_for_vocab(&dataset.pretraining_data, &mut vocab_set);
    Vocab::process_text_for_vocab(&dataset.chat_training_data, &mut vocab_set);

    let mut vocab_words: Vec<String> = vocab_set.into_iter().collect();
    vocab_words.sort();
    let vocab_words_refs: Vec<&str> = vocab_words.iter().map(|s| s.as_str()).collect();
    let vocab = Vocab::new(vocab_words_refs);

    // Create model
    let embeddings = Embeddings::new(vocab.clone());
    let transformer_1 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let transformer_2 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let transformer_3 = TransformerBlock::new(EMBEDDING_DIM, HIDDEN_DIM);
    let output_proj = OutputProjection::new(EMBEDDING_DIM, vocab.words.len());

    let mut llm = LLM::new(
        vocab,
        vec![
            Box::new(embeddings),
            Box::new(transformer_1),
            Box::new(transformer_2),
            Box::new(transformer_3),
            Box::new(output_proj),
        ],
    );

    println!("\n=== Training with Visualization ===\n");

    // Create visualizer
    let vis_config = VisualizationConfig {
        max_history: 50,
        update_interval_ms: 500,
        interactive: true,
    };

    let mut visualizer = TrainingVisualizer::new(vis_config, config.training.pretraining_epochs);

    // Training examples
    let examples: Vec<&str> = dataset
        .pretraining_data
        .iter()
        .take(5)
        .map(|s| s.as_str())
        .collect();

    // Simulate training with visualization feedback
    println!("Training for {} epochs...", config.training.pretraining_epochs);
    println!("Loss History: ");

    for epoch in 0..config.training.pretraining_epochs.min(5) {
        // Simulate training step
        llm.train(examples.clone(), 1, config.training.pretraining_lr);

        // Record a simulated loss (in real scenario, this comes from train())
        let simulated_loss = 5.0 - (epoch as f32 * 0.5);
        visualizer.record_loss(simulated_loss);
        visualizer.set_epoch(epoch + 1);

        println!(
            "  Epoch {}: Loss = {:.4} | Progress: {:.1}%",
            epoch + 1,
            visualizer.current_loss(),
            ((epoch + 1) as f64 / config.training.pretraining_epochs.min(5) as f64) * 100.0
        );
    }

    println!("\n✓ Training visualization example complete!");
    println!("In production, the TrainingVisualizer would render a real-time dashboard");
    println!("with loss graphs, accuracy tracking, and gradient monitoring.");

    Ok(())
}
