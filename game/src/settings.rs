#[allow(dead_code)]
pub struct GameSettings {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
    pub vsync: bool,
    pub show_debug_overlay: bool,
    pub master_volume: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
    pub render_distance: f32,
}

impl GameSettings {
    pub fn new() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            window_title: "Nexforge Engine".to_string(),
            vsync: true,
            show_debug_overlay: false,
            master_volume: 1.0,
            mouse_sensitivity: 0.002,
            fov: 70.0,
            render_distance: 1000.0,
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_env_settings() -> GameSettings {
    let mut settings = GameSettings::new();
    if let Ok(w) = std::env::var("NEXFORGE_WIDTH") {
        if let Ok(w) = w.parse::<u32>() {
            settings.window_width = w;
        }
    }
    if let Ok(h) = std::env::var("NEXFORGE_HEIGHT") {
        if let Ok(h) = h.parse::<u32>() {
            settings.window_height = h;
        }
    }
    if let Ok(title) = std::env::var("NEXFORGE_TITLE") {
        settings.window_title = title;
    }
    if let Ok(v) = std::env::var("NEXFORGE_VSYNC") {
        settings.vsync = v == "1" || v.to_lowercase() == "true";
    }
    if let Ok(v) = std::env::var("NEXFORGE_DEBUG") {
        settings.show_debug_overlay = v == "1" || v.to_lowercase() == "true";
    }
    if let Ok(v) = std::env::var("NEXFORGE_VOLUME") {
        if let Ok(v) = v.parse::<f32>() {
            settings.master_volume = v.clamp(0.0, 1.0);
        }
    }
    if let Ok(v) = std::env::var("NEXFORGE_SENSITIVITY") {
        if let Ok(v) = v.parse::<f32>() {
            settings.mouse_sensitivity = v;
        }
    }
    settings
}
