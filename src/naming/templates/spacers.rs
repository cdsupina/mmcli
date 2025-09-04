//! Unthreaded spacer naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all unthreaded spacer templates
pub fn initialize_spacer_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    let mut spacer_abbrevs = HashMap::new();
    
    // Material abbreviations for spacers
    spacer_abbrevs.insert("Acetal Plastic".to_string(), "ACET".to_string());
    spacer_abbrevs.insert("Acetal".to_string(), "ACET".to_string());
    spacer_abbrevs.insert("Polyethylene".to_string(), "PE".to_string());
    spacer_abbrevs.insert("Polypropylene".to_string(), "PP".to_string());
    spacer_abbrevs.insert("PEEK".to_string(), "PEEK".to_string());
    spacer_abbrevs.insert("PTFE".to_string(), "PTFE".to_string());
    spacer_abbrevs.insert("Polycarbonate".to_string(), "PC".to_string());
    spacer_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    spacer_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
    spacer_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    spacer_abbrevs.insert("Steel".to_string(), "S".to_string());
    spacer_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
    spacer_abbrevs.insert("Brass".to_string(), "Brass".to_string());
    spacer_abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    spacer_abbrevs.insert("Nylon".to_string(), "Nylon".to_string());
    spacer_abbrevs.insert("Titanium".to_string(), "TI".to_string());
    
    // Finish abbreviations for spacers
    spacer_abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
    spacer_abbrevs.insert("Zinc-Plated".to_string(), "ZP".to_string());
    spacer_abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
    spacer_abbrevs.insert("Zinc Yellow Chromate Plated".to_string(), "ZYC".to_string());
    spacer_abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
    spacer_abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
    spacer_abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
    spacer_abbrevs.insert("Cadmium-Plated".to_string(), "CD".to_string());
    spacer_abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
    spacer_abbrevs.insert("Nickel-Plated".to_string(), "NI".to_string());
    spacer_abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
    spacer_abbrevs.insert("Chrome-Plated".to_string(), "CR".to_string());
    spacer_abbrevs.insert("Galvanized".to_string(), "GAL".to_string());
    spacer_abbrevs.insert("Black Anodized".to_string(), "BA".to_string());
    spacer_abbrevs.insert("Black-Anodized".to_string(), "BA".to_string());
    
    // Create aliases for For Screw Size - prefer "For Screw Size" over precise "ID"
    let mut screw_size_aliases = HashMap::new();
    screw_size_aliases.insert("For Screw Size".to_string(), vec!["For Screw Size".to_string(), "For Screw Size".to_string()]);
    
    // Generic Unthreaded Spacer
    let spacer_template = NamingTemplate {
        prefix: "SP".to_string(),
        key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "OD".to_string(), "Length".to_string(), "Finish".to_string()],
        spec_aliases: Some(screw_size_aliases.clone()),
        spec_abbreviations: spacer_abbrevs.clone(),
    };
    category_templates.insert("unthreaded_spacer".to_string(), spacer_template);
    
    // Aluminum Unthreaded Spacer
    let aluminum_spacer_template = NamingTemplate {
        prefix: "ASP".to_string(),
        key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "OD".to_string(), "Length".to_string(), "Finish".to_string()],
        spec_aliases: Some(screw_size_aliases.clone()),
        spec_abbreviations: spacer_abbrevs.clone(),
    };
    category_templates.insert("aluminum_unthreaded_spacer".to_string(), aluminum_spacer_template);
    
    // Stainless Steel Unthreaded Spacer
    let stainless_spacer_template = NamingTemplate {
        prefix: "SSSP".to_string(),
        key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "OD".to_string(), "Length".to_string(), "Finish".to_string()],
        spec_aliases: Some(screw_size_aliases.clone()),
        spec_abbreviations: spacer_abbrevs.clone(),
    };
    category_templates.insert("stainless_steel_unthreaded_spacer".to_string(), stainless_spacer_template);
    
    // Nylon Unthreaded Spacer
    let nylon_spacer_template = NamingTemplate {
        prefix: "NSP".to_string(),
        key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "OD".to_string(), "Length".to_string()],
        spec_aliases: Some(screw_size_aliases),
        spec_abbreviations: spacer_abbrevs,
    };
    category_templates.insert("nylon_unthreaded_spacer".to_string(), nylon_spacer_template);
}