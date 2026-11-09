// NotchKu — Rust backend (Tauri v2)
// Window notch: satu window frameless transparan di tengah atas,
// dua ukuran: tertutup (pil kecil) dan terbuka (panel).
// Clean-room MVP. Lihat ARCHITECTURE.md untuk desain lengkap.

use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};

pub const COLLAPSED_W: u32 = 220;
pub const COLLAPSED_H: u32 = 32;
pub const EXPANDED_W: u32 = 560;
pub const EXPANDED_H: u32 = 380;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub show_weather: bool,
    pub show_clock: bool,
    pub pomodoro_focus_min: u32,
    pub pomodoro_break_min: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            city: "Jakarta".into(),
            latitude: -6.2,
            longitude: 106.8,
            show_weather: true,
            show_clock: true,
            pomodoro_focus_min: 25,
            pomodoro_break_min: 5,
        }
    }
}

pub struct AppState {
    pub expanded: std::sync::Mutex<bool>,
}

fn app_data_file(app: &AppHandle, name: &str) -> std::path::PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("app data dir");
    fs::create_dir_all(&dir).ok();
    dir.join(name)
}

// --- Storage commands (dipanggil frontend via invoke) ---

#[tauri::command]
fn load_settings(app: AppHandle) -> Settings {
    let path = app_data_file(&app, "settings.json");
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = app_data_file(&app, "settings.json");
    let text = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_todos(app: AppHandle) -> String {
    let path = app_data_file(&app, "todos.json");
    fs::read_to_string(path).unwrap_or_else(|_| "[]".into())
}

#[tauri::command]
fn save_todos(app: AppHandle, todos_json: String) -> Result<(), String> {
    let path = app_data_file(&app, "todos.json");
    // Validasi dulu supaya file tidak rusak kalau frontend kirim sampah.
    let v: serde_json::Value =
        serde_json::from_str(&todos_json).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

// --- Notch window ---

#[tauri::command]
fn toggle_notch(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let win = app.get_webview_window("notch").ok_or("no notch window")?;
    let mut expanded = state.expanded.lock().map_err(|e| e.to_string())?;
    *expanded = !*expanded;
    resize_notch(&win, *expanded)?;
    // Beri tahu frontend supaya ia render state yang benar.
    let _ = win.emit("notch-state", *expanded);
    Ok(())
}

#[tauri::command]
fn set_notch_expanded(app: AppHandle, state: State<AppState>, open: bool) -> Result<(), String> {
    let win = app.get_webview_window("notch").ok_or("no notch window")?;
    let mut expanded = state.expanded.lock().map_err(|e| e.to_string())?;
    if *expanded == open {
        return Ok(());
    }
    *expanded = open;
    resize_notch(&win, open)?;
    let _ = win.emit("notch-state", open);
    Ok(())
}

fn resize_notch(win: &tauri::WebviewWindow, expanded: bool) -> Result<(), String> {
    let monitor = win
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("no monitor")?;
    let msize = monitor.size();
    let (w, h) = if expanded {
        (EXPANDED_W, EXPANDED_H)
    } else {
        (COLLAPSED_W, COLLAPSED_H)
    };
    let x = (msize.width.saturating_sub(w)) / 2;
    let y = 0i32; // menempel di tepi atas
    win.set_size(PhysicalSize::new(w, h)).map_err(|e| e.to_string())?;
    win.set_position(PhysicalPosition::new(x as i32, y))
        .map_err(|e| e.to_string())?;
    // Saat tertutup, klik di luar pil harus tembus ke window belakang.
    // Tauri v2: ignore_cursor_events(true) saat collapsed.
    win.set_ignore_cursor_events(!expanded)
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Cuaca (Open-Meteo, tanpa API key) ---
#[tauri::command]
async fn fetch_weather(latitude: f64, longitude: f64) -> Result<String, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,relative_humidity_2m",
        latitude, longitude
    );
    let body = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    Ok(body)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            expanded: std::sync::Mutex::new(false),
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Posisi awal: pil tertutup di tengah atas layar utama.
            if let Some(win) = app.get_webview_window("notch") {
                resize_notch(&win, false)?;
                // Shortcut global Alt+N — cara utama membuka notch.
                let app_handle = app.handle().clone();
                app.global_shortcut().on_shortcut("alt+n", move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let state = app_handle.state::<AppState>();
                        let _ = toggle_notch(app_handle.clone(), state);
                    }
                })?;
                let _ = Code::KeyN;
                let _ = Modifiers::ALT;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            load_todos,
            save_todos,
            toggle_notch,
            set_notch_expanded,
            fetch_weather
        ])
        .run(tauri::generate_context!())
        .expect("error while running NotchKu");
}
