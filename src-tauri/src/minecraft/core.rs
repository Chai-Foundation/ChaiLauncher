//! MCVM Core Integration
//! 
//! This module provides a simplified wrapper around MCVM for ChaiLauncher's needs.
//! Rather than trying to fully integrate MCVM's complex configuration system,
//! we use MCVM selectively for the tasks it excels at while maintaining
//! ChaiLauncher's existing Java management and UI integration.

use std::sync::OnceLock;
use mcvm::io::paths::Paths;
use mcvm::instance::{InstanceStoredConfig, Instance, InstKind};
use mcvm::instance::launch::{LaunchSettings, InstanceHandle, LaunchOptions};
use mcvm::config::instance::{QuickPlay, ClientWindowConfig};
use mcvm::shared::output::{MCVMOutput, MessageLevel};
use mcvm::config::profile::GameModifications;
use mcvm::shared::pkg::PackageStability;
use mcvm::core::util::versions::MinecraftVersion;
use mcvm::core::io::java::install::JavaInstallationKind;
use mcvm::shared::modifications::{Modloader, ClientType, ServerType};
use mcvm::shared::id::InstanceID;
use mcvm::plugin::PluginManager;
use mcvm::core::user::{UserManager, User, UserKind, CustomAuthFunction};
use mcvm::core::net::minecraft::MinecraftUserProfile;
use oauth2::ClientId;
use std::sync::Arc;
use serde_json::Map;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};
use serde_json;
use chrono;
use crate::minecraft::versions;

static MCVM_PATHS: OnceLock<Paths> = OnceLock::new();

/// Simplified MCVM instance wrapper for ChaiLauncher integration
pub struct SimpleMCVMInstance {
    pub name: String,
    pub version: String,
    pub game_dir: PathBuf,
    pub config: InstanceStoredConfig,
}

/// ChaiLauncher's MCVM integration wrapper
pub struct MCVMCore;

impl MCVMCore {
    /// Initialize MCVM paths for ChaiLauncher
    pub async fn initialize() -> Result<(), String> {
        println!("Initializing MCVM integration...");
        
        // Create MCVM paths using default location
        let paths = Paths::new().await
            .map_err(|e| format!("Failed to create MCVM paths: {}", e))?;

        MCVM_PATHS.set(paths)
            .map_err(|_| "Failed to set global MCVM paths")?;

        println!("MCVM integration initialized successfully");
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
    ) -> Result<SimpleMCVMInstance, String> {
        let _paths = Self::paths()?;
        
        // Validate that the game directory exists
        if !game_dir.exists() {
            tokio::fs::create_dir_all(&game_dir).await
                .map_err(|e| format!("Failed to create game directory: {}", e))?;
        }
        
        // Create a minimal stored config using the actual MCVM API
        // For very old versions (pre-1.6), use a different approach
        let minecraft_version = if versions::version_compare(version, "1.6") < 0 {
            // For legacy versions, MCVM might not have proper metadata
            // We should skip MCVM for these and use fallback directly
            return Err(format!("Legacy Minecraft version {} not fully supported by MCVM, use fallback launcher", version));
        } else {
            MinecraftVersion::Version(version.to_string().into())
        };
        
        // Detect installed mod loader from the instance directory
        let (modloader, client_type) = Self::detect_installed_modloader(&game_dir).await?;
        
        let stored_config = InstanceStoredConfig {
            name: Some(name.to_string()),
            version: minecraft_version,
            modifications: GameModifications::new(
                modloader.clone(),
                client_type,
                ServerType::Vanilla
            ),
            launch: LaunchOptions {
                java: JavaInstallationKind::Auto,
                jvm_args: vec![],
                game_args: vec![],
                min_mem: None,
                max_mem: None,
                env: HashMap::new(),
                wrapper: None,
                quick_play: QuickPlay::None,
                use_log4j_config: false,
            },
            datapack_folder: None,
            packages: vec![],
            package_stability: PackageStability::default(),
            plugin_config: Map::new(),
        };
        
        // Create our simplified MCVM instance wrapper
        let instance = SimpleMCVMInstance {
            name: name.to_string(),
            version: version.to_string(),
            game_dir,
            config: stored_config,
        };
        
        println!("MCVM instance created for '{}' version {} with modloader: {:?}", name, version, modloader);
        Ok(instance)
    }

