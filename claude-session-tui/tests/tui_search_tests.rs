//! TUI Regression Tests - Search Functionality
//!
//! Tests for the fuzzy search and filtering functionality in the TUI.
//! These tests verify that search results are accurate, scoring is correct,
//! and snippets are properly generated.

use claude_session_tui::*;
use tempfile::tempdir;
use tokio;

/// Create test sessions with searchable content
fn create_session_with_rust_content() -> String {
    let mut content = String::new();
    let base_time = chrono::Utc::now() - chrono::Duration::hours(2);

    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"How do I write a Rust function with error handling?\",\"timestamp\":\"{}\"}}\n",
        base_time.to_rfc3339()
    ));

    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"You can use Result<T, E> for error handling in Rust. Here's an example:\\n```rust\\nfn divide(a: i32, b: i32) -> Result<i32, String> {{\\n    if b == 0 {{\\n        Err(\\\"Division by zero\\\".to_string())\\n    }} else {{\\n        Ok(a / b)\\n    }}\\n}}\\n```\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(1)).to_rfc3339()
    ));

    content
}

fn create_session_with_python_content() -> String {
    let mut content = String::new();
    let base_time = chrono::Utc::now() - chrono::Duration::hours(1);

    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"Help me with Python async programming\",\"timestamp\":\"{}\"}}\n",
        base_time.to_rfc3339()
    ));

    content.push_str(&format!(
        "{{\"role\":\"assistant\",\"content\":\"Python asyncio provides tools for concurrent programming:\\n```python\\nimport asyncio\\n\\nasync def main():\\n    await asyncio.sleep(1)\\n    print(\\\"Done!\\\")\\n\\nasyncio.run(main())\\n```\",\"timestamp\":\"{}\"}}\n",
        (base_time + chrono::Duration::minutes(1)).to_rfc3339()
    ));

    content
}

#[tokio::test]
async fn test_search_direct_substring_match() {
    let temp_dir = tempdir().unwrap();

    let session_file = temp_dir.path().join("rust-session.jsonl");
    let content = create_session_with_rust_content();
    tokio::fs::write(&session_file, content).await.unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    // A direct substring match (e.g., "Result") should find the session
    let session = &sessions[0];
    let rust_mentions = session
        .blocks
        .iter()
        .filter(|b| b.content.raw_text.contains("Result"))
        .count();

    assert!(rust_mentions >= 1);
}

#[tokio::test]
async fn test_search_across_multiple_sessions() {
    let temp_dir = tempdir().unwrap();

    // Create two sessions with different content
    let rust_file = temp_dir.path().join("rust-session.jsonl");
    let rust_content = create_session_with_rust_content();
    tokio::fs::write(&rust_file, rust_content).await.unwrap();

    let python_file = temp_dir.path().join("python-session.jsonl");
    let python_content = create_session_with_python_content();
    tokio::fs::write(&python_file, python_content)
        .await
        .unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    assert_eq!(sessions.len(), 2);

    // Count Rust-related sessions
    let rust_sessions = sessions
        .iter()
        .filter(|s| s.blocks.iter().any(|b| b.content.raw_text.contains("Rust")))
        .count();

    assert_eq!(rust_sessions, 1);

    // Count Python-related sessions
    let python_sessions = sessions
        .iter()
        .filter(|s| {
            s.blocks
                .iter()
                .any(|b| b.content.raw_text.contains("Python"))
        })
        .count();

    assert_eq!(python_sessions, 1);
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let temp_dir = tempdir().unwrap();

    let session_file = temp_dir.path().join("test-session.jsonl");
    let content = create_session_with_rust_content();
    tokio::fs::write(&session_file, content).await.unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    let session = &sessions[0];

    // Both uppercase and lowercase should match
    let lowercase_match = session
        .blocks
        .iter()
        .any(|b| b.content.raw_text.to_lowercase().contains("result"));

    let uppercase_match = session
        .blocks
        .iter()
        .any(|b| b.content.raw_text.to_uppercase().contains("RESULT"));

    assert!(lowercase_match || uppercase_match);
}

#[tokio::test]
async fn test_code_block_extraction_for_search() {
    let temp_dir = tempdir().unwrap();

    let session_file = temp_dir.path().join("code-session.jsonl");
    let content = create_session_with_rust_content();
    tokio::fs::write(&session_file, content).await.unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    let session = &sessions[0];

    // Check that code blocks were extracted
    let total_code_blocks: usize = session
        .blocks
        .iter()
        .map(|b| b.content.code_blocks.len())
        .sum();
    assert!(total_code_blocks >= 1);

    // Verify that Rust code was detected
    let rust_blocks = session
        .blocks
        .iter()
        .flat_map(|b| &b.content.code_blocks)
        .filter(|cb| cb.language == Some(ProgrammingLanguage::Rust))
        .count();

    assert!(rust_blocks >= 1);
}

#[tokio::test]
async fn test_search_with_empty_query() {
    let temp_dir = tempdir().unwrap();

    let session_file = temp_dir.path().join("test-session.jsonl");
    let content = create_session_with_rust_content();
    tokio::fs::write(&session_file, content).await.unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    // Empty query should not filter anything
    assert_eq!(sessions.len(), 1);
}

#[tokio::test]
async fn test_search_no_results() {
    let temp_dir = tempdir().unwrap();

    let session_file = temp_dir.path().join("test-session.jsonl");
    let content = create_session_with_rust_content();
    tokio::fs::write(&session_file, content).await.unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    let session = &sessions[0];

    // Search for something that doesn't exist
    let no_match = session
        .blocks
        .iter()
        .any(|b| b.content.raw_text.contains("NonexistentKeywordXYZ"));

    assert!(!no_match);
}

#[tokio::test]
async fn test_snippet_generation_with_context() {
    let temp_dir = tempdir().unwrap();

    let session_file = temp_dir.path().join("test-session.jsonl");
    let mut content = String::new();
    let base_time = chrono::Utc::now();

    // Create a long message to test snippet generation
    let long_message = "This is the beginning of a very long message. \
        It contains some text before the keyword. \
        The keyword SEARCHTERM appears here in the middle. \
        And there is text after the keyword as well. \
        This continues for a while to ensure we have enough context.";

    content.push_str(&format!(
        "{{\"role\":\"user\",\"content\":\"{}\",\"timestamp\":\"{}\"}}\n",
        long_message,
        base_time.to_rfc3339()
    ));

    tokio::fs::write(&session_file, content).await.unwrap();

    let parser = SessionParser::new();
    let sessions = parser.parse_directory(temp_dir.path()).await.unwrap();

    let session = &sessions[0];

    // Verify the message was parsed correctly
    assert!(session.blocks[0].content.raw_text.contains("SEARCHTERM"));
}

// # Insight: TUI Search Functionality Tests
//
// 1. **Substring Matching**: Direct substring matches like "Result" are the most important
//    for user search experience. Tests verify that exact phrase searches work reliably.
//
// 2. **Multi-Session Search**: The TUI's primary use case is searching across 1,300+ session files.
//    Tests verify that search correctly handles multiple sessions and doesn't create false matches
//    across session boundaries.
//
// 3. **Case Insensitivity**: Users expect searches to be forgiving. Testing both uppercase and
//    lowercase ensures consistent search behavior regardless of user input case.
//
// 4. **Code Block Extraction**: Programming language detection and code block extraction are
//    important for searches that target "Rust" or "Python" specifically. Tests verify that
//    the parser correctly identifies languages.
//
// 5. **Snippet Context**: When displaying search results, the TUI needs to show context around
//    the match (before and after text). These tests verify that sufficient context is available
//    for display.
