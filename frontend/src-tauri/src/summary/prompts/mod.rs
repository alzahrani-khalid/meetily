//! LLM prompt management for meeting summarization
//!
//! This module provides externalized, locale-aware prompts for the summary pipeline.
//! All prompts are embedded at compile time via `include_str!` for offline builds.
//!
//! # Usage
//!
//! ```rust
//! use crate::summary::prompts;
//!
//! // Get a prompt with locale-aware resolution (English fallback)
//! let system_prompt = prompts::get_prompt("chunk_summarizer_system", "ar")?;
//! let user_prompt = prompts::get_prompt("chunk_summarizer_user", "ar")?;
//! ```

mod defaults;
mod loader;

pub use loader::get_prompt;
