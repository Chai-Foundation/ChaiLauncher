use super::{ExternalInstance, LauncherDetector, LauncherLauncher};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use serde_json::Value;

pub struct ModrinthDetector;

#[async_trait]
impl LauncherDetector for ModrinthDetector {
    fn name(&self) -> &'static str {
        "modrinth"
    }
    
    async fn is_installed(&self) -> bool {
        self.get_config_path().await.is_ok()
    }
    
    async fn get_config_path(&self) -> Result<PathBuf, String> {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| "Could not get APPDATA environment variable")?;
            let path = PathBuf::from(appdata).join("com.modrinth.theseus");
            if path.exists() {
                Ok(path)
            } else {
                Err("Modrinth App config directory not found".to_string())
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "Could not get HOME environment variable")?;
            let path = PathBuf::from(home).join("Library/Application Support/com.modrinth.theseus");
            if path.exists() {
                Ok(path)
            } else {
                Err("Modrinth App config directory not found".to_string())
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "Could not get HOME environment variable")?;
            
            // Check XDG_CONFIG_HOME first
            if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
                let path = PathBuf::from(xdg_config).join("com.modrinth.theseus");
                if path.exists() {
                    return Ok(path);
                }
            }
            
            let path = PathBuf::from(home).join(".config/com.modrinth.theseus");
            if path.exists() {
                Ok(path)
            } else {
                Err("Modrinth App config directory not found".to_string())
            }
        }
    }
    
    async fn detect_instances(&self) -> Result<Vec<ExternalInstance>, String> {
        let config_path = self.get_config_path().await?;
        let profiles_file = config_path.join("profiles.json");
        
        if !profiles_file.exists() {
            return Ok(Vec::new());
        }
        
        let profiles_content = fs::read_to_string(&profiles_file)
            .await
            .map_err(|e| format!("Failed to read profiles.json: {}", e))?;
        
        let profiles: Value = serde_json::from_str(&profiles_content)
            .map_err(|e| format!("Failed to parse profiles.json: {}", e))?;
        
        let mut instances = Vec::new();
        
        if let Some(profiles_obj) = profiles.as_object() {
            for (profile_id, profile_data) in profiles_obj {
                match self.parse_profile(profile_id, profile_data, &config_path).await {
                    Ok(instance) => instances.push(instance),
                    Err(e) => {
                        eprintln!("Failed to parse Modrinth profile {}: {}", profile_id, e);
                    }
                }
            }
        }
        
        Ok(instances)
    }
    
    async fn get_executable_path(&self) -> Result<Option<PathBuf>, String> {
        #[cfg(target_os = "windows")]
        {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
            let common_paths = [
                format!("{}\\AppData\\Local\\Programs\\Modrinth App\\Modrinth App.exe", userprofile),
                "C:\\Program Files\\Modrinth App\\Modrinth App.exe".to_string(),
                "C:\\Program Files (x86)\\Modrinth App\\Modrinth App.exe".to_string(),
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
            let path = PathBuf::from("/Applications/Modrinth App.app/Contents/MacOS/Modrinth App");
            if path.exists() {
                return Ok(Some(path));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let common_paths = [
                "/usr/bin/modrinth-app",
                "/usr/local/bin/modrinth-app",
                "/opt/modrinth-app/modrinth-app",
                "/usr/bin/com.modrinth.theseus",
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

impl ModrinthDetector {
    async fn parse_profile(
        &self,
        profile_id: &str,
        profile_data: &Value,
        config_path: &PathBuf,
    ) -> Result<ExternalInstance, String> {
        let name = profile_data["name"]
            .as_str()
            .unwrap_or(profile_id)
            .to_string();
        
        let version = profile_data["game_version"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        
        let modpack = profile_data["metadata"]["linked_data"]["project_id"]
            .as_str()
            .or_else(|| profile_data["metadata"]["name"].as_str())
            .map(|s| s.to_string());
        
        let modpack_version = profile_data["metadata"]["linked_data"]["version_id"]
            .as_str()
            .or_else(|| profile_data["metadata"]["version"].as_str())
            .map(|s| s.to_string());
        
        let default_path = format!("profiles/{}", profile_id);
        let path_str = profile_data["path"]
            .as_str()
            .unwrap_or(&default_path);
        
        let path = if PathBuf::from(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            config_path.join(path_str)
        };
        
        let java_path = profile_data["java"]["path"]
            .as_str()
            .map(|s| s.to_string());
        
        let jvm_args = profile_data["java"]["extra_arguments"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            });
        
        let last_played = profile_data["last_played"]
            .as_str()
            .map(|s| s.to_string());
        
        let total_play_time = profile_data["recent_time_played"]
            .as_u64()
            .unwrap_or(0);
        
        let icon = profile_data["icon"]
            .as_str()
            .map(|s| s.to_string());
        
        // Check if it's modded
        let is_modded = modpack.is_some() || 
            profile_data["mods"].as_array().map_or(false, |arr| !arr.is_empty());
        
        let mods_count = profile_data["mods"]
            .as_array()
            .map_or(0, |arr| arr.len()) as u32;
        
        Ok(ExternalInstance {
            id: format!("modrinth-{}", profile_id),
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
            launcher_type: "modrinth".to_string(),
        })
    }
}

#[async_trait]
impl LauncherLauncher for ModrinthDetector {
    async fn can_launch(&self) -> bool {
        self.get_executable_path().await.unwrap_or(None).is_some()
    }
    
    async fn launch_instance(&self, instance_id: &str, _instance_path: &str) -> Result<(), String> {
        let executable = self.get_executable_path().await?
            .ok_or("Modrinth App executable not found")?;
        
        let actual_id = instance_id.strip_prefix("modrinth-").unwrap_or(instance_id);
        
        // Modrinth App can be launched with a profile ID as an argument
        tokio::process::Command::new(executable)
            .args(&["--launch", actual_id])
            .spawn()
            .map_err(|e| format!("Failed to launch Modrinth App instance: {}", e))?;
        
        Ok(())
    }
}