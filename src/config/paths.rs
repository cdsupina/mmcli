//! XDG-compliant path management

use dirs;
use std::path::PathBuf;

/// Get the XDG config directory for mmc
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mmc")
}

/// Get the token file path
pub fn get_token_path() -> PathBuf {
    get_config_dir().join("token")
}

/// Find certificate in default locations
pub fn find_certificate_path() -> Option<PathBuf> {
    let config_dir = get_config_dir();
    
    // Check XDG config directory first
    let candidates = [
        config_dir.join("certificate.pfx"),
        config_dir.join("certificate.p12"),
        // Legacy locations
        dirs::home_dir()?.join(".mmcli").join("certificate.pfx"),
        dirs::home_dir()?.join(".mmcli").join("certificate.p12"),
    ];
    
    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }
    
    None
}

/// Expand tilde in path strings
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(&path[2..])
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    }
}