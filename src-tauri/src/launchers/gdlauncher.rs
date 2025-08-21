use super::{ExternalInstance, LauncherDetector, LauncherLauncher};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use serde_json::Value;

pub struct GDLauncherDetector;

#[async_trait]
impl LauncherDetector for GDLauncherDetector {
    fn name(&self) -> &'static str {
        "gdlauncher"
    }
    
    async fn is_installed(&self) -> bool {
        self.get_config_path().await.is_ok()
    }
    
    async fn get_config_path(&self) -> Result<PathBuf, String> {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| "Could not get APPDATA environment variable")?;
            let path = PathBuf::from(appdata).join("gdlauncher_next");
            if path.exists() {
                Ok(path)
            } else {
                Err("GDLauncher config directory not found".to_string())
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "Could not get HOME environment variable")?;
            let path = PathBuf::from(home).join("Library/Application Support/gdlauncher_next");
            if path.exists() {
                Ok(path)
            } else {
                Err("GDLauncher config directory not found".to_string())
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "Could not get HOME environment variable")?;
            let path = PathBuf::from(home).join(".local/share/gdlauncher_next");
            if path.exists() {
                Ok(path)
            } else {
                Err("GDLauncher config directory not found".to_string())
            }
        }
    }
    
    async fn detect_instances(&self) -> Result<Vec<ExternalInstance>, String> {
        let config_path = self.get_config_path().await?;
        let instances_path = config_path.join("instances");
        
        if !instances_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut instances = Vec::new();
        let mut entries = fs::read_dir(&instances_path)
            .await
            .map_err(|e| format!("Failed to read GDLauncher instances directory: {}", e))?;
        
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map_or(false, |ft| ft.is_dir()) {
                let instance_dir = entry.path();
                let config_file = instance_dir.join("config.json");
                
                if config_file.exists() {
                    match self.parse_instance_config(&config_file).await {
                        Ok(instance) => instances.push(instance),
                        Err(e) => {
                            eprintln!("Failed to parse GDLauncher instance at {:?}: {}", config_file, e);
                        }
                    }
                }
            }
        }
        
        Ok(instances)
    }
    
    async fn get_executable_path(&self) -> Result<Option<PathBuf>, String> {
        #[cfg(target_os = "windows")]
        {
            let common_paths = [
                std::env::var("USERPROFILE").unwrap_or_default() + "\\AppData\\Local\\Programs\\GDLauncher\\GDLauncher.exe",
                "C:\\Program Files\\GDLauncher\\GDLauncher.exe".to_string(),
                "C:\\Program Files (x86)\\GDLauncher\\GDLauncher.exe".to_string(),
            ];
            
            for path_str in &common_paths {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(Some(path));
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Applications/GDLauncher.app/Contents/MacOS/GDLauncher");
            if path.exists() {
                return Ok(Some(path));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let common_paths = [
                "/usr/bin/gdlauncher",
                "/usr/local/bin/gdlauncher",
                "/opt/gdlauncher/gdlauncher",
            ];
            
            for path_str in &common_paths {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(Some(path));
                }
            }
        }
        
        Ok(None)
    }
}

impl GDLauncherDetector {
    async fn parse_instance_config(&self, config_path: &PathBuf) -> Result<ExternalInstance, String> {
        let content = fs::read_to_string(config_path)
            .await
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        let uuid = config["uuid"]
            .as_str()
            .ok_or("Missing uuid field")?
            .to_string();
        
        let name = config["name"]
            .as_str()
            .ok_or("Missing name field")?
            .to_string();
        
        let version = config["loader"]["mcVersion"]
            .as_str()
            .or_else(|| config["version"].as_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let modpack = config["loader"]["projectID"]
            .as_str()
            .or_else(|| config["modpack"]["name"].as_str())
            .map(|s| s.to_string());
        
        let modpack_version = config["loader"]["fileID"]
            .as_str()
            .or_else(|| config["modpack"]["version"].as_str())
            .map(|s| s.to_string());
        
        let path = config_path
            .parent()
            .ok_or("Invalid config path")?
            .to_path_buf();
        
        let java_path = config["settings"]["java"]["path"]
            .as_str()
            .map(|s| s.to_string());
        
        let jvm_args = config["settings"]["java"]["args"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });
        
        let last_played = config["lastPlayed"]
            .as_str()
            .map(|s| s.to_string());
        
        let total_play_time = config["totalPlayTime"]
            .as_u64()
            .unwrap_or(0);
        
        let icon = config["icon"]
            .as_str()
            .map(|s| s.to_string());
        
        let is_modded = modpack.is_some() || 
            config["mods"].as_array().map_or(false, |arr| !arr.is_empty());
        
        let mods_count = config["mods"]
            .as_array()
            .map_or(0, |arr| arr.len()) as u32;
        
        Ok(ExternalInstance {
            id: format!("gdl-{}", uuid),
            name,
            version,
            modpack,
            modpack_version,
            path,
            java_path,
            jvm_args,
            last_played,
            total_play_time,
            icon,
            is_modded,
            mods_count,
            launcher_type: "gdlauncher".to_string(),
        })
    }
}

#[async_trait]
impl LauncherLauncher for GDLauncherDetector {
    async fn can_launch(&self) -> bool {
        self.get_executable_path().await.unwrap_or(None).is_some()
    }
    
    async fn launch_instance(&self, instance_id: &str, _instance_path: &str) -> Result<(), String> {
        let executable = self.get_executable_path().await?
            .ok_or("GDLauncher executable not found")?;
        
        let actual_id = instance_id.strip_prefix("gdl-").unwrap_or(instance_id);
        
        tokio::process::Command::new(executable)
            .args(&["launch", actual_id])
            .spawn()
            .map_err(|e| format!("Failed to launch GDLauncher instance: {}", e))?;
        
        Ok(())
    }
}