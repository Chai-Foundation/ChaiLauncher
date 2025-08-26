//! ChaiLauncher Minecraft Module - MCVM-based backend
//! 
//! This module provides a simplified, MCVM-powered backend for Minecraft launching
//! while maintaining the same Tauri API surface for the frontend.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

// Core MCVM-based modules
pub mod core;          // MCVM wrapper and core functionality
pub mod instances;     // Instance management using MCVM
pub mod versions;      // Version management using MCVM  
pub mod launcher;      // Launching logic using MCVM
pub mod assets;        // Asset management using MCVM
pub mod java;          // Java management (enhanced with MCVM)
pub mod commands;      // Tauri commands (unchanged API)

// Re-export main types for compatibility
pub use core::MCVMCore;

/// Main Minecraft instance structure (compatible with existing frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstance {
    pub id: String,
    pub name: String,
    pub version: String,
    pub modpack: Option<String>,
    #[serde(rename = "modpackVersion")]
    pub modpack_version: Option<String>,
    #[serde(rename = "gameDir")]
    pub game_dir: PathBuf,
    #[serde(rename = "javaPath")]
    pub java_path: Option<String>,
    #[serde(rename = "jvmArgs")]
    pub jvm_args: Option<Vec<String>>,
    #[serde(rename = "lastPlayed")]
    pub last_played: Option<String>,
    #[serde(rename = "totalPlayTime")]
    pub total_play_time: u64,
    pub icon: Option<String>,
    #[serde(rename = "isModded")]
    pub is_modded: bool,
    #[serde(rename = "modsCount")]
    pub mods_count: u32,
    #[serde(rename = "isExternal")]
    pub is_external: Option<bool>,
    #[serde(rename = "externalLauncher")]
    pub external_launcher: Option<String>,
}

/// Authentication information
#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
}

impl Default for AuthInfo {
    fn default() -> Self {
        Self {
            username: "Player".to_string(),
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            access_token: "offline".to_string(),
            user_type: "offline".to_string(),
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

/// Main entry point for Minecraft operations
pub async fn initialize_minecraft() -> Result<(), String> {
    core::initialize().await
}

/// Launch a Minecraft instance using MCVM
pub async fn launch_minecraft(
    instance: &MinecraftInstance, 
    auth: Option<AuthInfo>, 
    memory: u32
) -> Result<LaunchResult, String> {
    launcher::launch_instance(instance, auth.unwrap_or_default(), memory).await
}