// NotchKu 0.2.0 — native Windows app (Slint UI, no WebView, no browser engine).
mod store;

use chrono::{Datelike, Local, NaiveDate};
use serde::Deserialize;
use slint::{Model, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

slint::include_modules!();

const COLLAPSED_W: f32 = 220.0;
const EXPANDED_W: f32 = 560.0;

fn screen_width() -> f32 {
    #[cfg(windows)]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN};
        unsafe { GetSystemMetrics(SM_CXSCREEN) as f32 }
    }
    #[cfg(not(windows))]
    {
        1920.0
    }
}

fn center_window(ui: &AppWindow) {
    let w = if ui.get_expanded() { EXPANDED_W } else { COLLAPSED_W };
    let x = (screen_width() - w) / 2.0;
    ui.window().set_position(slint::LogicalPosition::new(x, 0.0));
}

fn weather_label(code: i64) -> &'static str {
    match code {
        0 => "Cerah",
        1 | 2 => "Berawan",
        3 => "Mendung",
        45 | 48 => "Berkabut",
        51 | 53 | 55 => "Gerimis",
        61 | 63 | 65 | 80 | 81 | 82 => "Hujan",
        71 | 73 | 75 | 77 | 85 | 86 => "Salju",
        95 | 96 | 99 => "Badai petir",
        _ => "Cuaca",
    }
}

#[derive(Debug, Deserialize)]
struct WeatherResp {
    current: WeatherCurrent,
}

#[derive(Debug, Deserialize)]
struct WeatherCurrent {
    temperature_2m: f64,
    weather_code: i64,
    relative_humidity_2m: Option<f64>,
}

fn fetch_weather(lat: f64, lon: f64) -> Option<(String, String)> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code,relative_humidity_2m",
        lat, lon
    );
    let resp: WeatherResp = ureq::get(&url)
        .timeout(Duration::from_secs(10))
        .call()
        .ok()?
        .into_json()
        .ok()?;
    let c = resp.current;
    let short = format!("{:.0}° {}", c.temperature_2m, weather_label(c.weather_code));
    let detail = format!(
        "{:.0}°C — {}\nKelembapan {}%",
        c.temperature_2m,
        weather_label(c.weather_code),
        c.relative_humidity_2m.unwrap_or(0.0).round()
    );
    Some((short, detail))
}

fn calendar_weeks(year: i32, month: u32) -> Vec<Vec<String>> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let first_wd = first.weekday().num_days_from_sunday() as usize; // 0 = Sunday
    let days_in_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    }
    .signed_duration_since(first)
    .num_days() as usize;
    let mut weeks: Vec<Vec<String>> = Vec::new();
    let mut week: Vec<String> = vec![String::new(); 7];
    let mut day = 1usize;
    let mut idx = first_wd;
    while day <= days_in_month {
        week[idx] = day.to_string();
        day += 1;
        idx += 1;
        if idx == 7 {
            weeks.push(week);
            week = vec![String::new(); 7];
            idx = 0;
        }
    }
    if idx != 0 {
        weeks.push(week);
    }
    while weeks.len() < 6 {
        weeks.push(vec![String::new(); 7]);
    }
    weeks
}

fn fmt_mmss(total_secs: u32) -> String {
    format!("{:02}:{:02}", total_secs / 60, total_secs % 60)
}

#[derive(Clone, Copy, PartialEq)]
enum PomoMode {
    Focus,
    Break,
}

struct PomoState {
    mode: PomoMode,
    remaining: u32,
    running: bool,
    focus_min: u32,
    break_min: u32,
}

