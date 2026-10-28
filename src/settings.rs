use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeChoice { Graphite, Midnight, Light }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: ThemeChoice,
    pub font_size: f32,
    pub word_wrap: bool,
    pub autosave: bool,
    pub animations: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { theme: ThemeChoice::Graphite, font_size: 14.0, word_wrap: true, autosave: false, animations: true }
    }
}

fn settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))?;
    Some(base.join(".config").join("vyron-ide").join("settings.json"))
}

impl Settings {
    pub fn load() -> Self {
        settings_path().and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn save(&self) {
        if let Some(path) = settings_path() {
            if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
            if let Ok(json) = serde_json::to_string_pretty(self) { let _ = fs::write(path, json); }
        }
    }
}
