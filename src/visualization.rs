//! Terminal-based visualization for training metrics using Ratatui.
//!
//! This module provides real-time visualization of training loss, accuracy, and
//! gradient norms in a terminal UI.

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

/// Configuration for the training visualization UI
#[derive(Clone, Debug)]
pub struct VisualizationConfig {
    /// Maximum number of loss values to display in the graph
    pub max_history: usize,
    /// Update interval in milliseconds
    pub update_interval_ms: u64,
    /// Enable interactive mode (allow quit with 'q')
    pub interactive: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            max_history: 100,
            update_interval_ms: 100,
            interactive: true,
        }
    }
}

/// Manages the training visualization UI
pub struct TrainingVisualizer {
    config: VisualizationConfig,
    loss_history: Vec<u64>,
    accuracy_history: Vec<u64>,
    gradient_history: Vec<u64>,
    current_epoch: usize,
    total_epochs: usize,
}

impl TrainingVisualizer {
    /// Create a new training visualizer
    pub fn new(config: VisualizationConfig, total_epochs: usize) -> Self {
        Self {
            config,
            loss_history: Vec::new(),
            accuracy_history: Vec::new(),
            gradient_history: Vec::new(),
            current_epoch: 0,
            total_epochs,
        }
    }

    /// Record a loss value and update the visualization
    pub fn record_loss(&mut self, loss: f32) {
        let loss_u64 = (loss * 10000.0) as u64;
        self.loss_history.push(loss_u64);
        if self.loss_history.len() > self.config.max_history {
            self.loss_history.remove(0);
        }
    }

    /// Record an accuracy value
    pub fn record_accuracy(&mut self, accuracy: f32) {
        let acc_u64 = (accuracy * 10000.0) as u64;
        self.accuracy_history.push(acc_u64);
        if self.accuracy_history.len() > self.config.max_history {
            self.accuracy_history.remove(0);
        }
    }

    /// Record a gradient norm value
    pub fn record_gradient(&mut self, gradient_norm: f32) {
        let grad_u64 = (gradient_norm * 10000.0) as u64;
        self.gradient_history.push(grad_u64);
        if self.gradient_history.len() > self.config.max_history {
            self.gradient_history.remove(0);
        }
    }

    /// Update the current epoch
    pub fn set_epoch(&mut self, epoch: usize) {
        self.current_epoch = epoch;
    }

    /// Get current loss value
    pub fn current_loss(&self) -> f32 {
        self.loss_history
            .last()
            .map(|&loss| loss as f32 / 10000.0)
            .unwrap_or(0.0)
    }

    /// Get current accuracy value
    pub fn current_accuracy(&self) -> f32 {
        self.accuracy_history
            .last()
            .map(|&acc| acc as f32 / 10000.0)
            .unwrap_or(0.0)
    }

