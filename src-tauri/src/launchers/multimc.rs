use super::{ExternalInstance, LauncherDetector, LauncherLauncher};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

pub struct MultiMCDetector;

#[async_trait]
impl LauncherDetector for MultiMCDetector {
    fn name(&self) -> &'static str {
        "multimc"
    }
    
    async fn is_installed(&self) -> bool {
        self.get_config_path().await.is_ok()
    }
    
    async fn get_config_path(&self) -> Result<PathBuf, String> {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| "Could not get APPDATA environment variable")?;
            let path = PathBuf::from(appdata).join("MultiMC");
            if path.exists() {
                return Ok(path);
            }
            
            // Check for portable installation
            let exe_paths = [
                "C:\\MultiMC\\MultiMC.exe",
                "C:\\Program Files\\MultiMC\\MultiMC.exe",
                "C:\\Program Files (x86)\\MultiMC\\MultiMC.exe",
            ];
            
            for exe_path in &exe_paths {
                let exe = PathBuf::from(exe_path);
                if exe.exists() {
                    if let Some(parent) = exe.parent() {
                        return Ok(parent.to_path_buf());
                    }
                }
            }
            
            Err("MultiMC installation not found".to_string())
        }
        
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "Could not get HOME environment variable")?;
            let path = PathBuf::from(home).join("Library/Application Support/MultiMC");
            if path.exists() {
                Ok(path)
            } else {
                Err("MultiMC config directory not found".to_string())
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| "Could not get HOME environment variable")?;
            let path = PathBuf::from(home).join(".local/share/multimc");
            if path.exists() {
                return Ok(path);
            }
            
            // Check for portable installation
            let portable_path = PathBuf::from("/opt/multimc");
            if portable_path.exists() {
                Ok(portable_path)
            } else {
                Err("MultiMC installation not found".to_string())
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
            .map_err(|e| format!("Failed to read MultiMC instances directory: {}", e))?;
        
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.map_or(false, |ft| ft.is_dir()) {
                let instance_dir = entry.path();
                let instance_cfg = instance_dir.join("instance.cfg");
                let mmc_pack = instance_dir.join("mmc-pack.json");
                
                if instance_cfg.exists() {
                    match self.parse_instance_config(&instance_dir, &instance_cfg, &mmc_pack).await {
                        Ok(instance) => instances.push(instance),
                        Err(e) => {
                            eprintln!("Failed to parse MultiMC instance at {:?}: {}", instance_dir, e);
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
                "C:\\MultiMC\\MultiMC.exe",
                "C:\\Program Files\\MultiMC\\MultiMC.exe",
                "C:\\Program Files (x86)\\MultiMC\\MultiMC.exe",
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
            let path = PathBuf::from("/Applications/MultiMC.app/Contents/MacOS/MultiMC");
            if path.exists() {
                return Ok(Some(path));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            let common_paths = [
                "/usr/bin/multimc",
                "/usr/local/bin/multimc",
                "/opt/multimc/MultiMC",
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

impl MultiMCDetector {
    async fn parse_instance_config(
        &self,
        instance_dir: &PathBuf,
        instance_cfg: &PathBuf,
        mmc_pack: &PathBuf,
    ) -> Result<ExternalInstance, String> {
        let cfg_content = fs::read_to_string(instance_cfg)
            .await
            .map_err(|e| format!("Failed to read instance.cfg: {}", e))?;
        
        let mut name = instance_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let mut version = "Unknown".to_string();
        let mut icon: Option<String> = None;
        let mut java_path: Option<String> = None;
        let mut jvm_args: Option<Vec<String>> = None;
        let mut last_played: Option<String> = None;
        let mut total_play_time = 0u64;
        
        // Parse instance.cfg (INI-like format)
        for line in cfg_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "name" => name = value.trim().to_string(),
                    "IntendedVersion" => version = value.trim().to_string(),
                    "iconKey" => icon = Some(value.trim().to_string()),
                    "JavaPath" => java_path = Some(value.trim().to_string()),
                    "JvmArgs" => {
                        jvm_args = Some(
                            value.trim()
                                .split_whitespace()
                                .map(|s| s.to_string())
                                .collect()
                        );
                    },
                    "lastLaunchTime" => last_played = Some(value.trim().to_string()),
                    "totalTimePlayed" => {
                        if let Ok(time) = value.trim().parse::<u64>() {
                            total_play_time = time;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // Check for modpack information
        let mut modpack: Option<String> = None;
        let mut modpack_version: Option<String> = None;
        let mut is_modded = false;
        let mut mods_count = 0u32;
        
        if mmc_pack.exists() {
            match fs::read_to_string(mmc_pack).await {
                Ok(pack_content) => {
                    if let Ok(pack_json) = serde_json::from_str::<serde_json::Value>(&pack_content) {
                        modpack = pack_json["name"].as_str().map(|s| s.to_string());
                        modpack_version = pack_json["version"].as_str().map(|s| s.to_string());
                        is_modded = true;
                    }
                }
                Err(_) => {}
            }
        }
        
        // Check for mods folder
        let mods_dir = instance_dir.join(".minecraft").join("mods");
        if mods_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&mods_dir).await {
                let mut count = 0u32;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "jar" {
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    is_modded = true;
                    mods_count = count;
                }
            }
        }
        
        let id = instance_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(ExternalInstance {
            id: format!("multimc-{}", id),
            name,
            version,
            modpack,
            modpack_version,
            path: instance_dir.clone(),
            java_path,
            jvm_args,
            last_played,
            total_play_time,
            icon,
            is_modded,
            mods_count,
            launcher_type: "multimc".to_string(),
        })
    }
}

#[async_trait]
impl LauncherLauncher for MultiMCDetector {
    async fn can_launch(&self) -> bool {
        self.get_executable_path().await.unwrap_or(None).is_some()
    }
    
    async fn launch_instance(&self, instance_id: &str, _instance_path: &str) -> Result<(), String> {
        let executable = self.get_executable_path().await?
            .ok_or("MultiMC executable not found")?;
        
        let actual_id = instance_id.strip_prefix("multimc-").unwrap_or(instance_id);
        
        tokio::process::Command::new(executable)
            .args(&["--launch", actual_id])
            .spawn()
            .map_err(|e| format!("Failed to launch MultiMC instance: {}", e))?;
        
        Ok(())
    }
}