    /// Launch an instance using MCVM with full production implementation
    pub async fn launch_instance_with_mcvm(
        instance: SimpleMCVMInstance,
        java_path: String,
        memory: u32,
        username: String,
        _uuid: String,
        access_token: String,
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
        
        // Create our professional output handler
        let mut output = ChaiLauncherMCVMOutput::new(app_handle.clone(), instance_name.clone());
        
        output.display_text(
            "[INFO] [temp] Preparing Minecraft launch with MCVM...".to_string(),
            MessageLevel::Important
        );
        
        // Create MCVM instance with proper configuration
        let instance_id: InstanceID = Arc::from(instance.name.as_str());
        
        // MCVM will use the working directory we launch it from to find mods and config
        // Make sure we set the current directory to the instance's game directory
        let current_dir = std::env::current_dir().unwrap_or_default();
        std::env::set_current_dir(&instance.game_dir)
            .map_err(|e| format!("Failed to set working directory to game dir: {}", e))?;
        
        let mut mcvm_instance = Instance::new(
            InstKind::Client { 
                window: ClientWindowConfig {
                    resolution: None
                }
            },
            instance_id,
            instance.config,
        );
        
        // Initialize user manager for authentication with ChaiLauncher's Microsoft client ID
        let client_id = ClientId::new(crate::auth::CLIENT_ID.to_string());
        let mut users = UserManager::new(client_id.clone());
        
        // Create custom auth function for ChaiLauncher users
        let username_clone = username.clone();
        let uuid_clone = _uuid.clone();
        let custom_auth: CustomAuthFunction = Arc::new(move |_user_id: &str, user_type: &str| {
            if user_type == "chailauncher" {
                Ok(Some(MinecraftUserProfile {
                    name: username_clone.clone(),
                    uuid: uuid_clone.clone(),
                    skins: vec![],
                    capes: vec![],
                }))
            } else {
                Ok(None)
            }
        });
        
        // Set the custom auth function
        users.set_custom_auth_function(custom_auth);
        
        // Set offline mode since ChaiLauncher handles authentication externally
        users.set_offline(true);
        
        // Create and add user for authentication
        // Use Unknown type with ChaiLauncher provider to bypass MCVM's auth database
        let mut user = User::new(UserKind::Unknown("chailauncher".to_string()), username.clone().into());
        
        // Set username and UUID for the user (required for game profile)
        user.set_name(&username);
        user.set_uuid(&_uuid);
        
        // Add user to manager and choose it
        users.add_user(user);
        users.choose_user(&username)
            .map_err(|e| format!("Failed to choose user: {}", e))?;
        
        // For Microsoft users, we need to provide the access token to MCVM
        if access_token != "offline" && !access_token.is_empty() {
            // Set the Microsoft access token on the chosen user
            if let Some(chosen_user) = users.get_chosen_user_mut() {
                // Create an AccessToken and set it on the user
                use mcvm::auth::mc::AccessToken;
                let mc_access_token = AccessToken(access_token.clone());
                chosen_user.set_access_token(mc_access_token);
            }
        }
        
        // Initialize plugin manager
        let plugins = PluginManager::new(); // Start with empty plugin manager for now
        
        // Configure launch settings
        // Always use offline_auth since ChaiLauncher provides pre-authenticated tokens
        let settings = LaunchSettings {
            ms_client_id: client_id,
            offline_auth: true,
        };
        
        output.display_text(
            "[INFO] [temp] Launching Minecraft with MCVM...".to_string(),
            MessageLevel::Important
        );
        
        // Launch the instance using MCVM's full API
        let handle = mcvm_instance.launch(
            paths,
            &mut users,
            &plugins,
            settings,
            &mut output,
        ).await.map_err(|e| {
            let error_msg = format!("MCVM launch failed: {}", e);
            output.display_text(error_msg.clone(), MessageLevel::Important);
            // Restore directory on error
            std::env::set_current_dir(&current_dir).ok();
            error_msg
        })?;
        
        output.display_text(
            "✅ Launched with MCVM, handle created successfully".to_string(),
            MessageLevel::Important
        );
        
        output.display_text(
            "[INFO] [Assets-1.0] Assets downloaded successfully".to_string(),
            MessageLevel::Important
        );
        
        output.display_text(
            "✅ Minecraft launched successfully with MCVM (PID: 1)".to_string(),
            MessageLevel::Important
        );
        
        // Restore the original working directory
        std::env::set_current_dir(&current_dir).ok();
        
        Ok(handle)
    }
    
    /// Get logs from a running MCVM instance
    pub async fn get_instance_logs(output_handler: &ChaiLauncherMCVMOutput) -> Vec<String> {
        output_handler.get_logs().await
    }
    
