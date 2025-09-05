//! Latch naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all latch templates
pub fn initialize_latch_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    // Create shared abbreviations for all latch types
    let latch_abbrevs = create_latch_abbreviations();
    
    // Initialize latch templates
    initialize_latch_types(category_templates, &latch_abbrevs);
}

/// Create comprehensive abbreviations for latch templates
fn create_latch_abbreviations() -> HashMap<String, String> {
    let mut abbrevs = HashMap::new();
    
    // Material abbreviations
    abbrevs.insert("304 Stainless Steel".to_string(), "SS304".to_string());
    abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
    abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    abbrevs.insert("Steel".to_string(), "STEEL".to_string());
    abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    abbrevs.insert("Brass".to_string(), "BRASS".to_string());
    abbrevs.insert("Zinc Plated Steel".to_string(), "STEEL-ZP".to_string());
    abbrevs.insert("Chrome Plated Steel".to_string(), "STEEL-CR".to_string());
    
    // Mount type abbreviations
    abbrevs.insert("Screw On".to_string(), "SO".to_string());
    abbrevs.insert("Screw-On".to_string(), "SO".to_string());
    abbrevs.insert("Weld On".to_string(), "WO".to_string());
    abbrevs.insert("Weld-On".to_string(), "WO".to_string());
    abbrevs.insert("Bolt On".to_string(), "BO".to_string());
    abbrevs.insert("Bolt-On".to_string(), "BO".to_string());
    abbrevs.insert("Surface Mount".to_string(), "SM".to_string());
    abbrevs.insert("Surface".to_string(), "SM".to_string());
    
    // Latching distance conversion (convert fractions to decimals)
    abbrevs.insert("1/8\"".to_string(), "0.125".to_string());
    abbrevs.insert("3/16\"".to_string(), "0.1875".to_string());
    abbrevs.insert("1/4\"".to_string(), "0.25".to_string());
    abbrevs.insert("5/16\"".to_string(), "0.3125".to_string());
    abbrevs.insert("3/8\"".to_string(), "0.375".to_string());
    abbrevs.insert("1/2\"".to_string(), "0.5".to_string());
    abbrevs.insert("5/8\"".to_string(), "0.625".to_string());
    abbrevs.insert("3/4\"".to_string(), "0.75".to_string());
    abbrevs.insert("1\"".to_string(), "1".to_string());
    
    // Draw latch type abbreviations
    abbrevs.insert("Locking".to_string(), "L".to_string());
    abbrevs.insert("Nonlocking".to_string(), "NL".to_string());
    abbrevs.insert("Non-locking".to_string(), "NL".to_string());
    abbrevs.insert("Keyed".to_string(), "K".to_string());
    abbrevs.insert("Adjustable".to_string(), "ADJ".to_string());
    abbrevs.insert("Fixed".to_string(), "F".to_string());
    
    // Capacity abbreviations (clean up "lbs." suffix)
    abbrevs.insert("130 lbs.".to_string(), "130".to_string());
    abbrevs.insert("200 lbs.".to_string(), "200".to_string());
    abbrevs.insert("250 lbs.".to_string(), "250".to_string());
    abbrevs.insert("300 lbs.".to_string(), "300".to_string());
    abbrevs.insert("400 lbs.".to_string(), "400".to_string());
    abbrevs.insert("500 lbs.".to_string(), "500".to_string());
    
    abbrevs
}

/// Initialize latch type templates
fn initialize_latch_types(category_templates: &mut HashMap<String, NamingTemplate>, abbrevs: &HashMap<String, String>) {
    // Draw Latch template
    let draw_latch_template = NamingTemplate {
        prefix: "DL".to_string(),
        key_specs: vec![
            "Material".to_string(),
            "Mount Type".to_string(), 
            "Latching Distance".to_string(),
            "Draw Latch Type".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("draw_latch".to_string(), draw_latch_template);
    
    // Toggle Latch template
    let toggle_latch_template = NamingTemplate {
        prefix: "TL".to_string(),
        key_specs: vec![
            "Material".to_string(),
            "Mount Type".to_string(), 
            "Latching Distance".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("toggle_latch".to_string(), toggle_latch_template);
    
    // Compression Latch template
    let compression_latch_template = NamingTemplate {
        prefix: "CL".to_string(),
        key_specs: vec![
            "Material".to_string(),
            "Mount Type".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("compression_latch".to_string(), compression_latch_template);
    
    // Slam Latch template
    let slam_latch_template = NamingTemplate {
        prefix: "SL".to_string(),
        key_specs: vec![
            "Material".to_string(),
            "Mount Type".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("slam_latch".to_string(), slam_latch_template);
    
    // Generic latch (fallback)
    let generic_latch_template = NamingTemplate {
        prefix: "LATCH".to_string(),
        key_specs: vec![
            "Material".to_string(),
            "Mount Type".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("generic_latch".to_string(), generic_latch_template);
}