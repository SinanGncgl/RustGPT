# Using the Training Visualization Dashboard

## Quick Start

Run the model with the real-time training visualization dashboard:

```bash
cargo run -- --visualize
```

Or with short flag:

```bash
cargo run -- -v
```

## What You'll See

During training, you'll see an interactive terminal dashboard with:

```
┌─────────────────────── ⚙️  Pre-training ─────────────────────────┐
│                                                                   │
├───────────────────────── Epoch Progress ────────────────────────┤
│[████████████░░░░░░░░░░░░░░░░░░░░░░░░░░] 42/100                  │
├──────────────────────── Loss Trend ──────────────────────────────┤
│ ╱╲╲  ╲                                          Loss Trend      │
│ │ ╲╱╱═╲╲  ╲                                 ┌──────────────────┤
│ │      ╲╱═╲╱═                              │ Current Loss: 0.45│
│ │                                           │ Accuracy: 92.3%  │
│ │                                           │ Samples: 42      │
│ │                                           │ Statistics      │
├──────────────────────────────────────────────────────────────────┤
│ Press 'q' to quit • Space to pause                               │
└──────────────────────────────────────────────────────────────────┘
```

## Dashboard Components

### 1. **Loss Trend Graph** (Left side)
- Sparkline visualization showing loss over time
- Real-time updates as training progresses
- Shows the last 100 data points (configurable)

### 2. **Epoch Progress Bar** (Top)
- Visual progress indicator
- Shows current epoch number / total epochs
- Percentage completion

### 3. **Statistics Panel** (Right side)
- **Current Loss**: Latest training loss value
- **Accuracy**: Current accuracy metric
- **Samples**: Number of loss values recorded

### 4. **Interactive Controls** (Footer)
- Press 'q' to quit and exit visualization
- Press 'p' to pause (optional)

## Example Commands

### Basic visualization (default config)
```bash
cargo run -- --visualize
```

### With custom configuration file
```bash
cargo run -- --config config.toml --visualize
```

### Adjust pre-training epochs before visualization
Edit `config.toml`:
```toml
[training]
pretraining_epochs = 50  # Will show 50 epochs in progress bar
finetuning_epochs = 30   # Then 30 for instruction tuning
```

Then run:
```bash
cargo run -- -c config.toml -v
```

### Without visualization (just progress bars)
```bash
cargo run  # No --visualize flag
```

## Running Visualization in Different Terminals

The visualization uses Ratatui which supports most modern terminals:

### macOS
- Terminal.app ✓
- iTerm2 ✓
- Alacritty ✓

### Linux
- GNOME Terminal ✓
- Konsole ✓
- xterm ✓

### Windows
- Windows Terminal ✓
- ConEmu ✓

## What Gets Visualized

### Pre-training Phase
- Shows loss decreasing as model learns general language patterns
- Graph typically shows steep initial decline, then gradual improvement
- Title: "⚙️  Pre-training"

### Instruction Tuning Phase
- Shows loss as model adapts to Q&A format
- Graph typically shows smaller adjustments
- Title: "⚙️  Instruction Tuning"

## Tips for Best Results

1. **Terminal Size**: Ensure your terminal is at least 80x24 characters
2. **Full Screen**: For best visualization, use full terminal window
3. **Color Support**: Terminal should support 256 colors for best appearance
4. **Real-time Updates**: Dashboard updates every 100ms by default
5. **Monitor Progress**: Watch the sparkline for learning trends

## Performance Impact

The visualization dashboard has minimal performance impact:
- Non-blocking terminal rendering
- Efficient Ratatui backend
- Training performance reduced by <5%

## Troubleshooting

### Dashboard doesn't render
- Ensure terminal supports Ratatui (most modern terminals do)
- Try resizing terminal window
- Check terminal size is at least 80x24

### Loss not updating
- Verify training is running (check log messages)
- Ensure data loading was successful
- Check dataset has valid training samples

### Terminal gets corrupted
- Press 'q' to exit cleanly
- If stuck, press Ctrl+C to force exit
- Then run `reset` to restore terminal

## Advanced: Custom Visualization Config

In code, create custom visualization configuration:

```rust
use llm::{TrainingVisualizer, VisualizationConfig};

let config = VisualizationConfig {
    max_history: 200,        // Show more history
    update_interval_ms: 50,  // Update more frequently
    interactive: true,       // Allow user input
};

let visualizer = TrainingVisualizer::new(config, 100);
```

## Next Steps

- Modify training parameters in `config.toml`
- Analyze loss trends to tune learning rate
- Export loss data for further analysis
- Use visualization to detect training issues early

Happy training! 🚀
