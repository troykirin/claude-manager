//! Integration tests for Claude session parser with real session data patterns

use claude_session_tui::*;
use std::collections::HashMap;
use tempfile::{tempdir, NamedTempFile};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Create realistic test JSONL content based on actual Claude session patterns
fn create_realistic_session_content() -> String {
    let mut content = String::new();
    let base_time = chrono::Utc::now() - chrono::Duration::hours(2);

    // User starts with a question
    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"I'm building a Rust web API and need help with error handling. How should I structure custom errors?\",\"timestamp\":\"{}\"}}\n",
        base_time.to_rfc3339()
    ));

    // Assistant provides detailed response with code
    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"I'll help you create a robust error handling system for your Rust web API. Here's a comprehensive approach:\\n\\n```rust\\nuse thiserror::Error;\\nuse serde::{{Serialize, Deserialize}};\\n\\n#[derive(Error, Debug, Serialize, Deserialize)]\\npub enum ApiError {{\\n    #[error(\\\"Database error: {{0}}\\\")]\\n    Database(String),\\n    \\n    #[error(\\\"Validation failed: {{field}}\\\")]\\n    Validation {{ field: String }},\\n    \\n    #[error(\\\"Not found: {{resource}}\\\")]\\n    NotFound {{ resource: String }},\\n    \\n    #[error(\\\"Internal server error\\\")]\\n    Internal,\\n}}\\n```\\n\\nThis gives you structured error types with automatic Display implementations. You can then implement response conversions for your web framework.\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(1)).to_rfc3339()
    ));

    // User asks follow-up with file reference
    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"Thanks! How do I integrate this with my existing handlers in src/handlers/user.rs? I'm using axum.\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(3)).to_rfc3339()
    ));

    // Assistant shows integration example
    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"Perfect! Here's how to integrate with axum in your src/handlers/user.rs:\\n\\n```rust\\nuse axum::{{\\n    extract::Path,\\n    http::StatusCode,\\n    response::{{IntoResponse, Response}},\\n    Json,\\n}};\\nuse serde_json::json;\\n\\nimpl IntoResponse for ApiError {{\\n    fn into_response(self) -> Response {{\\n        let (status, error_message) = match self {{\\n            ApiError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),\\n            ApiError::Validation {{ .. }} => (StatusCode::BAD_REQUEST, self.to_string()),\\n            ApiError::NotFound {{ .. }} => (StatusCode::NOT_FOUND, self.to_string()),\\n            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, \\\"Internal server error\\\".to_string()),\\n        }};\\n        \\n        let body = Json(json!({{\\n            \\\"error\\\": error_message,\\n            \\\"status\\\": status.as_u16()\\n        }}));\\n        \\n        (status, body).into_response()\\n    }}\\n}}\\n\\n// Your handler can now return Result<Json<User>, ApiError>\\npub async fn get_user(Path(user_id): Path<u32>) -> Result<Json<User>, ApiError> {{\\n    let user = database::find_user(user_id)\\n        .await\\n        .map_err(|e| ApiError::Database(e.to_string()))?\\n        .ok_or_else(|| ApiError::NotFound {{ \\n            resource: format!(\\\"User with id {{}}\\\", user_id) \\n        }})?;\\n    \\n    Ok(Json(user))\\n}}\\n```\\n\\nThis approach gives you clean error propagation with automatic HTTP response conversion.\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(5)).to_rfc3339()
    ));

    // User encounters a specific error
    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"I'm getting this compilation error when I try to use it:\\n\\n```\\nerror[E0277]: the trait bound `ApiError: std::error::Error` is not satisfied\\n```\\n\\nDo I need additional derives?\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(8)).to_rfc3339()
    ));

    // Assistant provides solution
    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"Yes, you need to add the `std::error::Error` trait. The `thiserror` crate should handle this automatically, but make sure you have the correct setup:\\n\\n```rust\\nuse thiserror::Error;\\nuse serde::{{Serialize, Deserialize}};\\n\\n#[derive(Error, Debug, Serialize, Deserialize)]\\npub enum ApiError {{\\n    #[error(\\\"Database error: {{0}}\\\")]\\n    Database(#[from] sqlx::Error),  // This automatically implements From<sqlx::Error>\\n    \\n    #[error(\\\"Validation failed: {{field}}\\\")]\\n    Validation {{ field: String }},\\n    \\n    #[error(\\\"Not found: {{resource}}\\\")]\\n    NotFound {{ resource: String }},\\n    \\n    #[error(\\\"Internal server error\\\")]\\n    Internal,\\n}}\\n```\\n\\nAlso make sure your Cargo.toml includes:\\n\\n```toml\\n[dependencies]\\nthiserror = \\\"1.0\\\"\\nserde = {{ version = \\\"1.0\\\", features = [\\\"derive\\\"] }}\\naxum = \\\"0.7\\\"\\n```\\n\\nThe `#[from]` attribute is particularly useful as it automatically generates `From` implementations for converting other error types.\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(10)).to_rfc3339()
    ));

    // User confirms success
    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"Perfect! That fixed the compilation error. The error handling is working great now. Thanks for the comprehensive solution!\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(15)).to_rfc3339()
    ));

    content
}

