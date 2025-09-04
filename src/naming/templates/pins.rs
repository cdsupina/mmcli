//! Pin and collar naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all pin and collar templates
pub fn initialize_pin_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    let mut pin_abbrevs = HashMap::new();
    
    // Material abbreviations for pins
    pin_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    pin_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
    pin_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    pin_abbrevs.insert("Steel".to_string(), "S".to_string());
    pin_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
    pin_abbrevs.insert("Brass".to_string(), "Brass".to_string());
    pin_abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    pin_abbrevs.insert("Titanium".to_string(), "TI".to_string());
    
    // Steel grade abbreviations for pins
    pin_abbrevs.insert("Grade 1 Steel".to_string(), "SG1".to_string());
    pin_abbrevs.insert("Grade 2 Steel".to_string(), "SG2".to_string());
    pin_abbrevs.insert("Grade 5 Steel".to_string(), "SG5".to_string());
    pin_abbrevs.insert("Grade 8 Steel".to_string(), "SG8".to_string());
    pin_abbrevs.insert("8.8 Steel".to_string(), "S8.8".to_string());
    pin_abbrevs.insert("10.9 Steel".to_string(), "S10.9".to_string());
    pin_abbrevs.insert("12.9 Steel".to_string(), "S12.9".to_string());
    
    // Finish abbreviations for pins
    pin_abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
    pin_abbrevs.insert("Zinc-Plated".to_string(), "ZP".to_string());
    pin_abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
    pin_abbrevs.insert("Zinc Yellow Chromate Plated".to_string(), "ZYC".to_string());
    pin_abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
    pin_abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
    pin_abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
    pin_abbrevs.insert("Cadmium-Plated".to_string(), "CD".to_string());
    pin_abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
    pin_abbrevs.insert("Nickel-Plated".to_string(), "NI".to_string());
    pin_abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
    pin_abbrevs.insert("Chrome-Plated".to_string(), "CR".to_string());
    pin_abbrevs.insert("Galvanized".to_string(), "GAL".to_string());
    
    // End Type abbreviations for clevis pins
    pin_abbrevs.insert("Retaining Ring Groove".to_string(), "RRG".to_string());
    pin_abbrevs.insert("Plain".to_string(), "".to_string()); // Don't show plain end type
    
    // Regular Clevis Pin
    let clevis_pin_template = NamingTemplate {
        prefix: "CP".to_string(),
        key_specs: vec!["Material".to_string(), "Diameter".to_string(), "Usable Length".to_string(), "Finish".to_string()],
        spec_abbreviations: pin_abbrevs.clone(),
    };
    category_templates.insert("clevis_pin".to_string(), clevis_pin_template);
    
    // Clevis Pin with Retaining Ring Groove
    let clevis_pin_rrg_template = NamingTemplate {
        prefix: "CPRRG".to_string(),
        key_specs: vec!["Material".to_string(), "Diameter".to_string(), "Usable Length".to_string(), "Finish".to_string()],
        spec_abbreviations: pin_abbrevs.clone(),
    };
    category_templates.insert("clevis_pin_with_retaining_ring_groove".to_string(), clevis_pin_rrg_template);
    
    // Create shared abbreviations for shaft collars (reuse pin abbreviations plus specific materials)
    let mut collar_abbrevs = pin_abbrevs.clone();
    collar_abbrevs.insert("303 Stainless Steel".to_string(), "SS303".to_string());
    collar_abbrevs.insert("1215 Carbon Steel".to_string(), "1215S".to_string());
    
    // Face-Mount Shaft Collar
    let face_mount_collar_template = NamingTemplate {
        prefix: "FMSC".to_string(),
        key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Width".to_string(), "Finish".to_string()],
        spec_abbreviations: collar_abbrevs.clone(),
    };
    category_templates.insert("face_mount_shaft_collar".to_string(), face_mount_collar_template);
    
    // Flange-Mount Shaft Collar
    let flange_mount_collar_template = NamingTemplate {
        prefix: "FLSC".to_string(),
        key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Width".to_string(), "Finish".to_string()],
        spec_abbreviations: collar_abbrevs,
    };
    category_templates.insert("flange_mount_shaft_collar".to_string(), flange_mount_collar_template);
}