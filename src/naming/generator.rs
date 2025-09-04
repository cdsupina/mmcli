//! Name Generator Core

use super::abbreviations;
use super::converters;
use super::detectors;
use super::templates;
use crate::models::product::ProductDetail;
use std::collections::HashMap;

/// Specification mapping with aliases and transform options
#[derive(Debug, Clone)]
pub struct SpecMapping {
    pub aliases: Vec<String>,                        // ["Thread Size", "Thread (A) Size", "Thread (B) Size"]
    pub required: bool,                              // Whether this spec is required for naming
    pub transform: Option<TransformType>,            // Optional transformation to apply
}

/// Types of transformations that can be applied to specification values
#[derive(Debug, Clone)]
pub enum TransformType {
    FractionToDecimal,                               // Convert fractions like "1/4" to decimals
    ThreadConversion,                                // Convert hyphens to x in thread sizes
    MaterialFinishSplit,                             // Split material and finish
}

/// Template for generating names for a specific category of parts
#[derive(Debug, Clone)]
pub struct NamingTemplate {
    pub prefix: String,                              // BHCS, FW, BB
    pub key_specs: Vec<String>,                      // ["Material", "Thread Size", "Length"] (backward compatibility)
    pub spec_aliases: Option<HashMap<String, Vec<String>>>, // Optional aliases: "Thread Size" -> ["Thread Size", "Thread (A) Size"]
    pub spec_abbreviations: HashMap<String, String>, // "316 Stainless Steel" -> "SS316"
}

impl NamingTemplate {
    /// Create a new template with basic specs (backward compatible)
    pub fn new(prefix: &str, key_specs: Vec<&str>) -> Self {
        NamingTemplate {
            prefix: prefix.to_string(),
            key_specs: key_specs.into_iter().map(|s| s.to_string()).collect(),
            spec_aliases: None,
            spec_abbreviations: HashMap::new(),
        }
    }
    
    /// Add aliases for specifications
    pub fn with_aliases(mut self, aliases: HashMap<String, Vec<String>>) -> Self {
        self.spec_aliases = Some(aliases);
        self
    }
    
    /// Add abbreviations
    pub fn with_abbreviations(mut self, abbreviations: HashMap<String, String>) -> Self {
        self.spec_abbreviations = abbreviations;
        self
    }
}

impl SpecMapping {
    /// Create a simple required specification mapping with a single name
    pub fn simple(name: &str) -> Self {
        SpecMapping {
            aliases: vec![name.to_string()],
            required: true,
            transform: None,
        }
    }
    
    /// Create a specification mapping with multiple aliases
    pub fn with_aliases(aliases: Vec<&str>) -> Self {
        SpecMapping {
            aliases: aliases.into_iter().map(|s| s.to_string()).collect(),
            required: true,
            transform: None,
        }
    }
    
    /// Add a transform to this specification mapping
    pub fn with_transform(mut self, transform: TransformType) -> Self {
        self.transform = Some(transform);
        self
    }
    
    /// Make this specification optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Main naming system that generates human-readable technical names
pub struct NameGenerator {
    category_templates: HashMap<String, NamingTemplate>,
}

impl NameGenerator {
    /// Create a new name generator with all templates initialized
    pub fn new() -> Self {
        let mut generator = NameGenerator {
            category_templates: HashMap::new(),
        };
        generator.initialize_templates();
        generator
    }

    /// Initialize all naming templates for different fastener categories
    fn initialize_templates(&mut self) {
        templates::initialize_screw_templates(&mut self.category_templates);
        templates::initialize_washer_templates(&mut self.category_templates);
        templates::initialize_nut_templates(&mut self.category_templates);
        templates::initialize_standoff_templates(&mut self.category_templates);
        templates::initialize_spacer_templates(&mut self.category_templates);
        templates::initialize_pin_templates(&mut self.category_templates);
        templates::initialize_bearing_templates(&mut self.category_templates);
    }

    /// Generate a human-readable name for the given product
    pub fn generate_name(&self, product: &ProductDetail) -> String {
        let template_key = detectors::determine_category(product);

        if let Some(template) = self.category_templates.get(&template_key) {
            self.apply_template(product, template, &template_key)
        } else {
            self.fallback_name(product)
        }
    }