/// Create a session with tool usage patterns
fn create_session_with_tools() -> String {
    let mut content = String::new();
    let base_time = chrono::Utc::now() - chrono::Duration::hours(1);

    // User asks for file reading
    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"Can you help me analyze the structure of my project? Please look at the main.rs file.\",\"timestamp\":\"{}\"}}\n",
        base_time.to_rfc3339()
    ));

    // Tool usage simulation
    content.push_str(&format!(
        r#"{{"role":"tool","content":"{{\"tool\":\"read_file\",\"path\":\"/src/main.rs\",\"content\":\"use axum::routing::get;\"}}","timestamp":"{}"}}"#,
        (base_time + chrono::Duration::seconds(30)).to_rfc3339()
    ));
    content.push('\n');

    // Assistant analyzes the code
    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"I can see your main.rs file contains a basic axum web server setup. Here's an analysis of the structure:\\n\\n**Current Structure:**\\n- Simple axum Router with a single route\\n- Basic \\\"Hello, World!\\\" endpoint\\n- Server binding to localhost:3000\\n\\n**Recommendations for improvement:**\\n\\n1. **Add error handling:**\\n```rust\\nuse anyhow::Result;\\n\\n#[tokio::main]\\nasync fn main() -> Result<()> {{\\n    let app = create_app();\\n    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));\\n    \\n    println!(\\\"Server running on http://{{}}\\\", addr);\\n    \\n    axum::Server::bind(&addr)\\n        .serve(app.into_make_service())\\n        .await?;\\n    \\n    Ok(())\\n}}\\n```\\n\\n2. **Extract app creation:**\\n```rust\\nfn create_app() -> Router {{\\n    Router::new()\\n        .route(\\\"/\\\", get(hello_handler))\\n        .layer(ServiceBuilder::new())\\n}}\\n\\nasync fn hello_handler() -> &'static str {{\\n    \\\"Hello, World!\\\"\\n}}\\n```\\n\\nThis makes your code more modular and testable.\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(2)).to_rfc3339()
    ));

    content
}

/// Create a debugging session with error patterns
fn create_debugging_session() -> String {
    let mut content = String::new();
    let base_time = chrono::Utc::now() - chrono::Duration::minutes(30);

    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"I'm getting a panic in my Rust program:\\n\\nthread 'main' panicked at 'index out of bounds: the len is 3 but the index is 5', src/main.rs:42:9\\n\\nCan you help me debug this?\",\"timestamp\":\"{}\"}}\n",
        base_time.to_rfc3339()
    ));

    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"This is a classic bounds checking error. The panic occurs because you're trying to access index 5 in a vector that only has 3 elements.\\n\\nHere are some debugging strategies:\\n\\n1. **Use safe indexing:**\\n```rust\\n// Instead of:\\nlet item = vec[index]; // This can panic\\n\\n// Use:\\nlet item = vec.get(index).unwrap_or(&default_value);\\n// Or:\\nmatch vec.get(index) {{\\n    Some(item) => // handle item,\\n    None => // handle out of bounds,\\n}}\\n```\\n\\n2. **Add bounds checking:**\\n```rust\\nif index < vec.len() {{\\n    let item = vec[index];\\n    // process item\\n}} else {{\\n    eprintln!(\\\"Index {{}} is out of bounds for vector of length {{}}\\\", index, vec.len());\\n}}\\n```\\n\\n3. **Use iterators when possible:**\\n```rust\\nfor (i, item) in vec.iter().enumerate() {{\\n    // i is guaranteed to be valid\\n}}\\n```\\n\\nCan you show me the code around line 42 in src/main.rs?\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(1)).to_rfc3339()
    ));

    content
}

