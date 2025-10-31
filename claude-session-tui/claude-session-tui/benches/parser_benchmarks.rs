//! Performance benchmarks for Claude session parser

use claude_session_tui::{
    ClaudeSessionApi, ErrorRecoverySettings, ExtractionConfig, SessionParser,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};
use tokio::io::AsyncWriteExt;

/// Generate synthetic JSONL content for benchmarking
fn generate_test_jsonl(lines: usize, complexity: ContentComplexity) -> String {
    let mut content = String::new();

    for i in 0..lines {
        let (role, text) = match complexity {
            ContentComplexity::Simple => generate_simple_message(i),
            ContentComplexity::Medium => generate_medium_message(i),
            ContentComplexity::Complex => generate_complex_message(i),
        };

        let timestamp = chrono::Utc::now() - chrono::Duration::seconds((lines - i) as i64 * 60);
        let line = format!(
            "{{\"role\":\"{}\",\"content\":\"{}\",\"timestamp\":\"{}\"}}\n",
            role,
            text.replace('"', "\\\""),
            timestamp.to_rfc3339()
        );
        content.push_str(&line);
    }

    content
}

#[derive(Clone)]
enum ContentComplexity {
    Simple,
    Medium,
    Complex,
}

fn generate_simple_message(index: usize) -> (&'static str, String) {
    if index % 2 == 0 {
        ("user", format!("This is user message number {}", index))
    } else {
        (
            "assistant",
            format!("This is assistant response number {}", index),
        )
    }
}

fn generate_medium_message(index: usize) -> (&'static str, String) {
    if index % 2 == 0 {
        (
            "user",
            format!(
                "How do I implement feature {} in Rust? I'm having trouble with the async aspects.",
                index
            ),
        )
    } else {
        ("assistant", format!("Here's how you can implement feature {}:\\n```rust\\nfn example_{}() {{\\n    println!(\\\"Hello, feature {}!\\\");\\n}}\\n```\\nThis should solve your async issues.", index, index, index))
    }
}

fn generate_complex_message(index: usize) -> (&'static str, String) {
    if index % 2 == 0 {
        ("user", format!(
            "I'm working on a complex system with multiple components. Here's my current architecture:\\n\\n```rust\\nstruct Component{} {{\\n    data: Vec<String>,\\n    handler: Box<dyn Handler>,\\n}}\\n```\\n\\nI need to implement error handling, logging, and performance optimization. The system needs to handle 1000+ requests per second while maintaining data consistency. What's the best approach for this scenario? Also, I'm seeing some memory leaks in the /src/main.rs file around line 150.", 
            index
        ))
    } else {
        ("assistant", format!(
            "For a high-performance system like this, I recommend several approaches:\\n\\n1. **Error Handling Strategy:**\\n```rust\\nuse anyhow::{{Result, Context}};\\nuse thiserror::Error;\\n\\n#[derive(Error, Debug)]\\nenum SystemError {{\\n    #[error(\\\"Component {} failed: {{0}}\\\")]\\n    ComponentFailure(String),\\n    #[error(\\\"Performance threshold exceeded\\\")]\\n    PerformanceError,\\n}}\\n```\\n\\n2. **Performance Optimization:**\\nConsider using:\\n- `tokio::sync::RwLock` for concurrent access\\n- Connection pooling for database operations\\n- Metrics collection with `prometheus`\\n\\n3. **Memory Leak Fix:**\\nThe issue in /src/main.rs is likely due to circular references. Use `Weak` pointers or implement proper drop logic.", 
            index
        ))
    }
}

