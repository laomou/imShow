use crate::state::AppState;
use tauri::command;
use tauri::State;

pub fn add_image(state: State<AppState>, file_path: String) {
    state.image_paths.lock().unwrap().push(file_path);
}

#[command]
pub fn get_image_paths(state: State<AppState>) -> Vec<String> {
    let paths = state.image_paths.lock().unwrap().clone();
    paths
}

#[command]
pub fn quit_app(window: tauri::Window) {
    window.close().unwrap();
}
