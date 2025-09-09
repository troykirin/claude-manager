//! Conversation viewer with syntax highlighting and real-time interaction
//! 
//! Displays conversation blocks with role-based visual differentiation,
//! syntax highlighting for code blocks, and smooth navigation with vim-style controls.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, 
        ScrollbarOrientation, ScrollbarState, Wrap, Clear,
    },
    Frame,
};
use syntect::{
    highlighting::{Style as SyntectStyle, ThemeSet},
    parsing::{SyntaxSet, SyntaxReference},
    util::LinesWithEndings,
};
use unicode_width::UnicodeWidthStr;

use crate::models::{Conversation, Block, ToolCall, ToolResult};

use super::UIComponent;

/// Conversation viewer component
pub struct ConversationViewer {
    /// Current conversation being displayed
    conversation: Option<Conversation>,
    /// Rendered content blocks for display
    rendered_blocks: Vec<RenderedBlock>,
    /// Current scroll position
    scroll_position: usize,
    /// Current line within the conversation
    current_line: usize,
    /// Whether the component is focused
    focused: bool,
    /// Scrollbar state
    scrollbar_state: ScrollbarState,
    /// Search highlights
    search_highlights: Vec<SearchHighlight>,
    /// Syntax highlighting components
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    /// Current highlight mode
    highlight_mode: HighlightMode,
    /// Viewport height for scrolling calculations
    viewport_height: usize,
    /// Tool results expansion state
    expanded_tools: std::collections::HashSet<uuid::Uuid>,
    /// Performance metrics
    render_stats: RenderStats,
}

/// Rendered block with styling and layout information
#[derive(Debug, Clone)]
pub struct RenderedBlock {
    pub block_id: uuid::Uuid,
    pub role: String,
    pub lines: Vec<RenderedLine>,
    pub line_start: usize,
    pub line_end: usize,
    pub block_type: BlockType,
    pub is_expanded: bool,
}

/// Individual rendered line within a block
#[derive(Debug, Clone)]
pub struct RenderedLine {
    pub content: Text<'static>,
    pub line_number: usize,
    pub block_id: uuid::Uuid,
    pub is_highlighted: bool,
    pub indent_level: usize,
}

/// Search highlight information
#[derive(Debug, Clone)]
pub struct SearchHighlight {
    pub line_number: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub highlight_type: HighlightType,
}

/// Different types of content blocks
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    User,
    Assistant,
    Tool,
    ToolResult,
    Code,
    Error,
}

/// Syntax highlighting modes
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightMode {
    Full,     // Full syntax highlighting
    Minimal,  // Basic role-based highlighting
    Off,      // No highlighting
}

/// Highlight types for search and special content
#[derive(Debug, Clone, PartialEq)]
pub enum HighlightType {
    Search,
    GoldenNugget,
    Insight,
    Tool,
    Error,
}

/// Rendering performance statistics
#[derive(Debug, Default)]
pub struct RenderStats {
    pub total_lines: usize,
    pub visible_lines: usize,
    pub syntax_highlight_time: std::time::Duration,
    pub layout_time: std::time::Duration,
}

