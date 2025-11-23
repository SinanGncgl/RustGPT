//! Metrics and monitoring for training progress and model performance.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Training metrics tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Loss history
    losses: VecDeque<f32>,
    /// Training accuracies
    accuracies: VecDeque<f32>,
    /// Gradient norms
    gradient_norms: VecDeque<f32>,
    /// Learning rates used
    learning_rates: VecDeque<f32>,
    /// Maximum window size
    window_size: usize,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new(100)
    }
}

impl Metrics {
    /// Create a new metrics tracker with a window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            losses: VecDeque::with_capacity(window_size),
            accuracies: VecDeque::with_capacity(window_size),
            gradient_norms: VecDeque::with_capacity(window_size),
            learning_rates: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Record a loss value.
    pub fn record_loss(&mut self, loss: f32) {
        self.losses.push_back(loss);
        if self.losses.len() > self.window_size {
            self.losses.pop_front();
        }
    }

    /// Record an accuracy value.
    pub fn record_accuracy(&mut self, accuracy: f32) {
        self.accuracies.push_back(accuracy);
        if self.accuracies.len() > self.window_size {
            self.accuracies.pop_front();
        }
    }

    /// Record a gradient norm.
    pub fn record_gradient_norm(&mut self, norm: f32) {
        self.gradient_norms.push_back(norm);
        if self.gradient_norms.len() > self.window_size {
            self.gradient_norms.pop_front();
        }
    }

    /// Record a learning rate.
    pub fn record_learning_rate(&mut self, lr: f32) {
        self.learning_rates.push_back(lr);
        if self.learning_rates.len() > self.window_size {
            self.learning_rates.pop_front();
        }
    }

    /// Get average loss over the window.
    pub fn avg_loss(&self) -> f32 {
        if self.losses.is_empty() {
            0.0
        } else {
            self.losses.iter().sum::<f32>() / self.losses.len() as f32
        }
    }

    /// Get average accuracy over the window.
    pub fn avg_accuracy(&self) -> f32 {
        if self.accuracies.is_empty() {
            0.0
        } else {
            self.accuracies.iter().sum::<f32>() / self.accuracies.len() as f32
        }
    }

    /// Get average gradient norm.
    pub fn avg_gradient_norm(&self) -> f32 {
        if self.gradient_norms.is_empty() {
            0.0
        } else {
            self.gradient_norms.iter().sum::<f32>() / self.gradient_norms.len() as f32
        }
    }

    /// Get latest loss.
    pub fn latest_loss(&self) -> Option<f32> {
        self.losses.back().copied()
    }

    /// Get latest accuracy.
    pub fn latest_accuracy(&self) -> Option<f32> {
        self.accuracies.back().copied()
    }

    /// Get loss trend (true = increasing, false = decreasing).
    pub fn loss_trend(&self) -> Option<bool> {
        if self.losses.len() < 2 {
            return None;
        }
        let recent_avg =
            self.losses.iter().rev().take(5).sum::<f32>() / self.losses.len().min(5) as f32;
        let old_avg = self.losses.iter().take(5).sum::<f32>() / self.losses.len().min(5) as f32;
        Some(recent_avg > old_avg)
    }

    /// Export metrics as JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    /// Export metrics to CSV format.
    pub fn to_csv(&self) -> String {
        let mut csv = String::from("step,loss,accuracy,gradient_norm,learning_rate\n");

        let max_len = self
            .losses
            .len()
            .max(self.accuracies.len())
            .max(self.gradient_norms.len());

        for i in 0..max_len {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                i,
                self.losses
                    .get(i)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                self.accuracies
                    .get(i)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                self.gradient_norms
                    .get(i)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                self.learning_rates
                    .get(i)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ));
        }
        csv
    }

    /// Clear all metrics.
    pub fn clear(&mut self) {
        self.losses.clear();
        self.accuracies.clear();
        self.gradient_norms.clear();
        self.learning_rates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_tracking() {
        let mut metrics = Metrics::new(10);
        metrics.record_loss(1.5);
        metrics.record_loss(1.3);
        metrics.record_loss(1.1);

        assert_eq!(metrics.latest_loss(), Some(1.1));
        let avg = (1.5 + 1.3 + 1.1) / 3.0;
        assert!((metrics.avg_loss() - avg).abs() < 0.01);
    }

    #[test]
    fn test_csv_export() {
        let mut metrics = Metrics::new(10);
        metrics.record_loss(1.5);
        metrics.record_accuracy(0.8);

        let csv = metrics.to_csv();
        assert!(csv.contains("loss"));
        assert!(csv.contains("accuracy"));
    }
}
