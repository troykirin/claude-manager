# Claude Session TUI - Technical Architecture

## 📐 System Architecture

```
┌─────────────────────────────────────────────────────────┐
│  ~/.claude/projects/                                    │
│  (1,300+ JSONL session files)                           │
└────────────────┬────────────────────────────────────────┘
                 │
        ┌────────▼─────────┐
        │ SessionParser    │
        │ (Rust async)     │
        │                  │
        │ - WalkDir scan   │
        │ - Parallel parse │
        │ - Error recovery │
        └────────┬─────────┘
                 │
        ┌────────▼────────────────┐
        │ Session Model (Vec)      │
        │ - metadata              │
        │ - blocks (messages)     │
        │ - insights              │
        └────────┬────────────────┘
                 │
        ┌────────▼──────────────────┐
        │ App State                 │
        │ - sessions               │
        │ - filtered_sessions      │
        │ - search_matches         │
        │ - selected index         │
        └────────┬──────────────────┘
                 │
        ┌────────▼──────────────────┐
        │ Ratatui TUI              │
        │ - Render loop            │
        │ - Event handling         │
        │ - Key processing         │
        └────────┬──────────────────┘
                 │
        ┌────────▼──────────────────┐
        │ Terminal Output           │
        │ (Crossterm)              │
        └──────────────────────────┘
```

## 🔄 Data Flow

### Session Loading

```
main()
  → App::new()
    → load_sessions(~/.claude/projects)
      → parse_session_directory()
        → SessionParser::parse_directory()
          → WalkDir scan for *.jsonl
          → parse_files() with semaphore
            → For each file:
              → parse_file()
                → Line-by-line JSONL parsing
                → Convert to Block objects
                → Analyze session
            → Return Vec<Session>
      → Sort by created_at
      → Store in filtered_sessions
```

### Search Flow

```
Key('/') → is_searching = true

Key(char) → search_query += char

Key(Enter) → search_sessions()
  → expand_search_intent(query)
    → Extract pattern keywords
    → Generate domain keywords
    → Generate semantic variations
  → For each session:
    → For each block:
      → Fuzzy match with all expanded queries
      → Substring matching with priority
      → Score calculation
      → Create snippet
    → Store SearchMatch
  → Sort by score (highest first)
  → Update filtered_sessions & search_matches
  → Reset selection
```

### Rendering

```
render()
  → Layout splits (search bar, content, footer)
  → Left pane: Sessions list
    → Show filtered sessions
    → Highlight selected
  → Right pane: Details (mode-dependent)
    → Summary: Status, first match
    → FullJson: Full JSON display
    → SnippetBrowser: Match with scrolling
```

## 🏗️ Key Components

### 1. SessionParser (`parser.rs`)

**Purpose**: Async JSONL file parsing with error recovery

```rust
pub struct SessionParser {
    max_concurrent_files: usize,      // Default: CPU count
    memory_limit_mb: usize,            // Default: 1GB
    performance_threshold_ms: u64,    // Default: 5s
    error_recovery: ErrorRecoverySettings,
    extraction_config: ExtractionConfig,
}
```

**Key Methods**:
- `parse_file()` - Single file async parsing
- `parse_files()` - Parallel file parsing with semaphore
- `parse_directory()` - Recursive directory scanning with WalkDir
- `parse_jsonl_line()` - Individual JSONL line parsing
- `convert_to_block()` - Raw message → Block conversion

**Error Handling**:
- Skip malformed lines (configurable)
- Continue on consecutive errors (up to threshold)
- Memory limit enforcement
- Performance threshold monitoring

### 2. App State (`ui/app.rs`)

**Purpose**: TUI application state and logic

```rust
pub struct App {
    sessions: Vec<Session>,           // All loaded sessions
    filtered_sessions: Vec<Session>,  // Search results
    search_matches: Vec<SearchMatch>, // Detailed match info
    selected: usize,                  // Current selection index
    search_query: String,             // Active search query
    is_searching: bool,               // UI mode flag
    view_mode: ViewMode,              // Summary/FullJson/SnippetBrowser
    error_message: Option<String>,
    should_quit: bool,
}
```

**Key Methods**:
- `load_sessions()` - Parse directory and populate sessions
- `render()` - Render current UI state
- `handle_key_event()` - Process keyboard input
- `search_sessions()` - Perform fuzzy search
- `expand_search_intent()` - Intent-driven keyword expansion

