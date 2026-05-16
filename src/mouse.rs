use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_GETMOUSE, SPI_GETMOUSESPEED, SPI_SETMOUSE, SPI_SETMOUSESPEED,
    SPIF_SENDCHANGE,
};

#[derive(Debug, Clone)]
pub struct MouseSettings {
    pub speed: u32,
    pub enhance_precision: bool,
}

pub fn get_current_settings() -> Result<MouseSettings, String> {
    let mut speed: u32 = 0;
    unsafe {
        SystemParametersInfoW(
            SPI_GETMOUSESPEED,
            0,
            Some(&mut speed as *mut u32 as *mut _),
            SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("Failed to get mouse speed: {e}"))?;
    }

    let mut mouse_params: [u32; 3] = [0; 3];
    unsafe {
        SystemParametersInfoW(
            SPI_GETMOUSE,
            0,
            Some(mouse_params.as_mut_ptr() as *mut _),
            SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("Failed to get mouse params: {e}"))?;
    }

    let enhance_precision = mouse_params[2] != 0;

    Ok(MouseSettings {
        speed,
        enhance_precision,
    })
}

pub fn apply_settings(settings: &MouseSettings) -> Result<(), String> {
    unsafe {
        SystemParametersInfoW(
            SPI_SETMOUSESPEED,
            0,
            Some(settings.speed as *const u32 as *mut _),
            SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("Failed to set mouse speed: {e}"))?;

        let mut current_params: [u32; 3] = [0; 3];
        SystemParametersInfoW(
            SPI_GETMOUSE,
            0,
            Some(current_params.as_mut_ptr() as *mut _),
            SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("Failed to get current mouse params: {e}"))?;

        current_params[2] = if settings.enhance_precision { 1 } else { 0 };

        SystemParametersInfoW(
            SPI_SETMOUSE,
            0,
            Some(current_params.as_ptr() as *mut _),
            SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("Failed to set mouse acceleration: {e}"))?;
    }

    Ok(())
}
