# RustGPT - Production Enhanced

Enhanced fork of [github.com/tekaratzas/RustGPT](https://github.com/tekaratzas/RustGPT)

A Large Language Model implementation in pure Rust using only `ndarray` for matrix operations.

## Quick Start

```bash
# Training with live loss chart visualization
cargo run -- -v

# Standard training with progress bars
cargo run

# Run tests
cargo test
```

## Improvements

- **Production infrastructure**: Error handling, logging, configuration management
- **Model persistence**: Save/load trained models with checkpoint system
- **Real-time visualization**: Interactive terminal dashboard showing loss bar charts
- **Metrics collection**: Track loss, accuracy, and gradients during training
- **Bug fixes**: Fixed epoch numbering, progress bar display, and visualization scaling
- **Comprehensive tests**: 18 unit tests covering all modules



## Architecture

Core modules:
- `llm.rs` - Transformer LLM with attention, feed-forward, embeddings
- `self_attention.rs` - Multi-head self-attention mechanism
- `adam.rs` - Adam optimizer with gradient clipping
- `vocab.rs` - Tokenization and vocabulary management

Production modules:
- `error.rs`, `logging.rs`, `config.rs` - Infrastructure
- `checkpoint.rs` - Model persistence
- `metrics.rs` - Training metrics
- `visualization.rs`, `training_ui.rs` - Real-time dashboard

## Dependencies

- `ndarray` - Matrix operations
- `ratatui` + `crossterm` - Terminal UI
- `rand` - Random initialization

No PyTorch, TensorFlow, or heavy ML frameworks - pure Rust implementation.

## Testing

```bash
cargo test          # All 18 tests
cargo test --lib    # Library tests only
```

## Original Project

Full technical details: [github.com/tekaratzas/RustGPT](https://github.com/tekaratzas/RustGPT)
