//! Minecraft Launcher using MCVM
//! 
//! This module handles the actual launching of Minecraft instances using MCVM
//! while maintaining ChaiLauncher's Java management and API compatibility.

use super::{MinecraftInstance, AuthInfo, LaunchResult, MCVMCore, versions};

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

    // Launch using MCVM - no fallback since it doesn't work
    let result = try_mcvm_launch(instance, &auth, memory, &java_path).await?;
    println!("✅ Minecraft launched successfully with MCVM (PID: {})", result.process_id);
    Ok(result)
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
    let child = MCVMCore::launch_instance_with_mcvm(
        mcvm_instance,
        java_path.to_string(),
        memory,
        auth.username.clone(),
        auth.uuid.clone(),
        auth.access_token.clone(),
        None, // No app handle in this context
        instance.name.clone(),
    ).await?;

    let process_id = child.id();
    println!("✓ Minecraft launched successfully with PID: {}", process_id);

    Ok(LaunchResult {
        process_id,
        success: true,
        error: None,
        child: Some(child),
    })
}


