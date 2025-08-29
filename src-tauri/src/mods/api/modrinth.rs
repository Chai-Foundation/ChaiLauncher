use async_trait::async_trait;
use reqwest::Client;
use crate::mods::types::*;
use crate::mods::api::ModApi;
use std::path::Path;
use serde_json;
use chrono::{DateTime, Utc};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

/// Modrinth API client implementation
#[derive(Debug)]
pub struct ModrinthApi {
    client: Client,
    base_url: String,
}

impl ModrinthApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }

    async fn make_request<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, ModError> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModError::Api(reqwest::Error::from(response.error_for_status().unwrap_err())));
        }

        let json = response.json().await?;
        Ok(json)
    }

    fn convert_modrinth_project_to_mod_info(&self, project: serde_json::Value) -> Result<ModInfo, ModError> {
        // Handle both search results and project details
        let author = project["author"].as_str()
            .or_else(|| project["team"].as_str())
            .unwrap_or("Unknown")
            .to_string();

        // For search results, use latest_version. For project details, use first version from versions array
        let version = project["latest_version"].as_str()
            .or_else(|| {
                project["versions"].as_array()
                    .and_then(|versions| versions.first())
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("Unknown")
            .to_string();

        // Use project_id for search results, id for project details
        let id = project["project_id"].as_str()
            .or_else(|| project["id"].as_str())
            .unwrap_or_default()
            .to_string();

        Ok(ModInfo {
            id,
            name: project["title"].as_str().unwrap_or_default().to_string(),
            description: project["description"].as_str().unwrap_or_default().to_string(),
            author,
            version,
            game_versions: project["game_versions"]
                .as_array()
                .or_else(|| project["versions"].as_array()) // fallback for search results
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect(),
            loaders: project["loaders"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect(),
            downloads: project["downloads"].as_u64().unwrap_or(0) as u32,
            icon_url: project["icon_url"].as_str().map(String::from),
            website_url: project["issues_url"].as_str()
                .or_else(|| project["project_url"].as_str())
                .or_else(|| project["website_url"].as_str())
                .map(String::from),
            source_url: project["source_url"].as_str().map(String::from),
            license: project["license"]
                .as_object()
                .and_then(|l| l["id"].as_str())
                .or_else(|| project["license"].as_str())
                .map(String::from),
            categories: project["categories"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect(),
            side: match project["client_side"].as_str().unwrap_or("unknown") {
                "required" => match project["server_side"].as_str().unwrap_or("unknown") {
                    "required" => ModSide::Both,
                    "optional" | "unsupported" => ModSide::Client,
                    _ => ModSide::Client,
                },
                "optional" => match project["server_side"].as_str().unwrap_or("unknown") {
                    "required" => ModSide::Server,
                    _ => ModSide::Both,
                },
                _ => ModSide::Unknown,
            },
            source: ModSource::Modrinth,
            featured: project["featured"].as_bool().unwrap_or(false),
            date_created: DateTime::parse_from_rfc3339(
                project["date_created"].as_str()
                    .or_else(|| project["published"].as_str())
                    .unwrap_or("2020-01-01T00:00:00Z")
            ).unwrap_or_default().with_timezone(&Utc),
            date_updated: DateTime::parse_from_rfc3339(
                project["date_modified"].as_str()
                    .or_else(|| project["updated"].as_str())
                    .unwrap_or("2020-01-01T00:00:00Z")
            ).unwrap_or_default().with_timezone(&Utc),
        })
    }

    fn convert_modrinth_version_to_mod_file(&self, version: serde_json::Value) -> Result<ModFile, ModError> {
        let empty_vec = vec![];
        let files = version["files"].as_array().unwrap_or(&empty_vec);
        
        // Skip versions that have no files (this can happen with some Modrinth projects)
        if files.is_empty() {
            return Err(ModError::InvalidFile("Version has no downloadable files".to_string()));
        }
        
        let primary_file = files.iter().find(|f| f["primary"].as_bool().unwrap_or(false))
            .or_else(|| files.first())
            .ok_or_else(|| ModError::InvalidFile("No valid files found in version".to_string()))?;

        Ok(ModFile {
            id: version["id"].as_str().unwrap_or_default().to_string(),
            mod_id: version["project_id"].as_str().unwrap_or_default().to_string(),
            filename: primary_file["filename"].as_str().unwrap_or_default().to_string(),
            display_name: version["name"].as_str().unwrap_or_default().to_string(),
            version: version["version_number"].as_str().unwrap_or_default().to_string(),
            size: primary_file["size"].as_u64().unwrap_or(0),
            download_url: primary_file["url"].as_str().unwrap_or_default().to_string(),
            hashes: primary_file["hashes"]
                .as_object()
                .unwrap_or(&serde_json::Map::new())
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect(),
            dependencies: version["dependencies"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|dep| {
                    Some(ModDependency {
                        mod_id: dep["project_id"].as_str()?.to_string(),
                        version_id: dep["version_id"].as_str().map(String::from),
                        file_name: dep["file_name"].as_str().map(String::from),
                        dependency_type: match dep["dependency_type"].as_str()? {
                            "required" => DependencyType::Required,
                            "optional" => DependencyType::Optional,
                            "incompatible" => DependencyType::Incompatible,
                            "embedded" => DependencyType::Embedded,
                            _ => DependencyType::Optional,
                        },
                    })
                })
                .collect(),
            game_versions: version["game_versions"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect(),
            loaders: version["loaders"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect(),
            release_type: match version["version_type"].as_str().unwrap_or("release") {
                "release" => ReleaseType::Release,
                "beta" => ReleaseType::Beta,
                "alpha" => ReleaseType::Alpha,
                _ => ReleaseType::Release,
            },
            date_published: DateTime::parse_from_rfc3339(
                version["date_published"].as_str().unwrap_or("2020-01-01T00:00:00Z")
            ).unwrap_or_default().with_timezone(&Utc),
            primary: primary_file["primary"].as_bool().unwrap_or(false),
        })
    }
}

#[async_trait]
impl ModApi for ModrinthApi {
    async fn search_mods(
        &self,
        query: &str,
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        
        let mut params = vec![
            ("query", query),
            ("limit", &limit_str),
            ("offset", &offset_str),
        ];

        let mut facets = vec![];
        
        // Always filter for mods (not modpacks)
        facets.push("[\"project_type:mod\"]".to_string());
        
        if let Some(version) = game_version {
            facets.push(format!("[\"versions:{}\"]", version));
        }
        
        if let Some(loader) = mod_loader {
            facets.push(format!("[\"categories:{}\"]", loader));
        }
        
        let facets_str = format!("[{}]", facets.join(","));
        params.push(("facets", facets_str.as_str()));

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("search?{}", query_string);
        let response: serde_json::Value = self.make_request(&endpoint).await?;

        let empty_vec = vec![];
        let hits = response["hits"].as_array().unwrap_or(&empty_vec);
        let mut mods = Vec::new();

        for hit in hits {
            match self.convert_modrinth_project_to_mod_info(hit.clone()) {
                Ok(mod_info) => mods.push(mod_info),
                Err(e) => {
                    eprintln!("Failed to convert Modrinth project: {:?}", e);
                    continue;
                }
            }
        }

        Ok(mods)
    }

    async fn get_mod_details(&self, mod_id: &str) -> Result<ModInfo, ModError> {
        let endpoint = format!("project/{}", mod_id);
        let project: serde_json::Value = self.make_request(&endpoint).await?;
        self.convert_modrinth_project_to_mod_info(project)
    }

    async fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModFile>, ModError> {
        let endpoint = format!("project/{}/version", mod_id);
        let versions: Vec<serde_json::Value> = self.make_request(&endpoint).await?;

        let mut files = Vec::new();
        for version in versions {
            match self.convert_modrinth_version_to_mod_file(version) {
                Ok(file) => files.push(file),
                Err(e) => {
                    eprintln!("Failed to convert Modrinth version: {:?}", e);
                    continue;
                }
            }
        }

        Ok(files)
    }

    async fn get_mod_file(&self, _mod_id: &str, file_id: &str) -> Result<ModFile, ModError> {
        let endpoint = format!("version/{}", file_id);
        let version: serde_json::Value = self.make_request(&endpoint).await?;
        self.convert_modrinth_version_to_mod_file(version)
    }

    async fn download_mod_file(&self, file: &ModFile, path: &Path, progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>) -> Result<(), ModError> {
        let response = self.client
            .get(&file.download_url)
            .header("User-Agent", "ChaiLauncher/2.0.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ModError::DownloadFailed(format!("HTTP {}", response.status())));
        }

        let total_size = file.size;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        let mut file_handle = fs::File::create(path).await?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file_handle.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        file_handle.flush().await?;
        Ok(())
    }

    async fn get_featured_mods(
        &self,
        game_version: Option<&str>,
        mod_loader: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ModInfo>, ModError> {
        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        
        let mut params = vec![
            ("limit", limit_str.as_str()),
            ("offset", offset_str.as_str()),
            ("index", "downloads"), // Sort by downloads for most popular
        ];

        let mut facets = vec![];
        
        // Always filter for mods (not modpacks)
        facets.push("[\"project_type:mod\"]".to_string());
        
        if let Some(version) = game_version {
            facets.push(format!("[\"versions:{}\"]", version));
        }
        
        if let Some(loader) = mod_loader {
            facets.push(format!("[\"categories:{}\"]", loader));
        }
        
        let facets_str = format!("[{}]", facets.join(","));
        params.push(("facets", facets_str.as_str()));

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("search?{}", query_string);
        let response: serde_json::Value = self.make_request(&endpoint).await?;

        let empty_vec = vec![];
        let hits = response["hits"].as_array().unwrap_or(&empty_vec);
        let mut mods = Vec::new();

        for hit in hits {
            match self.convert_modrinth_project_to_mod_info(hit.clone()) {
                Ok(mut mod_info) => {
                    mod_info.featured = true; // Mark as featured
                    mods.push(mod_info);
                },
                Err(e) => {
                    eprintln!("Failed to convert Modrinth project: {:?}", e);
                    continue;
                }
            }
        }

        Ok(mods)
    }

    async fn get_categories(&self) -> Result<Vec<String>, ModError> {
        let endpoint = "tag/category";
        let categories: Vec<serde_json::Value> = self.make_request(endpoint).await?;
        
        Ok(categories
            .iter()
            .filter_map(|cat| cat["name"].as_str())
            .map(String::from)
            .collect())
    }

    async fn check_updates(&self, installed_mod: &InstalledMod) -> Result<Option<ModFile>, ModError> {
        let files = self.get_mod_files(&installed_mod.mod_info.id).await?;
        
        // Find the latest file that's newer than the installed one
        let latest = files
            .into_iter()
            .filter(|f| f.date_published > installed_mod.installed_file.date_published)
            .max_by_key(|f| f.date_published);
            
        Ok(latest)
    }
}