use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum CrosshairStyle {
    Cross,
    Circle,
    Dot,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn to_colorref(self) -> u32 {
        (self.r as u32) | ((self.g as u32) << 8) | ((self.b as u32) << 16)
    }

    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const RED: Color = Color { r: 255, g: 40, b: 40 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255 };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosshairConfig {
    #[serde(default = "default_style")]
    pub style: CrosshairStyle,
    #[serde(default = "default_color")]
    pub color: Color,
    #[serde(default = "default_size")]
    pub size: u32,            // Cross arm length / Circle radius / Dot size / Image size
    #[serde(default = "default_thickness")]
    pub thickness: u32,       // Line thickness (1-10)
    #[serde(default = "default_gap")]
    pub gap: u32,             // Center gap (0-30)
    #[serde(default = "default_true")]
    pub outline: bool,
    #[serde(default = "default_black")]
    pub outline_color: Color,
    #[serde(default)]
    pub dot: bool,            // Center dot toggle
    #[serde(default = "default_dot_size")]
    pub dot_size: u32,        // Center dot radius/size (1-10)
    #[serde(default = "default_opacity")]
    pub opacity: u8,          // 0-255
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub monitor_index: usize,
}

fn default_style() -> CrosshairStyle { CrosshairStyle::Cross }
fn default_color() -> Color { Color::GREEN }
fn default_size() -> u32 { 6 }
fn default_thickness() -> u32 { 2 }
fn default_gap() -> u32 { 3 }
fn default_true() -> bool { true }
fn default_black() -> Color { Color { r: 0, g: 0, b: 0 } }
fn default_dot_size() -> u32 { 2 }
fn default_opacity() -> u8 { 255 }

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            style: CrosshairStyle::Cross,
            color: Color::GREEN,
            size: 6,           // Crisp competitive default (not gigantic)
            thickness: 2,
            gap: 3,
            outline: true,
            outline_color: Color { r: 0, g: 0, b: 0 },
            dot: false,
            dot_size: 2,
            opacity: 255,
            image_path: None,
            monitor_index: 0,
        }
    }
}

impl CrosshairConfig {
    pub fn config_path() -> PathBuf {
        let mut p = dirs_config();
        p.push("crosshair.json");
        p
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str(&data) {
                    return cfg;
                }
            }
        }
        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn custom_crosshairs_dir() -> PathBuf {
        let local = PathBuf::from("custom_crosshairs");
        if local.exists() {
            return local;
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let exe_local = parent.join("custom_crosshairs");
                if exe_local.exists() {
                    return exe_local;
                }
            }
        }
        let mut p = dirs_config();
        p.push("custom_crosshairs");
        let _ = std::fs::create_dir_all(&p);
        p
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(path, json)
    }
}

fn dirs_config() -> PathBuf {
    let local = PathBuf::from("crosshair.json");
    if local.exists() {
        return PathBuf::from(".");
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_local = parent.join("crosshair.json");
            if exe_local.exists() {
                return parent.to_path_buf();
            }
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let mut p = PathBuf::from(appdata);
        p.push("MattsCrosshair");
        p
    } else {
        PathBuf::from(".")
    }
}
