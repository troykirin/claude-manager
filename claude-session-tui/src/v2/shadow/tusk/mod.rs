//! Tusk - UI Virtualization Shadow Agent
//!
//! Specializes in virtual scrolling, render caching, and 60fps UI performance
//! with intelligent viewport management and differential updates.

use crate::v2::core::traits::{AsyncRenderer, PerformanceMetrics, ShadowAgent};
use crate::v2::shadow::iron::IronInsight;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tusk's render context for TUI
#[derive(Debug, Clone)]
pub struct TuskRenderContext {
    pub viewport: Rect,
    pub focused: bool,
    pub theme: TuskTheme,
    pub virtualization_enabled: bool,
    pub max_visible_items: usize,
}

#[derive(Debug, Clone)]
pub struct TuskTheme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub accent: Color,
    pub text: Color,
    pub selection: Color,
}

impl Default for TuskTheme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            background: Color::Black,
            accent: Color::Yellow,
            text: Color::White,
            selection: Color::LightBlue,
        }
    }
}

/// Virtual list item for efficient rendering
#[derive(Debug, Clone)]
pub struct VirtualListItem<T> {
    pub id: String,
    pub data: T,
    pub cached_render: Option<Vec<Line<'static>>>,
    pub dirty: bool,
    pub height: u16,
}

/// Virtual scrolling list with caching
pub struct TuskVirtualList<T> {
    items: Vec<VirtualListItem<T>>,
    viewport_start: usize,
    viewport_size: usize,
    scroll_offset: usize,
    total_height: u16,
    cache: Arc<RwLock<TuskRenderCache>>,
}

impl<T> TuskVirtualList<T> {
    pub fn new(max_viewport_size: usize) -> Self {
        Self {
            items: Vec::new(),
            viewport_start: 0,
            viewport_size: max_viewport_size,
            scroll_offset: 0,
            total_height: 0,
            cache: Arc::new(RwLock::new(TuskRenderCache::new(1000))),
        }
    }

    pub fn set_items(&mut self, items: Vec<T>, item_renderer: impl Fn(&T) -> VirtualListItem<T>) {
        self.items = items.into_iter().map(item_renderer).collect();
        self.total_height = self.items.iter().map(|item| item.height).sum();
        self.update_viewport();
    }

    pub fn scroll_to(&mut self, index: usize) {
        self.scroll_offset = index.min(self.items.len().saturating_sub(1));
        self.update_viewport();
    }

    pub fn scroll_by(&mut self, delta: i32) {
        if delta > 0 {
            self.scroll_offset =
                (self.scroll_offset + delta as usize).min(self.items.len().saturating_sub(1));
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub((-delta) as usize);
        }
        self.update_viewport();
    }

    pub fn visible_items(&self) -> &[VirtualListItem<T>] {
        let end = (self.viewport_start + self.viewport_size).min(self.items.len());
        &self.items[self.viewport_start..end]
    }

    fn update_viewport(&mut self) {
        // Calculate which items are visible based on scroll position
        let mut current_height = 0u16;
        let mut start_found = false;

        for (idx, item) in self.items.iter().enumerate() {
            if !start_found && current_height >= self.scroll_offset as u16 {
                self.viewport_start = idx;
                start_found = true;
            }

            current_height += item.height;

            if start_found && (idx - self.viewport_start) >= self.viewport_size {
                break;
            }
        }
    }
}

/// High-performance UI renderer with virtualization
pub struct TuskRenderer {
    frame_limiter: TuskFrameLimiter,
    render_cache: Arc<RwLock<TuskRenderCache>>,
    virtual_lists: Arc<RwLock<std::collections::HashMap<String, TuskVirtualList<IronInsight>>>>,
    performance_monitor: TuskPerformanceMonitor,
}

impl TuskRenderer {
    pub fn new() -> Self {
        Self {
            frame_limiter: TuskFrameLimiter::new(60), // 60 FPS target
            render_cache: Arc::new(RwLock::new(TuskRenderCache::new(500))),
            virtual_lists: Arc::new(RwLock::new(std::collections::HashMap::new())),
            performance_monitor: TuskPerformanceMonitor::new(),
        }
    }

