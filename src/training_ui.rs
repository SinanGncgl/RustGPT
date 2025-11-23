//! Interactive training with real-time visualization dashboard.
//!
//! This module provides a complete training loop that displays a Ratatui dashboard
//! with live loss graphs, progress indicators, and training statistics.

use crate::{
    visualization::{
        check_user_input, init_terminal, restore_terminal, TrainingVisualizer, VisualizationConfig,
    },
    LLM,
};
use crossterm::event::KeyCode;
use indicatif::ProgressBar;
use std::time::Duration;

/// Run training with interactive visualization dashboard
pub fn train_with_dashboard(
    llm: &mut LLM,
    training_data: Vec<&str>,
    epochs: usize,
    learning_rate: f32,
    title: &str,
) -> crate::Result<()> {
    // Initialize terminal UI
    let mut terminal = init_terminal()
        .map_err(|e| crate::LlmError::Other(format!("Failed to init terminal: {}", e)))?;

    // Create visualizer
    let vis_config = VisualizationConfig {
        max_history: 100,
        update_interval_ms: 100,
        interactive: true,
    };
    let mut visualizer = TrainingVisualizer::new(vis_config, epochs);

    // Progress bar for actual training (runs in background)
    let pb = ProgressBar::new(epochs as u64);
    pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());

    // Training loop with dashboard
    for epoch in 0..epochs {
        // Tokenize data once per epoch
        let tokenized_data: Vec<Vec<usize>> = training_data
            .iter()
            .map(|input| llm.tokenize(input))
            .collect();

        // Training batch
        let mut total_loss = 0.0;
        for training_row in &tokenized_data {
            if training_row.len() < 2 {
                continue;
            }

            let input_ids = &training_row[..training_row.len() - 1];
            let target_ids = &training_row[1..];

            // Forward pass
            let mut input = ndarray::Array2::zeros((1, input_ids.len()));
            input.row_mut(0).assign(
                &input_ids
                    .iter()
                    .map(|&x| x as f32)
                    .collect::<ndarray::Array1<f32>>(),
            );

            for layer in &mut llm.network {
                input = layer.forward(&input);
            }

            let logits = input;
            let probs = LLM::softmax(&logits);
            total_loss += LLM::cross_entropy_loss_step(&probs, target_ids);

            // Backward pass
            let mut grads_output = LLM::compute_gradients_step(&probs, target_ids);
            LLM::clip_gradients(&mut grads_output, 5.0);

            for layer in llm.network.iter_mut().rev() {
                grads_output = layer.backward(&grads_output, learning_rate);
            }
        }

        // Calculate average loss
        let avg_loss = total_loss / tokenized_data.len().max(1) as f32;

        // Update visualizer
        visualizer.record_loss(avg_loss);
        visualizer.set_epoch(epoch + 1);

        // Render dashboard
        terminal
            .draw(|frame| {
                visualizer.render(frame, title);
            })
            .map_err(|e| crate::LlmError::Other(format!("Failed to draw: {}", e)))?;

        // Check for user input (non-blocking)
        match check_user_input()
            .map_err(|e| crate::LlmError::Other(format!("Input error: {}", e)))?
        {
            Some(KeyCode::Char('q')) => {
                tracing::info!("User requested quit");
                break;
            }
            Some(KeyCode::Char('p')) => {
                // Pause - hold terminal open
                tracing::info!("Training paused");
            }
            _ => {}
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Show final dashboard
    terminal
        .draw(|frame| {
            visualizer.render(frame, title);
        })
        .map_err(|e| crate::LlmError::Other(format!("Failed to draw final frame: {}", e)))?;

    // Hold the dashboard for 2 seconds before restoring
    std::thread::sleep(Duration::from_secs(2));

    // Restore terminal
    restore_terminal(&mut terminal)
        .map_err(|e| crate::LlmError::Other(format!("Failed to restore terminal: {}", e)))?;

    Ok(())
}