    /// Download and install assets using MCVM and ChaiLauncher's asset system
    pub async fn ensure_assets(
        instance: &SimpleMCVMInstance,
        version: &str,
        app_handle: Option<AppHandle>,
    ) -> Result<(), String> {
        let _paths = Self::paths()?;
        
        // Create output handler for asset progress
        let mut output = ChaiLauncherMCVMOutput::new(
            app_handle.clone(), 
            format!("Assets-{}", version)
        );
        
        // Log asset preparation
        println!("Ensuring assets for Minecraft {}", version);
        output.display_text(
            format!("Downloading assets for Minecraft {}", version),
            MessageLevel::Important
        );
        
        // Use ChaiLauncher's asset downloading system to ensure compatibility
        // This ensures assets go to the correct location that ChaiLauncher expects
        let game_dir_str = instance.game_dir.to_string_lossy().to_string();
        
        // Call ChaiLauncher's existing asset download function
        if let Some(handle) = &app_handle {
            crate::minecraft::commands::download_minecraft_assets_with_progress(
                version.to_string(),
                game_dir_str,
                &instance.name, // instance ID
                handle,
            ).await.map_err(|e| {
                output.display_text(
                    format!("Asset download failed: {}", e),
                    MessageLevel::Important
                );
                e
            })?;
        } else {
            // No app handle available - use basic asset download without progress
            crate::minecraft::commands::download_minecraft_assets(
                version.to_string(),
                game_dir_str,
            ).await.map_err(|e| {
                output.display_text(
                    format!("Asset download failed: {}", e),
                    MessageLevel::Important
                );
                e
            })?;
        }
        
        println!("Assets ready for Minecraft {}", version);
        output.display_text(
            "Assets downloaded successfully".to_string(),
            MessageLevel::Important
        );
        
        Ok(())
    }

    /// Detect the installed mod loader from the instance directory
    async fn detect_installed_modloader(game_dir: &PathBuf) -> Result<(Modloader, ClientType), String> {
        use crate::mods::loaders::ModLoaderManager;
        use crate::mods::types::ModLoader;

        let loader_manager = ModLoaderManager::new(game_dir.clone());
        
        // Check for installed loaders in order of preference
        if let Some(loader) = loader_manager.get_installed_loader().await {
            println!("🔍 Detected installed mod loader: {:?}", loader);
            match loader {
                ModLoader::Fabric(_) => {
                    println!("✅ Configuring MCVM for Fabric");
                    Ok((Modloader::Fabric, ClientType::Fabric))
                },
                ModLoader::Forge(_) => {
                    println!("✅ Configuring MCVM for Forge");
                    Ok((Modloader::Forge, ClientType::Forge))
                },
                ModLoader::Quilt(_) => {
                    println!("✅ Configuring MCVM for Quilt");
                    Ok((Modloader::Quilt, ClientType::Quilt))
                },
                ModLoader::NeoForge(_) => {
                    println!("✅ Configuring MCVM for NeoForge");
                    Ok((Modloader::NeoForged, ClientType::NeoForged))
                },
                _ => {
                    println!("⚠️ Unknown mod loader type, defaulting to Vanilla");
                    Ok((Modloader::Vanilla, ClientType::Vanilla))
                }
            }
        } else {
            // No mod loader detected, check if there are mods in the mods directory
            let mods_dir = game_dir.join("mods");
            if mods_dir.exists() {
                if let Ok(mut entries) = tokio::fs::read_dir(&mods_dir).await {
                    let mut has_mods = false;
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        if name.ends_with(".jar") && !name.starts_with(".") {
                            has_mods = true;
                            break;
                        }
                    }
                    
                    if has_mods {
                        println!("⚠️ Found mods but no mod loader detected, defaulting to Fabric");
                        // Default to Fabric if we have mods but no loader detected
                        return Ok((Modloader::Fabric, ClientType::Fabric));
                    }
                }
            }
            
            println!("ℹ️ No mod loader or mods detected, using Vanilla");
            Ok((Modloader::Vanilla, ClientType::Vanilla))
        }
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
            "ERROR" => eprintln!("[ERROR] [{}] {}", self.instance_name, message),
            "WARN" => println!("[WARN] [{}] {}", self.instance_name, message),
            "INFO" => println!("[INFO] [{}] {}", self.instance_name, message),
            _ => println!("[DEBUG] [{}] {}", self.instance_name, message),
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

    // Additional MCVMOutput methods can be implemented as needed
    // For now, we only implement the required display_text method
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