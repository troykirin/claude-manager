//! Minimal App implementation to get the TUI running.

use std::collections::HashSet;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{models::Session, parse_session_directory};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use textwrap;

/// Application struct with fuzzy search support
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Summary,
    FullJson,
    SnippetBrowser,
}

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub session_index: usize,
    pub block_index: usize,
    pub score: i64,
    pub snippet: String,
    pub full_json: String,
}

pub struct App {
    sessions: Vec<Session>,
    filtered_sessions: Vec<Session>,  // Filtered search results
    search_matches: Vec<SearchMatch>, // Detailed matches with snippets
    selected: usize,
    snippet_index: usize, // For snippet browsing
    snippet_scroll_offset: usize,
    search_query: String, // Current search input
    is_searching: bool,   // UI mode flag
    view_mode: ViewMode,  // View mode for details pane
    error_message: Option<String>,
    should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            sessions: Vec::new(),
            filtered_sessions: Vec::new(),
            search_matches: Vec::new(),
            selected: 0,
            snippet_index: 0,
            snippet_scroll_offset: 0,
            search_query: String::new(),
            is_searching: false,
            view_mode: ViewMode::Summary,
            error_message: None,
            should_quit: false,
        })
    }

    /// Load sessions from a directory using the parser
    pub async fn load_sessions(&mut self, directory: PathBuf) -> anyhow::Result<()> {
        match parse_session_directory(directory).await {
            Ok(mut sessions) => {
                sessions.sort_by_key(|s| s.metadata.created_at);
                self.selected = 0;
                self.sessions = sessions;
                self.filtered_sessions = self.sessions.clone(); // Initialize filtered
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        // Update help text based on view mode
        let help_text = match self.view_mode {
            ViewMode::SnippetBrowser => "n/p:navigate  ↑↓:scroll  v:exit  q:quit",
            _ => "Keys: / search  v toggle view  j/k/↑/↓ navigate (works in search)  q quit",
        };
        let size = frame.area();

        // New layout: search bar at top, main content below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(size);

        // Search bar at top with match count
        let search_bar = if self.is_searching {
            Paragraph::new(format!("Search: {}", self.search_query))
                .block(Block::default().borders(Borders::ALL).title("Search (Enter to apply)"))
        } else if !self.search_matches.is_empty() {
            let title = format!("Search - {} matches found (press 'v' to browse snippets)", self.search_matches.len());
            Paragraph::new(format!("Query: {}", self.search_query))
                .block(Block::default().borders(Borders::ALL).title(title))
                .style(Style::default().fg(Color::Green))
        } else if !self.search_query.is_empty() {
            Paragraph::new(format!("No matches for: {}", self.search_query))
                .block(Block::default().borders(Borders::ALL).title("Search"))
                .style(Style::default().fg(Color::Yellow))
        } else {
            Paragraph::new("Press '/' to search")
                .block(Block::default().borders(Borders::ALL).title("Search"))
        };
        frame.render_widget(search_bar, chunks[0]);

        // Create layout for main content with footer
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(chunks[1]);

        // Main content area (sessions + details)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(content_chunks[0]);

        // Left pane: filtered sessions list
        let items: Vec<ListItem> = if self.filtered_sessions.is_empty() {
            vec![ListItem::new("No sessions loaded").style(Style::default().fg(Color::Yellow))]
        } else {
            self.filtered_sessions
                .iter()
                .enumerate()
                .map(|(idx, s)| {
                    let fname = s
                        .metadata
                        .file_path
                        .rsplit('/')
                        .next()
                        .unwrap_or(&s.metadata.file_path);
                    let label = format!("{:>3}  {}", idx + 1, fname);
                    ListItem::new(label)
                })
                .collect()
        };
        let mut state = ratatui::widgets::ListState::default();
        state.select(if self.filtered_sessions.is_empty() {
            None
        } else {
            Some(self.selected)
        });
        let sessions_list = List::new(items)
            .block(Block::default().title("Sessions").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(sessions_list, main_chunks[0], &mut state);

        // Right pane: status, full JSON, or snippet browser
        let right_area = main_chunks[1];
        let right_title = match self.view_mode {
            ViewMode::Summary => "Status",
            ViewMode::FullJson => "Full JSON",
            ViewMode::SnippetBrowser => "Snippet Browser",
        };

        match self.view_mode {
            ViewMode::Summary => {
                let selected_session_name = self
                    .filtered_sessions
                    .get(self.selected)
                    .map(|s| s.metadata.file_path.clone())
                    .unwrap_or_else(|| "None".to_string());

                let status_text = if self.is_searching {
                    "Searching".to_string()
                } else if let Some(err) = &self.error_message {
                    format!("Error: {}", err)
                } else {
                    "Ready".to_string()
                };

                let mut right_content = vec![
                    Line::from(format!("Total Sessions: {}", self.sessions.len())),
                    Line::from(format!("Filtered: {}", self.filtered_sessions.len())),
                    Line::from(format!(
                        "Selected: {} ({})",
                        self.selected + 1,
                        selected_session_name
                    )),
                    Line::from(format!("Status: {}", status_text)),
                    Line::from(""),
                ];

                // Show first match snippet if available
                if !self.search_matches.is_empty() {
                    right_content.push(Line::from("─────────────────────────────────────"));
                    right_content.push(Line::from(format!("First match (of {}):", self.search_matches.len())));
                    right_content.push(Line::from(""));
                    
                    let first_match = &self.search_matches[0];
                    // Wrap the snippet text for display
                    let snippet_lines: Vec<String> = first_match.snippet
                        .split('\n')
                        .flat_map(|line| {
                            textwrap::wrap(line, 50)
                                .into_iter()
                                .map(|s| s.to_string())
                        })
                        .take(8) // Limit lines in summary
                        .collect();
                    
                    for line in snippet_lines {
                        right_content.push(Line::from(Span::styled(
                            line,
                            Style::default().fg(Color::Cyan)
                        )));
                    }
                    
                    right_content.push(Line::from(""));
                    right_content.push(Line::from(Span::styled(
                        "Press 'v' to browse all snippets",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    )));
                } else {
                    right_content.push(Line::from(help_text));
                }

                let details = Paragraph::new(right_content)
                    .block(Block::default().title(right_title).borders(Borders::ALL));
                frame.render_widget(details, right_area);
            }
            ViewMode::FullJson => {
                if let Some(match_info) = self.search_matches.get(self.snippet_index) {
                    // Show full JSON of the matched block
                    let json_text = match_info.full_json.clone();
                    let json_lines: Vec<Line> = json_text
                        .lines()
                        .take(50) // Limit lines to fit in pane
                        .enumerate()
                        .map(|(i, line)| {
                            let color = match i % 3 {
                                0 => Color::Cyan,
                                1 => Color::Green,
                                _ => Color::White,
                            };
                            Line::from(Span::styled(line.to_string(), Style::default().fg(color)))
                        })
                        .collect();

                    let json_content = Text::from(json_lines);
                    let details = Paragraph::new(json_content)
                        .block(Block::default().title(right_title).borders(Borders::ALL))
                        .wrap(ratatui::widgets::Wrap { trim: true });
                    frame.render_widget(details, right_area);
                } else if let Some(session) = self.filtered_sessions.get(self.selected) {
                    // Fallback: Show session JSON
                    let json_text = serde_json::to_string_pretty(&session)
                        .unwrap_or_else(|_| "JSON error".to_string());
                    let json_lines: Vec<Line> = json_text
                        .lines()
                        .take(50) // Limit lines to fit in pane
                        .enumerate()
                        .map(|(i, line)| {
                            let color = match i % 3 {
                                0 => Color::Cyan,
                                1 => Color::Green,
                                _ => Color::White,
                            };
                            Line::from(Span::styled(line.to_string(), Style::default().fg(color)))
                        })
                        .collect();

                    let json_content = Text::from(json_lines);
                    let details = Paragraph::new(json_content)
                        .block(Block::default().title(right_title).borders(Borders::ALL))
                        .wrap(ratatui::widgets::Wrap { trim: true });
                    frame.render_widget(details, right_area);
                } else {
                    let details = Paragraph::new("No data available")
                        .block(Block::default().title(right_title).borders(Borders::ALL));
                    frame.render_widget(details, right_area);
                }
            }
            ViewMode::SnippetBrowser => {
                self.render_snippet_browser(frame, right_area);
            }
        }

        // Footer with version info
        let version_text = format!("v{}", VERSION);
        let footer = Paragraph::new(version_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right);
        frame.render_widget(footer, content_chunks[1]);
    }

    /// Handle keyboard input
    pub fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.is_searching {
                    self.is_searching = false;
                    self.search_query.clear();
                    self.filtered_sessions = self.sessions.clone();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            // Vim-style navigation (prioritize navigation over search input)
            KeyCode::Char('j') => {
                if self.selected + 1 < self.filtered_sessions.len() {
                    self.selected += 1;
                } else if self.is_searching {
                    self.search_query.push('j');
                }
            }
            KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else if self.is_searching {
                    self.search_query.push('k');
                }
            }
            KeyCode::Char('v') => {
                if !self.is_searching {
                    self.view_mode = match self.view_mode {
                        ViewMode::Summary => ViewMode::FullJson,
                        ViewMode::FullJson => ViewMode::SnippetBrowser,
                        ViewMode::SnippetBrowser => ViewMode::Summary,
                    };
                    if self.view_mode == ViewMode::SnippetBrowser && !self.search_matches.is_empty()
                    {
                        self.snippet_index = 0;
                        self.snippet_scroll_offset = 0;
                    }
                }
            }
            // Snippet browser controls (work in any mode, don't add to search)
            KeyCode::Char('n') => {
                if self.view_mode == ViewMode::SnippetBrowser && !self.search_matches.is_empty() {
                    self.snippet_index = (self.snippet_index + 1) % self.search_matches.len();
                    self.snippet_scroll_offset = 0;
                } else if self.is_searching {
                    self.search_query.push('n');
                }
            }
            KeyCode::Char('p') => {
                if self.view_mode == ViewMode::SnippetBrowser && !self.search_matches.is_empty() {
                    self.snippet_index = if self.snippet_index == 0 {
                        self.search_matches.len() - 1
                    } else {
                        self.snippet_index - 1
                    };
                    self.snippet_scroll_offset = 0;
                } else if self.is_searching {
                    self.search_query.push('p');
                }
            }
            KeyCode::Char('/') => {
                self.is_searching = true;
                self.search_query.clear();
            }
            KeyCode::Enter => {
                if self.is_searching {
                    self.search_sessions();
                    self.is_searching = false;
                }
            }
            KeyCode::Backspace => {
                if self.is_searching {
                    self.search_query.pop();
                }
            }
            KeyCode::Char(c) => {
                if self.is_searching {
                    self.search_query.push(c);
                }
            }
            KeyCode::Up => {
                if self.view_mode == ViewMode::SnippetBrowser {
                    self.scroll_snippet(-1);
                } else if !self.is_searching && self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.view_mode == ViewMode::SnippetBrowser {
                    self.scroll_snippet(1);
                } else if !self.is_searching && self.selected + 1 < self.filtered_sessions.len() {
                    self.selected += 1;
                }
            }
            _ => {} // Catch all other key codes
        }
        Ok(())
    }

    /// Update state (no-op for now)
    pub fn update(&mut self) {}

    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    /// Expand search query with intent-driven keywords
    pub fn expand_search_intent(&self, original_intent: &str) -> Vec<String> {
        let mut keywords = vec![original_intent.to_string()];

        // Extract base keywords using pattern matching
        let pattern_keywords = self.extract_pattern_keywords(original_intent);
        keywords.extend(pattern_keywords);

        // Generate domain-specific expansions
        let domain_keywords = self.generate_domain_keywords(original_intent);
        keywords.extend(domain_keywords);

        // Extract semantic variations
        let semantic_keywords = self.generate_semantic_variations(original_intent);
        keywords.extend(semantic_keywords);

        // Remove duplicates
        keywords
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn extract_pattern_keywords(&self, intent: &str) -> Vec<String> {
        let intent_lower = intent.to_lowercase();
        let mut keywords = Vec::new();

        // Technical domain patterns
        let domain_patterns = [
            (
                "nabia",
                vec![
                    "federation",
                    "memchain",
                    "orchestration",
                    "agent",
                    "coordination",
                    "protocol",
                    "cognitive",
                    "intelligence",
                ],
            ),
            (
                "claude",
                vec![
                    "assistant",
                    "conversation",
                    "chat",
                    "ai",
                    "llm",
                    "dialogue",
                    "anthropic",
                    "subagent",
                ],
            ),
            (
                "linear",
                vec![
                    "issue",
                    "project",
                    "task",
                    "ticket",
                    "workflow",
                    "development",
                    "tracking",
                    "milestone",
                ],
            ),
            (
                "federation",
                vec![
                    "agent",
                    "coordination",
                    "protocol",
                    "handoff",
                    "orchestration",
                    "distributed",
                    "network",
                    "mesh",
                ],
            ),
            (
                "memory",
                vec![
                    "storage",
                    "retrieval",
                    "context",
                    "persistent",
                    "ephemeral",
                    "knowledge",
                    "cache",
                    "state",
                ],
            ),
            (
                "search",
                vec![
                    "query", "find", "lookup", "discover", "filter", "match", "locate", "identify",
                ],
            ),
            (
                "integration",
                vec![
                    "api",
                    "webhook",
                    "connection",
                    "sync",
                    "bridge",
                    "interface",
                    "mcp",
                    "proxy",
                ],
            ),
            (
                "architecture",
                vec![
                    "design",
                    "pattern",
                    "structure",
                    "framework",
                    "system",
                    "blueprint",
                    "topology",
                ],
            ),
            (
                "git",
                vec![
                    "commit",
                    "branch",
                    "merge",
                    "repository",
                    "version",
                    "control",
                    "diff",
                    "pull request",
                ],
            ),
            (
                "riff",
                vec![
                    "search",
                    "uuid",
                    "jsonl",
                    "conversation",
                    "logs",
                    "cli",
                    "tool",
                    "query",
                ],
            ),
            (
                "agent",
                vec![
                    "subagent",
                    "orchestrator",
                    "delegation",
                    "task",
                    "autonomous",
                    "cognitive",
                    "intelligent",
                ],
            ),
            (
                "oauth",
                vec![
                    "authentication",
                    "authorization",
                    "token",
                    "proxy",
                    "grok",
                    "notion",
                    "api",
                ],
            ),
        ];

        for (term, expansions) in domain_patterns.iter() {
            if intent_lower.contains(term) {
                keywords.extend(expansions.iter().map(|s| s.to_string()));
            }
        }

        // Extract quoted phrases
        if let Some(re) = regex::Regex::new(r#""([^"]*)""#).ok() {
            for cap in re.captures_iter(intent) {
                if let Some(phrase) = cap.get(1) {
                    keywords.push(phrase.as_str().to_string());
                }
            }
        }

        // Extract camelCase and kebab-case terms
        if let Some(re) = regex::Regex::new(r"\b[a-z]+[A-Z][a-zA-Z]*\b|\b[a-z]+-[a-z-]+\b").ok() {
            for cap in re.captures_iter(intent) {
                keywords.push(cap.get(0).unwrap().as_str().to_string());
            }
        }

        // Extract abbreviations
        if let Some(re) = regex::Regex::new(r"\b[A-Z]{2,}\b").ok() {
            for cap in re.captures_iter(intent) {
                keywords.push(cap.get(0).unwrap().as_str().to_string());
            }
        }

        keywords
    }

    fn generate_domain_keywords(&self, intent: &str) -> Vec<String> {
        let intent_lower = intent.to_lowercase();
        let mut keywords = Vec::new();

        // Action-based expansions
        let action_patterns = [
            (
                "find",
                vec![
                    "search", "locate", "discover", "identify", "retrieve", "lookup",
                ],
            ),
            (
                "search",
                vec!["find", "query", "scan", "browse", "explore", "investigate"],
            ),
            (
                "discuss",
                vec![
                    "talk",
                    "conversation",
                    "dialogue",
                    "chat",
                    "communication",
                    "exchange",
                ],
            ),
            (
                "implement",
                vec!["build", "create", "develop", "construct", "design", "code"],
            ),
            (
                "configure",
                vec![
                    "setup",
                    "initialize",
                    "customize",
                    "adjust",
                    "modify",
                    "tune",
                ],
            ),
            (
                "integrate",
                vec!["connect", "link", "bridge", "sync", "merge", "combine"],
            ),
            (
                "debug",
                vec![
                    "troubleshoot",
                    "diagnose",
                    "fix",
                    "resolve",
                    "investigate",
                    "analyze",
                ],
            ),
        ];

        for (action, synonyms) in action_patterns.iter() {
            if intent_lower.contains(action) {
                keywords.extend(synonyms.iter().take(3).map(|s| s.to_string()));
            }
        }

        // Context-aware technical expansions
        if intent_lower.contains("conversation")
            || intent_lower.contains("chat")
            || intent_lower.contains("talk")
        {
            keywords.extend(
                [
                    "message",
                    "dialogue",
                    "transcript",
                    "session",
                    "interaction",
                ]
                .into_iter()
                .map(|s| s.to_string()),
            );
        }
        if intent_lower.contains("project")
            || intent_lower.contains("build")
            || intent_lower.contains("develop")
        {
            keywords.extend(
                [
                    "implementation",
                    "feature",
                    "module",
                    "component",
                    "service",
                ]
                .into_iter()
                .map(|s| s.to_string()),
            );
        }
        if intent_lower.contains("error")
            || intent_lower.contains("issue")
            || intent_lower.contains("problem")
        {
            keywords.extend(
                ["bug", "fault", "exception", "failure", "crash"]
                    .into_iter()
                    .map(|s| s.to_string()),
            );
        }
        if intent_lower.contains("config")
            || intent_lower.contains("setup")
            || intent_lower.contains("install")
        {
            keywords.extend(
                [
                    "configuration",
                    "initialization",
                    "deployment",
                    "environment",
                ]
                .into_iter()
                .map(|s| s.to_string()),
            );
        }

        keywords.into_iter().map(|s| s.to_string()).collect()
    }

    fn generate_semantic_variations(&self, intent: &str) -> Vec<String> {
        let intent_lower = intent.to_lowercase();
        let mut keywords = Vec::new();

        // Semantic relationship mappings
        let semantic_maps = [
            (
                "agent",
                vec!["bot", "assistant", "worker", "service", "process"],
            ),
            (
                "system",
                vec!["platform", "framework", "infrastructure", "architecture"],
            ),
            (
                "data",
                vec!["information", "content", "payload", "dataset", "record"],
            ),
            (
                "process",
                vec!["workflow", "pipeline", "procedure", "operation", "task"],
            ),
            (
                "network",
                vec!["connection", "link", "channel", "communication", "protocol"],
            ),
            (
                "interface",
                vec!["api", "endpoint", "contract", "specification", "definition"],
            ),
            (
                "state",
                vec!["status", "condition", "mode", "phase", "situation"],
            ),
            (
                "event",
                vec!["message", "signal", "notification", "trigger", "callback"],
            ),
        ];

        for (base_term, variations) in semantic_maps.iter() {
            if intent_lower.contains(base_term) {
                keywords.extend(variations.iter().map(|s| s.to_string()));
            }
        }

        // Technical context variations
        if intent_lower.contains("cli") {
            keywords.extend(
                ["command", "terminal", "shell", "console", "interface"]
                    .into_iter()
                    .map(|s| s.to_string()),
            );
        }
        if intent_lower.contains("json") {
            keywords.extend(
                ["jsonl", "data", "format", "structure", "payload"]
                    .into_iter()
                    .map(|s| s.to_string()),
            );
        }
        if intent_lower.contains("uuid") {
            keywords.extend(
                ["identifier", "id", "key", "reference", "unique"]
                    .into_iter()
                    .map(|s| s.to_string()),
            );
        }

        keywords
    }

    /// Perform fuzzy search on sessions and blocks
    pub fn search_sessions(&mut self) {
        // Clear search if query is empty
        if self.search_query.is_empty() {
            self.filtered_sessions = self.sessions.clone();
            self.search_matches.clear();
            self.selected = 0;
            return;
        }

        // Handle case where no sessions are loaded
        if self.sessions.is_empty() {
            self.filtered_sessions.clear();
            self.search_matches.clear();
            self.selected = 0;
            return;
        }

        // Expand the search query with intent enhancement
        let expanded_queries = self.expand_search_intent(&self.search_query);

        let matcher = SkimMatcherV2::default();
        let mut scored_sessions = Vec::new();
        let mut search_matches = Vec::new();

        // Also do case-insensitive substring matching for better results
        let query_lower = self.search_query.to_lowercase();

        for (session_idx, session) in self.sessions.iter().enumerate() {
            let mut session_has_matches = false;
            let mut session_max_score = 0;

            for (block_idx, block) in session.blocks.iter().enumerate() {
                let content = format!("{} {}", session.metadata.file_path, block.content.raw_text);
                let content_lower = content.to_lowercase();

                // First, check for direct substring match (higher priority)
                let has_direct_match = content_lower.contains(&query_lower);
                
                // Search with expanded keywords using fuzzy matcher
                let mut max_score = if has_direct_match { Some(1000) } else { None };
                
                for query in &expanded_queries {
                    if let Some(score) = matcher.fuzzy_match(&content, query) {
                        let adjusted_score = if has_direct_match { score + 1000 } else { score };
                        if max_score.is_none() || adjusted_score > max_score.unwrap() {
                            max_score = Some(adjusted_score);
                        }
                    }
                }

                // Also check for substring matches on individual words
                if max_score.is_none() {
                    for word in self.search_query.split_whitespace() {
                        if content_lower.contains(&word.to_lowercase()) {
                            max_score = Some(500); // Give a moderate score for word matches
                            break;
                        }
                    }
                }

                if let Some(score) = max_score {
                    session_has_matches = true;
                    if score > session_max_score {
                        session_max_score = score;
                    }

                    // Create a better snippet for this block with more context
                    let snippet =
                        self.create_snippet(&block.content.raw_text, &self.search_query, 300);

                    // Full JSON for this block
                    let full_json = serde_json::to_string_pretty(&block)
                        .unwrap_or_else(|_| "JSON error".to_string());

                    search_matches.push(SearchMatch {
                        session_index: session_idx,
                        block_index: block_idx,
                        score: score as i64,
                        snippet,
                        full_json,
                    });
                }
            }

            if session_has_matches {
                scored_sessions.push((session.clone(), session_max_score));
            }
        }

        // Sort sessions by score (highest first)
        scored_sessions.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Store whether we have any matches before moving scored_sessions
        let has_matches = !scored_sessions.is_empty();
        
        self.filtered_sessions = scored_sessions
            .into_iter()
            .take(50)
            .map(|(s, _)| s)
            .collect();

        // Sort matches by score and limit
        search_matches.sort_by(|a, b| b.score.cmp(&a.score));
        search_matches.truncate(200); // Increase limit for more matches
        self.search_matches = search_matches;

        // Reset selection and ensure it's within bounds
        self.selected = 0;
        self.snippet_index = 0;
        self.snippet_scroll_offset = 0;
        
        // If no sessions matched, show all sessions (no filter)
        if self.filtered_sessions.is_empty() && has_matches {
            // Fall back to showing all sessions if something went wrong
            self.filtered_sessions = self.sessions.clone();
        }
    }

    fn create_snippet(&self, text: &str, query: &str, max_length: usize) -> String {
        // Handle empty text or query
        if text.is_empty() {
            return String::from("[Empty content]");
        }
        if query.is_empty() {
            let preview_len = text.len().min(max_length);
            return text[..preview_len].to_string();
        }

        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        // Find the best match position
        let mut match_pos = None;
        
        // First try exact phrase match
        if let Some(pos) = text_lower.find(&query_lower) {
            match_pos = Some(pos);
        }
        
        // If no exact match, try to find the first word
        if match_pos.is_none() {
            for word in query.split_whitespace() {
                if !word.is_empty() {
                    if let Some(pos) = text_lower.find(&word.to_lowercase()) {
                        match_pos = Some(pos);
                        break;
                    }
                }
            }
        }
        
        // Default to beginning if no match found
        let pos = match_pos.unwrap_or(0);
        
        // Calculate context window around the match
        let context_before = 100; // characters before match
        let context_after = 200;  // characters after match
        
        // Safely calculate start position with bounds checking
        let start = if pos > context_before && pos < text.len() { 
            // Try to start at a word boundary
            let mut s = pos.saturating_sub(context_before);
            // Use char_indices for safe UTF-8 boundary iteration
            let chars: Vec<_> = text.char_indices().collect();
            while s > 0 {
                let is_boundary = chars.iter()
                    .find(|(i, _)| *i == s)
                    .map(|(_, c)| c.is_whitespace())
                    .unwrap_or(false);
                if is_boundary {
                    break;
                }
                s = s.saturating_sub(1);
            }
            s
        } else { 
            0 
        };
        
        // Safely calculate end position
        let end = if pos + context_after < text.len() {
            // Try to end at a word boundary
            let mut e = (pos + context_after).min(text.len());
            let chars: Vec<_> = text.char_indices().collect();
            while e < text.len() {
                let is_boundary = chars.iter()
                    .find(|(i, _)| *i == e)
                    .map(|(_, c)| c.is_whitespace())
                    .unwrap_or(true);
                if is_boundary {
                    break;
                }
                e = (e + 1).min(text.len());
            }
            e
        } else {
            text.len()
        };

        // Ensure we're at valid UTF-8 boundaries using is_char_boundary
        let safe_start = if text.is_char_boundary(start) {
            start
        } else {
            // Find the nearest valid boundary before start
            (0..start).rev().find(|i| text.is_char_boundary(*i)).unwrap_or(0)
        };
        
        let safe_end = if text.is_char_boundary(end) {
            end
        } else {
            // Find the nearest valid boundary after end
            (end..=text.len()).find(|i| text.is_char_boundary(*i)).unwrap_or(text.len())
        };

        // Extract the snippet safely
        let mut snippet = text[safe_start..safe_end].to_string();
        
        // Add ellipsis if truncated
        if safe_start > 0 {
            snippet = format!("...{}", snippet.trim_start());
        }
        if safe_end < text.len() {
            snippet = format!("{}...", snippet.trim_end());
        }
        
        // Highlight the match in the snippet (simple uppercase for now)
        if let Some(match_idx) = snippet.to_lowercase().find(&query_lower) {
            // Ensure we don't go out of bounds when highlighting
            let match_end = (match_idx + query_lower.len()).min(snippet.len());
            if match_end <= snippet.len() {
                let before = &snippet[..match_idx];
                let matched = &snippet[match_idx..match_end];
                let after = &snippet[match_end..];
                snippet = format!("{}[{}]{}", before, matched.to_uppercase(), after);
            }
        }
        
        // Ensure snippet doesn't exceed max length
        if snippet.len() > max_length {
            // Find a valid char boundary for truncation
            let truncate_pos = if snippet.is_char_boundary(max_length) {
                max_length
            } else {
                (0..max_length).rev().find(|i| snippet.is_char_boundary(*i)).unwrap_or(0)
            };
            snippet.truncate(truncate_pos);
            snippet.push_str("...");
        }
        
        snippet
    }

    fn render_snippet_browser(&self, frame: &mut Frame, area: Rect) {
        if self.search_matches.is_empty() {
            let content = Paragraph::new("No matches found").block(
                Block::default()
                    .title("Snippet Browser")
                    .borders(Borders::ALL),
            );
            frame.render_widget(content, area);
            return;
        }

        let match_info = &self.search_matches[self.snippet_index];
        let header_text = format!(
            "Match {}/{} (Score: {}) - Session {} Block {}",
            self.snippet_index + 1,
            self.search_matches.len(),
            match_info.score,
            match_info.session_index + 1,
            match_info.block_index + 1
        );

        // Wrap snippet text for display
        let wrapped_lines: Vec<String> = match_info
            .snippet
            .split('\n')
            .flat_map(|line| textwrap::wrap(line, 60).into_iter().map(|s| s.to_string()))
            .collect();

        let total_lines = wrapped_lines.len();
        let max_display_lines = 15;
        let start_line = self.snippet_scroll_offset;
        let end_line = (start_line + max_display_lines).min(total_lines);
        let display_lines = &wrapped_lines[start_line..end_line];

        let snippet_display = display_lines
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        let scroll_info = if total_lines > max_display_lines {
            format!(
                " | Showing lines {}-{} of {} (↑↓ scroll, n/p navigate, v full, q exit)",
                start_line + 1,
                end_line,
                total_lines
            )
        } else {
            " | n/p navigate, v view full, q exit".to_string()
        };

        let full_content = format!(
            "{}\n{}{}\n\n{}",
            header_text,
            "─".repeat(60),
            snippet_display,
            scroll_info
        );

        let content = Paragraph::new(full_content)
            .block(
                Block::default()
                    .title("Snippet Browser")
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(content, area);
    }

    pub fn scroll_snippet(&mut self, delta: i32) {
        if self.view_mode != ViewMode::SnippetBrowser || self.search_matches.is_empty() {
            return;
        }

        let match_info = &self.search_matches[self.snippet_index];
        let total_lines = match_info
            .snippet
            .split('\n')
            .map(|line| textwrap::wrap(line, 60).len())
            .sum::<usize>();

        let max_display_lines = 15;
        let max_scroll = if total_lines > max_display_lines {
            total_lines - max_display_lines
        } else {
            0
        };

        self.snippet_scroll_offset =
            ((self.snippet_scroll_offset as i32 + delta).max(0) as usize).min(max_scroll);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