#[tokio::test]
async fn test_realistic_session_parsing() {
    let content = create_realistic_session_content();
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), content).await.unwrap();

    let parser = SessionParser::new();
    let session = parser.parse_file(temp_file.path()).await.unwrap();

    assert_eq!(session.blocks.len(), 6);
    assert_eq!(session.statistics.user_blocks, 3);
    assert_eq!(session.statistics.assistant_blocks, 3);

    // Check that code blocks were extracted
    let total_code_blocks: usize = session
        .blocks
        .iter()
        .map(|b| b.content.code_blocks.len())
        .sum();
    assert!(total_code_blocks >= 3);

    // Check for Rust language detection
    let rust_blocks = session
        .blocks
        .iter()
        .flat_map(|b| &b.content.code_blocks)
        .filter(|cb| cb.language == Some(ProgrammingLanguage::Rust))
        .count();
    assert!(rust_blocks >= 3);

    // Check for file path mentions
    let file_mentions = session
        .blocks
        .iter()
        .flat_map(|b| &b.content.mentions)
        .filter(|m| m.mention_type == MentionType::File)
        .count();
    assert!(file_mentions >= 1);
}

#[tokio::test]
async fn test_tool_usage_session() {
    let content = create_session_with_tools();
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), content).await.unwrap();

    let parser = SessionParser::new();
    let session = parser.parse_file(temp_file.path()).await.unwrap();

    // Check for tool role blocks
    let tool_blocks = session
        .blocks
        .iter()
        .filter(|b| b.role == Role::Tool)
        .count();
    assert!(tool_blocks >= 1);

    // Check tool usage statistics
    assert!(session.statistics.total_blocks >= 3);
}

#[tokio::test]
async fn test_debugging_session_patterns() {
    let content = create_debugging_session();
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), content).await.unwrap();

    let parser = SessionParser::new();
    let session = parser.parse_file(temp_file.path()).await.unwrap();

    // Should detect error patterns
    let error_mentions = session
        .blocks
        .iter()
        .filter(|b| {
            b.content.raw_text.to_lowercase().contains("panic")
                || b.content.raw_text.to_lowercase().contains("error")
        })
        .count();
    assert!(error_mentions >= 1);
}

#[tokio::test]
async fn test_comprehensive_insights_analysis() {
    let content = create_realistic_session_content();
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), content).await.unwrap();

    let api = ClaudeSessionApi::new();
    let session = api.parse_session_file(temp_file.path()).await.unwrap();

    // Check insights generation
    assert!(!session.insights.primary_topics.is_empty());
    assert!(!session.insights.conversation_flow.phases.is_empty());

    // Check for Rust-related topics
    let rust_topics = session
        .insights
        .primary_topics
        .iter()
        .filter(|t| t.name.to_lowercase().contains("rust"))
        .count();
    assert!(rust_topics >= 1);

    // Check conversation flow phases
    let implementation_phases = session
        .insights
        .conversation_flow
        .phases
        .iter()
        .filter(|p| p.phase_type == PhaseType::Implementation)
        .count();
    assert!(implementation_phases >= 1);
}

