//! Subscription tracking and management

use anyhow::Result;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use crate::config::paths::{expand_path, get_subscriptions_path};
use crate::models::auth::Credentials;

/// Manager for local subscription tracking
pub struct SubscriptionManager {
    file_path: PathBuf,
    parts: HashSet<String>, // In-memory cache for O(1) lookups and automatic deduplication
}

impl SubscriptionManager {
    /// Create a new subscription manager with the given credentials
    pub fn new(credentials: &Option<Credentials>) -> Result<Self> {
        let file_path = if let Some(creds) = credentials {
            if let Some(ref path) = creds.subscriptions_file {
                expand_path(path)
            } else {
                // Fall back to default location if not specified
                get_subscriptions_path()
            }
        } else {
            // No credentials provided, use default location
            get_subscriptions_path()
        };

        let mut manager = SubscriptionManager {
            file_path,
            parts: HashSet::new(),
        };

        // Load existing subscriptions from file
        manager.load_from_file()?;

        Ok(manager)
    }

    /// Load subscriptions from file into HashSet (automatically deduplicates)
    fn load_from_file(&mut self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // If file doesn't exist, start with empty set
        if !self.file_path.exists() {
            return Ok(());
        }

        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?.trim().to_string();
            if !line.is_empty() && !line.starts_with('#') {
                // Remove any whitespace and convert to uppercase for consistency
                let part_number = line.trim().to_uppercase();
                self.parts.insert(part_number);
            }
        }

        Ok(())
    }

    /// Save all parts to file (automatically deduplicated)
    fn save_to_file(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        let mut writer = BufWriter::new(file);

        // Write header comment
        writeln!(
            writer,
            "# McMaster-Carr Subscribed Parts\n# Auto-managed by mmcli - do not edit manually\n"
        )?;

        // Write sorted part numbers (one per line)
        let mut sorted_parts: Vec<_> = self.parts.iter().collect();
        sorted_parts.sort();

        for part in sorted_parts {
            writeln!(writer, "{}", part)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Add part to subscription tracking (only writes if new)
    pub fn add_part(&mut self, part_number: &str) -> Result<bool> {
        let normalized_part = part_number.trim().to_uppercase();
        
        // Only add and save if it's actually new
        if self.parts.insert(normalized_part) {
            self.save_to_file()?; // Only write if part was newly added
            Ok(true) // Part was new
        } else {
            Ok(false) // Part already existed, no file write needed
        }
    }

    /// Remove part from subscription tracking
    pub fn remove_part(&mut self, part_number: &str) -> Result<bool> {
        let normalized_part = part_number.trim().to_uppercase();
        
        if self.parts.remove(&normalized_part) {
            self.save_to_file()?;
            Ok(true) // Part was removed
        } else {
            Ok(false) // Part wasn't in the set
        }
    }

    /// Check if part exists in local cache
    pub fn has_part(&self, part_number: &str) -> bool {
        let normalized_part = part_number.trim().to_uppercase();
        self.parts.contains(&normalized_part)
    }

    /// Get all subscribed parts (sorted)
    pub fn get_all_parts(&self) -> Vec<String> {
        let mut parts: Vec<_> = self.parts.iter().cloned().collect();
        parts.sort();
        parts
    }

    /// Get count of tracked parts
    pub fn count(&self) -> usize {
        self.parts.len()
    }

    /// Import parts from a file (auto-deduplicates)
    pub fn import_from_file(&mut self, import_path: &str) -> Result<usize> {
        let path = expand_path(import_path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        let mut imported_count = 0;

        for line in reader.lines() {
            let line = line?.trim().to_string();
            if !line.is_empty() && !line.starts_with('#') {
                let part_number = line.trim().to_uppercase();
                if self.parts.insert(part_number) {
                    imported_count += 1;
                }
            }
        }

        if imported_count > 0 {
            self.save_to_file()?;
        }

        Ok(imported_count)
    }

    /// Clear all parts (for testing or reset)
    pub fn clear(&mut self) -> Result<()> {
        self.parts.clear();
        self.save_to_file()?;
        Ok(())
    }

    /// Get the path to the subscription file being used
    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_subscription_manager_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test_subscriptions.txt");

        // Create credentials with custom path
        let creds = Some(Credentials {
            username: "test".to_string(),
            password: "test".to_string(),
            certificate_path: None,
            certificate_password: None,
            subscriptions_file: Some(test_file.to_string_lossy().to_string()),
        });

        let mut manager = SubscriptionManager::new(&creds).unwrap();

        // Test adding parts
        assert!(manager.add_part("91831A030").unwrap());
        assert!(!manager.add_part("91831A030").unwrap()); // Duplicate should return false
        assert!(manager.add_part("92141A008").unwrap());

        // Test checking parts
        assert!(manager.has_part("91831A030"));
        assert!(manager.has_part("91831a030")); // Case insensitive
        assert!(!manager.has_part("99999X999"));

        // Test getting all parts
        let parts = manager.get_all_parts();
        assert_eq!(parts.len(), 2);
        assert!(parts.contains(&"91831A030".to_string()));
        assert!(parts.contains(&"92141A008".to_string()));

        // Test removing parts
        assert!(manager.remove_part("91831A030").unwrap());
        assert!(!manager.remove_part("91831A030").unwrap()); // Already removed
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_configurable_subscription_file_path() {
        let temp_dir = tempdir().unwrap();
        let custom_path = temp_dir.path().join("custom_location").join("my_subs.txt");

        // Test with custom path specified
        let creds_custom = Some(Credentials {
            username: "test".to_string(),
            password: "test".to_string(),
            certificate_path: None,
            certificate_password: None,
            subscriptions_file: Some(custom_path.to_string_lossy().to_string()),
        });

        let manager_custom = SubscriptionManager::new(&creds_custom).unwrap();
        assert_eq!(manager_custom.get_file_path(), &custom_path);

        // Test with no path specified (should use default)
        let creds_default = Some(Credentials {
            username: "test".to_string(),
            password: "test".to_string(),
            certificate_path: None,
            certificate_password: None,
            subscriptions_file: None,
        });

        let manager_default = SubscriptionManager::new(&creds_default).unwrap();
        assert!(manager_default.get_file_path().ends_with("subscriptions.txt"));
        assert!(manager_default.get_file_path().to_string_lossy().contains(".config/mmc"));

        // Test with no credentials (should use default)
        let manager_none = SubscriptionManager::new(&None).unwrap();
        assert!(manager_none.get_file_path().ends_with("subscriptions.txt"));
        assert!(manager_none.get_file_path().to_string_lossy().contains(".config/mmc"));
    }
}