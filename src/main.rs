#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod config;
mod mouse;
mod tray;

use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
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

struct App {
    profile: CurrentProfile,
    config: Config,
    tray: AppTray,
    hotkey_toggle_id: u32,
    hotkey_speed_up_id: u32,
    hotkey_speed_down_id: u32,
    log_file: Option<std::fs::File>,
}

impl App {
    fn log(&mut self, msg: &str) {
        if let Some(ref mut f) = self.log_file {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

fn open_log_file() -> Option<std::fs::File> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let log_path = exe_dir.join("mouse-switcher.log");
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .ok()
}

fn parse_hotkey(s: &str) -> Option<global_hotkey::hotkey::HotKey> {
    if let Ok(h) = global_hotkey::hotkey::HotKey::from_str(s) {
        return Some(h);
    }
    let fallback = match s.to_lowercase().as_str() {
        s if s.contains("arrowup") || s.contains("up") => s
            .to_lowercase()
            .replace("arrowup", "Plus")
            .replace("up", "Plus"),
        s if s.contains("arrowdown") || s.contains("down") => s
            .to_lowercase()
            .replace("arrowdown", "Minus")
            .replace("down", "Minus"),
        _ => return None,
    };
    global_hotkey::hotkey::HotKey::from_str(&fallback).ok()
}

fn main() {
    let mut log_file = open_log_file();
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "=== Mouse Switcher starting ===");
    }

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "FATAL: {}", e);
            }
            show_error(&format!("Failed to load config: {}", e));
            return;
        }
    };

    if let Err(e) = apply_settings(&config.normal.clone().into()) {
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "WARN: Failed to apply initial profile: {}", e);
        }
    }

    let app_tray = match tray::build_tray() {
        Ok(t) => t,
        Err(e) => {
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "FATAL: Failed to create tray: {}", e);
            }
            show_error(&format!("Failed to create tray icon: {}", e));
            return;
        }
    };

    let hotkey_manager = match global_hotkey::GlobalHotKeyManager::new() {
        Ok(m) => m,
        Err(e) => {
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "FATAL: Failed to create hotkey manager: {}", e);
            }
            show_error(&format!("Failed to create hotkey manager: {}", e));
            return;
        }
    };

    let hotkey_toggle = match parse_hotkey(&config.hotkey.toggle) {
        Some(h) => h,
        None => {
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "WARN: Failed to parse toggle hotkey: {}", config.hotkey.toggle);
            }
            show_error(&format!(
                "Failed to parse hotkey '{}'. Edit settings.toml.",
                config.hotkey.toggle
            ));
            return;
        }
    };

    let hotkey_speed_up = match parse_hotkey(&config.hotkey.speed_up) {
        Some(h) => h,
        None => {
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "WARN: Failed to parse speed_up hotkey: {}", config.hotkey.speed_up);
            }
            global_hotkey::hotkey::HotKey::from_str("Ctrl+Alt+Plus").unwrap()
        }
    };

    let hotkey_speed_down = match parse_hotkey(&config.hotkey.speed_down) {
        Some(h) => h,
        None => {
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "WARN: Failed to parse speed_down hotkey: {}", config.hotkey.speed_down);
            }
            global_hotkey::hotkey::HotKey::from_str("Ctrl+Alt+Minus").unwrap()
        }
    };

    let hotkey_toggle_id = hotkey_toggle.id();
    let hotkey_speed_up_id = hotkey_speed_up.id();
    let hotkey_speed_down_id = hotkey_speed_down.id();

    if let Err(e) = hotkey_manager.register(hotkey_toggle) {
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "WARN: Failed to register toggle hotkey: {}", e);
        }
    }
    if let Err(e) = hotkey_manager.register(hotkey_speed_up) {
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "WARN: Failed to register speed_up hotkey: {}", e);
        }
    }
    if let Err(e) = hotkey_manager.register(hotkey_speed_down) {
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "WARN: Failed to register speed_down hotkey: {}", e);
        }
    }

    tray::update_tray_ui(
        &app_tray,
        CurrentProfile::Normal,
        config.normal.speed,
        config.normal.enhance_precision,
    );

    let app = Rc::new(RefCell::new(App {
        profile: CurrentProfile::Normal,
        config,
        tray: app_tray,
        hotkey_toggle_id,
        hotkey_speed_up_id,
        hotkey_speed_down_id,
        log_file,
    }));

    {
        let mut a = app.borrow_mut();
        let toggle = a.config.hotkey.toggle.clone();
        let speed_up = a.config.hotkey.speed_up.clone();
        let speed_down = a.config.hotkey.speed_down.clone();
        a.log("Mouse Switcher started.");
        a.log(&format!("  Toggle: {}", toggle));
        a.log(&format!("  Speed Up: {}", speed_up));
        a.log(&format!("  Speed Down: {}", speed_down));
    }

    run_event_loop(app);
}

