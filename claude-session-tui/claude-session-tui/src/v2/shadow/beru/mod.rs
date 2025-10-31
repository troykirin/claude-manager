//! Beru - Data Model Archaeology Shadow Agent
//!
//! Specializes in reverse-engineering and implementing accurate Claude JSONL data models
//! through comprehensive schema analysis and validation.

use crate::v2::core::traits::{
    InsightCategory, MessageData, MessageRole, PerformanceMetrics, SessionData, ShadowAgent,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Beru's Claude session model based on reverse-engineered schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeruSession {
    pub uuid: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<BeruMessage>,
    pub project_uuid: Option<String>,
    pub conversation_id: String,
    pub metadata: BeruSessionMetadata,
}

impl SessionData for BeruSession {
    fn id(&self) -> &str {
        &self.uuid
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// Comprehensive message type matching Claude's actual format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeruMessage {
    pub uuid: String,
    pub content: BeruMessageContent,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub parent_uuid: Option<String>,
    pub is_active: bool,
    pub attachments: Vec<BeruAttachment>,
    pub tool_calls: Option<Vec<BeruToolCall>>,
    pub usage: Option<BeruUsage>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MessageData for BeruMessage {
    fn content(&self) -> &str {
        match &self.content {
            BeruMessageContent::Text(text) => text,
            BeruMessageContent::Multimodal(blocks) => {
                // Return first text block for simplified interface
                blocks
                    .iter()
                    .find_map(|block| match block {
                        BeruContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .unwrap_or("[Multimodal Content]")
            }
        }
    }

    fn role(&self) -> MessageRole {
        match self.role.as_str() {
            "human" | "user" => MessageRole::Human,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            "tool" => MessageRole::Tool,
            _ => MessageRole::Human, // Default fallback
        }
    }

    fn timestamp(&self) -> Option<DateTime<Utc>> {
        Some(self.created_at)
    }
}

/// Message content supporting both text and multimodal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BeruMessageContent {
    Text(String),
    Multimodal(Vec<BeruContentBlock>),
}

/// Content block for multimodal messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BeruContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: BeruImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        alt_text: Option<String>,
    },
    #[serde(rename = "document")]
    Document {
        source: BeruDocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
}

/// Tool call representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeruToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// Attachment for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeruAttachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Image source for multimodal content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BeruImageSource {
    #[serde(rename = "base64")]
    Base64 { media_type: String, data: String },
    #[serde(rename = "url")]
    Url { url: String },
}

/// Document source for file attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeruDocumentSource {
    pub media_type: String,
    pub data: String,
}

/// Usage statistics for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeruUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}

