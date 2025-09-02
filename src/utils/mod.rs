//! Utilities
//! 
//! This module contains utility functions and types used throughout
//! the application, including output formatting and error handling.

pub mod error;
pub mod output;

pub use error::ClientError;
pub use output::{OutputFormat, ProductField};