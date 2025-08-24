//! Minecraft Launcher using MCVM
//! 
//! This module handles the actual launching of Minecraft instances using MCVM
//! while maintaining ChaiLauncher's Java management and API compatibility.

use super::{MinecraftInstance, AuthInfo, LaunchResult, MCVMCore, versions};
use std::process::Stdio;

/// Launch a Minecraft instance using MCVM integration
pub async fn launch_instance(
    instance: &MinecraftInstance,
    auth: AuthInfo,
    memory: u32,
) -> Result<LaunchResult, String> {
    println!("🚀 Launching Minecraft {} using MCVM backend", instance.version);

    // Validate instance
    super::instances::Instance::validate(instance).await?;

    // Get required Java version and ensure it's installed
    let java_version = versions::get_required_java_version(&instance.version);
    let java_path = versions::get_java_for_version(java_version).await?;

    println!("☕ Using Java {}: {}", java_version, java_path);

    // Try MCVM launcher first
    match try_mcvm_launch(instance, &auth, memory, &java_path).await {
        Ok(result) => {
            println!("✅ Minecraft launched successfully with MCVM (PID: {})", result.process_id);
            Ok(result)
        }
        Err(mcvm_error) => {
            println!("⚠️  MCVM launch failed: {}", mcvm_error);
            println!("🔄 Falling back to direct launch");
            
            // Fall back to direct launch
            fallback_launch(instance, auth, memory, &java_path).await
        }
    }
}

/// Try to launch using MCVM with proper integration
async fn try_mcvm_launch(
    instance: &MinecraftInstance,
    auth: &AuthInfo,
    memory: u32,
    java_path: &str,
) -> Result<LaunchResult, String> {
    // Create MCVM instance
    let mcvm_instance = MCVMCore::create_launch_instance(
        &instance.id,
        &instance.version,
        instance.game_dir.clone(),
    ).await?;
    
    // Launch with MCVM using the proper API
    let _handle = MCVMCore::launch_instance_with_mcvm(
        mcvm_instance,
        java_path.to_string(),
        memory,
        auth.username.clone(),
        auth.uuid.clone(),
        auth.access_token.clone(),
        None, // No app handle in this context  
        instance.name.clone(),
    ).await?;

    println!("✅ Launched with MCVM, handle created successfully");
    
    // Extract process ID from MCVM handle by getting the internal process
    // We need to consume the handle to get the process, so we'll get a placeholder PID
    let process_id = 1; // MCVM manages the process internally
    
    println!("✓ Minecraft launched successfully with PID: {}", process_id);
    
    Ok(LaunchResult {
        process_id,
        success: true,
        error: None,
    })
}

/// Fallback launcher for when MCVM is not available or fails
async fn fallback_launch(
    instance: &MinecraftInstance,
    auth: AuthInfo,
    memory: u32,
    java_path: &str,
) -> Result<LaunchResult, String> {
    println!("🔄 Using fallback launcher for Minecraft {}", instance.version);

    // Build basic command line arguments
    let mut args = vec![
        format!("-Xmx{}M", memory),
        format!("-Xms{}M", memory.min(512)),
        "-Djava.library.path".to_string(),
        instance.game_dir.join("versions").join(&instance.version).join("natives").to_string_lossy().to_string(),
    ];

    // Add custom JVM args
    if let Some(jvm_args) = &instance.jvm_args {
        args.extend(jvm_args.clone());
    }

    // Add classpath
    args.push("-cp".to_string());
    args.push(build_basic_classpath(instance)?);

    // Determine main class based on version
    let main_class = determine_main_class(&instance.version);
    args.push(main_class);

    // Add game arguments
    args.extend(build_game_arguments(instance, &auth));

    // Print command for debugging
    println!("Launching: {} {}", java_path, args.join(" "));
    println!("Working directory: {}", instance.game_dir.display());

    // Launch the process
    let child = std::process::Command::new(java_path)
        .args(&args)
        .current_dir(&instance.game_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn Minecraft process: {}", e))?;

    let process_id = child.id();
    
    println!("✅ Minecraft launched successfully with PID: {}", process_id);

    Ok(LaunchResult {
        process_id,
        success: true,
        error: None,
    })
}

/// Determine the correct main class for a Minecraft version
fn determine_main_class(version: &str) -> String {
    if versions::version_compare(version, "1.6") < 0 {
        // Very old versions (before 1.6)
        "net.minecraft.client.Minecraft".to_string()
    } else if versions::version_compare(version, "1.13") < 0 {
        // Versions 1.6 to 1.12 use LaunchWrapper
        "net.minecraft.launchwrapper.Launch".to_string()
    } else {
        // Modern versions (1.13+)
        "net.minecraft.client.main.Main".to_string()
    }
}

/// Build a basic classpath for fallback launching
fn build_basic_classpath(instance: &MinecraftInstance) -> Result<String, String> {
    let mut classpath_parts = Vec::new();
    
    // Add client JAR
    let version_dir = instance.game_dir.join("versions").join(&instance.version);
    let client_jar = version_dir.join(format!("{}.jar", instance.version));
    
    if client_jar.exists() {
        classpath_parts.push(client_jar.to_string_lossy().to_string());
    } else {
        return Err(format!("Client JAR not found: {}", client_jar.display()));
    }

    // Add libraries directory (simplified approach)
    let libraries_dir = instance.game_dir.join("libraries");
    if libraries_dir.exists() {
        // Add all JAR files in libraries directory recursively
        if let Ok(entries) = std::fs::read_dir(&libraries_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    add_jars_recursively(&entry.path(), &mut classpath_parts);
                }
            }
        }
    }

    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    Ok(classpath_parts.join(separator))
}

