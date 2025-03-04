mod commands;
mod state;

use state::AppState;
use std::path::Path;
use tauri::Manager;
use tauri_plugin_cli::CliExt;
use tauri_plugin_log::Target;
use tauri_plugin_log::TargetKind;

const SUPPORTED_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "bmp", "webp"];

fn is_supported_image(path: &Path) -> bool {
    path.is_file()
        && path.exists()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
}

fn handle_cli_args(app: &tauri::App) {
    let matches = app.cli().matches().unwrap();

    if let Some(arg) = matches.args.get("img") {
        if let Some(paths) = arg.value.as_array() {
            for path in paths.iter().filter_map(|p| p.as_str()) {
                let src_file = Path::new(path);
                if is_supported_image(src_file) {
                    commands::add_image(
                        app.try_state().unwrap(),
                        src_file.to_string_lossy().to_string(),
                    );
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            handle_cli_args(app);
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_image_paths,
            commands::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
