//! Cable holder naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all cable holder templates
pub fn initialize_cable_holder_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    // Create shared abbreviations for all cable holder types
    let cable_holder_abbrevs = create_cable_holder_abbreviations();
    
    // Initialize cable holder template
    initialize_cable_holder_types(category_templates, &cable_holder_abbrevs);
}

/// Create comprehensive abbreviations for cable holder templates
fn create_cable_holder_abbreviations() -> HashMap<String, String> {
    let mut abbrevs = HashMap::new();
    
    // Material abbreviations
    abbrevs.insert("Nylon Plastic".to_string(), "NY".to_string());
    abbrevs.insert("Plastic".to_string(), "PL".to_string());
    abbrevs.insert("Polyethylene".to_string(), "PE".to_string());
    abbrevs.insert("Polypropylene".to_string(), "PP".to_string());
    abbrevs.insert("PVC".to_string(), "PVC".to_string());
    abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    abbrevs.insert("Steel".to_string(), "STEEL".to_string());
    abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    
    // Mount type abbreviations
    abbrevs.insert("Screw In".to_string(), "SI".to_string());
    abbrevs.insert("Screw-In".to_string(), "SI".to_string());
    abbrevs.insert("Adhesive".to_string(), "ADH".to_string());
    abbrevs.insert("Self-Adhesive".to_string(), "ADH".to_string());
    abbrevs.insert("Snap In".to_string(), "SNP".to_string());
    abbrevs.insert("Snap-In".to_string(), "SNP".to_string());
    abbrevs.insert("Push Mount".to_string(), "PUSH".to_string());
    abbrevs.insert("Tie Mount".to_string(), "TIE".to_string());
    
    // Bundle diameter conversion (convert fractions to decimals)
    abbrevs.insert("1/8\"".to_string(), "0.125".to_string());
    abbrevs.insert("3/16\"".to_string(), "0.1875".to_string());
    abbrevs.insert("1/4\"".to_string(), "0.25".to_string());
    abbrevs.insert("5/16\"".to_string(), "0.3125".to_string());
    abbrevs.insert("3/8\"".to_string(), "0.375".to_string());
    abbrevs.insert("1/2\"".to_string(), "0.5".to_string());
    abbrevs.insert("5/8\"".to_string(), "0.625".to_string());
    abbrevs.insert("3/4\"".to_string(), "0.75".to_string());
    abbrevs.insert("1\"".to_string(), "1".to_string());
    
    // Screw size abbreviations
    abbrevs.insert("No. 4".to_string(), "4".to_string());
    abbrevs.insert("No. 6".to_string(), "6".to_string());
    abbrevs.insert("No. 8".to_string(), "8".to_string());
    abbrevs.insert("No. 10".to_string(), "10".to_string());
    abbrevs.insert("#4".to_string(), "4".to_string());
    abbrevs.insert("#6".to_string(), "6".to_string());
    abbrevs.insert("#8".to_string(), "8".to_string());
    abbrevs.insert("#10".to_string(), "10".to_string());
    
    abbrevs
}

/// Initialize cable holder type templates
fn initialize_cable_holder_types(category_templates: &mut HashMap<String, NamingTemplate>, abbrevs: &HashMap<String, String>) {
    // Cable Holder template
    let cable_holder_template = NamingTemplate {
        prefix: "CH".to_string(),
        key_specs: vec![
            "Material".to_string(), 
            "Mount Type".to_string(), 
            "For Maximum Bundle Diameter".to_string(), 
            "For Screw Size".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("cable_holder".to_string(), cable_holder_template);
    
    // Generic cable holder (fallback)
    let generic_cable_holder_template = NamingTemplate {
        prefix: "CH".to_string(),
        key_specs: vec![
            "Material".to_string(), 
            "Mount Type".to_string(), 
            "For Maximum Bundle Diameter".to_string()
        ],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("generic_cable_holder".to_string(), generic_cable_holder_template);
}