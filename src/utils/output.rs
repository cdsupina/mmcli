//! Output formatting utilities

use std::fmt;
use clap::ValueEnum;

/// Output format options for displaying product information
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum OutputFormat {
    /// Human-friendly output with formatting and emojis (default)
    Human,
    /// Machine-readable JSON output
    Json,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Human => write!(f, "human"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

/// Represents different fields that can be displayed for a product
#[derive(Debug, Clone)]
pub enum ProductField {
    PartNumber,
    DetailDescription,
    FamilyDescription,
    Category,
    Status,
    Specification(String),
    AllSpecs,
    BasicInfo,
}

impl ProductField {
    /// Parse a comma-separated string of field names into ProductField enum values
    pub fn parse_fields(fields_str: &str) -> Vec<ProductField> {
        if fields_str == "all" {
            return vec![
                ProductField::PartNumber,
                ProductField::DetailDescription,
                ProductField::FamilyDescription,
                ProductField::Category,
                ProductField::Status,
                ProductField::AllSpecs,
            ];
        }

        if fields_str == "basic" {
            return vec![
                ProductField::PartNumber,
                ProductField::DetailDescription,
                ProductField::FamilyDescription,
                ProductField::Category,
                ProductField::Status,
            ];
        }

        if fields_str == "specs" {
            return vec![ProductField::AllSpecs];
        }

        fields_str
            .split(',')
            .map(|s| s.trim())
            .map(|field| match field {
                "part-number" | "partnumber" => ProductField::PartNumber,
                "detail-description" | "detail" | "description" => ProductField::DetailDescription,
                "family-description" | "family" => ProductField::FamilyDescription,
                "category" | "product-category" => ProductField::Category,
                "status" | "product-status" => ProductField::Status,
                "specs" | "specifications" => ProductField::AllSpecs,
                "basic" => ProductField::BasicInfo,
                _ => {
                    // Convert kebab-case to proper specification name
                    let spec_name = field.replace('-', " ");
                    let spec_name = spec_name.split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().chain(chars).collect(),
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    ProductField::Specification(spec_name)
                }
            })
            .collect()
    }
}