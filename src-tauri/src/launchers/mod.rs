use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod gdlauncher;
pub mod multimc;
pub mod prism;
pub mod modrinth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalInstance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub modpack: Option<String>,
    pub modpack_version: Option<String>,
    pub path: PathBuf,
    pub java_path: Option<String>,
    pub jvm_args: Option<Vec<String>>,
    pub last_played: Option<String>,
    pub total_play_time: u64,
    pub icon: Option<String>,
    pub is_modded: bool,
    pub mods_count: u32,
    pub launcher_type: String,
}

#[async_trait]
pub trait LauncherDetector {
    /// Get the name of this launcher
    fn name(&self) -> &'static str;
    
    /// Check if this launcher is installed on the system
    async fn is_installed(&self) -> bool;
    
    /// Get the configuration/data directory for this launcher
    async fn get_config_path(&self) -> Result<PathBuf, String>;
    
    /// Detect all instances from this launcher
    async fn detect_instances(&self) -> Result<Vec<ExternalInstance>, String>;
    
    /// Get the executable path for this launcher
    async fn get_executable_path(&self) -> Result<Option<PathBuf>, String>;
}

#[async_trait]
pub trait LauncherLauncher {
    /// Launch an instance using this launcher's native method
    async fn launch_instance(&self, instance_id: &str, instance_path: &str) -> Result<(), String>;
    
    /// Check if we can launch instances for this launcher
    async fn can_launch(&self) -> bool;
}

pub struct LauncherManager {
    detectors: Vec<Box<dyn LauncherDetector + Send + Sync>>,
}

impl LauncherManager {
    pub fn new() -> Self {
        let detectors: Vec<Box<dyn LauncherDetector + Send + Sync>> = vec![
            Box::new(gdlauncher::GDLauncherDetector),
            Box::new(multimc::MultiMCDetector),
            Box::new(prism::PrismDetector),
            Box::new(modrinth::ModrinthDetector),
        ];
        
        Self { detectors }
    }
    
    pub async fn detect_all_instances(&self) -> Result<Vec<ExternalInstance>, String> {
        let mut all_instances = Vec::new();
        
        for detector in &self.detectors {
            if detector.is_installed().await {
                match detector.detect_instances().await {
                    Ok(instances) => {
                        // Filter out instances with invalid paths
                        let valid_instances: Vec<ExternalInstance> = instances.into_iter()
                            .filter(|instance| {
                                if instance.path.as_os_str().is_empty() {
                                    eprintln!("Warning: Found {} instance '{}' with empty path, excluding from list", 
                                        detector.name(), instance.name);
                                    false
                                } else {
                                    true
                                }
                            })
                            .collect();
                        all_instances.extend(valid_instances);
                    }
                    Err(e) => {
                        eprintln!("Failed to detect instances from {}: {}", detector.name(), e);
                    }
                }
            }
        }
        
        Ok(all_instances)
    }
    
    pub async fn get_launcher_for_instance(&self, launcher_type: &str) -> Option<&Box<dyn LauncherDetector + Send + Sync>> {
        for detector in &self.detectors {
            if detector.name() == launcher_type {
                return Some(detector);
            }
        }
        None
    }
}