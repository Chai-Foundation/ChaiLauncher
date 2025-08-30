use serde::{Deserialize, Serialize};
use tauri::{command, Emitter};
use std::path::PathBuf;
use tokio::fs;
use reqwest;

use crate::minecraft::versions::load_version_manifest;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<MinecraftVersionInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

/// Get Minecraft versions from Mojang API
#[command]
pub async fn get_minecraft_versions() -> Result<VersionManifest, String> {
    let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    
    let response = reqwest::get(url).await
        .map_err(|e| {
            let os_info = if cfg!(target_os = "macos") {
                "macOS"
            } else if cfg!(target_os = "windows") {
                "Windows"
            } else {
                "Linux"
            };
            format!("Failed to fetch version manifest from {} (Platform: {}): {}", url, os_info, e)
        })?;
    
    let manifest: VersionManifest = response.json().await
        .map_err(|e| format!("Failed to parse version manifest JSON: {}. The response may be malformed or the API may have changed.", e))?;
    
    Ok(manifest)
}

/// Download Minecraft assets for a specific version
#[command]
pub async fn download_minecraft_assets(version: String, game_dir: String) -> Result<(), String> {
    let game_path = PathBuf::from(game_dir);
    let assets_dir = game_path.join("assets");
    
    // Create assets directory structure
    fs::create_dir_all(&assets_dir).await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
    
    // Load version manifest to get asset index
    if let Ok(Some(version_json)) = load_version_manifest(&game_path, &version).await {
        if let Some(asset_index) = version_json.get("assetIndex") {
            let index_id = asset_index.get("id").and_then(|v| v.as_str()).unwrap_or(&version);
            let index_url = asset_index.get("url").and_then(|v| v.as_str());
            
            if let Some(url) = index_url {
                println!("📥 Downloading asset index for {}", version);
                
                // Download asset index
                let indexes_dir = assets_dir.join("indexes");
                fs::create_dir_all(&indexes_dir).await
                    .map_err(|e| format!("Failed to create indexes directory: {}", e))?;
                
                let index_file = indexes_dir.join(format!("{}.json", index_id));
                let response = reqwest::get(url).await
                    .map_err(|e| format!("Failed to download asset index: {}", e))?;
                
                let index_content = response.text().await
                    .map_err(|e| format!("Failed to read asset index: {}", e))?;
                
                fs::write(&index_file, &index_content).await
                    .map_err(|e| format!("Failed to write asset index: {}", e))?;
                
                // Parse and download assets
                if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&index_content) {
                    if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
                        let objects_dir = assets_dir.join("objects");
                        fs::create_dir_all(&objects_dir).await
                            .map_err(|e| format!("Failed to create objects directory: {}", e))?;
                        
                        let mut downloaded = 0;
                        let total = objects.len();
                        
                        for (_name, asset_info) in objects.iter() {
                            if let Some(hash) = asset_info.get("hash").and_then(|v| v.as_str()) {
                                let hash_prefix = &hash[0..2];
                                let object_dir = objects_dir.join(hash_prefix);
                                let object_file = object_dir.join(hash);
                                
                                if !object_file.exists() {
                                    fs::create_dir_all(&object_dir).await
                                        .map_err(|e| format!("Failed to create object directory: {}", e))?;
                                    
                                    let asset_url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);
                                    
                                    if let Ok(response) = reqwest::get(&asset_url).await {
                                        if let Ok(bytes) = response.bytes().await {
                                            let _ = fs::write(&object_file, &bytes).await;
                                        }
                                    }
                                }
                            }
                            downloaded += 1;
                            
                            if downloaded % 50 == 0 {
                                println!("📦 Downloaded {}/{} assets", downloaded, total);
                            }
                        }
                        
                        println!("✓ Downloaded {} assets for {}", total, version);
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Download Minecraft assets with progress reporting
pub async fn download_minecraft_assets_with_progress(
    version: String, 
    game_dir: String, 
    instance_id: &str, 
    app_handle: &tauri::AppHandle
) -> Result<(), String> {
    let game_path = PathBuf::from(&game_dir);
    let assets_dir = game_path.join("assets");
    
    // Create assets directory structure
    fs::create_dir_all(&assets_dir).await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
    
    // Emit progress update
    let _ = app_handle.emit("download_progress", serde_json::json!({
        "instance_id": instance_id,
        "phase": "assets",
        "progress": 0,
        "message": format!("Preparing to download assets for {}", version)
    }));
    
    // Load version manifest to get asset index
    if let Ok(Some(version_json)) = load_version_manifest(&game_path, &version).await {
        if let Some(asset_index) = version_json.get("assetIndex") {
            let index_id = asset_index.get("id").and_then(|v| v.as_str()).unwrap_or(&version);
            let index_url = asset_index.get("url").and_then(|v| v.as_str());
            
            if let Some(url) = index_url {
                // Emit progress update
                let _ = app_handle.emit("download_progress", serde_json::json!({
                    "instance_id": instance_id,
                    "phase": "assets",
                    "progress": 10,
                    "message": format!("Downloading asset index for {}", version)
                }));
                
                // Download asset index
                let indexes_dir = assets_dir.join("indexes");
                fs::create_dir_all(&indexes_dir).await
                    .map_err(|e| format!("Failed to create indexes directory: {}", e))?;
                
                let index_file = indexes_dir.join(format!("{}.json", index_id));
                let response = reqwest::get(url).await
                    .map_err(|e| format!("Failed to download asset index: {}", e))?;
                
                let index_content = response.text().await
                    .map_err(|e| format!("Failed to read asset index: {}", e))?;
                
                fs::write(&index_file, &index_content).await
                    .map_err(|e| format!("Failed to write asset index: {}", e))?;
                
                // Parse and download assets
                if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&index_content) {
                    if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
                        let objects_dir = assets_dir.join("objects");
                        fs::create_dir_all(&objects_dir).await
                            .map_err(|e| format!("Failed to create objects directory: {}", e))?;
                        
                        let mut downloaded = 0;
                        let total = objects.len();
                        
                        // Emit progress update
                        let _ = app_handle.emit("download_progress", serde_json::json!({
                            "instance_id": instance_id,
                            "phase": "assets",
                            "progress": 20,
                            "message": format!("Downloading {} assets", total)
                        }));
                        
                        for (_name, asset_info) in objects.iter() {
                            if let Some(hash) = asset_info.get("hash").and_then(|v| v.as_str()) {
                                let hash_prefix = &hash[0..2];
                                let object_dir = objects_dir.join(hash_prefix);
                                let object_file = object_dir.join(hash);
                                
                                if !object_file.exists() {
                                    fs::create_dir_all(&object_dir).await
                                        .map_err(|e| format!("Failed to create object directory: {}", e))?;
                                    
                                    let asset_url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);
                                    
                                    if let Ok(response) = reqwest::get(&asset_url).await {
                                        if let Ok(bytes) = response.bytes().await {
                                            let _ = fs::write(&object_file, &bytes).await;
                                        }
                                    }
                                }
                            }
                            downloaded += 1;
                            
                            // Update progress every 50 assets
                            if downloaded % 50 == 0 {
                                let progress = 20 + ((downloaded as f64 / total as f64) * 70.0) as u32;
                                let _ = app_handle.emit("download_progress", serde_json::json!({
                                    "instance_id": instance_id,
                                    "phase": "assets",
                                    "progress": progress,
                                    "message": format!("Downloaded {}/{} assets", downloaded, total)
                                }));
                            }
                        }
                        
                        // Emit completion
                        let _ = app_handle.emit("download_progress", serde_json::json!({
                            "instance_id": instance_id,
                            "phase": "assets",
                            "progress": 100,
                            "message": format!("Downloaded {} assets for {}", total, version)
                        }));
                        
                        println!("✓ Downloaded {} assets for {}", total, version);
                    }
                }
            }
        }
    }
    
    Ok(())
}