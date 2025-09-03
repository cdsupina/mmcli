//! API response models

use serde::Deserialize;

/// A single link item in API responses
#[derive(Debug, Deserialize)]
pub struct LinkItem {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: String,
}

/// Product API response containing links
#[derive(Debug, Deserialize)]
pub struct ProductResponse {
    #[serde(rename = "Links")]
    pub links: Option<Vec<LinkItem>>,
}

/// CAD file information
#[derive(Debug, Clone)]
pub struct CadFile {
    pub format: CadFormat,
    pub url: String,
    pub key: String, // Original API key like "2-D DWG", "3-D STEP"
}

/// CAD file format enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum CadFormat {
    Dwg,
    Step,
    Dxf,
    Iges,
    Solidworks,
    Sat,
    Edrw,
    Pdf,
}

impl CadFormat {
    /// Create CadFormat from API key string
    pub fn from_api_key(key: &str) -> Option<Self> {
        match key {
            k if k.contains("DWG") => Some(CadFormat::Dwg),
            k if k.contains("STEP") => Some(CadFormat::Step),
            k if k.contains("DXF") => Some(CadFormat::Dxf),
            k if k.contains("IGES") => Some(CadFormat::Iges),
            k if k.contains("SLDPRT") || k.contains("SLDDRW") || k.contains("Solidworks") => Some(CadFormat::Solidworks),
            k if k.contains("SAT") => Some(CadFormat::Sat),
            k if k.contains("EDRW") => Some(CadFormat::Edrw),
            k if k.contains("PDF") => Some(CadFormat::Pdf),
            _ => None,
        }
    }
    
    /// Check if this format matches the given filter string
    pub fn matches_filter(&self, filter: &str) -> bool {
        match filter {
            "dwg" => matches!(self, CadFormat::Dwg),
            "step" => matches!(self, CadFormat::Step),
            "dxf" => matches!(self, CadFormat::Dxf),
            "iges" => matches!(self, CadFormat::Iges),
            "solidworks" => matches!(self, CadFormat::Solidworks),
            "sat" => matches!(self, CadFormat::Sat),
            "edrw" => matches!(self, CadFormat::Edrw),
            "pdf" => matches!(self, CadFormat::Pdf),
            _ => false,
        }
    }
}

/// Product links for downloads
pub struct ProductLinks {
    pub images: Vec<String>,
    pub cad: Vec<CadFile>,
    pub datasheets: Vec<String>,
}

/// Basic product information
#[derive(Debug, Deserialize)]
pub struct ProductInfo {
    #[serde(rename = "PartNumber")]
    pub part_number: Option<String>,
    #[serde(rename = "DetailDescription")]
    pub detail_description: Option<String>,
    #[serde(rename = "FamilyDescription")]
    pub family_description: Option<String>,
}