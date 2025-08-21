use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};
use reqwest::Client;
use tauri::Emitter;
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthPack {
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u32,
    pub icon_url: Option<String>,
    pub website_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub date_published: String,
    pub downloads: u32,
    pub version_type: String,
    pub status: String,
    pub requested_status: String,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeModpack {
    pub id: u32,
    pub name: String,
    pub summary: String,
    pub download_count: u32,
    pub categories: Vec<CurseForgeCategory>,
    pub authors: Vec<CurseForgeAuthor>,
    pub logo: Option<CurseForgeLogo>,
    pub screenshots: Vec<CurseForgeScreenshot>,
    pub main_file_id: u32,
    pub latest_files: Vec<CurseForgeFile>,
    pub game_id: u32,
    pub game_name: String,
    pub game_slug: String,
    pub game_version_latest_files: Vec<CurseForgeGameVersionFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeCategory {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub icon_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeAuthor {
    pub id: u32,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeLogo {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeScreenshot {
    pub id: u32,
    pub mod_id: u32,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeFile {
    pub id: u32,
    pub game_id: u32,
    pub mod_id: u32,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: u32,
    pub file_status: u32,
    pub hashes: Vec<CurseForgeHash>,
    pub file_date: String,
    pub file_length: u64,
    pub download_count: u32,
    pub download_url: Option<String>,
    pub game_versions: Vec<String>,
    pub sortable_game_versions: Vec<CurseForgeSortableGameVersion>,
    pub dependencies: Vec<CurseForgeDependency>,
    pub alternate_file_id: Option<u32>,
    pub is_server_pack: bool,
    pub file_fingerprint: u64,
    pub modules: Vec<CurseForgeModule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeHash {
    pub value: String,
    pub algo: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeSortableGameVersion {
    pub game_version_name: String,
    pub game_version_padded: String,
    pub game_version: String,
    pub game_version_release_date: String,
    pub game_version_type_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeDependency {
    pub mod_id: u32,
    pub relation_type: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeModule {
    pub name: String,
    pub fingerprint: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurseForgeGameVersionFile {
    pub game_version: String,
    pub project_file_id: u32,
    pub project_file_name: String,
    pub file_type: u32,
    pub game_version_flavor: Option<String>,
}

pub struct ModpackInstaller {
    client: Client,
    instance_dir: PathBuf,
}

impl ModpackInstaller {
    pub fn new(instance_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            instance_dir,
        }
    }

    pub async fn search_modrinth_packs(&self, query: &str, limit: u32) -> Result<Vec<ModrinthPack>> {
        let url = format!(
            "https://api.modrinth.com/v2/search?query={}&facets=[[\"project_type:modpack\"]]&limit={}",
            urlencoding::encode(query),
            limit
        );

        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to search Modrinth packs")?;

        #[derive(Deserialize)]
        struct SearchResponse {
            hits: Vec<ModrinthSearchHit>,
        }

        #[derive(Deserialize)]
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
            latest_version: String,
            license: String,
            gallery: Vec<String>,
            featured_gallery: Option<String>,
            versions: Vec<String>,
        }

        let search_result: SearchResponse = response.json().await
            .context("Failed to parse search response")?;

        let mut packs = Vec::new();
        for hit in search_result.hits {
            if hit.project_type == "modpack" {
                let pack = ModrinthPack {
                    project_id: hit.project_id,
                    version_id: hit.latest_version,
                    name: hit.title,
                    description: hit.description,
                    author: hit.author,
                    game_versions: hit.versions,
                    loaders: hit.categories,
                    downloads: hit.downloads,
                    icon_url: hit.icon_url,
                    website_url: None,
                };
                packs.push(pack);
            }
        }

        Ok(packs)
    }

    pub async fn get_modrinth_pack_versions(&self, project_id: &str) -> Result<Vec<ModrinthVersion>> {
        let url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);

        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to get pack versions")?;

        let versions: Vec<ModrinthVersion> = response.json().await
            .context("Failed to parse versions response")?;

        Ok(versions)
    }

    pub async fn install_modrinth_pack<F>(
        &self,
    _project_id: &str,
        version_id: &str,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f64, String) + Send + Sync,
    {
        progress_callback(0.0, "Fetching modpack information".to_string());

        // Get version details
        let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to get version details")?;

        let version: ModrinthVersion = response.json().await
            .context("Failed to parse version details")?;

        progress_callback(10.0, "Downloading modpack file".to_string());

        // Find the primary file
        let pack_file = version.files.iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .context("No files found for this version")?;

        // Download the modpack file
        let pack_path = self.instance_dir.join(&pack_file.filename);
        self.download_file(&pack_file.url, &pack_path, pack_file.size, |downloaded, total| {
            let progress = 10.0 + (downloaded as f64 / total as f64) * 30.0;
            progress_callback(progress, format!("Downloading {}", pack_file.filename));
        }).await?;

        progress_callback(40.0, "Extracting modpack".to_string());

        // Extract the modpack
        self.extract_modpack(&pack_path).await?;

        progress_callback(60.0, "Installing dependencies".to_string());

        // Install dependencies (mods)
        self.install_dependencies(&version.dependencies, |progress| {
            let progress = 60.0 + progress * 35.0;
            progress_callback(progress, "Installing mods".to_string());
        }).await?;

        progress_callback(95.0, "Finalizing installation".to_string());

        // Clean up downloaded pack file
        fs::remove_file(&pack_path).await.ok();

        progress_callback(100.0, "Installation complete".to_string());

        Ok(())
    }

    async fn download_file<F>(
        &self,
        url: &str,
        path: &PathBuf,
        expected_size: u64,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64),
    {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create parent directory")?;
        }

        let response = self.client.get(url).send().await
            .context("Failed to start download")?;
        
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let mut file = fs::File::create(path).await
            .context("Failed to create file")?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk).await
                .context("Failed to write chunk")?;
            
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, expected_size);
        }

        file.flush().await.context("Failed to flush file")?;
        Ok(())
    }

    async fn extract_modpack(&self, pack_path: &PathBuf) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Read;

        let file = std::fs::File::open(pack_path)
            .context("Failed to open modpack file")?;
        let mut archive = ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;

        // Collect extraction jobs synchronously
        let mut jobs = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read ZIP entry")?;
            let outpath = self.instance_dir.join(file.name());
            if file.name().ends_with('/') {
                jobs.push((outpath, None));
            } else {
                if let Some(parent) = outpath.parent() {
                    jobs.push((parent.to_path_buf(), None));
                }
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)
                    .context("Failed to read file contents")?;
                jobs.push((outpath, Some(contents)));
            }
        }

        // Perform async writes
        for (path, contents) in jobs {
            if let Some(data) = contents {
                fs::write(&path, data).await
                    .context("Failed to write extracted file")?;
            } else {
                fs::create_dir_all(&path).await
                    .context("Failed to create directory")?;
            }
        }
        Ok(())
    }

    async fn install_dependencies<F>(&self, dependencies: &[ModrinthDependency], progress_callback: F) -> Result<()>
    where
        F: Fn(f64),
    {
        let mods_dir = self.instance_dir.join("mods");
        fs::create_dir_all(&mods_dir).await
            .context("Failed to create mods directory")?;

        let total_deps = dependencies.len();
        for (i, dep) in dependencies.iter().enumerate() {
            if dep.dependency_type == "required" {
                if let Some(version_id) = &dep.version_id {
                    // Download the dependency
                    let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
                    let response = self.client.get(&url)
                        .header("User-Agent", "ChaiLauncher/2.0.0")
                        .send()
                        .await
                        .context("Failed to get dependency details")?;

                    let version: ModrinthVersion = response.json().await
                        .context("Failed to parse dependency details")?;

                    if let Some(file) = version.files.iter().find(|f| f.primary).or_else(|| version.files.first()) {
                        let mod_path = mods_dir.join(&file.filename);
                        self.download_file(&file.url, &mod_path, file.size, |_, _| {}).await?;
                    }
                }
            }
            
            progress_callback(i as f64 / total_deps as f64);
        }

        Ok(())
    }

    pub async fn search_curseforge_packs(&self, _query: &str, _limit: u32) -> Result<Vec<CurseForgeModpack>> {
        // This would require a CurseForge API key
        // For now, return empty list
        Ok(Vec::new())
    }

    pub async fn install_curseforge_pack<F>(
        &self,
        _modpack_id: u32,
        _file_id: u32,
        _progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f64, String) + Send + Sync,
    {
        // CurseForge installation would go here
        // This requires API key and proper implementation
        Err(anyhow::anyhow!("CurseForge installation not yet implemented"))
    }
}

// Command functions for Tauri
use tauri::command;

#[command]
pub async fn search_modpacks(query: String, platform: String, limit: u32) -> Result<Vec<ModrinthPack>, String> {
    let installer = ModpackInstaller::new(PathBuf::new());
    
    match platform.as_str() {
        "modrinth" => {
            installer.search_modrinth_packs(&query, limit).await
                .map_err(|e| format!("Failed to search modpacks: {}", e))
        }
        "curseforge" => {
            installer.search_curseforge_packs(&query, limit).await
                .map_err(|e| format!("Failed to search modpacks: {}", e))
                .map(|packs| {
                    packs.into_iter().map(|cf_pack| {
                        ModrinthPack {
                            project_id: cf_pack.id.to_string(),
                            version_id: cf_pack.main_file_id.to_string(),
                            name: cf_pack.name,
                            description: cf_pack.summary,
                            author: cf_pack.authors.get(0).map_or(String::new(), |a| a.name.clone()),
                            game_versions: vec![cf_pack.game_name],
                            loaders: vec![],
                            downloads: cf_pack.download_count,
                            icon_url: cf_pack.logo.as_ref().map(|l| l.url.clone()),
                            website_url: None,
                        }
                    }).collect()
                })
        }
        _ => Err("Unsupported platform".to_string())
    }
}

#[command]
pub async fn install_modpack(
    instance_dir: String,
    platform: String,
    project_id: String,
    version_id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let installer = ModpackInstaller::new(PathBuf::from(&instance_dir));
    
    match platform.as_str() {
        "modrinth" => {
            let app_handle_clone = app_handle.clone();
            installer.install_modrinth_pack(&project_id, &version_id, move |progress, stage| {
                let _ = app_handle_clone.emit("modpack_install_progress", ModpackInstallProgress {
                    instance_dir: instance_dir.clone(),
                    progress,
                    stage,
                });
            }).await.map_err(|e| format!("Failed to install modpack: {}", e))
        }
        _ => Err("Unsupported platform".to_string())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ModpackInstallProgress {
    pub instance_dir: String,
    pub progress: f64,
    pub stage: String,
}