    /// Render insights with virtual scrolling
    pub async fn render_insights(
        &self,
        insights: &[IronInsight],
        context: &TuskRenderContext,
    ) -> Result<String, TuskError> {
        let start_time = std::time::Instant::now();

        // Check if we need to limit frame rate
        self.frame_limiter.wait_for_next_frame().await;

        // Create virtual list if it doesn't exist
        let list_id = "main_insights".to_string();
        {
            let mut lists = self.virtual_lists.write().await;
            if !lists.contains_key(&list_id) {
                lists.insert(
                    list_id.clone(),
                    TuskVirtualList::new(context.max_visible_items),
                );
            }
        }

        // Update virtual list with new data
        {
            let mut lists = self.virtual_lists.write().await;
            if let Some(list) = lists.get_mut(&list_id) {
                list.set_items(insights.to_vec(), |insight| VirtualListItem {
                    id: insight.session_id.clone(),
                    data: insight.clone(),
                    cached_render: None,
                    dirty: true,
                    height: self.calculate_item_height(insight),
                });
            }
        }

        // Render visible items with caching
        let rendered_content = self.render_virtual_list(&list_id, context).await?;

        // Record performance metrics
        let render_time = start_time.elapsed();
        self.performance_monitor.record_render_time(render_time);

        Ok(rendered_content)
    }

    async fn render_virtual_list(
        &self,
        list_id: &str,
        context: &TuskRenderContext,
    ) -> Result<String, TuskError> {
        let lists = self.virtual_lists.read().await;
        let list = lists
            .get(list_id)
            .ok_or_else(|| TuskError::ListNotFound(list_id.to_string()))?;

        let visible_items = list.visible_items();
        let mut output = String::new();

        // Render header
        output.push_str(&format!("┌─ Insights ({}) ─┐\n", visible_items.len()));

        for (idx, item) in visible_items.iter().enumerate() {
            let rendered_item = self
                .render_insight_item(&item.data, context, idx == 0)
                .await?;
            output.push_str(&rendered_item);
            output.push('\n');
        }

        // Render footer with scroll info
        output.push_str(&format!(
            "└─ Scroll: {}/{} ─┘",
            list.scroll_offset,
            list.items.len()
        ));

        Ok(output)
    }

    async fn render_insight_item(
        &self,
        insight: &IronInsight,
        context: &TuskRenderContext,
        is_selected: bool,
    ) -> Result<String, TuskError> {
        // Check cache first
        let cache_key = format!("{}_{}", insight.session_id, insight.computed_at.timestamp());
        {
            let cache = self.render_cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(self.apply_selection_style(cached.clone(), is_selected, context));
            }
        }

        // Render fresh content
        let mut content = String::new();

        // Category icon
        let category_icon = match insight.category {
            crate::v2::core::traits::InsightCategory::Topic => "📝",
            crate::v2::core::traits::InsightCategory::Sentiment => "😊",
            crate::v2::core::traits::InsightCategory::Complexity => "🧠",
            crate::v2::core::traits::InsightCategory::ToolUsage => "🔧",
            crate::v2::core::traits::InsightCategory::ErrorPattern => "⚠️",
            crate::v2::core::traits::InsightCategory::Performance => "⚡",
        };

        // Format insight line
        content.push_str(&format!(
            "{} {} | {} | {:.0}%",
            category_icon,
            insight.summary,
            insight.session_id[..8].to_string(),
            insight.confidence * 100.0
        ));

        // Cache the rendered content
        {
            let mut cache = self.render_cache.write().await;
            cache.insert(cache_key, content.clone());
        }

        Ok(self.apply_selection_style(content, is_selected, context))
    }

    fn apply_selection_style(
        &self,
        content: String,
        is_selected: bool,
        context: &TuskRenderContext,
    ) -> String {
        if is_selected && context.focused {
            format!("▶ {}", content) // Selection indicator
        } else {
            format!("  {}", content) // Normal padding
        }
    }

    fn calculate_item_height(&self, _insight: &IronInsight) -> u16 {
        // Simple height calculation - could be more sophisticated
        1 // Single line per insight
    }

    /// Handle keyboard input for navigation
    pub async fn handle_input(&self, list_id: &str, key: TuskKey) -> Result<(), TuskError> {
        let mut lists = self.virtual_lists.write().await;
        let list = lists
            .get_mut(list_id)
            .ok_or_else(|| TuskError::ListNotFound(list_id.to_string()))?;

        match key {
            TuskKey::Up => list.scroll_by(-1),
            TuskKey::Down => list.scroll_by(1),
            TuskKey::PageUp => list.scroll_by(-(list.viewport_size as i32)),
            TuskKey::PageDown => list.scroll_by(list.viewport_size as i32),
            TuskKey::Home => list.scroll_to(0),
            TuskKey::End => list.scroll_to(list.items.len().saturating_sub(1)),
        }

        Ok(())
    }
}

#[async_trait]
impl AsyncRenderer<IronInsight> for TuskRenderer {
    type Error = TuskError;
    type RenderContext = TuskRenderContext;