fn main() {
    let settings = store::load_settings();
    let ui = AppWindow::new().unwrap();
    let weak = ui.as_weak();

    // ---------- Initial state ----------
    let now = Local::now();
    ui.set_clock_text(SharedString::from(now.format("%H:%M").to_string()));
    ui.set_calendar_title(SharedString::from(format!(
        "{} {} — hari ini {}",
        now.format("%B"),
        now.year(),
        now.day()
    )));
    let weeks: Vec<ModelRc<SharedString>> = calendar_weeks(now.year(), now.month() as u32)
        .into_iter()
        .map(|w| {
            ModelRc::from(Rc::new(VecModel::from(
                w.into_iter().map(SharedString::from).collect::<Vec<_>>(),
            )))
        })
        .collect();
    ui.set_cal_weeks(ModelRc::from(Rc::new(VecModel::from(weeks))));

    // Todos
    let initial_todos: Vec<TodoItem> = store::load_todos()
        .into_iter()
        .map(|t| TodoItem { text: SharedString::from(t.text), done: t.done })
        .collect();
    let todos_model = Rc::new(VecModel::from(initial_todos));
    ui.set_todos(ModelRc::from(todos_model.clone()));

    // ---------- Clock timer (1s) ----------
    let clock_timer = Timer::default();
    {
        let weak = weak.clone();
        clock_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            if let Some(ui) = weak.upgrade() {
                let now = Local::now();
                ui.set_clock_text(SharedString::from(now.format("%H:%M").to_string()));
            }
        });
    }

    // ---------- Weather (background thread, refresh on demand) ----------
    let do_fetch_weather = {
        let weak = weak.clone();
        let lat = settings.latitude;
        let lon = settings.longitude;
        move || {
            let weak = weak.clone();
            std::thread::spawn(move || {
                if let Some((short, detail)) = fetch_weather(lat, lon) {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak.upgrade() {
                            ui.set_weather_text(SharedString::from(short));
                            ui.set_weather_detail(SharedString::from(detail));
                        }
                    });
                }
            });
        }
    };
    do_fetch_weather();

    // ---------- Toggle / close / tabs ----------
    {
        let weak = weak.clone();
        ui.on_toggle_expanded(move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_expanded(!ui.get_expanded());
                center_window(&ui);
            }
        });
    }
    {
        let weak = weak.clone();
        ui.on_collapse(move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_expanded(false);
                center_window(&ui);
            }
        });
    }
    {
        let weak = weak.clone();
        ui.on_set_tab(move |tab| {
            if let Some(ui) = weak.upgrade() {
                ui.set_tab(tab);
            }
        });
    }

    // ---------- Todos ----------
    {
        let model = todos_model.clone();
        ui.on_toggle_todo(move |index| {
            let i = index as usize;
            if let Some(mut item) = model.row_data(i) {
                item.done = !item.done;
                model.set_row_data(i, item);
                persist_todos(&model);
            }
        });
    }
    {
        let model = todos_model.clone();
        ui.on_delete_todo(move |index| {
            model.remove(index as usize);
            persist_todos(&model);
        });
    }
    {
        let weak = weak.clone();
        let model = todos_model.clone();
        ui.on_add_todo(move || {
            if let Some(ui) = weak.upgrade() {
                let text = ui.get_new_todo().trim().to_string();
                if !text.is_empty() {
                    model.push(TodoItem { text: SharedString::from(text), done: false });
                    ui.set_new_todo(SharedString::from(""));
                    persist_todos(&model);
                }
            }
        });
    }

    // ---------- Music demo ----------
    let playlist: Vec<(String, String)> = vec![
        ("Lagu Demo Satu".into(), "NotchKu Demo".into()),
        ("Lagu Demo Dua".into(), "NotchKu Demo".into()),
        ("Lagu Demo Tiga".into(), "NotchKu Demo".into()),
    ];
    let music = Rc::new(RefCell::new(MusicState {
        index: 0,
        playing: false,
        progress: 0.0,
    }));
    {
        let m = music.borrow();
        ui.set_music_title(SharedString::from(playlist[m.index].0.clone()));
        ui.set_music_artist(SharedString::from(playlist[m.index].1.clone()));
    }
    {
        let weak = weak.clone();
        let music = music.clone();
        ui.on_music_toggle(move || {
            let mut m = music.borrow_mut();
            m.playing = !m.playing;
            if let Some(ui) = weak.upgrade() {
                ui.set_music_playing(m.playing);
            }
        });
    }
    {
        let weak = weak.clone();
        let music = music.clone();
        let playlist = playlist.clone();
        let next = move |dir: i32| {
            let mut m = music.borrow_mut();
            let n = playlist.len() as i32;
            m.index = ((m.index as i32 + dir).rem_euclid(n)) as usize;
            m.progress = 0.0;
            if let Some(ui) = weak.upgrade() {
                ui.set_music_title(SharedString::from(playlist[m.index].0.clone()));
                ui.set_music_artist(SharedString::from(playlist[m.index].1.clone()));
                ui.set_music_progress(0.0);
            }
        };
        let next2 = next.clone();
        ui.on_music_next(move || next(1));
        ui.on_music_prev(move || next2(-1));
    }
    // Music progress timer
    let music_timer = Timer::default();
    {
        let weak = weak.clone();
        let music = music.clone();
        music_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let mut m = music.borrow_mut();
            if m.playing {
                m.progress += 1.0 / 180.0;
                if m.progress >= 1.0 {
                    m.progress = 0.0;
                    m.index = (m.index + 1) % 3;
                }
                if let Some(ui) = weak.upgrade() {
                    ui.set_music_progress(m.progress as f32);
                    ui.set_music_title(SharedString::from(
                        ["Lagu Demo Satu", "Lagu Demo Dua", "Lagu Demo Tiga"][m.index],
                    ));
                }
            }
        });
    }

    // ---------- Pomodoro ----------
    let pomo = Rc::new(RefCell::new(PomoState {
        mode: PomoMode::Focus,
        remaining: settings.pomodoro_focus_min * 60,
        running: false,
        focus_min: settings.pomodoro_focus_min,
        break_min: settings.pomodoro_break_min,
    }));
    {
        let weak = weak.clone();
        let pomo = pomo.clone();
        ui.on_pomodoro_toggle(move || {
            let mut p = pomo.borrow_mut();
            p.running = !p.running;
            if let Some(ui) = weak.upgrade() {
                ui.set_pomodoro_running(p.running);
            }
        });
    }
    {
        let weak = weak.clone();
        let pomo = pomo.clone();
        ui.on_pomodoro_reset(move || {
            let mut p = pomo.borrow_mut();
            p.running = false;
            p.mode = PomoMode::Focus;
            p.remaining = p.focus_min * 60;
            if let Some(ui) = weak.upgrade() {
                ui.set_pomodoro_running(false);
                ui.set_pomodoro_mode(SharedString::from("Fokus"));
                ui.set_pomodoro_text(SharedString::from(fmt_mmss(p.remaining)));
            }
        });
    }
    let pomo_timer = Timer::default();
    {
        let weak = weak.clone();
        let pomo = pomo.clone();
        pomo_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let mut p = pomo.borrow_mut();
            if !p.running {
                return;
            }
            if p.remaining > 0 {
                p.remaining -= 1;
            }
            if p.remaining == 0 {
                match p.mode {
                    PomoMode::Focus => {
                        p.mode = PomoMode::Break;
                        p.remaining = p.break_min * 60;
                    }
                    PomoMode::Break => {
                        p.mode = PomoMode::Focus;
                        p.remaining = p.focus_min * 60;
                    }
                }
                p.running = false;
            }
            if let Some(ui) = weak.upgrade() {
                ui.set_pomodoro_text(SharedString::from(fmt_mmss(p.remaining)));
                ui.set_pomodoro_mode(SharedString::from(match p.mode {
                    PomoMode::Focus => "Fokus",
                    PomoMode::Break => "Istirahat",
                }));
                ui.set_pomodoro_running(p.running);
            }
        });
    }

    // ---------- Weather refresh ----------
    {
        let f = do_fetch_weather.clone();
        ui.on_refresh_weather(move || {
            f();
        });
    }

    // ---------- Global hotkey Alt+N ----------
    {
        use global_hotkey::{
            GlobalHotKeyEvent, GlobalHotKeyManager,
            hotkey::{Code, HotKey, Modifiers},
        };
        if let Ok(manager) = GlobalHotKeyManager::new() {
            let _ = manager.register(HotKey::new(Some(Modifiers::ALT), Code::KeyN));
            // Keep the manager alive for the app lifetime.
            std::mem::forget(manager);
            let weak = weak.clone();
            std::thread::spawn(move || loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.state == global_hotkey::HotKeyState::Pressed {
                        let weak = weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak.upgrade() {
                                ui.set_expanded(!ui.get_expanded());
                                center_window(&ui);
                            }
                        });
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            });
        }
    }

    // ---------- Show ----------
    center_window(&ui);
    ui.run().unwrap();
}

struct MusicState {
    index: usize,
    playing: bool,
    progress: f64,
}

fn persist_todos(model: &VecModel<TodoItem>) {
    let todos: Vec<store::Todo> = (0..model.row_count())
        .filter_map(|i| model.row_data(i))
        .map(|t| store::Todo { text: t.text.to_string(), done: t.done })
        .collect();
    store::save_todos(&todos);
}
