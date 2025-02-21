use std::path::Path;
use std::sync::Mutex;
use tauri::command;
use tauri::Manager;
use tauri::State;
use tauri_plugin_cli::CliExt;

struct AppState {
    image_names: Mutex<Vec<String>>,
}

fn add_image(state: State<AppState>, file_name: String) {
    state.image_names.lock().unwrap().push(file_name);
}

#[command]
fn get_image_names(state: State<AppState>) -> Vec<String> {
    state.image_names.lock().unwrap().clone()
}

#[command]
fn quit_app(window: tauri::Window) {
    window.close().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        image_names: Mutex::new(Vec::new()),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            let cache_dir = app.path().app_cache_dir().unwrap();
            match app.cli().matches() {
                Ok(matches) => {
                    if let Some(arg) = matches.args.get("img") {
                        if let Some(arr) = arg.value.as_array() {
                            let mut count = 0;
                            for item in arr {
                                if let Some(path) = item.as_str() {
                                    let src_file = Path::new(path);
                                    if src_file.is_file() && src_file.exists() {
                                        let extension = src_file.extension().unwrap().to_str().unwrap();
                                        let file_name = format!("{}.{}", count, extension);
                                        let dest_file = cache_dir.join(&file_name);
                                        if extension == "png"
                                            || extension == "jpg"
                                            || extension == "jpeg"
                                            || extension == "bmp"
                                        {
                                            std::fs::copy(src_file, dest_file).unwrap();
                                            add_image(
                                                app.try_state().unwrap(),
                                                file_name.to_string(),
                                            );
                                            count += 1;
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
        .invoke_handler(tauri::generate_handler![quit_app, get_image_names,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