### 3. Search Engine

**Purpose**: Fuzzy matching with intent expansion

```rust
pub fn search_sessions(&mut self) {
    let expanded_queries = self.expand_search_intent(&self.search_query);

    for session in &self.sessions {
        for block in &session.blocks {
            // Direct substring match (high priority)
            let has_direct_match = content_lower.contains(&query_lower);

            // Fuzzy match with expanded keywords
            for query in &expanded_queries {
                if let Some(score) = matcher.fuzzy_match(&content, query) {
                    // Score boosted if direct match found
                }
            }

            // Word-level substring matching
            for word in query.split_whitespace() {
                if content_lower.contains(&word) {
                    score = Some(500);
                }
            }
        }
    }
}
```

**Scoring Algorithm**:
1. Direct substring match: +1000 points
2. Fuzzy match: Base score (higher for better match)
3. Word match: 500 points
4. Results sorted by highest score

### 4. Intent-Driven Keyword Expansion

The search automatically expands queries with domain-specific keywords:

```rust
// Example: "nabia" expands to:
vec![
    "nabia",
    "federation", "memchain", "orchestration",
    "agent", "coordination", "protocol",
    "cognitive", "intelligence",
]

// Example: "find" expands to:
vec![
    "find",
    "search", "locate", "discover",
    "identify", "retrieve", "lookup",
]
```

**Expansion Categories**:
- **Pattern Keywords**: Extract from domain-specific terms
- **Domain Keywords**: Action-based and context-aware
- **Semantic Variations**: Synonyms and related concepts
- **Quoted Phrases**: Explicit strings in query
- **Identifiers**: camelCase, kebab-case, ACRONYMS

## 📊 Data Models

### Session

```rust
pub struct Session {
    pub metadata: SessionMetadata,
    pub blocks: Vec<Block>,
    pub insights: SessionInsights,
}

pub struct SessionMetadata {
    pub file_path: String,
    pub file_size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub line_count: usize,
}

pub struct Block {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub role: MessageRole,  // User/Assistant/System
    pub content: Content,
}

pub struct Content {
    pub raw_text: String,
    pub tokens: Option<usize>,
    pub language: Option<String>,
}
```

### SearchMatch

```rust
pub struct SearchMatch {
    pub session_index: usize,      // Session in filtered_sessions
    pub block_index: usize,        // Block within session
    pub score: i64,                // Relevance score
    pub snippet: String,           // Context snippet (300 chars)
    pub full_json: String,         // Full JSON of block
}
```

## ⚡ Performance Characteristics

### Loading Performance

| Metric | Value |
|--------|-------|
| Files to load | 1,322 |
| Max concurrent | CPU count (up to 16) |
| Avg file size | ~100KB |
| Total size | ~130MB |
| Load time | 2-5 seconds |
| Memory usage | ~300MB |

### Search Performance

| Operation | Time |
|-----------|------|
| Simple query | <50ms |
| Complex multi-keyword | <100ms |
| Fuzzy matching (1000 files) | <200ms |
| Snippet generation | <10ms per match |

### Memory Management