impl ConversationViewer {
    /// Create a new conversation viewer
    pub fn new() -> Self {
        Self {
            conversation: None,
            rendered_blocks: Vec::new(),
            scroll_position: 0,
            current_line: 0,
            focused: false,
            scrollbar_state: ScrollbarState::new(0),
            search_highlights: Vec::new(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            highlight_mode: HighlightMode::Full,
            viewport_height: 0,
            expanded_tools: std::collections::HashSet::new(),
            render_stats: RenderStats::default(),
        }
    }
    
    /// Load a conversation for viewing
    pub fn load_conversation(&mut self, conversation: Conversation) {
        self.conversation = Some(conversation);
        self.render_conversation();
        self.scroll_position = 0;
        self.current_line = 0;
        self.update_scrollbar();
    }
    
    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    
    /// Get current line number for marker placement
    pub fn get_current_line(&self) -> usize {
        self.current_line
    }
    
    /// Add search highlights
    pub fn add_search_highlights(&mut self, highlights: Vec<SearchHighlight>) {
        self.search_highlights = highlights;
        self.update_line_highlights();
    }
    
    /// Clear search highlights
    pub fn clear_search_highlights(&mut self) {
        self.search_highlights.clear();
        self.update_line_highlights();
    }
    
    /// Toggle syntax highlighting mode
    pub fn toggle_highlight_mode(&mut self) {
        self.highlight_mode = match self.highlight_mode {
            HighlightMode::Full => HighlightMode::Minimal,
            HighlightMode::Minimal => HighlightMode::Off,
            HighlightMode::Off => HighlightMode::Full,
        };
        self.render_conversation();
    }
    
    /// Render the current conversation into display blocks
    fn render_conversation(&mut self) {
        let render_start = std::time::Instant::now();
        
        if let Some(ref conversation) = self.conversation {
            self.rendered_blocks.clear();
            let mut current_line = 0;
            
            for block in &conversation.blocks {
                let rendered_block = self.render_block(block, current_line);
                current_line = rendered_block.line_end + 1;
                self.rendered_blocks.push(rendered_block);
            }
            
            self.render_stats.total_lines = current_line;
        }
        
        self.render_stats.layout_time = render_start.elapsed();
        self.update_scrollbar();
    }
    
    /// Render a single block into a rendered block
    fn render_block(&self, block: &Block, start_line: usize) -> RenderedBlock {
        let block_type = self.classify_block(block);
        let mut lines = Vec::new();
        let mut current_line = start_line;
        
        // Render block header with role and metadata
        let header_line = self.create_block_header(block, current_line);
        lines.push(header_line);
        current_line += 1;
        
        // Render main content
        if block.content.contains("```") {
            // Handle code blocks with syntax highlighting
            let code_lines = self.render_code_content(&block.content, current_line, block.id);
            current_line += code_lines.len();
            lines.extend(code_lines);
        } else {
            // Render regular text content
            let content_lines = self.render_text_content(&block.content, current_line, block.id);
            current_line += content_lines.len();
            lines.extend(content_lines);
        }
        
        // Render tool calls if present
        if !block.tool_calls.is_empty() {
            let tool_lines = self.render_tool_calls(&block.tool_calls, current_line, block.id);
            current_line += tool_lines.len();
            lines.extend(tool_lines);
        }
        
        // Render tool results if present
        if !block.tool_results.is_empty() {
            let result_lines = self.render_tool_results(&block.tool_results, current_line, block.id);
            current_line += result_lines.len();
            lines.extend(result_lines);
        }
        
        // Add spacing line
        lines.push(RenderedLine {
            content: Text::from(""),
            line_number: current_line,
            block_id: block.id,
            is_highlighted: false,
            indent_level: 0,
        });
        current_line += 1;
        
        RenderedBlock {
            block_id: block.id,
            role: block.role.clone(),
            lines,
            line_start: start_line,
            line_end: current_line - 1,
            block_type,
            is_expanded: true, // TODO: Implement collapsing
        }
    }
    
    /// Create a block header line with role and metadata
    fn create_block_header(&self, block: &Block, line_number: usize) -> RenderedLine {
        let role_icon = match block.role.as_str() {
            "user" => "👤",
            "assistant" => "🤖",
            "tool" => "🔧",
            _ => "❓",
        };
        
        let role_color = match block.role.as_str() {
            "user" => Color::Cyan,
            "assistant" => Color::Green,
            "tool" => Color::Yellow,
            _ => Color::Gray,
        };
        
        let timestamp = block.timestamp.format("%H:%M:%S").to_string();
        let tokens = block.tokens.map(|t| format!(" ({}t)", t)).unwrap_or_default();
        
        let content = Text::from(vec![
            Line::from(vec![
                Span::raw(role_icon),
                Span::raw(" "),
                Span::styled(
                    format!("{}{}", block.role.to_uppercase(), tokens),
                    Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" • "),
                Span::styled(timestamp, Style::default().fg(Color::Gray)),
            ])
        ]);
        
        RenderedLine {
            content,
            line_number,
            block_id: block.id,
            is_highlighted: false,
            indent_level: 0,
        }
    }
    
    /// Render code content with syntax highlighting
    fn render_code_content(&self, content: &str, start_line: usize, block_id: uuid::Uuid) -> Vec<RenderedLine> {
        if self.highlight_mode == HighlightMode::Off {
            return self.render_text_content(content, start_line, block_id);
        }
        
        let highlight_start = std::time::Instant::now();
        let mut lines = Vec::new();
        let mut current_line = start_line;
        let mut in_code_block = false;
        let mut current_syntax: Option<&SyntaxReference> = None;
        
        for line in content.lines() {
            if line.starts_with("```") {
                if !in_code_block {
                    // Starting a code block
                    in_code_block = true;
                    let language = line.strip_prefix("```").unwrap_or("").trim();
                    current_syntax = if !language.is_empty() {
                        self.syntax_set.find_syntax_by_token(language)
                    } else {
                        None
                    };
                    
                    // Render code block start
                    let content = Text::from(Line::from(vec![
                        Span::styled("```", Style::default().fg(Color::DarkGray)),
                        Span::styled(language, Style::default().fg(Color::Blue)),
                    ]));
                    
                    lines.push(RenderedLine {
                        content,
                        line_number: current_line,
                        block_id,
                        is_highlighted: false,
                        indent_level: 0,
                    });
                } else {
                    // Ending a code block
                    in_code_block = false;
                    current_syntax = None;
                    
                    let content = Text::from(Line::from(vec![
                        Span::styled("```", Style::default().fg(Color::DarkGray)),
                    ]));
                    
                    lines.push(RenderedLine {
                        content,
                        line_number: current_line,
                        block_id,
                        is_highlighted: false,
                        indent_level: 0,
                    });
                }
            } else if in_code_block && self.highlight_mode == HighlightMode::Full {
                // Apply syntax highlighting
                let highlighted_line = if let Some(syntax) = current_syntax {
                    self.apply_syntax_highlighting(line, syntax)
                } else {
                    Text::from(Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]))
                };
                
                lines.push(RenderedLine {
                    content: highlighted_line,
                    line_number: current_line,
                    block_id,
                    is_highlighted: false,
                    indent_level: 1,
                });
            } else {
                // Regular text line
                let content = if in_code_block {
                    Text::from(Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]))
                } else {
                    Text::from(line.to_string())
                };
                
