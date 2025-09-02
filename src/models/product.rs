//! Product data models

use serde::{Deserialize, Serialize};

/// Product pricing information
#[derive(Debug, Deserialize, Serialize)]
pub struct PriceInfo {
    #[serde(rename = "Amount")]
    pub amount: f64,
    #[serde(rename = "MinimumQuantity")]
    pub minimum_quantity: f64,
    #[serde(rename = "UnitOfMeasure")]
    pub unit_of_measure: String,
}

/// Complete product details including specifications
#[derive(Debug, Deserialize, Serialize)]
pub struct ProductDetail {
    #[serde(rename = "PartNumber")]
    pub part_number: String,
    #[serde(rename = "DetailDescription")]
    pub detail_description: String,
    #[serde(rename = "FamilyDescription")]
    pub family_description: String,
    #[serde(rename = "ProductCategory")]
    pub product_category: String,
    #[serde(rename = "ProductStatus")]
    pub product_status: String,
    #[serde(rename = "Specifications", default)]
    pub specifications: Vec<Specification>,
}

/// Product specification attribute and values
#[derive(Debug, Deserialize, Serialize)]
pub struct Specification {
    #[serde(rename = "Attribute")]
    pub attribute: String,
    #[serde(rename = "Values")]
    pub values: Vec<String>,
}