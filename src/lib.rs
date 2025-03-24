//! # gemini-rust
//! 
//! A Rust client library for Google's Gemini 2.0 API.

mod client;
mod error;
mod models;
mod tools;

pub use client::Gemini;
pub use models::{
    GenerationConfig, Message, Role, Content, Part, GenerationResponse, 
    Candidate, SafetyRating, CitationMetadata, FunctionCallingMode
};
pub use error::Error;
pub use tools::{Tool, FunctionDeclaration, FunctionCall, FunctionParameters, PropertyDetails};

/// Result type for this crate
pub type Result<T> = std::result::Result<T, Error>;