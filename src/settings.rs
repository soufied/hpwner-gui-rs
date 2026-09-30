use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_exe_path() -> String {
    "hpwnr".to_string()
}

fn default_dark_mode() -> bool {
    true
}

fn default_accent_index() -> usize {
    0
}

fn default_sidebar_collapsed() -> bool {
    false
}

fn default_last_section() -> String {
    "decrypt".to_string()
}

fn default_window_width() -> f32 {
    1080.0
}

fn default_window_height() -> f32 {
    720.0
}

fn default_reduce_motion() -> bool {
    false
}

fn default_font_scale() -> f32 {
    1.0
}

fn default_compact_view() -> bool {
    false
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_exe_path")]
    pub exe_path: String,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
    #[serde(default = "default_accent_index")]
    pub accent_index: usize,
    #[serde(default = "default_sidebar_collapsed")]
    pub sidebar_collapsed: bool,
    #[serde(default = "default_last_section")]
    pub last_section: String,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default = "default_reduce_motion")]
    pub reduce_motion: bool,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
    #[serde(default = "default_compact_view")]
    pub compact_view: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            exe_path: default_exe_path(),
            dark_mode: default_dark_mode(),
            accent_index: default_accent_index(),
            sidebar_collapsed: default_sidebar_collapsed(),
            last_section: default_last_section(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            reduce_motion: default_reduce_motion(),
            font_scale: default_font_scale(),
            compact_view: default_compact_view(),
        }
    }
}

impl Settings {
    pub fn file_path() -> PathBuf {
        if let Ok(mut dir) = std::env::current_exe() {
            dir.pop();
            dir.push("hpwnr_settings.json");
            return dir;
        }
        PathBuf::from("hpwnr_settings.json")
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
                return settings.sanitized();
            }
        }
        Self::default()
    }

    pub fn sanitized(mut self) -> Self {
        if self.window_width < 480.0 || self.window_width > 8000.0 {
            self.window_width = default_window_width();
        }
        if self.window_height < 360.0 || self.window_height > 8000.0 {
            self.window_height = default_window_height();
        }
        if self.accent_index >= crate::ui::theme::Accent::ALL.len() {
            self.accent_index = default_accent_index();
        }
        if self.font_scale < 0.8 || self.font_scale > 1.6 {
            self.font_scale = default_font_scale();
        }
        self
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::file_path();
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn reset(&mut self) {
        *self = Self::default();
        let _ = self.save();
    }
}
