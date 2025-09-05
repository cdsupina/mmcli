//! Dimension and format conversion utilities

use regex::Regex;
use crate::models::product::ProductDetail;

/// Parse a fraction string into a decimal value
fn parse_fraction_to_decimal(fraction_str: &str) -> Option<f64> {
    // Handle mixed numbers like "2-3/8" or simple fractions like "3/4"
    if fraction_str.contains('-') && fraction_str.contains('/') {
        // Mixed number: "2-3/8"
        let parts: Vec<&str> = fraction_str.split('-').collect();
        if parts.len() == 2 {
            let whole: f64 = parts[0].parse().ok()?;
            let fraction_decimal = parse_simple_fraction(parts[1])?;
            Some(whole + fraction_decimal)
        } else {
            None
        }
    } else if fraction_str.contains('/') {
        // Simple fraction: "3/4"
        parse_simple_fraction(fraction_str)
    } else {
        // Just a number: "2"
        fraction_str.parse().ok()
    }
}

/// Parse a simple fraction like "3/8" into decimal
fn parse_simple_fraction(fraction: &str) -> Option<f64> {
    let parts: Vec<&str> = fraction.split('/').collect();
    if parts.len() == 2 {
        let numerator: f64 = parts[0].parse().ok()?;
        let denominator: f64 = parts[1].parse().ok()?;
        if denominator != 0.0 {
            Some(numerator / denominator)
        } else {
            None
        }
    } else {
        None
    }
}

/// Convert length measurements from fractions to decimals
pub fn convert_length_to_decimal(value: &str) -> String {
    // Handle measurements with inch marks
    if value.contains("\"") {
        let clean_value = value.replace("\"", "").replace(" ", "-"); // Convert space format to hyphen format
        
        // Try to parse as fraction and convert to decimal
        if let Some(decimal) = parse_fraction_to_decimal(&clean_value) {
            // Format with appropriate precision
            if decimal.fract() == 0.0 {
                // Whole number
                format!("{}", decimal as i32)
            } else {
                // Use up to 5 decimal places, removing trailing zeros
                format!("{:.5}", decimal).trim_end_matches('0').trim_end_matches('.').to_string()
            }
        } else {
            // If parsing fails, return the cleaned value as-is
            clean_value
        }
    } else if value.contains("mm") {
        // Remove mm suffix for metric dimensions
        value.replace("mm", "").trim().to_string()
    } else {
        value.to_string()
    }
}

