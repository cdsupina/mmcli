//! Dimension and format conversion utilities

use regex::Regex;
use crate::models::product::ProductDetail;

/// Convert length measurements from fractions to decimals
pub fn convert_length_to_decimal(value: &str) -> String {
    // Convert common fractions to decimals for screw lengths
    if value.contains("\"") {
        let clean_value = value.replace("\"", "").replace(" ", "-"); // Convert space format to hyphen format
        match clean_value.as_str() {
            "1/8" => "0.125".to_string(),
            "3/16" => "0.1875".to_string(),
            "1/4" => "0.25".to_string(),
            "5/16" => "0.3125".to_string(),
            "3/8" => "0.375".to_string(),
            "7/16" => "0.4375".to_string(),
            "1/2" => "0.5".to_string(),
            "9/16" => "0.5625".to_string(),
            "5/8" => "0.625".to_string(),
            "11/16" => "0.6875".to_string(),
            "3/4" => "0.75".to_string(),
            "13/16" => "0.8125".to_string(),
            "7/8" => "0.875".to_string(),
            "15/16" => "0.9375".to_string(),
            "1" => "1".to_string(),
            "1-1/8" => "1.125".to_string(),
            "1-1/4" => "1.25".to_string(),
            "1-3/8" => "1.375".to_string(),
            "1-1/2" => "1.5".to_string(),
            "1-5/8" => "1.625".to_string(),
            "1-3/4" => "1.75".to_string(),
            "1-7/8" => "1.875".to_string(),
            "2" => "2".to_string(),
            "2-1/4" => "2.25".to_string(),
            "2-1/2" => "2.5".to_string(),
            "2-3/4" => "2.75".to_string(),
            "3" => "3".to_string(),
            "3-1/4" => "3.25".to_string(),
            "3-1/2" => "3.5".to_string(),
            "3-3/4" => "3.75".to_string(),
            "4" => "4".to_string(),
            "4-1/2" => "4.5".to_string(),
            "5" => "5".to_string(),
            "5-1/2" => "5.5".to_string(),
            "6" => "6".to_string(),
            _ => clean_value, // Return as-is if not in our conversion table
        }
    } else if value.contains("mm") {
        // Remove mm suffix for metric dimensions
        value.replace("mm", "").trim().to_string()
    } else {
        value.to_string()
    }
}

/// Extract thread size with pitch for metric threads, convert separators to 'x'
pub fn extract_thread_with_pitch(product: &ProductDetail, thread_size: &str) -> String {
    // First, handle the separator conversion (hyphen to x)
    let thread_with_x = thread_size.replace("-", "x");
    
    // For metric threads, try to extract pitch from detail description
    if thread_with_x.starts_with('M') && !thread_with_x.contains('x') {
        // Look for pitch information in the detail description
        let detail = &product.detail_description;
        
        // Create regex to find metric thread pitch patterns
        let re = Regex::new(r"M\d+\s*x?\s*(\d+\.?\d*)\s*mm").unwrap();
        
        if let Some(captures) = re.captures(detail) {
            if let Some(pitch_match) = captures.get(1) {
                let pitch = pitch_match.as_str();
                return format!("{}x{}", thread_with_x, pitch);
            }
        }
        
        // Fallback: try to extract pitch from specifications
        if let Some(pitch_spec) = product.specifications.iter()
            .find(|s| s.attribute.eq_ignore_ascii_case("Thread Pitch")) {
            if let Some(pitch_value) = pitch_spec.values.first() {
                let clean_pitch = pitch_value.replace("mm", "").trim().to_string();
                return format!("{}x{}", thread_with_x, clean_pitch);
            }
        }
    }
    
    thread_with_x
}