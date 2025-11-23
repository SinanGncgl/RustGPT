# Migration Guide: Old API → New Production API

This guide helps you migrate from the old toy implementation to the new production-ready API.

## Overview of Changes

The codebase has been refactored for production while maintaining core functionality. Most changes are additive, but some APIs now return `Result<T>` instead of panicking.

---

## 1. Dataset Loading

### Old API
```rust
let dataset = Dataset::new(
    String::from("data/pretraining.json"),
    String::from("data/chat.json"),
    DatasetType::JSON,
);
// Would panic on file not found
```

### New API
```rust
let dataset = Dataset::new(
    "data/pretraining.json",
    "data/chat.json",
    DatasetType::JSON,
)?;  // Returns Result<Dataset, LlmError>

// Optional: validate dataset
dataset.validate()?;

// New: Get dataset statistics
println!("Samples: {}", dataset.total_samples());
```

### Migration Steps
1. Add `?` operator (requires returning `Result`)
2. Handle error cases gracefully
3. Use `&str` instead of `String` for paths

---

## 2. Vocabulary Management

### Old API
```rust
let vocab = Vocab::new(vocab_words_refs);
// Debug println included in constructor

let token_id = vocab.encode("word");  // Option<usize>
let word = vocab.decode(token_id);    // Option<&String>
```

### New API
```rust
let vocab = Vocab::new(vocab_words_refs);
// No debug output (use logging instead)

// Same as before (backward compatible)
let token_id = vocab.encode("word");  // Option<usize>

// New: Error-aware variants
let token_id = vocab.encode_or_error("word")?;  // Result<usize>
let word = vocab.decode_or_error(token_id)?;    // Result<String>

// New: Utility methods
vocab.size()
vocab.contains("word")
vocab.from_texts(&texts)
vocab.statistics()
```

### Migration Steps
1. Optional: Use `_or_error` variants for better error handling
2. No code changes required for existing code
3. Remove any workarounds for debug output

---

## 3. Main Application Flow

### Old API
```rust
fn main() {
    // Hardcoded paths and values
    let string = String::from("User: How do mountains form?");
    
    let dataset = Dataset::new(
        String::from("data/pretraining_data.json"),
        String::from("data/chat_training_data.json"),
        DatasetType::JSON,
    );
    
    // ... training with hardcoded epochs and LR
    llm.train(pretraining_examples, 50, 0.0005);
}
```

### New API
```rust
fn main() -> Result<()> {
    // Initialize logging
    init_logging("info")?;
    
    // Load configuration
    let config = Config::from_toml(Path::new("config.toml"))?;
    config.validate()?;
    
    // Load dataset
    let dataset = Dataset::new(
        &config.data.pretraining_data,
        &config.data.chat_training_data,
        DatasetType::JSON,
    )?;
    dataset.validate()?;
    
    // ... training with configurable values
    llm.train(
        pretraining_examples,
        config.training.pretraining_epochs,
        config.training.pretraining_lr,
    );
    
    Ok(())
}
```

### Migration Steps
1. Create `config.toml` from `config.example.toml`
2. Initialize logging early: `init_logging("info")?`
3. Change `fn main()` to `fn main() -> Result<()>`
4. Use configuration values instead of hardcoded constants
5. Add `?` for error handling
6. Return `Ok(())` at the end

---

## 4. Error Handling

### Old API
```rust
// Explicit unwrap calls
let data = fs::read_to_string(path).unwrap();
let config: Config = toml::from_str(&data).unwrap();

// Panics on None
let vocab = Vocab::encode("word").unwrap();
```

### New API
```rust
// Use ? operator with Result types
let data = fs::read_to_string(path)?;
let config = Config::from_toml(path)?;

// Error-aware variants
let token_id = vocab.encode_or_error("word")?;

// Or match on Option
match vocab.encode("word") {
    Some(id) => println!("Token: {}", id),
    None => eprintln!("Unknown word"),
}
```