/// Benchmark parsing performance with different file sizes
fn bench_parse_file_sizes(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("parse_file_sizes");

    for size in [100, 500, 1000, 5000].iter() {
        let content = generate_test_jsonl(*size, ContentComplexity::Medium);
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("lines", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let mut temp_file = tokio::fs::File::create("bench_temp.jsonl").await.unwrap();
                temp_file
                    .write_all(black_box(content.as_bytes()))
                    .await
                    .unwrap();
                drop(temp_file);

                let parser = SessionParser::new();
                let session = parser.parse_file("bench_temp.jsonl").await.unwrap();
                tokio::fs::remove_file("bench_temp.jsonl").await.unwrap();
                black_box(session)
            });
        });
    }

    group.finish();
}

/// Benchmark parsing with different content complexity levels
fn bench_content_complexity(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("content_complexity");

    let complexities = [
        ("simple", ContentComplexity::Simple),
        ("medium", ContentComplexity::Medium),
        ("complex", ContentComplexity::Complex),
    ];

    for (name, complexity) in complexities.iter() {
        let content = generate_test_jsonl(1000, complexity.clone());
        group.throughput(Throughput::Bytes(content.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("complexity", name),
            &content,
            |b, content| {
                b.to_async(&rt).iter(|| async {
                    let mut temp_file = tokio::fs::File::create("bench_temp.jsonl").await.unwrap();
                    temp_file
                        .write_all(black_box(content.as_bytes()))
                        .await
                        .unwrap();
                    drop(temp_file);

                    let parser = SessionParser::new();
                    let session = parser.parse_file("bench_temp.jsonl").await.unwrap();
                    tokio::fs::remove_file("bench_temp.jsonl").await.unwrap();
                    black_box(session)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel parsing performance
fn bench_parallel_parsing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("parallel_parsing");

    for file_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("files", file_count),
            file_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let temp_dir = tempdir().unwrap();
                    let mut file_paths = Vec::new();

                    // Create temporary files
                    for i in 0..count {
                        let content = generate_test_jsonl(500, ContentComplexity::Medium);
                        let file_path = temp_dir.path().join(format!("session_{}.jsonl", i));
                        tokio::fs::write(&file_path, content).await.unwrap();
                        file_paths.push(file_path);
                    }

                    let parser = SessionParser::new();
                    let sessions = parser.parse_files(black_box(file_paths)).await.unwrap();
                    black_box(sessions)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark error recovery performance
fn bench_error_recovery(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("error_recovery");

    for error_rate in [0, 10, 25, 50].iter() {
        let content = generate_malformed_jsonl(1000, *error_rate);
        group.bench_with_input(
            BenchmarkId::new("error_rate_pct", error_rate),
            &content,
            |b, content| {
                b.to_async(&rt).iter(|| async {
                    let mut temp_file = tokio::fs::File::create("bench_temp.jsonl").await.unwrap();
                    temp_file
                        .write_all(black_box(content.as_bytes()))
                        .await
                        .unwrap();
                    drop(temp_file);

                    let parser = SessionParser::new();
                    let session = parser.parse_file("bench_temp.jsonl").await.unwrap();
                    tokio::fs::remove_file("bench_temp.jsonl").await.unwrap();
                    black_box(session)
                });
            },
        );
    }

    group.finish();
}

fn generate_malformed_jsonl(lines: usize, error_rate_percent: usize) -> String {
    let mut content = String::new();

    for i in 0..lines {
        if i % 100 < error_rate_percent {
            // Generate malformed JSON
            content.push_str(&format!(
                "{{\"role\":\"user\",\"content\":\"malformed {} unclosed\n",
                i
            ));
        } else {
            // Generate valid JSON
            let timestamp = chrono::Utc::now() - chrono::Duration::seconds((lines - i) as i64 * 60);
            let line = format!(
                "{{\"role\":\"user\",\"content\":\"Valid message {}\",\"timestamp\":\"{}\"}}\n",
                i,
                timestamp.to_rfc3339()
            );
            content.push_str(&line);
        }
    }

    content
}

criterion_group!(
    benches,
    bench_parse_file_sizes,
    bench_content_complexity,
    bench_parallel_parsing,
    bench_error_recovery
);
criterion_main!(benches);