- **Default limit**: 1GB per file
- **Total budget**: Unlimited (loads all files)
- **Streaming**: Line-by-line parsing (doesn't load full file into memory)
- **Parallel semaphore**: Limits concurrent file operations

## 🔧 Key Features Implementation

### 1. Parallel File Loading

```rust
// Semaphore controls concurrency
let semaphore = Arc::new(Semaphore::new(self.max_concurrent_files));

// Each file waits for permit before parsing
tokio::spawn(async move {
    let _permit = permit.acquire().await.unwrap();
    parser.parse_file(path).await
})
```

**Benefits**:
- Prevents file descriptor exhaustion
- Controlled CPU utilization
- Predictable memory usage

### 2. Error Recovery

```rust
// Configurable strategies
skip_malformed_lines: true,          // Continue on parse errors
max_consecutive_errors: 50,          // Threshold before abort
continue_on_critical_errors: true,   // Don't fail on file open errors
detailed_error_reporting: false,     // Reduce log noise
```

### 3. Snippet Generation

```rust
// Context window around match
let context_before = 100;  // chars before match
let context_after = 200;   // chars after match

// Extract snippet with:
// - UTF-8 boundary safety
// - Word boundary alignment
// - Ellipsis indicators
// - Query highlighting (uppercase)
```

### 4. View Mode Cycling

```
Summary (status + first match)
    ↓ press 'v'
FullJson (complete JSON)
    ↓ press 'v'
SnippetBrowser (interactive match browsing)
    ↓ press 'v'
Summary (loop back)
```

## 🚀 Performance Optimizations

### Current Optimizations
✅ **Time-based filtering** - Load only recent files by modification time
✅ Parallel file parsing
✅ Streaming JSONL parsing (not loading full files)
✅ Semaphore-based concurrency control
✅ LRU caching for fuzzy matcher
✅ Reduced logging in normal operation (opt-in via RUST_LOG)

### How Time Filtering Works

```rust
// Filter files before parsing
let cutoff = SystemTime::now() - Duration::from_secs(7 * 86400);  // 7 days ago

for entry in walkdir {
    if let Ok(metadata) = fs::metadata(entry.path()) {
        if metadata.modified()? > cutoff {
            parse_file(entry)  // Only parse recent files
        }
    }
}
```

**Impact**:
- **Without filter**: 1,329 files → 10+ seconds
- **With `--since 7d`**: 521 files → 2-3 seconds
- **Speedup**: 4-5x faster loading!

### Future Optimizations
- [ ] Persist parsed sessions to SQLite for instant reload
- [ ] Incremental indexing (detect new files since last run)
- [ ] Full-text search engine (Tantivy integration)
- [ ] Session compression (already in Cargo.toml as optional)
- [ ] Memory mapping for large files (memmap2)

## 🧪 Testing Strategy

### Unit Tests
- Snippet generation with UTF-8 boundaries
- Keyword expansion logic
- Path component extraction
- Search scoring

### Integration Tests
- Full directory parsing
- Search with demo data
- All view modes
- Keyboard event handling

### Performance Tests
Benchmarks available in `benches/parser_benchmarks.rs`

## 📚 Code Organization

```
src/
├── main.rs                    # Entry point, terminal setup
├── lib.rs                     # Public API
├── models.rs                  # Session/Block/Content structs
├── parser.rs                  # JSONL parsing logic (500+ lines)
├── error.rs                   # Error types and handling
├── ui/
│   ├── mod.rs                # Public UI exports
│   ├── app.rs                # App state and logic (1100+ lines)
│   ├── session_tree.rs       # Session tree widget
│   └── conversation.rs       # Conversation widget
├── search/                    # Search engine (optional feature)
├── v2/                        # V2 architecture (experimental)
└── bin/
    └── benchmark.rs          # Performance benchmarks
```

## 🔌 Extension Points

### Adding New View Modes

```rust
pub enum ViewMode {
    Summary,       // Existing
    FullJson,      // Existing
    SnippetBrowser, // Existing
    Timeline,      // TODO: Add
    Statistics,    // TODO: Add
}

// Implement in render() match block
ViewMode::Timeline => self.render_timeline(frame, area),
```

### Custom Search Strategies

```rust
// Extend expand_search_intent() with new domain patterns
domain_patterns.push((
    "my_domain",
    vec!["keyword1", "keyword2", "synonym"],
));
```

### New Data Extractors

```rust
// Add to ExtractionConfig
pub extract_my_feature: bool,

// Implement in convert_to_block()
if self.extraction_config.extract_my_feature {
    extract_my_feature(&raw_message)?;
}
```

## 🎯 Design Decisions

| Decision | Rationale |
|----------|-----------|
| Async Rust | High performance, safe concurrency |
| Ratatui TUI | Cross-platform, rich widgets |
| JSONL streaming | Memory efficient for large files |
| Fuzzy matcher | Better UX than exact matching |
| Keyword expansion | Find relevant results even with typos |
| Read-only mode | Safe exploration without accidents |

## 📖 References

- **Session format**: JSONL (newline-delimited JSON)
- **Parser**: `walkdir` for directory traversal, `tokio` for async
- **UI**: `ratatui` + `crossterm` for terminal rendering
- **Search**: `fuzzy_matcher` (skim algorithm) for fuzzy matching

---

**Last Updated**: 2025-10-29
**Architecture Version**: 2.0
**Status**: Production Ready
