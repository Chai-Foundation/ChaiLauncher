use anyhow::{Result, Context};

use super::types::*;

impl ModpackInstaller {
    pub async fn search_modrinth_packs(&self, query: &str, limit: u32, offset: u32) -> Result<Vec<ModrinthPack>> {
        let url = format!(
            "https://api.modrinth.com/v2/search?query={}&facets=[[\"project_type:modpack\"]]&limit={}&offset={}",
            urlencoding::encode(query),
            limit,
            offset
        );

        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to search Modrinth packs")?;

        #[derive(serde::Deserialize)]
        struct SearchResponse {
            hits: Vec<ModrinthSearchHit>,
        }

        #[derive(serde::Deserialize)]
        struct ModrinthSearchHit {
            project_id: String,
            project_type: String,
            slug: String,
            author: String,
            title: String,
            description: String,
            categories: Vec<String>,
            client_side: String,
            server_side: String,
            downloads: u32,
            follows: u32,
            icon_url: Option<String>,
            date_created: String,
            date_modified: String,
            latest_version: Option<String>,
            license: String,
            gallery: Vec<String>,
            featured_gallery: Option<String>,
        }

        let search_response: SearchResponse = response.json().await
            .context("Failed to parse search response")?;

        let packs: Vec<ModrinthPack> = search_response.hits.into_iter().map(|hit| {
            ModrinthPack {
                project_id: hit.project_id,
                version_id: hit.latest_version.unwrap_or_default(),
                name: hit.title,
                description: hit.description,
                author: hit.author,
                game_versions: vec![], // Will be populated from version info
                loaders: vec![], // Will be populated from version info  
                downloads: hit.downloads,
                icon_url: hit.icon_url,
                website_url: None,
            }
        }).collect();

        Ok(packs)
    }

    pub async fn get_modpack_versions(&self, project_id: &str) -> Result<Vec<ModrinthVersion>> {
        let url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);

        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")  
            .send()
            .await
            .context("Failed to get modpack versions")?;

        let versions: Vec<ModrinthVersion> = response.json().await
            .context("Failed to parse versions response")?;

