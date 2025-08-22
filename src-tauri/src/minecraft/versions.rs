use std::path::PathBuf;
use serde_json::Value;

/// Find Java executable in extracted directory (handles nested JDK directories)
fn find_java_executable(java_dir: &PathBuf) -> Result<PathBuf, String> {
    use std::fs;
    
    #[cfg(target_os = "windows")]
    let java_exe_name = "java.exe";
    #[cfg(not(target_os = "windows"))]
    let java_exe_name = "java";
    
    // First try direct path (java_dir/bin/java.exe)
    let direct_path = java_dir.join("bin").join(java_exe_name);
    if direct_path.exists() {
        return Ok(direct_path);
    }
    
    // Search for nested JDK directories (like jdk-17.0.16+8, jdk8u462-b08)
    let entries = fs::read_dir(java_dir)
        .map_err(|e| format!("Failed to read Java directory: {}", e))?;
        
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let potential_java = path.join("bin").join(java_exe_name);
            if potential_java.exists() {
                return Ok(potential_java);
            }
        }
    }
    
    Err(format!("Java executable not found in {}", java_dir.display()))
}

/// Get Java path for a specific Java version
pub async fn get_java_for_version(major_version: u32) -> Result<String, String> {
    // First try bundled Java installations
    let launcher_dir = crate::storage::get_launcher_dir();
    let java_dir = launcher_dir.join("java").join(format!("java{}", major_version));
    
    // Use find_java_executable to handle nested directories like jdk8u462-b08
    if java_dir.exists() {
        match find_java_executable(&java_dir) {
            Ok(java_exe) => return Ok(java_exe.to_string_lossy().to_string()),
            Err(_) => {} // Continue to system Java search
        }
    }
    
    // Try system Java installation
    #[cfg(target_os = "windows")]
    let system_java_paths = vec![
        format!("C:\\Program Files\\Java\\jdk-{}\\bin\\java.exe", major_version),
        format!("C:\\Program Files\\Java\\jre{}\\bin\\java.exe", major_version),
        format!("C:\\Program Files (x86)\\Java\\jdk-{}\\bin\\java.exe", major_version),
        format!("C:\\Program Files (x86)\\Java\\jre{}\\bin\\java.exe", major_version),
        "C:\\Program Files\\Java\\jdk\\bin\\java.exe".to_string(),
        "C:\\Program Files (x86)\\Java\\jdk\\bin\\java.exe".to_string(),
    ];
    
    #[cfg(not(target_os = "windows"))]
    let system_java_paths = vec![
        format!("/usr/lib/jvm/java-{}-openjdk/bin/java", major_version),
        format!("/usr/lib/jvm/java-{}-oracle/bin/java", major_version),
        "/usr/bin/java".to_string(),
        "/usr/local/bin/java".to_string(),
    ];
    
    for path in system_java_paths {
        if tokio::fs::metadata(&path).await.is_ok() {
            // Verify this is the correct Java version
            if let Ok(output) = tokio::process::Command::new(&path)
                .arg("-version")
                .output()
                .await
            {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains(&format!("\"{}.", major_version)) || 
                   stderr.contains(&format!("version \"{}", major_version)) {
                    return Ok(path);
                }
            }
        }
    }
    
    // Check if system java is available and compatible
    #[cfg(target_os = "windows")]
    let java_cmd = "java.exe";
    #[cfg(not(target_os = "windows"))]
    let java_cmd = "java";
    
    if let Ok(output) = tokio::process::Command::new(java_cmd)
        .arg("-version")
        .output()
        .await
    {
        if output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("System Java version output: {}", stderr);
            if stderr.contains(&format!("\"{}.", major_version)) || 
               stderr.contains(&format!("version \"{}", major_version)) {
                println!("✓ Found compatible system Java {} for requirement {}", stderr.lines().next().unwrap_or(""), major_version);
                return Ok(java_cmd.to_string());
            } else if major_version == 8 && (stderr.contains("\"1.8") || stderr.contains("version \"8")) {
                // Java 8 reports as 1.8.x
                println!("✓ Found compatible Java 8 (1.8.x) for requirement {}", major_version);
                return Ok(java_cmd.to_string());
            } else {
                println!("System Java version {} does not match requirement {}", stderr.lines().next().unwrap_or("unknown"), major_version);
            }
        }
    } else {
        println!("No system Java found in PATH");
    }
    
    Err(format!("Java {} not found. Please install Java {} first.", major_version, major_version))
}

/// Parse a Minecraft version string and extract major.minor
pub fn parse_version(version: &str) -> (u32, u32) {
    if let Some(stripped) = version.strip_prefix("1.") {
        let parts: Vec<&str> = stripped.split('.').collect();
        if let Ok(minor) = parts[0].parse::<u32>() {
            return (1, minor);
        }
    }
    
    // Handle non-1.x versions or parsing failures
    (0, 0)
}

/// Check if a version is a snapshot
pub fn is_snapshot(version: &str) -> bool {
    version.contains("w") || version.contains("-") || version.contains("pre") || version.contains("rc")
}

/// Get the asset index name for a version (some versions use different asset indexes)
pub fn get_asset_index(version: &str) -> String {
    let (major, minor) = parse_version(version);
    
    match (major, minor) {
        (1, 0..=5) => "pre-1.6".to_string(),
        (1, 6..=7) => format!("1.{}", minor),
        _ => version.to_string(),
    }
}

/// Determine if a version needs special handling for arguments
pub fn uses_legacy_arguments(version: &str) -> bool {
    let (major, minor) = parse_version(version);
    match (major, minor) {
        (1, 0..=12) => true,
        _ => false,
    }
}

/// Get the main class for a Minecraft version
pub fn get_main_class_for_version(version: &str) -> String {
    let (major, minor) = parse_version(version);
    
    match (major, minor) {
        // Very old versions use launchwrapper
        (1, 0..=5) => "net.minecraft.launchwrapper.Launch".to_string(),
        // Most versions use the standard main class
        _ => "net.minecraft.client.main.Main".to_string(),
    }
}

/// Get appropriate Java version for a Minecraft version
pub fn get_required_java_version(version: &str) -> u32 {
    let (major, minor) = parse_version(version);
    
    match (major, minor) {
        (1, 0..=16) => 8,   // Java 8 for old versions
        (1, 17..=20) => 17, // Java 17 for newer versions
        _ => 17, // Default to Java 17
    }
}

/// Load version manifest JSON if it exists
pub async fn load_version_manifest(instance_path: &PathBuf, version: &str) -> Result<Option<Value>, String> {
    let version_json_path = instance_path
        .join("versions")
        .join(version)
        .join(format!("{}.json", version));
    
    if !version_json_path.exists() {
        return Ok(None);
    }
    
    match tokio::fs::read_to_string(&version_json_path).await {
        Ok(json_str) => {
            match serde_json::from_str::<Value>(&json_str) {
                Ok(json) => Ok(Some(json)),
                Err(e) => Err(format!("Failed to parse version JSON: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to read version JSON: {}", e)),
    }
}