    /// Create a line chart widget for loss visualization
    fn create_loss_line_chart(&self) -> impl Widget {
        let mut chart_content = String::new();

        if self.loss_history.is_empty() {
            chart_content = "Waiting for data...".to_string();
        } else {
            // Find min and max for scaling
            let max_loss = self.loss_history.iter().copied().max().unwrap_or(1000) as f64 / 10000.0;
            let min_loss = self.loss_history.iter().copied().min().unwrap_or(0) as f64 / 10000.0;

            let range = (max_loss - min_loss).max(0.01);

            // Simple bar chart - show all data
            let height = 10;
            let len = self.loss_history.len();

            // Sample data if too many epochs to fit on screen
            let display_width = 60;
            let step = (len / display_width).max(1);
            let displayed_len = len.div_ceil(step);

            // Build bar chart
            for row in 0..height {
                let level = max_loss - (range * row as f64 / height as f64);
                chart_content.push_str(&format!("{:6.2} │ ", level));

                for display_idx in 0..displayed_len {
                    let actual_idx = display_idx * step;
                    if actual_idx < len {
                        let loss_u64 = self.loss_history[actual_idx];
                        let loss = (loss_u64 as f64) / 10000.0;

                        // Calculate if this bar should show at this height
                        let bar_height = ((loss - min_loss) / range * height as f64) as usize;
                        let current_height = height - row - 1;

                        if bar_height > current_height {
                            chart_content.push('█');
                        } else {
                            chart_content.push(' ');
                        }
                    }
                }
                chart_content.push('\n');
            }

            // X-axis
            chart_content.push_str("       └");
            chart_content.push_str(&"─".repeat(displayed_len));
            chart_content.push('\n');

            // Labels
            chart_content.push_str(&format!(
                "        0{}{}\n",
                " ".repeat(displayed_len.saturating_sub(10) / 2),
                len
            ));
            chart_content.push_str(&format!(
                "Min: {:.4} | Max: {:.4} | Current: {:.4}",
                min_loss,
                max_loss,
                self.current_loss()
            ));
        }

        Paragraph::new(chart_content)
            .block(
                Block::default()
                    .title(" 📊 Loss Bar Chart ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Green).bold()),
            )
            .style(Style::default().fg(Color::Cyan))
    }

    /// Render the training dashboard to terminal
    pub fn render(&self, frame: &mut Frame, title: &str) {
        let size = frame.area();
        let loss_data = &self.loss_history;

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(size);

        // Title and progress
        let title_text = Paragraph::new(format!("⚙️  {}", title))
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center);
        frame.render_widget(title_text, chunks[0]);

        // Progress gauge
        let progress = (self.current_epoch as f64 / self.total_epochs.max(1) as f64) * 100.0;
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title("Epoch Progress")
                    .borders(Borders::ALL),
            )
            .gauge_style(Style::default().fg(Color::Green).bold())
            .percent(progress as u16)
            .label(format!("{}/{}", self.current_epoch, self.total_epochs));
        frame.render_widget(gauge, chunks[1]);

        // Loss graph and stats
        let graph_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(chunks[2]);

        // Line chart for loss visualization
        let loss_line = self.create_loss_line_chart();
        frame.render_widget(loss_line, graph_chunks[0]);

        // Stats panel
        let stats = format!(
            "Current Loss: {:.4}\nAccuracy: {:.2}%\nSamples: {}",
            self.current_loss(),
            self.current_accuracy(),
            loss_data.len()
        );
        let stats_widget = Paragraph::new(stats)
            .block(
                Block::default()
                    .title("Statistics")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::Green));

        frame.render_widget(stats_widget, graph_chunks[1]);

        // Footer
        let footer = Paragraph::new("Press 'q' to quit • Space to pause")
            .style(Style::default().fg(Color::Gray).italic())
            .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);
    }
}

/// Initialize terminal for UI rendering
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal to normal state
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Check for user input (non-blocking)
pub fn check_user_input() -> io::Result<Option<KeyCode>> {
    if event::poll(Duration::from_millis(0))? {
        if let Event::Key(key) = event::read()? {
            return Ok(Some(key.code));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let visualizer = TrainingVisualizer::new(VisualizationConfig::default(), 100);
        assert_eq!(visualizer.current_epoch, 0);
        assert_eq!(visualizer.total_epochs, 100);
        assert!(visualizer.loss_history.is_empty());
    }

    #[test]
    fn test_record_loss() {
        let mut visualizer = TrainingVisualizer::new(VisualizationConfig::default(), 100);
        visualizer.record_loss(0.5);
        assert_eq!(visualizer.current_loss(), 0.5);
    }

    #[test]
    fn test_loss_history_limit() {
        let config = VisualizationConfig {
            max_history: 5,
            ..Default::default()
        };
        let mut visualizer = TrainingVisualizer::new(config, 100);
        for i in 0..10 {
            visualizer.record_loss(i as f32 / 10.0);
        }
        assert_eq!(visualizer.loss_history.len(), 5);
    }

    #[test]
    fn test_epoch_progress() {
        let mut visualizer = TrainingVisualizer::new(VisualizationConfig::default(), 100);
        visualizer.set_epoch(50);
        assert_eq!(visualizer.current_epoch, 50);
    }
}
