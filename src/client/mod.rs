//! McMaster-Carr API Client
//! 
//! This module provides the main client interface for interacting with
//! McMaster-Carr's Product Information API, including authentication,
//! product management, and file downloads.

pub mod api;
pub mod auth;
pub mod downloads;
pub mod subscriptions;

pub use api::McmasterClient;