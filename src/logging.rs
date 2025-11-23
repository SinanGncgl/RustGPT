//! Logging utilities for the LLM framework.
//!
//! Provides structured logging with configurable levels and outputs.

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize the logging system with the specified filter level.
///
/// # Arguments
/// * `filter_level` - The logging filter level (e.g., "debug", "info", "warn", "error")
///
/// # Example
/// ```ignore
/// init_logging("debug").expect("Failed to initialize logging");
/// ```
pub fn init_logging(filter_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    Ok(())
}

/// Initialize JSON logging for structured output.
pub fn init_json_logging(filter_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        let result = init_logging("debug");
        assert!(result.is_ok());
    }
}
