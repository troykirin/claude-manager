//! Minimal App implementation to get the TUI running.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{models::Session, parse_session_directory};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use std::collections::HashSet;

/// Application struct with fuzzy search support
pub struct App {
    sessions: Vec<Session>,
    filtered_sessions: Vec<Session>, // Filtered search results
    selected: usize,
    search_query: String, // Current search input
    is_searching: bool,   // UI mode flag for search
    error_message: Option<String>,
    should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            sessions: Vec::new(),
            filtered_sessions: Vec::new(),
            selected: 0,
            search_query: String::new(),
            is_searching: false,
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
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        // New layout: search bar at top, main content below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(size);

        // Search bar at top
        let search_bar = if self.is_searching {
            Paragraph::new(format!("Search: {}", self.search_query))
                .block(Block::default().borders(Borders::ALL).title("Search"))
        } else {
            Paragraph::new("Press '/' to search")
                .block(Block::default().borders(Borders::ALL).title("Search"))
        };
        frame.render_widget(search_bar, chunks[0]);

        // Main content below
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(chunks[1]);

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

        // Right pane: always show status info like ratatui-demo
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

        let right_content = vec![
            Line::from(format!("Total Sessions: {}", self.sessions.len())),
            Line::from(format!("Filtered: {}", self.filtered_sessions.len())),
            Line::from(format!(
                "Selected: {} ({})",
                self.selected + 1,
                selected_session_name
            )),
            Line::from(format!("Status: {}", status_text)),
            Line::from(""),
            Line::from("Keys: / search  j/k/↑/↓ navigate  q quit"),
        ];

        let details = Paragraph::new(right_content)
            .block(Block::default().title("Status").borders(Borders::ALL));
        frame.render_widget(details, main_chunks[1]);
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
            // Vim-style navigation (must come before general Char)
            KeyCode::Char('j') => {
                if !self.is_searching && self.selected + 1 < self.filtered_sessions.len() {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') => {
                if !self.is_searching && self.selected > 0 {
                    self.selected -= 1;
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
                if !self.is_searching && self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if !self.is_searching && self.selected + 1 < self.filtered_sessions.len() {
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
        if self.search_query.is_empty() {
            self.filtered_sessions = self.sessions.clone();
            return;
        }

        // Expand the search query with intent enhancement
        let expanded_queries = self.expand_search_intent(&self.search_query);

        let matcher = SkimMatcherV2::default();
        let mut scored_sessions = Vec::new();

        for session in &self.sessions {
            let content = format!(
                "{} {}",
                session.metadata.file_path,
                session
                    .blocks
                    .iter()
                    .map(|b| b.content.raw_text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            // Search with expanded keywords
            let mut max_score = None;
            for query in &expanded_queries {
                if let Some(score) = matcher.fuzzy_match(&content, query) {
                    if max_score.is_none() || score > max_score.unwrap() {
                        max_score = Some(score);
                    }
                }
            }

            if let Some(score) = max_score {
                scored_sessions.push((session.clone(), score));
            }
        }

        // Sort by score (highest first) and limit results
        scored_sessions.sort_by(|a, b| b.1.cmp(&a.1));
        self.filtered_sessions = scored_sessions
            .into_iter()
            .take(50) // Limit for performance
            .map(|(s, _)| s)
            .collect();

        self.selected = 0; // Reset selection
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