### Migration Steps
1. Replace `.unwrap()` with `?` operator
2. Use error-aware method variants where available
3. Ensure function returns `Result<T>`
4. Add error context where needed: `.context("message")?`

---

## 5. Logging

### Old API
```rust
println!("Training started");
println!("Epoch {}: Loss = {:.4}", epoch, loss);
```

### New API
```rust
use tracing::info;

info!("Training started");
info!(epoch, loss = loss, "Epoch completed");
```

### Migration Steps
1. Import logging macros: `use tracing::{info, warn, debug, error}`
2. Replace `println!` with appropriate log level
3. Initialize logging: `init_logging("info")?` in main
4. No binary changes needed - just log messages

---

## 6. Configuration

### Old API
```rust
const MAX_SEQ_LEN: usize = 80;
const EMBEDDING_DIM: usize = 128;
const HIDDEN_DIM: usize = 256;
const PRETRAINING_EPOCHS: usize = 50;
```

### New API
```rust
// Create config.toml
let config = Config::from_toml(Path::new("config.toml"))?;

// Access values
config.model.embedding_dim
config.model.max_seq_len
config.training.pretraining_epochs

// From environment
let config = Config::from_env()?;
```

### Migration Steps
1. Create `config.toml` with desired settings
2. Load in main: `let config = Config::from_toml(...)?`
3. Pass config through application
4. Replace hardcoded constants with config values

---

## 7. Model Checkpointing

### Old API
```rust
// No checkpointing - all state lost
llm.train(examples, epochs, lr);
// If interrupted, must restart from scratch
```

### New API
```rust
let manager = CheckpointManager::new(
    Path::new("./checkpoints"),
    true,  // keep best checkpoints
    5,     // max 5 checkpoints
)?;

// After training
let checkpoint = Checkpoint::new(epoch, loss, config);
manager.save(&checkpoint)?;

// Load best checkpoint
let checkpoint = manager.load_best()?;
```

### Migration Steps
1. Create CheckpointManager in main
2. Save checkpoints periodically
3. Load from checkpoint if resuming training
4. No code changes to training loop itself

---

## 8. Metrics Tracking

### Old API
```rust
// Manual tracking
let mut total_loss = 0.0;
for epoch in 0..epochs {
    // ... training ...
    total_loss += loss;
}
println!("Final loss: {}", total_loss);
```

### New API
```rust
let mut metrics = Metrics::new(100);  // 100-step window

for epoch in 0..epochs {
    // ... training ...
    metrics.record_loss(loss);
    metrics.record_gradient_norm(norm);
}

// Query metrics
println!("Avg loss: {:.4}", metrics.avg_loss());
println!("Trend: {:?}", metrics.loss_trend());

// Export
println!("{}", metrics.to_csv());
```

### Migration Steps
1. Create Metrics instance
2. Call `record_*` methods during training
3. Query metrics for analysis
4. Export to CSV/JSON for visualization

---

## 9. CLI Arguments

### Old API
```rust
// No CLI interface
// Modify source code to change parameters
```

### New API
```bash
# Show help
./llm --help

# Run with configuration
./llm --config config.toml --train --log-level debug

# Override settings
./llm --pretraining-data data/custom.json
```

### Migration Steps
1. Parse arguments in main
2. Use clap framework automatically (included)
3. Accept configuration from command line
4. No code changes for existing functionality

---

## 10. Library vs Binary

### Old API
```rust
// Library only exported core types
pub use llm::{LLM, Layer};
// Main function was coupled to library
```

### New API
```rust
// Library exports more abstractions
pub use llm::{
    LLM, Layer, Config, Dataset, Checkpoint,
    Metrics, Result, LlmError,
};

// Binary in main.rs is independent
// Can be replaced with custom binary
```

### Migration Steps
1. Use re-exported types from main library
2. `use llm::*` brings in all public items
3. Create custom binary for your needs
4. Original main.rs serves as template

