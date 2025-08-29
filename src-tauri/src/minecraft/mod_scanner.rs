//! Mod JAR Scanner for Java Version Requirements
//! 
//! This module scans installed mods to determine their Java version requirements
//! by reading metadata files from within the mod JAR files.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;
use std::fs::File;
use std::io::Read;
use anyhow::{Result, Context};

/// Java version requirement from a mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModJavaRequirement {
    pub mod_name: String,
    pub mod_id: String,
    pub mod_version: String,
    pub java_requirement: String, // e.g., ">=17", "21", ">=8"
    pub loader_type: ModLoaderType,
    pub resolved_java_version: Option<u32>, // Parsed minimum version
}

/// Supported mod loader types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModLoaderType {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
    Unknown,
}

/// Instance Java requirements analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceJavaAnalysis {
    pub instance_path: PathBuf,
    pub mod_requirements: Vec<ModJavaRequirement>,
    pub minimum_java_version: Option<u32>,
    pub recommended_java_version: u32,
    pub conflicting_requirements: Vec<String>,
    pub analysis_date: String,
}

/// Fabric mod metadata structure
#[derive(Debug, Deserialize)]
struct FabricModJson {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    id: String,
    version: String,
    name: Option<String>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    depends: Option<HashMap<String, String>>,
    suggests: Option<HashMap<String, String>>,
    breaks: Option<HashMap<String, String>>,
    conflicts: Option<HashMap<String, String>>,
}

/// Forge mod metadata structure (mods.toml)
#[derive(Debug, Deserialize)]
struct ForgeModsToml {
    #[serde(rename = "modLoader")]
    mod_loader: Option<String>,
    #[serde(rename = "loaderVersion")]
    loader_version: Option<String>,
    license: Option<String>,
    #[serde(rename = "issueTrackerURL")]
    issue_tracker_url: Option<String>,
    mods: Option<Vec<ForgeMod>>,
    dependencies: Option<HashMap<String, Vec<ForgeDependency>>>,
}

#[derive(Debug, Deserialize)]
struct ForgeMod {
    #[serde(rename = "modId")]
    mod_id: String,
    version: String,
    #[serde(rename = "displayName")]
    display_name: String,
    description: Option<String>,
    authors: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ForgeDependency {
    #[serde(rename = "modId")]
    mod_id: String,
    mandatory: bool,
    #[serde(rename = "versionRange")]
    version_range: String,
    ordering: Option<String>,
    side: Option<String>,
}

/// Mod JAR scanner
pub struct ModJarScanner {
    instance_path: PathBuf,
}

impl ModJarScanner {
    pub fn new(instance_path: PathBuf) -> Self {
        Self { instance_path }
    }
    
    /// Scan all mods in the instance and determine Java requirements
    pub async fn analyze_instance_java_requirements(&self) -> Result<InstanceJavaAnalysis> {
        let mods_dir = self.instance_path.join("mods");
        
        if !mods_dir.exists() {
            return Ok(InstanceJavaAnalysis {
                instance_path: self.instance_path.clone(),
                mod_requirements: vec![],
                minimum_java_version: None,
                recommended_java_version: 17, // Default to Java 17
                conflicting_requirements: vec![],
                analysis_date: chrono::Utc::now().to_rfc3339(),
            });
        }
        
        let mut mod_requirements = Vec::new();
        let mut entries = tokio::fs::read_dir(&mods_dir).await
            .context("Failed to read mods directory")?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Only process .jar files
            if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                if let Ok(requirement) = self.scan_mod_jar(&path).await {
                    mod_requirements.push(requirement);
                }
            }
        }
        
        // Analyze requirements and find conflicts
        let (min_java, recommended_java, conflicts) = self.resolve_java_requirements(&mod_requirements);
        
