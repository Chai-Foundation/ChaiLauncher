use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use anyhow::{Result, Context};
use crate::minecraft::MinecraftInstance;
use crate::docker::types::{DockerConnection, ServerInstance};

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
    pub docker_connections: HashMap<String, DockerConnection>,
    pub servers: HashMap<String, ServerInstance>,
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
    pub background_image: Option<String>,
    pub color_scheme: String,
    pub primary_base_color: Option<String>,
    pub secondary_base_color: Option<String>,
    pub auto_update: bool,
    pub auth_token: Option<String>,
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
            background_image: None,
            color_scheme: "primary".to_string(),
            primary_base_color: Some("#78716c".to_string()),
            secondary_base_color: Some("#d97706".to_string()),
            auto_update: true,
            auth_token: None,
        }
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            instances: HashMap::new(),
            docker_connections: HashMap::new(),
            servers: HashMap::new(),
            settings: LauncherSettings::default(),
            version: "2.1.0".to_string(),
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

        // If we just migrated (version is 2.1.0 and we loaded from existing file), save the migrated config
        if config_path.exists() && config.version == "2.1.0" {
            // Check if this is a fresh migration by looking for the docker_connections field in the raw file
            let raw_content = fs::read_to_string(&config_path).await?;
            if !raw_content.contains("docker_connections") {
                Self::save_config(&config_path, &config).await?;
                println!("💾 Saved migrated config to disk");
            }
        }

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
        
        // First, try to parse as the new format
        match serde_json::from_str::<LauncherConfig>(&content) {
            Ok(config) => Ok(config),
            Err(_) => {
                // If that fails, try to parse as the old format and migrate
                let old_config: serde_json::Value = serde_json::from_str(&content)
                    .context("Failed to parse config file as JSON")?;
                
                Self::migrate_config(old_config)
            }
        }
    }

    fn migrate_config(old_config: serde_json::Value) -> Result<LauncherConfig> {
        // Create a new config with the old data + new fields
        let mut new_config = LauncherConfig::default();
        
        // Migrate instances if they exist
        if let Some(instances) = old_config.get("instances") {
            if let Ok(instances_map) = serde_json::from_value::<HashMap<String, InstanceMetadata>>(instances.clone()) {
                println!("📦 Migrated {} instances", instances_map.len());
                new_config.instances = instances_map;
            }
        }
        
        // Migrate settings if they exist
        if let Some(settings) = old_config.get("settings") {
            if let Ok(settings_struct) = serde_json::from_value::<LauncherSettings>(settings.clone()) {
                new_config.settings = settings_struct;
                println!("⚙️  Migrated launcher settings");
            }
        }
        
        // Version is now "2.1.0" to indicate migration
        new_config.version = "2.1.0".to_string();
        
        println!("✅ Successfully migrated config from v2.0.0 to v2.1.0");
        println!("   Added support for Docker connections and server management");
        
        Ok(new_config)
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
        // Validate game directory path
        if instance.game_dir.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("Instance game directory cannot be empty"));
        }
        
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
        // Validate game directory path
        if instance.game_dir.as_os_str().is_empty() {
            return Err(anyhow::anyhow!("Instance game directory cannot be empty"));
        }
        
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
        self.config.instances.values()
            .filter(|instance| {
                if instance.game_dir.as_os_str().is_empty() {
                    eprintln!("Warning: Found instance '{}' with empty game_dir, excluding from list", instance.name);
                    false
                } else {
                    true
                }
            })
            .collect()
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

    // Docker connection management
    pub async fn add_docker_connection(&mut self, connection: DockerConnection) -> Result<()> {
        self.config.docker_connections.insert(connection.id.clone(), connection);
        self.save().await
    }

    pub async fn update_docker_connection(&mut self, connection: DockerConnection) -> Result<()> {
        if self.config.docker_connections.contains_key(&connection.id) {
            self.config.docker_connections.insert(connection.id.clone(), connection);
            self.save().await
        } else {
            Err(anyhow::anyhow!("Docker connection not found: {}", connection.id))
        }
    }

    pub async fn remove_docker_connection(&mut self, connection_id: &str) -> Result<()> {
        if self.config.docker_connections.remove(connection_id).is_some() {
            // Also remove any servers using this connection
            self.config.servers.retain(|_, server| server.docker_connection_id != connection_id);
            self.save().await
        } else {
            Err(anyhow::anyhow!("Docker connection not found: {}", connection_id))
        }
    }

    pub fn get_docker_connection(&self, connection_id: &str) -> Option<&DockerConnection> {
        self.config.docker_connections.get(connection_id)
    }

    pub fn get_docker_connections(&self) -> Vec<&DockerConnection> {
        self.config.docker_connections.values().collect()
    }

    // Server management
    pub async fn add_server(&mut self, server: ServerInstance) -> Result<()> {
        self.config.servers.insert(server.id.clone(), server);
        self.save().await
    }

    pub async fn update_server(&mut self, server: ServerInstance) -> Result<()> {
        if self.config.servers.contains_key(&server.id) {
            self.config.servers.insert(server.id.clone(), server);
            self.save().await
        } else {
            Err(anyhow::anyhow!("Server not found: {}", server.id))
        }
    }

    pub async fn remove_server(&mut self, server_id: &str) -> Result<()> {
        if self.config.servers.remove(server_id).is_some() {
            self.save().await
        } else {
            Err(anyhow::anyhow!("Server not found: {}", server_id))
        }
    }

    pub fn get_server(&self, server_id: &str) -> Option<&ServerInstance> {
        self.config.servers.get(server_id)
    }

    pub fn get_servers(&self) -> Vec<&ServerInstance> {
        self.config.servers.values().collect()
    }

    pub fn get_servers_for_instance(&self, instance_id: &str) -> Vec<&ServerInstance> {
        self.config.servers.values()
            .filter(|s| s.minecraft_instance_id == instance_id)
            .collect()
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

// Conversion is now handled in minecraft::commands module

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