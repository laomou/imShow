use std::path::Path;
use std::sync::Mutex;
use tauri::command;
use tauri::Manager;
use tauri::State;
use tauri_plugin_cli::CliExt;

struct AppState {
    image_paths: Mutex<Vec<String>>,
}

fn add_image(state: State<AppState>, file_name: String) {
    state.image_paths.lock().unwrap().push(file_name);
}

#[command]
fn get_image_paths(state: State<AppState>) -> Vec<String> {
    state.image_paths.lock().unwrap().clone()
}

#[command]
fn quit_app(window: tauri::Window) {
    window.close().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        image_paths: Mutex::new(Vec::new()),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            match app.cli().matches() {
                Ok(matches) => {
                    if let Some(arg) = matches.args.get("img") {
                        if let Some(arr) = arg.value.as_array() {
                            for item in arr {
                                if let Some(path) = item.as_str() {
                                    let src_file = Path::new(path);
                                    if src_file.is_file() && src_file.exists() {
                                        let extension =
                                            src_file.extension().unwrap().to_str().unwrap();
                                        if extension == "png"
                                            || extension == "jpg"
                                            || extension == "jpeg"
                                            || extension == "bmp"
                                        {
                                            add_image(
                                                app.try_state().unwrap(),
                                                src_file.to_string_lossy().to_string(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }

            Ok(())
        })
        .manage(state)
        .invoke_handler(tauri::generate_handler![quit_app, get_image_paths,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