    async fn render(
        &self,
        data: &IronInsight,
        context: &Self::RenderContext,
    ) -> Result<(), Self::Error> {
        let insights = vec![data.clone()];
        let _rendered = self.render_insights(&insights, context).await?;
        // In a real implementation, this would update the terminal
        Ok(())
    }

    async fn stream_render<S>(
        &self,
        mut data_stream: S,
        context: &Self::RenderContext,
    ) -> Result<(), Self::Error>
    where
        S: Stream<Item = IronInsight> + Send + Unpin,
    {
        let mut frame_count = 0;
        let mut insights_buffer = Vec::new();

        while let Some(insight) = data_stream.next().await {
            insights_buffer.push(insight);
            frame_count += 1;

            // Render every N insights or on a timer
            if frame_count >= 10 || insights_buffer.len() >= context.max_visible_items {
                let _rendered = self.render_insights(&insights_buffer, context).await?;
                insights_buffer.clear();
                frame_count = 0;

                // Frame rate limiting
                self.frame_limiter.wait_for_next_frame().await;
            }
        }

        // Render any remaining insights
        if !insights_buffer.is_empty() {
            let _rendered = self.render_insights(&insights_buffer, context).await?;
        }

        Ok(())
    }

    fn supports_virtualization(&self) -> bool {
        true
    }

    fn target_fps(&self) -> u32 {
        self.frame_limiter.target_fps
    }
}

impl ShadowAgent for TuskRenderer {
    const NAME: &'static str = "Tusk";
    const SPECIALIZATION: &'static str = "UI Virtualization";

    fn performance_targets(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            parse_duration: std::time::Duration::from_millis(16), // 60 FPS = 16ms per frame
            memory_usage: 30 * 1024 * 1024,                       // 30MB for UI caching
            throughput: 1000.0,                                   // 1000 items rendered per second
            error_rate: 0.001,                                    // 0.1% error tolerance for UI
        }
    }
}

/// Frame rate limiter for 60 FPS
pub struct TuskFrameLimiter {
    target_fps: u32,
    frame_duration: std::time::Duration,
    last_frame: std::time::Instant,
}

impl TuskFrameLimiter {
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            frame_duration: std::time::Duration::from_nanos(1_000_000_000 / target_fps as u64),
            last_frame: std::time::Instant::now(),
        }
    }

    pub async fn wait_for_next_frame(&mut self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_duration {
            let wait_time = self.frame_duration - elapsed;
            tokio::time::sleep(wait_time).await;
        }
        self.last_frame = std::time::Instant::now();
    }
}

/// Render cache for Tusk
pub struct TuskRenderCache {
    cache: std::collections::HashMap<String, String>,
    access_order: VecDeque<String>,
    max_size: usize,
}

impl TuskRenderCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        if self.cache.contains_key(key) {
            // Move to front for LRU
            self.access_order.retain(|k| k != key);
            self.access_order.push_front(key.to_string());
            self.cache.get(key)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        if self.cache.len() >= self.max_size {
            // Remove LRU item
            if let Some(lru_key) = self.access_order.pop_back() {
                self.cache.remove(&lru_key);
            }
        }

        self.cache.insert(key.clone(), value);
        self.access_order.push_front(key);
    }
}

/// Performance monitoring for Tusk
pub struct TuskPerformanceMonitor {
    render_times: VecDeque<std::time::Duration>,
    max_samples: usize,
}

impl TuskPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            render_times: VecDeque::new(),
            max_samples: 100,
        }
    }

    pub fn record_render_time(&mut self, duration: std::time::Duration) {
        if self.render_times.len() >= self.max_samples {
            self.render_times.pop_front();
        }
        self.render_times.push_back(duration);
    }

    pub fn average_render_time(&self) -> std::time::Duration {
        if self.render_times.is_empty() {
            return std::time::Duration::from_millis(0);
        }

        let total: std::time::Duration = self.render_times.iter().sum();
        total / self.render_times.len() as u32
    }

    pub fn current_fps(&self) -> f64 {
        let avg_time = self.average_render_time();
        if avg_time.is_zero() {
            return 0.0;
        }

        1.0 / avg_time.as_secs_f64()
    }
}

/// Keyboard input for Tusk
#[derive(Debug, Clone, Copy)]
pub enum TuskKey {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

/// Tusk-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TuskError {
    #[error("Render failed: {0}")]
    RenderFailed(String),
    #[error("List not found: {0}")]
    ListNotFound(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Frame rate exceeded target")]
    FrameRateExceeded,
}
