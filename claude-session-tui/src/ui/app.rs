//! Minimal App implementation to get the TUI running.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{models::Session, parse_session_directory};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

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

        // Right pane: details/help
        let right_content = if let Some(err) = &self.error_message {
            Text::from(vec![Line::from(vec![
                Span::styled(
                    "Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(err),
            ])])
        } else if let Some(session) = self.filtered_sessions.get(self.selected) {
            let header = Line::from(vec![
                Span::styled(
                    "Session: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(session.metadata.file_path.clone()),
            ]);
            let stats = Line::from(format!(
                "Blocks: {}  Duration: {:?}",
                session.statistics.total_blocks,
                session.duration()
            ));
            let help = Line::from("Keys: / search  ↑/↓ navigate  q quit");
            Text::from(vec![header, stats, Line::from(""), help])
        } else {
            Text::from("Select a session. Keys: / search  ↑/↓ navigate  q quit")
        };
        let details = Paragraph::new(right_content)
            .block(Block::default().title("Details").borders(Borders::ALL));
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
            _ => {}
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

    /// Perform fuzzy search on sessions and blocks
    pub fn search_sessions(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_sessions = self.sessions.clone();
            return;
        }

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

            if let Some(score) = matcher.fuzzy_match(&content, &self.search_query) {
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