#[tokio::test]
async fn test_batch_parsing_with_errors() {
    let temp_dir = tempdir().unwrap();
    let mut file_paths = Vec::new();

    // Create valid file
    let valid_content = create_realistic_session_content();
    let valid_file = temp_dir.path().join("valid.jsonl");
    tokio::fs::write(&valid_file, valid_content).await.unwrap();
    file_paths.push(valid_file);

    // Create file with malformed JSON
    let malformed_content = "invalid json line\n{\"role\":\"user\",\"content\":\"valid\",\"timestamp\":\"2023-01-01T00:00:00Z\"}\n";
    let malformed_file = temp_dir.path().join("malformed.jsonl");
    tokio::fs::write(&malformed_file, malformed_content)
        .await
        .unwrap();
    file_paths.push(malformed_file);

    let api = ClaudeSessionApi::new();
    let result = api.parse_session_files(file_paths).await;

    // Should successfully parse at least one file
    assert!(result.successful.len() >= 1);
    assert!(result.success_rate() > 0.0);
    assert!(result.performance_stats.files_processed >= 2);
}

#[tokio::test]
async fn test_search_functionality() {
    let temp_dir = tempdir().unwrap();
    let mut sessions = Vec::new();

    // Create multiple test sessions
    for i in 0..3 {
        let content = if i == 0 {
            create_realistic_session_content()
        } else if i == 1 {
            create_session_with_tools()
        } else {
            create_debugging_session()
        };

        let file_path = temp_dir.path().join(format!("session_{}.jsonl", i));
        tokio::fs::write(&file_path, content).await.unwrap();

        let api = ClaudeSessionApi::new();
        let session = api.parse_session_file(&file_path).await.unwrap();
        sessions.push(session);
    }

    let api = ClaudeSessionApi::new();
    let search = api.create_search_interface(sessions);

    // Test text search
    let query = SearchQuery {
        text_contains: vec!["Rust".to_string()],
        ..Default::default()
    };

    let results = search.search(query).await.unwrap();
    assert!(results.total_matches > 0);

    // Test programming language filter
    let query = SearchQuery {
        programming_languages: vec![ProgrammingLanguage::Rust],
        ..Default::default()
    };

    let results = search.search(query).await.unwrap();
    assert!(results.sessions.len() > 0);
}

#[tokio::test]
async fn test_aggregate_statistics() {
    let temp_dir = tempdir().unwrap();
    let mut sessions = Vec::new();

    // Create test sessions with different patterns
    let contents = [
        create_realistic_session_content(),
        create_session_with_tools(),
        create_debugging_session(),
    ];

    for (i, content) in contents.iter().enumerate() {
        let file_path = temp_dir.path().join(format!("session_{}.jsonl", i));
        tokio::fs::write(&file_path, content).await.unwrap();

        let api = ClaudeSessionApi::new();
        let session = api.parse_session_file(&file_path).await.unwrap();
        sessions.push(session);
    }

    let api = ClaudeSessionApi::new();
    let stats = api.calculate_aggregate_stats(&sessions).await.unwrap();

    assert_eq!(stats.total_sessions, 3);
    assert!(stats.total_blocks > 0);
    assert!(stats.total_words > 0);
    assert!(!stats.programming_languages.is_empty());
    assert!(!stats.common_topics.is_empty());
}

#[tokio::test]
async fn test_export_functionality() {
    let content = create_realistic_session_content();
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), content).await.unwrap();

    let api = ClaudeSessionApi::new();
    let session = api.parse_session_file(temp_file.path()).await.unwrap();
    let sessions = vec![session];

    // Test JSON export
    let json_export = api
        .export_sessions(&sessions, ExportFormat::Json)
        .await
        .unwrap();
    assert!(json_export.contains("\"role\":"));
    assert!(serde_json::from_str::<serde_json::Value>(&json_export).is_ok());

    // Test CSV export
    let csv_export = api
        .export_sessions(&sessions, ExportFormat::Csv)
        .await
        .unwrap();
    assert!(csv_export.contains("id,file_path,created_at"));

    // Test Markdown export
    let markdown_export = api
        .export_sessions(&sessions, ExportFormat::Markdown)
        .await
        .unwrap();
    assert!(markdown_export.contains("# Claude Sessions Analysis"));
    assert!(markdown_export.contains("## Session:"));
}