        Ok(versions)
    }

    pub async fn download_and_install_modpack(
        &self,
        version: &ModrinthVersion,
        progress_callback: impl Fn(ModpackInstallProgress) + Send + Sync,
    ) -> Result<()> {
        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 0.0,
            stage: "Starting modpack installation".to_string(),
        });

        // Get the primary modpack file
        let modpack_file = version.files.iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .context("No modpack file found")?;

        // Download modpack file
        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 10.0,
            stage: "Downloading modpack".to_string(),
        });

        let response = self.client.get(&modpack_file.url).send().await
            .context("Failed to download modpack")?;
        
        let modpack_data = response.bytes().await
            .context("Failed to read modpack data")?;

        // Create temporary file for the modpack
        let temp_file = std::env::temp_dir().join(&modpack_file.filename);
        tokio::fs::write(&temp_file, &modpack_data).await
            .context("Failed to write modpack file")?;

        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 30.0,
            stage: "Installing modpack".to_string(),
        });

        // Handle modpack installation based on file type
        let extension = temp_file.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "mrpack" => {
                // For .mrpack files, use our proper implementation that downloads individual mods
                self.extract_mrpack(&temp_file).await
                    .context("Failed to install .mrpack modpack")?;
            },
            "zip" => {
                // For .zip files, use traditional extraction
                self.extract_zip(&temp_file).await
                    .context("Failed to extract .zip modpack")?;
            },
            _ => {
                return Err(anyhow::anyhow!("Unsupported modpack format: {}", extension));
            }
        }

        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 100.0,
            stage: "Modpack installation complete".to_string(),
        });

        // Clean up temporary file
        let _ = tokio::fs::remove_file(&temp_file).await;

        Ok(())
    }

    async fn extract_modpack(&self, modpack_path: &std::path::Path) -> Result<()> {
        // Create instance directory if it doesn't exist
        tokio::fs::create_dir_all(&self.instance_dir).await
            .context("Failed to create instance directory")?;

        // Extract based on file type
        let extension = modpack_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "zip" => self.extract_zip(modpack_path).await,
            "mrpack" => self.extract_mrpack(modpack_path).await,
            _ => Err(anyhow::anyhow!("Unsupported modpack format: {}", extension)),
        }
    }

    async fn extract_zip(&self, zip_path: &std::path::Path) -> Result<()> {
        use zip::ZipArchive;
        use std::fs::File;
        use std::io::Read;

        let zip_path = zip_path.to_path_buf();
        let instance_dir = self.instance_dir.clone();
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = File::open(&zip_path)
                .context("Failed to open ZIP file")?;
            
            let mut archive = ZipArchive::new(file)
                .context("Failed to read ZIP archive")?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .context("Failed to read file from archive")?;
                
                let outpath = match file.enclosed_name() {
                    Some(path) => instance_dir.join(path),
                    None => continue,
                };

                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath)
                        .context("Failed to create directory")?;
                } else {
                    if let Some(p) = outpath.parent() {
                        std::fs::create_dir_all(p)
                            .context("Failed to create parent directory")?;
                    }
                    
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .context("Failed to read file content")?;
                    
                    std::fs::write(&outpath, &buffer)
                        .context("Failed to write extracted file")?;
                }
            }

            Ok(())
        }).await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

        Ok(())
    }

    async fn extract_mrpack(&self, mrpack_path: &std::path::Path) -> Result<()> {
        use zip::ZipArchive;
        use std::fs::File;
        use std::io::Read;
        use super::types::MrpackIndex;

        println!("🔧 Processing .mrpack file with proper dependency resolution");
        
        let mrpack_path = mrpack_path.to_path_buf();
        let instance_dir = self.instance_dir.clone();
        
        let index = tokio::task::spawn_blocking(move || -> Result<MrpackIndex> {
            let file = File::open(&mrpack_path)
                .context("Failed to open mrpack file")?;
            
            let mut archive = ZipArchive::new(file)
                .context("Failed to read mrpack archive")?;

            // Step 1: Parse modrinth.index.json
            let mut index_json = String::new();
            {
                let mut index_file = archive.by_name("modrinth.index.json")
                    .context("Missing modrinth.index.json in mrpack file")?;
                index_file.read_to_string(&mut index_json)
                    .context("Failed to read modrinth.index.json")?;
            }
            
            let index: MrpackIndex = serde_json::from_str(&index_json)
                .context("Failed to parse modrinth.index.json")?;

            // Validate format
            if index.format_version != 1 {
                return Err(anyhow::anyhow!("Unsupported mrpack format version: {}", index.format_version));
            }
            
            if index.game != "minecraft" {
                return Err(anyhow::anyhow!("Unsupported game: {}", index.game));
            }

            println!("📦 Installing modpack '{}' (version {})", index.name, index.version_id);
            println!("🔍 Found {} files to download", index.files.len());

            // Step 2: Handle overrides first (extract from ZIP)
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)
                    .context("Failed to read file from archive")?;
                
                let file_path = file.name().to_string();
                
                // Only extract overrides and client-overrides (ignore server-overrides for client)
                if file_path.starts_with("overrides/") || file_path.starts_with("client-overrides/") {
                    let relative_path = if file_path.starts_with("overrides/") {
                        &file_path[10..] // Remove "overrides/" prefix
                    } else {
                        &file_path[17..] // Remove "client-overrides/" prefix
                    };
                    
                    if relative_path.is_empty() || file_path.ends_with('/') {
                        continue; // Skip directories and empty paths
                    }
                    
                    let outpath = instance_dir.join(relative_path);
                    
                    if let Some(p) = outpath.parent() {
                        std::fs::create_dir_all(p)
                            .context("Failed to create parent directory")?;
                    }
                    
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .context("Failed to read file content")?;
                    
                    std::fs::write(&outpath, &buffer)
                        .context("Failed to write extracted file")?;
                    
                    println!("📁 Extracted override: {}", relative_path);
                }
            }

            Ok(index)
        }).await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

        // Step 3: Download individual mod files (async part)
        self.download_mrpack_files(&index.files).await
            .context("Failed to download mrpack files")?;

        println!("✅ Successfully installed mrpack with proper dependency resolution");
        Ok(())
    }

    /// Download individual mod files from mrpack index with hash verification (parallel)
    async fn download_mrpack_files(&self, files: &[super::types::MrpackFile]) -> Result<()> {
        use std::collections::HashSet;
        use std::sync::{Arc, Mutex};
        use tokio::sync::Semaphore;
        use futures::stream::{FuturesUnordered, StreamExt};

        // Create mods directory
        let mods_dir = self.instance_dir.join("mods");
        tokio::fs::create_dir_all(&mods_dir).await
            .context("Failed to create mods directory")?;

        // Thread-safe collections for tracking downloads
        let downloaded_files = Arc::new(Mutex::new(HashSet::new()));
        let failed_downloads = Arc::new(Mutex::new(Vec::new()));
        
        // Limit concurrent downloads to prevent overwhelming the server/network
        // Use environment variable or default to 8 concurrent downloads
        let max_concurrent = std::env::var("CHAI_MAX_CONCURRENT_DOWNLOADS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8)
            .clamp(1, 16); // Clamp between 1 and 16
        let semaphore = Arc::new(Semaphore::new(max_concurrent));
        let total_files = files.len();
        let completed_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        println!("🚀 Starting parallel download of {} mod files (max {} concurrent)", total_files, max_concurrent);

        // Create futures for all downloads
        let mut download_futures = FuturesUnordered::new();
        
        for (i, file) in files.iter().enumerate() {
            // Skip duplicates early
            if downloaded_files.lock().unwrap().contains(&file.path) {
                println!("⚠️  Skipping duplicate file: {}", file.path);
                completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                continue;
            }

            let semaphore = semaphore.clone();
            let downloaded_files = downloaded_files.clone();
            let failed_downloads = failed_downloads.clone();
            let completed_count = completed_count.clone();
            let client = self.client.clone();
            let instance_dir = self.instance_dir.clone();
            
            let file_path = file.path.clone();
            let file_downloads = file.downloads.clone();
            let expected_sha1 = file.hashes.sha1.clone();
            let file_size = file.file_size;
            let file_index = i + 1;

            let download_future = async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                // Check again if already downloaded (race condition protection)
                if downloaded_files.lock().unwrap().contains(&file_path) {
                    let current = completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    println!("⚠️  [{}/{}] Skipping duplicate file: {}", current, total_files, file_path);
                    return Ok(());
                }

                println!("📥 [{}/{}] Downloading: {}", file_index, total_files, file_path);

                // Try each download URL with retry logic
                let mut download_success = false;
                for (url_index, url) in file_downloads.iter().enumerate() {
                    // Retry each URL up to 3 times
                    for retry_attempt in 0..3 {
                        match Self::download_and_verify_file_static(
                            &client, 
                            &instance_dir, 
                            url, 
                            &file_path, 
                            &expected_sha1, 
                            file_size
                        ).await {
                            Ok(_) => {
                                downloaded_files.lock().unwrap().insert(file_path.clone());
                                download_success = true;
                                let current = completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                                if retry_attempt > 0 {
                                    println!("✅ [{}/{}] Downloaded (retry {}): {}", current, total_files, retry_attempt + 1, file_path);
                                } else {
                                    println!("✅ [{}/{}] Downloaded: {}", current, total_files, file_path);
                                }
                                break;
                            }
                            Err(e) => {
                                if retry_attempt < 2 {
                                    println!("⚠️  Retry {}/3 for {} from URL {}: {}", retry_attempt + 1, file_path, url_index + 1, e);
                                    tokio::time::sleep(std::time::Duration::from_millis(1000 * (retry_attempt + 1) as u64)).await;
                                } else {
                                    eprintln!("⚠️  Failed to download {} from URL {}: {}", file_path, url_index + 1, e);
                                }
                                continue;
                            }
                        }
                    }
                    
                    if download_success {
                        break;
                    }
                }

                if !download_success {
                    failed_downloads.lock().unwrap().push(file_path.clone());
                    let current = completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    eprintln!("❌ [{}/{}] Failed to download: {}", current, total_files, file_path);
                    return Err(anyhow::anyhow!("Failed to download: {}", file_path));
                }

                Ok(())
            };

            download_futures.push(download_future);
        }

        // Execute all downloads concurrently
        let mut success_count = 0;
        let mut failure_count = 0;

        while let Some(result) = download_futures.next().await {
            match result {
                Ok(_) => success_count += 1,
                Err(_) => failure_count += 1,
            }
        }

        // Check final results
        let failed = failed_downloads.lock().unwrap();
        if !failed.is_empty() {
            return Err(anyhow::anyhow!("Failed to download {} files: {:?}", 
                failed.len(), *failed));
        }

        let downloaded = downloaded_files.lock().unwrap();
        println!("🎉 Successfully downloaded {} mod files in parallel (Success: {}, Skipped duplicates: {})", 
            downloaded.len(), success_count, total_files - success_count - failure_count);
        Ok(())
    }

    /// Download a file and verify its hash
    async fn download_and_verify_file(&self, url: &str, filename: &str, expected_sha1: &str, expected_size: u64) -> Result<()> {
        use sha1::{Digest, Sha1};

        let file_path = self.instance_dir.join("mods").join(filename);
        
        // Skip if file already exists with correct hash
        if file_path.exists() {
            if let Ok(existing_data) = tokio::fs::read(&file_path).await {
                if existing_data.len() as u64 == expected_size {
                    let mut hasher = Sha1::new();
                    hasher.update(&existing_data);
                    let hash = hex::encode(hasher.finalize());
                    
                    if hash == expected_sha1 {
                        println!("✅ File already exists with correct hash: {}", filename);
                        return Ok(());
                    }
                }
            }
        }

        // Download the file
        let response = self.client.get(url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error {}: {}", response.status(), url));
        }

        let data = response.bytes().await
            .context("Failed to read file data")?;

        // Verify file size
        if data.len() as u64 != expected_size {
            return Err(anyhow::anyhow!("File size mismatch: expected {}, got {}", expected_size, data.len()));
        }

        // Verify SHA1 hash
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let hash = hex::encode(hasher.finalize());
        
        if hash != expected_sha1 {
            return Err(anyhow::anyhow!("Hash mismatch: expected {}, got {}", expected_sha1, hash));
        }

        // Write the file
        tokio::fs::write(&file_path, &data).await
            .context("Failed to write file")?;

        println!("✅ Downloaded and verified: {}", filename);
        Ok(())
    }

    /// Static version of download_and_verify_file for use in parallel contexts
    async fn download_and_verify_file_static(
        client: &reqwest::Client,
        instance_dir: &std::path::Path,
        url: &str, 
        filename: &str, 
        expected_sha1: &str, 
        expected_size: u64
    ) -> Result<()> {
        use sha1::{Digest, Sha1};

        let file_path = instance_dir.join("mods").join(filename);
        
        // Skip if file already exists with correct hash
        if file_path.exists() {
            if let Ok(existing_data) = tokio::fs::read(&file_path).await {
                if existing_data.len() as u64 == expected_size {
                    let mut hasher = Sha1::new();
                    hasher.update(&existing_data);
                    let hash = hex::encode(hasher.finalize());
                    
                    if hash == expected_sha1 {
                        return Ok(()); // File already exists with correct hash
                    }
                }
            }
        }

        // Download the file
        let response = client.get(url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error {}: {}", response.status(), url));
        }

        let data = response.bytes().await
            .context("Failed to read file data")?;

        // Verify file size
        if data.len() as u64 != expected_size {
            return Err(anyhow::anyhow!("File size mismatch: expected {}, got {}", expected_size, data.len()));
        }

        // Verify SHA1 hash
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let hash = hex::encode(hasher.finalize());
        
        if hash != expected_sha1 {
            return Err(anyhow::anyhow!("Hash mismatch: expected {}, got {}", expected_sha1, hash));
        }

        // Write the file
        tokio::fs::write(&file_path, &data).await
            .context("Failed to write file")?;

        Ok(())
    }

    /// Install .mrpack directly from version metadata without downloading the pack file
    pub async fn install_mrpack_direct<F>(&self, version: &ModrinthVersion, mut progress_callback: F) -> Result<()>
    where
        F: FnMut(ModpackInstallProgress),
    {
        use super::types::{MrpackIndex, MrpackFile, MrpackHashes};

        println!("🔧 Installing modpack '{}' directly from metadata", version.name);
        
        // Create instance directory
        tokio::fs::create_dir_all(&self.instance_dir).await
            .context("Failed to create instance directory")?;

        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 10.0,
            stage: "Preparing modpack installation".to_string(),
        });

        // Convert ModrinthVersion to our internal format for processing
        let mrpack_files: Vec<MrpackFile> = version.files.iter().map(|file| {
            MrpackFile {
                path: file.filename.clone(),
                hashes: MrpackHashes {
                    sha1: file.hashes.get("sha1").cloned().unwrap_or_default(),
                    sha512: file.hashes.get("sha512").cloned(),
                },
                env: None, // Modrinth doesn't provide this in the same format
                downloads: vec![file.url.clone()],
                file_size: file.size as u64,
            }
        }).collect();

        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 20.0,
            stage: format!("Downloading {} mods in parallel", mrpack_files.len()),
        });

        // Use our parallel download system
        self.download_mrpack_files(&mrpack_files).await
            .context("Failed to download modpack files")?;

        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 90.0,
            stage: "Finalizing modpack installation".to_string(),
        });

        // TODO: Handle overrides if they exist in version metadata
        // For now, Modrinth versions don't typically include override data in the API

        progress_callback(ModpackInstallProgress {
            instance_dir: self.instance_dir.to_string_lossy().to_string(),
            progress: 100.0,
            stage: "Modpack installation complete".to_string(),
        });

        println!("✅ Successfully installed modpack '{}' with {} mods", version.name, mrpack_files.len());
        Ok(())
    }
}

