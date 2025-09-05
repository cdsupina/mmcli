//! Pulley naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all pulley templates
pub fn initialize_pulley_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    let mut pulley_abbrevs = HashMap::new();
    
    // Material abbreviations for pulleys
    pulley_abbrevs.insert("Steel".to_string(), "S".to_string());
    pulley_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    pulley_abbrevs.insert("303 Stainless Steel".to_string(), "SS303".to_string());
    pulley_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    pulley_abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    pulley_abbrevs.insert("Bronze".to_string(), "BR".to_string());
    pulley_abbrevs.insert("Cast Iron".to_string(), "CI".to_string());
    pulley_abbrevs.insert("Plastic".to_string(), "PL".to_string());
    pulley_abbrevs.insert("Nylon".to_string(), "NYL".to_string());
    
    // Bearing type abbreviations
    pulley_abbrevs.insert("Ball".to_string(), "BALL".to_string());
    pulley_abbrevs.insert("Plain".to_string(), "PLAIN".to_string());
    pulley_abbrevs.insert("Roller".to_string(), "ROLLER".to_string());
    pulley_abbrevs.insert("None".to_string(), "NONE".to_string());
    
    // Application abbreviations
    pulley_abbrevs.insert("For Pulling".to_string(), "PULL".to_string());
    pulley_abbrevs.insert("For Lifting".to_string(), "LIFT".to_string());
    pulley_abbrevs.insert("For Horizontal Pulling".to_string(), "HPULL".to_string());
    
    // Wire Rope Pulley - most specific type
    let wire_rope_pulley_template = NamingTemplate {
        prefix: "WRP".to_string(),
        key_specs: vec!["Material".to_string(), "For Rope Diameter".to_string(), "OD".to_string(), "Bearing Type".to_string()],
        spec_aliases: None,
        spec_abbreviations: pulley_abbrevs.clone(),
    };
    category_templates.insert("wire_rope_pulley".to_string(), wire_rope_pulley_template);
    
    // Generic Rope Pulley (for other rope types)
    let rope_pulley_template = NamingTemplate {
        prefix: "RP".to_string(),
        key_specs: vec!["Material".to_string(), "For Rope Diameter".to_string(), "OD".to_string(), "Bearing Type".to_string()],
        spec_aliases: None,
        spec_abbreviations: pulley_abbrevs.clone(),
    };
    category_templates.insert("rope_pulley".to_string(), rope_pulley_template);
    
    // V-Belt Pulley
    let v_belt_pulley_template = NamingTemplate {
        prefix: "VBP".to_string(),
        key_specs: vec!["Material".to_string(), "For Belt Width".to_string(), "OD".to_string(), "Bearing Type".to_string()],
        spec_aliases: None,
        spec_abbreviations: pulley_abbrevs.clone(),
    };
    category_templates.insert("v_belt_pulley".to_string(), v_belt_pulley_template);
    
    // Generic Pulley (fallback for unspecified types)
    let generic_pulley_template = NamingTemplate {
        prefix: "PUL".to_string(),
        key_specs: vec!["Material".to_string(), "OD".to_string(), "Bearing Type".to_string()],
        spec_aliases: None,
        spec_abbreviations: pulley_abbrevs.clone(),
    };
    category_templates.insert("pulley".to_string(), generic_pulley_template);
    
    // Sheave (alternative name for pulley, especially in lifting applications)
    let sheave_template = NamingTemplate {
        prefix: "SHV".to_string(),
        key_specs: vec!["Material".to_string(), "For Rope Diameter".to_string(), "OD".to_string(), "Bearing Type".to_string()],
        spec_aliases: None,
        spec_abbreviations: pulley_abbrevs.clone(),
    };
    category_templates.insert("sheave".to_string(), sheave_template);
}