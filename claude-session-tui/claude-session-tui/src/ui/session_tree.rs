//! Session tree browser with fzf-style navigation and fuzzy search
//! 
//! Provides an interactive tree view of all Claude sessions organized by project,
//! with instant fuzzy search, visual indicators, and keyboard navigation.

use std::collections::HashMap;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use tui_input::{Input, InputRequest};

use crate::models::{Session, Conversation};

use super::UIComponent;

/// Session tree browser component
pub struct SessionTree {
    /// All loaded sessions
    sessions: Vec<Session>,
    /// Filtered sessions based on current search
    filtered_sessions: Vec<SessionEntry>,
    /// Current list state for navigation
    list_state: ListState,
    /// Search input field
    search_input: Input,
    /// Whether the component is focused
    focused: bool,
    /// Whether in search mode
    searching: bool,
    /// Fuzzy matcher for search
    matcher: SkimMatcherV2,
    /// Project organization map
    projects: HashMap<String, Vec<usize>>,
    /// Expanded projects
    expanded_projects: std::collections::HashSet<String>,
    /// Current view mode
    view_mode: ViewMode,
}

/// Session entry for display with metadata
#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub index: usize,
    pub session: Session,
    pub match_score: i64,
    pub project_path: Option<String>,
    pub session_type: SessionType,
    pub conversation_count: usize,
}

/// Visual indicators for different session types
#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    ToolHeavy,     // 🔧
    InsightRich,   // 💡
    CodeFocused,   // 📝
    Discussion,    // 💬
    Research,      // 🔍
    Unknown,       // 📄
}

/// View mode for the session tree
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Flat,        // All sessions in a flat list
    ProjectTree, // Organized by project hierarchy
}