                lines.push(RenderedLine {
                    content,
                    line_number: current_line,
                    block_id,
                    is_highlighted: false,
                    indent_level: if in_code_block { 1 } else { 0 },
                });
            }
            
            current_line += 1;
        }
        
        // Update performance stats
        // self.render_stats.syntax_highlight_time += highlight_start.elapsed();
        
        lines
    }
    
    /// Render regular text content
    fn render_text_content(&self, content: &str, start_line: usize, block_id: uuid::Uuid) -> Vec<RenderedLine> {
        let mut lines = Vec::new();
        let mut current_line = start_line;
        
        for line in content.lines() {
            let content = Text::from(line.to_string());
            
            lines.push(RenderedLine {
                content,
                line_number: current_line,
                block_id,
                is_highlighted: false,
                indent_level: 0,
            });
            
            current_line += 1;
        }
        
        // Handle empty content
        if lines.is_empty() {
            lines.push(RenderedLine {
                content: Text::from(""),
                line_number: current_line,
                block_id,
                is_highlighted: false,
                indent_level: 0,
            });
        }
        
        lines
    }
    
    /// Apply syntax highlighting to a code line
    fn apply_syntax_highlighting(&self, line: &str, syntax: &SyntaxReference) -> Text<'static> {
        // Simplified syntax highlighting - in practice you'd use syntect properly
        // This is a mock implementation for now
        let styled_line = if line.trim_start().starts_with("//") || line.trim_start().starts_with("#") {
            Line::from(vec![
                Span::styled(line, Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)),
            ])
        } else if line.contains("fn ") || line.contains("def ") || line.contains("function") {
            Line::from(vec![
                Span::styled(line, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ])
        } else if line.contains("\"") || line.contains("'") {
            Line::from(vec![
                Span::styled(line, Style::default().fg(Color::Magenta)),
            ])
        } else {
            Line::from(vec![
                Span::styled(line, Style::default().fg(Color::White)),
            ])
        };
        
        Text::from(vec![styled_line])
    }
    
    /// Render tool calls
    fn render_tool_calls(&self, tool_calls: &[ToolCall], start_line: usize, block_id: uuid::Uuid) -> Vec<RenderedLine> {
        let mut lines = Vec::new();
        let mut current_line = start_line;
        
        for tool_call in tool_calls {
            // Tool call header
            let header_content = Text::from(Line::from(vec![
                Span::raw("🔧 "),
                Span::styled(
                    format!("Tool: {}", tool_call.name),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
            
            lines.push(RenderedLine {
                content: header_content,
                line_number: current_line,
                block_id,
                is_highlighted: false,
                indent_level: 1,
            });
            current_line += 1;
            
            // Tool arguments (collapsed by default)
            if self.expanded_tools.contains(&block_id) {
                let args_text = serde_json::to_string_pretty(&tool_call.arguments)
                    .unwrap_or_else(|_| "Invalid JSON".to_string());
                
                for line in args_text.lines() {
                    let content = Text::from(Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::Gray)),
                    ]));
                    
                    lines.push(RenderedLine {
                        content,
                        line_number: current_line,
                        block_id,
                        is_highlighted: false,
                        indent_level: 2,
                    });
                    current_line += 1;
                }
            }
        }
        
        lines
    }
    
    /// Render tool results
    fn render_tool_results(&self, tool_results: &[ToolResult], start_line: usize, block_id: uuid::Uuid) -> Vec<RenderedLine> {
        let mut lines = Vec::new();
        let mut current_line = start_line;
        
        for result in tool_results {
            // Result header
            let header_content = Text::from(Line::from(vec![
                Span::raw("📋 "),
                Span::styled(
                    "Tool Result",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                if result.is_error.unwrap_or(false) {
                    Span::styled(" (Error)", Style::default().fg(Color::Red))
                } else {
                    Span::raw("")
                },
            ]));
            
            lines.push(RenderedLine {
                content: header_content,
                line_number: current_line,
                block_id,
                is_highlighted: false,
                indent_level: 1,
            });
            current_line += 1;
            
            // Result content
            for line in result.content.lines() {
                let style = if result.is_error.unwrap_or(false) {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::White)
                };
                
                let content = Text::from(Line::from(vec![
                    Span::styled(line, style),
                ]));
                
                lines.push(RenderedLine {
                    content,
                    line_number: current_line,
                    block_id,
                    is_highlighted: false,
                    indent_level: 2,
                });
                current_line += 1;
            }
        }
        
        lines
    }
    
    /// Classify a block's type for styling
    fn classify_block(&self, block: &Block) -> BlockType {
        match block.role.as_str() {
            "user" => BlockType::User,
            "assistant" => BlockType::Assistant,
            "tool" => BlockType::Tool,
            _ => {
                if !block.tool_results.is_empty() {
                    BlockType::ToolResult
                } else if block.content.contains("```") {
                    BlockType::Code
                } else {
                    BlockType::Assistant
                }
            }
        }
    }
    
    /// Update scrollbar state
    fn update_scrollbar(&mut self) {
        let total_lines = self.render_stats.total_lines;
        self.scrollbar_state = self.scrollbar_state.content_length(total_lines);
        self.scrollbar_state = self.scrollbar_state.position(self.scroll_position);
    }
    
    /// Update line highlights based on search results
    fn update_line_highlights(&mut self) {
        // Reset all highlights
        for block in &mut self.rendered_blocks {
            for line in &mut block.lines {
                line.is_highlighted = false;
            }
        }
        
        // Apply search highlights
        for highlight in &self.search_highlights {
            for block in &mut self.rendered_blocks {
                for line in &mut block.lines {
                    if line.line_number == highlight.line_number {
                        line.is_highlighted = true;
                    }
                }
            }
        }
    }
    
    /// Get the visible lines for rendering
    fn get_visible_lines(&self, viewport_height: usize) -> Vec<&RenderedLine> {
        let end_line = (self.scroll_position + viewport_height).min(self.render_stats.total_lines);
        
        self.rendered_blocks
            .iter()
            .flat_map(|block| &block.lines)
            .filter(|line| {
                line.line_number >= self.scroll_position && line.line_number < end_line
            })
            .collect()
    }
    
    /// Scroll to a specific line
    pub fn scroll_to_line(&mut self, line: usize) {
        self.current_line = line.min(self.render_stats.total_lines.saturating_sub(1));
        
        // Adjust scroll position to keep current line visible
        let viewport_center = self.viewport_height / 2;
        if self.current_line >= viewport_center {
            self.scroll_position = (self.current_line - viewport_center).min(
                self.render_stats.total_lines.saturating_sub(self.viewport_height)
            );
        } else {
            self.scroll_position = 0;
        }
        
        self.update_scrollbar();
    }
}