impl ModpackCreator {
    pub async fn create_modpack<F>(
        &self,
        request: &ModpackCreationRequest,
        progress_callback: F,
    ) -> Result<String>
    where
        F: Fn(f64, String) + Send + Sync,
    {
        use tokio::fs;
        
        let instance_path = std::path::PathBuf::from(&request.instance_path);
        let modpack_name = request.metadata.name.replace(" ", "_");
        let output_path = instance_path.parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .join(format!("{}_v{}.zip", modpack_name, request.metadata.version));
        
        progress_callback(0.0, "Initializing modpack creation".to_string());
        
        if !instance_path.exists() {
            return Err(anyhow::anyhow!("Instance path does not exist: {}", instance_path.display()));
        }
        
        progress_callback(10.0, "Analyzing instance files".to_string());
        
        // Create a temporary directory for modpack assembly
        let temp_dir = std::env::temp_dir().join(format!("modpack_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).await
            .context("Failed to create temporary directory")?;
        
        progress_callback(20.0, "Collecting mods and dependencies".to_string());
        
        // Copy mods directory
        let mods_src = instance_path.join("mods");
        if mods_src.exists() {
            let mods_dst = temp_dir.join("mods");
            self.copy_directory_sync(&mods_src, &mods_dst)?;
        }
        
        progress_callback(40.0, "Copying configuration files".to_string());
        
        // Copy config directory if requested
        if request.metadata.include_config {
            let config_src = instance_path.join("config");
            if config_src.exists() {
                let config_dst = temp_dir.join("config");
                self.copy_directory_sync(&config_src, &config_dst)?;
            }
        }
        
        progress_callback(60.0, "Including resource packs and shaders".to_string());
        
        // Copy resource packs if requested
        if request.metadata.include_resource_packs {
            let rp_src = instance_path.join("resourcepacks");
            if rp_src.exists() {
                let rp_dst = temp_dir.join("resourcepacks");
                self.copy_directory_sync(&rp_src, &rp_dst)?;
            }
        }
        
        // Copy shader packs if requested
        if request.metadata.include_shader_packs {
            let sp_src = instance_path.join("shaderpacks");
            if sp_src.exists() {
                let sp_dst = temp_dir.join("shaderpacks");
                self.copy_directory_sync(&sp_src, &sp_dst)?;
            }
        }
        
        progress_callback(80.0, "Creating modpack archive".to_string());
        
        // Create the ZIP archive
        self.create_zip_archive(&temp_dir, &output_path).await
            .context("Failed to create modpack archive")?;
        
        progress_callback(90.0, "Cleaning up temporary files".to_string());
        
        // Clean up temporary directory
        let _ = fs::remove_dir_all(&temp_dir).await;
        
        progress_callback(100.0, "Modpack creation complete".to_string());
        
        Ok(output_path.to_string_lossy().to_string())
    }
    
    fn copy_directory_sync(&self, src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        use walkdir::WalkDir;
        use std::fs;
        
        for entry in WalkDir::new(src) {
            let entry = entry.context("Failed to read directory entry")?;
            let src_path = entry.path();
            let relative_path = src_path.strip_prefix(src)
                .context("Failed to get relative path")?;
            let dst_path = dst.join(relative_path);
            
            if src_path.is_dir() {
                fs::create_dir_all(&dst_path)
                    .context("Failed to create directory")?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)
                        .context("Failed to create parent directory")?;
                }
                fs::copy(src_path, &dst_path)
                    .context("Failed to copy file")?;
            }
        }
        
        Ok(())
    }
    
    async fn create_zip_archive(&self, src_dir: &std::path::Path, output_path: &std::path::Path) -> Result<()> {
        use zip::{ZipWriter, CompressionMethod};
        use std::fs::File;
        use std::io::Write;
        use walkdir::WalkDir;
        
        let file = File::create(output_path)
            .context("Failed to create output ZIP file")?;
        let mut zip = ZipWriter::new(file);
        
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(CompressionMethod::Deflated);
        
        for entry in WalkDir::new(src_dir) {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            let name = path.strip_prefix(src_dir)
                .context("Failed to get relative path")?;
            
            if path.is_file() {
                zip.start_file(name.to_string_lossy().as_ref(), options)
                    .context("Failed to start ZIP file entry")?;
                let content = std::fs::read(path)
                    .context("Failed to read file content")?;
                zip.write_all(&content)
                    .context("Failed to write to ZIP")?;
            } else if path.is_dir() && !name.as_os_str().is_empty() {
                zip.add_directory(name.to_string_lossy().as_ref(), options)
                    .context("Failed to add directory to ZIP")?;
            }
        }
        
        zip.finish()
            .context("Failed to finalize ZIP archive")?;
        
        Ok(())
    }
}