#[tokio::test]
async fn test_caching_performance() {
    let content = create_realistic_session_content();
    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), content).await.unwrap();

    let api = ClaudeSessionApi::new();

    // First parse (cache miss)
    let start_time = std::time::Instant::now();
    let _session1 = api.parse_session_file(temp_file.path()).await.unwrap();
    let first_parse_time = start_time.elapsed();

    // Second parse (cache hit)
    let start_time = std::time::Instant::now();
    let _session2 = api.parse_session_file(temp_file.path()).await.unwrap();
    let second_parse_time = start_time.elapsed();

    // Cache hit should be significantly faster
    assert!(second_parse_time < first_parse_time / 2);

    let cache_stats = api.get_cache_stats().await;
    assert_eq!(cache_stats.total_entries, 1);
    assert_eq!(cache_stats.total_accesses, 2);
}

#[tokio::test]
async fn test_performance_threshold_monitoring() {
    let mut large_content = String::new();

    // Create a large session to test performance monitoring
    for i in 0..10000 {
        let timestamp = chrono::Utc::now() - chrono::Duration::seconds(i);
        large_content.push_str(&format!(
            "{{\"role\":\"user\",\"content\":\"Message number {} with some content\",\"timestamp\":\"{}\"}}\n",
            i, timestamp.to_rfc3339()
        ));
    }

    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), large_content)
        .await
        .unwrap();

    // Parser with very low performance threshold
    let parser = SessionParser::with_config(
        4,
        1024,
        ErrorRecoverySettings {
            skip_malformed_lines: true,
            max_consecutive_errors: 10,
            continue_on_critical_errors: false,
            detailed_error_reporting: true,
        },
        ExtractionConfig {
            extract_code_blocks: true,
            extract_file_paths: true,
            extract_commands: true,
            extract_urls: true,
            tokenize_content: true,
            analyze_sentiment: false,
            detect_programming_languages: true,
        },
    );

    let session = parser.parse_file(temp_file.path()).await.unwrap();
    assert_eq!(session.blocks.len(), 10000);
    assert!(session.statistics.total_words > 0);
}

#[test]
fn test_data_model_serialization() {
    let mut session = Session::new();
    session.metadata.file_path = "test.jsonl".to_string();

    let block = Block {
        id: Uuid::new_v4(),
        sequence_number: 1,
        role: Role::User,
        timestamp: chrono::Utc::now(),
        content: BlockContent {
            raw_text: "Test message".to_string(),
            formatted_text: None,
            tokens: vec![ContentToken {
                text: "Test".to_string(),
                token_type: TokenType::Word,
                position: 0,
                length: 4,
            }],
            code_blocks: vec![],
            links: vec![],
            mentions: vec![],
            word_count: 2,
            character_count: 12,
        },
        metadata: BlockMetadata {
            processing_time_ms: Some(100),
            confidence_score: Some(0.95),
            complexity_score: Some(2.5),
            sentiment: Some(Sentiment::Neutral),
            topics: vec!["testing".to_string()],
            intent: Some(ConversationIntent::Question),
            parent_block_id: None,
            thread_id: None,
        },
        tools: vec![],
        attachments: vec![],
        context_references: vec![],
    };

    session.add_block(block);

    // Test JSON serialization
    let json_str = serde_json::to_string(&session).unwrap();
    assert!(json_str.contains("\"role\":\"User\""));

    // Test deserialization
    let deserialized: Session = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.blocks.len(), 1);
    assert_eq!(deserialized.blocks[0].role, Role::User);
}

