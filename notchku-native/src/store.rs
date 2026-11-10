// Local JSON storage: settings + todos, no network, no account.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub pomodoro_focus_min: u32,
    pub pomodoro_break_min: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            city: "Jakarta".into(),
            latitude: -6.2,
            longitude: 106.8,
            pomodoro_focus_min: 25,
            pomodoro_break_min: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Todo {
    pub text: String,
    pub done: bool,
}

pub fn app_data_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("NotchKu");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("notchku");
    }
    PathBuf::from(".notchku")
}

fn file_path(name: &str) -> PathBuf {
    let dir = app_data_dir();
    let _ = fs::create_dir_all(&dir);
    dir.join(name)
}

pub fn load_settings() -> Settings {
    fs::read_to_string(file_path("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn save_settings(settings: &Settings) {
    let _ = fs::write(
        file_path("settings.json"),
        serde_json::to_string_pretty(settings).unwrap_or_default(),
    );
}

pub fn load_todos() -> Vec<Todo> {
    fs::read_to_string(file_path("todos.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| {
            vec![
                Todo { text: "Coba NotchKu — tekan Alt+N".into(), done: false },
                Todo { text: "Atur jadwal di tab Kalender".into(), done: false },
            ]
        })
}

pub fn save_todos(todos: &[Todo]) {
    let _ = fs::write(
        file_path("todos.json"),
        serde_json::to_string_pretty(todos).unwrap_or_default(),
    );
}
