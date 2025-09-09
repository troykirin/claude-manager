//! Minimal App implementation to get the TUI running.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::{
    models::Session,
    parse_session_directory,
};

/// Minimal application struct
pub struct App {
    sessions: Vec<Session>,
    selected: usize,
    error_message: Option<String>,
    should_quit: bool,
}

impl App {
    /// Create a new application instance
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            sessions: Vec::new(),
            selected: 0,
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
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Render the UI
    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(size);

        // Left pane: sessions list
        let items: Vec<ListItem> = if self.sessions.is_empty() {
            vec![ListItem::new("No sessions loaded").style(Style::default().fg(Color::Yellow))]
        } else {
            self.sessions
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
        state.select(if self.sessions.is_empty() { None } else { Some(self.selected) });
        let sessions_list = List::new(items)
            .block(Block::default().title("Sessions").borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        frame.render_stateful_widget(sessions_list, chunks[0], &mut state);

        // Right pane: details/help
        let right_content = if let Some(err) = &self.error_message {
            Text::from(vec![
                Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(err),
                ]),
            ])
        } else if let Some(session) = self.sessions.get(self.selected) {
            let header = Line::from(vec![
                Span::styled("Session: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(session.metadata.file_path.clone()),
            ]);
            let stats = Line::from(format!(
                "Blocks: {}  Duration: {:?}",
                session.statistics.total_blocks,
                session.duration()
            ));
            let help = Line::from("Keys: ↑/↓ select  q to quit");
            Text::from(vec![header, stats, Line::from(""), help])
        } else {
            Text::from("Select a session. Keys: ↑/↓, q to quit")
        };
        let details = Paragraph::new(right_content)
            .block(Block::default().title("Details").borders(Borders::ALL));
        frame.render_widget(details, chunks[1]);
    }

    /// Handle keyboard input
    pub fn handle_key_event(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected + 1 < self.sessions.len() {
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
}

impl Default for App {
    fn default() -> Self { Self::new().unwrap() }
}