/// Extract thread size with pitch for both metric and inch threads, convert separators to 'x'
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
    
    // For inch screws (e.g., "No. 4", "1/4", "5/16"), check for threads per inch
    if let Some(tpi_spec) = product.specifications.iter()
        .find(|s| s.attribute.contains("Threads per Inch") || s.attribute.contains("threads per inch")) {
        if let Some(tpi_value) = tpi_spec.values.first() {
            // Convert "No. 4" → "4x20" or "1/4" → "1/4x20"
            let clean_size = thread_size.replace("No. ", "");
            return format!("{}x{}", clean_size, tpi_value.trim());
        }
    }
    
    thread_with_x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fractions() {
        // Test common simple fractions
        assert_eq!(convert_length_to_decimal("1/8\""), "0.125");
        assert_eq!(convert_length_to_decimal("1/4\""), "0.25");
        assert_eq!(convert_length_to_decimal("3/8\""), "0.375");
        assert_eq!(convert_length_to_decimal("1/2\""), "0.5");
        assert_eq!(convert_length_to_decimal("5/8\""), "0.625");
        assert_eq!(convert_length_to_decimal("3/4\""), "0.75");
        assert_eq!(convert_length_to_decimal("7/8\""), "0.875");
    }

    #[test]
    fn test_sixteenths_fractions() {
        assert_eq!(convert_length_to_decimal("3/16\""), "0.1875");
        assert_eq!(convert_length_to_decimal("5/16\""), "0.3125");
        assert_eq!(convert_length_to_decimal("7/16\""), "0.4375");
        assert_eq!(convert_length_to_decimal("9/16\""), "0.5625");
        assert_eq!(convert_length_to_decimal("11/16\""), "0.6875");
        assert_eq!(convert_length_to_decimal("13/16\""), "0.8125");
        assert_eq!(convert_length_to_decimal("15/16\""), "0.9375");
    }

    #[test]
    fn test_thirty_seconds_fractions() {
        assert_eq!(convert_length_to_decimal("13/32\""), "0.40625");
    }

    #[test]
    fn test_mixed_numbers() {
        assert_eq!(convert_length_to_decimal("1-1/8\""), "1.125");
        assert_eq!(convert_length_to_decimal("1-1/4\""), "1.25");
        assert_eq!(convert_length_to_decimal("1-3/8\""), "1.375");
        assert_eq!(convert_length_to_decimal("1-1/2\""), "1.5");
        assert_eq!(convert_length_to_decimal("1-5/8\""), "1.625");
        assert_eq!(convert_length_to_decimal("1-3/4\""), "1.75");
        assert_eq!(convert_length_to_decimal("1-7/8\""), "1.875");
    }

    #[test]
    fn test_two_inch_mixed_numbers() {
        assert_eq!(convert_length_to_decimal("2-1/8\""), "2.125");
        assert_eq!(convert_length_to_decimal("2-1/4\""), "2.25");
        assert_eq!(convert_length_to_decimal("2-3/8\""), "2.375");
        assert_eq!(convert_length_to_decimal("2-1/2\""), "2.5");
        assert_eq!(convert_length_to_decimal("2-5/8\""), "2.625");
        assert_eq!(convert_length_to_decimal("2-3/4\""), "2.75");
        assert_eq!(convert_length_to_decimal("2-7/8\""), "2.875");
    }

    #[test]
    fn test_larger_mixed_numbers() {
        assert_eq!(convert_length_to_decimal("3-1/4\""), "3.25");
        assert_eq!(convert_length_to_decimal("3-1/2\""), "3.5");
        assert_eq!(convert_length_to_decimal("3-3/4\""), "3.75");
        assert_eq!(convert_length_to_decimal("4-1/2\""), "4.5");
        assert_eq!(convert_length_to_decimal("5-1/2\""), "5.5");
    }

    #[test]
    fn test_whole_numbers() {
        assert_eq!(convert_length_to_decimal("1\""), "1");
        assert_eq!(convert_length_to_decimal("2\""), "2");
        assert_eq!(convert_length_to_decimal("3\""), "3");
        assert_eq!(convert_length_to_decimal("4\""), "4");
        assert_eq!(convert_length_to_decimal("5\""), "5");
        assert_eq!(convert_length_to_decimal("6\""), "6");
    }

    #[test]
    fn test_space_format() {
        // Test space format (e.g., "2 3/8"") gets converted correctly
        assert_eq!(convert_length_to_decimal("2 3/8\""), "2.375");
        assert_eq!(convert_length_to_decimal("1 1/2\""), "1.5");
    }

    #[test]
    fn test_metric_dimensions() {
        assert_eq!(convert_length_to_decimal("10mm"), "10");
        assert_eq!(convert_length_to_decimal("25.4mm"), "25.4");
        assert_eq!(convert_length_to_decimal("5mm"), "5");
    }

    #[test]
    fn test_no_suffix() {
        assert_eq!(convert_length_to_decimal("10"), "10");
        assert_eq!(convert_length_to_decimal("5.5"), "5.5");
        assert_eq!(convert_length_to_decimal("test"), "test");
    }

    #[test]
    fn test_edge_cases() {
        // Test unusual fractions that should still work
        assert_eq!(convert_length_to_decimal("7/64\""), "0.10938");
        assert_eq!(convert_length_to_decimal("21/64\""), "0.32812");
        
        // Test invalid fractions (should return cleaned input)
        assert_eq!(convert_length_to_decimal("invalid\""), "invalid");
        assert_eq!(convert_length_to_decimal("1/0\""), "1/0"); // Division by zero case
    }

    #[test]
    fn test_parse_fraction_to_decimal() {
        assert_eq!(parse_fraction_to_decimal("1/2"), Some(0.5));
        assert_eq!(parse_fraction_to_decimal("3/4"), Some(0.75));
        assert_eq!(parse_fraction_to_decimal("1-1/4"), Some(1.25));
        assert_eq!(parse_fraction_to_decimal("2"), Some(2.0));
        assert_eq!(parse_fraction_to_decimal("invalid"), None);
        assert_eq!(parse_fraction_to_decimal("1/0"), None); // Division by zero
    }
}