    /// Apply a naming template to generate the final name
    fn apply_template(&self, product: &ProductDetail, template: &NamingTemplate, template_category: &str) -> String {
        let mut name_parts = vec![template.prefix.clone()];
        let mut extracted_finish: Option<String> = None;

        for spec_name in &template.key_specs {
            // Try to find the specification, checking aliases if available
            let found_spec = self.find_specification_with_aliases(product, spec_name, template);
            
            if let Some((spec, actual_name)) = found_spec {
                let value = spec.values.first().unwrap_or(&"".to_string()).clone();
                
                // Process the specification value using existing legacy logic
                let processed_value = self.process_legacy_specification(
                    product,
                    template,
                    template_category,
                    &actual_name,
                    &value,
                    &mut extracted_finish,
                );
                
                if let Some(processed) = processed_value {
                    name_parts.push(processed);
                }
            } else if spec_name.eq_ignore_ascii_case("Finish") && extracted_finish.is_some() {
                // Special case: if looking for Finish but no explicit field exists, 
                // use extracted finish from material if available
                let processed_value = self.process_legacy_specification(
                    product,
                    template,
                    template_category,
                    spec_name,
                    "", // Empty value to trigger extracted_finish logic
                    &mut extracted_finish,
                );
                
                if let Some(processed) = processed_value {
                    name_parts.push(processed);
                }
            }
            // For other missing specifications, silently skip (existing behavior)
        }

        name_parts.join("-")
    }
    
    /// Find a specification by trying the main name and any aliases
    fn find_specification_with_aliases<'a>(
        &self, 
        product: &'a ProductDetail, 
        spec_name: &str, 
        template: &NamingTemplate
    ) -> Option<(&'a crate::models::product::Specification, String)> {
        // First try the main specification name
        if let Some(spec) = product.specifications.iter()
            .find(|s| s.attribute.eq_ignore_ascii_case(spec_name)) {
            return Some((spec, spec_name.to_string()));
        }
        
        // If not found and aliases exist, try each alias
        if let Some(ref aliases) = template.spec_aliases {
            if let Some(alias_list) = aliases.get(spec_name) {
                for alias in alias_list {
                    if let Some(spec) = product.specifications.iter()
                        .find(|s| s.attribute.eq_ignore_ascii_case(alias)) {
                        return Some((spec, alias.clone()));
                    }
                }
            }
        }
        
        None
    }
    