/// Session metadata from Claude
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeruSessionMetadata {
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools_enabled: Vec<String>,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Beru Shadow Agent - Data Model Archaeologist
pub struct BeruAgent {
    schema_cache: HashMap<String, serde_json::Value>,
    validation_enabled: bool,
}

impl BeruAgent {
    pub fn new() -> Self {
        Self {
            schema_cache: HashMap::new(),
            validation_enabled: true,
        }
    }

    /// Analyze and validate Claude session structure
    pub async fn analyze_schema(&self, raw_data: &str) -> Result<BeruSchemaAnalysis, BeruError> {
        let parsed: serde_json::Value =
            serde_json::from_str(raw_data).map_err(|e| BeruError::ParseError(e.to_string()))?;

        Ok(BeruSchemaAnalysis {
            detected_version: self.detect_claude_version(&parsed),
            field_coverage: self.analyze_field_coverage(&parsed),
            schema_compliance: self.validate_schema_compliance(&parsed),
            recommendations: self.generate_recommendations(&parsed),
        })
    }

    /// Convert raw JSONL to Beru's structured format
    pub async fn extract_session(&self, raw_data: &str) -> Result<BeruSession, BeruError> {
        // Parse raw JSONL line by line
        let lines: Vec<&str> = raw_data.lines().collect();
        let mut messages = Vec::new();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<BeruMessage>(line) {
                Ok(message) => messages.push(message),
                Err(e) => {
                    // Attempt flexible parsing for malformed messages
                    if let Ok(flexible_msg) = self.parse_flexible_message(line).await {
                        messages.push(flexible_msg);
                    } else {
                        return Err(BeruError::ParseError(format!(
                            "Failed to parse message: {}",
                            e
                        )));
                    }
                }
            }
        }

        // Extract session metadata from first message or create default
        let session_id = messages
            .first()
            .map(|m| m.uuid.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(BeruSession {
            uuid: session_id,
            name: Some("Extracted Session".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages,
            project_uuid: None,
            conversation_id: uuid::Uuid::new_v4().to_string(),
            metadata: BeruSessionMetadata {
                model: Some("claude-3-sonnet".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                tools_enabled: vec![],
                settings: HashMap::new(),
            },
        })
    }

    async fn parse_flexible_message(&self, line: &str) -> Result<BeruMessage, BeruError> {
        // Flexible parsing for various Claude JSONL formats
        let raw: serde_json::Value = serde_json::from_str(line)?;

        let uuid = raw
            .get("uuid")
            .or_else(|| raw.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or(&uuid::Uuid::new_v4().to_string())
            .to_string();

        let content = raw
            .get("content")
            .or_else(|| raw.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let role = raw
            .get("role")
            .or_else(|| raw.get("sender"))
            .and_then(|v| v.as_str())
            .unwrap_or("human")
            .to_string();

        Ok(BeruMessage {
            uuid,
            content: BeruMessageContent::Text(content),
            role,
            created_at: Utc::now(),
            updated_at: None,
            parent_uuid: None,
            is_active: true,
            attachments: vec![],
            tool_calls: None,
            usage: None,
            metadata: HashMap::new(),
        })
    }

    fn detect_claude_version(&self, data: &serde_json::Value) -> String {
        // Analyze structure to detect Claude API version
        if data.get("usage").is_some() && data.get("stop_reason").is_some() {
            "claude-3".to_string()
        } else if data.get("message").is_some() {
            "claude-2".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn analyze_field_coverage(&self, data: &serde_json::Value) -> f32 {
        // Calculate percentage of expected fields present
        let expected_fields = vec!["uuid", "content", "role", "created_at"];
        let present_fields = expected_fields
            .iter()
            .filter(|field| data.get(*field).is_some())
            .count();

        present_fields as f32 / expected_fields.len() as f32
    }

    fn validate_schema_compliance(&self, data: &serde_json::Value) -> bool {
        // Validate against known Claude schema patterns
        data.get("uuid").is_some() && data.get("role").is_some()
    }

    fn generate_recommendations(&self, data: &serde_json::Value) -> Vec<String> {
        let mut recommendations = Vec::new();

        if data.get("uuid").is_none() {
            recommendations.push("Add UUID field for message identification".to_string());
        }

        if data.get("created_at").is_none() {
            recommendations.push("Add timestamp for temporal ordering".to_string());
        }

        recommendations
    }
}

impl ShadowAgent for BeruAgent {
    const NAME: &'static str = "Beru";
    const SPECIALIZATION: &'static str = "Data Model Archaeology";

    fn performance_targets(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            parse_duration: std::time::Duration::from_millis(50), // 20x faster than v1
            memory_usage: 10 * 1024 * 1024,                       // 10MB max
            throughput: 1000.0,                                   // 1000 messages per second
            error_rate: 0.01,                                     // 1% error tolerance
        }
    }
}

/// Schema analysis results from Beru
#[derive(Debug, Clone)]
pub struct BeruSchemaAnalysis {
    pub detected_version: String,
    pub field_coverage: f32,
    pub schema_compliance: bool,
    pub recommendations: Vec<String>,
}

/// Beru-specific errors
#[derive(Debug, thiserror::Error)]
pub enum BeruError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

impl From<serde_json::Error> for BeruError {
    fn from(err: serde_json::Error) -> Self {
        BeruError::ParseError(err.to_string())
    }
}
