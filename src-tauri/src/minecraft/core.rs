//! MCVM Core Integration
//! 
//! This module provides a simplified wrapper around MCVM for ChaiLauncher's needs.
//! Rather than trying to fully integrate MCVM's complex configuration system,
//! we use MCVM selectively for the tasks it excels at while maintaining
//! ChaiLauncher's existing Java management and UI integration.

use std::sync::OnceLock;
use mcvm::io::paths::Paths;
use std::path::PathBuf;

static MCVM_PATHS: OnceLock<Paths> = OnceLock::new();

/// ChaiLauncher's MCVM integration wrapper
pub struct MCVMCore;

impl MCVMCore {
    /// Initialize MCVM paths for ChaiLauncher
    pub async fn initialize() -> Result<(), String> {
        println!("🔧 Initializing MCVM integration...");
        
        // Create MCVM paths using default location
        let paths = Paths::new().await
            .map_err(|e| format!("Failed to create MCVM paths: {}", e))?;

        MCVM_PATHS.set(paths)
            .map_err(|_| "Failed to set global MCVM paths")?;

        println!("✅ MCVM integration initialized successfully");
        Ok(())
    }

    /// Get the global MCVM paths
    pub fn paths() -> Result<&'static Paths, String> {
        MCVM_PATHS.get().ok_or_else(|| "MCVM not initialized".to_string())
    }

    /// Create a basic MCVM instance for launching (placeholder)
    /// For now, we'll validate through MCVM's path system only
    pub async fn create_launch_instance(
        name: &str,
        version: &str,
        game_dir: PathBuf,
    ) -> Result<(), String> {
        // For now, we just validate that MCVM paths are available
        let _paths = Self::paths()?;
        
        // Validate that the game directory exists
        if !game_dir.exists() {
            return Err(format!("Game directory does not exist: {}", game_dir.display()));
        }
        
        println!("✅ MCVM validation passed for instance '{}' version {}", name, version);
        Ok(())
    }

    /// Prepare for launch using MCVM validation (placeholder)
    /// This validates the launch environment but returns an error to fall back to direct launch
    pub async fn launch_instance_with_java(
        java_path: String,
        memory: u32,
        username: String,
        game_dir: PathBuf,
    ) -> Result<u32, String> {
        let _paths = Self::paths()?;
        
        // Validate launch parameters
        if !std::path::Path::new(&java_path).exists() {
            return Err(format!("Java path does not exist: {}", java_path));
        }
        
        if memory < 512 {
            return Err("Memory must be at least 512MB".to_string());
        }
        
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        
        if !game_dir.exists() {
            return Err(format!("Game directory does not exist: {}", game_dir.display()));
        }
        
        // For now, we'll always fall back to direct launch
        // This allows MCVM to validate parameters while maintaining compatibility
        Err("MCVM launch validation passed - using fallback launcher for better compatibility".to_string())
    }
}

// MCVM output handler removed - will be added back when full MCVM integration is implemented

/// Initialize MCVM integration
pub async fn initialize() -> Result<(), String> {
    MCVMCore::initialize().await
}