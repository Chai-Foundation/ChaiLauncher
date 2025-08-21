use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use anyhow::{Result, Context};
use crate::minecraft::MinecraftInstance;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub modpack: Option<String>,
    pub modpack_version: Option<String>,
    pub game_dir: PathBuf,
    pub java_path: Option<String>,
    pub jvm_args: Option<Vec<String>>,
    pub last_played: Option<String>,
    pub total_play_time: u64,
    pub icon: Option<String>,
    pub is_modded: bool,
    pub mods_count: u32,
    pub created_at: String,
    pub size_mb: Option<u64>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub instances: HashMap<String, InstanceMetadata>,
    pub settings: LauncherSettings,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherSettings {
    pub default_java_path: Option<String>,
    pub default_memory: u32,
    pub default_jvm_args: Vec<String>,
    pub instances_dir: PathBuf,
    pub downloads_dir: PathBuf,
    pub theme: String,
    pub auto_update: bool,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        let launcher_dir = get_launcher_dir();
        Self {
            default_java_path: None,
            default_memory: 4096,
            default_jvm_args: vec![
                "-XX:+UnlockExperimentalVMOptions".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-XX:G1NewSizePercent=20".to_string(),
                "-XX:G1ReservePercent=20".to_string(),
                "-XX:MaxGCPauseMillis=50".to_string(),
                "-XX:G1HeapRegionSize=32M".to_string(),
            ],
            instances_dir: launcher_dir.join("instances"),
            downloads_dir: launcher_dir.join("downloads"),
            theme: "dark".to_string(),
            auto_update: true,
        }
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            instances: HashMap::new(),
            settings: LauncherSettings::default(),
            version: "2.0.0".to_string(),
        }
    }
}

pub struct StorageManager {
    config_path: PathBuf,
    config: LauncherConfig,
}

impl StorageManager {
    pub async fn new() -> Result<Self> {
        let config_path = get_config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create config directory")?;
        }

        let config = if config_path.exists() {
            Self::load_config(&config_path).await?
        } else {
            let default_config = LauncherConfig::default();
            Self::save_config(&config_path, &default_config).await?;
            default_config
        };

        // Ensure essential directories exist
        fs::create_dir_all(&config.settings.instances_dir).await
            .context("Failed to create instances directory")?;
        fs::create_dir_all(&config.settings.downloads_dir).await
            .context("Failed to create downloads directory")?;

        Ok(Self { config_path, config })
    }

    async fn load_config(path: &PathBuf) -> Result<LauncherConfig> {
        let content = fs::read_to_string(path).await
            .context("Failed to read config file")?;
        let config: LauncherConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    }

    async fn save_config(path: &PathBuf, config: &LauncherConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)
            .context("Failed to serialize config")?;
        fs::write(path, content).await
            .context("Failed to write config file")?;
        Ok(())
    }

    pub async fn save(&self) -> Result<()> {
        Self::save_config(&self.config_path, &self.config).await
    }

    pub async fn add_instance(&mut self, instance: InstanceMetadata) -> Result<()> {
        // Ensure instance directory exists
        fs::create_dir_all(&instance.game_dir).await
            .context("Failed to create instance directory")?;

        // Calculate instance size
        let size_mb = calculate_directory_size(&instance.game_dir).await.ok();
        
        let mut instance = instance;
        instance.size_mb = size_mb;
        
        self.config.instances.insert(instance.id.clone(), instance);
        self.save().await
    }

    pub async fn remove_instance(&mut self, instance_id: &str) -> Result<Option<InstanceMetadata>> {
        if let Some(instance) = self.config.instances.remove(instance_id) {
            // Optionally remove instance directory
            if instance.game_dir.exists() {
                fs::remove_dir_all(&instance.game_dir).await
                    .context("Failed to remove instance directory")?;
            }
            self.save().await?;
            Ok(Some(instance))
        } else {
            Ok(None)
        }
    }

    pub async fn update_instance(&mut self, instance: InstanceMetadata) -> Result<()> {
        if self.config.instances.contains_key(&instance.id) {
            self.config.instances.insert(instance.id.clone(), instance);
            self.save().await
        } else {
            Err(anyhow::anyhow!("Instance not found: {}", instance.id))
        }
    }

    pub fn get_instance(&self, instance_id: &str) -> Option<&InstanceMetadata> {
        self.config.instances.get(instance_id)
    }

    pub fn get_all_instances(&self) -> Vec<&InstanceMetadata> {
        self.config.instances.values().collect()
    }

    pub fn get_settings(&self) -> &LauncherSettings {
        &self.config.settings
    }

    pub async fn update_settings(&mut self, settings: LauncherSettings) -> Result<()> {
        // Ensure new directories exist
        fs::create_dir_all(&settings.instances_dir).await
            .context("Failed to create new instances directory")?;
        fs::create_dir_all(&settings.downloads_dir).await
            .context("Failed to create new downloads directory")?;

        self.config.settings = settings;
        self.save().await
    }

    pub async fn refresh_instance_sizes(&mut self) -> Result<()> {
        for instance in self.config.instances.values_mut() {
            if instance.game_dir.exists() {
                instance.size_mb = calculate_directory_size(&instance.game_dir).await.ok();
            }
        }
        self.save().await
    }

    pub async fn backup_instance(&self, instance_id: &str, backup_path: &PathBuf) -> Result<()> {
        if let Some(instance) = self.get_instance(instance_id) {
            backup_directory(&instance.game_dir, backup_path).await
        } else {
            Err(anyhow::anyhow!("Instance not found: {}", instance_id))
        }
    }

    pub async fn restore_instance(&mut self, backup_path: &PathBuf, instance_id: &str) -> Result<()> {
        if let Some(instance) = self.config.instances.get(instance_id) {
            let instance_dir = &instance.game_dir;
            
            // Remove existing instance directory
            if instance_dir.exists() {
                fs::remove_dir_all(instance_dir).await
                    .context("Failed to remove existing instance")?;
            }

            // Restore from backup
            restore_directory(backup_path, instance_dir).await
        } else {
            Err(anyhow::anyhow!("Instance not found: {}", instance_id))
        }
    }
}

