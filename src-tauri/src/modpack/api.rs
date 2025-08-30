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
            stage: "Extracting modpack".to_string(),
        });

        // Extract modpack to instance directory
        self.extract_modpack(&temp_file).await
            .context("Failed to extract modpack")?;

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
        // .mrpack files are essentially ZIP files with a specific structure
        self.extract_zip(mrpack_path).await
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