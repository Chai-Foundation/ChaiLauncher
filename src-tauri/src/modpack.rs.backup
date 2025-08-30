use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};
use reqwest::Client;
use tauri::Emitter;
use futures::StreamExt;
use chrono;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub changelog_url: Option<String>,
    pub date_published: String,
    pub downloads: u32,
    pub version_type: String,
    pub status: String,
    pub requested_status: Option<String>,
    pub files: Vec<ModrinthFile>,
    pub dependencies: Vec<ModrinthDependency>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthFile {
    pub hashes: HashMap<String, String>,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
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
                    version_id: "latest".to_string(), // Use "latest" as a safe default
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
        project_id: &str,
        version_id: &str,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f64, String) + Send + Sync,
    {
        progress_callback(0.0, "Fetching modpack information".to_string());

        // Get project details and versions
        let project_url = format!("https://api.modrinth.com/v2/project/{}", project_id);
        let project_response = self.client.get(&project_url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to get project details")?;
        let project_info: serde_json::Value = project_response.json().await?;

        let versions_url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
        let versions_response = self.client.get(&versions_url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to get project versions")?;

        let versions: Vec<ModrinthVersion> = versions_response.json().await
            .context("Failed to parse project versions")?;

        // Find the target version
        let version = if version_id.is_empty() || version_id == "latest" {
            versions.into_iter().next().context("No versions found for this project")?
        } else {
            let versions_clone = versions.clone();
            versions.into_iter()
                .find(|v| v.id == version_id || v.version_number == version_id)
                .or_else(|| versions_clone.into_iter().next())
                .context("No valid version found for this project")?
        };

        progress_callback(5.0, format!("Installing {} {}", 
            project_info["title"].as_str().unwrap_or("Modpack"), 
            version.version_number));

        // Determine Minecraft version and mod loaders
        let mc_version = version.game_versions.first()
            .context("No Minecraft version specified")?;
        let mod_loaders = &version.loaders;

        progress_callback(10.0, "Setting up Minecraft instance".to_string());

        // Create proper instance structure
        self.setup_instance_structure().await?;

        progress_callback(15.0, "Installing mod loaders".to_string());

        // Install mod loaders if specified
        if !mod_loaders.is_empty() {
            self.install_mod_loaders(mc_version, mod_loaders, |progress| {
                let progress = 15.0 + progress * 20.0;
                progress_callback(progress, "Installing mod loaders".to_string());
            }).await?;
        }

        progress_callback(35.0, "Downloading modpack file".to_string());

        // Find and download the primary modpack file
        let pack_file = version.files.iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .context("No files found for this version")?;

        let pack_path = self.instance_dir.join(&pack_file.filename);
        self.download_file(&pack_file.url, &pack_path, pack_file.size, |downloaded, total| {
            let progress = 35.0 + (downloaded as f64 / total as f64) * 15.0;
            progress_callback(progress, format!("Downloading {}", pack_file.filename));
        }).await?;

        progress_callback(50.0, "Extracting modpack".to_string());

        // Extract the modpack with proper structure
        self.extract_modpack_structured(&pack_path, mc_version).await?;

        progress_callback(60.0, "Installing modpack mods".to_string());

        // Install mods included in the modpack
        let manifest_path = self.instance_dir.join("modrinth.index.json");
        if manifest_path.exists() {
            self.install_modpack_mods(&manifest_path, |progress| {
                let progress = 60.0 + progress * 30.0;
                progress_callback(progress, "Installing mods".to_string());
            }).await?;
        }

        progress_callback(90.0, "Installing additional dependencies".to_string());

        // Install additional dependencies
        if !version.dependencies.is_empty() {
            self.install_additional_dependencies(&version.dependencies, |progress| {
                let progress = 90.0 + progress * 8.0;
                progress_callback(progress, "Installing dependencies".to_string());
            }).await?;
        }

        progress_callback(98.0, "Finalizing installation".to_string());

        // Generate instance configuration
        self.create_instance_config(&project_info, &version, mc_version, mod_loaders).await?;

        // Clean up downloaded pack file
        fs::remove_file(&pack_path).await.ok();

        progress_callback(100.0, format!("Successfully installed {}", 
            project_info["title"].as_str().unwrap_or("Modpack")));

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

    // CurseForge support removed - ChaiLauncher is now Modrinth-only
    /*
    pub async fn search_curseforge_packs(&self, query: &str, limit: u32) -> Result<Vec<CurseForgeModpack>> {
        // For now, we'll implement a basic search using CurseForge's public API
        // This is limited but functional without requiring an API key
        let url = format!(
            "https://api.curseforge.com/v1/mods/search?gameId=432&classId=4471&searchFilter={}&pageSize={}",
            urlencoding::encode(query),
            limit
        );

        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .header("Accept", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    #[derive(Deserialize)]
                    struct CurseForgeSearchResponse {
                        data: Vec<CurseForgeSearchHit>,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeSearchHit {
                        id: u32,
                        name: String,
                        summary: String,
                        #[serde(rename = "downloadCount")]
                        download_count: u32,
                        categories: Vec<CurseForgeCategorySimple>,
                        authors: Vec<CurseForgeAuthorSimple>,
                        logo: Option<CurseForgeLogoSimple>,
                        screenshots: Vec<CurseForgeScreenshotSimple>,
                        #[serde(rename = "mainFileId")]
                        main_file_id: u32,
                        #[serde(rename = "latestFiles")]
                        latest_files: Vec<CurseForgeFileSimple>,
                        #[serde(rename = "gameId")]
                        game_id: u32,
                        #[serde(rename = "gameName")]
                        game_name: String,
                        #[serde(rename = "gameSlug")]
                        game_slug: String,
                        #[serde(rename = "gameVersionLatestFiles")]
                        game_version_latest_files: Vec<CurseForgeGameVersionFileSimple>,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeCategorySimple {
                        id: u32,
                        name: String,
                        slug: String,
                        url: String,
                        #[serde(rename = "iconUrl")]
                        icon_url: String,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeAuthorSimple {
                        id: u32,
                        name: String,
                        url: String,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeLogoSimple {
                        id: u32,
                        #[serde(rename = "modId")]
                        mod_id: u32,
                        title: String,
                        description: String,
                        #[serde(rename = "thumbnailUrl")]
                        thumbnail_url: String,
                        url: String,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeScreenshotSimple {
                        id: u32,
                        #[serde(rename = "modId")]
                        mod_id: u32,
                        title: String,
                        description: String,
                        #[serde(rename = "thumbnailUrl")]
                        thumbnail_url: String,
                        url: String,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeFileSimple {
                        id: u32,
                        #[serde(rename = "gameId")]
                        game_id: u32,
                        #[serde(rename = "modId")]
                        mod_id: u32,
                        #[serde(rename = "isAvailable")]
                        is_available: bool,
                        #[serde(rename = "displayName")]
                        display_name: String,
                        #[serde(rename = "fileName")]
                        file_name: String,
                        #[serde(rename = "releaseType")]
                        release_type: u32,
                        #[serde(rename = "fileStatus")]
                        file_status: u32,
                        hashes: Vec<CurseForgeHashSimple>,
                        #[serde(rename = "fileDate")]
                        file_date: String,
                        #[serde(rename = "fileLength")]
                        file_length: u64,
                        #[serde(rename = "downloadCount")]
                        download_count: u32,
                        #[serde(rename = "downloadUrl")]
                        download_url: Option<String>,
                        #[serde(rename = "gameVersions")]
                        game_versions: Vec<String>,
                        #[serde(rename = "sortableGameVersions")]
                        sortable_game_versions: Vec<CurseForgeSortableGameVersionSimple>,
                        dependencies: Vec<CurseForgeDependencySimple>,
                        #[serde(rename = "alternateFileId")]
                        alternate_file_id: Option<u32>,
                        #[serde(rename = "isServerPack")]
                        is_server_pack: bool,
                        #[serde(rename = "fileFingerprint")]
                        file_fingerprint: u64,
                        modules: Vec<CurseForgeModuleSimple>,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeHashSimple {
                        value: String,
                        algo: u32,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeSortableGameVersionSimple {
                        #[serde(rename = "gameVersionName")]
                        game_version_name: String,
                        #[serde(rename = "gameVersionPadded")]
                        game_version_padded: String,
                        #[serde(rename = "gameVersion")]
                        game_version: String,
                        #[serde(rename = "gameVersionReleaseDate")]
                        game_version_release_date: String,
                        #[serde(rename = "gameVersionTypeId")]
                        game_version_type_id: Option<u32>,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeDependencySimple {
                        #[serde(rename = "modId")]
                        mod_id: u32,
                        #[serde(rename = "relationType")]
                        relation_type: u32,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeModuleSimple {
                        name: String,
                        fingerprint: u64,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeGameVersionFileSimple {
                        #[serde(rename = "gameVersion")]
                        game_version: String,
                        #[serde(rename = "projectFileId")]
                        project_file_id: u32,
                        #[serde(rename = "projectFileName")]
                        project_file_name: String,
                        #[serde(rename = "fileType")]
                        file_type: u32,
                        #[serde(rename = "gameVersionFlavor")]
                        game_version_flavor: Option<String>,
                    }

                    match resp.json::<CurseForgeSearchResponse>().await {
                        Ok(search_result) => {
                            let mut packs = Vec::new();
                            for hit in search_result.data {
                                let pack = CurseForgeModpack {
                                    id: hit.id,
                                    name: hit.name,
                                    summary: hit.summary,
                                    download_count: hit.download_count,
                                    categories: hit.categories.into_iter().map(|c| CurseForgeCategory {
                                        id: c.id,
                                        name: c.name,
                                        slug: c.slug,
                                        url: c.url,
                                        icon_url: c.icon_url,
                                    }).collect(),
                                    authors: hit.authors.into_iter().map(|a| CurseForgeAuthor {
                                        id: a.id,
                                        name: a.name,
                                        url: a.url,
                                    }).collect(),
                                    logo: hit.logo.map(|l| CurseForgeLogo {
                                        id: l.id,
                                        mod_id: l.mod_id,
                                        title: l.title,
                                        description: l.description,
                                        thumbnail_url: l.thumbnail_url,
                                        url: l.url,
                                    }),
                                    screenshots: hit.screenshots.into_iter().map(|s| CurseForgeScreenshot {
                                        id: s.id,
                                        mod_id: s.mod_id,
                                        title: s.title,
                                        description: s.description,
                                        thumbnail_url: s.thumbnail_url,
                                        url: s.url,
                                    }).collect(),
                                    main_file_id: hit.main_file_id,
                                    latest_files: hit.latest_files.into_iter().map(|f| CurseForgeFile {
                                        id: f.id,
                                        game_id: f.game_id,
                                        mod_id: f.mod_id,
                                        is_available: f.is_available,
                                        display_name: f.display_name,
                                        file_name: f.file_name,
                                        release_type: f.release_type,
                                        file_status: f.file_status,
                                        hashes: f.hashes.into_iter().map(|h| CurseForgeHash {
                                            value: h.value,
                                            algo: h.algo,
                                        }).collect(),
                                        file_date: f.file_date,
                                        file_length: f.file_length,
                                        download_count: f.download_count,
                                        download_url: f.download_url,
                                        game_versions: f.game_versions,
                                        sortable_game_versions: f.sortable_game_versions.into_iter().map(|sgv| CurseForgeSortableGameVersion {
                                            game_version_name: sgv.game_version_name,
                                            game_version_padded: sgv.game_version_padded,
                                            game_version: sgv.game_version,
                                            game_version_release_date: sgv.game_version_release_date,
                                            game_version_type_id: sgv.game_version_type_id,
                                        }).collect(),
                                        dependencies: f.dependencies.into_iter().map(|d| CurseForgeDependency {
                                            mod_id: d.mod_id,
                                            relation_type: d.relation_type,
                                        }).collect(),
                                        alternate_file_id: f.alternate_file_id,
                                        is_server_pack: f.is_server_pack,
                                        file_fingerprint: f.file_fingerprint,
                                        modules: f.modules.into_iter().map(|m| CurseForgeModule {
                                            name: m.name,
                                            fingerprint: m.fingerprint,
                                        }).collect(),
                                    }).collect(),
                                    game_id: hit.game_id,
                                    game_name: hit.game_name,
                                    game_slug: hit.game_slug,
                                    game_version_latest_files: hit.game_version_latest_files.into_iter().map(|gvf| CurseForgeGameVersionFile {
                                        game_version: gvf.game_version,
                                        project_file_id: gvf.project_file_id,
                                        project_file_name: gvf.project_file_name,
                                        file_type: gvf.file_type,
                                        game_version_flavor: gvf.game_version_flavor,
                                    }).collect(),
                                };
                                packs.push(pack);
                            }
                            Ok(packs)
                        }
                        Err(e) => {
                            eprintln!("Failed to parse CurseForge search response: {}", e);
                            Ok(Vec::new()) // Return empty list on parse error
                        }
                    }
                } else {
                    eprintln!("CurseForge API returned status: {}", resp.status());
                    Ok(Vec::new()) // Return empty list on API error
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to CurseForge API: {}", e);
                Ok(Vec::new()) // Return empty list on network error
            }
        }
    }
    */

    /*
    pub async fn install_curseforge_pack<F>(
        &self,
        modpack_id: u32,
        file_id: u32,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f64, String) + Send + Sync,
    {
        progress_callback(0.0, "Fetching CurseForge modpack information".to_string());

        // Get modpack file details
        let url = format!("https://api.curseforge.com/v1/mods/{}/files/{}", modpack_id, file_id);
        let response = self.client.get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .header("Accept", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    #[derive(Deserialize)]
                    struct CurseForgeFileResponse {
                        data: CurseForgeFileSimple,
                    }

                    #[derive(Deserialize)]
                    struct CurseForgeFileSimple {
                        id: u32,
                        #[serde(rename = "fileName")]
                        file_name: String,
                        #[serde(rename = "downloadUrl")]
                        download_url: Option<String>,
                        #[serde(rename = "fileLength")]
                        file_length: u64,
                    }

                    match resp.json::<CurseForgeFileResponse>().await {
                        Ok(file_response) => {
                            let file_data = file_response.data;
                            
                            if let Some(download_url) = file_data.download_url {
                                progress_callback(10.0, "Downloading CurseForge modpack".to_string());

                                // Download the modpack file
                                let pack_path = self.instance_dir.join(&file_data.file_name);
                                self.download_file(&download_url, &pack_path, file_data.file_length, |downloaded, total| {
                                    let progress = 10.0 + (downloaded as f64 / total as f64) * 40.0;
                                    progress_callback(progress, format!("Downloading {}", file_data.file_name));
                                }).await?;

                                progress_callback(50.0, "Extracting CurseForge modpack".to_string());

                                // Extract the modpack (CurseForge packs are typically ZIP files)
                                self.extract_modpack(&pack_path).await?;

                                progress_callback(70.0, "Processing modpack manifest".to_string());

                                // CurseForge modpacks contain a manifest.json file with mod information
                                self.process_curseforge_manifest().await?;

                                progress_callback(95.0, "Finalizing installation".to_string());

                                // Clean up downloaded pack file
                                fs::remove_file(&pack_path).await.ok();

                                progress_callback(100.0, "CurseForge modpack installation complete".to_string());
                                Ok(())
                            } else {
                                Err(anyhow::anyhow!("No download URL available for this CurseForge modpack"))
                            }
                        }
                        Err(e) => {
                            Err(anyhow::anyhow!("Failed to parse CurseForge file response: {}", e))
                        }
                    }
                } else {
                    Err(anyhow::anyhow!("CurseForge API returned status: {}", resp.status()))
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to connect to CurseForge API: {}", e))
            }
        }
    }

    async fn process_curseforge_manifest(&self) -> Result<()> {
        let manifest_path = self.instance_dir.join("manifest.json");
        
        if !manifest_path.exists() {
            return Ok(()); // No manifest found, skip processing
        }

        #[derive(Deserialize)]
        struct CurseForgeManifest {
            name: String,
            version: String,
            author: String,
            #[serde(rename = "manifestType")]
            manifest_type: String,
            #[serde(rename = "manifestVersion")]
            manifest_version: u32,
            #[serde(rename = "minecraftVersion")]
            minecraft_version: String,
            #[serde(rename = "modLoaders")]
            mod_loaders: Vec<CurseForgeModLoader>,
            files: Vec<CurseForgeModFile>,
        }

        #[derive(Deserialize)]
        struct CurseForgeModLoader {
            id: String,
            primary: bool,
        }

        #[derive(Deserialize)]
        struct CurseForgeModFile {
            #[serde(rename = "projectID")]
            project_id: u32,
            #[serde(rename = "fileID")]
            file_id: u32,
            required: bool,
        }

        let manifest_content = fs::read_to_string(&manifest_path).await
            .context("Failed to read manifest.json")?;
        
        let manifest: CurseForgeManifest = serde_json::from_str(&manifest_content)
            .context("Failed to parse manifest.json")?;

        println!("Processing CurseForge modpack: {} v{}", manifest.name, manifest.version);
        println!("Minecraft version: {}", manifest.minecraft_version);
        println!("Mod count: {}", manifest.files.len());

        // Create mods directory
        let mods_dir = self.instance_dir.join("mods");
        fs::create_dir_all(&mods_dir).await
            .context("Failed to create mods directory")?;

        // Download required mods
        for (i, mod_file) in manifest.files.iter().enumerate() {
            if mod_file.required {
                println!("Downloading mod {}/{}: project_id={}, file_id={}", 
                    i + 1, manifest.files.len(), mod_file.project_id, mod_file.file_id);
                
                // This would require downloading each mod file individually
                // For now, we'll just log the required files
                // In a full implementation, we'd download from CurseForge API
            }
        }

        Ok(())
    }
    */

    // New helper methods for proper modpack installation

    async fn setup_instance_structure(&self) -> Result<()> {
        // Create standard Minecraft instance directories
        let dirs = ["mods", "config", "resourcepacks", "shaderpacks", "saves", "screenshots"];
        for dir in &dirs {
            fs::create_dir_all(self.instance_dir.join(dir)).await
                .context(format!("Failed to create {} directory", dir))?;
        }
        Ok(())
    }

    async fn install_mod_loaders<F>(&self, mc_version: &str, loaders: &[String], progress_callback: F) -> Result<()>
    where
        F: Fn(f64),
    {
        use crate::mods::loaders::ModLoaderManager;
        use crate::mods::types::ModLoader;

        let loader_manager = ModLoaderManager::new(self.instance_dir.clone());
        
        for (i, loader_name) in loaders.iter().enumerate() {
            progress_callback(i as f64 / loaders.len() as f64);
            
            // Get latest version for this loader and MC version
            let versions = loader_manager.get_available_versions(loader_name, mc_version).await?;
            if let Some(latest_version) = versions.first() {
                let mod_loader = match loader_name.to_lowercase().as_str() {
                    "forge" => ModLoader::Forge(latest_version.clone()),
                    "fabric" => ModLoader::Fabric(latest_version.clone()),
                    "quilt" => ModLoader::Quilt(latest_version.clone()),
                    "neoforge" => ModLoader::NeoForge(latest_version.clone()),
                    _ => continue,
                };
                
                loader_manager.install_loader(&mod_loader, mc_version).await
                    .context(format!("Failed to install {} {}", loader_name, latest_version))?;
            }
        }
        
        progress_callback(1.0);
        Ok(())
    }

    async fn extract_modpack_structured(&self, pack_path: &PathBuf, _mc_version: &str) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Read;

        let file = std::fs::File::open(pack_path)
            .context("Failed to open modpack file")?;
        let mut archive = ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;

        // Collect extraction jobs synchronously first to avoid Send issues
        let mut extraction_jobs = Vec::new();
        let mut override_jobs = Vec::new();
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read file from archive")?;
            
            let file_path = match file.enclosed_name() {
                Some(path) => path.to_path_buf(),
                None => continue,
            };

            let is_directory = file.name().ends_with('/');

            if !is_directory {
                // Read file content synchronously
                let mut contents = Vec::new();
                file.read_to_end(&mut contents)
                    .context("Failed to read file content")?;

                // Check if this file is in an overrides directory
                if let Some(path_str) = file_path.to_str() {
                    if path_str.starts_with("overrides/") || path_str.starts_with("overrides\\") {
                        // Override file - extract to instance root, removing "overrides/" prefix
                        // Handle both Unix and Windows path separators
                        let relative_path = if path_str.starts_with("overrides/") {
                            file_path.strip_prefix("overrides/").unwrap_or(&file_path)
                        } else {
                            file_path.strip_prefix("overrides\\").unwrap_or(&file_path)
                        };
                        let outpath = self.instance_dir.join(relative_path);
                        override_jobs.push((outpath, contents));
                        continue;
                    }
                }

                // Regular file - extract to instance directory maintaining structure
                let outpath = self.instance_dir.join(&file_path);
                extraction_jobs.push((outpath, contents));
            } else {
                // Directory - create it unless it's the overrides directory itself
                if let Some(path_str) = file_path.to_str() {
                    if !path_str.starts_with("overrides/") && !path_str.starts_with("overrides\\") && path_str != "overrides" {
                        let outpath = self.instance_dir.join(&file_path);
                        extraction_jobs.push((outpath, Vec::new())); // Empty for directories
                    }
                }
            }
        }

        // Extract regular files first
        for (outpath, contents) in extraction_jobs {
            if contents.is_empty() && outpath.to_string_lossy().ends_with('/') {
                // Directory
                fs::create_dir_all(&outpath).await
                    .context("Failed to create directory")?;
            } else {
                // File - create parent dirs if needed
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent).await
                        .context("Failed to create parent directory")?;
                }

                // Write file
                fs::write(&outpath, contents).await
                    .context("Failed to write extracted file")?;
            }
        }

        // Extract override files (these should overwrite any existing files)
        for (outpath, contents) in override_jobs {
            // Create parent dirs if needed
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).await
                    .context("Failed to create override parent directory")?;
            }

            // Write override file (overwrites existing files)
            fs::write(&outpath, contents).await
                .context("Failed to write override file")?;
        }
        Ok(())
    }

    async fn install_modpack_mods<F>(&self, manifest_path: &PathBuf, progress_callback: F) -> Result<()>
    where
        F: Fn(f64),
    {
        // Read Modrinth modpack manifest
        let manifest_content = fs::read_to_string(manifest_path).await
            .context("Failed to read modpack manifest")?;
        
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .context("Failed to parse modpack manifest")?;

        let files = manifest["files"].as_array()
            .context("No files section in manifest")?;

        let mods_dir = self.instance_dir.join("mods");
        fs::create_dir_all(&mods_dir).await?;

        let total_files = files.len();
        for (i, file_info) in files.iter().enumerate() {
            progress_callback(i as f64 / total_files as f64);

            if let Some(downloads) = file_info["downloads"].as_array() {
                if let Some(download_url) = downloads.first().and_then(|d| d.as_str()) {
                    if let Some(path) = file_info["path"].as_str() {
                        let file_path = self.instance_dir.join(path);
                        
                        // Create parent directories if needed
                        if let Some(parent) = file_path.parent() {
                            fs::create_dir_all(parent).await?;
                        }

                        // Download file
                        let file_size = file_info["fileSize"].as_u64().unwrap_or(0);
                        self.download_file(download_url, &file_path, file_size, |_, _| {}).await
                            .context(format!("Failed to download {}", path))?;
                    }
                }
            }
        }

        progress_callback(1.0);
        Ok(())
    }

    async fn install_additional_dependencies<F>(&self, dependencies: &[ModrinthDependency], progress_callback: F) -> Result<()>
    where
        F: Fn(f64),
    {
        let mods_dir = self.instance_dir.join("mods");
        fs::create_dir_all(&mods_dir).await?;

        let total_deps = dependencies.len();
        for (i, dep) in dependencies.iter().enumerate() {
            progress_callback(i as f64 / total_deps as f64);

            if dep.dependency_type == "required" {
                if let Some(version_id) = &dep.version_id {
                    // Get dependency details
                    let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
                    let response = self.client.get(&url)
                        .header("User-Agent", "ChaiLauncher/2.0.0")
                        .send()
                        .await
                        .context("Failed to get dependency details")?;

                    let version: ModrinthVersion = response.json().await
                        .context("Failed to parse dependency details")?;

                    // Download primary file
                    if let Some(file) = version.files.iter().find(|f| f.primary).or_else(|| version.files.first()) {
                        let mod_path = mods_dir.join(&file.filename);
                        self.download_file(&file.url, &mod_path, file.size, |_, _| {}).await
                            .context(format!("Failed to download dependency {}", file.filename))?;
                    }
                }
            }
        }

        progress_callback(1.0);
        Ok(())
    }

    async fn create_instance_config(
        &self, 
        project_info: &serde_json::Value, 
        version: &ModrinthVersion, 
        mc_version: &str,
        mod_loaders: &[String]
    ) -> Result<()> {
        // Create instance metadata file for ChaiLauncher
        let instance_config = serde_json::json!({
            "name": project_info["title"].as_str().unwrap_or("Modpack"),
            "description": project_info["description"].as_str().unwrap_or(""),
            "minecraft_version": mc_version,
            "modpack_id": project_info["id"].as_str().unwrap_or(""),
            "modpack_version": version.version_number,
            "modpack_version_id": version.id,
            "mod_loaders": mod_loaders,
            "created_date": chrono::Utc::now().to_rfc3339(),
            "icon_url": project_info["icon_url"].as_str(),
            "source": "modrinth"
        });

        let config_path = self.instance_dir.join("instance.json");
        let config_content = serde_json::to_string_pretty(&instance_config)?;
        fs::write(config_path, config_content).await
            .context("Failed to write instance configuration")?;

        // Register instance with ChaiLauncher's storage system
        self.register_with_chailauncher(project_info, version, mc_version, mod_loaders).await?;

        Ok(())
    }

    /// Register the installed modpack instance with ChaiLauncher's storage system
    async fn register_with_chailauncher(
        &self,
        project_info: &serde_json::Value,
        version: &ModrinthVersion,
        mc_version: &str,
        mod_loaders: &[String],
    ) -> Result<()> {
        use crate::storage::{StorageManager, InstanceMetadata};
        use uuid::Uuid;

        // Create a unique instance ID
        let instance_id = Uuid::new_v4().to_string();
        
        // Count mods in the mods directory
        let mods_dir = self.instance_dir.join("mods");
        let mods_count = if mods_dir.exists() {
            match fs::read_dir(&mods_dir).await {
                Ok(mut entries) => {
                    let mut count = 0;
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".jar") && !name.starts_with(".") {
                                count += 1;
                            }
                        }
                    }
                    count
                }
                Err(_) => 0,
            }
        } else {
            0
        };

        // Create instance metadata
        let instance_metadata = InstanceMetadata {
            id: instance_id,
            name: project_info["title"].as_str().unwrap_or("Modpack").to_string(),
            version: mc_version.to_string(),
            modpack: Some(project_info["title"].as_str().unwrap_or("Modpack").to_string()),
            modpack_version: Some(version.version_number.clone()),
            game_dir: self.instance_dir.clone(),
            java_path: None, // Will be set by launcher
            jvm_args: None,  // Will be set by launcher
            last_played: None,
            total_play_time: 0,
            icon: project_info["icon_url"].as_str().map(|s| s.to_string()),
            is_modded: !mod_loaders.is_empty() || mods_count > 0,
            mods_count,
            created_at: chrono::Utc::now().to_rfc3339(),
            size_mb: None, // Will be calculated by storage manager
            description: project_info["description"].as_str().map(|s| s.to_string()),
            tags: vec!["modpack".to_string(), "modrinth".to_string()],
            resolved_java_version: None,
            java_analysis_date: None,
        };

        // Register with storage manager
        let mut storage = StorageManager::new().await
            .context("Failed to initialize storage manager")?;
        
        storage.add_instance(instance_metadata).await
            .context("Failed to register instance with ChaiLauncher")?;

        println!("✅ Instance registered with ChaiLauncher storage system");
        Ok(())
    }
}

