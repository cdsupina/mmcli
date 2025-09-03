//! Material and specification abbreviations

use crate::models::product::ProductDetail;

/// Parse material string that might contain embedded finish information
pub fn parse_material_and_finish(material_value: &str) -> (String, Option<String>) {
    let finish_prefixes = [
        "Black-Oxide ", "Black Oxide ", "Zinc Plated ", "Zinc-Plated ",
        "Zinc Yellow-Chromate Plated ", "Zinc Yellow Chromate Plated ",
        "Galvanized ", "Cadmium Plated ", "Cadmium-Plated ",
        "Nickel Plated ", "Nickel-Plated ", "Chrome Plated ", "Chrome-Plated ",
        "Passivated ", "Plain ", "Unfinished "
    ];
    
    for prefix in &finish_prefixes {
        if material_value.starts_with(prefix) {
            let finish = prefix.trim().to_string();
            let material = material_value.strip_prefix(prefix).unwrap_or(material_value).to_string();
            return (material, Some(finish));
        }
    }
    
    (material_value.to_string(), None)
}

/// Get steel grade material description with appropriate abbreviation
pub fn get_steel_grade_material(product: &ProductDetail, original_material: &str) -> String {
    // Check if there's a fastener strength grade/class specification
    if let Some(grade_spec) = product.specifications.iter()
        .find(|s| s.attribute.eq_ignore_ascii_case("Fastener Strength Grade/Class")) {
        if let Some(grade_value) = grade_spec.values.first() {
            if grade_value.contains("Grade 1") {
                return "Grade 1 Steel".to_string();
            } else if grade_value.contains("Grade 2") {
                return "Grade 2 Steel".to_string();
            } else if grade_value.contains("Grade 5") {
                return "Grade 5 Steel".to_string();
            } else if grade_value.contains("Grade 8") {
                return "Grade 8 Steel".to_string();
            } else if grade_value.contains("8.8") {
                return "8.8 Steel".to_string();
            } else if grade_value.contains("10.9") {
                return "10.9 Steel".to_string();
            } else if grade_value.contains("12.9") {
                return "12.9 Steel".to_string();
            }
        }
    }
    
    // If no grade found, return original material
    original_material.to_string()
}

/// Create a basic abbreviation for values not in the template mappings
pub fn abbreviate_value(value: &str) -> String {
    // Special handling for thread sizes - preserve full thread designation
    if value.contains("x") && (value.contains("/") || value.starts_with('M') || value.chars().next().map_or(false, |c| c.is_ascii_digit())) {
        // This looks like a thread size (e.g., "5/16x18", "M8x1.25", or "10x24"), preserve it
        return value.replace("\"", "");
    }
    
    // Special handling for screw size format "No. X" -> "X"
    if let Some(no_match) = value.strip_prefix("No. ") {
        return no_match.to_string();
    }
    
    // Convert to uppercase and take first few characters for basic abbreviation
    let clean_value = value.replace("\"", "").replace(" ", "");
    if clean_value.len() <= 3 {
        clean_value.to_uppercase()
    } else {
        // Take first 4 characters for longer values
        clean_value.chars().take(4).collect::<String>().to_uppercase()
    }
}