// Utility functions

pub fn get_launcher_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ChaiLauncher")
}

pub fn get_config_path() -> PathBuf {
    get_launcher_dir().join("config.json")
}

async fn calculate_directory_size(path: &PathBuf) -> Result<u64> {
    let mut total_size = 0u64;
    let mut stack = vec![path.clone()];

    while let Some(current_path) = stack.pop() {
        if current_path.is_dir() {
            let mut entries = fs::read_dir(&current_path).await
                .context("Failed to read directory")?;
                
            while let Some(entry) = entries.next_entry().await
                .context("Failed to get directory entry")? {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                } else {
                    let metadata = entry.metadata().await
                        .context("Failed to get file metadata")?;
                    total_size += metadata.len();
                }
            }
        }
    }

    Ok(total_size / 1024 / 1024) // Convert to MB
}

async fn backup_directory(source: &PathBuf, destination: &PathBuf) -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    // Create backup directory
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).await
            .context("Failed to create backup directory")?;
    }

    // Use system commands for efficient copying
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("robocopy")
            .arg(source)
            .arg(destination)
            .arg("/E") // Copy subdirectories including empty ones
            .arg("/MT") // Multi-threaded copying
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .context("Failed to execute robocopy")?;

        // Robocopy returns different exit codes, 0-7 are success
        if !output.success() && output.code().unwrap_or(8) > 7 {
            return Err(anyhow::anyhow!("Backup failed"));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("cp")
            .arg("-r")
            .arg(source)
            .arg(destination)
            .status()
            .await
            .context("Failed to execute cp")?;

        if !output.success() {
            return Err(anyhow::anyhow!("Backup failed"));
        }
    }

    Ok(())
}

async fn restore_directory(source: &PathBuf, destination: &PathBuf) -> Result<()> {
    backup_directory(source, destination).await
}

// Convert between storage types and API types
impl From<InstanceMetadata> for MinecraftInstance {
    fn from(metadata: InstanceMetadata) -> Self {
        Self {
            id: metadata.id,
            name: metadata.name,
            version: metadata.version,
            modpack: metadata.modpack,
            modpack_version: metadata.modpack_version,
            game_dir: metadata.game_dir,
            java_path: metadata.java_path,
            jvm_args: metadata.jvm_args,
            last_played: metadata.last_played,
            total_play_time: metadata.total_play_time,
            icon: metadata.icon,
            is_modded: metadata.is_modded,
            mods_count: metadata.mods_count,
            is_external: Some(false),
            external_launcher: None,
        }
    }
}

impl From<MinecraftInstance> for InstanceMetadata {
    fn from(instance: MinecraftInstance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            version: instance.version,
            modpack: instance.modpack,
            modpack_version: instance.modpack_version,
            game_dir: instance.game_dir,
            java_path: instance.java_path,
            jvm_args: instance.jvm_args,
            last_played: instance.last_played,
            total_play_time: instance.total_play_time,
            icon: instance.icon,
            is_modded: instance.is_modded,
            mods_count: instance.mods_count,
            created_at: chrono::Utc::now().to_rfc3339(),
            size_mb: None,
            description: None,
            tags: Vec::new(),
        }
    }
}