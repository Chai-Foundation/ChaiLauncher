//! MCVM Core Integration
//! 
//! This module provides a simplified wrapper around MCVM for ChaiLauncher's needs.
//! Rather than trying to fully integrate MCVM's complex configuration system,
//! we use MCVM selectively for the tasks it excels at while maintaining
//! ChaiLauncher's existing Java management and UI integration.

use std::sync::OnceLock;
use mcvm::io::paths::Paths;
use mcvm::instance::{Instance as MCVMInstance, InstKind, InstanceStoredConfig};
use mcvm::instance::launch::{LaunchSettings, InstanceHandle};
use mcvm::config::instance::ClientWindowConfig;
use mcvm::shared::output::{MCVMOutput, MessageLevel, MessageContents};
use mcvm::core::user::UserManager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};
use serde_json;
use chrono;

static MCVM_PATHS: OnceLock<Paths> = OnceLock::new();

/// ChaiLauncher's MCVM integration wrapper
pub struct MCVMCore;

impl MCVMCore {
    /// Initialize MCVM paths for ChaiLauncher
    pub async fn initialize() -> Result<(), String> {
        println!("🔧 Initializing MCVM integration...");
        
        // Create MCVM paths using default location
        let paths = Paths::new().await
            .map_err(|e| format!("Failed to create MCVM paths: {}", e))?;

        MCVM_PATHS.set(paths)
            .map_err(|_| "Failed to set global MCVM paths")?;

        println!("✅ MCVM integration initialized successfully");
        Ok(())
    }

    /// Get the global MCVM paths
    pub fn paths() -> Result<&'static Paths, String> {
        MCVM_PATHS.get().ok_or_else(|| "MCVM not initialized".to_string())
    }

    /// Create a proper MCVM instance for launching
    pub async fn create_launch_instance(
        name: &str,
        version: &str,
        game_dir: PathBuf,
    ) -> Result<MCVMInstance, String> {
        let _paths = Self::paths()?;
        
        // Validate that the game directory exists
        if !game_dir.exists() {
            tokio::fs::create_dir_all(&game_dir).await
                .map_err(|e| format!("Failed to create game directory: {}", e))?;
        }
        
        // Create the MCVM instance with client type
        let instance_kind = InstKind::Client { 
            window: ClientWindowConfig::default(),
        };
        
        // Create instance ID and basic config
        let instance_id = name.to_string().into();
        
        // Create a minimal stored config
        let stored_config = InstanceStoredConfig {
            name: Some(name.to_string()),
            modifications: vec![],
            launch: None,
            datapack_folder: None,
            packages: vec![],
            schema_version: mcvm_shared::util::versions::INSTANCE_SCHEMA_VERSION,
            version: version.to_string().into(),
        };
        
        let instance = MCVMInstance::new(
            instance_kind,
            instance_id,
            stored_config,
        );
        
        println!("✅ MCVM instance created for '{}' version {}", name, version);
        Ok(instance)
    }

    /// Launch an instance using MCVM with proper output handling
    pub async fn launch_instance_with_java(
        mut instance: MCVMInstance,
        java_path: String,
        memory: u32,
        username: String,
        app_handle: Option<AppHandle>,
        instance_name: String,
    ) -> Result<InstanceHandle, String> {
        let paths = Self::paths()?;
        
        // Validate launch parameters
        if !std::path::Path::new(&java_path).exists() {
            return Err(format!("Java path does not exist: {}", java_path));
        }
        
        if memory < 512 {
            return Err("Memory must be at least 512MB".to_string());
        }
        
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        
        // Create launch settings for MCVM - simplified for offline use
        let launch_settings = LaunchSettings {
            ms_client_id: mcvm_auth::mc::ClientId::new("00000000-0000-0000-0000-000000000000".to_string()),
            offline_auth: true, // Use offline auth for ChaiLauncher
        };

        // Create our professional output handler
        let mut output = ChaiLauncherMCVMOutput::new(app_handle, instance_name);

        // Create managers (MCVM expects these)
        let ms_client_id = mcvm_auth::mc::ClientId::new("00000000-0000-0000-0000-000000000000".to_string());
        let mut user_manager = UserManager::new(ms_client_id);
        let plugin_manager = mcvm::plugin::PluginManager::new();

        // Launch the instance with proper logging
        let handle = instance.launch(
            paths,
            &mut user_manager,
            &plugin_manager,
            launch_settings,
            &mut output,
        ).await.map_err(|e| format!("Failed to launch with MCVM: {}", e))?;

        Ok(handle)
    }
    
    /// Get logs from a running MCVM instance
    pub async fn get_instance_logs(output_handler: &ChaiLauncherMCVMOutput) -> Vec<String> {
        output_handler.get_logs().await
    }
    
    /// Download and install assets using MCVM
    pub async fn ensure_assets(
        instance: &mut MCVMInstance,
        version: &str,
        app_handle: Option<AppHandle>,
    ) -> Result<(), String> {
        let paths = Self::paths()?;
        
        // Create output handler for asset progress
        let mut output = ChaiLauncherMCVMOutput::new(
            app_handle.clone(), 
            format!("Assets-{}", version)
        );
        
        // Use MCVM's create method to ensure assets are downloaded
        let ms_client_id = mcvm_auth::mc::ClientId::new("00000000-0000-0000-0000-000000000000".to_string());
        let mut user_manager = UserManager::new(ms_client_id);
        let plugin_manager = mcvm::plugin::PluginManager::new();
        let client = reqwest::Client::new();
        
        // Log asset preparation
        println!("📦 Ensuring assets for Minecraft {}", version);
        
        // Create the instance which will download assets if needed
        instance.create(
            &mut mcvm::instance::update::manager::UpdateManager::new(false, true),
            &plugin_manager,
            paths,
            &user_manager,
            &client,
            &mut output,
        ).await.map_err(|e| format!("Failed to create instance/assets: {}", e))?;
        
        println!("✅ Assets ready for Minecraft {}", version);
        
        Ok(())
    }
}

