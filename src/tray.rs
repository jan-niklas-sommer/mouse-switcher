use crate::config::Config;
use std::sync::{Arc, Mutex};
use tray_icon::{
    menu::{Check, Menu, MenuId, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

const MENU_ID_NORMAL: &str = "normal";
const MENU_ID_GAMING: &str = "gaming";
const MENU_ID_TOGGLE: &str = "toggle";
const MENU_ID_OPEN_CONFIG: &str = "open_config";
const MENU_ID_QUIT: &str = "quit";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentProfile {
    Normal,
    Gaming,
}

impl CurrentProfile {
    pub fn label(&self) -> &str {
        match self {
            CurrentProfile::Normal => "Normal",
            CurrentProfile::Gaming => "Gaming",
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            CurrentProfile::Normal => CurrentProfile::Gaming,
            CurrentProfile::Gaming => CurrentProfile::Normal,
        };
    }
}

pub struct AppIcon {
    pub tray: TrayIcon,
    pub menu_normal: Check<()>,
    pub menu_gaming: Check<()>,
}

pub struct AppState {
    pub current_profile: CurrentProfile,
    pub config: Config,
    pub icon: AppIcon,
}

pub fn build_tray(config: &Config) -> Result<AppIcon, String> {
    let icon = create_icon();

    let menu_normal =
        Check::with_id(MENU_ID_NORMAL, true, "Normal", true, None::<&str>);
    let menu_gaming =
        Check::with_id(MENU_ID_GAMING, false, "Gaming", true, None::<&str>);
    let toggle =
        MenuItem::with_id(MENU_ID_TOGGLE, "Toggle Profile", true, None::<&str>);
    let open_config =
        MenuItem::with_id(MENU_ID_OPEN_CONFIG, "Open Settings", true, None::<&str>);
    let quit =
        MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None::<&str>);

    let tray_menu = Menu::new();
    tray_menu
        .append(&menu_normal)
        .map_err(|e| format!("Menu error: {e}"))?;
    tray_menu
        .append(&menu_gaming)
        .map_err(|e| format!("Menu error: {e}"))?;
    tray_menu
        .append(&PredefinedMenuItem::separator())
        .map_err(|e| format!("Menu error: {e}"))?;
    tray_menu
        .append(&toggle)
        .map_err(|e| format!("Menu error: {e}"))?;
    tray_menu
        .append(&PredefinedMenuItem::separator())
        .map_err(|e| format!("Menu error: {e}"))?;
    tray_menu
        .append(&open_config)
        .map_err(|e| format!("Menu error: {e}"))?;
    tray_menu
        .append(&quit)
        .map_err(|e| format!("Menu error: {e}"))?;

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Mouse Switcher - Normal")
        .menu(&tray_menu)
        .build()
        .map_err(|e| format!("Tray error: {e}"))?;

    Ok(AppIcon {
        tray,
        menu_normal,
        menu_gaming,
    })
}

pub fn update_tray_ui(state: &Arc<Mutex<AppState>>) {
    let s = state.lock().unwrap();
    let profile = s.current_profile;
    s.icon.tray.set_tooltip(Some(&format!(
        "Mouse Switcher - {}",
        profile.label()
    )));
    s.icon.menu_normal.set_checked(profile == CurrentProfile::Normal);
    s.icon.menu_gaming.set_checked(profile == CurrentProfile::Gaming);
}

fn create_icon() -> tray_icon::icon::Icon {
    let rgba = generate_icon_data();
    tray_icon::icon::Icon::from_rgba(rgba, 32, 32).expect("Failed to create icon")
}

fn generate_icon_data() -> Vec<u8> {
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let center = size as f32 / 2.0;
    let radius = center - 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                let (r, g, b) = if dist <= radius * 0.35 {
                    (100u8, 230u8, 140u8)
                } else {
                    (60u8, 170u8, 100u8)
                };
                let alpha = if dist > radius - 1.5 {
                    ((radius - dist) / 1.5 * 255.0) as u8
                } else {
                    255u8
                };
                rgba.extend_from_slice(&[r, g, b, alpha]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    rgba
}

pub fn menu_id_to_profile(id: &MenuId) -> Option<CurrentProfile> {
    match id.as_ref() {
        MENU_ID_NORMAL => Some(CurrentProfile::Normal),
        MENU_ID_GAMING => Some(CurrentProfile::Gaming),
        _ => None,
    }
}

pub fn is_toggle(id: &MenuId) -> bool {
    id.as_ref() == MENU_ID_TOGGLE
}

pub fn is_open_config(id: &MenuId) -> bool {
    id.as_ref() == MENU_ID_OPEN_CONFIG
}

pub fn is_quit(id: &MenuId) -> bool {
    id.as_ref() == MENU_ID_QUIT
}

pub fn open_config_file() {
    let path = Config::config_path();
    if !path.exists() {
        let config = Config::default_values();
        let _ = config.save();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("notepad").arg(&path).spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Config file location: {}", path.display());
    }
}
