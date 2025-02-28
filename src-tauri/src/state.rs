use std::sync::Mutex;
use serde::Serialize;

#[derive(Serialize)]
pub struct AppState {
    pub image_paths: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            image_paths: Mutex::new(Vec::new()),
        }
    }
}
