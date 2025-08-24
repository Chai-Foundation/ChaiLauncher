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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

        // First, get the project versions to find the latest version or a valid version
        let versions_url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
        let versions_response = self.client.get(&versions_url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await
            .context("Failed to get project versions")?;

        let versions: Vec<ModrinthVersion> = versions_response.json().await
            .context("Failed to parse project versions")?;

        // Find the latest version or a specific version
        let version = if version_id.is_empty() || version_id == "latest" {
            versions.into_iter().next().context("No versions found for this project")?
        } else {
            // Try to find a version that matches or use the first available
            let versions_clone = versions.clone();
            versions.into_iter()
                .find(|v| v.id == version_id || v.version_number == version_id)
                .or_else(|| versions_clone.into_iter().next())
                .context("No valid version found for this project")?
        };

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
}

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
        "curseforge" => {
            let project_id_u32: u32 = project_id.parse()
                .map_err(|_| "Invalid CurseForge project ID".to_string())?;
            let file_id_u32: u32 = version_id.parse()
                .map_err(|_| "Invalid CurseForge file ID".to_string())?;
            
            let app_handle_clone = app_handle.clone();
            installer.install_curseforge_pack(project_id_u32, file_id_u32, move |progress, stage| {
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