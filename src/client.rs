use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use dirs::{home_dir, config_dir};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use native_tls::{Identity, TlsConnector};
use std::fs as std_fs;
use std::io::{self, Write};
use std::collections::HashMap;
use crate::OutputFormat;

#[derive(Debug, Clone, PartialEq)]
pub enum ProductField {
    PartNumber,
    DetailDescription,
    FamilyDescription,
    Category,
    Status,
    Specification(String),
    AllSpecs,
    BasicInfo,
}

impl ProductField {
    pub fn parse_fields(fields_str: &str) -> Vec<ProductField> {
        if fields_str == "all" {
            return vec![
                ProductField::PartNumber,
                ProductField::DetailDescription,
                ProductField::FamilyDescription,
                ProductField::Category,
                ProductField::Status,
                ProductField::AllSpecs,
            ];
        }

        if fields_str == "basic" {
            return vec![
                ProductField::PartNumber,
                ProductField::DetailDescription,
                ProductField::FamilyDescription,
                ProductField::Category,
                ProductField::Status,
            ];
        }

        if fields_str == "specs" {
            return vec![ProductField::AllSpecs];
        }

        fields_str
            .split(',')
            .map(|s| s.trim())
            .map(|field| match field {
                "part-number" | "partnumber" => ProductField::PartNumber,
                "detail-description" | "detail" | "description" => ProductField::DetailDescription,
                "family-description" | "family" => ProductField::FamilyDescription,
                "category" | "product-category" => ProductField::Category,
                "status" | "product-status" => ProductField::Status,
                "specs" | "specifications" => ProductField::AllSpecs,
                "basic" => ProductField::BasicInfo,
                _ => {
                    // Convert kebab-case to proper specification name
                    let spec_name = field.replace('-', " ");
                    let spec_name = spec_name.split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => first.to_uppercase().chain(chars).collect(),
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    ProductField::Specification(spec_name)
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct NamingTemplate {
    pub prefix: String,                           // BHCS, FW, BB
    pub key_specs: Vec<String>,                  // ["Material", "Thread Size", "Length"]
    pub spec_abbreviations: HashMap<String, String>, // "316 Stainless Steel" -> "SS316"
}

pub struct NameGenerator {
    category_templates: HashMap<String, NamingTemplate>,
}

impl NameGenerator {
    pub fn new() -> Self {
        let mut generator = NameGenerator {
            category_templates: HashMap::new(),
        };
        generator.init_templates();
        generator
    }

    fn init_templates(&mut self) {
        // Initialize screw templates
        let mut screw_abbrevs = HashMap::new();
        
        // Material abbreviations
        screw_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
        screw_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
        screw_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
        screw_abbrevs.insert("Steel".to_string(), "S".to_string());
        screw_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
        
        // Steel grade abbreviations
        screw_abbrevs.insert("Grade 1 Steel".to_string(), "SG1".to_string());
        screw_abbrevs.insert("Grade 2 Steel".to_string(), "SG2".to_string());
        screw_abbrevs.insert("Grade 5 Steel".to_string(), "SG5".to_string());
        screw_abbrevs.insert("Grade 8 Steel".to_string(), "SG8".to_string());
        screw_abbrevs.insert("8.8 Steel".to_string(), "S8.8".to_string());
        screw_abbrevs.insert("10.9 Steel".to_string(), "S10.9".to_string());
        screw_abbrevs.insert("12.9 Steel".to_string(), "S12.9".to_string());
        
        // Alloy steel grade abbreviations
        screw_abbrevs.insert("Grade 1 Alloy Steel".to_string(), "SG1".to_string());
        screw_abbrevs.insert("Grade 2 Alloy Steel".to_string(), "SG2".to_string());
        screw_abbrevs.insert("Grade 5 Alloy Steel".to_string(), "SG5".to_string());
        screw_abbrevs.insert("Grade 8 Alloy Steel".to_string(), "SG8".to_string());
        screw_abbrevs.insert("8.8 Alloy Steel".to_string(), "S8.8".to_string());
        screw_abbrevs.insert("10.9 Alloy Steel".to_string(), "S10.9".to_string());
        screw_abbrevs.insert("12.9 Alloy Steel".to_string(), "S12.9".to_string());
        screw_abbrevs.insert("Brass".to_string(), "Brass".to_string());
        screw_abbrevs.insert("Aluminum".to_string(), "Al".to_string());
        
        // Finish abbreviations for screws
        screw_abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
        screw_abbrevs.insert("Zinc-Plated".to_string(), "ZP".to_string());
        screw_abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
        screw_abbrevs.insert("Zinc Yellow Chromate Plated".to_string(), "ZYC".to_string());
        screw_abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
        screw_abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
        screw_abbrevs.insert("Passivated".to_string(), "PASS".to_string());
        screw_abbrevs.insert("Plain".to_string(), "PLAIN".to_string());
        screw_abbrevs.insert("Unfinished".to_string(), "UF".to_string());
        screw_abbrevs.insert("Galvanized".to_string(), "GALV".to_string());
        screw_abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
        screw_abbrevs.insert("Cadmium-Plated".to_string(), "CD".to_string());
        screw_abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
        screw_abbrevs.insert("Nickel-Plated".to_string(), "NI".to_string());
        screw_abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
        screw_abbrevs.insert("Chrome-Plated".to_string(), "CR".to_string());
        
        // Drive style abbreviations (comprehensive list from McMaster-Carr)
        screw_abbrevs.insert("4-Flute Spline".to_string(), "4FS".to_string());
        screw_abbrevs.insert("6-Flute Spline".to_string(), "6FS".to_string());
        screw_abbrevs.insert("Asymmetrical".to_string(), "ASYM".to_string());
        screw_abbrevs.insert("Clutch".to_string(), "CLUTCH".to_string());
        screw_abbrevs.insert("Double Square".to_string(), "DSQUARE".to_string());
        screw_abbrevs.insert("Drilled Spanner".to_string(), "DSPAN".to_string());
        screw_abbrevs.insert("External 12-Point".to_string(), "EXT12".to_string());
        screw_abbrevs.insert("External Hex".to_string(), "EHEX".to_string()); // External hex
        screw_abbrevs.insert("External Pentagon".to_string(), "EPENT".to_string());
        screw_abbrevs.insert("External Square".to_string(), "ESQUARE".to_string());
        screw_abbrevs.insert("Frearson".to_string(), "FREAR".to_string());
        screw_abbrevs.insert("Hex".to_string(), "HEX".to_string()); // Internal hex (socket)
        screw_abbrevs.insert("Hex with Pilot Recess".to_string(), "HEXPILOT".to_string());
        screw_abbrevs.insert("Hi-Torque".to_string(), "HITORQUE".to_string());
        screw_abbrevs.insert("Microstix".to_string(), "MICRO".to_string());
        screw_abbrevs.insert("Mortorq®".to_string(), "MORTORQ".to_string());
        screw_abbrevs.insert("Mortorq® Super".to_string(), "MORTORQS".to_string());
        screw_abbrevs.insert("No Drive".to_string(), "NODRIVE".to_string());
        screw_abbrevs.insert("One Way".to_string(), "ONEWAY".to_string());
        screw_abbrevs.insert("Pentagon".to_string(), "PENT".to_string());
        screw_abbrevs.insert("Pentalobe".to_string(), "PLOBE".to_string());
        screw_abbrevs.insert("Phillips".to_string(), "PH".to_string());
        screw_abbrevs.insert("Phillips Terminal Screw".to_string(), "PHTERM".to_string());
        screw_abbrevs.insert("Pozidriv®".to_string(), "PZ".to_string());
        screw_abbrevs.insert("Pozidriv® Terminal Screw".to_string(), "PZTERM".to_string());
        screw_abbrevs.insert("RIBE".to_string(), "RIBE".to_string());
        screw_abbrevs.insert("Slotted".to_string(), "SL".to_string());
        screw_abbrevs.insert("Spring Plunger Driver".to_string(), "SPRING".to_string());
        screw_abbrevs.insert("Square".to_string(), "SQUARE".to_string());
        screw_abbrevs.insert("Square/Phillips".to_string(), "SQPH".to_string());
        screw_abbrevs.insert("Tamper-Resistant Hex".to_string(), "TRHEX".to_string());
        screw_abbrevs.insert("Tamper-Resistant Pentalobe".to_string(), "TRPLOBE".to_string());
        screw_abbrevs.insert("Tamper-Resistant Phillips".to_string(), "TRPH".to_string());
        screw_abbrevs.insert("Tamper-Resistant Square".to_string(), "TRSQUARE".to_string());
        screw_abbrevs.insert("Tamper-Resistant Torx".to_string(), "TRTX".to_string());
        screw_abbrevs.insert("Tamper-Resistant Torx Plus".to_string(), "TRTXP".to_string());
        screw_abbrevs.insert("Torq-Set®".to_string(), "TORQSET".to_string());
        screw_abbrevs.insert("Torx".to_string(), "TX".to_string());
        screw_abbrevs.insert("Torx Plus".to_string(), "TXP".to_string());
        screw_abbrevs.insert("Triangle".to_string(), "TRI".to_string());
        screw_abbrevs.insert("Tri-Groove".to_string(), "TRIGROOVE".to_string());
        screw_abbrevs.insert("Tri-Lobe".to_string(), "TRILOBE".to_string());
        screw_abbrevs.insert("Triple Square".to_string(), "TRISQUARE".to_string());
        screw_abbrevs.insert("Tri-Wing®".to_string(), "TRIWING".to_string());
        screw_abbrevs.insert("Wrench Flats".to_string(), "WFLATS".to_string());
        
        // Button Head Screw template
        let bhs_template = NamingTemplate {
            prefix: "BHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        
        self.category_templates.insert("button_head_screw".to_string(), bhs_template);
        
        // Flat Head Screw template
        let fhs_template = NamingTemplate {
            prefix: "FHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("flat_head_screw".to_string(), fhs_template);
        
        // Flat Head subcategory templates  
        // Narrow Flat Head Screw
        let narrow_fhs_template = NamingTemplate {
            prefix: "NFHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("narrow_flat_head_screw".to_string(), narrow_fhs_template);
        
        // Standard Flat Head Screw
        let standard_fhs_template = NamingTemplate {
            prefix: "SFHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("standard_flat_head_screw".to_string(), standard_fhs_template);
        
        // Undercut Flat Head Screw
        let undercut_fhs_template = NamingTemplate {
            prefix: "UFHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("undercut_flat_head_screw".to_string(), undercut_fhs_template);
        
        // Wide Flat Head Screw
        let wide_fhs_template = NamingTemplate {
            prefix: "WFHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("wide_flat_head_screw".to_string(), wide_fhs_template);
        
        // Socket Head Screw template
        let shs_template = NamingTemplate {
            prefix: "SHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("socket_head_screw".to_string(), shs_template);
        
        // Socket Head subcategory templates
        // High Socket Head Screw
        let high_shs_template = NamingTemplate {
            prefix: "HSHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("high_socket_head_screw".to_string(), high_shs_template);
        
        // Low Socket Head Screw  
        let low_shs_template = NamingTemplate {
            prefix: "LSHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("low_socket_head_screw".to_string(), low_shs_template);
        
        // Ultra Low Socket Head Screw
        let ultra_low_shs_template = NamingTemplate {
            prefix: "ULSHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("ultra_low_socket_head_screw".to_string(), ultra_low_shs_template);
        
        // Standard Socket Head Screw
        let standard_shs_template = NamingTemplate {
            prefix: "SSHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("standard_socket_head_screw".to_string(), standard_shs_template);
        
        // Pan Head Screw template
        let phs_template = NamingTemplate {
            prefix: "PHS".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
                "Drive Style".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("pan_head_screw".to_string(), phs_template);
        
        // Generic screw template
        let generic_screw_template = NamingTemplate {
            prefix: "SCREW".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(), 
                "Length".to_string(),
            ],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("generic_screw".to_string(), generic_screw_template);
        
        // All head type templates
        // 12-Point Head Screw
        let twelve_point_template = NamingTemplate {
            prefix: "12PHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("12_point_head_screw".to_string(), twelve_point_template);
        
        // Domed Head Screw
        let domed_template = NamingTemplate {
            prefix: "DHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("domed_head_screw".to_string(), domed_template);
        
        // Eye Screw
        let eye_template = NamingTemplate {
            prefix: "EYE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("eye_screw".to_string(), eye_template);
        
        // Headless Screw (Set Screw)
        let headless_template = NamingTemplate {
            prefix: "HEADLESS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("headless_screw".to_string(), headless_template);
        
        // Hex Head Screw
        let hex_head_template = NamingTemplate {
            prefix: "HHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("hex_head_screw".to_string(), hex_head_template);
        
        // Hook Screw
        let hook_template = NamingTemplate {
            prefix: "HOOK".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("hook_screw".to_string(), hook_template);
        
        // Knob Screw
        let knob_template = NamingTemplate {
            prefix: "KNOB".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("knob_screw".to_string(), knob_template);
        
        // L-Handle Screw
        let l_handle_template = NamingTemplate {
            prefix: "LHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("l_handle_screw".to_string(), l_handle_template);
        
        // Oval Head Screw
        let oval_template = NamingTemplate {
            prefix: "OHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("oval_head_screw".to_string(), oval_template);
        
        // Oval Head subcategory templates
        // Standard Oval Head Screw
        let standard_oval_template = NamingTemplate {
            prefix: "SOHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("standard_oval_head_screw".to_string(), standard_oval_template);
        
        // Undercut Oval Head Screw
        let undercut_oval_template = NamingTemplate {
            prefix: "UOHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("undercut_oval_head_screw".to_string(), undercut_oval_template);
        
        // Pentagon Head Screw
        let pentagon_head_template = NamingTemplate {
            prefix: "PENTHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("pentagon_head_screw".to_string(), pentagon_head_template);
        
        // Ring Screw
        let ring_template = NamingTemplate {
            prefix: "RING".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("ring_screw".to_string(), ring_template);
        
        // Rounded Head Screw
        let rounded_template = NamingTemplate {
            prefix: "RHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("rounded_head_screw".to_string(), rounded_template);
        
        // Square Head Screw
        let square_head_template = NamingTemplate {
            prefix: "SQHS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("square_head_screw".to_string(), square_head_template);
        
        // Tee Screw
        let tee_template = NamingTemplate {
            prefix: "TEE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("tee_screw".to_string(), tee_template);
        
        // T-Handle Screw
        let t_handle_template = NamingTemplate {
            prefix: "THS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("t_handle_screw".to_string(), t_handle_template);
        
        // Threaded Screw
        let threaded_template = NamingTemplate {
            prefix: "THREADED".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("threaded_screw".to_string(), threaded_template);
        
        // Thumb Screw
        let thumb_template = NamingTemplate {
            prefix: "THUMB".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("thumb_screw".to_string(), thumb_template);
        
        // Thumb Screw subcategory templates
        // Four Arm Thumb Screw
        let four_arm_thumb_template = NamingTemplate {
            prefix: "4ARM".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("four_arm_thumb_screw".to_string(), four_arm_thumb_template);
        
        // Hex Thumb Screw
        let hex_thumb_template = NamingTemplate {
            prefix: "HEXTHUMB".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("hex_thumb_screw".to_string(), hex_thumb_template);
        
        // Multilobe Thumb Screw
        let multilobe_thumb_template = NamingTemplate {
            prefix: "MULTILOBE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("multilobe_thumb_screw".to_string(), multilobe_thumb_template);
        
        // Rectangle Thumb Screw
        let rectangle_thumb_template = NamingTemplate {
            prefix: "RECTTHUMB".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("rectangle_thumb_screw".to_string(), rectangle_thumb_template);
        
        // Round Thumb Screw
        let round_thumb_template = NamingTemplate {
            prefix: "ROUNDTHUMB".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("round_thumb_screw".to_string(), round_thumb_template);
        
        // Spade Thumb Screw
        let spade_thumb_template = NamingTemplate {
            prefix: "SPADE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("spade_thumb_screw".to_string(), spade_thumb_template);
        
        // Two Arm Thumb Screw
        let two_arm_thumb_template = NamingTemplate {
            prefix: "2ARM".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("two_arm_thumb_screw".to_string(), two_arm_thumb_template);
        
        // Wing Thumb Screw
        let wing_thumb_template = NamingTemplate {
            prefix: "WINGTHUMB".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("wing_thumb_screw".to_string(), wing_thumb_template);
        
        // T-Slot Screw
        let t_slot_template = NamingTemplate {
            prefix: "TSLOT".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("t_slot_screw".to_string(), t_slot_template);
        
        // Rounded head subcategory templates
        // Binding Head Screw
        let binding_head_template = NamingTemplate {
            prefix: "BINDING".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("binding_head_screw".to_string(), binding_head_template);
        
        // Carriage Head Screw
        let carriage_head_template = NamingTemplate {
            prefix: "CARRIAGE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("carriage_head_screw".to_string(), carriage_head_template);
        
        // Cheese Head Screw
        let cheese_head_template = NamingTemplate {
            prefix: "CHEESE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("cheese_head_screw".to_string(), cheese_head_template);
        
        // Fillister Head Screw
        let fillister_head_template = NamingTemplate {
            prefix: "FILLISTER".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("fillister_head_screw".to_string(), fillister_head_template);
        
        // Pancake Head Screw
        let pancake_head_template = NamingTemplate {
            prefix: "PANCAKE".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("pancake_head_screw".to_string(), pancake_head_template);
        
        // Round Head Screw
        let round_head_template = NamingTemplate {
            prefix: "ROUND".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("round_head_screw".to_string(), round_head_template);
        
        // Truss Head Screw
        let truss_head_template = NamingTemplate {
            prefix: "TRUSS".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Drive Style".to_string(), "Finish".to_string()],
            spec_abbreviations: screw_abbrevs.clone(),
        };
        self.category_templates.insert("truss_head_screw".to_string(), truss_head_template);
        
        // Washer templates - comprehensive support for all washer types
        let mut washer_abbrevs = HashMap::new();
        washer_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
        washer_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
        washer_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
        washer_abbrevs.insert("Steel".to_string(), "S".to_string());
        washer_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
        
        // Steel grade abbreviations for washers
        washer_abbrevs.insert("Grade 1 Steel".to_string(), "SG1".to_string());
        washer_abbrevs.insert("Grade 2 Steel".to_string(), "SG2".to_string());
        washer_abbrevs.insert("Grade 5 Steel".to_string(), "SG5".to_string());
        washer_abbrevs.insert("Grade 8 Steel".to_string(), "SG8".to_string());
        washer_abbrevs.insert("8.8 Steel".to_string(), "S8.8".to_string());
        washer_abbrevs.insert("10.9 Steel".to_string(), "S10.9".to_string());
        washer_abbrevs.insert("12.9 Steel".to_string(), "S12.9".to_string());
        
        // Alloy steel grade abbreviations for washers
        washer_abbrevs.insert("Grade 1 Alloy Steel".to_string(), "SG1".to_string());
        washer_abbrevs.insert("Grade 2 Alloy Steel".to_string(), "SG2".to_string());
        washer_abbrevs.insert("Grade 5 Alloy Steel".to_string(), "SG5".to_string());
        washer_abbrevs.insert("Grade 8 Alloy Steel".to_string(), "SG8".to_string());
        washer_abbrevs.insert("8.8 Alloy Steel".to_string(), "S8.8".to_string());
        washer_abbrevs.insert("10.9 Alloy Steel".to_string(), "S10.9".to_string());
        washer_abbrevs.insert("12.9 Alloy Steel".to_string(), "S12.9".to_string());
        washer_abbrevs.insert("Brass".to_string(), "Brass".to_string());
        washer_abbrevs.insert("Aluminum".to_string(), "Al".to_string());
        washer_abbrevs.insert("Copper".to_string(), "Cu".to_string());
        washer_abbrevs.insert("Nylon".to_string(), "Nylon".to_string());
        washer_abbrevs.insert("Plastic".to_string(), "Plastic".to_string());
        washer_abbrevs.insert("Rubber".to_string(), "Rubber".to_string());
        
        // Screw size abbreviations for washers
        washer_abbrevs.insert("No. 0".to_string(), "0".to_string());
        washer_abbrevs.insert("No. 1".to_string(), "1".to_string());
        washer_abbrevs.insert("No. 2".to_string(), "2".to_string());
        washer_abbrevs.insert("No. 3".to_string(), "3".to_string());
        washer_abbrevs.insert("No. 4".to_string(), "4".to_string());
        washer_abbrevs.insert("No. 5".to_string(), "5".to_string());
        washer_abbrevs.insert("No. 6".to_string(), "6".to_string());
        washer_abbrevs.insert("No. 8".to_string(), "8".to_string());
        washer_abbrevs.insert("No. 10".to_string(), "10".to_string());
        washer_abbrevs.insert("No. 12".to_string(), "12".to_string());
        washer_abbrevs.insert("No. 14".to_string(), "14".to_string());
        
        // Finish abbreviations for washers
        washer_abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
        washer_abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
        washer_abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
        washer_abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
        washer_abbrevs.insert("Passivated".to_string(), "PASS".to_string());
        washer_abbrevs.insert("Plain".to_string(), "PLAIN".to_string());
        washer_abbrevs.insert("Unfinished".to_string(), "UF".to_string());
        washer_abbrevs.insert("Galvanized".to_string(), "GALV".to_string());
        washer_abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
        washer_abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
        washer_abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
        
        // Cup Washer
        let cup_washer_template = NamingTemplate {
            prefix: "CW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("cup_washer".to_string(), cup_washer_template);
        
        // Curved Washer
        let curved_washer_template = NamingTemplate {
            prefix: "CRVW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("curved_washer".to_string(), curved_washer_template);
        
        // Dished Washer
        let dished_washer_template = NamingTemplate {
            prefix: "DW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("dished_washer".to_string(), dished_washer_template);
        
        // Domed Washer
        let domed_washer_template = NamingTemplate {
            prefix: "DMW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("domed_washer".to_string(), domed_washer_template);
        
        // Double Clipped Washer
        let double_clipped_washer_template = NamingTemplate {
            prefix: "DCW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("double_clipped_washer".to_string(), double_clipped_washer_template);
        
        // Clipped Washer (single clipped)
        let clipped_washer_template = NamingTemplate {
            prefix: "CLW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("clipped_washer".to_string(), clipped_washer_template);
        
        // Flat Washer (default/standard)
        let flat_washer_template = NamingTemplate {
            prefix: "FW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("flat_washer".to_string(), flat_washer_template);
        
        // Hillside Washer
        let hillside_washer_template = NamingTemplate {
            prefix: "HW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("hillside_washer".to_string(), hillside_washer_template);
        
        // Notched Washer
        let notched_washer_template = NamingTemplate {
            prefix: "NW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("notched_washer".to_string(), notched_washer_template);
        
        // Perforated Washer
        let perforated_washer_template = NamingTemplate {
            prefix: "PW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("perforated_washer".to_string(), perforated_washer_template);
        
        // Pronged Washer
        let pronged_washer_template = NamingTemplate {
            prefix: "PRW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("pronged_washer".to_string(), pronged_washer_template);
        
        // Rectangular Washer
        let rectangular_washer_template = NamingTemplate {
            prefix: "RW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("rectangular_washer".to_string(), rectangular_washer_template);
        
        // Sleeve Washer
        let sleeve_washer_template = NamingTemplate {
            prefix: "SW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("sleeve_washer".to_string(), sleeve_washer_template);
        
        // Slotted Washer
        let slotted_washer_template = NamingTemplate {
            prefix: "SLW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("slotted_washer".to_string(), slotted_washer_template);
        
        // Spherical Washer
        let spherical_washer_template = NamingTemplate {
            prefix: "SPW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("spherical_washer".to_string(), spherical_washer_template);
        
        // Split Washer (Lock Washer)
        let split_washer_template = NamingTemplate {
            prefix: "SPLW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("split_washer".to_string(), split_washer_template);
        
        // Square Washer
        let square_washer_template = NamingTemplate {
            prefix: "SQW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("square_washer".to_string(), square_washer_template);
        
        // Tab Washer
        let tab_washer_template = NamingTemplate {
            prefix: "TW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("tab_washer".to_string(), tab_washer_template);
        
        // Tapered Washer
        let tapered_washer_template = NamingTemplate {
            prefix: "TPW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("tapered_washer".to_string(), tapered_washer_template);
        
        // Tooth Washer
        let tooth_washer_template = NamingTemplate {
            prefix: "TOW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("tooth_washer".to_string(), tooth_washer_template);
        
        // Wave Washer
        let wave_washer_template = NamingTemplate {
            prefix: "WW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("wave_washer".to_string(), wave_washer_template);
        
        // Wedge Washer
        let wedge_washer_template = NamingTemplate {
            prefix: "WDW".to_string(),
            key_specs: vec!["Material".to_string(), "For Screw Size".to_string(), "Finish".to_string()],
            spec_abbreviations: washer_abbrevs.clone(),
        };
        self.category_templates.insert("wedge_washer".to_string(), wedge_washer_template);
        
        // Nut templates
        let mut nut_abbrevs = HashMap::new();
        nut_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
        nut_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
        nut_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
        nut_abbrevs.insert("Steel".to_string(), "S".to_string());
        nut_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
        
        // Steel grade abbreviations for nuts
        nut_abbrevs.insert("Grade 1 Steel".to_string(), "SG1".to_string());
        nut_abbrevs.insert("Grade 2 Steel".to_string(), "SG2".to_string());
        nut_abbrevs.insert("Grade 5 Steel".to_string(), "SG5".to_string());
        nut_abbrevs.insert("Grade 8 Steel".to_string(), "SG8".to_string());
        nut_abbrevs.insert("8.8 Steel".to_string(), "S8.8".to_string());
        nut_abbrevs.insert("10.9 Steel".to_string(), "S10.9".to_string());
        nut_abbrevs.insert("12.9 Steel".to_string(), "S12.9".to_string());
        
        // Alloy steel grade abbreviations for nuts
        nut_abbrevs.insert("Grade 1 Alloy Steel".to_string(), "SG1".to_string());
        nut_abbrevs.insert("Grade 2 Alloy Steel".to_string(), "SG2".to_string());
        nut_abbrevs.insert("Grade 5 Alloy Steel".to_string(), "SG5".to_string());
        nut_abbrevs.insert("Grade 8 Alloy Steel".to_string(), "SG8".to_string());
        nut_abbrevs.insert("8.8 Alloy Steel".to_string(), "S8.8".to_string());
        nut_abbrevs.insert("10.9 Alloy Steel".to_string(), "S10.9".to_string());
        nut_abbrevs.insert("12.9 Alloy Steel".to_string(), "S12.9".to_string());
        nut_abbrevs.insert("Brass".to_string(), "Brass".to_string());
        nut_abbrevs.insert("Aluminum".to_string(), "Al".to_string());
        
        // Finish abbreviations for nuts
        nut_abbrevs.insert("Zinc Plated".to_string(), "ZP".to_string());
        nut_abbrevs.insert("Zinc-Plated".to_string(), "ZP".to_string());
        nut_abbrevs.insert("Zinc Yellow-Chromate Plated".to_string(), "ZYC".to_string());
        nut_abbrevs.insert("Zinc Yellow Chromate Plated".to_string(), "ZYC".to_string());
        nut_abbrevs.insert("Black Oxide".to_string(), "BO".to_string());
        nut_abbrevs.insert("Black-Oxide".to_string(), "BO".to_string());
        nut_abbrevs.insert("Passivated".to_string(), "PASS".to_string());
        nut_abbrevs.insert("Plain".to_string(), "PLAIN".to_string());
        nut_abbrevs.insert("Unfinished".to_string(), "UF".to_string());
        nut_abbrevs.insert("Galvanized".to_string(), "GALV".to_string());
        nut_abbrevs.insert("Cadmium Plated".to_string(), "CD".to_string());
        nut_abbrevs.insert("Cadmium-Plated".to_string(), "CD".to_string());
        nut_abbrevs.insert("Nickel Plated".to_string(), "NI".to_string());
        nut_abbrevs.insert("Nickel-Plated".to_string(), "NI".to_string());
        nut_abbrevs.insert("Chrome Plated".to_string(), "CR".to_string());
        nut_abbrevs.insert("Chrome-Plated".to_string(), "CR".to_string());
        
        // Locknut template (nylon-insert, prevailing torque, etc.)
        let locknut_template = NamingTemplate {
            prefix: "LN".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("locknut".to_string(), locknut_template);
        
        // Hex nut template
        let hex_nut_template = NamingTemplate {
            prefix: "HN".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("hex_nut".to_string(), hex_nut_template);
        
        // Wing nut template
        let wing_nut_template = NamingTemplate {
            prefix: "WN".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("wing_nut".to_string(), wing_nut_template);
        
        // Cap nut template
        let cap_nut_template = NamingTemplate {
            prefix: "CN".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("cap_nut".to_string(), cap_nut_template);
        
        // Generic nut template
        let generic_nut_template = NamingTemplate {
            prefix: "N".to_string(),
            key_specs: vec![
                "Material".to_string(),
                "Thread Size".to_string(),
                "Finish".to_string(),
            ],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("generic_nut".to_string(), generic_nut_template);
        
        // Comprehensive nut type templates
        
        // Adhesive Mount Nut
        let adhesive_mount_nut_template = NamingTemplate {
            prefix: "AMN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("adhesive_mount_nut".to_string(), adhesive_mount_nut_template);
        
        // Clip On Nut
        let clip_on_nut_template = NamingTemplate {
            prefix: "CON".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("clip_on_nut".to_string(), clip_on_nut_template);
        
        // Coupling Nut
        let coupling_nut_template = NamingTemplate {
            prefix: "COUP".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("coupling_nut".to_string(), coupling_nut_template);
        
        // Dowel Nut
        let dowel_nut_template = NamingTemplate {
            prefix: "DN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("dowel_nut".to_string(), dowel_nut_template);
        
        // Externally Threaded Nut
        let ext_threaded_nut_template = NamingTemplate {
            prefix: "ETN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("externally_threaded_nut".to_string(), ext_threaded_nut_template);
        
        // Flange Nut
        let flange_nut_template = NamingTemplate {
            prefix: "FN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("flange_nut".to_string(), flange_nut_template);
        
        // Panel Nut
        let panel_nut_template = NamingTemplate {
            prefix: "PN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("panel_nut".to_string(), panel_nut_template);
        
        // Press Fit Nut
        let press_fit_nut_template = NamingTemplate {
            prefix: "PFN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("press_fit_nut".to_string(), press_fit_nut_template);
        
        // Push Button Nut
        let push_button_nut_template = NamingTemplate {
            prefix: "PBN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("push_button_nut".to_string(), push_button_nut_template);
        
        // Push Nut
        let push_nut_template = NamingTemplate {
            prefix: "PUSHN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("push_nut".to_string(), push_nut_template);
        
        // Rivet Mount Nut
        let rivet_mount_nut_template = NamingTemplate {
            prefix: "RMN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("rivet_mount_nut".to_string(), rivet_mount_nut_template);
        
        // Rivet Nut
        let rivet_nut_template = NamingTemplate {
            prefix: "RN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("rivet_nut".to_string(), rivet_nut_template);
        
        // Round Nut
        let round_nut_template = NamingTemplate {
            prefix: "ROUNDN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("round_nut".to_string(), round_nut_template);
        
        // Screw Mount Nut
        let screw_mount_nut_template = NamingTemplate {
            prefix: "SMN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("screw_mount_nut".to_string(), screw_mount_nut_template);
        
        // Snap In Nut
        let snap_in_nut_template = NamingTemplate {
            prefix: "SIN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("snap_in_nut".to_string(), snap_in_nut_template);
        
        // Socket Nut
        let socket_nut_template = NamingTemplate {
            prefix: "SN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("socket_nut".to_string(), socket_nut_template);
        
        // Speed Nut
        let speed_nut_template = NamingTemplate {
            prefix: "SPEEDN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("speed_nut".to_string(), speed_nut_template);
        
        // Square Nut
        let square_nut_template = NamingTemplate {
            prefix: "SQN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("square_nut".to_string(), square_nut_template);
        
        // Tamper Resistant Nut
        let tamper_resistant_nut_template = NamingTemplate {
            prefix: "TRN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("tamper_resistant_nut".to_string(), tamper_resistant_nut_template);
        
        // Threadless Nut
        let threadless_nut_template = NamingTemplate {
            prefix: "TLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("threadless_nut".to_string(), threadless_nut_template);
        
        // Thumb Nut
        let thumb_nut_template = NamingTemplate {
            prefix: "THUMBN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("thumb_nut".to_string(), thumb_nut_template);
        
        // Tube End Nut
        let tube_end_nut_template = NamingTemplate {
            prefix: "TEN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("tube_end_nut".to_string(), tube_end_nut_template);
        
        // Twist Close Nut
        let twist_close_nut_template = NamingTemplate {
            prefix: "TCN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("twist_close_nut".to_string(), twist_close_nut_template);
        
        // Weld Nut
        let weld_nut_template = NamingTemplate {
            prefix: "WELD".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("weld_nut".to_string(), weld_nut_template);
        
        // With Pilot Hole Nut
        let with_pilot_hole_nut_template = NamingTemplate {
            prefix: "WPHN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("with_pilot_hole_nut".to_string(), with_pilot_hole_nut_template);
        
        // Locking nut specific types (these will override the generic locknut when detected)
        
        // Cotter Pin Locknut
        let cotter_pin_locknut_template = NamingTemplate {
            prefix: "CPLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("cotter_pin_locknut".to_string(), cotter_pin_locknut_template);
        
        // Distorted Thread Locknut
        let distorted_thread_locknut_template = NamingTemplate {
            prefix: "DTLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("distorted_thread_locknut".to_string(), distorted_thread_locknut_template);
        
        // Flex-Top Locknut
        let flex_top_locknut_template = NamingTemplate {
            prefix: "FTLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("flex_top_locknut".to_string(), flex_top_locknut_template);
        
        // Lock Washer Locknut
        let lock_washer_locknut_template = NamingTemplate {
            prefix: "LWLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("lock_washer_locknut".to_string(), lock_washer_locknut_template);
        
        // Nylon Insert Locknut (keep existing LN for most common type)
        let nylon_insert_locknut_template = NamingTemplate {
            prefix: "LN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("nylon_insert_locknut".to_string(), nylon_insert_locknut_template);
        
        // Serrations Locknut
        let serrations_locknut_template = NamingTemplate {
            prefix: "SRLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("serrations_locknut".to_string(), serrations_locknut_template);
        
        // Spring-Stop Locknut
        let spring_stop_locknut_template = NamingTemplate {
            prefix: "SSLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("spring_stop_locknut".to_string(), spring_stop_locknut_template);
        
        // Steel Insert Locknut
        let steel_insert_locknut_template = NamingTemplate {
            prefix: "SILN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("steel_insert_locknut".to_string(), steel_insert_locknut_template);
        
        // Thread Forming Locknut
        let thread_forming_locknut_template = NamingTemplate {
            prefix: "TFLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("thread_forming_locknut".to_string(), thread_forming_locknut_template);
        
        // Threadlocker Locknut
        let threadlocker_locknut_template = NamingTemplate {
            prefix: "TLLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("threadlocker_locknut".to_string(), threadlocker_locknut_template);
        
        // Two-Piece Clamp Locknut
        let two_piece_clamp_locknut_template = NamingTemplate {
            prefix: "2PCLN".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Finish".to_string()],
            spec_abbreviations: nut_abbrevs.clone(),
        };
        self.category_templates.insert("two_piece_clamp_locknut".to_string(), two_piece_clamp_locknut_template);
        
        // Threaded Standoff templates
        let mut standoff_abbrevs = HashMap::new();
        
        // Material abbreviations for standoffs
        standoff_abbrevs.insert("316 Stainless Steel".to_string(), "SS316".to_string());
        standoff_abbrevs.insert("18-8 Stainless Steel".to_string(), "SS188".to_string());
        standoff_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
        standoff_abbrevs.insert("Steel".to_string(), "S".to_string());
        standoff_abbrevs.insert("Alloy Steel".to_string(), "S".to_string());
        standoff_abbrevs.insert("Brass".to_string(), "Brass".to_string());
        standoff_abbrevs.insert("Aluminum".to_string(), "Al".to_string());
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
        self.category_templates.insert("male_female_hex_standoff".to_string(), male_female_hex_standoff_template);
        
        // Female Threaded Hex Standoff
        let female_hex_standoff_template = NamingTemplate {
            prefix: "FSO".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: standoff_abbrevs.clone(),
        };
        self.category_templates.insert("female_hex_standoff".to_string(), female_hex_standoff_template);
        
        // Generic Threaded Standoff (fallback)
        let generic_standoff_template = NamingTemplate {
            prefix: "SO".to_string(),
            key_specs: vec!["Material".to_string(), "Thread Size".to_string(), "Length".to_string(), "Finish".to_string()],
            spec_abbreviations: standoff_abbrevs,
        };
        self.category_templates.insert("generic_standoff".to_string(), generic_standoff_template);
        
        // Bearing templates
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
        bearing_abbrevs.insert("Steel".to_string(), "STL".to_string());
        bearing_abbrevs.insert("Stainless Steel".to_string(), "SS".to_string());
        bearing_abbrevs.insert("Aluminum".to_string(), "AL".to_string());
        bearing_abbrevs.insert("Plastic".to_string(), "PL".to_string());
        
        // Flanged Sleeve Bearing
        let flanged_sleeve_bearing_template = NamingTemplate {
            prefix: "FSB".to_string(),
            key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Length".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("flanged_sleeve_bearing".to_string(), flanged_sleeve_bearing_template);
        
        // Plain Sleeve Bearing
        let sleeve_bearing_template = NamingTemplate {
            prefix: "SB".to_string(),
            key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Length".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("sleeve_bearing".to_string(), sleeve_bearing_template);
        
        // Flanged Bearing (generic)
        let flanged_bearing_template = NamingTemplate {
            prefix: "FB".to_string(),
            key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "OD".to_string(), "Length".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("flanged_bearing".to_string(), flanged_bearing_template);
        
        // Ball Bearing
        let ball_bearing_template = NamingTemplate {
            prefix: "BB".to_string(),
            key_specs: vec!["Material".to_string(), "Bore".to_string(), "OD".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("ball_bearing".to_string(), ball_bearing_template);
        
        // Linear Bearing
        let linear_bearing_template = NamingTemplate {
            prefix: "LB".to_string(),
            key_specs: vec!["Material".to_string(), "For Shaft Diameter".to_string(), "Length".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("linear_bearing".to_string(), linear_bearing_template);
        
        // Needle Bearing
        let needle_bearing_template = NamingTemplate {
            prefix: "NB".to_string(),
            key_specs: vec!["Material".to_string(), "Bore".to_string(), "OD".to_string(), "Length".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("needle_bearing".to_string(), needle_bearing_template);
        
        // Roller Bearing
        let roller_bearing_template = NamingTemplate {
            prefix: "RB".to_string(),
            key_specs: vec!["Material".to_string(), "Bore".to_string(), "OD".to_string(), "Length".to_string()],
            spec_abbreviations: bearing_abbrevs.clone(),
        };
        self.category_templates.insert("roller_bearing".to_string(), roller_bearing_template);
        
        // Generic Bearing (fallback)
        let generic_bearing_template = NamingTemplate {
            prefix: "BRG".to_string(),
            key_specs: vec!["Material".to_string(), "Type".to_string()],
            spec_abbreviations: bearing_abbrevs,
        };
        self.category_templates.insert("generic_bearing".to_string(), generic_bearing_template);
    }

    pub fn generate_name(&self, product: &ProductDetail) -> String {
        // Determine category/template based on product data
        let template_key = self.determine_category(product);
        
        if let Some(template) = self.category_templates.get(&template_key) {
            self.apply_template(product, template)
        } else {
            // Fallback to basic naming if no template matches
            self.generate_fallback_name(product)
        }
    }

    fn determine_category(&self, product: &ProductDetail) -> String {
        let family_lower = product.family_description.to_lowercase();
        let category_lower = product.product_category.to_lowercase();
        let _detail_lower = product.detail_description.to_lowercase();
        
        // Check for specific screw head types (order matters - more specific first)
        if family_lower.contains("button head") && family_lower.contains("screw") {
            "button_head_screw".to_string()
        } else if family_lower.contains("high socket head") && family_lower.contains("screw") {
            "high_socket_head_screw".to_string()
        } else if family_lower.contains("low socket head") && family_lower.contains("screw") {
            "low_socket_head_screw".to_string()
        } else if family_lower.contains("ultra low socket head") && family_lower.contains("screw") {
            "ultra_low_socket_head_screw".to_string()
        } else if family_lower.contains("standard socket head") && family_lower.contains("screw") {
            "standard_socket_head_screw".to_string()
        } else if family_lower.contains("socket head") && family_lower.contains("screw") {
            "socket_head_screw".to_string()
        } else if family_lower.contains("narrow flat head") && family_lower.contains("screw") {
            "narrow_flat_head_screw".to_string()
        } else if family_lower.contains("standard flat head") && family_lower.contains("screw") {
            "standard_flat_head_screw".to_string()
        } else if family_lower.contains("undercut flat head") && family_lower.contains("screw") {
            "undercut_flat_head_screw".to_string()
        } else if family_lower.contains("wide flat head") && family_lower.contains("screw") {
            "wide_flat_head_screw".to_string()
        } else if family_lower.contains("flat head") && family_lower.contains("screw") {
            "flat_head_screw".to_string()
        } else if family_lower.contains("pan head") && family_lower.contains("screw") {
            "pan_head_screw".to_string()
        } else if family_lower.contains("hex head") && family_lower.contains("screw") {
            "hex_head_screw".to_string()
        } else if family_lower.contains("standard oval head") && family_lower.contains("screw") {
            "standard_oval_head_screw".to_string()
        } else if family_lower.contains("undercut oval head") && family_lower.contains("screw") {
            "undercut_oval_head_screw".to_string()
        } else if family_lower.contains("oval head") && family_lower.contains("screw") {
            "oval_head_screw".to_string()
        } else if family_lower.contains("square head") && family_lower.contains("screw") {
            "square_head_screw".to_string()
        } else if family_lower.contains("binding head") && family_lower.contains("screw") {
            "binding_head_screw".to_string()
        } else if family_lower.contains("carriage head") && family_lower.contains("screw") {
            "carriage_head_screw".to_string()
        } else if family_lower.contains("cheese head") && family_lower.contains("screw") {
            "cheese_head_screw".to_string()
        } else if family_lower.contains("fillister head") && family_lower.contains("screw") {
            "fillister_head_screw".to_string()
        } else if family_lower.contains("pancake head") && family_lower.contains("screw") {
            "pancake_head_screw".to_string()
        } else if family_lower.contains("round head") && family_lower.contains("screw") {
            "round_head_screw".to_string()
        } else if family_lower.contains("truss head") && family_lower.contains("screw") {
            "truss_head_screw".to_string()
        } else if family_lower.contains("rounded head") && family_lower.contains("screw") {  // More specific than just "rounded"
            "rounded_head_screw".to_string()
        } else if family_lower.contains("12-point") && family_lower.contains("screw") {
            "12_point_head_screw".to_string()
        } else if family_lower.contains("t-handle") && family_lower.contains("screw") {
            "t_handle_screw".to_string()
        } else if family_lower.contains("t-slot") && family_lower.contains("screw") {
            "t_slot_screw".to_string()
        } else if family_lower.contains("l-handle") && family_lower.contains("screw") {
            "l_handle_screw".to_string()
        } else if family_lower.contains("domed") && family_lower.contains("screw") {
            "domed_head_screw".to_string()
        } else if family_lower.contains("headless") && family_lower.contains("screw") {
            "headless_screw".to_string()
        } else if family_lower.contains("pentagon") && family_lower.contains("screw") {
            "pentagon_head_screw".to_string()
        } else if family_lower.contains("four arm thumb") && family_lower.contains("screw") {
            "four_arm_thumb_screw".to_string()
        } else if family_lower.contains("hex thumb") && family_lower.contains("screw") {
            "hex_thumb_screw".to_string()
        } else if family_lower.contains("multilobe thumb") && family_lower.contains("screw") {
            "multilobe_thumb_screw".to_string()
        } else if family_lower.contains("rectangle thumb") && family_lower.contains("screw") {
            "rectangle_thumb_screw".to_string()
        } else if family_lower.contains("round thumb") && family_lower.contains("screw") {
            "round_thumb_screw".to_string()
        } else if family_lower.contains("spade thumb") && family_lower.contains("screw") {
            "spade_thumb_screw".to_string()
        } else if family_lower.contains("two arm thumb") && family_lower.contains("screw") {
            "two_arm_thumb_screw".to_string()
        } else if family_lower.contains("wing thumb") && family_lower.contains("screw") {
            "wing_thumb_screw".to_string()
        } else if family_lower.contains("thumb") && family_lower.contains("screw") {
            "thumb_screw".to_string()
        } else if family_lower.contains("hook") && family_lower.contains("screw") {
            "hook_screw".to_string()
        } else if family_lower.contains("ring") && family_lower.contains("screw") {
            "ring_screw".to_string()
        } else if family_lower.contains("eye") && family_lower.contains("screw") {
            "eye_screw".to_string()
        } else if family_lower.contains("knob") && family_lower.contains("screw") {
            "knob_screw".to_string()
        } else if family_lower.contains("threaded") && family_lower.contains("screw") {
            "threaded_screw".to_string()
        } else if family_lower.contains("tee") && family_lower.contains("screw") {
            "tee_screw".to_string()
        } else if category_lower.contains("screw") || family_lower.contains("screw") {
            "generic_screw".to_string()
        } else if category_lower.contains("washer") || family_lower.contains("washer") {
            // Determine specific washer type
            if family_lower.contains("cup") {
                "cup_washer".to_string()
            } else if family_lower.contains("curved") {
                "curved_washer".to_string()
            } else if family_lower.contains("dished") {
                "dished_washer".to_string()
            } else if family_lower.contains("domed") {
                "domed_washer".to_string()
            } else if family_lower.contains("double clipped") {
                "double_clipped_washer".to_string()
            } else if family_lower.contains("clipped") {
                "clipped_washer".to_string()
            } else if family_lower.contains("flat") {
                "flat_washer".to_string()
            } else if family_lower.contains("hillside") {
                "hillside_washer".to_string()
            } else if family_lower.contains("notched") {
                "notched_washer".to_string()
            } else if family_lower.contains("perforated") {
                "perforated_washer".to_string()
            } else if family_lower.contains("pronged") {
                "pronged_washer".to_string()
            } else if family_lower.contains("rectangular") {
                "rectangular_washer".to_string()
            } else if family_lower.contains("sleeve") {
                "sleeve_washer".to_string()
            } else if family_lower.contains("slotted") {
                "slotted_washer".to_string()
            } else if family_lower.contains("spherical") {
                "spherical_washer".to_string()
            } else if family_lower.contains("split") {
                "split_washer".to_string()
            } else if family_lower.contains("square") {
                "square_washer".to_string()
            } else if family_lower.contains("tab") {
                "tab_washer".to_string()
            } else if family_lower.contains("tapered") {
                "tapered_washer".to_string()
            } else if family_lower.contains("tooth") {
                "tooth_washer".to_string()
            } else if family_lower.contains("wave") {
                "wave_washer".to_string()
            } else if family_lower.contains("wedge") {
                "wedge_washer".to_string()
            } else {
                "flat_washer".to_string() // Default to flat washer
            }
        } else if category_lower.contains("nuts") || category_lower.contains("nut") || family_lower.contains("nut") {
            // Determine specific nut type (more specific types first)
            
            // Locking nut sub-types (most specific first)
            if family_lower.contains("cotter pin") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "cotter_pin_locknut".to_string()
            } else if family_lower.contains("distorted thread") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "distorted_thread_locknut".to_string()
            } else if family_lower.contains("flex-top") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "flex_top_locknut".to_string()
            } else if family_lower.contains("lock washer") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "lock_washer_locknut".to_string()
            } else if family_lower.contains("nylon insert") || family_lower.contains("nylon-insert") {
                "nylon_insert_locknut".to_string()
            } else if family_lower.contains("serrations") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "serrations_locknut".to_string()
            } else if family_lower.contains("spring-stop") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "spring_stop_locknut".to_string()
            } else if family_lower.contains("steel insert") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "steel_insert_locknut".to_string()
            } else if family_lower.contains("thread forming") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "thread_forming_locknut".to_string()
            } else if family_lower.contains("threadlocker") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "threadlocker_locknut".to_string()
            } else if family_lower.contains("two-piece clamp") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
                "two_piece_clamp_locknut".to_string()
            } else if family_lower.contains("locknut") || family_lower.contains("lock nut") || 
                     family_lower.contains("prevailing torque") {
                "locknut".to_string()
            
            // Other nut types
            } else if family_lower.contains("adhesive mount") {
                "adhesive_mount_nut".to_string()
            } else if family_lower.contains("clip on") || family_lower.contains("clip-on") {
                "clip_on_nut".to_string()
            } else if family_lower.contains("coupling") {
                "coupling_nut".to_string()
            } else if family_lower.contains("dowel") {
                "dowel_nut".to_string()
            } else if family_lower.contains("externally threaded") {
                "externally_threaded_nut".to_string()
            } else if family_lower.contains("flange") {
                "flange_nut".to_string()
            } else if family_lower.contains("hex nut") || family_lower.contains("hexnut") {
                "hex_nut".to_string()
            } else if family_lower.contains("panel") {
                "panel_nut".to_string()
            } else if family_lower.contains("press fit") || family_lower.contains("press-fit") {
                "press_fit_nut".to_string()
            } else if family_lower.contains("push button") {
                "push_button_nut".to_string()
            } else if family_lower.contains("push nut") {
                "push_nut".to_string()
            } else if family_lower.contains("rivet mount") {
                "rivet_mount_nut".to_string()
            } else if family_lower.contains("rivet nut") {
                "rivet_nut".to_string()
            } else if family_lower.contains("round nut") {
                "round_nut".to_string()
            } else if family_lower.contains("screw mount") {
                "screw_mount_nut".to_string()
            } else if family_lower.contains("snap in") || family_lower.contains("snap-in") {
                "snap_in_nut".to_string()
            } else if family_lower.contains("socket nut") {
                "socket_nut".to_string()
            } else if family_lower.contains("speed") {
                "speed_nut".to_string()
            } else if family_lower.contains("square") {
                "square_nut".to_string()
            } else if family_lower.contains("tamper resistant") || family_lower.contains("tamper-resistant") {
                "tamper_resistant_nut".to_string()
            } else if family_lower.contains("threadless") {
                "threadless_nut".to_string()
            } else if family_lower.contains("thumb") {
                "thumb_nut".to_string()
            } else if family_lower.contains("tube end") {
                "tube_end_nut".to_string()
            } else if family_lower.contains("twist close") || family_lower.contains("twist-close") {
                "twist_close_nut".to_string()
            } else if family_lower.contains("weld") {
                "weld_nut".to_string()
            } else if family_lower.contains("with pilot hole") {
                "with_pilot_hole_nut".to_string()
            } else if family_lower.contains("wing nut") || family_lower.contains("wingnut") {
                "wing_nut".to_string()
            } else if family_lower.contains("cap nut") || family_lower.contains("capnut") {
                "cap_nut".to_string()
            } else {
                "generic_nut".to_string()
            }
        } else if category_lower.contains("standoffs") || category_lower.contains("standoff") || 
                  family_lower.contains("standoff") || family_lower.contains("spacer") {
            // Determine specific standoff type
            if family_lower.contains("male-female") || family_lower.contains("male female") {
                "male_female_hex_standoff".to_string()
            } else if family_lower.contains("female") && family_lower.contains("threaded") {
                "female_hex_standoff".to_string()
            } else {
                "generic_standoff".to_string()
            }
        } else if category_lower.contains("bearing") || family_lower.contains("bearing") {
            // Determine specific bearing type
            let plain_type = product.specifications.iter()
                .find(|s| s.attribute.eq_ignore_ascii_case("Plain Bearing Type"))
                .and_then(|s| s.values.first())
                .map(|v| v.as_str())
                .unwrap_or("");
                
            if family_lower.contains("flanged") || plain_type.eq_ignore_ascii_case("Flanged") {
                if family_lower.contains("sleeve") || family_lower.contains("plain") {
                    "flanged_sleeve_bearing".to_string()
                } else {
                    "flanged_bearing".to_string()
                }
            } else if family_lower.contains("sleeve") || family_lower.contains("plain") {
                "sleeve_bearing".to_string()
            } else if family_lower.contains("ball") {
                "ball_bearing".to_string()
            } else if family_lower.contains("linear") {
                "linear_bearing".to_string()
            } else if family_lower.contains("needle") {
                "needle_bearing".to_string()
            } else if family_lower.contains("roller") {
                "roller_bearing".to_string()
            } else {
                "generic_bearing".to_string()
            }
        } else {
            "unknown".to_string()
        }
    }

    fn apply_template(&self, product: &ProductDetail, template: &NamingTemplate) -> String {
        let mut name_parts = vec![template.prefix.clone()];
        let mut extracted_finish: Option<String> = None;
        
        for spec_name in &template.key_specs {
            if let Some(spec) = product.specifications.iter()
                .find(|s| s.attribute.eq_ignore_ascii_case(spec_name)) {
                
                let value = spec.values.first().unwrap_or(&"".to_string()).clone();
                
                // Special handling for Material that might include finish
                if spec_name.eq_ignore_ascii_case("Material") {
                    let (material, finish) = self.parse_material_and_finish(&value);
                    
                    // Special handling for bearings with filler material
                    let final_material = if template.prefix.ends_with("B") && (template.prefix.starts_with("FSB") || template.prefix.starts_with("SB") || template.prefix.starts_with("BB")) {
                        // Check for filler material for bearings
                        if let Some(filler_spec) = product.specifications.iter()
                            .find(|s| s.attribute.eq_ignore_ascii_case("Filler Material")) {
                            if let Some(filler_value) = filler_spec.values.first() {
                                if !filler_value.is_empty() && filler_value != "None" && filler_value != "Not Specified" {
                                    // Combine filler with base material
                                    format!("{}-Filled {}", filler_value, material)
                                } else {
                                    material
                                }
                            } else {
                                material
                            }
                        } else {
                            material
                        }
                    } else if material.eq_ignore_ascii_case("Steel") || material.eq_ignore_ascii_case("Alloy Steel") {
                        // Check for steel grade to make steel more descriptive
                        self.get_steel_grade_material(product, &material)
                    } else {
                        material
                    };
                    
                    // Add material abbreviation
                    let material_abbrev = template.spec_abbreviations.get(&final_material)
                        .cloned()
                        .unwrap_or_else(|| self.abbreviate_value(&final_material));
                    if !material_abbrev.is_empty() {
                        name_parts.push(material_abbrev);
                    }
                    
                    // Store extracted finish for later use
                    extracted_finish = finish;
                } else if spec_name.eq_ignore_ascii_case("Finish") {
                    // Check if we have a separate finish spec, or use the extracted one
                    let finish_value = if !value.is_empty() {
                        value.clone()
                    } else {
                        extracted_finish.clone().unwrap_or_default()
                    };
                    
                    if !finish_value.is_empty() {
                        let finish_abbrev = template.spec_abbreviations.get(&finish_value)
                            .cloned()
                            .unwrap_or_else(|| self.abbreviate_value(&finish_value));
                        // Skip passivated finish as it doesn't add meaningful information
                        if !finish_abbrev.is_empty() && finish_abbrev != "PASS" {
                            name_parts.push(finish_abbrev);
                        }
                    }
                } else if spec_name.eq_ignore_ascii_case("Length") {
                    // Special handling for Length - convert fractions to decimals for screws
                    let length_value = self.convert_length_to_decimal(&value);
                    let abbreviated = template.spec_abbreviations.get(&length_value)
                        .cloned()
                        .unwrap_or(length_value);
                    
                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                } else if spec_name.eq_ignore_ascii_case("For Shaft Diameter") || spec_name.eq_ignore_ascii_case("OD") {
                    // Special handling for bearing dimensions - convert fractions to decimals
                    let dimension_value = self.convert_length_to_decimal(&value);
                    let abbreviated = template.spec_abbreviations.get(&dimension_value)
                        .cloned()
                        .unwrap_or(dimension_value);
                    
                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                } else if spec_name.eq_ignore_ascii_case("Thread Size") {
                    // Special handling for Thread Size - extract pitch for metric threads
                    let thread_value = self.extract_thread_with_pitch(product, &value);
                    let abbreviated = template.spec_abbreviations.get(&thread_value)
                        .cloned()
                        .unwrap_or_else(|| self.abbreviate_value(&thread_value));
                    
                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                } else {
                    // Normal handling for other specs
                    let abbreviated = template.spec_abbreviations.get(&value)
                        .cloned()
                        .unwrap_or_else(|| self.abbreviate_value(&value));
                    
                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                }
            } else if spec_name.eq_ignore_ascii_case("Finish") && extracted_finish.is_some() {
                // Handle case where there's no "Finish" attribute but we extracted finish from material
                let finish_value = extracted_finish.clone().unwrap();
                let finish_abbrev = template.spec_abbreviations.get(&finish_value)
                    .cloned()
                    .unwrap_or_else(|| self.abbreviate_value(&finish_value));
                // Skip passivated finish as it doesn't add meaningful information
                if !finish_abbrev.is_empty() && finish_abbrev != "PASS" {
                    name_parts.push(finish_abbrev);
                }
            }
        }
        
        name_parts.join("-")
    }

    fn parse_material_and_finish(&self, material_value: &str) -> (String, Option<String>) {
        // Common finish prefixes that can appear in material specifications
        let finish_prefixes = [
            "Black-Oxide ", "Black Oxide ", "Zinc Plated ", "Zinc-Plated ", 
            "Zinc Yellow-Chromate Plated ", "Zinc Yellow Chromate Plated ",
            "Galvanized ", "Cadmium Plated ", "Cadmium-Plated ", 
            "Nickel Plated ", "Nickel-Plated ", "Chrome Plated ", "Chrome-Plated ",
            "Passivated ", "Plain ", "Unfinished "
        ];
        
        for prefix in &finish_prefixes {
            if material_value.starts_with(prefix) {
                let finish = prefix.trim().to_string();
                let material = material_value.strip_prefix(prefix).unwrap_or(material_value).to_string();
                return (material, Some(finish));
            }
        }
        
        // No finish prefix found, return the whole value as material
        (material_value.to_string(), None)
    }

    fn convert_length_to_decimal(&self, value: &str) -> String {
        // Convert common fractions to decimals for screw lengths
        if value.contains("\"") {
            let clean_value = value.replace("\"", "").replace(" ", "-"); // Convert space format to hyphen format
            match clean_value.as_str() {
                "1/8" => "0.125".to_string(),
                "3/16" => "0.1875".to_string(),
                "1/4" => "0.25".to_string(),
                "5/16" => "0.3125".to_string(),
                "3/8" => "0.375".to_string(),
                "7/16" => "0.4375".to_string(),
                "1/2" => "0.5".to_string(),
                "9/16" => "0.5625".to_string(),
                "5/8" => "0.625".to_string(),
                "11/16" => "0.6875".to_string(),
                "3/4" => "0.75".to_string(),
                "13/16" => "0.8125".to_string(),
                "7/8" => "0.875".to_string(),
                "15/16" => "0.9375".to_string(),
                "1-1/8" => "1.125".to_string(),
                "1-1/4" => "1.25".to_string(),
                "1-3/8" => "1.375".to_string(),
                "1-1/2" => "1.5".to_string(),
                "1-5/8" => "1.625".to_string(),
                "1-3/4" => "1.75".to_string(),
                "1-7/8" => "1.875".to_string(),
                "2-1/4" => "2.25".to_string(),
                "2-1/2" => "2.5".to_string(),
                "2-3/4" => "2.75".to_string(),
                "3-1/4" => "3.25".to_string(),
                "3-1/2" => "3.5".to_string(),
                "3-3/4" => "3.75".to_string(),
                _ => clean_value, // Return as-is if not in our conversion table
            }
        } else if value.contains("mm") {
            // Handle metric lengths - remove "mm" and extra spaces
            value.replace("mm", "").trim().to_string()
        } else {
            // Return as-is for already decimal values
            value.to_string()
        }
    }

    fn get_steel_grade_material(&self, product: &ProductDetail, original_material: &str) -> String {
        // Look for "Fastener Strength Grade/Class" specification to get more descriptive steel naming
        if let Some(grade_spec) = product.specifications.iter()
            .find(|s| s.attribute.eq_ignore_ascii_case("Fastener Strength Grade/Class") || 
                     s.attribute.contains("Grade") || 
                     s.attribute.contains("Strength")) 
        {
            if let Some(grade_value) = grade_spec.values.first() {
                // Determine if we should use Steel or Alloy Steel suffix based on original material
                let steel_suffix = if original_material.eq_ignore_ascii_case("Alloy Steel") {
                    "Alloy Steel"
                } else {
                    "Steel"
                };
                
                // Extract grade number from various formats
                if grade_value.contains("Grade 5") || grade_value.contains("grade 5") {
                    return format!("Grade 5 {}", steel_suffix);
                } else if grade_value.contains("Grade 8") || grade_value.contains("grade 8") {
                    return format!("Grade 8 {}", steel_suffix);
                } else if grade_value.contains("Grade 2") || grade_value.contains("grade 2") {
                    return format!("Grade 2 {}", steel_suffix);
                } else if grade_value.contains("Grade 1") || grade_value.contains("grade 1") {
                    return format!("Grade 1 {}", steel_suffix);
                } else if grade_value.contains("10.9") {
                    return format!("10.9 {}", steel_suffix);
                } else if grade_value.contains("12.9") {
                    return format!("12.9 {}", steel_suffix);
                } else if grade_value.contains("8.8") {
                    return format!("8.8 {}", steel_suffix);
                }
            }
        }
        
        // Fallback to original material if no grade found
        original_material.to_string()
    }

    fn extract_thread_with_pitch(&self, product: &ProductDetail, thread_size: &str) -> String {
        // For metric threads, try to extract pitch from detail description
        if thread_size.starts_with("M") {
            // Look for pattern like "M3 x 0.50 mm Thread" in detail description
            if let Some(captures) = regex::Regex::new(r"(M\d+(?:\.\d+)?)\s*x\s*(\d+\.?\d*)\s*mm\s*Thread")
                .ok()
                .and_then(|re| re.captures(&product.detail_description)) 
            {
                if let (Some(size), Some(pitch)) = (captures.get(1), captures.get(2)) {
                    return format!("{}x{}", size.as_str(), pitch.as_str());
                }
            }
        } else if thread_size.contains("-") {
            // For customary threads like "8-32", convert hyphen to "x"
            return thread_size.replace("-", "x");
        }
        
        // For threads without pitch info, return as-is
        thread_size.to_string()
    }

    fn abbreviate_value(&self, value: &str) -> String {
        // Handle common dimension formats
        if value.contains("\"") {
            // Keep fractions as-is, just remove quotes (for non-length specs like washers)
            value.replace("\"", "").to_string()
        } else {
            // Return as-is for thread sizes and other values
            value.to_string()
        }
    }

    fn generate_fallback_name(&self, product: &ProductDetail) -> String {
        // Simple fallback based on family description
        let family_words: Vec<&str> = product.family_description
            .split_whitespace()
            .take(3)
            .collect();
        
        format!("{}-{}", 
            family_words.join("-").to_uppercase(),
            product.part_number)
    }
}

const BASE_URL: &str = "https://api.mcmaster.com";

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    #[serde(rename = "UserName")]
    pub user_name: String,
    #[serde(rename = "Password")]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "AuthToken")]
    pub token: String,
    #[serde(rename = "ExpirationTS")]
    #[allow(dead_code)]
    pub expiration: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "ErrorCode")]
    #[allow(dead_code)]
    pub error_code: Option<String>,
    #[serde(rename = "ErrorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "ErrorDescription")]
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub certificate_path: Option<String>,
    pub certificate_password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LinkItem {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ProductResponse {
    #[serde(rename = "Links")]
    pub links: Option<Vec<LinkItem>>,
}

#[derive(Debug, Clone)]
pub struct CadFile {
    pub format: CadFormat,
    pub url: String,
    pub key: String, // Original API key like "2-D DWG", "3-D STEP"
}

#[derive(Debug, Clone, PartialEq)]
pub enum CadFormat {
    Dwg,
    Step,
    Dxf,
    Iges,
    Solidworks,
    Sat,
    Edrw,
    Pdf,
}

impl CadFormat {
    fn from_api_key(key: &str) -> Option<Self> {
        match key {
            k if k.contains("DWG") => Some(CadFormat::Dwg),
            k if k.contains("STEP") => Some(CadFormat::Step),
            k if k.contains("DXF") => Some(CadFormat::Dxf),
            k if k.contains("IGES") => Some(CadFormat::Iges),
            k if k.contains("SLDPRT") || k.contains("SLDDRW") || k.contains("Solidworks") => Some(CadFormat::Solidworks),
            k if k.contains("SAT") => Some(CadFormat::Sat),
            k if k.contains("EDRW") => Some(CadFormat::Edrw),
            k if k.contains("PDF") => Some(CadFormat::Pdf),
            _ => None,
        }
    }
    
    fn matches_filter(&self, filter: &str) -> bool {
        match filter {
            "dwg" => matches!(self, CadFormat::Dwg),
            "step" => matches!(self, CadFormat::Step),
            "dxf" => matches!(self, CadFormat::Dxf),
            "iges" => matches!(self, CadFormat::Iges),
            "solidworks" => matches!(self, CadFormat::Solidworks),
            "sat" => matches!(self, CadFormat::Sat),
            "edrw" => matches!(self, CadFormat::Edrw),
            "pdf" => matches!(self, CadFormat::Pdf),
            _ => false,
        }
    }
}

pub struct ProductLinks {
    pub images: Vec<String>,
    pub cad: Vec<CadFile>,
    pub datasheets: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProductInfo {
    #[serde(rename = "PartNumber")]
    pub part_number: Option<String>,
    #[serde(rename = "DetailDescription")]
    pub detail_description: Option<String>,
    #[serde(rename = "FamilyDescription")]
    pub family_description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriceInfo {
    #[serde(rename = "Amount")]
    pub amount: f64,
    #[serde(rename = "MinimumQuantity")]
    pub minimum_quantity: f64,
    #[serde(rename = "UnitOfMeasure")]
    pub unit_of_measure: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductDetail {
    #[serde(rename = "PartNumber")]
    pub part_number: String,
    #[serde(rename = "DetailDescription")]
    pub detail_description: String,
    #[serde(rename = "FamilyDescription")]
    pub family_description: String,
    #[serde(rename = "ProductCategory")]
    pub product_category: String,
    #[serde(rename = "ProductStatus")]
    pub product_status: String,
    #[serde(rename = "Specifications", default)]
    pub specifications: Vec<Specification>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Specification {
    #[serde(rename = "Attribute")]
    pub attribute: String,
    #[serde(rename = "Values")]
    pub values: Vec<String>,
}

pub struct McmasterClient {
    client: Client,
    token: Option<String>,
    credentials: Option<Credentials>,
    quiet_mode: bool, // For suppressing output when in JSON mode
    name_generator: NameGenerator,
}

impl McmasterClient {
    pub fn new_with_credentials(credentials: Option<Credentials>) -> Result<Self> {
        Self::new_with_credentials_internal(credentials, false)
    }

    pub fn new_with_credentials_quiet(credentials: Option<Credentials>) -> Result<Self> {
        Self::new_with_credentials_internal(credentials, true)
    }

    fn new_with_credentials_internal(credentials: Option<Credentials>, quiet: bool) -> Result<Self> {
        let mut client_builder = Client::builder();

        // Try to find and load certificate
        if let Some(ref creds) = credentials {
            let cert_path = if let Some(ref explicit_path) = creds.certificate_path {
                // Use explicitly specified path
                Some(PathBuf::from(explicit_path))
            } else {
                // Try to find certificate in default locations
                Self::find_default_certificate_quiet(quiet)
            };

            if let Some(cert_path) = cert_path {
                if cert_path.exists() {
                    if !quiet {
                        println!("Loading client certificate: {}", cert_path.display());
                    }
                    
                    // Read the PKCS12 file
                    let cert_data = std_fs::read(&cert_path)
                        .context("Failed to read certificate file")?;
                    
                    // Get certificate password
                    let cert_password = creds.certificate_password
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    
                    // Create identity from PKCS12
                    let identity = Identity::from_pkcs12(&cert_data, cert_password)
                        .context("Failed to create identity from PKCS12 certificate")?;
                    
                    // Create TLS connector with the identity
                    let tls_connector = TlsConnector::builder()
                        .identity(identity)
                        .danger_accept_invalid_certs(true)  // API endpoints sometimes need this
                        .danger_accept_invalid_hostnames(true)
                        .build()
                        .context("Failed to build TLS connector")?;
                    
                    client_builder = client_builder.use_preconfigured_tls(tls_connector);
                    if !quiet {
                        println!("Client certificate loaded successfully");
                    }
                } else if creds.certificate_path.is_some() {
                    // Explicit path was provided but file doesn't exist
                    return Err(anyhow::anyhow!("Certificate file not found: {}", cert_path.display()));
                } else {
                    // No explicit path and no default certificate found
                    println!("No client certificate found in default locations:");
                    for location in Self::get_default_cert_locations() {
                        println!("  - {}", location.display());
                    }
                    return Err(anyhow::anyhow!("No client certificate found. Place certificate in default location or specify certificate_path in credentials"));
                }
            }
        }

        let client = client_builder
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            token: None,
            credentials,
            quiet_mode: quiet,
            name_generator: NameGenerator::new(),
        })
    }

    pub fn set_quiet_mode(&mut self, quiet: bool) {
        self.quiet_mode = quiet;
    }

    pub async fn login(&mut self, username: String, password: String) -> Result<()> {
        let login_request = LoginRequest {
            user_name: username,
            password,
        };

        let response = self
            .client
            .post(&format!("{}/v1/login", BASE_URL))
            .json(&login_request)
            .send()
            .await
            .context("Failed to send login request")?;

        println!("Response status: {}", response.status());
        println!("Response headers: {:?}", response.headers());
        
        let response_text = response.text().await.context("Failed to get response text")?;
        println!("Response body: {}", response_text);

        if response_text.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty response from server"));
        }

        // Try to parse as success response first
        if let Ok(login_response) = serde_json::from_str::<LoginResponse>(&response_text) {
            self.token = Some(login_response.token.clone());
            self.save_token(&login_response.token).await?;
            println!("Login successful");
            return Ok(());
        }

        // Try to parse as error response
        if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
            return Err(anyhow::anyhow!(
                "Login failed: {} - {}",
                error.error_message.unwrap_or_default(),
                error.error_description.unwrap_or_default()
            ));
        }

        // If we can't parse as JSON, return the raw response
        return Err(anyhow::anyhow!(
            "Unexpected response format. Response: {}",
            response_text
        ));
    }

    pub async fn logout(&mut self) -> Result<()> {
        if let Some(token) = &self.token {
            let response = self
                .client
                .post(&format!("{}/v1/logout", BASE_URL))
                .bearer_auth(token)
                .send()
                .await
                .context("Failed to send logout request")?;

            if !response.status().is_success() {
                let error: ErrorResponse = response
                    .json()
                    .await
                    .context("Failed to parse error response")?;
                
                println!(
                    "Logout warning: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                );
            }
        }

        self.token = None;
        self.remove_token().await?;
        println!("Logged out");
        Ok(())
    }

    pub async fn load_token(&mut self) -> Result<()> {
        if let Some(token_path) = self.get_token_path() {
            if token_path.exists() {
                let token = fs::read_to_string(&token_path)
                    .await
                    .context("Failed to read token file")?;
                self.token = Some(token.trim().to_string());
            }
        }
        Ok(())
    }

    async fn save_token(&self, token: &str) -> Result<()> {
        if let Some(token_path) = self.get_token_path() {
            if let Some(parent) = token_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .context("Failed to create token directory")?;
            }
            
            fs::write(&token_path, token)
                .await
                .context("Failed to write token file")?;
        }
        Ok(())
    }

    async fn remove_token(&self) -> Result<()> {
        if let Some(token_path) = self.get_token_path() {
            if token_path.exists() {
                fs::remove_file(&token_path)
                    .await
                    .context("Failed to remove token file")?;
            }
        }
        Ok(())
    }

    fn get_token_path(&self) -> Option<PathBuf> {
        // Use XDG config directory first
        if let Some(config_dir) = config_dir() {
            let mut path = config_dir;
            path.push("mmc");
            path.push("token");
            return Some(path);
        }
        
        // Fallback to legacy location
        home_dir().map(|mut path| {
            path.push(".mmcli");
            path.push("token");
            path
        })
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    pub async fn add_product(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products", BASE_URL);
        let request_body = serde_json::json!({
            "URL": format!("https://mcmaster.com/{}", product)
        });
        
        let response = self
            .client
            .put(&url)  // PUT instead of POST
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request_body)
            .send()
            .await
            .context("Failed to add product")?;

        if response.status().is_success() {
            let response_text = response.text().await.context("Failed to get response text")?;
            
            // Try to parse product info for clean display
            if let Ok(product_info) = serde_json::from_str::<ProductInfo>(&response_text) {
                println!("✅ Added {} to subscription", product);
                
                // Build description line
                let mut description_parts = Vec::new();
                if let Some(detail) = &product_info.detail_description {
                    description_parts.push(detail.as_str());
                }
                if let Some(family) = &product_info.family_description {
                    description_parts.push(family.as_str());
                }
                
                if !description_parts.is_empty() {
                    println!("   {}", description_parts.join(" - "));
                }
            } else {
                // Fallback if we can't parse the response
                println!("✅ Product {} added to subscription", product);
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to add product {}. Status: {}. Response: {}",
                product, status, error_text
            ));
        }

        Ok(())
    }

    pub async fn remove_product(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products", BASE_URL);
        let request_body = serde_json::json!({
            "URL": format!("https://mcmaster.com/{}", product)
        });
        
        let response = self
            .client
            .delete(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .json(&request_body)
            .send()
            .await
            .context("Failed to remove product")?;

        if response.status().is_success() {
            println!("✅ Removed {} from subscription", product);
        } else {
            // Try to parse as JSON error response, but handle parsing failures gracefully
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(anyhow::anyhow!(
                    "Failed to remove product: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to remove product {}. Status: {}. Response: {}",
                    product, status, response_text
                ));
            }
        }

        Ok(())
    }

    pub async fn get_product(&self, product: &str, output_format: OutputFormat, fields_str: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get product")?;

        if response.status().is_success() {
            let response_text = response.text().await.context("Failed to get response text")?;
            let fields = ProductField::parse_fields(fields_str);
            
            match output_format {
                OutputFormat::Human => {
                    // Try to parse as structured product data for clean display
                    if let Ok(product_detail) = serde_json::from_str::<ProductDetail>(&response_text) {
                        println!("{}", self.format_product_output(&product_detail, &fields));
                    } else {
                        // Fallback to pretty-printed JSON if parsing fails
                        let product_data: serde_json::Value = serde_json::from_str(&response_text)
                            .context("Failed to parse product response")?;
                        println!("{}", serde_json::to_string_pretty(&product_data)?);
                    }
                },
                OutputFormat::Json => {
                    // For JSON output, try to parse and create filtered output
                    if let Ok(product_detail) = serde_json::from_str::<ProductDetail>(&response_text) {
                        let filtered_json = self.create_filtered_json(&product_detail, &fields)?;
                        println!("{}", serde_json::to_string_pretty(&filtered_json)?);
                    } else {
                        // Fallback to original response if parsing fails
                        let product_data: serde_json::Value = serde_json::from_str(&response_text)
                            .context("Failed to parse product response")?;
                        println!("{}", serde_json::to_string_pretty(&product_data)?);
                    }
                }
            }
        } else if response.status().as_u16() == 403 {
            // Product is not in subscription - offer to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "y" || input == "yes" || input.is_empty() {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
                println!("✅ Product added! Getting product information...");
                
                // Retry the product request after adding to subscription
                let url = format!("{}/v1/products/{}", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get product after adding to subscription")?;
                
                if response.status().is_success() {
                    let product_data: serde_json::Value = response
                        .json()
                        .await
                        .context("Failed to parse product response")?;
                    
                    println!("{}", serde_json::to_string_pretty(&product_data)?);
                    return Ok(());
                } else {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Failed to get product after adding to subscription. Status: {}. Response: {}",
                        status, response_text
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription. Add it first with: mmc add {}",
                    product, product
                ));
            }
        } else {
            // Try to parse as JSON error response, but handle parsing failures gracefully
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get product: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to get product. Status: {}. Response: {}",
                    status,
                    response_text
                ));
            }
        }

        Ok(())
    }

    pub async fn get_price(&self, product: &str, output_format: OutputFormat) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}/price", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get price")?;

        if response.status().is_success() {
            let price_data: Vec<PriceInfo> = response
                .json()
                .await
                .context("Failed to parse price response")?;
            
            match output_format {
                OutputFormat::Human => {
                    self.format_price_output(product, &price_data);
                },
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&price_data)?);
                }
            }
        } else if response.status().as_u16() == 403 {
            // Product is not in subscription - offer to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "y" || input == "yes" || input.is_empty() {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
                println!("✅ Product added! Getting price information...");
                
                // Retry the price request after adding to subscription
                let url = format!("{}/v1/products/{}/price", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get price after adding to subscription")?;
                
                if response.status().is_success() {
                    let price_data: Vec<PriceInfo> = response
                        .json()
                        .await
                        .context("Failed to parse price response")?;
                    
                    match output_format {
                        OutputFormat::Human => {
                            self.format_price_output(product, &price_data);
                        },
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&price_data)?);
                        }
                    }
                    return Ok(());
                } else {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Failed to get price after adding to subscription. Status: {}. Response: {}",
                        status, response_text
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription. Add it first with: mmc add {}",
                    product, product
                ));
            }
        } else {
            // Try to parse as JSON error response, but handle parsing failures gracefully
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(anyhow::anyhow!(
                    "Failed to get price: {} - {}",
                    error.error_message.unwrap_or_default(),
                    error.error_description.unwrap_or_default()
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to get price. Status: {}. Response: {}",
                    status,
                    response_text
                ));
            }
        }

        Ok(())
    }

    pub async fn get_changes(&self, start_date: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/changes?start={}", BASE_URL, urlencoding::encode(start_date));
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get changes")?;

        println!("Changes response status: {}", response.status());
        let response_text = response.text().await.context("Failed to get response text")?;
        println!("Changes response: {}", response_text);

        // Try to parse as JSON
        if let Ok(changes_data) = serde_json::from_str::<serde_json::Value>(&response_text) {
            println!("{}", serde_json::to_string_pretty(&changes_data)?);
        } else {
            return Err(anyhow::anyhow!(
                "Unexpected response format for changes: {}",
                response_text
            ));
        }

        Ok(())
    }

    fn ensure_authenticated(&self) -> Result<()> {
        if !self.is_authenticated() {
            return Err(anyhow::anyhow!(
                "Not authenticated. Please login first with: mmcli login -u <username> -p <password>"
            ));
        }
        Ok(())
    }


    pub async fn login_with_stored_credentials(&mut self) -> Result<()> {
        let credentials = self.credentials.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No credentials loaded. Please login first."))?;

        self.login(credentials.username.clone(), credentials.password.clone()).await
    }

    pub async fn save_credentials_template(&self, path: &str) -> Result<()> {
        let template = if path.ends_with(".json") {
            // For JSON, we can't easily add comments, so include optional field
            serde_json::to_string_pretty(&Credentials {
                username: "your_username".to_string(),
                password: "your_password".to_string(),
                certificate_path: Some("~/.config/mmc/certificate.pfx".to_string()),
                certificate_password: Some("certificate_password".to_string()),
            })?
        } else if path.ends_with(".toml") {
            // For TOML, we can add comments explaining the defaults
            format!(
r#"username = "your_username"
password = "your_password"

# Certificate settings (optional - will auto-discover if not specified)
# Default locations checked:
#   ~/.config/mmc/certificate.pfx
#   ~/.config/mmc/certificate.p12  
#   ~/.mmcli/certificate.pfx (legacy)
#   ~/.mmcli/certificate.p12 (legacy)
certificate_path = "~/.config/mmc/certificate.pfx"
certificate_password = "certificate_password"
"#)
        } else {
            return Err(anyhow::anyhow!("Unsupported file format. Use .json or .toml"));
        };

        fs::write(path, template)
            .await
            .context("Failed to write credentials template")?;

        println!("Credentials template saved to: {}", path);
        println!("Please edit the file with your actual credentials.");
        println!("Certificate will be auto-discovered from ~/.config/mmc/certificate.pfx if certificate_path is not specified.");
        Ok(())
    }


    fn get_default_cert_locations() -> Vec<PathBuf> {
        let mut locations = Vec::new();
        
        // XDG config directory locations (preferred)
        if let Some(config_dir) = config_dir() {
            let mut cert_path = config_dir;
            cert_path.push("mmc");
            
            // Try different common extensions
            for ext in &["pfx", "p12"] {
                let mut path = cert_path.clone();
                path.push(format!("certificate.{}", ext));
                locations.push(path);
            }
        }
        
        // Legacy locations (fallback)
        if let Some(home) = home_dir() {
            let mut legacy_path = home;
            legacy_path.push(".mmcli");
            
            for ext in &["pfx", "p12"] {
                let mut path = legacy_path.clone();
                path.push(format!("certificate.{}", ext));
                locations.push(path);
            }
        }
        
        locations
    }

    fn find_default_certificate() -> Option<PathBuf> {
        Self::find_default_certificate_quiet(false)
    }

    fn find_default_certificate_quiet(quiet: bool) -> Option<PathBuf> {
        for location in Self::get_default_cert_locations() {
            if location.exists() {
                if !quiet {
                    println!("Found certificate at: {}", location.display());
                }
                return Some(location);
            }
        }
        None
    }

    // Helper method to get product links
    async fn get_product_links(&self, product: &str) -> Result<ProductLinks> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get product information")?;

        if response.status().is_success() {
            let product_data: ProductResponse = response
                .json()
                .await
                .context("Failed to parse product response")?;
            
            if let Some(link_items) = product_data.links {
                let mut images = Vec::new();
                let mut cad = Vec::new();
                let mut datasheets = Vec::new();
                
                for link in link_items {
                    match link.key.as_str() {
                        "Image" => images.push(link.value),
                        key if CadFormat::from_api_key(key).is_some() => {
                            if let Some(format) = CadFormat::from_api_key(key) {
                                cad.push(CadFile {
                                    format,
                                    url: link.value,
                                    key: link.key,
                                });
                            }
                        },
                        "Datasheet" | "Data Sheet" => datasheets.push(link.value),
                        _ => {} // Ignore other link types like "Price", "ProductDetail"
                    }
                }
                
                Ok(ProductLinks {
                    images,
                    cad,
                    datasheets,
                })
            } else {
                Err(anyhow::anyhow!("No asset links found for product {}", product))
            }
        } else if response.status().as_u16() == 403 {
            // Product is not in subscription - offer to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "y" || input == "yes" || input.is_empty() {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
                println!("✅ Product added! Getting asset links...");
                
                // Retry the request after adding to subscription
                let url = format!("{}/v1/products/{}", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get product information after adding to subscription")?;
                
                if response.status().is_success() {
                    let product_data: ProductResponse = response
                        .json()
                        .await
                        .context("Failed to parse product response")?;
                    
                    if let Some(link_items) = product_data.links {
                        let mut images = Vec::new();
                        let mut cad = Vec::new();
                        let mut datasheets = Vec::new();
                        
                        for link in link_items {
                            match link.key.as_str() {
                                "Image" => images.push(link.value),
                                key if CadFormat::from_api_key(key).is_some() => {
                                    if let Some(format) = CadFormat::from_api_key(key) {
                                        cad.push(CadFile {
                                            format,
                                            url: link.value,
                                            key: link.key,
                                        });
                                    }
                                },
                                "Datasheet" | "Data Sheet" => datasheets.push(link.value),
                                _ => {} // Ignore other link types like "Price", "ProductDetail"
                            }
                        }
                        
                        return Ok(ProductLinks {
                            images,
                            cad,
                            datasheets,
                        });
                    } else {
                        return Err(anyhow::anyhow!("No asset links found for product {}", product));
                    }
                } else {
                    let status = response.status();
                    let response_text = response.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Failed to get product information after adding to subscription. Status: {}. Response: {}",
                        status, response_text
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription. Add it first with: mmc add {}",
                    product, product
                ));
            }
        } else {
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to get product information. Status: {}. Response: {}",
                status,
                response_text
            ));
        }
    }

    // Helper method to get default download directory
    fn get_default_download_dir(product: &str, asset_type: &str) -> Result<PathBuf> {
        let home = home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        
        let mut path = home;
        path.push("Downloads");
        path.push("mmc");
        path.push(product);
        path.push(asset_type);
        
        Ok(path)
    }

    // Helper method to ensure directory exists
    async fn ensure_directory_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        }
        Ok(())
    }

    // Helper method to format price output
    fn format_price_output(&self, product: &str, price_data: &[PriceInfo]) {
        println!("💰 Price for {}:", product);
        
        for price in price_data {
            // Format the unit of measure (singular if minimum is 1)
            let unit = if price.minimum_quantity == 1.0 && price.unit_of_measure.ends_with('s') {
                // Remove the 's' for singular form when minimum is 1
                price.unit_of_measure.trim_end_matches('s')
            } else {
                &price.unit_of_measure
            };
            
            println!("   ${:.2} per {}", price.amount, unit.to_lowercase());
            
            if price.minimum_quantity > 1.0 {
                println!("   Minimum order: {} {}", 
                    if price.minimum_quantity.fract() == 0.0 {
                        format!("{:.0}", price.minimum_quantity)
                    } else {
                        format!("{}", price.minimum_quantity)
                    },
                    price.unit_of_measure.to_lowercase()
                );
            } else {
                println!("   Minimum order: 1 {}", unit.to_lowercase());
            }
        }
    }

    fn format_product_output(&self, product: &ProductDetail, fields: &[ProductField]) -> String {
        let mut output = String::new();
        
        // Always show part number as header if it's included
        if fields.iter().any(|f| matches!(f, ProductField::PartNumber)) {
            output.push_str(&format!("🔧 {}", product.part_number));
            if fields.iter().any(|f| matches!(f, ProductField::Status)) {
                output.push_str(&format!(" ({})", product.product_status));
            }
            output.push('\n');
        }
        
        // Show selected basic fields
        let mut has_descriptions = false;
        for field in fields {
            match field {
                ProductField::DetailDescription => {
                    output.push_str(&format!("   {}\n", product.detail_description));
                    has_descriptions = true;
                },
                ProductField::FamilyDescription => {
                    output.push_str(&format!("   {}\n", product.family_description));
                    has_descriptions = true;
                },
                ProductField::Category => {
                    output.push_str(&format!("   Category: {}\n", product.product_category));
                    has_descriptions = true;
                },
                _ => {} // Handle specifications separately
            }
        }
        
        // Handle specifications
        let spec_fields: Vec<_> = fields.iter()
            .filter_map(|f| match f {
                ProductField::Specification(name) => Some(name.clone()),
                ProductField::AllSpecs => Some("*".to_string()), // Special marker for all specs
                _ => None,
            })
            .collect();
            
        if !spec_fields.is_empty() && !product.specifications.is_empty() {
            if has_descriptions {
                output.push('\n');
            }
            output.push_str("📋 Specifications:\n");
            
            if spec_fields.contains(&"*".to_string()) {
                // Show all specs with important ones first
                let important_specs = ["Thread Size", "Length", "Material", "Drive Style", "Drive Size", "Head Diameter"];
                for spec_name in &important_specs {
                    if let Some(spec) = product.specifications.iter().find(|s| s.attribute == *spec_name) {
                        output.push_str(&format!("   {}: {}\n", spec.attribute, spec.values.join(", ")));
                    }
                }
                
                for spec in &product.specifications {
                    if !important_specs.contains(&spec.attribute.as_str()) {
                        output.push_str(&format!("   {}: {}\n", spec.attribute, spec.values.join(", ")));
                    }
                }
            } else {
                // Show only requested specifications
                for spec_name in &spec_fields {
                    if let Some(spec) = product.specifications.iter().find(|s| 
                        s.attribute.eq_ignore_ascii_case(spec_name) ||
                        s.attribute.to_lowercase().contains(&spec_name.to_lowercase())
                    ) {
                        output.push_str(&format!("   {}: {}\n", spec.attribute, spec.values.join(", ")));
                    }
                }
            }
        }
        
        output
    }

    fn create_filtered_json(&self, product: &ProductDetail, fields: &[ProductField]) -> Result<serde_json::Value> {
        let mut json_obj = serde_json::Map::new();
        
        for field in fields {
            match field {
                ProductField::PartNumber => {
                    json_obj.insert("PartNumber".to_string(), serde_json::Value::String(product.part_number.clone()));
                },
                ProductField::DetailDescription => {
                    json_obj.insert("DetailDescription".to_string(), serde_json::Value::String(product.detail_description.clone()));
                },
                ProductField::FamilyDescription => {
                    json_obj.insert("FamilyDescription".to_string(), serde_json::Value::String(product.family_description.clone()));
                },
                ProductField::Category => {
                    json_obj.insert("ProductCategory".to_string(), serde_json::Value::String(product.product_category.clone()));
                },
                ProductField::Status => {
                    json_obj.insert("ProductStatus".to_string(), serde_json::Value::String(product.product_status.clone()));
                },
                ProductField::AllSpecs => {
                    let specs: Vec<serde_json::Value> = product.specifications.iter()
                        .map(|spec| serde_json::json!({
                            "Attribute": spec.attribute,
                            "Values": spec.values
                        }))
                        .collect();
                    json_obj.insert("Specifications".to_string(), serde_json::Value::Array(specs));
                },
                ProductField::Specification(spec_name) => {
                    if let Some(spec) = product.specifications.iter().find(|s| 
                        s.attribute.eq_ignore_ascii_case(spec_name) ||
                        s.attribute.to_lowercase().contains(&spec_name.to_lowercase())
                    ) {
                        // Add to a specifications object
                        let mut specs_obj = if let Some(existing) = json_obj.get_mut("Specifications") {
                            if let serde_json::Value::Object(ref mut obj) = existing {
                                obj.clone()
                            } else {
                                serde_json::Map::new()
                            }
                        } else {
                            serde_json::Map::new()
                        };
                        
                        specs_obj.insert(spec.attribute.clone(), serde_json::Value::Array(
                            spec.values.iter().map(|v| serde_json::Value::String(v.clone())).collect()
                        ));
                        
                        json_obj.insert("Specifications".to_string(), serde_json::Value::Object(specs_obj));
                    }
                },
                ProductField::BasicInfo => {
                    json_obj.insert("PartNumber".to_string(), serde_json::Value::String(product.part_number.clone()));
                    json_obj.insert("DetailDescription".to_string(), serde_json::Value::String(product.detail_description.clone()));
                    json_obj.insert("FamilyDescription".to_string(), serde_json::Value::String(product.family_description.clone()));
                    json_obj.insert("ProductCategory".to_string(), serde_json::Value::String(product.product_category.clone()));
                    json_obj.insert("ProductStatus".to_string(), serde_json::Value::String(product.product_status.clone()));
                },
            }
        }
        
        Ok(serde_json::Value::Object(json_obj))
    }

    pub async fn generate_name(&self, product: &str) -> Result<()> {
        self.ensure_authenticated()?;
        
        let url = format!("{}/v1/products/{}", BASE_URL, product);
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .context("Failed to get product")?;

        if response.status().is_success() {
            let response_text = response.text().await.context("Failed to get response text")?;
            
            if let Ok(product_detail) = serde_json::from_str::<ProductDetail>(&response_text) {
                // Print descriptions for verification
                println!("{}", product_detail.family_description);
                println!("{}", product_detail.detail_description);
                let human_name = self.name_generator.generate_name(&product_detail);
                println!("{}", human_name);
            } else {
                return Err(anyhow::anyhow!("Failed to parse product data for name generation"));
            }
        } else if response.status().as_u16() == 403 {
            // Product is not in subscription - offer to add it
            println!("❌ Product {} is not in your subscription.", product);
            print!("Would you like to add it to your subscription? (Y/n): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            
            if input == "y" || input == "yes" || input.is_empty() {
                println!("Adding product {} to subscription...", product);
                self.add_product(product).await?;
                println!("✅ Product added! Generating name...");
                
                // Retry the product request after adding to subscription
                let url = format!("{}/v1/products/{}", BASE_URL, product);
                let response = self
                    .client
                    .get(&url)
                    .bearer_auth(self.token.as_ref().unwrap())
                    .send()
                    .await
                    .context("Failed to get product after adding to subscription")?;
                
                if response.status().is_success() {
                    let response_text = response.text().await.context("Failed to get response text")?;
                    
                    if let Ok(product_detail) = serde_json::from_str::<ProductDetail>(&response_text) {
                        // Print descriptions for verification
                        println!("{}", product_detail.family_description);
                        println!("{}", product_detail.detail_description);
                        let human_name = self.name_generator.generate_name(&product_detail);
                        println!("{}", human_name);
                    } else {
                        return Err(anyhow::anyhow!("Failed to parse product data for name generation"));
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "Failed to get product {} after adding to subscription. Status: {}",
                        product, response.status()
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Product {} is not in your subscription. Add it first with: mmc add {}",
                    product, product
                ));
            }
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to get product {}. Status: {}. Response: {}",
                product, status, error_text
            ));
        }

        Ok(())
    }

    // Helper method to download a single asset
    async fn download_asset(&self, asset_path: &str, output_dir: &Path, product: &str) -> Result<String> {
        let url = format!("{}{}", BASE_URL, asset_path);
        
        // Extract file extension from original filename
        let original_filename = asset_path
            .split('/')
            .last()
            .unwrap_or("download");
        
        let extension = if let Some(dot_pos) = original_filename.rfind('.') {
            &original_filename[dot_pos..].to_lowercase()
        } else {
            ""
        };
        
        // Generate clean filename: product.ext or product_variant.ext
        let final_filename = if asset_path.contains("NO%20THREADS") || asset_path.contains("NO THREADS") {
            format!("{}_no_threads{}", product, extension)
        } else if asset_path.contains("3D_") || asset_path.contains("3-D") {
            // For 3D PDFs or other 3D variants, add _3d suffix
            if extension == ".pdf" && (asset_path.contains("3D_") || asset_path.contains("3-D")) {
                format!("{}_3d{}", product, extension)
            } else {
                format!("{}{}", product, extension)
            }
        } else {
            format!("{}{}", product, extension)
        };
        
        let output_path = output_dir.join(&final_filename);
        
        println!("Downloading {}...", final_filename);
        
        let response = self
            .client
            .get(&url)
            .bearer_auth(self.token.as_ref().unwrap())
            .send()
            .await
            .with_context(|| format!("Failed to download {}", final_filename))?;
        
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            let mut file = fs::File::create(&output_path).await
                .with_context(|| format!("Failed to create file: {}", output_path.display()))?;
            
            file.write_all(&bytes).await
                .with_context(|| format!("Failed to write file: {}", output_path.display()))?;
            
            println!("✅ Downloaded {} ({} bytes)", final_filename, bytes.len());
            Ok(final_filename)
        } else {
            Err(anyhow::anyhow!(
                "Failed to download {}. Status: {}",
                final_filename,
                response.status()
            ))
        }
    }

    pub async fn download_images(&self, product: &str, output_dir: Option<&str>) -> Result<()> {
        let links = self.get_product_links(product).await?;
        
        let output_path = if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            Self::get_default_download_dir(product, "images")?
        };
        
        Self::ensure_directory_exists(&output_path).await?;
        
        if links.images.is_empty() {
            println!("No images available for product {}", product);
            return Ok(());
        }
        
        println!("Found {} image(s) for product {}", links.images.len(), product);
        
        let mut downloaded = 0;
        for image_path in &links.images {
            match self.download_asset(image_path, &output_path, product).await {
                Ok(_) => downloaded += 1,
                Err(e) => println!("⚠️  Failed to download image: {}", e),
            }
        }
        
        println!("\n✅ Downloaded {}/{} images to: {}", 
            downloaded, links.images.len(), output_path.display());
        
        Ok(())
    }

    pub async fn download_cad(&self, product: &str, output_dir: Option<&str>, formats: &[&str], download_all: bool) -> Result<()> {
        let links = self.get_product_links(product).await?;
        
        let output_path = if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            Self::get_default_download_dir(product, "cad")?
        };
        
        Self::ensure_directory_exists(&output_path).await?;
        
        if links.cad.is_empty() {
            println!("No CAD files available for product {}", product);
            return Ok(());
        }
        
        // Filter CAD files based on requested formats
        let files_to_download: Vec<&CadFile> = if download_all {
            links.cad.iter().collect()
        } else {
            links.cad.iter()
                .filter(|cad_file| formats.iter().any(|&format| cad_file.format.matches_filter(format)))
                .collect()
        };
        
        if files_to_download.is_empty() {
            if !download_all {
                println!("No CAD files found matching the requested formats: {}", formats.join(", "));
                println!("Available formats for product {}: {:?}", product, 
                    links.cad.iter().map(|f| &f.key).collect::<Vec<_>>());
            }
            return Ok(());
        }
        
        if !download_all && !formats.is_empty() {
            println!("Found {} CAD file(s) matching requested formats [{}] for product {}", 
                files_to_download.len(), formats.join(", "), product);
        } else {
            println!("Found {} CAD file(s) for product {}", files_to_download.len(), product);
        }
        
        let mut downloaded = 0;
        for cad_file in files_to_download {
            match self.download_asset(&cad_file.url, &output_path, product).await {
                Ok(filename) => {
                    println!("  📁 {} ({})", filename, cad_file.key);
                    downloaded += 1;
                }
                Err(e) => println!("⚠️  Failed to download {}: {}", cad_file.key, e),
            }
        }
        
        println!("\n✅ Downloaded {} CAD files to: {}", downloaded, output_path.display());
        
        Ok(())
    }

    pub async fn download_datasheets(&self, product: &str, output_dir: Option<&str>) -> Result<()> {
        let links = self.get_product_links(product).await?;
        
        let output_path = if let Some(dir) = output_dir {
            PathBuf::from(dir)
        } else {
            Self::get_default_download_dir(product, "datasheets")?
        };
        
        Self::ensure_directory_exists(&output_path).await?;
        
        if links.datasheets.is_empty() {
            println!("No datasheets available for product {}", product);
            return Ok(());
        }
        
        println!("Found {} datasheet(s) for product {}", links.datasheets.len(), product);
        
        let mut downloaded = 0;
        for datasheet_path in &links.datasheets {
            match self.download_asset(datasheet_path, &output_path, product).await {
                Ok(_) => downloaded += 1,
                Err(e) => println!("⚠️  Failed to download datasheet: {}", e),
            }
        }
        
        println!("\n✅ Downloaded {}/{} datasheets to: {}", 
            downloaded, links.datasheets.len(), output_path.display());
        
        Ok(())
    }
}