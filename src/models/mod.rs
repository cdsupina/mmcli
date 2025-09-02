//! Data Models
//! 
//! This module contains all the data structures used for API communication,
//! authentication, and product information.

pub mod api;
pub mod auth;
pub mod product;

pub use api::{ProductResponse, LinkItem, CadFile, CadFormat, ProductLinks};
pub use auth::{Credentials, LoginRequest, LoginResponse, ErrorResponse};
pub use product::{ProductDetail, Specification, PriceInfo};
pub use api::ProductInfo;