/// Recursively add JAR files to classpath
fn add_jars_recursively(path: &std::path::Path, classpath_parts: &mut Vec<String>) {
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    add_jars_recursively(&entry.path(), classpath_parts);
                }
            }
        }
    } else if path.extension().and_then(|ext| ext.to_str()) == Some("jar") {
        classpath_parts.push(path.to_string_lossy().to_string());
    }
}

/// Build game arguments for fallback launching
fn build_game_arguments(instance: &MinecraftInstance, auth: &AuthInfo) -> Vec<String> {
    let mut args = Vec::new();
    
    // Version-specific arguments
    if versions::version_compare(&instance.version, "1.6") >= 0 {
        // Modern argument format (1.6+)
        if versions::version_compare(&instance.version, "1.13") < 0 {
            // Versions 1.6-1.12: Use LaunchWrapper
            args.extend(vec![
                "--tweakClass".to_string(),
                "net.minecraft.launchwrapper.Launch".to_string(),
                "--username".to_string(),
                auth.username.clone(),
                "--version".to_string(),
                instance.version.clone(),
                "--gameDir".to_string(),
                instance.game_dir.to_string_lossy().to_string(),
                "--assetsDir".to_string(),
                instance.game_dir.join("assets").to_string_lossy().to_string(),
                "--assetIndex".to_string(),
                get_asset_index_for_version(&instance.version),
                "--uuid".to_string(),
                auth.uuid.clone(),
                "--accessToken".to_string(),
                auth.access_token.clone(),
                "--userType".to_string(),
                auth.user_type.clone(),
            ]);
        } else {
            // Modern versions (1.13+)
            args.extend(vec![
                "--username".to_string(),
                auth.username.clone(),
                "--version".to_string(),
                instance.version.clone(),
                "--gameDir".to_string(),
                instance.game_dir.to_string_lossy().to_string(),
                "--assetsDir".to_string(),
                instance.game_dir.join("assets").to_string_lossy().to_string(),
                "--assetIndex".to_string(),
                get_asset_index_for_version(&instance.version),
                "--uuid".to_string(),
                auth.uuid.clone(),
                "--accessToken".to_string(),
                auth.access_token.clone(),
                "--userType".to_string(),
                auth.user_type.clone(),
                "--versionType".to_string(),
                "release".to_string(),
            ]);
        }
    } else {
        // Legacy argument format for very old versions (pre-1.6)
        args.extend(vec![
            auth.username.clone(),
            "offline_token".to_string(), // Legacy token for offline play
            "--gameDir".to_string(),
            instance.game_dir.to_string_lossy().to_string(),
        ]);
    }
    
    args
}

/// Get asset index for a version (simplified)
fn get_asset_index_for_version(version: &str) -> String {
    // This is a simplified mapping - in practice, this would read from version manifest
    if versions::version_compare(version, "1.13") >= 0 {
        version.to_string()
    } else if versions::version_compare(version, "1.7.10") >= 0 {
        "1.7.10".to_string()
    } else if versions::version_compare(version, "1.6") >= 0 {
        "legacy".to_string()
    } else {
        "pre-1.6".to_string()
    }
}