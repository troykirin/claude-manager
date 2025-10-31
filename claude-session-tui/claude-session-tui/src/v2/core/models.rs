//! V2 data models for Shadow Renaissance Architecture

use crate::v2::core::traits::{
    InsightCategory, InsightData, MessageData, MessageRole, SessionData,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// V2 session representation optimized for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2Session {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<V2Message>,
    pub metadata: V2SessionMetadata,
}

impl SessionData for V2Session {
    fn id(&self) -> &str {
        &self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// Optimized message representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2Message {
    pub id: String,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MessageData for V2Message {
    fn content(&self) -> &str {
        &self.content
    }

    fn role(&self) -> MessageRole {
        self.role
    }

    fn timestamp(&self) -> Option<DateTime<Utc>> {
        Some(self.timestamp)
    }
}

/// Session metadata for v2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2SessionMetadata {
    pub project_id: Option<String>,
    pub conversation_id: String,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Optimized insight representation
#[derive(Debug, Clone)]
pub struct V2Insight {
    pub session_id: String,
    pub category: InsightCategory,
    pub confidence: f32,
    pub summary: String,
    pub details: HashMap<String, serde_json::Value>,
}

impl InsightData for V2Insight {
    fn category(&self) -> InsightCategory {
        self.category
    }

    fn confidence(&self) -> f32 {
        self.confidence
    }

    fn summary(&self) -> &str {
        &self.summary
    }
}