impl SessionTree {
    /// Create a new session tree browser
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            filtered_sessions: Vec::new(),
            list_state: ListState::default(),
            search_input: Input::default(),
            focused: false,
            searching: false,
            matcher: SkimMatcherV2::default(),
            projects: HashMap::new(),
            expanded_projects: std::collections::HashSet::new(),
            view_mode: ViewMode::ProjectTree,
        }
    }
    
    /// Update sessions data
    pub fn update_sessions(&mut self, sessions: &[Session]) {
        self.sessions = sessions.to_vec();
        self.organize_by_projects();
        self.apply_filter();
    }
    
    /// Get the currently selected session index
    pub fn get_selected_session(&self) -> Option<usize> {
        self.list_state.selected().and_then(|i| {
            self.filtered_sessions.get(i).map(|entry| entry.index)
        })
    }
    
    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if focused && self.list_state.selected().is_none() && !self.filtered_sessions.is_empty() {
            self.list_state.select(Some(0));
        }
    }
    
    /// Toggle search mode
    pub fn toggle_search(&mut self) {
        self.searching = !self.searching;
        if !self.searching {
            self.search_input = Input::default();
            self.apply_filter();
        }
    }
    
    /// Toggle view mode
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Flat => ViewMode::ProjectTree,
            ViewMode::ProjectTree => ViewMode::Flat,
        };
        self.apply_filter();
    }
    
    /// Organize sessions by project paths
    fn organize_by_projects(&mut self) {
        self.projects.clear();
        
        for (i, session) in self.sessions.iter().enumerate() {
            let project = session.project_path
                .as_deref()
                .unwrap_or("Unknown Project");
            
            self.projects
                .entry(project.to_string())
                .or_insert_with(Vec::new)
                .push(i);
        }
        
        // Auto-expand projects with few sessions
        for (project, sessions) in &self.projects {
            if sessions.len() <= 3 {
                self.expanded_projects.insert(project.clone());
            }
        }
    }
    
    /// Apply current search filter
    fn apply_filter(&mut self) {
        let query = self.search_input.value();
        
        if query.is_empty() {
            // Show all sessions
            self.filtered_sessions = self.sessions
                .iter()
                .enumerate()
                .map(|(i, session)| SessionEntry {
                    index: i,
                    session: session.clone(),
                    match_score: 0,
                    project_path: session.project_path.clone(),
                    session_type: self.classify_session(session),
                    conversation_count: session.conversations.len(),
                })
                .collect();
        } else {
            // Apply fuzzy search
            let mut scored_sessions: Vec<_> = self.sessions
                .iter()
                .enumerate()
                .filter_map(|(i, session)| {
                    let search_text = format!(
                        "{} {} {}",
                        session.title.as_deref().unwrap_or(""),
                        session.project_path.as_deref().unwrap_or(""),
                        session.conversations.iter()
                            .map(|c| c.title.as_deref().unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    
                    self.matcher.fuzzy_match(&search_text, query).map(|score| {
                        SessionEntry {
                            index: i,
                            session: session.clone(),
                            match_score: score,
                            project_path: session.project_path.clone(),
                            session_type: self.classify_session(session),
                            conversation_count: session.conversations.len(),
                        }
                    })
                })
                .collect();
            
            // Sort by match score
            scored_sessions.sort_by(|a, b| b.match_score.cmp(&a.match_score));
            self.filtered_sessions = scored_sessions;
        }
        
        // Reset selection
        if !self.filtered_sessions.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }
    
    /// Classify session type based on content
    fn classify_session(&self, session: &Session) -> SessionType {
        let total_blocks: usize = session.conversations
            .iter()
            .map(|c| c.blocks.len())
            .sum();
        
        let tool_blocks: usize = session.conversations
            .iter()
            .map(|c| c.blocks.iter().filter(|b| !b.tool_calls.is_empty()).count())
            .sum();
        
        let code_blocks: usize = session.conversations
            .iter()
            .map(|c| c.blocks.iter().filter(|b| b.content.contains("```")).count())
            .sum();
        
        if total_blocks == 0 {
            return SessionType::Unknown;
        }
        
        let tool_ratio = tool_blocks as f32 / total_blocks as f32;
        let code_ratio = code_blocks as f32 / total_blocks as f32;
        
        if tool_ratio > 0.3 {
            SessionType::ToolHeavy
        } else if code_ratio > 0.2 {
            SessionType::CodeFocused
        } else if session.title.as_deref().unwrap_or("").contains("insight") {
            SessionType::InsightRich
        } else if session.title.as_deref().unwrap_or("").contains("research") {
            SessionType::Research
        } else {
            SessionType::Discussion
        }
    }
    
    /// Get session type icon
    fn get_session_type_icon(&self, session_type: &SessionType) -> &'static str {
        match session_type {
            SessionType::ToolHeavy => "🔧",
            SessionType::InsightRich => "💡",
            SessionType::CodeFocused => "📝",
            SessionType::Discussion => "💬",
            SessionType::Research => "🔍",
            SessionType::Unknown => "📄",
        }
    }
    
    /// Create list items for rendering
    fn create_list_items(&self) -> Vec<ListItem<'static>> {
        match self.view_mode {
            ViewMode::Flat => self.create_flat_list_items(),
            ViewMode::ProjectTree => self.create_tree_list_items(),
        }
    }
    
    /// Create flat list items
    fn create_flat_list_items(&self) -> Vec<ListItem<'static>> {
        self.filtered_sessions
            .iter()
            .map(|entry| {
                let icon = self.get_session_type_icon(&entry.session_type);
                let title = entry.session.title.as_deref().unwrap_or("Untitled Session");
                let project = entry.project_path.as_deref().unwrap_or("Unknown");
                let conv_count = entry.conversation_count;
                
                let line = Line::from(vec![
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::styled(
                        title,
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    ),
                    Span::raw(format!(" ({} convs)", conv_count)),
                    Span::raw(" • "),
                    Span::styled(
                        project,
                        Style::default().fg(Color::Gray)
                    ),
                ]);
                
                ListItem::new(line)
            })
            .collect()
    }
    
    /// Create tree-structured list items
    fn create_tree_list_items(&self) -> Vec<ListItem<'static>> {
        let mut items = Vec::new();
        
        // Group filtered sessions by project
        let mut project_sessions: HashMap<String, Vec<&SessionEntry>> = HashMap::new();
        
        for entry in &self.filtered_sessions {
            let project = entry.project_path.as_deref().unwrap_or("Unknown Project");
            project_sessions
                .entry(project.to_string())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        
        // Create tree structure
        for (project, sessions) in project_sessions {
            let is_expanded = self.expanded_projects.contains(&project);
            let project_icon = if is_expanded { "📂" } else { "📁" };
            
            // Project header
            let project_line = Line::from(vec![
                Span::raw(project_icon),
                Span::raw(" "),
                Span::styled(
                    &project,
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                ),
                Span::raw(format!(" ({} sessions)", sessions.len())),
            ]);
            items.push(ListItem::new(project_line));
            
            // Sessions under project (if expanded)
            if is_expanded {
                for entry in sessions {
                    let icon = self.get_session_type_icon(&entry.session_type);
                    let title = entry.session.title.as_deref().unwrap_or("Untitled Session");
                    let conv_count = entry.conversation_count;
                    
                    let session_line = Line::from(vec![
                        Span::raw("  "),
                        Span::raw(icon),
                        Span::raw(" "),
                        Span::styled(
                            title,
                            Style::default().fg(Color::White)
                        ),
                        Span::raw(format!(" ({} convs)", conv_count)),
                    ]);
                    items.push(ListItem::new(session_line));
                }
            }
        }
        
        items
    }
}

