//! Error types for v2 Shadow Renaissance Architecture

use thiserror::Error;

/// Core v2 error type with comprehensive error handling
#[derive(Error, Debug)]
pub enum V2Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Shadow agent error: {agent} - {message}")]
    ShadowAgent { agent: String, message: String },

    #[error(
        "Performance target violation: {component} exceeded {target_ms}ms (actual: {actual_ms}ms)"
    )]
    PerformanceViolation {
        component: String,
        target_ms: u64,
        actual_ms: u64,
    },

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Virtualization error: {0}")]
    Virtualization(String),

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl V2Error {
    pub fn shadow_agent(agent: impl Into<String>, message: impl Into<String>) -> Self {
        V2Error::ShadowAgent {
            agent: agent.into(),
            message: message.into(),
        }
    }

    pub fn performance_violation(
        component: impl Into<String>,
        target_ms: u64,
        actual_ms: u64,
    ) -> Self {
        V2Error::PerformanceViolation {
            component: component.into(),
            target_ms,
            actual_ms,
        }
    }

    pub fn timeout(timeout_ms: u64) -> Self {
        V2Error::Timeout { timeout_ms }
    }

    pub fn resource_exhausted(resource: impl Into<String>) -> Self {
        V2Error::ResourceExhausted {
            resource: resource.into(),
        }
    }
}

pub type V2Result<T> = Result<T, V2Error>;
