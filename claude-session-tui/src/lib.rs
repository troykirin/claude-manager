//! Claude Session TUI - High-performance JSONL parser for Claude session files
//!
//! This library provides robust, type-safe parsing of Claude JSONL session files
//! with high performance streaming support and comprehensive error handling.

pub mod api;
pub mod error;
pub mod extractor;
pub mod insights;
pub mod models;
pub mod parser;
#[cfg(feature = "tui")]
pub mod ui;

pub use api::*;
pub use error::*;
pub use extractor::*;
pub use insights::*;
pub use models::*;
pub use parser::*;
#[cfg(feature = "tui")]
pub use ui::*;

use tracing::info;

/// Initialize the Claude session parser with logging
pub fn init() -> anyhow::Result<()> {
    info!("Claude Session TUI parser initialized");
    Ok(())
}

/// Parse a single JSONL file and extract structured conversation data
pub async fn parse_session_file<P: AsRef<std::path::Path>>(path: P) -> Result<Session> {
    let parser = SessionParser::new();
    parser.parse_file(path).await
}

/// Parse multiple JSONL files in parallel
pub async fn parse_session_files<P: AsRef<std::path::Path> + Send + 'static>(
    paths: Vec<P>,
) -> Result<Vec<Session>> {
    let parser = SessionParser::new();
    parser.parse_files(paths).await
}

/// Parse a directory of JSONL files
pub async fn parse_session_directory<P: AsRef<std::path::Path>>(
    dir_path: P,
) -> Result<Vec<Session>> {
    let parser = SessionParser::new();
    parser.parse_directory(dir_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init() {
        assert!(init().is_ok());
    }
}
