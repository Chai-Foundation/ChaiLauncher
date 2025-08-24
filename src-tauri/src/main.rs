// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod minecraft;
mod types;
mod commands;
mod storage;
mod modpack;
mod auth;

use tauri::Manager;
use reqwest;

#[tauri::command]
async fn fetch_news() -> Result<String, String> {
    let url = "https://net-secondary.web.minecraft-services.net/api/v1.0/en-us/search?pageSize=24&sortType=Recent&category=News&newsOnly=true";
    let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let body = resp.text().await.map_err(|e| e.to_string())?;
    Ok(body)
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
            fetch_news
        ])
        .setup(|app| {
            // Initialize MCVM integration
            tauri::async_runtime::spawn(async {
                if let Err(e) = minecraft::initialize_minecraft().await {
                    eprintln!("⚠️  Failed to initialize MCVM: {}", e);
                    eprintln!("   ChaiLauncher will continue with fallback systems");
                }
            });
            
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}