//! Configuration
//! 
//! This module handles configuration management, including XDG-compliant
//! paths and certificate discovery.

pub mod paths;

pub use paths::{get_config_dir, get_token_path, find_certificate_path};