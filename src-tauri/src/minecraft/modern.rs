use super::*;
use crate::minecraft::versions::*;
use tokio::fs;

/// Handles Minecraft versions 1.13 and above (modern launcher format)
pub struct ModernLauncher;

impl ModernLauncher {
    pub fn new() -> Self {
        Self
    }
    
    /// Parse modern arguments object format
    fn parse_modern_arguments(&self, arguments: &serde_json::Value, instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String> {
        let mut args = Vec::new();
        
        if let Some(game_args) = arguments.get("game").and_then(|v| v.as_array()) {
            for arg in game_args {
                if let Some(arg_str) = arg.as_str() {
                    args.push(self.substitute_argument_variables(arg_str, instance, auth));
                } else if let Some(arg_obj) = arg.as_object() {
                    // Handle conditional arguments
                    if self.should_include_argument(arg_obj) {
                        if let Some(value) = arg_obj.get("value") {
                            if let Some(value_str) = value.as_str() {
                                args.push(self.substitute_argument_variables(value_str, instance, auth));
                            } else if let Some(value_array) = value.as_array() {
                                for val in value_array {
                                    if let Some(val_str) = val.as_str() {
                                        args.push(self.substitute_argument_variables(val_str, instance, auth));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Fallback to default modern arguments
            args = vec![
                "--version".to_string(),
                instance.version.clone(),
                "--gameDir".to_string(),
                instance.game_dir.to_string_lossy().to_string(),
                "--assetsDir".to_string(),
                instance.game_dir.join("assets").to_string_lossy().to_string(),
                "--assetIndex".to_string(),
                get_asset_index(&instance.version),
                "--username".to_string(),
                auth.username.clone(),
                "--uuid".to_string(),
                auth.uuid.clone(),
                "--accessToken".to_string(),
                auth.access_token.clone(),
                "--userType".to_string(),
                auth.user_type.clone(),
            ];
        }
        
        args
    }
    
    /// Substitute template variables in argument strings
    fn substitute_argument_variables(&self, template: &str, instance: &MinecraftInstance, auth: &AuthInfo) -> String {
        let asset_index = get_asset_index(&instance.version);
        
        template
            .replace("${auth_player_name}", &auth.username)
            .replace("${version_name}", &instance.version)
            .replace("${game_directory}", &instance.game_dir.to_string_lossy())
            .replace("${assets_root}", &instance.game_dir.join("assets").to_string_lossy())
            .replace("${assets_index_name}", &asset_index)
            .replace("${auth_uuid}", &auth.uuid)
            .replace("${auth_access_token}", &auth.access_token)
            .replace("${user_type}", &auth.user_type)
            .replace("${version_type}", "release")
    }
    
    /// Check if a conditional argument should be included
    fn should_include_argument(&self, arg_obj: &serde_json::Map<String, serde_json::Value>) -> bool {
        if let Some(rules) = arg_obj.get("rules").and_then(|v| v.as_array()) {
            self.evaluate_rules(rules)
        } else {
            true // No rules means always include
        }
    }
    
    /// Evaluate argument rules (similar to library rules)
    fn evaluate_rules(&self, rules: &[serde_json::Value]) -> bool {
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
    
    /// Build classpath from libraries with modern format support
    async fn build_classpath_from_json(&self, instance: &MinecraftInstance, version_json: &serde_json::Value) -> Result<String, String> {
        let mut entries = Vec::new();
        let libraries_path = instance.game_dir.join("libraries");
        
        if let Some(libraries) = version_json.get("libraries").and_then(|v| v.as_array()) {
            for lib in libraries {
                if let Some(name) = lib.get("name").and_then(|v| v.as_str()) {
                    // Check rules for modern library format
                    if !self.evaluate_library_rules(lib) {
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
                        } else {
                            println!("Warning: Missing library: {}", jar_path.display());
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
    
    /// Evaluate library rules for modern format
    fn evaluate_library_rules(&self, library: &serde_json::Value) -> bool {
        if let Some(rules) = library.get("rules").and_then(|v| v.as_array()) {
            self.evaluate_rules(rules)
        } else {
            true // No rules means include
        }
    }
}

impl MinecraftLauncher for ModernLauncher {
    fn supported_versions(&self) -> &str {
        "1.13+"
    }
    
    fn can_handle_version(&self, version: &str) -> bool {
        let (major, minor) = parse_version(version);
        matches!((major, minor), (1, 13..=99) | (2, _))
    }
    
    fn required_java_version(&self, version: &str) -> u32 {
        let (major, minor) = parse_version(version);
        match (major, minor) {
            (1, 17..) => 17, // Java 17 for 1.17+
            _ => 8, // Java 8 for 1.13-1.16
        }
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
        
        println!("✓ Modern Minecraft {} validation passed", instance.version);
        Ok(())
    }
    
    async fn build_classpath(&self, instance: &MinecraftInstance) -> Result<String, String> {
        // Load version JSON for precise classpath building
        if let Ok(Some(version_json)) = load_version_manifest(&instance.game_dir, &instance.version).await {
            self.build_classpath_from_json(instance, &version_json).await
        } else {
            Err("Cannot build classpath: version JSON not found".to_string())
        }
    }
    
    fn build_game_arguments(&self, instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String> {
        // Try to load version JSON for modern arguments
        if let Ok(Some(version_json)) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                load_version_manifest(&instance.game_dir, &instance.version)
            )
        }) {
            if let Some(arguments) = version_json.get("arguments") {
                return self.parse_modern_arguments(arguments, instance, auth);
            }
        }
        
        // Fallback to standard arguments
        vec![
            "--version".to_string(),
            instance.version.clone(),
            "--gameDir".to_string(),
            instance.game_dir.to_string_lossy().to_string(),
            "--assetsDir".to_string(),
            instance.game_dir.join("assets").to_string_lossy().to_string(),
            "--assetIndex".to_string(),
            get_asset_index(&instance.version),
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
        vec![
            format!("-Xmx{}M", memory),
            format!("-Xms{}M", memory),
            "-XX:+UnlockExperimentalVMOptions".to_string(),
            "-XX:+UseG1GC".to_string(),
            "-XX:G1NewSizePercent=20".to_string(),
            "-XX:G1ReservePercent=20".to_string(),
            "-XX:MaxGCPauseMillis=50".to_string(),
            "-XX:G1HeapRegionSize=32M".to_string(),
            "-Dlog4j2.formatMsgNoLookups=true".to_string(),
            "-Dminecraft.launcher.brand=ChaiLauncher".to_string(),
            "-Dminecraft.launcher.version=1.0.0".to_string(),
            "-Dfile.encoding=UTF-8".to_string(),
            "-Duser.language=en".to_string(),
            "-Duser.country=US".to_string(),
        ]
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
        
        // Modern versions always use this main class
        Ok("net.minecraft.client.main.Main".to_string())
    }
    
    async fn extract_natives(&self, instance: &MinecraftInstance) -> Result<(), String> {
        // Modern versions typically handle natives automatically
        // Just ensure the natives directory exists
        let natives_dir = instance.game_dir.join("natives");
        
        if !natives_dir.exists() {
            fs::create_dir_all(&natives_dir).await
                .map_err(|e| format!("Failed to create natives directory: {}", e))?;
        }
        
        Ok(())
    }
}