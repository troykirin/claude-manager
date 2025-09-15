//! V2 data models matching Claude's actual JSONL schema
//! 
//! These models accurately represent Claude's session data structure
//! discovered through reverse engineering of the JSONL format.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Claude session representation for v2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSession {
    pub uuid: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ClaudeMessage>,
    pub metadata: SessionMetadata,
}

/// Comprehensive message type matching Claude's format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClaudeMessage {
    #[serde(rename = "human")]
    Human {
        uuid: String,
        content: MessageContent,
        created_at: DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attachments: Option<Vec<Attachment>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        uuid: String,
        content: MessageContent,
        created_at: DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "system")]
    System {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        is_error: bool,
    },
}

/// Message content supporting both text and multimodal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentBlock>),
}

/// Content block for multimodal messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { 
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt_text: Option<String>,
    },
    #[serde(rename = "document")]
    Document {
        source: DocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
}

/// Tool call representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// Attachment for human messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
}

/// Image source for multimodal content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    #[serde(rename = "base64")]
    Base64 {
        media_type: String,
        data: String,
    },
    #[serde(rename = "url")]
    Url { url: String },
}

/// Document source for file attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    pub media_type: String,
    pub data: String,
}

/// Session metadata from Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub project_id: Option<String>,
    pub conversation_id: String,
    pub parent_message_id: Option<String>,
    pub agent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub settings: SessionSettings,
}

/// Session settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub tools_enabled: Vec<String>,
}

// Trait implementations for core traits

impl super::traits::SessionData for ClaudeSession {
    fn id(&self) -> &str {
        &self.uuid
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl super::traits::MessageData for ClaudeMessage {
    fn content(&self) -> &str {
        match self {
            ClaudeMessage::Human { content, .. } |
            ClaudeMessage::Assistant { content, .. } => {
                match content {
                    MessageContent::Text(text) => text,
                    MessageContent::Multimodal(blocks) => {
                        blocks.iter()
                            .filter_map(|b| match b {
                                ContentBlock::Text { text } => Some(text.as_str()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                            .as_str()
                    }
                }
            }
            ClaudeMessage::System { content, .. } => content,
            ClaudeMessage::ToolResult { .. } => "[Tool Result]",
        }
    }
    
    fn role(&self) -> super::traits::MessageRole {
        match self {
            ClaudeMessage::Human { .. } => super::traits::MessageRole::Human,
            ClaudeMessage::Assistant { .. } => super::traits::MessageRole::Assistant,
            ClaudeMessage::System { .. } => super::traits::MessageRole::System,
            ClaudeMessage::ToolResult { .. } => super::traits::MessageRole::Tool,
        }
    }
    
    fn timestamp(&self) -> Option<DateTime<Utc>> {
        match self {
            ClaudeMessage::Human { created_at, .. } |
            ClaudeMessage::Assistant { created_at, .. } => Some(*created_at),
            _ => None,
        }
    }
}