//! Error types for v2 architecture

use thiserror::Error;
use std::io;

/// Core v2 error type
#[derive(Error, Debug)]
pub enum V2Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Adapter error: {0}")]
    Adapter(String),
    
    #[error("Feature not supported in v1: {0}")]
    V1Unsupported(String),
    
    #[error("Performance target not met: expected {expected}ms, got {actual}ms")]
    PerformanceViolation { expected: u64, actual: u64 },
    
    #[error("Shadow agent error: {agent} - {message}")]
    ShadowAgent { agent: String, message: String },
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl V2Error {
    pub fn parse(msg: impl Into<String>) -> Self {
        V2Error::Parse(msg.into())
    }
    
    pub fn adapter(msg: impl Into<String>) -> Self {
        V2Error::Adapter(msg.into())
    }
    
    pub fn v1_unsupported(feature: impl Into<String>) -> Self {
        V2Error::V1Unsupported(feature.into())
    }
    
    pub fn shadow_agent(agent: impl Into<String>, message: impl Into<String>) -> Self {
        V2Error::ShadowAgent {
            agent: agent.into(),
            message: message.into(),
        }
    }
}

pub type V2Result<T> = Result<T, V2Error>;