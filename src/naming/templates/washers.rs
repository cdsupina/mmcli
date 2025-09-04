//! Washer naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all washer templates
pub fn initialize_washer_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    // Create shared abbreviations for all washer types
    let washer_abbrevs = create_washer_abbreviations();
    
    // Initialize all 19 washer type templates
    initialize_all_washer_types(category_templates, &washer_abbrevs);
}

/// Create comprehensive abbreviations for washer templates
fn create_washer_abbreviations() -> HashMap<String, String> {
    let mut abbrevs = HashMap::new();
    
    // Material abbreviations
    abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
    abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    abbrevs.insert("Steel".to_string(), "Steel".to_string()); // Keep full name for washers
    abbrevs.insert("Alloy Steel".to_string(), "Steel".to_string());
    abbrevs.insert("Brass".to_string(), "Brass".to_string());
    abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    abbrevs.insert("Nylon".to_string(), "Nylon".to_string());
    abbrevs.insert("Plastic".to_string(), "Plastic".to_string());
    abbrevs.insert("Rubber".to_string(), "Rubber".to_string());
    abbrevs.insert("Spring Steel".to_string(), "Steel".to_string());
    
    // Finish abbreviations
    abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
    abbrevs.insert("Zinc-Plated".to_string(), "ZP".to_string());
    abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
    abbrevs.insert("Zinc Yellow Chromate Plated".to_string(), "ZYC".to_string());
    abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
    abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
    abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
    abbrevs.insert("Cadmium-Plated".to_string(), "CD".to_string());
    abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
    abbrevs.insert("Nickel-Plated".to_string(), "NI".to_string());
    abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
    abbrevs.insert("Chrome-Plated".to_string(), "CR".to_string());
    abbrevs.insert("Galvanized".to_string(), "GAL".to_string());
    
    abbrevs
}

/// Initialize all 19 washer type templates
fn initialize_all_washer_types(category_templates: &mut HashMap<String, NamingTemplate>, abbrevs: &HashMap<String, String>) {
    let washer_types = [
        ("cup_washer", "CW"),
        ("curved_washer", "CRVW"),
        ("dished_washer", "DW"),
        ("domed_washer", "DMW"),
        ("double_clipped_washer", "DCW"),
        ("clipped_washer", "CLW"),
        ("flat_washer", "FW"),
        ("hillside_washer", "HW"),
        ("notched_washer", "NW"),
        ("perforated_washer", "PW"),
        ("pronged_washer", "PRW"),
        ("rectangular_washer", "RW"),
        ("sleeve_washer", "SW"),
        ("slotted_washer", "SLW"),
        ("spherical_washer", "SPW"),
        ("split_washer", "SPLW"),
        ("square_washer", "SQW"),
        ("tab_washer", "TW"),
        ("tapered_washer", "TPW"),
        ("tooth_washer", "TOW"),
        ("wave_washer", "WW"),
        ("wedge_washer", "WDW"),
    ];
    
    for (washer_type, prefix) in &washer_types {
        let washer_template = NamingTemplate {
            prefix: prefix.to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_aliases: None,
            spec_abbreviations: abbrevs.clone(),
        };
        category_templates.insert(washer_type.to_string(), washer_template);
    }
}