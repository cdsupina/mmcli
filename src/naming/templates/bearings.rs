//! Bearing naming templates

use std::collections::HashMap;
use crate::naming::generator::NamingTemplate;

/// Initialize all bearing templates
pub fn initialize_bearing_templates(category_templates: &mut HashMap<String, NamingTemplate>) {
    let mut bearing_abbrevs = HashMap::new();
    
    // Material abbreviations for bearings
    bearing_abbrevs.insert("MDS-Filled Nylon Plastic".to_string(), "MDSNYL".to_string());
    bearing_abbrevs.insert("MDS-Filled Nylon".to_string(), "MDSNYL".to_string());
    bearing_abbrevs.insert("Nylon Plastic".to_string(), "NYL".to_string());
    bearing_abbrevs.insert("Bronze SAE 841".to_string(), "BR841".to_string());
    bearing_abbrevs.insert("Bronze SAE 863".to_string(), "BR863".to_string());
    bearing_abbrevs.insert("Cast Bronze".to_string(), "CB".to_string());
    bearing_abbrevs.insert("Oil-Filled Bronze".to_string(), "OFB".to_string());
    bearing_abbrevs.insert("PTFE".to_string(), "PTFE".to_string());
    bearing_abbrevs.insert("Rulon".to_string(), "RUL".to_string());
    bearing_abbrevs.insert("Graphite".to_string(), "GRAPH".to_string());
    bearing_abbrevs.insert("Steel-Backed PTFE".to_string(), "SBPTFE".to_string());
    bearing_abbrevs.insert("Bronze".to_string(), "BR".to_string());
    bearing_abbrevs.insert("Steel".to_string(), "S".to_string());
    bearing_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
    bearing_abbrevs.insert("303 Stainless Steel".to_string(), "SS303".to_string());
    bearing_abbrevs.insert("Aluminum".to_string(), "AL".to_string());
    bearing_abbrevs.insert("Plastic".to_string(), "PL".to_string());
    
    // Flanged Sleeve Bearing
    let flanged_sleeve_bearing_template = NamingTemplate {
        prefix: "FSB".to_string(),
        key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Length".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("flanged_sleeve_bearing".to_string(), flanged_sleeve_bearing_template);
    
    // Plain Sleeve Bearing
    let sleeve_bearing_template = NamingTemplate {
        prefix: "SB".to_string(),
        key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Length".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("sleeve_bearing".to_string(), sleeve_bearing_template);
    
    // Flanged Bearing (generic)
    let flanged_bearing_template = NamingTemplate {
        prefix: "FB".to_string(),
        key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Length".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("flanged_bearing".to_string(), flanged_bearing_template);
    
    // Ball Bearing
    let ball_bearing_template = NamingTemplate {
        prefix: "BB".to_string(),
        key_specs: vec!["Material".to_string(), "Bore".to_string(), "OD".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("ball_bearing".to_string(), ball_bearing_template);
    
    // Linear Bearing
    let linear_bearing_template = NamingTemplate {
        prefix: "LB".to_string(),
        key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "Length".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("linear_bearing".to_string(), linear_bearing_template);
    
    // Needle Bearing
    let needle_bearing_template = NamingTemplate {
        prefix: "NB".to_string(),
        key_specs: vec!["Material".to_string(), "Bore".to_string(), "OD".to_string(), "Length".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("needle_bearing".to_string(), needle_bearing_template);
    
    // Roller Bearing
    let roller_bearing_template = NamingTemplate {
        prefix: "RB".to_string(),
        key_specs: vec!["Material".to_string(), "Bore".to_string(), "OD".to_string(), "Length".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("roller_bearing".to_string(), roller_bearing_template);
    
    // Flange Mounted Ball Bearing
    let flange_mounted_ball_bearing_template = NamingTemplate {
        prefix: "MFBB".to_string(),
        key_specs: vec!["Housing Material".to_string(), "For Shaft Diameter".to_string(), "Mounting Hole Center -to-Center".to_string(), "Overall Height".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("flange_mounted_ball_bearing".to_string(), flange_mounted_ball_bearing_template);
    
    // Low-Profile Flange Mounted Ball Bearing
    let low_profile_flange_mounted_ball_bearing_template = NamingTemplate {
        prefix: "LPMFBB".to_string(),
        key_specs: vec!["Housing Material".to_string(), "For Shaft Diameter".to_string(), "Mounting Hole Center -to-Center".to_string(), "Overall Height".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("low_profile_flange_mounted_ball_bearing".to_string(), low_profile_flange_mounted_ball_bearing_template);
    
    // Pillow Block Mounted Ball Bearing
    let pillow_block_mounted_ball_bearing_template = NamingTemplate {
        prefix: "PBMBB".to_string(),
        key_specs: vec!["Housing Material".to_string(), "For Shaft Diameter".to_string(), "Mounting Hole Center -to-Center".to_string(), "Overall Height".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("pillow_block_mounted_ball_bearing".to_string(), pillow_block_mounted_ball_bearing_template);
    
    // Generic Mounted Bearing
    let generic_mounted_bearing_template = NamingTemplate {
        prefix: "MBB".to_string(),
        key_specs: vec!["Housing Material".to_string(), "For Shaft Diameter".to_string(), "Overall Height".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs.clone(),
    };
    category_templates.insert("generic_mounted_bearing".to_string(), generic_mounted_bearing_template);
    
    // Generic Bearing (fallback)
    let generic_bearing_template = NamingTemplate {
        prefix: "BRG".to_string(),
        key_specs: vec!["Material".to_string(), "Type".to_string()],
        spec_aliases: None,
        spec_abbreviations: bearing_abbrevs,
    };
    category_templates.insert("generic_bearing".to_string(), generic_bearing_template);
}