//! McMaster-Carr CLI Library
//!
//! A comprehensive library for interacting with McMaster-Carr's Product Information API
//! and generating human-readable technical names for fasteners and components.

pub mod client;
pub mod config;
pub mod models;
pub mod naming;
pub mod utils;

// Re-export main types for convenience
pub use client::McmasterClient;
pub use models::{
    api::ProductInfo,
    api::{CadFile, CadFormat, LinkItem, ProductResponse},
    auth::{Credentials, LoginRequest, LoginResponse},
    product::{PriceInfo, ProductDetail, Specification},
};
pub use naming::{NameGenerator, NamingTemplate};
pub use utils::error::ClientError;
pub use utils::output::{OutputFormat, ProductField};