impl UIComponent for SessionTree {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Create layout for search input and list
        let chunks = if self.searching {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Search input
                    Constraint::Min(1),    // Session list
                ])
                .split(area)
        } else {
            vec![area] // Just the session list
        };
        
        let list_area = if self.searching { chunks[1] } else { chunks[0] };
        
        // Render search input if in search mode
        if self.searching {
            let search_block = Block::default()
                .title("Search Sessions")
                .borders(Borders::ALL)
                .style(Style::default().fg(if self.focused { Color::Cyan } else { Color::Gray }));
            
            let search_paragraph = Paragraph::new(self.search_input.value())
                .block(search_block);
            
            frame.render_widget(search_paragraph, chunks[0]);
            
            // Position cursor
            if self.focused {
                frame.set_cursor(
                    chunks[0].x + self.search_input.visual_cursor() as u16 + 1,
                    chunks[0].y + 1,
                );
            }
        }
        
        // Create the main list
        let list_items = self.create_list_items();
        
        let title = match self.view_mode {
            ViewMode::Flat => "Sessions (Flat)",
            ViewMode::ProjectTree => "Sessions (Tree)",
        };
        
        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(
                        if self.focused { Color::Cyan } else { Color::Gray }
                    ))
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            );
        
        frame.render_stateful_widget(list, list_area, &mut self.list_state);
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.searching {
            match key.code {
                KeyCode::Enter => {
                    self.searching = false;
                    true
                }
                KeyCode::Esc => {
                    self.searching = false;
                    self.search_input = Input::default();
                    self.apply_filter();
                    true
                }
                _ => {
                    let request = self.search_input.handle_event(&crossterm::event::Event::Key(key));
                    if matches!(request, Some(InputRequest::Redraw)) {
                        self.apply_filter();
                    }
                    true
                }
            }
        } else {
            match (key.code, key.modifiers) {
                // Navigation
                (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i >= self.filtered_sessions.len().saturating_sub(1) {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                    true
                }
                (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.filtered_sessions.len().saturating_sub(1)
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                    true
                }
                
                // First/Last
                (KeyCode::Home | KeyCode::Char('g'), KeyModifiers::NONE) => {
                    if !self.filtered_sessions.is_empty() {
                        self.list_state.select(Some(0));
                    }
                    true
                }
                (KeyCode::End | KeyCode::Char('G'), KeyModifiers::NONE) => {
                    if !self.filtered_sessions.is_empty() {
                        self.list_state.select(Some(self.filtered_sessions.len() - 1));
                    }
                    true
                }
                
                // Toggle search
                (KeyCode::Char('/'), KeyModifiers::NONE) => {
                    self.toggle_search();
                    true
                }
                
                // Toggle view mode
                (KeyCode::Char('v'), KeyModifiers::NONE) => {
                    self.toggle_view_mode();
                    true
                }
                
                // Expand/collapse in tree mode
                (KeyCode::Right | KeyCode::Char('l'), KeyModifiers::NONE) => {
                    // TODO: Implement project expansion
                    true
                }
                (KeyCode::Left | KeyCode::Char('h'), KeyModifiers::NONE) => {
                    // TODO: Implement project collapse
                    true
                }
                
                _ => false,
            }
        }
    }
    
    fn update(&mut self) {
        // Update any time-based UI elements if needed
    }
}

impl Default for SessionTree {
    fn default() -> Self {
        Self::new()
    }
}