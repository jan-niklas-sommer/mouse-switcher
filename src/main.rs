mod config;
mod mouse;
mod tray;

use std::cell::RefCell;
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
    hotkey_toggle: global_hotkey::hotkey::HotKey,
    hotkey_speed_up: global_hotkey::hotkey::HotKey,
    hotkey_speed_down: global_hotkey::hotkey::HotKey,
}

fn main() {
    let config = Config::load().expect("Failed to load config. Delete settings.toml to reset.");

    apply_settings(&config.normal.clone().into()).expect("Failed to apply initial profile");

    let app_tray = tray::build_tray().expect("Failed to create tray icon");

    let hotkey_manager =
        global_hotkey::GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

    let hotkey_toggle = global_hotkey::hotkey::HotKey::from_str(&config.hotkey.toggle)
        .expect("Failed to parse toggle hotkey");
    let hotkey_speed_up = global_hotkey::hotkey::HotKey::from_str(&config.hotkey.speed_up)
        .expect("Failed to parse speed_up hotkey");
    let hotkey_speed_down = global_hotkey::hotkey::HotKey::from_str(&config.hotkey.speed_down)
        .expect("Failed to parse speed_down hotkey");

    hotkey_manager.register(hotkey_toggle).expect("Failed to register toggle hotkey");
    hotkey_manager.register(hotkey_speed_up).expect("Failed to register speed_up hotkey");
    hotkey_manager.register(hotkey_speed_down).expect("Failed to register speed_down hotkey");

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
        hotkey_toggle,
        hotkey_speed_up,
        hotkey_speed_down,
    }));

    println!("Mouse Switcher started.");
    println!("  Toggle: {}", app.borrow().config.hotkey.toggle);
    println!("  Speed Up: {}", app.borrow().config.hotkey.speed_up);
    println!("  Speed Down: {}", app.borrow().config.hotkey.speed_down);

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

fn refresh_ui(app: &Rc<RefCell<App>>) {
    let a = app.borrow();
    let (speed, accel) = match a.profile {
        CurrentProfile::Normal => (a.config.normal.speed, a.config.normal.enhance_precision),
        CurrentProfile::Gaming => (a.config.gaming.speed, a.config.gaming.enhance_precision),
    };
    tray::update_tray_ui(&a.tray, a.profile, speed, accel);
}

fn handle_hotkey(app: &Rc<RefCell<App>>, hotkey_id: global_hotkey::hotkey::HotKey) {
    let (is_toggle, is_up, is_down) = {
        let a = app.borrow();
        (
            hotkey_id == a.hotkey_toggle,
            hotkey_id == a.hotkey_speed_up,
            hotkey_id == a.hotkey_speed_down,
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
        println!("Quitting Mouse Switcher...");
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
        Ok(()) => println!("Switched to {} profile", profile.label()),
        Err(e) => eprintln!("Failed to apply {} profile: {}", profile.label(), e),
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
        Ok(()) => println!("Speed changed to {}", new_speed),
        Err(e) => eprintln!("Failed to change speed: {}", e),
    }

    {
        let mut a = app.borrow_mut();
        match a.profile {
            CurrentProfile::Normal => a.config.normal = prof,
            CurrentProfile::Gaming => a.config.gaming = prof,
        }
        if let Err(e) = a.config.save() {
            eprintln!("Failed to save config: {}", e);
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
        Ok(()) => println!(
            "Acceleration {}",
            if prof.enhance_precision { "ON" } else { "OFF" }
        ),
        Err(e) => eprintln!("Failed to toggle acceleration: {}", e),
    }

    {
        let mut a = app.borrow_mut();
        match a.profile {
            CurrentProfile::Normal => a.config.normal = prof,
            CurrentProfile::Gaming => a.config.gaming = prof,
        }
        if let Err(e) = a.config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }

    refresh_ui(app);
}