impl UIComponent for ConversationViewer {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.viewport_height = area.height as usize;
        
        if self.conversation.is_none() {
            // Show empty state
            let empty_text = Paragraph::new("No conversation selected\n\nPress Tab to navigate to session tree")
                .block(
                    Block::default()
                        .title("Conversation")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Gray))
                )
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(empty_text, area);
            return;
        }
        
        // Create layout for conversation content and scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),      // Content
                Constraint::Length(1),   // Scrollbar
            ])
            .split(area);
        
        let content_area = chunks[0];
        let scrollbar_area = chunks[1];
        
        // Get visible lines
        let visible_lines = self.get_visible_lines(content_area.height as usize);
        self.render_stats.visible_lines = visible_lines.len();
        
        // Create list items from visible lines
        let list_items: Vec<ListItem> = visible_lines
            .iter()
            .map(|line| {
                let indent = "  ".repeat(line.indent_level);
                let mut content = line.content.clone();
                
                // Add indent to the first span
                if let Some(first_line) = content.lines.first_mut() {
                    first_line.spans.insert(0, Span::raw(indent));
                }
                
                // Apply highlight if needed
                if line.is_highlighted {
                    for content_line in &mut content.lines {
                        for span in &mut content_line.spans {
                            span.style = span.style.bg(Color::DarkGray);
                        }
                    }
                }
                
                ListItem::new(content)
            })
            .collect();
        
        // Create the conversation list
        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(format!(
                        "Conversation {} (Line {}/{})",
                        self.conversation.as_ref()
                            .and_then(|c| c.title.as_deref())
                            .unwrap_or("Untitled"),
                        self.current_line + 1,
                        self.render_stats.total_lines
                    ))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(
                        if self.focused { Color::Cyan } else { Color::Gray }
                    ))
            );
        
        frame.render_widget(list, content_area);
        
        // Render scrollbar
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::DarkGray))
            .thumb_style(Style::default().fg(Color::Gray));
        
        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut self.scrollbar_state);
        
        // Render current line indicator if focused
        if self.focused && !visible_lines.is_empty() {
            let indicator_y = content_area.y + 1 + 
                (self.current_line.saturating_sub(self.scroll_position)) as u16;
            
            if indicator_y < content_area.y + content_area.height - 1 {
                frame.render_widget(
                    Block::default().style(Style::default().bg(Color::DarkGray)),
                    Rect {
                        x: content_area.x,
                        y: indicator_y,
                        width: content_area.width,
                        height: 1,
                    },
                );
            }
        }
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            // Vertical navigation
            (KeyCode::Down | KeyCode::Char('j'), KeyModifiers::NONE) => {
                if self.current_line < self.render_stats.total_lines.saturating_sub(1) {
                    self.current_line += 1;
                    
                    // Auto-scroll if needed
                    if self.current_line >= self.scroll_position + self.viewport_height {
                        self.scroll_position += 1;
                        self.update_scrollbar();
                    }
                }
                true
            }
            (KeyCode::Up | KeyCode::Char('k'), KeyModifiers::NONE) => {
                if self.current_line > 0 {
                    self.current_line -= 1;
                    
                    // Auto-scroll if needed
                    if self.current_line < self.scroll_position {
                        self.scroll_position = self.scroll_position.saturating_sub(1);
                        self.update_scrollbar();
                    }
                }
                true
            }
            
            // Page navigation
            (KeyCode::PageDown | KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                let page_size = self.viewport_height.saturating_sub(1);
                self.current_line = (self.current_line + page_size)
                    .min(self.render_stats.total_lines.saturating_sub(1));
                self.scroll_position = (self.scroll_position + page_size)
                    .min(self.render_stats.total_lines.saturating_sub(self.viewport_height));
                self.update_scrollbar();
                true
            }
            (KeyCode::PageUp | KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                let page_size = self.viewport_height.saturating_sub(1);
                self.current_line = self.current_line.saturating_sub(page_size);
                self.scroll_position = self.scroll_position.saturating_sub(page_size);
                self.update_scrollbar();
                true
            }
            
            // Home/End
            (KeyCode::Home | KeyCode::Char('g'), KeyModifiers::NONE) => {
                self.current_line = 0;
                self.scroll_position = 0;
                self.update_scrollbar();
                true
            }
            (KeyCode::End | KeyCode::Char('G'), KeyModifiers::NONE) => {
                self.current_line = self.render_stats.total_lines.saturating_sub(1);
                self.scroll_position = self.render_stats.total_lines
                    .saturating_sub(self.viewport_height);
                self.update_scrollbar();
                true
            }
            
            // Toggle highlighting mode
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.toggle_highlight_mode();
                true
            }
            
            // Toggle tool expansion
            (KeyCode::Char('t'), KeyModifiers::NONE) => {
                // Find current block and toggle tool expansion
                if let Some(block) = self.rendered_blocks.iter()
                    .find(|b| self.current_line >= b.line_start && self.current_line <= b.line_end)
                {
                    if self.expanded_tools.contains(&block.block_id) {
                        self.expanded_tools.remove(&block.block_id);
                    } else {
                        self.expanded_tools.insert(block.block_id);
                    }
                    self.render_conversation();
                }
                true
            }
            
            _ => false,
        }
    }
    
    fn update(&mut self) {
        // Update any time-based elements if needed
    }
}

impl Default for ConversationViewer {
    fn default() -> Self {
        Self::new()
    }
}