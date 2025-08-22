use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod legacy;
pub mod modern;
pub mod versions;
pub mod commands;

/// Common structures shared across all Minecraft versions
#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftInstance {
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
    pub is_external: Option<bool>,
    pub external_launcher: Option<String>,
}

/// Launch configuration for Minecraft
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    pub java_path: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub classpath: String,
    pub main_class: String,
    pub working_directory: PathBuf,
    pub environment_vars: HashMap<String, String>,
}

/// Authentication information
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String, // "msa" or "legacy"
}

impl Default for AuthInfo {
    fn default() -> Self {
        Self {
            username: "Player".to_string(),
            uuid: "12345678-90ab-cdef-1234-567890abcdef".to_string(),
            access_token: "123456".to_string(),
            user_type: "legacy".to_string(),
        }
    }
}

/// Launch result information
#[derive(Debug)]
pub struct LaunchResult {
    pub process_id: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// Main trait that all Minecraft version handlers must implement
pub trait MinecraftLauncher {
    /// Get the version range this launcher supports (e.g., "1.0-1.6", "1.7-1.12", "1.13+")
    fn supported_versions(&self) -> &str;
    
    /// Check if this launcher can handle the given version
    fn can_handle_version(&self, version: &str) -> bool;
    
    /// Get the required Java version for this Minecraft version
    fn required_java_version(&self, version: &str) -> u32;
    
    /// Validate that the instance has all required files
    async fn validate_instance(&self, instance: &MinecraftInstance) -> Result<(), String>;
    
    /// Build the classpath for this version
    async fn build_classpath(&self, instance: &MinecraftInstance) -> Result<String, String>;
    
    /// Get game arguments for this version
    fn build_game_arguments(&self, instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String>;
    
    /// Get JVM arguments specific to this version
    fn build_jvm_arguments(&self, instance: &MinecraftInstance, memory: u32) -> Vec<String>;
    
    /// Get the main class for this version
    fn get_main_class(&self, instance: &MinecraftInstance) -> Result<String, String>;
    
    /// Extract native libraries if needed
    async fn extract_natives(&self, instance: &MinecraftInstance) -> Result<(), String>;
    
    /// Launch the Minecraft instance
    async fn launch(&self, instance: &MinecraftInstance, auth: &AuthInfo, memory: u32) -> Result<LaunchResult, String> {
        // Validate instance
        self.validate_instance(instance).await?;
        
        // Extract natives
        self.extract_natives(instance).await?;
        
        // Get Java path
        let java_version = self.required_java_version(&instance.version);
        let java_path = crate::minecraft::versions::get_java_for_version(java_version).await?;
        
        // Build launch configuration
        let config = LaunchConfig {
            java_path,
            jvm_args: self.build_jvm_arguments(instance, memory),
            game_args: self.build_game_arguments(instance, auth),
            classpath: self.build_classpath(instance).await?,
            main_class: self.get_main_class(instance)?,
            working_directory: instance.game_dir.clone(),
            environment_vars: HashMap::new(),
        };
        
        // Launch the process
        self.launch_process(&config).await
    }
    
    /// Internal method to launch the actual process
    async fn launch_process(&self, config: &LaunchConfig) -> Result<LaunchResult, String> {
        use std::process::Command;
        
        let mut args = config.jvm_args.clone();
        args.push("-cp".to_string());
        args.push(config.classpath.clone());
        args.push(config.main_class.clone());
        args.extend(config.game_args.clone());
        
        println!("Launching: {} {}", config.java_path, args.join(" "));
        println!("Working directory: {}", config.working_directory.display());
        
        match Command::new(&config.java_path)
            .args(&args)
            .current_dir(&config.working_directory)
            .spawn()
        {
            Ok(child) => Ok(LaunchResult {
                process_id: child.id(),
                success: true,
                error: None,
            }),
            Err(e) => Err(format!("Failed to launch Minecraft: {}", e)),
        }
    }
}

/// Launcher enum to handle different version types
pub enum Launcher {
    Legacy(legacy::LegacyLauncher),
    Modern(modern::ModernLauncher),
}

impl MinecraftLauncher for Launcher {
    fn supported_versions(&self) -> &str {
        match self {
            Launcher::Legacy(l) => l.supported_versions(),
            Launcher::Modern(l) => l.supported_versions(),
        }
    }
    
    fn can_handle_version(&self, version: &str) -> bool {
        match self {
            Launcher::Legacy(l) => l.can_handle_version(version),
            Launcher::Modern(l) => l.can_handle_version(version),
        }
    }
    
    fn required_java_version(&self, version: &str) -> u32 {
        match self {
            Launcher::Legacy(l) => l.required_java_version(version),
            Launcher::Modern(l) => l.required_java_version(version),
        }
    }
    
    async fn validate_instance(&self, instance: &MinecraftInstance) -> Result<(), String> {
        match self {
            Launcher::Legacy(l) => l.validate_instance(instance).await,
            Launcher::Modern(l) => l.validate_instance(instance).await,
        }
    }
    
    async fn build_classpath(&self, instance: &MinecraftInstance) -> Result<String, String> {
        match self {
            Launcher::Legacy(l) => l.build_classpath(instance).await,
            Launcher::Modern(l) => l.build_classpath(instance).await,
        }
    }
    
    fn build_game_arguments(&self, instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String> {
        match self {
            Launcher::Legacy(l) => l.build_game_arguments(instance, auth),
            Launcher::Modern(l) => l.build_game_arguments(instance, auth),
        }
    }
    
    fn build_jvm_arguments(&self, instance: &MinecraftInstance, memory: u32) -> Vec<String> {
        match self {
            Launcher::Legacy(l) => l.build_jvm_arguments(instance, memory),
            Launcher::Modern(l) => l.build_jvm_arguments(instance, memory),
        }
    }
    
    fn get_main_class(&self, instance: &MinecraftInstance) -> Result<String, String> {
        match self {
            Launcher::Legacy(l) => l.get_main_class(instance),
            Launcher::Modern(l) => l.get_main_class(instance),
        }
    }
    
    async fn extract_natives(&self, instance: &MinecraftInstance) -> Result<(), String> {
        match self {
            Launcher::Legacy(l) => l.extract_natives(instance).await,
            Launcher::Modern(l) => l.extract_natives(instance).await,
        }
    }
}

/// Get the appropriate launcher for a Minecraft version
pub fn get_launcher_for_version(version: &str) -> Result<Launcher, String> {
    let legacy = legacy::LegacyLauncher::new();
    if legacy.can_handle_version(version) {
        return Ok(Launcher::Legacy(legacy));
    }
    
    let modern = modern::ModernLauncher::new();
    if modern.can_handle_version(version) {
        return Ok(Launcher::Modern(modern));
    }
    
    Err(format!("No launcher found for Minecraft version {}", version))
}

/// Main launch function - automatically selects the right launcher
pub async fn launch_minecraft(instance: &MinecraftInstance, auth: Option<AuthInfo>, memory: u32) -> Result<LaunchResult, String> {
    let launcher = get_launcher_for_version(&instance.version)?;
    let auth_info = auth.unwrap_or_default();
    launcher.launch(instance, &auth_info, memory).await
}