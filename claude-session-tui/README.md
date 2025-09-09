# Claude Session TUI - High-Performance JSONL Parser

A robust, type-safe Rust library for parsing Claude JSONL session files with advanced analytics, streaming support, and comprehensive error handling. Built as the foundation for the claude-session-tui project in the nabia-ai-stack.

## Features

### 🚀 High Performance
- **Streaming Parser**: Handles thousands of JSONL files efficiently
- **Parallel Processing**: Concurrent parsing with configurable limits
- **Memory Optimization**: Stream processing for large files
- **Performance Monitoring**: Built-in metrics and threshold monitoring

### 🛡️ Type Safety
- **Full Rust Type System**: Comprehensive type annotations throughout
- **Error Handling**: Structured error types with context
- **Data Models**: Rich type-safe models for all conversation data
- **Validation**: Runtime validation with compile-time guarantees

### 🔍 Advanced Analysis
- **Content Extraction**: Code blocks, file paths, URLs, commands
- **Language Detection**: Automatic programming language identification  
- **Conversation Insights**: Flow analysis, learning outcomes, productivity metrics
- **Search Engine Ready**: Tokenization and indexing for fast search

### 🎯 Integration Ready
- **Clean API**: Simple interfaces for other components
- **Caching System**: In-memory caching with LRU eviction
- **Export Formats**: JSON, CSV, Markdown output
- **Comprehensive Testing**: Integration tests with realistic data

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
claude-session-tui = { path = "./claude-session-tui" }
tokio = { version = "1.40", features = ["full"] }
```

### Basic Usage

```rust
use claude_session_tui::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the parser
    claude_session_tui::init()?;
    
    // Parse a single session file
    let session = parse_session_file("session.jsonl").await?;
    println!("Parsed {} blocks", session.blocks.len());
    
    // Parse multiple files in parallel
    let paths = vec!["session1.jsonl", "session2.jsonl"];
    let sessions = parse_session_files(paths).await?;
    
    // Parse entire directory
    let directory_sessions = parse_session_directory("./sessions").await?;
    
    Ok(())
}
```

### Advanced API Usage

```rust
use claude_session_tui::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create API with custom configuration
    let config = ApiConfig {
        enable_caching: true,
        max_cache_size: 1000,
        enable_background_analysis: true,
        performance_monitoring: true,
        auto_insights_extraction: true,
    };
    
    let api = ClaudeSessionApi::with_config(config);
    
    // Parse with full insights analysis
    let session = api.parse_session_file("complex_session.jsonl").await?;
    
    // Access detailed insights
    println!("Primary topics: {:?}", session.insights.primary_topics);
    println!("Conversation phases: {:?}", session.insights.conversation_flow.phases);
    println!("Learning outcomes: {:?}", session.insights.learning_outcomes);
    
    // Calculate aggregate statistics
    let stats = api.calculate_aggregate_stats(&[session.clone()]).await?;
    println!("Total words: {}", stats.total_words);
    println!("Programming languages: {:?}", stats.programming_languages);
    
    // Export to different formats
    let json_export = api.export_sessions(&[session.clone()], ExportFormat::Json).await?;
    let csv_export = api.export_sessions(&[session.clone()], ExportFormat::Csv).await?;
    let markdown_export = api.export_sessions(&[session], ExportFormat::Markdown).await?;
    
    Ok(())
}
```

### Search Functionality

```rust
use claude_session_tui::*;

