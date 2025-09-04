//! Name Generator Core

use super::abbreviations;
use super::converters;
use super::detectors;
use super::templates;
use crate::models::product::ProductDetail;
use std::collections::HashMap;

/// Template for generating names for a specific category of parts
#[derive(Debug, Clone)]
pub struct NamingTemplate {
    pub prefix: String,                              // BHCS, FW, BB
    pub key_specs: Vec<String>,                      // ["Material", "Thread Size", "Length"]
    pub spec_abbreviations: HashMap<String, String>, // "316 Stainless Steel" -> "SS316"
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
            self.apply_template(product, template)
        } else {
            self.fallback_name(product)
        }
    }

    /// Apply a naming template to generate the final name
    fn apply_template(&self, product: &ProductDetail, template: &NamingTemplate) -> String {
        let mut name_parts = vec![template.prefix.clone()];
        let mut extracted_finish: Option<String> = None;

        for spec_name in &template.key_specs {
            if let Some(spec) = product
                .specifications
                .iter()
                .find(|s| s.attribute.eq_ignore_ascii_case(spec_name))
            {
                let value = spec.values.first().unwrap_or(&"".to_string()).clone();

                // Special handling for Material that might include finish
                if spec_name.eq_ignore_ascii_case("Material") {
                    let (material, finish) = abbreviations::parse_material_and_finish(&value);

                    // Special handling for bearings with filler material
                    let final_material = if template.prefix.ends_with("B")
                        && (template.prefix.starts_with("FSB")
                            || template.prefix.starts_with("SB")
                            || template.prefix.starts_with("BB"))
                    {
                        // Check for filler material for bearings
                        if let Some(filler_spec) = product
                            .specifications
                            .iter()
                            .find(|s| s.attribute.eq_ignore_ascii_case("Filler Material"))
                        {
                            if let Some(filler_value) = filler_spec.values.first() {
                                if !filler_value.is_empty()
                                    && filler_value != "None"
                                    && filler_value != "Not Specified"
                                {
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
                    } else if material.eq_ignore_ascii_case("Steel")
                        || material.eq_ignore_ascii_case("Alloy Steel")
                    {
                        // Check for steel grade to make steel more descriptive
                        abbreviations::get_steel_grade_material(product, &material)
                    } else {
                        material
                    };

                    // Add material abbreviation
                    let material_abbrev = template
                        .spec_abbreviations
                        .get(&final_material)
                        .cloned()
                        .unwrap_or_else(|| abbreviations::abbreviate_value(&final_material));
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
                        let finish_abbrev = template
                            .spec_abbreviations
                            .get(&finish_value)
                            .cloned()
                            .unwrap_or_else(|| abbreviations::abbreviate_value(&finish_value));
                        // Skip passivated finish as it doesn't add meaningful information
                        if !finish_abbrev.is_empty() && finish_abbrev != "PASS" {
                            name_parts.push(finish_abbrev);
                        }
                    }
                } else if spec_name.eq_ignore_ascii_case("Length") {
                    // Special handling for Length - convert fractions to decimals for screws
                    let length_value = converters::convert_length_to_decimal(&value);
                    let abbreviated = template
                        .spec_abbreviations
                        .get(&length_value)
                        .cloned()
                        .unwrap_or(length_value);

                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                } else if spec_name.eq_ignore_ascii_case("For Shaft Diameter")
                    || spec_name.eq_ignore_ascii_case("OD")
                    || spec_name.eq_ignore_ascii_case("Diameter")
                    || spec_name.eq_ignore_ascii_case("Usable Length")
                    || spec_name.eq_ignore_ascii_case("Width")
                    || spec_name.eq_ignore_ascii_case("Overall Height")
                    || spec_name.eq_ignore_ascii_case("Overall Length")
                    || spec_name.eq_ignore_ascii_case("Mounting Hole Center -to-Center")
                {
                    // Special handling for dimensions - convert fractions to decimals
                    let dimension_value = converters::convert_length_to_decimal(&value);
                    let abbreviated = template
                        .spec_abbreviations
                        .get(&dimension_value)
                        .cloned()
                        .unwrap_or(dimension_value);

                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                } else if spec_name.eq_ignore_ascii_case("Thread Size") || spec_name.eq_ignore_ascii_case("Thread (A) Size") {
                    // Special handling for Thread Size - extract pitch for metric threads
                    let thread_value = converters::extract_thread_with_pitch(product, &value);
                    let abbreviated = template
                        .spec_abbreviations
                        .get(&thread_value)
                        .cloned()
                        .unwrap_or_else(|| abbreviations::abbreviate_value(&thread_value));

                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                } else {
                    // Normal handling for other specs
                    let abbreviated = template
                        .spec_abbreviations
                        .get(&value)
                        .cloned()
                        .unwrap_or_else(|| abbreviations::abbreviate_value(&value));

                    if !abbreviated.is_empty() {
                        name_parts.push(abbreviated);
                    }
                }
            } else if spec_name.eq_ignore_ascii_case("Finish") && extracted_finish.is_some() {
                // Handle case where there's no "Finish" attribute but we extracted finish from material
                let finish_value = extracted_finish.clone().unwrap();
                let finish_abbrev = template
                    .spec_abbreviations
                    .get(&finish_value)
                    .cloned()
                    .unwrap_or_else(|| abbreviations::abbreviate_value(&finish_value));
                // Skip passivated finish as it doesn't add meaningful information
                if !finish_abbrev.is_empty() && finish_abbrev != "PASS" {
                    name_parts.push(finish_abbrev);
                }
            }
        }

        name_parts.join("-")
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
