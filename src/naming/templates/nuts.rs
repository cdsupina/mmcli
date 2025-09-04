//! Nut naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all nut templates
pub fn initialize_nut_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    // Create shared abbreviations for all nut types
    let nut_abbrevs = create_nut_abbreviations();
    
    // Initialize all nut type templates
    initialize_common_nuts(category_templates, &nut_abbrevs);
    initialize_locking_nuts(category_templates, &nut_abbrevs);
    initialize_specialty_nuts(category_templates, &nut_abbrevs);
}

/// Create comprehensive abbreviations for nut templates
fn create_nut_abbreviations() -> HashMap<String, String> {
    let mut abbrevs = HashMap::new();
    
    // Material abbreviations
    abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
    abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
    abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    abbrevs.insert("Steel".to_string(), "S".to_string());
    abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
    abbrevs.insert("Brass".to_string(), "Brass".to_string());
    abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    abbrevs.insert("Nylon".to_string(), "Nylon".to_string());
    abbrevs.insert("Plastic".to_string(), "Plastic".to_string());
    
    // Steel grade abbreviations
    abbrevs.insert("Grade 1 Steel".to_string(), "SG1".to_string());
    abbrevs.insert("Grade 2 Steel".to_string(), "SG2".to_string());
    abbrevs.insert("Grade 5 Steel".to_string(), "SG5".to_string());
    abbrevs.insert("Grade 8 Steel".to_string(), "SG8".to_string());
    abbrevs.insert("8.8 Steel".to_string(), "S8.8".to_string());
    abbrevs.insert("10.9 Steel".to_string(), "S10.9".to_string());
    abbrevs.insert("12.9 Steel".to_string(), "S12.9".to_string());
    abbrevs.insert("Grade 1 Alloy Steel".to_string(), "SG1".to_string());
    abbrevs.insert("Grade 2 Alloy Steel".to_string(), "SG2".to_string());
    abbrevs.insert("Grade 5 Alloy Steel".to_string(), "SG5".to_string());
    abbrevs.insert("Grade 8 Alloy Steel".to_string(), "SG8".to_string());
    abbrevs.insert("8.8 Alloy Steel".to_string(), "S8.8".to_string());
    abbrevs.insert("10.9 Alloy Steel".to_string(), "S10.9".to_string());
    abbrevs.insert("12.9 Alloy Steel".to_string(), "S12.9".to_string());
    
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

/// Initialize common nut types
fn initialize_common_nuts(category_templates: &mut HashMap<String, NamingTemplate>, abbrevs: &HashMap<String, String>) {
    // Hex Nut
    let hex_nut_template = NamingTemplate {
        prefix: "HN".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("hex_nut".to_string(), hex_nut_template);
    
    // Wing Nut
    let wing_nut_template = NamingTemplate {
        prefix: "WN".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("wing_nut".to_string(), wing_nut_template);
    
    // Cap Nut
    let cap_nut_template = NamingTemplate {
        prefix: "CN".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("cap_nut".to_string(), cap_nut_template);
    
    // Flange Nut
    let flange_nut_template = NamingTemplate {
        prefix: "FN".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("flange_nut".to_string(), flange_nut_template);
    
    // Generic Nut (fallback)
    let generic_nut_template = NamingTemplate {
        prefix: "N".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("generic_nut".to_string(), generic_nut_template);
}

/// Initialize locking nut types  
fn initialize_locking_nuts(category_templates: &mut HashMap<String, NamingTemplate>, abbrevs: &HashMap<String, String>) {
    // Nylon Insert Locknut
    let nylon_insert_locknut_template = NamingTemplate {
        prefix: "LN".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("nylon_insert_locknut".to_string(), nylon_insert_locknut_template);
    
    // Generic Locknut (fallback for all locking types)
    let generic_locknut_template = NamingTemplate {
        prefix: "LN".to_string(),
        key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
        spec_aliases: None,
        spec_abbreviations: abbrevs.clone(),
    };
    category_templates.insert("generic_locknut".to_string(), generic_locknut_template);
    
    // Add other locking nut types with LN prefix...
    let locking_types = [
        "cotter_pin_locknut", "distorted_thread_locknut", "flex_top_locknut",
        "lock_washer_locknut", "serrations_locknut", "spring_stop_locknut",
        "steel_insert_locknut"
    ];
    
    for nut_type in &locking_types {
        let locknut_template = NamingTemplate {
            prefix: "LN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_aliases: None,
            spec_abbreviations: abbrevs.clone(),
        };
        category_templates.insert(nut_type.to_string(), locknut_template);
    }
}

/// Initialize specialty nut types (abbreviated prefixes)
fn initialize_specialty_nuts(category_templates: &mut HashMap<String, NamingTemplate>, abbrevs: &HashMap<String, String>) {
    let specialty_nuts = [
        ("acorn_nut", "AN"),
        ("barrel_nut", "BN"),
        ("cage_nut", "CAGEN"),
        ("castle_nut", "CASN"),
        ("clinch_nut", "CLIN"),
        ("coupling_nut", "COUPN"),
        ("jam_nut", "JN"),
        ("knurled_thumb_nut", "KTN"),
        ("machine_screw_nut", "MSN"),
        ("panel_nut", "PN"),
        ("push_on_nut", "PON"),
        ("rivet_nut", "RN"),
        ("round_nut", "ROUNDN"),
        ("screw_mount_nut", "SMN"),
        ("snap_in_nut", "SIN"),
        ("socket_nut", "SN"),
        ("speed_nut", "SPEEDN"),
        ("square_nut", "SQN"),
        ("tamper_resistant_nut", "TRN"),
        ("threadless_nut", "TLN"),
        ("thumb_nut", "TN"),
        ("tube_end_nut", "TEN"),
        ("twist_close_nut", "TCN"),
        ("weld_nut", "WLN"),
        ("with_pilot_hole_nut", "PHN"),
    ];
    
    for (nut_type, prefix) in &specialty_nuts {
        let nut_template = NamingTemplate {
            prefix: prefix.to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_aliases: None,
            spec_abbreviations: abbrevs.clone(),
        };
        category_templates.insert(nut_type.to_string(), nut_template);
    }
}