#[cfg(target_os = "windows")]
fn run_event_loop(app: Rc<RefCell<App>>) {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, TranslateMessage, MSG,
    };

    let hotkey_rx = global_hotkey::GlobalHotKeyEvent::receiver();
    let menu_rx = tray_icon::menu::MenuEvent::receiver();

    let mut msg = MSG::default();

    loop {
        unsafe {
            if GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
        }

        if let Ok(event) = hotkey_rx.try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                handle_hotkey(&app, event.id);
            }
        }

        if let Ok(event) = menu_rx.try_recv() {
            handle_menu_event(&app, event);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn run_event_loop(app: Rc<RefCell<App>>) {
    let hotkey_rx = global_hotkey::GlobalHotKeyEvent::receiver();
    let menu_rx = tray_icon::menu::MenuEvent::receiver();

    loop {
        crossbeam_channel::select! {
            recv(hotkey_rx) -> event => {
                if let Ok(event) = event {
                    if event.state == global_hotkey::HotKeyState::Pressed {
                        handle_hotkey(&app, event.id);
                    }
                }
            }
            recv(menu_rx) -> event => {
                if let Ok(event) = event {
                    handle_menu_event(&app, event);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn show_error(msg: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::MessageBoxW;
    let wide: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "Mouse Switcher Error\0".encode_utf16().collect();
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(wide.as_ptr()),
            PCWSTR(title.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn show_error(msg: &str) {
    eprintln!("ERROR: {}", msg);
}

fn refresh_ui(app: &Rc<RefCell<App>>) {
    let a = app.borrow();
    let (speed, accel) = match a.profile {
        CurrentProfile::Normal => (a.config.normal.speed, a.config.normal.enhance_precision),
        CurrentProfile::Gaming => (a.config.gaming.speed, a.config.gaming.enhance_precision),
    };
    tray::update_tray_ui(&a.tray, a.profile, speed, accel);
}

fn handle_hotkey(app: &Rc<RefCell<App>>, hotkey_id: u32) {
    let (is_toggle, is_up, is_down) = {
        let a = app.borrow();
        (
            hotkey_id == a.hotkey_toggle_id,
            hotkey_id == a.hotkey_speed_up_id,
            hotkey_id == a.hotkey_speed_down_id,
        )
    };

    if is_toggle {
        do_toggle(app);
    } else if is_up {
        do_change_speed(app, 1);
    } else if is_down {
        do_change_speed(app, -1);
    }
}

fn handle_menu_event(app: &Rc<RefCell<App>>, event: tray_icon::menu::MenuEvent) {
    if tray::is_toggle(&event.id) {
        do_toggle(app);
    } else if let Some(profile) = tray::menu_id_to_profile(&event.id) {
        switch_to_profile(app, profile);
    } else if tray::is_speed_up(&event.id) {
        do_change_speed(app, 1);
    } else if tray::is_speed_down(&event.id) {
        do_change_speed(app, -1);
    } else if tray::is_accel_toggle(&event.id) {
        do_toggle_accel(app);
    } else if tray::is_open_config(&event.id) {
        tray::open_config_file();
    } else if tray::is_quit(&event.id) {
        {
            let mut a = app.borrow_mut();
            a.log("Quitting Mouse Switcher...");
        }
        std::process::exit(0);
    }
}

fn do_toggle(app: &Rc<RefCell<App>>) {
    let next = {
        let a = app.borrow();
        match a.profile {
            CurrentProfile::Normal => CurrentProfile::Gaming,
            CurrentProfile::Gaming => CurrentProfile::Normal,
        }
    };
    switch_to_profile(app, next);
}

fn switch_to_profile(app: &Rc<RefCell<App>>, profile: CurrentProfile) {
    {
        let a = app.borrow();
        if a.profile == profile {
            return;
        }
    }

    let profile_config = {
        let a = app.borrow();
        match profile {
            CurrentProfile::Normal => a.config.normal.clone(),
            CurrentProfile::Gaming => a.config.gaming.clone(),
        }
    };

    match apply_settings(&profile_config.clone().into()) {
        Ok(()) => {
            let mut a = app.borrow_mut();
            a.log(&format!("Switched to {} profile", profile.label()));
        }
        Err(e) => {
            let mut a = app.borrow_mut();
            a.log(&format!("Failed to apply {} profile: {}", profile.label(), e));
        }
    }

    app.borrow_mut().profile = profile;
    refresh_ui(app);
}

fn do_change_speed(app: &Rc<RefCell<App>>, delta: i32) {
    let mut prof = {
        let a = app.borrow();
        match a.profile {
            CurrentProfile::Normal => a.config.normal.clone(),
            CurrentProfile::Gaming => a.config.gaming.clone(),
        }
    };

    let new_speed = (prof.speed as i32 + delta).clamp(1, 20) as u32;
    if new_speed == prof.speed {
        return;
    }
    prof.speed = new_speed;

    match apply_settings(&prof.clone().into()) {
        Ok(()) => {
            let mut a = app.borrow_mut();
            a.log(&format!("Speed changed to {}", new_speed));
        }
        Err(e) => {
            let mut a = app.borrow_mut();
            a.log(&format!("Failed to change speed: {}", e));
        }
    }

    {
        let mut a = app.borrow_mut();
        match a.profile {
            CurrentProfile::Normal => a.config.normal = prof,
            CurrentProfile::Gaming => a.config.gaming = prof,
        }
        if let Err(e) = a.config.save() {
            a.log(&format!("Failed to save config: {}", e));
        }
    }

    refresh_ui(app);
}

fn do_toggle_accel(app: &Rc<RefCell<App>>) {
    let mut prof = {
        let a = app.borrow();
        match a.profile {
            CurrentProfile::Normal => a.config.normal.clone(),
            CurrentProfile::Gaming => a.config.gaming.clone(),
        }
    };
    prof.enhance_precision = !prof.enhance_precision;

    match apply_settings(&prof.clone().into()) {
        Ok(()) => {
            let mut a = app.borrow_mut();
            a.log(&format!(
                "Acceleration {}",
                if prof.enhance_precision { "ON" } else { "OFF" }
            ));
        }
        Err(e) => {
            let mut a = app.borrow_mut();
            a.log(&format!("Failed to toggle acceleration: {}", e));
        }
    }

    {
        let mut a = app.borrow_mut();
        match a.profile {
            CurrentProfile::Normal => a.config.normal = prof,
            CurrentProfile::Gaming => a.config.gaming = prof,
        }
        if let Err(e) = a.config.save() {
            a.log(&format!("Failed to save config: {}", e));
        }
    }

    refresh_ui(app);
}