#[tokio::main]
async fn main() -> Result<()> {
    let api = ClaudeSessionApi::new();
    
    // Load sessions
    let sessions = api.parse_directory("./sessions").await?.successful;
    
    // Create search interface
    let search = api.create_search_interface(sessions);
    
    // Build search query
    let query = SearchQuery {
        text_contains: vec!["Rust".to_string(), "async".to_string()],
        programming_languages: vec![ProgrammingLanguage::Rust],
        has_code_blocks: Some(true),
        complexity_range: Some((5.0, 10.0)),
        ..Default::default()
    };
    
    // Execute search
    let results = search.search(query).await?;
    
    println!("Found {} total matches", results.total_matches);
    println!("Search completed in {}ms", results.search_time_ms);
    
    // Process session matches
    for session_match in results.sessions {
        println!("Session: {} (relevance: {:.2})", 
            session_match.session.metadata.file_path,
            session_match.relevance_score
        );
        println!("Match reasons: {:?}", session_match.match_reasons);
    }
    
    // Process block matches
    for block_match in results.blocks {
        println!("Block content: {}", block_match.highlighted_content);
        println!("Context: {} surrounding blocks", block_match.context_blocks.len());
    }
    
    Ok(())
}
```

### Custom Parser Configuration

```rust
use claude_session_tui::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure error recovery
    let error_recovery = ErrorRecoverySettings {
        skip_malformed_lines: true,
        max_consecutive_errors: 50,
        continue_on_critical_errors: true,
        detailed_error_reporting: true,
    };
    
    // Configure content extraction
    let extraction_config = ExtractionConfig {
        extract_code_blocks: true,
        extract_file_paths: true,
        extract_commands: true,
        extract_urls: true,
        tokenize_content: true,
        analyze_sentiment: false,  // Disable expensive operations
        detect_programming_languages: true,
    };
    
    // Create custom parser
    let parser = SessionParser::with_config(
        16,    // max_concurrent_files
        2048,  // memory_limit_mb
        error_recovery,
        extraction_config
    );
    
    // Parse with custom configuration
    let session = parser.parse_file("large_session.jsonl").await?;
    
    Ok(())
}
```

## Data Models

The library provides comprehensive, type-safe data models:

### Core Types

- **`Session`**: Complete session with metadata, blocks, insights, and statistics
- **`Block`**: Individual conversation block with role, content, and metadata
- **`Role`**: Enum for User, Assistant, System, Tool
- **`BlockContent`**: Structured content with tokens, code blocks, links, mentions

### Analysis Types

- **`SessionInsights`**: AI-generated insights including topics, flow, learning outcomes
- **`ConversationFlow`**: Phase analysis with transitions and complexity evolution
- **`ProductivityMetrics`**: Task completion, problem resolution, code quality scores
- **`CollaborationPatterns`**: Interaction styles and knowledge transfer metrics

### Programming Language Support

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProgrammingLanguage {
    Rust, Python, JavaScript, TypeScript, Java, Go, Cpp, C,
    Swift, Kotlin, Ruby, PHP, Dart, Shell, SQL, HTML, CSS,
    Markdown, JSON, YAML, TOML, Unknown(String)
}
```

## Performance Benchmarks

Run comprehensive performance tests:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench parse_file_sizes
cargo bench content_complexity
cargo bench parallel_parsing
cargo bench error_recovery
```

Expected performance targets:
- **1000+ session files** parsed in <5 seconds
- **10,000+ conversation blocks** processed per second
- **Memory usage** <1GB for large datasets
- **Error recovery** handles 50%+ malformed data gracefully

## Testing

The library includes extensive integration tests with realistic Claude session data:

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture

# Test with realistic session data
cargo test test_realistic_session_parsing
```

## Error Handling

The library provides structured error handling with context:

```rust
use claude_session_tui::{Result, ClaudeSessionError};

match parse_session_file("session.jsonl").await {
    Ok(session) => println!("Parsed successfully: {} blocks", session.blocks.len()),
    Err(ClaudeSessionError::FileNotFound { path }) => {
        eprintln!("File not found: {}", path);
    },
    Err(ClaudeSessionError::JsonParsing { line, source }) => {
        eprintln!("JSON error at line {}: {}", line, source);
    },
    Err(ClaudeSessionError::PerformanceThreshold { operation, duration_ms, limit_ms }) => {
        eprintln!("Performance warning: {} took {}ms (limit: {}ms)", 
                 operation, duration_ms, limit_ms);
    },
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Architecture

### Parser Pipeline
1. **File Discovery**: Recursive directory scanning with filtering
2. **Streaming Parse**: Line-by-line JSONL processing with error recovery
3. **Content Extraction**: Code blocks, file paths, URLs, commands, tokens
4. **Language Detection**: Programming language identification from content
5. **Insights Analysis**: Conversation flow, learning patterns, productivity
6. **Caching**: LRU cache with configurable size and eviction

### Integration Points
- **Search Engine**: Tokenized content ready for full-text search
- **TUI Components**: Clean APIs for terminal user interfaces  
- **Export Systems**: Multiple format support (JSON, CSV, Markdown)
- **Analytics Pipeline**: Rich metrics and aggregate statistics

## Contributing

This library is part of the nabia-ai-stack claude-session-tui project. Key development principles:

1. **Type Safety First**: All operations must be type-safe
2. **Performance Focused**: Target 1000+ files in <5 seconds
3. **Error Recovery**: Graceful handling of malformed data
4. **Integration Ready**: Clean APIs for other components
5. **Comprehensive Testing**: Real session data in tests

## License

Part of the nabia-ai-stack project.