// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod minecraft;
mod types;
mod commands;
mod launchers;
mod storage;
mod installer;
mod modpack;
mod auth;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            minecraft::get_minecraft_versions,
            minecraft::create_instance,
            minecraft::launch_minecraft,
            minecraft::get_bundled_java_path,
            minecraft::download_and_install_java,
            minecraft::get_java_installations,
            minecraft::validate_java_installation,
            minecraft::get_system_memory,
            minecraft::download_minecraft_assets,
            minecraft::detect_gdlauncher_instances,
            minecraft::detect_all_external_instances,
            minecraft::launch_instance,
            minecraft::launch_external_instance,
            minecraft::load_instances,
            minecraft::save_instance,
            minecraft::delete_instance,
            minecraft::update_instance,
            minecraft::get_launcher_settings,
            minecraft::update_launcher_settings,
            minecraft::install_minecraft_version,
            minecraft::backup_instance,
            minecraft::restore_instance,
            minecraft::refresh_instance_sizes,
            modpack::search_modpacks,
            modpack::install_modpack,
            commands::open_folder,
            commands::open_instance_folder,
            commands::set_auth_token,
            commands::get_auth_token,
            commands::clear_auth_token,
            auth::start_microsoft_oauth,
            auth::start_oauth_with_server,
            auth::complete_microsoft_oauth,
            auth::get_stored_accounts,
            auth::refresh_minecraft_token,
            auth::remove_minecraft_account
        ])
        .setup(|app| {
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