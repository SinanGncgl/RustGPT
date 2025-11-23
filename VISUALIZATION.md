# Training Visualization Guide

## Overview

RustGPT now includes a **terminal-based visualization dashboard** using Ratatui for real-time monitoring of training metrics during model training. This provides a rich, interactive UI to track loss curves, training progress, and key statistics.

## Features

- **Real-time Loss Graph**: Sparkline visualization of loss values during training
- **Epoch Progress Bar**: Visual indicator of training progress
- **Live Statistics**: Current loss, accuracy, and sample count displayed in real-time
- **Interactive Mode**: Press 'q' to quit, Space to pause (optional)
- **Low Overhead**: Efficient rendering with minimal performance impact

## Installation

The visualization module is automatically included. Dependencies are already in `Cargo.toml`:

```toml
ratatui = "0.28"
crossterm = { version = "0.28", features = ["events"] }
```

## Basic Usage

### Creating a Visualizer

```rust
use llm::{TrainingVisualizer, VisualizationConfig};

// Default configuration
let visualizer = TrainingVisualizer::new(
    VisualizationConfig::default(),
    100  // total epochs
);

// Custom configuration
let config = VisualizationConfig {
    max_history: 50,           // Show last 50 loss values
    update_interval_ms: 100,   // Update every 100ms
    interactive: true,         // Allow user input
};
let visualizer = TrainingVisualizer::new(config, epochs);
```

### Recording Metrics

```rust
// Record loss value during training
visualizer.record_loss(0.5234);

// Record accuracy
visualizer.record_accuracy(0.92);

// Record gradient norm
visualizer.record_gradient(0.0012);

// Update current epoch
visualizer.set_epoch(epoch_number);
```

### Rendering the Dashboard

```rust
// Initialize terminal
let mut terminal = llm::visualization::init_terminal()?;

// Main event loop
loop {
    terminal.draw(|frame| {
        visualizer.render(frame);
    })?;

    // Check for user input
    if let Some(key) = llm::visualization::check_user_input()? {
        match key {
            KeyCode::Char('q') => break,
            KeyCode::Char(' ') => { /* pause */ },
            _ => {}
        }
    }
}

// Restore terminal
llm::visualization::restore_terminal(&mut terminal)?;
```

## Integration with Training Loop

The `train_with_visualizer` method integrates loss recording directly:

```rust
let mut visualizer = TrainingVisualizer::new(
    VisualizationConfig::default(),
    config.training.pretraining_epochs
);

llm.train_with_visualizer(
    training_examples,
    config.training.pretraining_epochs,
    config.training.pretraining_lr,
    None,  // progress bar (optional)
    Some(&mut visualizer),
);
```

## Dashboard Components

### Header
- Title: "⚙️  Training Dashboard"
- Clear visual indicator that you're monitoring training

### Progress Gauge
- Shows current epoch vs total epochs
- Visual bar representation
- Percentage display

### Loss Graph
- Sparkline visualization of loss history
- Automatically scales to data
- Shows recent 100 samples (configurable)

### Statistics Panel
- **Current Loss**: Last recorded loss value
- **Accuracy**: Latest accuracy metric
- **Samples**: Number of recorded values in history

### Footer
- Instructions: "Press 'q' to quit • Space to pause"
- Interactive hints for user control

## Configuration Options

```rust
pub struct VisualizationConfig {
    /// Maximum number of loss values to display (default: 100)
    pub max_history: usize,
    
    /// Update interval in milliseconds (default: 100)
    pub update_interval_ms: u64,
    
    /// Enable interactive mode with user input handling (default: true)
    pub interactive: bool,
}
```

## Example

See `examples/visualization.rs` for a complete working example:

```bash
cargo run --example visualization
```

This example demonstrates:
- Creating a visualizer
- Recording loss values
- Displaying metrics
- Proper cleanup

## API Reference

### TrainingVisualizer

```rust
impl TrainingVisualizer {
    pub fn new(config: VisualizationConfig, total_epochs: usize) -> Self;
    pub fn record_loss(&mut self, loss: f32);
    pub fn record_accuracy(&mut self, accuracy: f32);
    pub fn record_gradient(&mut self, gradient_norm: f32);
    pub fn set_epoch(&mut self, epoch: usize);
    pub fn current_loss(&self) -> f32;
    pub fn current_accuracy(&self) -> f32;
    pub fn render(&self, frame: &mut Frame);
}
```

### Terminal Functions

```rust
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>>;
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()>;
pub fn check_user_input() -> io::Result<Option<KeyCode>>;
```

## Performance Considerations

- **History Limit**: Default 100 values; adjust based on memory constraints
- **Update Frequency**: 100ms default; reduce for higher frequency updates
- **Rendering**: Ratatui is optimized for terminal rendering with minimal overhead
- **Non-blocking**: User input checking is non-blocking and won't freeze training

## Troubleshooting

### "Terminal not responding"
- Ensure you call `init_terminal()` before drawing
- Check that event polling is working with `check_user_input()`

### "Graph not updating"
- Verify `record_loss()` is being called each epoch
- Check that `set_epoch()` is also called to update progress

### "Colors not showing"
- Ensure your terminal supports colors (most modern terminals do)
- Try a different terminal if colors don't appear

## Future Enhancements

Planned improvements for the visualization module:

- [ ] Multi-chart view (loss + accuracy + gradient norms)
- [ ] Export graph to image/HTML
- [ ] Keyboard shortcuts for zooming/panning
- [ ] Real-time performance metrics (samples/sec)
- [ ] Customizable color schemes
- [ ] Recording and replay of training sessions

## License

Part of RustGPT - MIT License
