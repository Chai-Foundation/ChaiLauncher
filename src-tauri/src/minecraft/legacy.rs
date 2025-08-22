use super::*;
use crate::minecraft::versions::*;
use std::path::PathBuf;
use tokio::fs;

/// Handles Minecraft versions 1.0 through 1.12 (legacy launcher format)
pub struct LegacyLauncher;

impl LegacyLauncher {
    pub fn new() -> Self {
        Self
    }
    
    /// Parse legacy minecraftArguments string template into actual arguments
    fn parse_legacy_arguments(&self, template: &str, instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String> {
        let asset_index = get_asset_index(&instance.version);
        
        let substituted = template
            .replace("${auth_player_name}", &auth.username)
            .replace("${version_name}", &instance.version)
            .replace("${game_directory}", &instance.game_dir.to_string_lossy())
            .replace("${assets_root}", &instance.game_dir.join("assets").to_string_lossy())
            .replace("${game_assets}", &instance.game_dir.join("assets").to_string_lossy()) // Legacy field name
            .replace("${assets_index_name}", &asset_index)
            .replace("${auth_uuid}", &auth.uuid)
            .replace("${auth_access_token}", &auth.access_token)
            .replace("${auth_session}", &auth.access_token) // Legacy field name
            .replace("${user_type}", &auth.user_type)
            .replace("${version_type}", "release");
        
        substituted.split_whitespace().map(|s| s.to_string()).collect()
    }
    
    /// Build classpath from libraries listed in version JSON
    async fn build_classpath_from_json(&self, instance: &MinecraftInstance, version_json: &serde_json::Value) -> Result<String, String> {
        let mut entries = Vec::new();
        let libraries_path = instance.game_dir.join("libraries");
        
        // Parse libraries from JSON
        if let Some(libraries) = version_json.get("libraries").and_then(|v| v.as_array()) {
            for lib in libraries {
                if let Some(name) = lib.get("name").and_then(|v| v.as_str()) {
                    // Check if library should be included (rules-based filtering)
                    if !self.should_include_library(lib) {
                        continue;
                    }
                    
                    let parts: Vec<&str> = name.split(':').collect();
                    if parts.len() >= 3 {
                        let group = parts[0].replace(".", "/");
                        let artifact = parts[1];
                        let version = parts[2];
                        
                        let jar_path = libraries_path
                            .join(&group)
                            .join(&artifact)
                            .join(&version)
                            .join(format!("{}-{}.jar", artifact, version));
                        
                        if jar_path.exists() {
                            entries.push(jar_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        
        // Add main version JAR
        let main_jar = instance.game_dir
            .join("versions")
            .join(&instance.version)
            .join(format!("{}.jar", instance.version));
        
        if main_jar.exists() {
            entries.push(main_jar.to_string_lossy().to_string());
        } else {
            return Err(format!("Main JAR not found: {}", main_jar.display()));
        }
        
        let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        Ok(entries.join(sep))
    }
    
    /// Check if a library should be included based on rules
    fn should_include_library(&self, library: &serde_json::Value) -> bool {
        let Some(rules) = library.get("rules").and_then(|v| v.as_array()) else {
            return true; // No rules means include
        };
        
        if rules.is_empty() {
            return true;
        }
        
        let current_os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };
        
        let mut allow = false;
        
        for rule in rules {
            if let Some(action) = rule.get("action").and_then(|v| v.as_str()) {
                let rule_applies = if let Some(os_obj) = rule.get("os") {
                    if let Some(os_name) = os_obj.get("name").and_then(|v| v.as_str()) {
                        os_name == current_os
                    } else {
                        true
                    }
                } else {
                    true
                };
                
                if rule_applies {
                    match action {
                        "allow" => allow = true,
                        "disallow" => allow = false,
                        _ => {}
                    }
                }
            }
        }
        
        allow
    }
}

impl MinecraftLauncher for LegacyLauncher {
    fn supported_versions(&self) -> &str {
        "1.0-1.12"
    }
    
    fn can_handle_version(&self, version: &str) -> bool {
        let (major, minor) = parse_version(version);
        matches!((major, minor), (1, 0..=12))
    }
    
    fn required_java_version(&self, _version: &str) -> u32 {
        8 // Legacy versions all need Java 8
    }
    
    async fn validate_instance(&self, instance: &MinecraftInstance) -> Result<(), String> {
        // Check basic directory structure
        let versions_dir = instance.game_dir.join("versions").join(&instance.version);
        if !versions_dir.exists() {
            return Err(format!("Version directory not found: {}", versions_dir.display()));
        }
        
        // Check for version JAR
        let version_jar = versions_dir.join(format!("{}.jar", instance.version));
        if !version_jar.exists() {
            return Err(format!("Version JAR not found: {}", version_jar.display()));
        }
        
        // Check for version JSON
        let version_json = versions_dir.join(format!("{}.json", instance.version));
        if !version_json.exists() {
            return Err(format!("Version metadata not found: {}", version_json.display()));
        }
        
        // Check libraries directory
        let libraries_dir = instance.game_dir.join("libraries");
        if !libraries_dir.exists() {
            return Err(format!("Libraries directory not found: {}", libraries_dir.display()));
        }
        
        // Check assets directory
        let assets_dir = instance.game_dir.join("assets");
        if !assets_dir.exists() {
            return Err(format!("Assets directory not found: {}", assets_dir.display()));
        }
        
        println!("✓ Legacy Minecraft {} validation passed", instance.version);
        Ok(())
    }
    
    async fn build_classpath(&self, instance: &MinecraftInstance) -> Result<String, String> {
        // Try to load version JSON for precise classpath building
        if let Ok(Some(version_json)) = load_version_manifest(&instance.game_dir, &instance.version).await {
            self.build_classpath_from_json(instance, &version_json).await
        } else {
            // Fallback to simple classpath
            let main_jar = instance.game_dir
                .join("versions")
                .join(&instance.version)
                .join(format!("{}.jar", instance.version));
            
            if main_jar.exists() {
                Ok(main_jar.to_string_lossy().to_string())
            } else {
                Err("Cannot build classpath: version JAR not found".to_string())
            }
        }
    }
    
    fn build_game_arguments(&self, instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String> {
        // Try to load version JSON for arguments
        if let Ok(Some(version_json)) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                load_version_manifest(&instance.game_dir, &instance.version)
            )
        }) {
            if let Some(minecraft_args) = version_json.get("minecraftArguments").and_then(|v| v.as_str()) {
                return self.parse_legacy_arguments(minecraft_args, instance, auth);
            }
        }
        
        // Fallback to standard legacy arguments
        let asset_index = get_asset_index(&instance.version);
        vec![
            "--version".to_string(),
            instance.version.clone(),
            "--gameDir".to_string(),
            instance.game_dir.to_string_lossy().to_string(),
            "--assetsDir".to_string(),
            instance.game_dir.join("assets").to_string_lossy().to_string(),
            "--assetIndex".to_string(),
            asset_index,
            "--username".to_string(),
            auth.username.clone(),
            "--uuid".to_string(),
            auth.uuid.clone(),
            "--accessToken".to_string(),
            auth.access_token.clone(),
            "--userType".to_string(),
            auth.user_type.clone(),
        ]
    }
    
    fn build_jvm_arguments(&self, _instance: &MinecraftInstance, memory: u32) -> Vec<String> {
        let args = vec![
            format!("-Xmx{}M", memory),
            format!("-Xms{}M", memory),
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+UseG1GC".to_string(),
            "-Dlog4j2.formatMsgNoLookups=true".to_string(),
            "-Dorg.slf4j.simpleLogger.defaultLogLevel=warn".to_string(),
            "-Dminecraft.launcher.brand=ChaiLauncher".to_string(),
            "-Dminecraft.launcher.version=1.0.0".to_string(),
            "-Dfile.encoding=UTF-8".to_string(),
        ];
        
        // Legacy versions (like MC 1.0) should use Java 8 which doesn't support --add-opens
        // Don't add any Java 9+ specific arguments
        
        args
    }
    
    fn get_main_class(&self, instance: &MinecraftInstance) -> Result<String, String> {
        // Try to get main class from version JSON
        if let Ok(Some(version_json)) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                load_version_manifest(&instance.game_dir, &instance.version)
            )
        }) {
            if let Some(main_class) = version_json.get("mainClass").and_then(|v| v.as_str()) {
                return Ok(main_class.to_string());
            }
        }
        
        // Fallback based on version
        Ok(get_main_class_for_version(&instance.version))
    }
    
    async fn extract_natives(&self, instance: &MinecraftInstance) -> Result<(), String> {
        // Check if natives directory exists
        let instance_natives = instance.game_dir.join("natives");
        let version_natives = instance.game_dir
            .join("versions")
            .join(&instance.version)
            .join("natives");
        
        // For legacy versions, prefer version-specific natives
        let natives_exist = version_natives.exists() || instance_natives.exists();
        
        if !natives_exist {
            // Create version-specific natives directory
            fs::create_dir_all(&version_natives).await
                .map_err(|e| format!("Failed to create natives directory: {}", e))?;
            
            println!("ℹ️  Created natives directory for legacy version");
        }
        
        Ok(())
    }
}