---

## Complete Migration Example

### Before (Old Code)
```rust
use llm::{EMBEDDING_DIM, HIDDEN_DIM, MAX_SEQ_LEN};
use llm::{Dataset, DatasetType, Vocab, LLM, /* ... */};

fn main() {
    let dataset = Dataset::new(
        String::from("data/pretraining_data.json"),
        String::from("data/chat_training_data.json"),
        DatasetType::JSON,
    );

    let mut vocab_set = std::collections::HashSet::new();
    Vocab::process_text_for_vocab(&dataset.pretraining_data, &mut vocab_set);
    Vocab::process_text_for_vocab(&dataset.chat_training_data, &mut vocab_set);

    // ... build model ...

    llm.train(pretraining_examples, 50, 0.0005);
    llm.train(chat_training_examples, 50, 0.0001);

    // Interactive loop...
}
```

### After (New Code)
```rust
use llm::{
    Config, Dataset, DatasetType, Vocab, LLM,
    init_logging, Result as LlmResult,
    EMBEDDING_DIM, HIDDEN_DIM,
};
use tracing::info;

fn main() -> LlmResult<()> {
    // Initialize logging
    init_logging("info")?;
    info!("Starting application");

    // Load configuration
    let config = Config::default();
    config.validate()?;

    // Load dataset with error handling
    let dataset = Dataset::new(
        &config.data.pretraining_data,
        &config.data.chat_training_data,
        DatasetType::JSON,
    )?;
    dataset.validate()?;
    info!("Dataset loaded");

    // Build vocabulary
    let mut vocab_set = std::collections::HashSet::new();
    Vocab::process_text_for_vocab(&dataset.pretraining_data, &mut vocab_set);
    Vocab::process_text_for_vocab(&dataset.chat_training_data, &mut vocab_set);

    // ... build model (same as before) ...

    // Training with metrics
    let mut metrics = llm::Metrics::new(100);
    for epoch in 0..config.training.pretraining_epochs {
        llm.train(
            pretraining_examples.clone(),
            1,
            config.training.pretraining_lr,
        );
        info!(epoch, loss = metrics.avg_loss(), "Training progress");
    }

    // Interactive loop (same as before)
    Ok(())
}
```

---

## Backward Compatibility

### What Remains Unchanged
- ✅ Core LLM training/inference logic
- ✅ Layer trait implementations
- ✅ Model architecture (transformers, attention, etc.)
- ✅ Vocabulary encoding/decoding (with new error variants)
- ✅ Dataset formats (JSON/CSV)

### What Changed
- ⚠️ `Dataset::new()` now returns `Result`
- ⚠️ `main()` signature (added error handling)
- ⚠️ No more `println!` output (use logging)
- ⚠️ No more hardcoded configuration

### Migration Effort
- **Low**: Just use new APIs with `?` operator
- **Estimated Time**: 30 minutes for typical application

---

## Troubleshooting

### Error: "cannot find type `Result` in scope"
**Solution**: Add `use llm::Result as LlmResult` or return `Result<()>`

### Error: "function declared as returning `()` but panicked"
**Solution**: Change `fn main()` to `fn main() -> Result<()>` and add `?` operators

### Error: "Dataset::new does not accept String"
**Solution**: Use `&str` or `&Path` instead of `String`

### Missing Logging Output
**Solution**: Call `init_logging("debug")?` in main before any logs

### Configuration File Not Found
**Solution**: Create `config.toml` from `config.example.toml` or use `Config::default()`

---

## Resources

- **Configuration Guide**: See `PRODUCTION.md`
- **API Documentation**: `cargo doc --open`
- **Examples**: `cargo run --example interactive`
- **Tests**: `cargo test` to verify installation

---

## Support

For issues during migration:
1. Check this guide thoroughly
2. Review example in `examples/interactive.rs`
3. Run tests: `cargo test`
4. Check logs: `RUST_LOG=debug cargo run`
