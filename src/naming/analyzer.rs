//! Part analysis for debugging naming system

use crate::models::product::ProductDetail;
use crate::naming::detectors;
use crate::naming::generator::{NameGenerator, NamingTemplate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detailed analysis of a part's specifications and naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartAnalysis {
    /// Basic product information
    pub part_number: String,
    pub category: String,
    pub detected_type: String,
    pub family_description: String,
    pub product_description: String,
    
    /// Specification analysis
    pub specifications: Vec<SpecAnalysis>,
    pub missing_specs: Vec<String>,
    pub unmapped_specs: Vec<String>,
    
    /// Template analysis
    pub template_used: Option<String>,
    pub template_specs: Vec<String>,
    pub spec_aliases: HashMap<String, Vec<String>>,
    
    /// Name generation results
    pub generated_name: String,
    pub suggested_name: Option<String>, // Name with potential finish if applicable
    pub name_components: Vec<NameComponent>,
    
    /// Recommendations and insights
    pub suggestions: Vec<String>,
}

/// Analysis of individual specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecAnalysis {
    pub name: String,
    pub values: Vec<String>,
    pub used_in_name: bool,
    pub processed_value: Option<String>,
    pub source: String, // "direct", "alias", "extracted"
}

/// Components of the generated name with their sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameComponent {
    pub component: String,
    pub source: String,
    pub original_value: Option<String>,
}

/// Main analyzer for part specifications
pub struct PartAnalyzer {
    name_generator: NameGenerator,
}

impl PartAnalyzer {
    /// Create a new part analyzer
    pub fn new() -> Self {
        PartAnalyzer {
            name_generator: NameGenerator::new(),
        }
    }
    
    /// Analyze a product for naming system debugging
    pub fn analyze_part(&self, product: &ProductDetail, _show_template: bool, _show_aliases: bool) -> PartAnalysis {
        let detected_type = detectors::determine_category(product);
        let template = self.name_generator.get_template(&detected_type);
        
        // Analyze specifications
        let mut spec_analyses = Vec::new();
        let mut missing_specs = Vec::new();
        let mut unmapped_specs = Vec::new();
        
        // Track which specs from the product are mapped vs unmapped
        for spec in &product.specifications {
            let is_mapped = if let Some(template) = &template {
                self.is_spec_mapped(&spec.attribute, template)
            } else {
                false
            };
            
            let spec_analysis = SpecAnalysis {
                name: spec.attribute.clone(),
                values: spec.values.clone(),
                used_in_name: is_mapped,
                processed_value: None, // TODO: Add processed value logic
                source: if is_mapped { "direct".to_string() } else { "unmapped".to_string() },
            };
            
            if is_mapped {
                spec_analyses.push(spec_analysis);
            } else {
                spec_analyses.push(spec_analysis.clone());
                unmapped_specs.push(spec.attribute.clone());
            }
        }
        
        // Check for missing expected specifications
        if let Some(template) = &template {
            for expected_spec in &template.key_specs {
                if !self.find_spec_in_product(expected_spec, product, template).is_some() {
                    missing_specs.push(expected_spec.clone());
                }
            }
        }
        
        // Generate the name and break it down
        let generated_name = self.name_generator.generate_name(product);
        let suggested_name = self.generate_suggested_name(product, &template, &missing_specs);
        let name_components = self.breakdown_name(&generated_name, &detected_type, template);
        
        // Get template information for display
        let (template_used, template_specs, spec_aliases) = if let Some(template) = &template {
            (
                Some(detected_type.clone()),
                template.key_specs.clone(),
                template.spec_aliases.clone().unwrap_or_default(),
            )
        } else {
            (None, Vec::new(), HashMap::new())
        };
        
        // Generate suggestions
        let suggestions = self.generate_suggestions(product, template, &unmapped_specs, &missing_specs);
        
        PartAnalysis {
            part_number: product.part_number.clone(),
            category: product.product_category.clone(),
            detected_type,
            family_description: product.family_description.clone(),
            product_description: product.detail_description.clone(),
            specifications: spec_analyses,
            missing_specs,
            unmapped_specs,
            template_used,
            template_specs,
            spec_aliases,
            generated_name,
            suggested_name,
            name_components,
            suggestions,
        }
    }
    
