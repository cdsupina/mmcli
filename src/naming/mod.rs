//! Part Naming System
//! 
//! This module provides comprehensive part naming functionality for McMaster-Carr
//! fasteners and components, generating human-readable abbreviated technical names.

pub mod abbreviations;
pub mod analyzer;
pub mod converters;
pub mod detectors;
pub mod generator;
pub mod templates;

pub use generator::{NameGenerator, NamingTemplate};
pub use analyzer::{PartAnalyzer, PartAnalysis};