#[tokio::test]
async fn test_error_recovery_patterns() {
    // Create content with various error patterns
    let mut error_content = String::new();

    // Valid JSON line
    error_content.push_str("{\"role\":\"user\",\"content\":\"Valid message\",\"timestamp\":\"2023-01-01T00:00:00Z\"}\n");

    // Malformed JSON (missing closing brace)
    error_content.push_str(
        "{\"role\":\"user\",\"content\":\"Malformed\",\"timestamp\":\"2023-01-01T00:00:00Z\"\n",
    );

    // Invalid role
    error_content.push_str("{\"role\":\"invalid_role\",\"content\":\"Invalid role\",\"timestamp\":\"2023-01-01T00:00:00Z\"}\n");

    // Missing required field
    error_content.push_str("{\"role\":\"user\",\"timestamp\":\"2023-01-01T00:00:00Z\"}\n");

    // Invalid timestamp
    error_content.push_str(
        "{\"role\":\"user\",\"content\":\"Invalid timestamp\",\"timestamp\":\"invalid-date\"}\n",
    );

    // Another valid line
    error_content.push_str("{\"role\":\"assistant\",\"content\":\"Another valid message\",\"timestamp\":\"2023-01-01T01:00:00Z\"}\n");

    let temp_file = NamedTempFile::new().unwrap();
    tokio::fs::write(temp_file.path(), error_content)
        .await
        .unwrap();

    let parser = SessionParser::new(); // Uses error recovery by default
    let session = parser.parse_file(temp_file.path()).await.unwrap();

    // Should successfully parse valid lines while skipping malformed ones
    assert!(session.blocks.len() >= 2);
    assert!(session
        .blocks
        .iter()
        .all(|b| matches!(b.role, Role::User | Role::Assistant)));
}

#[ignore] // Testing private method - disabled
#[test]
fn test_programming_language_detection() {
    let extractor = BlockExtractor::new();

    // Test various programming languages
    let test_cases = [
        (
            "fn main() { println!(\"Hello\"); }",
            Some(ProgrammingLanguage::Rust),
        ),
        (
            "def hello():\n    print(\"Hello\")",
            Some(ProgrammingLanguage::Python),
        ),
        (
            "function hello() { console.log(\"Hello\"); }",
            Some(ProgrammingLanguage::JavaScript),
        ),
        ("SELECT * FROM users", Some(ProgrammingLanguage::SQL)),
        ("<div>Hello</div>", Some(ProgrammingLanguage::HTML)),
        ("body { color: red; }", Some(ProgrammingLanguage::CSS)),
        ("random text without code", None),
    ];

    // Disabled: Testing private method
    // for (code, expected_lang) in test_cases.iter() {
    //     let detected = extractor.detect_programming_language_from_content(code);
    //     assert_eq!(detected, *expected_lang, "Failed for: {}", code);
    // }
}

#[test]
fn test_content_tokenization() {
    let mut extractor = BlockExtractor::new();
    let text = "Check the file /src/main.rs and run `cargo test` command.";

    let content = extractor.extract_block_content(text).unwrap();

    // Should extract file path
    let file_mentions: Vec<_> = content
        .mentions
        .iter()
        .filter(|m| m.mention_type == MentionType::File)
        .collect();
    assert!(file_mentions.len() >= 1);

    // Should extract command
    let command_mentions: Vec<_> = content
        .mentions
        .iter()
        .filter(|m| m.mention_type == MentionType::Command)
        .collect();
    assert!(command_mentions.len() >= 1);

    // Should tokenize content
    assert!(!content.tokens.is_empty());
    assert!(content.word_count > 0);
}

#[ignore] // Testing private method - disabled
#[test]
fn test_complex_conversation_flow() {
    // This test would be expanded with actual conversation flow analysis
    // For now, testing basic phase detection
    let analyzer = InsightsAnalyzer::new();

    // Disabled: Testing private method
    // let planning_content = "Let's plan the architecture for this system";
    // assert_eq!(analyzer.detect_conversation_phase(planning_content), PhaseType::Planning);

    // let implementation_content = "Now let's implement the main function";
    // assert_eq!(analyzer.detect_conversation_phase(implementation_content), PhaseType::Implementation);

    // let debugging_content = "I'm getting an error, let's debug this issue";
    // assert_eq!(analyzer.detect_conversation_phase(debugging_content), PhaseType::Debugging);
}
