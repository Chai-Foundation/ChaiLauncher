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

/// Download a specific version manifest from Mojang API and save it to the local versions directory
pub async fn download_version_manifest(
    version_id: &str, 
    game_dir: &std::path::Path
) -> Result<serde_json::Value, String> {
    println!("🔍 Downloading version manifest for {}", version_id);
    
    // First, get the main version manifest to find the URL for our specific version
    let main_manifest = get_minecraft_versions().await?;
    
    // Find the version in the manifest
    let version_info = main_manifest.versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version {} not found in Minecraft version manifest", version_id))?;
    
    println!("🌐 Fetching version data from: {}", version_info.url);
    
    // Download the specific version manifest
    let response = reqwest::get(&version_info.url).await
        .map_err(|e| format!("Failed to download version manifest for {}: {}", version_id, e))?;
    
    let version_manifest: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse version manifest JSON for {}: {}", version_id, e))?;
    
    // Create the version directory
    let version_dir = game_dir.join("versions").join(version_id);
    fs::create_dir_all(&version_dir).await
        .map_err(|e| format!("Failed to create version directory: {}", e))?;
    
    // Save the manifest to {game_dir}/versions/{version_id}/{version_id}.json
    let manifest_path = version_dir.join(format!("{}.json", version_id));
    let manifest_content = serde_json::to_string_pretty(&version_manifest)
        .map_err(|e| format!("Failed to serialize version manifest: {}", e))?;
    
    fs::write(&manifest_path, &manifest_content).await
        .map_err(|e| format!("Failed to write version manifest to {}: {}", manifest_path.display(), e))?;
    
    println!("✅ Version manifest for {} saved to {}", version_id, manifest_path.display());
    
    Ok(version_manifest)
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
                                    
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(30),
                                        reqwest::get(&asset_url)
                                    ).await {
                                        Ok(Ok(response)) => {
                                            println!("✅ Asset HTTP response received");
                                            match tokio::time::timeout(
                                                std::time::Duration::from_secs(30),
                                                response.bytes()
                                            ).await {
                                                Ok(Ok(bytes)) => {
                                                    println!("📦 Asset bytes received ({} bytes)", bytes.len());
                                                    let _ = fs::write(&object_file, &bytes).await;
                                                    println!("💾 Asset written to file");
                                                }
                                                Ok(Err(e)) => println!("⚠️  Failed to read asset bytes {}: {}", hash, e),
                                                Err(_) => println!("⚠️  Timeout reading asset bytes: {}", hash),
                                            }
                                        }
                                        Ok(Err(e)) => println!("⚠️  Failed to download asset {}: {}", hash, e),
                                        Err(_) => println!("⚠️  Timeout downloading asset: {}", hash),
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
    app_handle: &tauri::AppHandle,
    version_manifest: &serde_json::Value
) -> Result<(), String> {
    println!("🎯 Entered download_minecraft_assets_with_progress for version: {}", version);
    println!("📂 Game directory: {}", game_dir);
    
    let game_path = PathBuf::from(&game_dir);
    let assets_dir = game_path.join("assets");
    
    // Create assets directory structure
    println!("📁 Creating assets directory: {}", assets_dir.display());
    fs::create_dir_all(&assets_dir).await
        .map_err(|e| format!("Failed to create assets directory: {}", e))?;
    println!("✅ Assets directory created successfully");
    
    // Emit progress update
    println!("📡 Emitting progress update...");
    let _ = app_handle.emit("download_progress", serde_json::json!({
        "instance_id": instance_id,
        "phase": "assets",
        "progress": 0,
        "message": format!("Preparing to download assets for {}", version)
    }));
    println!("✅ Progress update emitted");
    
    // Use the provided version manifest to get asset index
    println!("🔍 Looking for asset index in version manifest...");
    {
        let version_json = version_manifest;
        if let Some(asset_index) = version_json.get("assetIndex") {
            println!("✅ Found asset index in manifest");
            let index_id = asset_index.get("id").and_then(|v| v.as_str()).unwrap_or(&version);
            let index_url = asset_index.get("url").and_then(|v| v.as_str());
            println!("📋 Asset index ID: {}", index_id);
            println!("🌐 Asset index URL: {:?}", index_url);
            
            if let Some(url) = index_url {
                println!("🔽 Starting asset index download from: {}", url);
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
                
                let index_content = match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    reqwest::get(url)
                ).await {
                    Ok(Ok(response)) => {
                        println!("✅ Asset index downloaded successfully");
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(30),
                            response.text()
                        ).await {
                            Ok(Ok(content)) => {
                                println!("✅ Asset index content read successfully");
                                content
                            }
                            Ok(Err(e)) => return Err(format!("Failed to read asset index: {}", e)),
                            Err(_) => return Err("Timeout reading asset index content".to_string()),
                        }
                    }
                    Ok(Err(e)) => return Err(format!("Failed to download asset index: {}", e)),
                    Err(_) => return Err("Timeout downloading asset index".to_string()),
                };
                
                println!("💾 Writing asset index to file...");
                fs::write(&index_file, &index_content).await
                    .map_err(|e| format!("Failed to write asset index: {}", e))?;
                println!("✅ Asset index written to file successfully");
                
                // Parse and download assets
                println!("🔍 Parsing asset index JSON...");
                if let Ok(index_json) = serde_json::from_str::<serde_json::Value>(&index_content) {
                    println!("✅ Asset index JSON parsed successfully");
                    if let Some(objects) = index_json.get("objects").and_then(|v| v.as_object()) {
                        println!("📦 Found {} assets to download", objects.len());
                        let objects_dir = assets_dir.join("objects");
                        println!("📁 Creating objects directory...");
                        fs::create_dir_all(&objects_dir).await
                            .map_err(|e| format!("Failed to create objects directory: {}", e))?;
                        println!("✅ Objects directory created successfully");
                        
                        let mut downloaded = 0;
                        let total = objects.len();
                        
                        // Emit progress update
                        println!("📡 Emitting asset download progress update...");
                        let _ = app_handle.emit("download_progress", serde_json::json!({
                            "instance_id": instance_id,
                            "phase": "assets",
                            "progress": 20,
                            "message": format!("Downloading {} assets", total)
                        }));
                        println!("✅ Asset download progress update emitted");
                        
                        println!("🔄 Starting parallel asset download for {} assets (max 8 concurrent)...", total);
                        
                        use std::sync::Arc;
                        use tokio::sync::{Semaphore, Mutex};
                        use futures::future::join_all;
                        
                        let semaphore = Arc::new(Semaphore::new(8)); // Limit to 8 concurrent downloads
                        let downloaded_count = Arc::new(Mutex::new(0usize));
                        let failed_count = Arc::new(Mutex::new(0usize));
                        
                        let mut tasks = Vec::new();
                        
                        for (asset_name, asset_info) in objects.iter() {
                            if let Some(hash) = asset_info.get("hash").and_then(|v| v.as_str()) {
                                let asset_name = asset_name.clone();
                                let hash = hash.to_string();
                                let hash_prefix = hash[0..2].to_string();
                                let object_dir = objects_dir.join(&hash_prefix);
                                let object_file = object_dir.join(&hash);
                                
                                let permit = semaphore.clone();
                                let downloaded_count = downloaded_count.clone();
                                let failed_count = failed_count.clone();
                                let app_handle = app_handle.clone();
                                let instance_id = instance_id.to_string();
                                
                                let task = tokio::spawn(async move {
                                    let _permit = permit.acquire().await.unwrap();
                                    
                                    let mut success = false;
                                    if !object_file.exists() {
                                        println!("📥 Downloading asset: {} ({})", asset_name, hash);
                                        
                                        if let Some(parent) = object_file.parent() {
                                            let _ = fs::create_dir_all(parent).await;
                                        }
                                        
                                        let asset_url = format!("https://resources.download.minecraft.net/{}/{}", hash_prefix, hash);
                                        
                                        match tokio::time::timeout(
                                            std::time::Duration::from_secs(30),
                                            reqwest::get(&asset_url)
                                        ).await {
                                            Ok(Ok(response)) => {
                                                match tokio::time::timeout(
                                                    std::time::Duration::from_secs(30),
                                                    response.bytes()
                                                ).await {
                                                    Ok(Ok(bytes)) => {
                                                        if let Ok(_) = fs::write(&object_file, &bytes).await {
                                                            success = true;
                                                        } else {
                                                            println!("⚠️  Failed to write asset file: {}", asset_name);
                                                        }
                                                    }
                                                    Ok(Err(e)) => println!("⚠️  Failed to read asset bytes {}: {}", asset_name, e),
                                                    Err(_) => println!("⚠️  Timeout reading asset bytes: {}", asset_name),
                                                }
                                            }
                                            Ok(Err(e)) => println!("⚠️  Failed to download asset {}: {}", asset_name, e),
                                            Err(_) => println!("⚠️  Timeout downloading asset: {}", asset_name),
                                        }
                                    } else {
                                        success = true; // File already exists
                                    }
                                    
                                    // Update counters
                                    let mut downloaded = downloaded_count.lock().await;
                                    *downloaded += 1;
                                    let current_downloaded = *downloaded;
                                    
                                    if !success {
                                        let mut failed = failed_count.lock().await;
                                        *failed += 1;
                                    }
                                    
                                    // Update progress every 50 assets
                                    if current_downloaded % 50 == 0 {
                                        let progress = 20 + ((current_downloaded as f64 / total as f64) * 70.0) as u32;
                                        let _ = app_handle.emit("download_progress", serde_json::json!({
                                            "instance_id": instance_id,
                                            "phase": "assets",
                                            "progress": progress,
                                            "message": format!("Downloaded {}/{} assets", current_downloaded, total)
                                        }));
                                        println!("📊 Progress: {}/{} assets downloaded", current_downloaded, total);
                                    }
                                    
                                    success
                                });
                                
                                tasks.push(task);
                            }
                        }
                        
                        println!("🚀 Starting {} parallel asset download tasks...", tasks.len());
                        let results = join_all(tasks).await;
                        
                        let successful_downloads = results.iter().filter_map(|r| r.as_ref().ok()).filter(|&&success| success).count();
                        let total_failed = downloaded_count.lock().await.clone() - successful_downloads;
                        
                        println!("✅ Asset downloads completed: {} successful, {} failed", successful_downloads, total_failed);
                        
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
    
    println!("🎉 Asset download function completed successfully");
    Ok(())
}

/// Download Minecraft assets with progress reporting (wrapper that loads version manifest first)
pub async fn download_minecraft_assets_with_progress_auto(
    version: String, 
    game_dir: String, 
    instance_id: &str, 
    app_handle: &tauri::AppHandle
) -> Result<(), String> {
    let game_path = std::path::PathBuf::from(&game_dir);
    
    // Load version manifest first
    if let Ok(Some(version_manifest)) = load_version_manifest(&game_path, &version).await {
        download_minecraft_assets_with_progress(
            version, 
            game_dir, 
            instance_id, 
            app_handle, 
            &version_manifest
        ).await
    } else {
        Err(format!("Version manifest for {} not found", version))
    }
}