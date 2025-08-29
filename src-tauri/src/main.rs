// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod minecraft;
mod types;
mod commands;
mod storage;
mod modpack;
mod mods;
mod auth;
mod docker;

use reqwest;
use tauri::Manager;

#[tauri::command]
async fn fetch_news() -> Result<String, String> {
    let url = "https://net-secondary.web.minecraft-services.net/api/v1.0/en-us/search?pageSize=24&sortType=Recent&category=News&newsOnly=true";
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let body = resp.text().await.map_err(|e| e.to_string())?;
    Ok(body)
}

#[tauri::command]
async fn fetch_news_page(page: u32) -> Result<String, String> {
    let url = format!(
        "https://net-secondary.web.minecraft-services.net/api/v1.0/en-us/search?page={}&pageSize=24&sortType=Recent&category=News&newsOnly=true",
        page
    );
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let body = resp.text().await.map_err(|e| e.to_string())?;
    Ok(body)
}

// Expose the app version as a Tauri command
#[tauri::command]
fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            minecraft::commands::get_minecraft_versions,
            minecraft::commands::create_instance,
            minecraft::commands::launch_minecraft,
            minecraft::commands::get_bundled_java_path,
            minecraft::commands::get_bundled_java_path_for_version,
            minecraft::commands::download_and_install_java,
            minecraft::commands::download_and_install_java8,
            minecraft::commands::download_and_install_java17,
            minecraft::commands::download_and_install_java_version,
            minecraft::commands::download_and_install_both_java,
            minecraft::commands::get_java_installations,
            minecraft::commands::get_required_java_version,
            minecraft::commands::get_java_for_minecraft_version,
            minecraft::commands::is_java_version_installed,
            minecraft::commands::validate_java_installation,
            minecraft::commands::get_system_memory,
            minecraft::commands::download_minecraft_assets,
            minecraft::commands::detect_gdlauncher_instances,
            minecraft::commands::detect_all_external_instances,
            minecraft::commands::launch_instance,
            minecraft::commands::launch_external_instance,
            minecraft::commands::load_instances,
            minecraft::commands::import_orphaned_instances,
            minecraft::commands::save_instance,
            minecraft::commands::delete_instance,
            minecraft::commands::update_instance,
            minecraft::commands::get_launcher_settings,
            minecraft::commands::update_launcher_settings,
            minecraft::commands::install_minecraft_version,
            minecraft::commands::backup_instance,
            minecraft::commands::restore_instance,
            minecraft::commands::refresh_instance_sizes,
            modpack::search_modpacks,
            modpack::install_modpack,
            modpack::create_modpack,
            mods::commands::search_mods,
            mods::commands::get_mod_details,
            mods::commands::install_mod,
            mods::commands::uninstall_mod,
            mods::commands::update_mod,
            mods::commands::get_installed_mods,
            mods::commands::set_mod_enabled,
            mods::commands::check_mod_updates,
            mods::commands::get_mod_loader_versions,
            mods::commands::install_mod_loader,
            mods::commands::get_installed_mod_loader,
            mods::commands::get_featured_mods,
            mods::commands::get_mod_categories,
            commands::open_folder,
            commands::open_instance_folder,
            commands::set_auth_token,
            commands::get_auth_token,
            commands::clear_auth_token,
            commands::get_auth_status,
            auth::start_microsoft_oauth,
            auth::start_oauth_with_server,
            auth::complete_microsoft_oauth,
            auth::get_stored_accounts,
            auth::refresh_minecraft_token,
            auth::remove_minecraft_account,
            fetch_news,
            fetch_news_page,
            get_app_version,
            docker::commands::test_docker_connection,
            docker::commands::add_docker_connection,
            docker::commands::deploy_minecraft_server,
            docker::commands::start_server,
            docker::commands::stop_server,
            docker::commands::remove_server,
            docker::commands::get_servers,
            docker::commands::get_servers_for_instance,
            docker::commands::get_docker_connections,
            docker::commands::get_server_status,
            docker::commands::get_server_logs,
            docker::commands::execute_server_command,
            docker::commands::restart_server,
            docker::commands::backup_server,
            docker::commands::get_server_stats,
            minecraft::commands::analyze_instance_java_requirements,
            minecraft::commands::get_mod_java_requirements
        ])
        .setup(|app| {
            // Initialize Docker manager and MCVM concurrently
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Initialize Docker manager state with persistent storage
                match docker::DockerManager::new().await {
                    Ok(docker_manager) => {
                        app_handle.manage(docker::commands::DockerManagerState::new(docker_manager));
                        println!("✅ Docker manager initialized with persistent storage");
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to initialize Docker manager: {}", e);
                        eprintln!("   Docker functionality will be unavailable");
                    }
                }
                
                // Initialize MCVM integration
                if let Err(e) = minecraft::initialize_minecraft().await {
                    eprintln!("⚠️  Failed to initialize MCVM: {}", e);
                    eprintln!("   ChaiLauncher will continue with fallback systems");
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}