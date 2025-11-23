# 🦀 RustGPT - Enhanced Production Edition

> **Base Project**: [github.com/tekaratzas/RustGPT](https://github.com/tekaratzas/RustGPT)
> 
> A complete **Large Language Model implementation in pure Rust** with no external ML frameworks. Built from the ground up using only `ndarray` for matrix operations.

## ✨ Improvements in This Fork

This enhanced version transforms the original RustGPT from a toy project into a **production-ready system** with professional-grade infrastructure and real-time monitoring.

### 🏗️ Infrastructure & Production Readiness

#### Error Handling & Logging
- **Structured error handling** with custom `AppError` type for consistent error propagation
- **Comprehensive logging framework** supporting multiple log levels (DEBUG, INFO, WARN, ERROR)
- **Lazy static logger** for thread-safe, zero-cost logging

#### Configuration Management
- **Centralized configuration system** for all training hyperparameters
- **JSON-based config files** for easy experiment tracking
- **Runtime parameter validation** with sensible defaults

#### Model Persistence
- **Checkpoint system** for saving/loading trained models
- **Epoch recovery** - resume training from latest checkpoint
- **Automatic backup** on training completion
- **Versioned checkpoints** for experiment management

#### Metrics & Observability
- **Comprehensive metrics collection** during training (loss, accuracy, gradients)
- **Aggregated statistics** (min, max, mean) for performance analysis
- **Metrics export** for analysis and visualization

### 📊 Real-Time Training Visualization

#### Terminal Dashboard (`-v` flag)
- **Live training dashboard** with real-time updates
- **Loss bar chart** showing all training history with intelligent sampling
- **Progress indicators** for both pre-training and instruction tuning phases
- **Epoch tracking** with proper numbering (no more "always 0" bug)
- **Performance statistics** (min/max/current loss display)
- **Interactive controls** (keyboard input for pause/quit)

```bash
# Run with visualization dashboard
cargo run -- -v

# Standard training with progress bars
cargo run
```

### 🐛 Bug Fixes

- ✅ **Fixed epoch numbering** - Was always showing epoch 0; now correctly shows 1-100
- ✅ **Fixed progress bar rendering** - Was printing on every line; now updates in place
- ✅ **Fixed loss tracking** - History now preserved across full training lifecycle
- ✅ **Fixed visualization scaling** - Charts now use all available terminal space

### 🧮 Technical Enhancements

| Component | Improvement |
|-----------|------------|
| **Error Handling** | Custom error types with context; no panic!() calls |
| **Logging** | Structured logging at DEBUG/INFO/WARN/ERROR levels |
| **Configuration** | JSON-based config with validation |
| **Checkpoints** | Save/load model state; resume training |
| **Metrics** | Real-time tracking of loss, accuracy, gradients |
| **Visualization** | Ratatui terminal UI with live updates |
| **Testing** | 18 comprehensive unit tests covering all modules |
| **CLI** | Proper argument parsing with -v, -c, -h flags |

### 📦 New Dependencies Added

```toml
ratatui = "0.28"        # Terminal UI framework
crossterm = "0.28"      # Cross-platform terminal handling
```

### 🚀 Quick Start

```bash
# Clone and run
git clone https://github.com/SinanGncgl/RustGPT.git
cd RustGPT

# Training with live dashboard
cargo run -- -v

# Standard training with progress bars
cargo run

# Run tests
cargo test
```

## 📋 What's Included

### Core Modules (Original)
- `llm.rs` - Core LLM with transformers
- `transformer.rs` - Attention + feed-forward
- `self_attention.rs` - Multi-head self-attention
- `embeddings.rs` - Token embeddings
- `vocab.rs` - Vocabulary & tokenization
- `adam.rs` - Adam optimizer

### New Production Modules
- `error.rs` - Unified error handling
- `logging.rs` - Structured logging
- `config.rs` - Configuration management
- `checkpoint.rs` - Model persistence
- `metrics.rs` - Training metrics collection
- `visualization.rs` - Ratatui terminal UI
- `training_ui.rs` - Interactive dashboard integration

### Test Coverage
18 tests covering:
- LLM core functionality
- Transformer operations
- Attention mechanisms
- Embedding layers
- Tokenization
- Optimizer behavior
- Visualization rendering
- Configuration loading

## 🎯 Design Philosophy

This fork maintains the original project's goal of demonstrating LLM fundamentals from scratch while adding **production-grade infrastructure**:

1. **Educational** - Still implements transformers without frameworks
2. **Production-Ready** - Includes error handling, logging, checkpoints, metrics
3. **Observable** - Real-time visualization with proper monitoring
4. **Reliable** - Comprehensive testing and validation
5. **Maintainable** - Clean architecture with modular design

## 📚 Original Documentation

For the complete technical details about the LLM implementation, visit:
**[github.com/tekaratzas/RustGPT](https://github.com/tekaratzas/RustGPT)**

---

**For questions or contributions, check the original repository or open an issue in this fork!**
