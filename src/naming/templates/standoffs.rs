//! Threaded standoff naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all threaded standoff templates
pub fn initialize_standoff_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    let mut standoff_abbrevs = HashMap::new();
    
    // Material abbreviations for standoffs
    standoff_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    standoff_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
    standoff_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    standoff_abbrevs.insert("Steel".to_string(), "S".to_string());
    standoff_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
    standoff_abbrevs.insert("Brass".to_string(), "Brass".to_string());
    standoff_abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    standoff_abbrevs.insert("Nylon".to_string(), "Nylon".to_string());
    
    // Steel grade abbreviations for standoffs
    standoff_abbrevs.insert("Grade 1 Steel".to_string(), "SG1".to_string());
    standoff_abbrevs.insert("Grade 2 Steel".to_string(), "SG2".to_string());
    standoff_abbrevs.insert("Grade 5 Steel".to_string(), "SG5".to_string());
    standoff_abbrevs.insert("Grade 8 Steel".to_string(), "SG8".to_string());
    standoff_abbrevs.insert("8.8 Steel".to_string(), "S8.8".to_string());
    standoff_abbrevs.insert("10.9 Steel".to_string(), "S10.9".to_string());
    standoff_abbrevs.insert("12.9 Steel".to_string(), "S12.9".to_string());
    standoff_abbrevs.insert("Grade 1 Alloy Steel".to_string(), "SG1".to_string());
    standoff_abbrevs.insert("Grade 2 Alloy Steel".to_string(), "SG2".to_string());
    standoff_abbrevs.insert("Grade 5 Alloy Steel".to_string(), "SG5".to_string());
    standoff_abbrevs.insert("Grade 8 Alloy Steel".to_string(), "SG8".to_string());
    standoff_abbrevs.insert("8.8 Alloy Steel".to_string(), "S8.8".to_string());
    standoff_abbrevs.insert("10.9 Alloy Steel".to_string(), "S10.9".to_string());
    standoff_abbrevs.insert("12.9 Alloy Steel".to_string(), "S12.9".to_string());
    
    // Finish abbreviations for standoffs
    standoff_abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
    standoff_abbrevs.insert("Zinc-Plated".to_string(), "ZP".to_string());
    standoff_abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
    standoff_abbrevs.insert("Zinc Yellow Chromate Plated".to_string(), "ZYC".to_string());
    standoff_abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
    standoff_abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
    standoff_abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
    standoff_abbrevs.insert("Cadmium-Plated".to_string(), "CD".to_string());
    standoff_abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
    standoff_abbrevs.insert("Nickel-Plated".to_string(), "NI".to_string());
    standoff_abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
    standoff_abbrevs.insert("Chrome-Plated".to_string(), "CR".to_string());
    standoff_abbrevs.insert("Galvanized".to_string(), "GAL".to_string());
    
    // Male-Female Threaded Hex Standoff
    let male_female_hex_standoff_template = NamingTemplate {
        prefix: "MFSO".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
        spec_abbreviations: standoff_abbrevs.clone(),
    };
    category_templates.insert("male_female_hex_standoff".to_string(), male_female_hex_standoff_template);
    
    // Female Threaded Hex Standoff
    let female_hex_standoff_template = NamingTemplate {
        prefix: "FSO".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
        spec_abbreviations: standoff_abbrevs.clone(),
    };
    category_templates.insert("female_hex_standoff".to_string(), female_hex_standoff_template);
    
    // Generic Threaded Standoff (fallback)
    let generic_standoff_template = NamingTemplate {
        prefix: "SO".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
        spec_abbreviations: standoff_abbrevs,
    };
    category_templates.insert("generic_standoff".to_string(), generic_standoff_template);
}