    /// Process a specification value using the existing legacy logic
    fn process_legacy_specification(
        &self,
        product: &ProductDetail,
        template: &NamingTemplate,
        template_category: &str,
        spec_name: &str,
        value: &str,
        extracted_finish: &mut Option<String>,
    ) -> Option<String> {
        // This preserves all the existing logic from the original apply_template method
        
        // Special handling for Material that might include finish
        if self.is_material_field(spec_name) {
            let (material, finish) = abbreviations::parse_material_and_finish(value);
            
            // Store extracted finish for later use
            *extracted_finish = finish;
            
            // Special handling for bearings with filler material
            let final_material = if template.prefix.ends_with("B")
                && (template.prefix.starts_with("FSB")
                    || template.prefix.starts_with("SB")
                    || template.prefix.starts_with("BB"))
            {
                if let Some(filler_spec) = product.specifications.iter()
                    .find(|s| s.attribute.eq_ignore_ascii_case("Filler Material")) {
                    if let Some(filler_value) = filler_spec.values.first() {
                        if !filler_value.is_empty() && filler_value != "None" && filler_value != "Not Specified" {
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
                abbreviations::get_steel_grade_material(product, &material)
            } else {
                material
            };
            
            let material_abbrev = template.spec_abbreviations.get(&final_material)
                .cloned()
                .unwrap_or_else(|| abbreviations::abbreviate_value(&final_material));
                
            if material_abbrev.is_empty() { None } else { Some(material_abbrev) }
            
        } else if spec_name.eq_ignore_ascii_case("Finish") {
            let finish_value = if !value.is_empty() {
                value.to_string()
            } else {
                extracted_finish.clone().unwrap_or_default()
            };
            
            if !finish_value.is_empty() {
                let finish_abbrev = template.spec_abbreviations.get(&finish_value)
                    .cloned()
                    .unwrap_or_else(|| abbreviations::abbreviate_value(&finish_value));
                // Skip passivated finish as it doesn't add meaningful information
                if !finish_abbrev.is_empty() && finish_abbrev != "PASS" {
                    Some(finish_abbrev)
                } else {
                    None
                }
            } else {
                None
            }
            
        } else if spec_name.eq_ignore_ascii_case("Length") {
            let length_value = converters::convert_length_to_decimal(value);
            let abbreviated = template.spec_abbreviations.get(&length_value)
                .cloned()
                .unwrap_or(length_value);
            if abbreviated.is_empty() { None } else { Some(abbreviated) }
            
        } else if spec_name.eq_ignore_ascii_case("For Screw Size") {
            // Context-sensitive handling for "For Screw Size"
            if self.is_washer_template(template_category) {
                // Washer templates - use screw size abbreviation (preserve "6", "1/4", etc.)
                let abbreviated = template.spec_abbreviations.get(value)
                    .cloned()
                    .unwrap_or_else(|| abbreviations::abbreviate_value(value));
                if abbreviated.is_empty() { None } else { Some(abbreviated) }
            } else {
                // Other templates (spacers, etc.) - use dimension conversion
                let dimension_value = converters::convert_length_to_decimal(value);
                let abbreviated = template.spec_abbreviations.get(&dimension_value)
                    .cloned()
                    .unwrap_or(dimension_value);
                if abbreviated.is_empty() { None } else { Some(abbreviated) }
            }
            
        } else if self.is_dimension_field(spec_name) {
            let dimension_value = converters::convert_length_to_decimal(value);
            let abbreviated = template.spec_abbreviations.get(&dimension_value)
                .cloned()
                .unwrap_or(dimension_value);
            if abbreviated.is_empty() { None } else { Some(abbreviated) }
            
        } else if self.is_thread_size_field(spec_name) {
            let thread_value = converters::extract_thread_with_pitch(product, value);
            let abbreviated = template.spec_abbreviations.get(&thread_value)
                .cloned()
                .unwrap_or_else(|| abbreviations::abbreviate_value(&thread_value));
            if abbreviated.is_empty() { None } else { Some(abbreviated) }
            
        } else {
            // Normal handling for other specs
            let abbreviated = template.spec_abbreviations.get(value)
                .cloned()
                .unwrap_or_else(|| abbreviations::abbreviate_value(value));
            if abbreviated.is_empty() { None } else { Some(abbreviated) }
        }
    }
    
    /// Check if a specification name represents a dimension field
    fn is_dimension_field(&self, spec_name: &str) -> bool {
        let spec_lower = spec_name.to_lowercase();
        // More specific patterns to avoid false matches
        self.is_diameter_field(&spec_lower) ||
        self.is_length_field(&spec_lower) ||
        spec_lower == "width" || spec_lower.ends_with(" width") ||
        spec_lower == "height" || spec_lower.ends_with(" height") ||
        spec_lower == "od" || spec_lower.ends_with(" od") ||
        spec_lower == "id" || spec_lower.ends_with(" id") ||
        spec_lower == "for screw size" ||
        spec_lower.contains("mounting hole center")
    }

    /// Check if a field name represents a diameter measurement
    fn is_diameter_field(&self, spec_lower: &str) -> bool {
        spec_lower == "diameter" ||
        spec_lower.ends_with(" diameter") ||
        spec_lower.starts_with("diameter ") ||
        spec_lower == "bore" ||
        spec_lower.ends_with(" bore")
    }

    /// Check if a field name represents a length measurement  
    fn is_length_field(&self, spec_lower: &str) -> bool {
        spec_lower == "length" ||
        spec_lower.ends_with(" length") ||
        spec_lower.starts_with("length ") ||
        spec_lower.starts_with("overall ") && spec_lower.ends_with(" length")
    }

    /// Check if a template category represents a washer type
    fn is_washer_template(&self, template_category: &str) -> bool {
        template_category.contains("washer")
    }

    /// Check if a specification name represents a thread size field
    fn is_thread_size_field(&self, spec_name: &str) -> bool {
        let spec_lower = spec_name.to_lowercase();
        // More specific patterns to avoid false matches
        spec_lower == "thread size" ||
        spec_lower == "thread (a) size" ||
        spec_lower == "thread (b) size" ||
        spec_lower.starts_with("thread size") ||
        spec_lower.starts_with("thread (") && spec_lower.contains(") size")
    }

    /// Check if a specification name represents a material field
    fn is_material_field(&self, spec_name: &str) -> bool {
        let spec_lower = spec_name.to_lowercase();
        spec_lower == "material" ||
        spec_lower == "housing material" ||
        spec_lower.ends_with(" material")
    }

    /// Generate fallback name for unsupported categories
    fn fallback_name(&self, product: &ProductDetail) -> String {
        // Extract key words from family description and use part number as suffix
        let family_words: Vec<&str> = product
            .family_description
            .split_whitespace()
            .take(4)
            .collect();

        let fallback_name = if !family_words.is_empty() {
            format!(
                "{}-{}",
                family_words.join("-").to_uppercase().replace(",", ""),
                product.part_number
            )
        } else {
            format!("UNKNOWN-{}", product.part_number)
        };

        fallback_name
    }
}
