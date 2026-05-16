mod config;
mod mouse;
mod tray;

use std::cell::Cell;
use std::str::FromStr;

use config::Config;
use mouse::{apply_settings, MouseSettings};
use tray::{AppTray, CurrentProfile};

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

    let app_tray = tray::build_tray().expect("Failed to create tray icon");

    let hotkey_manager =
        global_hotkey::GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    let hotkey = global_hotkey::hotkey::HotKey::from_str(&config.hotkey.toggle)
        .expect("Failed to parse hotkey. Use format like: Ctrl+Alt+M");
    hotkey_manager
        .register(hotkey)
        .expect("Failed to register hotkey");

    let hotkey_rx = global_hotkey::GlobalHotKeyEvent::receiver();
    let menu_rx = tray_icon::menu::MenuEvent::receiver();

    let current_profile = Cell::new(CurrentProfile::Normal);

    println!("Mouse Switcher started. Hotkey: {}", config.hotkey.toggle);

    loop {
        crossbeam_channel::select! {
            recv(hotkey_rx) -> event => {
                if let Ok(event) = event {
                    if event.state == global_hotkey::HotKeyState::Pressed {
                        do_toggle(&app_tray, &config, &current_profile);
                    }
                }
            }
            recv(menu_rx) -> event => {
                if let Ok(event) = event {
                    handle_menu_event(&app_tray, &config, &current_profile, event);
                }
            }
        }
    }
}

fn do_toggle(tray: &AppTray, config: &Config, current: &Cell<CurrentProfile>) {
    let next = match current.get() {
        CurrentProfile::Normal => CurrentProfile::Gaming,
        CurrentProfile::Gaming => CurrentProfile::Normal,
    };
    switch_to_profile(tray, config, current, next);
}

fn switch_to_profile(
    tray: &AppTray,
    config: &Config,
    current: &Cell<CurrentProfile>,
    profile: CurrentProfile,
) {
    if current.get() == profile {
        return;
    }

    let profile_config = match profile {
        CurrentProfile::Normal => config.normal.clone(),
        CurrentProfile::Gaming => config.gaming.clone(),
    };

    match apply_settings(&profile_config.into()) {
        Ok(()) => println!("Switched to {} profile", profile.label()),
        Err(e) => eprintln!("Failed to apply {} profile: {}", profile.label(), e),
    }

    current.set(profile);
    tray::update_tray_ui(tray, profile);
}

fn handle_menu_event(
    tray: &AppTray,
    config: &Config,
    current: &Cell<CurrentProfile>,
    event: tray_icon::menu::MenuEvent,
) {
    if tray::is_toggle(&event.id) {
        do_toggle(tray, config, current);
    } else if let Some(profile) = tray::menu_id_to_profile(&event.id) {
        switch_to_profile(tray, config, current, profile);
    } else if tray::is_open_config(&event.id) {
        tray::open_config_file();
    } else if tray::is_quit(&event.id) {
        println!("Quitting Mouse Switcher...");
        std::process::exit(0);
    }
}
