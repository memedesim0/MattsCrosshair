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
    /// Directory in which the executable resides (always used instead of current working directory)
    pub fn exe_dir() -> PathBuf {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                return parent.to_path_buf();
            }
        }
        PathBuf::from(".")
    }

    /// Full path to crosshair.json in the executable's directory
    pub fn config_path() -> PathBuf {
        Self::exe_dir().join("crosshair.json")
    }

    /// Full path to custom_crosshairs/ in the executable's directory.
    /// Automatically created if missing, and initializes default logo.png.
    pub fn custom_crosshairs_dir() -> PathBuf {
        let dir = Self::exe_dir().join("custom_crosshairs");
        let _ = std::fs::create_dir_all(&dir);

        let default_logo = dir.join("logo.png");
        if !default_logo.exists() {
            let png_bytes = include_bytes!("../assets/logo.png");
            let _ = std::fs::write(&default_logo, png_bytes);
        }

        dir
    }

    /// Loads config from executable's directory, or creates default on first launch.
    pub fn load() -> Self {
        // Ensure custom crosshairs folder and default logo are initialized
        let _ = Self::custom_crosshairs_dir();

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

    /// Saves config to crosshair.json in the executable's directory
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(path, json)
    }
}