/// Professional MCVM Output Handler for ChaiLauncher
/// Handles all MCVM output including logs, progress updates, and user prompts
pub struct ChaiLauncherMCVMOutput {
    app_handle: Option<AppHandle>,
    instance_name: String,
    log_buffer: Arc<Mutex<Vec<String>>>,
}

impl ChaiLauncherMCVMOutput {
    pub fn new(app_handle: Option<AppHandle>, instance_name: String) -> Self {
        Self {
            app_handle,
            instance_name,
            log_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn get_logs(&self) -> Vec<String> {
        let buffer = self.log_buffer.lock().await;
        buffer.clone()
    }
    
    async fn emit_log(&self, message: &str, level: &str) {
        // Store in buffer
        {
            let mut buffer = self.log_buffer.lock().await;
            buffer.push(format!("[{}] {}", level, message));
            
            // Keep only last 1000 log entries
            if buffer.len() > 1000 {
                buffer.remove(0);
            }
        }
        
        // Emit to frontend if available
        if let Some(app) = &self.app_handle {
            let _ = app.emit("minecraft_log", serde_json::json!({
                "instance": self.instance_name,
                "level": level,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
        }
        
        // Also print to console for debugging
        match level {
            "ERROR" => eprintln!("🔴 [{}] {}", self.instance_name, message),
            "WARN" => println!("🟡 [{}] {}", self.instance_name, message),
            "INFO" => println!("🔵 [{}] {}", self.instance_name, message),
            _ => println!("⚪ [{}] {}", self.instance_name, message),
        }
    }
}

impl MCVMOutput for ChaiLauncherMCVMOutput {
    fn display_text(&mut self, text: String, level: MessageLevel) {
        let level_str = match level {
            MessageLevel::Important => "INFO",
            MessageLevel::Extra => "INFO",
            MessageLevel::Debug => "DEBUG",
            MessageLevel::Trace => "TRACE",
        };
        
        // Use tokio spawn to handle async in sync context
        let output_clone = Arc::new(Mutex::new(self.clone()));
        let text_clone = text.clone();
        let level_clone = level_str.to_string();
        
        tokio::spawn(async move {
            let output = output_clone.lock().await;
            output.emit_log(&text_clone, &level_clone).await;
        });
    }

    fn prompt_yes_no(
        &mut self,
        default: bool,
        message: mcvm::shared::output::MessageContents,
    ) -> anyhow::Result<bool> {
        // Log the prompt  
        let prompt_text = format!("Prompt: {} (default: {})", "User prompt", default);
        let output_clone = Arc::new(Mutex::new(self.clone()));
        let prompt_clone = prompt_text.clone();
        
        tokio::spawn(async move {
            let output = output_clone.lock().await;
            output.emit_log(&prompt_clone, "PROMPT").await;
        });
        
        // For automated operation, return the default
        // In a full implementation, this could wait for user input via the frontend
        Ok(default)
    }
}

// Implement Clone for the output handler
impl Clone for ChaiLauncherMCVMOutput {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            instance_name: self.instance_name.clone(),
            log_buffer: self.log_buffer.clone(),
        }
    }
}

/// Initialize MCVM integration
pub async fn initialize() -> Result<(), String> {
    MCVMCore::initialize().await
}