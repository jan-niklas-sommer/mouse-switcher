mod config;
mod mouse;
mod tray;

use std::sync::{Arc, Mutex};

use config::Config;
use mouse::{apply_settings, MouseSettings};
use tray::{AppState, CurrentProfile};

impl From<config::Profile> for MouseSettings {
    fn from(p: config::Profile) -> Self {
        MouseSettings {
            speed: p.speed,
            enhance_precision: p.enhance_precision,
        }
    }
}

fn main() {
    let config = Config::load().expect("Failed to load config. Delete settings.toml to reset.");

    apply_settings(&config.normal.clone().into()).expect("Failed to apply initial profile");

    let app_icon = tray::build_tray(&config).expect("Failed to create tray icon");

    let state = Arc::new(Mutex::new(AppState {
        current_profile: CurrentProfile::Normal,
        config: config.clone(),
        icon: app_icon,
    }));

    let hotkey_manager =
        global_hotkey::GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let hotkey = global_hotkey::hotkey::HotKey::new_from_str(&config.hotkey.toggle)
        .expect("Failed to parse hotkey. Use format like: Ctrl+Alt+M");
    hotkey_manager
        .register(hotkey)
        .expect("Failed to register hotkey");

    let hotkey_rx = global_hotkey::GlobalHotKeyEvent::receiver();
    let menu_rx = tray_icon::menu::MenuEvent::receiver();

    println!("Mouse Switcher started. Hotkey: {}", config.hotkey.toggle);

    loop {
        crossbeam_channel::select! {
            recv(hotkey_rx) -> event => {
                if let Ok(event) = event {
                    if event.state() == global_hotkey::HotKeyState::Pressed {
                        toggle_profile(&state);
                    }
                }
            }
            recv(menu_rx) -> event => {
                if let Ok(event) = event {
                    handle_menu_event(&state, event);
                }
            }
        }
    }
}

fn switch_to_profile(state: &Arc<Mutex<AppState>>, profile: CurrentProfile) {
    let mut s = state.lock().unwrap();
    if s.current_profile == profile {
        return;
    }
    let profile_config = match profile {
        CurrentProfile::Normal => s.config.normal.clone(),
        CurrentProfile::Gaming => s.config.gaming.clone(),
    };
    s.current_profile = profile;
    let label = profile.label().to_string();
    drop(s);

    match apply_settings(&profile_config.into()) {
        Ok(()) => println!("Switched to {} profile", label),
        Err(e) => eprintln!("Failed to apply {} profile: {}", label, e),
    }

    tray::update_tray_ui(state);
}

fn toggle_profile(state: &Arc<Mutex<AppState>>) {
    let next = {
        let s = state.lock().unwrap();
        match s.current_profile {
            CurrentProfile::Normal => CurrentProfile::Gaming,
            CurrentProfile::Gaming => CurrentProfile::Normal,
        }
    };
    switch_to_profile(state, next);
}

fn handle_menu_event(state: &Arc<Mutex<AppState>>, event: tray_icon::menu::MenuEvent) {
    if tray::is_toggle(&event.id) {
        toggle_profile(state);
    } else if let Some(profile) = tray::menu_id_to_profile(&event.id) {
        switch_to_profile(state, profile);
    } else if tray::is_open_config(&event.id) {
        tray::open_config_file();
    } else if tray::is_quit(&event.id) {
        println!("Quitting Mouse Switcher...");
        std::process::exit(0);
    }
}
