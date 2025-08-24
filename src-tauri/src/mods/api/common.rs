use crate::mods::types::*;
use std::path::Path;
use sha1::{Sha1, Digest};
use tokio::fs;

/// Common utilities for mod API implementations
pub struct ApiUtils;

impl ApiUtils {
    /// Verify file integrity using SHA1 hash
    pub async fn verify_file_hash(file_path: &Path, expected_hash: &str) -> Result<bool, ModError> {
        let contents = fs::read(file_path).await?;
        let mut hasher = Sha1::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        let computed_hash = hex::encode(result);
        
        Ok(computed_hash.eq_ignore_ascii_case(expected_hash))
    }
    
    /// Create a user agent string for API requests
    pub fn user_agent() -> &'static str {
        "ChaiLauncher/2.0.0"
    }
    
    /// Parse minecraft version to determine mod loader compatibility
    pub fn parse_minecraft_version(version: &str) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            if let (Ok(major), Ok(minor)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                let patch = if parts.len() >= 3 {
                    parts[2].parse::<u32>().unwrap_or(0)
                } else {
                    0
                };
                return Some((major, minor, patch));
            }
        }
        None
    }
    
    /// Check if a mod loader is compatible with a minecraft version
    pub fn is_loader_compatible(loader: &str, mc_version: &str) -> bool {
        match loader.to_lowercase().as_str() {
            "forge" => {
                // Forge is available for most versions
                if let Some((major, minor, _)) = Self::parse_minecraft_version(mc_version) {
                    // Forge is available from 1.2.5 onwards
                    major >= 1 && (major > 1 || minor >= 2)
                } else {
                    false
                }
            }
            "fabric" => {
                // Fabric is available from 1.14 onwards
                if let Some((major, minor, _)) = Self::parse_minecraft_version(mc_version) {
                    major >= 1 && (major > 1 || minor >= 14)
                } else {
                    false
                }
            }
            "quilt" => {
                // Quilt is available from 1.14 onwards (similar to Fabric)
                if let Some((major, minor, _)) = Self::parse_minecraft_version(mc_version) {
                    major >= 1 && (major > 1 || minor >= 14)
                } else {
                    false
                }
            }
            "neoforge" => {
                // NeoForge is available from 1.20.1 onwards
                if let Some((major, minor, patch)) = Self::parse_minecraft_version(mc_version) {
                    major >= 1 && (major > 1 || minor > 20 || (minor == 20 && patch >= 1))
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Sanitize filename for safe filesystem usage
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '|' | '?' | '*' => '_',
                '/' | '\\' => '_',
                c if c.is_control() => '_',
                c => c,
            })
            .collect()
    }
    
    /// Format file size for display
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

/// Rate limiter for API requests
pub struct RateLimiter {
    requests_per_minute: u32,
    last_request_times: std::sync::Mutex<Vec<std::time::Instant>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request_times: std::sync::Mutex::new(Vec::new()),
        }
    }
    
    /// Wait if necessary to respect rate limits
    pub async fn wait_if_needed(&self) -> Result<(), ModError> {
        let now = std::time::Instant::now();
        let one_minute_ago = now - std::time::Duration::from_secs(60);
        
        {
            let mut times = self.last_request_times.lock().unwrap();
            
            // Remove old requests
            times.retain(|&time| time > one_minute_ago);
            
            // Check if we need to wait
            if times.len() >= self.requests_per_minute as usize {
                let wait_until = times[0] + std::time::Duration::from_secs(60);
                if wait_until > now {
                    let wait_duration = wait_until - now;
                    tokio::time::sleep(wait_duration).await;
                }
            }
            
            // Record this request
            times.push(now);
        }
        
        Ok(())
    }
}