use tauri::command;

#[command]
pub async fn search_modpacks(query: String, platform: String, limit: u32, offset: Option<u32>) -> Result<Vec<ModrinthPack>, String> {
    let installer = ModpackInstaller::new(PathBuf::new());
    
    match platform.as_str() {
        "modrinth" => {
            installer.search_modrinth_packs(&query, limit, offset.unwrap_or(0)).await
                .map_err(|e| format!("Failed to search modpacks: {}", e))
        }
        "curseforge" => {
            Err("CurseForge support has been removed. Please use Modrinth instead.".to_string())
        }
        _ => Err("Unsupported platform".to_string())
    }
}

#[command]
pub async fn get_modpack_versions(
    project_id: String,
    platform: String,
) -> Result<Vec<ModrinthVersion>, String> {
    let installer = ModpackInstaller::new(PathBuf::new());
    
    match platform.as_str() {
        "modrinth" => {
            installer.get_modrinth_pack_versions(&project_id).await
                .map_err(|e| format!("Failed to get modpack versions: {}", e))
        }
        "curseforge" => {
            Err("CurseForge support has been removed. Please use Modrinth instead.".to_string())
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
        "curseforge" => {
            Err("CurseForge support has been removed. Please use Modrinth instead.".to_string())
        }
        _ => Err("Unsupported platform".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackCreationRequest {
    pub instance_id: String,
    pub instance_path: String,
    pub metadata: ModpackMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub minecraft_version: String,
    pub tags: Vec<String>,
    pub icon_path: Option<String>,
    pub include_user_data: bool,
    pub include_resource_packs: bool,
    pub include_shader_packs: bool,
    pub include_config: bool,
    pub include_saves: bool,
}

#[command]
pub async fn create_modpack(
    request: ModpackCreationRequest,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let creator = ModpackCreator::new();
    let instance_id = request.instance_id.clone(); // Clone before moving into closure
    
    let app_handle_clone = app_handle.clone();
    let result = creator.create_modpack(&request, move |progress, stage| {
        let _ = app_handle_clone.emit("modpack_creation_progress", ModpackCreationProgress {
            instance_id: instance_id.clone(),
            progress,
            stage,
        });
    }).await;
    
    match result {
        Ok(modpack_path) => Ok(modpack_path),
        Err(e) => Err(format!("Failed to create modpack: {}", e))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ModpackCreationProgress {
    pub instance_id: String,
    pub progress: f64,
    pub stage: String,
}

pub struct ModpackCreator;

impl ModpackCreator {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn create_modpack<F>(
        &self,
        request: &ModpackCreationRequest,
        progress_callback: F,
    ) -> Result<String>
    where
        F: Fn(f64, String) + Send + Sync,
    {
        let instance_path = PathBuf::from(&request.instance_path);
        let modpack_name = request.metadata.name.replace(" ", "_");
        let output_path = instance_path.parent()
            .unwrap_or(&PathBuf::from("."))
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
        
        progress_callback(40.0, "Processing configuration files".to_string());
        
        // Copy config directory if requested
        if request.metadata.include_config {
            let config_src = instance_path.join("config");
            if config_src.exists() {
                let config_dst = temp_dir.join("config");
                self.copy_directory_sync(&config_src, &config_dst)?;
            }
        }
        
        progress_callback(55.0, "Including resource packs".to_string());
        
        // Copy resource packs if requested
        if request.metadata.include_resource_packs {
            let resourcepacks_src = instance_path.join("resourcepacks");
            if resourcepacks_src.exists() {
                let resourcepacks_dst = temp_dir.join("resourcepacks");
                self.copy_directory_sync(&resourcepacks_src, &resourcepacks_dst)?;
            }
        }
        
        // Copy shader packs if requested
        if request.metadata.include_shader_packs {
            let shaderpacks_src = instance_path.join("shaderpacks");
            if shaderpacks_src.exists() {
                let shaderpacks_dst = temp_dir.join("shaderpacks");
                self.copy_directory_sync(&shaderpacks_src, &shaderpacks_dst)?;
            }
        }
        
        progress_callback(65.0, "Including world saves".to_string());
        
        // Copy saves if requested
        if request.metadata.include_saves {
            let saves_src = instance_path.join("saves");
            if saves_src.exists() {
                let saves_dst = temp_dir.join("saves");
                self.copy_directory_sync(&saves_src, &saves_dst)?;
            }
        }
        
        progress_callback(75.0, "Creating modpack manifest".to_string());
        
        // Create CurseForge-compatible manifest
        self.create_manifest(&temp_dir, &request.metadata).await?;
        
        progress_callback(85.0, "Compressing modpack archive".to_string());
        
        // Create ZIP archive
        self.create_zip_archive(&temp_dir, &output_path).await?;
        
        progress_callback(95.0, "Cleaning up temporary files".to_string());
        
        // Clean up temporary directory
        fs::remove_dir_all(&temp_dir).await.ok();
        
        progress_callback(100.0, "Modpack creation complete".to_string());
        
        Ok(output_path.to_string_lossy().to_string())
    }
    
    fn copy_directory_sync(&self, src: &PathBuf, dst: &PathBuf) -> Result<()> {
        std::fs::create_dir_all(dst)?;
        
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if entry_path.is_dir() {
                self.copy_directory_sync(&entry_path, &dst_path)?;
            } else {
                std::fs::copy(&entry_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
    
    async fn create_manifest(&self, temp_dir: &PathBuf, metadata: &ModpackMetadata) -> Result<()> {
        use serde_json::json;
        
        // Create a CurseForge-compatible manifest
        let manifest = json!({
            "manifestType": "minecraftModpack",
            "manifestVersion": 1,
            "name": metadata.name,
            "version": metadata.version,
            "author": metadata.author,
            "description": metadata.description,
            "minecraft": {
                "version": metadata.minecraft_version,
                "modLoaders": [
                    {
                        "id": "forge-latest",
                        "primary": true
                    }
                ]
            },
            "files": [],
            "overrides": "overrides"
        });
        
        let manifest_path = temp_dir.join("manifest.json");
        let manifest_content = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_content).await?;
        
        // Create modlist.html for human-readable mod list
        let modlist_html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{} v{} - Mod List</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #2c3e50; }}
        .info {{ background: #ecf0f1; padding: 20px; border-radius: 5px; margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>{} v{}</h1>
    <div class="info">
        <p><strong>Author:</strong> {}</p>
        <p><strong>Minecraft Version:</strong> {}</p>
        <p><strong>Description:</strong> {}</p>
        <p><strong>Tags:</strong> {}</p>
    </div>
    <p>This modpack was created with ChaiLauncher.</p>
</body>
</html>"#,
            metadata.name,
            metadata.version,
            metadata.name,
            metadata.version,
            metadata.author,
            metadata.minecraft_version,
            metadata.description,
            metadata.tags.join(", ")
        );
        
        let modlist_path = temp_dir.join("modlist.html");
        fs::write(&modlist_path, modlist_html).await?;
        
        Ok(())
    }
    
    async fn create_zip_archive(&self, source_dir: &PathBuf, output_path: &PathBuf) -> Result<()> {
        let file = std::fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        self.add_directory_to_zip_sync(&mut zip, source_dir, source_dir)?;
        
        zip.finish()?;
        Ok(())
    }
    
    fn add_directory_to_zip_sync<W: std::io::Write + std::io::Seek>(
        &self,
        zip: &mut zip::ZipWriter<W>,
        base_path: &PathBuf,
        current_path: &PathBuf,
    ) -> Result<()> {
        use std::io::Write;
        
        for entry in std::fs::read_dir(current_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(base_path)
                .map_err(|e| anyhow::anyhow!("Failed to create relative path: {}", e))?;
            
            if entry_path.is_dir() {
                // Add directory
                zip.add_directory::<String, ()>(
                    relative_path.to_string_lossy().to_string() + "/",
                    zip::write::FileOptions::default(),
                )?;
                
                // Recursively add contents
                self.add_directory_to_zip_sync(zip, base_path, &entry_path)?;
            } else {
                // Add file
                let file_content = std::fs::read(&entry_path)?;
                zip.start_file::<String, ()>(
                    relative_path.to_string_lossy().to_string(),
                    zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated),
                )?;
                zip.write_all(&file_content)?;
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ModpackInstallProgress {
    pub instance_dir: String,
    pub progress: f64,
    pub stage: String,
}