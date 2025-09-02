//! Naming Templates
//! 
//! This module contains all the templates used for generating names
//! for different categories of fasteners and components.

pub mod bearings;
pub mod nuts;
pub mod screws;
pub mod standoffs;
pub mod washers;

pub use bearings::initialize_bearing_templates;
pub use nuts::initialize_nut_templates;
pub use screws::initialize_screw_templates;
pub use standoffs::initialize_standoff_templates;
pub use washers::initialize_washer_templates;