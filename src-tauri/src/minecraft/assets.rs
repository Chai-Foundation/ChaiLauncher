//! Asset Management using MCVM
//! 
//! This module handles Minecraft asset downloading and management using MCVM.

use super::MCVMCore;

/// Download and manage Minecraft assets for a version
pub async fn download_assets(
    version: &str,
    game_dir: &std::path::Path,
    progress_callback: Option<Box<dyn Fn(u32, String)>>,
) -> Result<(), String> {
    println!("📦 Downloading assets for Minecraft {} via MCVM", version);

    if let Some(callback) = &progress_callback {
        callback(10, "Starting asset download...".to_string());
    }

    let core = MCVMCore::instance()?;
    
    // Use MCVM to handle asset download
    // This is a simplified implementation - MCVM handles the complexity internally
    core.download_assets(version, game_dir).await
        .map_err(|e| format!("Failed to download assets: {}", e))?;

    if let Some(callback) = progress_callback {
        callback(100, "Asset download complete!".to_string());
    }

    println!("✅ Assets for Minecraft {} downloaded successfully", version);
    Ok(())
}

/// Check if assets are available for a version
pub async fn assets_available(version: &str, game_dir: &std::path::Path) -> bool {
    let assets_dir = game_dir.join("assets");
    let indexes_dir = assets_dir.join("indexes");
    let objects_dir = assets_dir.join("objects");
    
    // Basic check for asset structure
    assets_dir.exists() && indexes_dir.exists() && objects_dir.exists()
}