    /// Check if a specification is mapped in the template
    fn is_spec_mapped(&self, spec_name: &str, template: &NamingTemplate) -> bool {
        // Check direct match in key_specs
        if template.key_specs.contains(&spec_name.to_string()) {
            return true;
        }
        
        // Check aliases if they exist
        if let Some(aliases) = &template.spec_aliases {
            for (_, alias_list) in aliases {
                if alias_list.contains(&spec_name.to_string()) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Find a specification in the product using template aliases
    fn find_spec_in_product(&self, spec_name: &str, product: &ProductDetail, template: &NamingTemplate) -> Option<String> {
        // Direct match
        for spec in &product.specifications {
            if spec.attribute == spec_name {
                return spec.values.first().cloned();
            }
        }
        
        // Check aliases
        if let Some(aliases) = &template.spec_aliases {
            if let Some(alias_list) = aliases.get(spec_name) {
                for alias in alias_list {
                    for spec in &product.specifications {
                        if spec.attribute == *alias {
                            return spec.values.first().cloned();
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Break down the generated name into components
    fn breakdown_name(&self, name: &str, category: &str, template: Option<&NamingTemplate>) -> Vec<NameComponent> {
        let mut components = Vec::new();
        
        if let Some(template) = template {
            let parts: Vec<&str> = name.split('-').collect();
            
            if !parts.is_empty() {
                // First part is always the prefix
                components.push(NameComponent {
                    component: parts[0].to_string(),
                    source: format!("Template prefix ({})", category),
                    original_value: None,
                });
                
                // Remaining parts correspond to template specs (best effort)
                for (i, part) in parts.iter().skip(1).enumerate() {
                    let spec_name = if i < template.key_specs.len() {
                        template.key_specs[i].clone()
                    } else {
                        format!("Unknown spec {}", i + 1)
                    };
                    
                    components.push(NameComponent {
                        component: part.to_string(),
                        source: spec_name,
                        original_value: None, // TODO: Map back to original values
                    });
                }
            }
        } else {
            // No template found, treat as single component
            components.push(NameComponent {
                component: name.to_string(),
                source: "Unknown template".to_string(),
                original_value: None,
            });
        }
        
        components
    }
    
    /// Generate a suggested name with potential finish if applicable
    fn generate_suggested_name(&self, product: &ProductDetail, template: &Option<&NamingTemplate>, missing_specs: &[String]) -> Option<String> {
        if let Some(template) = template {
            // Check if this template expects finish and it's missing
            if template.key_specs.contains(&"Finish".to_string()) && missing_specs.contains(&"Finish".to_string()) {
                // Try to extract finish from material or infer a common one
                let suggested_finish = self.suggest_finish_for_material(product);
                if let Some(finish) = suggested_finish {
                    let current_name = self.name_generator.generate_name(product);
                    // Only suggest if the finish isn't already in the name
                    if !current_name.ends_with(&format!("-{}", finish)) {
                        return Some(format!("{}-{}", current_name, finish));
                    }
                }
            }
        }
        None
    }
    
    /// Suggest a finish based on material and common patterns
    fn suggest_finish_for_material(&self, product: &ProductDetail) -> Option<String> {
        // Look for material specification
        for spec in &product.specifications {
            if spec.attribute == "Material" {
                if let Some(material) = spec.values.first() {
                    let material_lower = material.to_lowercase();
                    
                    // Common finish patterns for different materials
                    if material_lower.contains("stainless") {
                        return Some("PASS".to_string()); // Passivated is common for stainless
                    } else if material_lower.contains("steel") && !material_lower.contains("stainless") {
                        return Some("ZP".to_string()); // Zinc plated is common for steel
                    } else if material_lower.contains("brass") {
                        return Some("UNFINISHED".to_string()); // Brass often unfinished
                    } else if material_lower.contains("aluminum") {
                        return Some("CLEAR".to_string()); // Clear anodized common for aluminum
                    }
                }
            }
        }
        
        // Default suggestion for missing finish
        Some("?".to_string())
    }
    
    /// Generate helpful suggestions based on analysis
    fn generate_suggestions(&self, product: &ProductDetail, template: Option<&NamingTemplate>, unmapped_specs: &[String], missing_specs: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if template.is_none() {
            suggestions.push("⚠️  No template found for this part type. Consider adding a new template.".to_string());
            return suggestions;
        }
        
        if missing_specs.is_empty() && unmapped_specs.is_empty() {
            suggestions.push("✅ Template uses all available key specifications".to_string());
        }
        
        if !missing_specs.is_empty() {
            suggestions.push(format!("🔍 Missing expected specifications: {}", missing_specs.join(", ")));
            suggestions.push("   → These specs might be named differently in the API".to_string());
            suggestions.push("   → Consider adding aliases to the template".to_string());
        }
        
        if !unmapped_specs.is_empty() {
            suggestions.push(format!("📋 Unmapped specifications available: {}", unmapped_specs.join(", ")));
            suggestions.push("   → These could be added to the template for more detailed names".to_string());
        }
        
        // Check for common patterns
        let family_lower = product.family_description.to_lowercase();
        if family_lower.contains("stainless") && !missing_specs.is_empty() {
            suggestions.push("💡 Stainless steel parts often have finish embedded in material".to_string());
        }
        
        suggestions
    }
    
    /// Format analysis output for human reading
    pub fn format_human(&self, analysis: &PartAnalysis, show_template: bool, show_aliases: bool) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("🔍 Part Analysis: {}\n\n", analysis.part_number));
        
        // Product info
        output.push_str("📦 Product Info:\n");
        output.push_str(&format!("   Category: {}\n", analysis.category));
        output.push_str(&format!("   Family: {}\n", analysis.family_description));
        output.push_str(&format!("   Detected Type: {}\n\n", analysis.detected_type));
        
        // Specifications
        output.push_str(&format!("📋 Specifications ({} found):\n", analysis.specifications.len()));
        for spec in &analysis.specifications {
            let status = if spec.used_in_name { "✓" } else { "✗" };
            let values_str = spec.values.join(", ");
            output.push_str(&format!("   {} {}: {} → ({})\n", 
                status, 
                spec.name, 
                values_str,
                if spec.used_in_name { "used in name" } else { "not used" }
            ));
        }
        output.push('\n');
        
        // Template info (if requested)
        if show_template {
            if let Some(template_name) = &analysis.template_used {
                output.push_str(&format!("🏷️  Template: {} \n", template_name));
                output.push_str(&format!("   Expected: [{}]\n", analysis.template_specs.join(", ")));
                
                if show_aliases && !analysis.spec_aliases.is_empty() {
                    output.push_str("   Aliases:\n");
                    for (spec, aliases) in &analysis.spec_aliases {
                        output.push_str(&format!("   - {}: {}\n", spec, aliases.join(", ")));
                    }
                }
                output.push('\n');
            } else {
                output.push_str("🏷️  Template: None found\n\n");
            }
        }
        
        // Generated name breakdown
        output.push_str(&format!("🔧 Generated Name: {}\n", analysis.generated_name));
        if let Some(suggested) = &analysis.suggested_name {
            output.push_str(&format!("💡 Suggested Name with Finish: {}\n", suggested));
        }
        if !analysis.name_components.is_empty() {
            output.push_str("   Components:\n");
            for component in &analysis.name_components {
                output.push_str(&format!("   - {}: {}\n", component.component, component.source));
            }
        }
        output.push('\n');
        
        // Suggestions
        if !analysis.suggestions.is_empty() {
            output.push_str("💡 Suggestions:\n");
            for suggestion in &analysis.suggestions {
                output.push_str(&format!("   {}\n", suggestion));
            }
        }
        
        output
    }
    
    /// Format analysis output as JSON
    pub fn format_json(&self, analysis: &PartAnalysis) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(analysis)
    }
}