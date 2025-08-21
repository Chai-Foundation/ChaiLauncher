use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use anyhow::{Result, Context};
use reqwest::Client;
use sha1::{Sha1, Digest};
use futures::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
#[serde(default, deny_unknown_fields)]
pub struct VersionMetadata {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "time")]
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "sha1", default)]
    pub sha1: Option<String>,
    #[serde(rename = "complianceLevel", default)]
    pub compliance_level: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
#[serde(default, deny_unknown_fields)]
pub struct VersionManifest {
    #[serde(rename = "latest")]
    pub latest: HashMap<String, String>,
    #[serde(rename = "versions")]
    pub versions: Vec<VersionMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDetails {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "time")]
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "arguments", default)]
    pub arguments: Option<GameArguments>,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    #[serde(rename = "assets")]
    pub assets: String,
    #[serde(rename = "complianceLevel", default)]
    pub compliance_level: Option<u8>,
    #[serde(rename = "downloads")]
    pub downloads: GameDownloads,
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    #[serde(rename = "libraries")]
    pub libraries: Vec<Library>,
    #[serde(rename = "logging", default)]
    pub logging: Option<LoggingConfig>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameArguments {
    pub game: Vec<ArgumentValue>,
    pub jvm: Vec<ArgumentValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    Array(Vec<String>),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValueValue,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValueValue {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameDownloads {
    pub client: Download,
    pub client_mappings: Option<Download>,
    pub server: Option<Download>,
    pub server_mappings: Option<Download>,
    pub windows_server: Option<Download>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    pub downloads: Option<LibraryDownloads>,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<ExtractRules>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractRules {
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub client: Option<LoggingClientConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingClientConfig {
    pub argument: String,
    pub file: LoggingFile,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndexContent {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub stage: String,
    pub progress: f64,
    pub current_file: Option<String>,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}

pub struct MinecraftInstaller {
    client: Client,
    game_dir: PathBuf,
}

impl MinecraftInstaller {
    pub fn new(game_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            game_dir,
        }
    }

    pub async fn get_version_manifest(&self) -> Result<VersionManifest> {
        println!("[DEBUG] get_version_manifest called");
        let url = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
        
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.client.get(url).send()
        ).await
            .context("Request timed out after 30 seconds")?
            .context("Failed to fetch version manifest")?;

        println!("[DEBUG] Got response, parsing JSON...");
        let manifest: VersionManifest = response.json().await
            .context("Failed to parse version manifest")?;

        println!("[DEBUG] Successfully parsed VersionManifest with {} versions", manifest.versions.len());
        Ok(manifest)
    }

    pub async fn get_version_details(&self, version_url: &str) -> Result<VersionDetails> {
        println!("[DEBUG] get_version_details called with url: {}", version_url);
        
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.client.get(version_url).send()
        ).await
            .context("Request timed out after 30 seconds")?
            .context("Failed to fetch version details")?;

        println!("[DEBUG] Got version details response, reading text...");
        let text = response.text().await
            .context("Failed to read version details response text")?;

        let details: VersionDetails = serde_json::from_str(&text)
            .map_err(|e| {
                println!("[DEBUG] VersionDetails parse error: {}", e);
                anyhow::anyhow!("Failed to parse version details: {}", e)
            })?;

        println!("[DEBUG] Successfully parsed VersionDetails for id: {}", details.id);
        Ok(details)
    }

    pub async fn install_version<F>(&self, version_id: &str, progress_callback: F) -> Result<()>
    where
        F: Fn(InstallProgress) + Send + Sync,
    {
    println!("[DEBUG] install_version called for version_id: {}", version_id);
        progress_callback(InstallProgress {
            stage: "Fetching version manifest".to_string(),
            progress: 0.0,
            current_file: None,
            bytes_downloaded: 0,
            total_bytes: 0,
        });

        // Get version manifest
        let manifest = self.get_version_manifest().await?;
        
        // Find the specific version
        let version_meta = manifest.versions.iter()
            .find(|v| v.id == version_id)
            .context("Version not found")?;

        progress_callback(InstallProgress {
            stage: "Fetching version details".to_string(),
            progress: 5.0,
            current_file: None,
            bytes_downloaded: 0,
            total_bytes: 0,
        });

        // Get version details
        let version_details = self.get_version_details(&version_meta.url).await?;

        // Create necessary directories
        let versions_dir = self.game_dir.join("versions").join(&version_details.id);
        let libraries_dir = self.game_dir.join("libraries");
        let assets_dir = self.game_dir.join("assets");
        
        fs::create_dir_all(&versions_dir).await
            .context("Failed to create versions directory")?;
        fs::create_dir_all(&libraries_dir).await
            .context("Failed to create libraries directory")?;
        fs::create_dir_all(&assets_dir).await
            .context("Failed to create assets directory")?;

        progress_callback(InstallProgress {
            stage: "Downloading client JAR".to_string(),
            progress: 10.0,
            current_file: Some(format!("{}.jar", version_details.id)),
            bytes_downloaded: 0,
            total_bytes: version_details.downloads.client.size,
        });

        // Download client JAR
        let client_jar_path = versions_dir.join(format!("{}.jar", version_details.id));
        self.download_file(
            &version_details.downloads.client.url,
            &client_jar_path,
            Some(&version_details.downloads.client.sha1),
            |downloaded, total| {
                progress_callback(InstallProgress {
                    stage: "Downloading client JAR".to_string(),
                    progress: 10.0 + (downloaded as f64 / total as f64) * 15.0,
                    current_file: Some(format!("{}.jar", version_details.id)),
                    bytes_downloaded: downloaded,
                    total_bytes: total,
                });
            },
        ).await?;

        // Save version JSON
        let version_json_path = versions_dir.join(format!("{}.json", version_details.id));
        let version_json = serde_json::to_string_pretty(&version_details)
            .context("Failed to serialize version details")?;
        fs::write(&version_json_path, version_json).await
            .context("Failed to write version JSON")?;

        progress_callback(InstallProgress {
            stage: "Downloading libraries".to_string(),
            progress: 25.0,
            current_file: None,
            bytes_downloaded: 0,
            total_bytes: 0,
        });

        // Download libraries
        let total_libraries = version_details.libraries.len();
        for (i, library) in version_details.libraries.iter().enumerate() {
            if self.should_include_library(library) {
                if let Some(downloads) = &library.downloads {
                    // Download main artifact
                    if let Some(artifact) = &downloads.artifact {
                        let library_path = libraries_dir.join(&artifact.path);
                        if let Some(parent) = library_path.parent() {
                            fs::create_dir_all(parent).await
                                .context("Failed to create library directory")?;
                        }

                        progress_callback(InstallProgress {
                            stage: "Downloading libraries".to_string(),
                            progress: 25.0 + (i as f64 / total_libraries as f64) * 40.0,
                            current_file: Some(library.name.clone()),
                            bytes_downloaded: 0,
                            total_bytes: artifact.size,
                        });

                        self.download_file(
                            &artifact.url,
                            &library_path,
                            Some(&artifact.sha1),
                            |_, _| {}, // Skip individual file progress for libraries
                        ).await?;
                    }

                    // Download native libraries
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_key) = self.get_native_key(library) {
                            if let Some(native_artifact) = classifiers.get(&native_key) {
                                let native_path = libraries_dir.join(&native_artifact.path);
                                if let Some(parent) = native_path.parent() {
                                    fs::create_dir_all(parent).await
                                        .context("Failed to create native library directory")?;
                                }

                                self.download_file(
                                    &native_artifact.url,
                                    &native_path,
                                    Some(&native_artifact.sha1),
                                    |_, _| {},
                                ).await?;

                                // Extract native library if needed
                                if library.extract.is_some() {
                                    self.extract_native_library(&native_path, &versions_dir.join("natives")).await?;
                                }
                            }
                        }
                    }
                }
            }
        }

        progress_callback(InstallProgress {
            stage: "Downloading assets".to_string(),
            progress: 65.0,
            current_file: None,
            bytes_downloaded: 0,
            total_bytes: 0,
        });

        // Download assets
        self.download_assets(&version_details, |progress| {
            progress_callback(InstallProgress {
                stage: "Downloading assets".to_string(),
                progress: 65.0 + progress * 30.0,
                current_file: None,
                bytes_downloaded: 0,
                total_bytes: 0,
            });
        }).await?;

        progress_callback(InstallProgress {
            stage: "Installation complete".to_string(),
            progress: 100.0,
            current_file: None,
            bytes_downloaded: 0,
            total_bytes: 0,
        });

        Ok(())
    }

    async fn download_file<F>(
        &self,
        url: &str,
        path: &PathBuf,
        expected_sha1: Option<&str>,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64),
    {
    println!("[DEBUG] download_file called for url: {}", url);
        // Check if file already exists and has correct hash
        if path.exists() {
            if let Some(expected_hash) = expected_sha1 {
                if let Ok(content) = fs::read(path).await {
                    let mut hasher = Sha1::new();
                    hasher.update(&content);
                    let hash = format!("{:x}", hasher.finalize());
                    if hash == expected_hash {
                        progress_callback(content.len() as u64, content.len() as u64);
                        return Ok(()); // File already exists and is valid
                    }
                }
            }
        }

        let response = self.client.get(url).send().await
            .context("Failed to start download")?;
        
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        let mut file = fs::File::create(path).await
            .context("Failed to create file")?;

        let mut hasher = Sha1::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk).await
                .context("Failed to write chunk")?;
            
            hasher.update(&chunk);
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        file.flush().await.context("Failed to flush file")?;

        // Verify hash if provided
        if let Some(expected_hash) = expected_sha1 {
            let hash = format!("{:x}", hasher.finalize());
            if hash != expected_hash {
                fs::remove_file(path).await.ok(); // Clean up invalid file
                return Err(anyhow::anyhow!("File hash mismatch: expected {}, got {}", expected_hash, hash));
            }
        }

        Ok(())
    }

    async fn download_assets<F>(&self, version_details: &VersionDetails, progress_callback: F) -> Result<()>
    where
        F: Fn(f64),
    {
    println!("[DEBUG] download_assets called for version id: {}", version_details.id);
        let assets_dir = self.game_dir.join("assets");
        let indexes_dir = assets_dir.join("indexes");
        let objects_dir = assets_dir.join("objects");
        
        fs::create_dir_all(&indexes_dir).await
            .context("Failed to create assets indexes directory")?;
        fs::create_dir_all(&objects_dir).await
            .context("Failed to create assets objects directory")?;

        // Download asset index
        let asset_index_path = indexes_dir.join(format!("{}.json", version_details.asset_index.id));
        self.download_file(
            &version_details.asset_index.url,
            &asset_index_path,
            Some(&version_details.asset_index.sha1),
            |_, _| {},
        ).await?;

        // Parse asset index
        let asset_index_content = fs::read_to_string(&asset_index_path).await
            .context("Failed to read asset index")?;
        let asset_index: AssetIndexContent = serde_json::from_str(&asset_index_content)
            .context("Failed to parse asset index")?;

        // Download asset objects
        let total_assets = asset_index.objects.len();
        for (i, (_, asset_object)) in asset_index.objects.iter().enumerate() {
            let hash = &asset_object.hash;
            let first_two = &hash[0..2];
            let object_dir = objects_dir.join(first_two);
            let object_path = object_dir.join(hash);

            fs::create_dir_all(&object_dir).await
                .context("Failed to create asset object directory")?;

            if !object_path.exists() {
                let url = format!("https://resources.download.minecraft.net/{}/{}", first_two, hash);
                self.download_file(&url, &object_path, Some(hash), |_, _| {}).await?;
            }

            progress_callback(i as f64 / total_assets as f64);
        }

        Ok(())
    }

    fn should_include_library(&self, library: &Library) -> bool {
        if let Some(rules) = &library.rules {
            for rule in rules {
                if !self.evaluate_rule(rule) {
                    return false;
                }
            }
        }
        true
    }

    fn evaluate_rule(&self, rule: &Rule) -> bool {
        let allow = rule.action == "allow";
        
        if let Some(os_rule) = &rule.os {
            if let Some(os_name) = &os_rule.name {
                let current_os = std::env::consts::OS;
                let matches = match os_name.as_str() {
                    "windows" => current_os == "windows",
                    "osx" => current_os == "macos",
                    "linux" => current_os == "linux",
                    _ => false,
                };
                return matches == allow;
            }
        }

        allow
    }

    fn get_native_key(&self, library: &Library) -> Option<String> {
        if let Some(natives) = &library.natives {
            let os = std::env::consts::OS;
            let key = match os {
                "windows" => "windows",
                "macos" => "osx",
                "linux" => "linux",
                _ => return None,
            };
            natives.get(key).cloned()
        } else {
            None
        }
    }

    async fn extract_native_library(&self, archive_path: &PathBuf, extract_dir: &PathBuf) -> Result<()> {
        use zip::ZipArchive;
        use std::io::Read;

        fs::create_dir_all(extract_dir).await
            .context("Failed to create natives directory")?;

        let file = std::fs::File::open(archive_path)
            .context("Failed to open native library archive")?;
        let mut archive = ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;

        // Collect extraction jobs synchronously
        let mut jobs = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read ZIP entry")?;
            let outpath = extract_dir.join(file.name());
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
}