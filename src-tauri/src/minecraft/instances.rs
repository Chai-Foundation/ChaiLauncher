//! Instance Management using MCVM
//! 
//! This module handles Minecraft instance creation, validation, and management
//! using MCVM's instance system while maintaining compatibility with ChaiLauncher's API.

use super::{MinecraftInstance, MCVMCore};
use std::path::PathBuf;
use uuid;
use chrono;

/// Instance management wrapper
pub struct Instance;

impl Instance {
    /// Create a new Minecraft instance using MCVM validation
    pub async fn create(
        name: String,
        version: String,
        game_dir: PathBuf,
    ) -> Result<MinecraftInstance, String> {
        println!("🏗️  Creating instance '{}' with Minecraft {}", name, version);

        // Validate using MCVM
        MCVMCore::create_launch_instance(
            &name,
            &version,
            game_dir.clone(),
        ).await?;

        // Create ChaiLauncher instance
        let chai_instance = MinecraftInstance {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            version: version.clone(),
            modpack: None,
            modpack_version: None,
            game_dir: game_dir.clone(),
            java_path: None,
            jvm_args: None,
            last_played: None,
            total_play_time: 0,
            icon: None,
            is_modded: false,
            mods_count: 0,
            is_external: Some(false),
            external_launcher: None,
        };

        // Store in ChaiLauncher's storage system
        let mut storage = crate::storage::StorageManager::new().await
            .map_err(|e| format!("Failed to initialize storage: {}", e))?;
        
        // Convert to InstanceMetadata for storage
        let instance_metadata = crate::storage::InstanceMetadata {
            id: chai_instance.id.clone(),
            name: chai_instance.name.clone(),
            version: chai_instance.version.clone(),
            game_dir: chai_instance.game_dir.clone(),
            java_path: chai_instance.java_path.clone(),
            jvm_args: chai_instance.jvm_args.clone(),
            last_played: chai_instance.last_played.clone(),
            total_play_time: chai_instance.total_play_time,
            icon: chai_instance.icon.clone(),
            is_modded: chai_instance.is_modded,
            mods_count: chai_instance.mods_count,
            modpack: chai_instance.modpack.clone(),
            modpack_version: chai_instance.modpack_version.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_mb: None, // Will be calculated later
            description: None,
            tags: vec![],
        };
        
        storage.add_instance(instance_metadata).await
            .map_err(|e| format!("Failed to add instance: {}", e))?;

        println!("✅ Instance '{}' created successfully", chai_instance.name);
        Ok(chai_instance)
    }

    /// Validate an instance directory structure
    pub async fn validate(instance: &MinecraftInstance) -> Result<(), String> {
        // Check if game directory exists
        if !instance.game_dir.exists() {
            return Err(format!("Game directory does not exist: {}", instance.game_dir.display()));
        }

        // Validate using MCVM
        MCVMCore::create_launch_instance(
            &instance.id,
            &instance.version,
            instance.game_dir.clone(),
        ).await?;

        Ok(())
    }

    /// Get instance information using ChaiLauncher's storage
    pub async fn get_info(instance_id: &str) -> Result<Option<MinecraftInstance>, String> {
        // Load from ChaiLauncher's storage system
        let storage = crate::storage::StorageManager::new().await
            .map_err(|e| format!("Failed to initialize storage: {}", e))?;
        
        match storage.get_instance(instance_id) {
            Some(metadata) => {
                // Convert from InstanceMetadata to MinecraftInstance
                let instance = MinecraftInstance {
                    id: metadata.id.clone(),
                    name: metadata.name.clone(),
                    version: metadata.version.clone(),
                    game_dir: metadata.game_dir.clone(),
                    java_path: metadata.java_path.clone(),
                    jvm_args: metadata.jvm_args.clone(),
                    last_played: metadata.last_played.clone(),
                    total_play_time: metadata.total_play_time,
                    icon: metadata.icon.clone(),
                    is_modded: metadata.is_modded,
                    mods_count: metadata.mods_count,
                    is_external: Some(false), // Default for MCVM-managed instances
                    external_launcher: None,
                    modpack: metadata.modpack.clone(),
                    modpack_version: metadata.modpack_version.clone(),
                };
                Ok(Some(instance))
            },
            None => Ok(None)
        }
    }

    /// List all instances from ChaiLauncher's storage
    pub async fn list_all() -> Result<Vec<MinecraftInstance>, String> {
        let storage = crate::storage::StorageManager::new().await
            .map_err(|e| format!("Failed to initialize storage: {}", e))?;
        
        // Get all instances from storage and convert them
        let metadata_instances = storage.get_all_instances();
        
        let mut instances = Vec::new();
        for metadata in metadata_instances {
            let instance = MinecraftInstance {
                id: metadata.id.clone(),
                name: metadata.name.clone(),
                version: metadata.version.clone(),
                game_dir: metadata.game_dir.clone(),
                java_path: metadata.java_path.clone(),
                jvm_args: metadata.jvm_args.clone(),
                last_played: metadata.last_played.clone(),
                total_play_time: metadata.total_play_time,
                icon: metadata.icon.clone(),
                is_modded: metadata.is_modded,
                mods_count: metadata.mods_count,
                is_external: Some(false),
                external_launcher: None,
                modpack: metadata.modpack.clone(),
                modpack_version: metadata.modpack_version.clone(),
            };
            instances.push(instance);
        }
        
        Ok(instances)
    }

    /// Delete an instance
    pub async fn delete(instance_id: &str) -> Result<(), String> {
        let mut storage = crate::storage::StorageManager::new().await
            .map_err(|e| format!("Failed to initialize storage: {}", e))?;
        
        storage.remove_instance(instance_id).await
            .map_err(|e| format!("Failed to remove instance: {}", e))?;

        println!("✅ Instance '{}' deleted successfully", instance_id);
        Ok(())
    }
}