        Ok(InstanceJavaAnalysis {
            instance_path: self.instance_path.clone(),
            mod_requirements,
            minimum_java_version: min_java,
            recommended_java_version: recommended_java,
            conflicting_requirements: conflicts,
            analysis_date: chrono::Utc::now().to_rfc3339(),
        })
    }
    
    /// Scan a single mod JAR file for Java requirements
    pub async fn scan_mod_jar(&self, jar_path: &Path) -> Result<ModJavaRequirement> {
        let file = File::open(jar_path)
            .with_context(|| format!("Failed to open JAR file: {}", jar_path.display()))?;
        
        let mut archive = ZipArchive::new(file)
            .with_context(|| format!("Failed to read ZIP archive: {}", jar_path.display()))?;
        
        // Try Fabric first (fabric.mod.json)
        if let Ok(fabric_req) = self.scan_fabric_mod(&mut archive, jar_path).await {
            return Ok(fabric_req);
        }
        
        // Try Forge (META-INF/mods.toml)
        if let Ok(forge_req) = self.scan_forge_mod(&mut archive, jar_path).await {
            return Ok(forge_req);
        }
        
        // Try Quilt (quilt.mod.json)
        if let Ok(quilt_req) = self.scan_quilt_mod(&mut archive, jar_path).await {
            return Ok(quilt_req);
        }
        
        // Default fallback - assume Java 8 compatible
        Ok(ModJavaRequirement {
            mod_name: jar_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            mod_id: jar_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            mod_version: "unknown".to_string(),
            java_requirement: ">=8".to_string(),
            loader_type: ModLoaderType::Unknown,
            resolved_java_version: Some(8),
        })
    }
    
    /// Scan Fabric mod metadata
    async fn scan_fabric_mod(&self, archive: &mut ZipArchive<File>, jar_path: &Path) -> Result<ModJavaRequirement> {
        let mut fabric_json = archive.by_name("fabric.mod.json")
            .context("No fabric.mod.json found")?;
        
        let mut contents = String::new();
        fabric_json.read_to_string(&mut contents)
            .context("Failed to read fabric.mod.json")?;
        
        let fabric_mod: FabricModJson = serde_json::from_str(&contents)
            .context("Failed to parse fabric.mod.json")?;
        
        // Extract Java requirement from dependencies
        let java_requirement = fabric_mod.depends
            .as_ref()
            .and_then(|deps| deps.get("java"))
            .unwrap_or(&">=8".to_string())
            .clone();
        
        let resolved_version = Self::parse_java_version_requirement(&java_requirement)?;
        
        Ok(ModJavaRequirement {
            mod_name: fabric_mod.name.unwrap_or_else(|| fabric_mod.id.clone()),
            mod_id: fabric_mod.id,
            mod_version: fabric_mod.version,
            java_requirement,
            loader_type: ModLoaderType::Fabric,
            resolved_java_version: Some(resolved_version),
        })
    }
    
    /// Scan Forge mod metadata
    async fn scan_forge_mod(&self, archive: &mut ZipArchive<File>, jar_path: &Path) -> Result<ModJavaRequirement> {
        let mut mods_toml = archive.by_name("META-INF/mods.toml")
            .context("No META-INF/mods.toml found")?;
        
        let mut contents = String::new();
        mods_toml.read_to_string(&mut contents)
            .context("Failed to read mods.toml")?;
        
        let forge_mod: ForgeModsToml = toml::from_str(&contents)
            .context("Failed to parse mods.toml")?;
        
        // For Forge, we need to check the loader version and dependencies
        // Most Forge mods don't explicitly specify Java versions, so we infer from Forge version
        let java_version = self.infer_java_from_forge_version(
            forge_mod.loader_version.as_deref()
        );
        
        let first_mod = forge_mod.mods
            .as_ref()
            .and_then(|mods| mods.first());
        
        let mod_name = first_mod
            .map(|m| m.display_name.clone())
            .unwrap_or_else(|| jar_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string());
        
        let mod_id = first_mod
            .map(|m| m.mod_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let mod_version = first_mod
            .map(|m| m.version.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(ModJavaRequirement {
            mod_name,
            mod_id,
            mod_version,
            java_requirement: format!(">={}", java_version),
            loader_type: ModLoaderType::Forge,
            resolved_java_version: Some(java_version),
        })
    }
    
    /// Scan Quilt mod metadata (similar to Fabric but quilt.mod.json)
    async fn scan_quilt_mod(&self, archive: &mut ZipArchive<File>, jar_path: &Path) -> Result<ModJavaRequirement> {
        let mut quilt_json = archive.by_name("quilt.mod.json")
            .context("No quilt.mod.json found")?;
        
        let mut contents = String::new();
        quilt_json.read_to_string(&mut contents)
            .context("Failed to read quilt.mod.json")?;
        
        // Quilt uses a similar structure to Fabric
        let quilt_mod: FabricModJson = serde_json::from_str(&contents)
            .context("Failed to parse quilt.mod.json")?;
        
        let java_requirement = quilt_mod.depends
            .as_ref()
            .and_then(|deps| deps.get("java"))
            .unwrap_or(&">=8".to_string())
            .clone();
        
        let resolved_version = Self::parse_java_version_requirement(&java_requirement)?;
        
        Ok(ModJavaRequirement {
            mod_name: quilt_mod.name.unwrap_or_else(|| quilt_mod.id.clone()),
            mod_id: quilt_mod.id,
            mod_version: quilt_mod.version,
            java_requirement,
            loader_type: ModLoaderType::Quilt,
            resolved_java_version: Some(resolved_version),
        })
    }
    
    /// Parse Java version requirement string (e.g., ">=17", "21", ">=8")
    fn parse_java_version_requirement(requirement: &str) -> Result<u32> {
        let requirement = requirement.trim();
        
        // Handle different formats
        if requirement.starts_with(">=") {
            requirement[2..].parse()
                .context("Failed to parse Java version number")
        } else if requirement.starts_with(">") {
            let version: u32 = requirement[1..].parse()
                .context("Failed to parse Java version number")?;
            Ok(version + 1) // If ">17", we need at least 18
        } else if requirement.starts_with("<=") {
            requirement[2..].parse()
                .context("Failed to parse Java version number")
        } else if requirement.starts_with("<") {
            let version: u32 = requirement[1..].parse()
                .context("Failed to parse Java version number")?;
            Ok(version - 1) // If "<21", we can use up to 20
        } else if requirement.starts_with("=") {
            requirement[1..].parse()
                .context("Failed to parse Java version number")
        } else {
            // Assume it's just a number
            requirement.parse()
                .context("Failed to parse Java version number")
        }
    }
    
    /// Infer Java version from Forge loader version
    fn infer_java_from_forge_version(&self, forge_version: Option<&str>) -> u32 {
        match forge_version {
            Some(version) => {
                // Parse Forge version to determine Java requirement
                if version.starts_with("47.") || version.starts_with("48.") || version.starts_with("49.") {
                    17 // Forge 47+ typically requires Java 17
                } else if version.starts_with("40.") || version.starts_with("41.") || version.starts_with("42.") || version.starts_with("43.") {
                    17 // Modern 1.19+ Forge often needs Java 17
                } else if version.starts_with("36.") || version.starts_with("37.") || version.starts_with("38.") || version.starts_with("39.") {
                    8 // Older Forge versions work with Java 8
                } else {
                    17 // Default to Java 17 for unknown modern Forge versions
                }
            }
            None => 8, // Default to Java 8 if no version specified
        }
    }
    
    /// Resolve Java requirements from all mods
    fn resolve_java_requirements(&self, requirements: &[ModJavaRequirement]) -> (Option<u32>, u32, Vec<String>) {
        if requirements.is_empty() {
            return (None, 17, vec![]);
        }
        
        let mut java_versions: Vec<u32> = requirements
            .iter()
            .filter_map(|req| req.resolved_java_version)
            .collect();
        
        if java_versions.is_empty() {
            return (None, 17, vec![]);
        }
        
        java_versions.sort();
        java_versions.dedup();
        
        // The minimum Java version is the highest requirement
        let minimum_java = *java_versions.iter().max().unwrap_or(&17);
        
        // Check for conflicts (mods that require different versions)
        let mut conflicts = Vec::new();
        let unique_versions: std::collections::HashSet<u32> = java_versions.into_iter().collect();
        
        if unique_versions.len() > 1 {
            let versions_str: Vec<String> = unique_versions.iter().map(|v| v.to_string()).collect();
            conflicts.push(format!("Conflicting Java versions required: {}", versions_str.join(", ")));
        }
        
        // Recommend the minimum viable version
        let recommended = if minimum_java < 17 { 17 } else { minimum_java };
        
        (Some(minimum_java), recommended, conflicts)
    }
}