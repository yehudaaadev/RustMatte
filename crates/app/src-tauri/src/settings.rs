//! Atomic JSON settings persistence in the OS application config directory.

use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const CURRENT_WIZARD_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    pub tier: String,
    pub device: String,
    pub refine_level: String,
    pub output_folder: Option<String>,
    pub max_side: Option<u32>,
    pub background_mode: String,
    pub background_hex: String,
    pub never_use_network: bool,
    pub wizard_completed: bool,
    pub wizard_version: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "system".to_string(),
            tier: "quality".to_string(),
            device: "cpu".to_string(),
            refine_level: "standard".to_string(),
            output_folder: None,
            max_side: None,
            background_mode: "transparent".to_string(),
            background_hex: "#ffffff".to_string(),
            never_use_network: false,
            wizard_completed: false,
            wizard_version: 0,
        }
    }
}

impl AppSettings {
    /// Sanitizes enum-like fields while preserving valid user preferences.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        if self.language != "en" && self.language != "id" {
            self.language = "en".to_string();
        }
        if !matches!(self.theme.as_str(), "system" | "dark" | "light") {
            self.theme = "system".to_string();
        }
        if !matches!(self.tier.as_str(), "quality" | "fast" | "maximum") {
            self.tier = "quality".to_string();
        }
        if !matches!(self.device.as_str(), "auto" | "cpu" | "directml" | "coreml") {
            self.device = "cpu".to_string();
        }
        if self.refine_level == "off" {
            self.refine_level = "none".to_string();
        }
        if !matches!(self.refine_level.as_str(), "none" | "light" | "standard") {
            self.refine_level = "standard".to_string();
        }
        if let Some(ref folder) = self.output_folder {
            if folder.trim().is_empty() {
                self.output_folder = None;
            }
        }
        if let Some(max) = self.max_side {
            if !(128..=16384).contains(&max) {
                self.max_side = None;
            }
        }
        if self.background_mode != "transparent" && self.background_mode != "solid" {
            self.background_mode = "transparent".to_string();
        }
        if !is_valid_hex_color(&self.background_hex) {
            self.background_hex = "#ffffff".to_string();
        }
        if !self.wizard_completed {
            self.wizard_version = 0;
        } else if self.wizard_version == 0 {
            self.wizard_version = CURRENT_WIZARD_VERSION;
        }
        self
    }
}

fn is_valid_hex_color(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

/// Loads settings from disk, gracefully returning defaults if missing or corrupt.
#[must_use]
pub fn load_settings_from_path(path: &Path) -> AppSettings {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return AppSettings::default();
    };
    match serde_json::from_str::<AppSettings>(&contents) {
        Ok(parsed) => parsed.sanitized(),
        Err(err) => {
            eprintln!(
                "Warning: Settings file '{}' is corrupt ({err}); using defaults.",
                path.display()
            );
            AppSettings::default()
        }
    }
}

/// Atomically writes settings to disk via a temporary file + rename.
pub fn save_settings_atomic(path: &Path, settings: &AppSettings) -> Result<(), String> {
    let clean = settings.clone().sanitized();
    let parent = path
        .parent()
        .ok_or_else(|| "Settings path has no parent directory".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create settings directory: {e}"))?;

    let json = serde_json::to_string_pretty(&clean)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;

    let tmp_name = format!(
        ".{}.tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json"),
        std::process::id()
    );
    let tmp_path = parent.join(tmp_name);

    {
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create temporary settings file: {e}"))?;
        if let Err(e) = file
            .write_all(json.as_bytes())
            .and_then(|_| file.sync_all())
        {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("Failed to write settings file: {e}"));
        }
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("Failed to commit settings file: {e}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_settings_path(label: &str) -> (PathBuf, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "rustmatte_app_settings_{label}_{}_{}",
            std::process::id(),
            nanos
        ));
        (dir.join("settings.json"), dir)
    }

    #[test]
    fn missing_file_returns_defaults() {
        let (path, dir) = temp_settings_path("missing");
        let loaded = load_settings_from_path(&path);
        assert_eq!(loaded, AppSettings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_and_load_round_trip() {
        let (path, dir) = temp_settings_path("roundtrip");
        let custom = AppSettings {
            language: "id".to_string(),
            theme: "light".to_string(),
            tier: "fast".to_string(),
            device: "cpu".to_string(),
            refine_level: "light".to_string(),
            output_folder: Some("C:\\tmp\\out".to_string()),
            max_side: Some(1024),
            background_mode: "solid".to_string(),
            background_hex: "#1234ab".to_string(),
            never_use_network: true,
            wizard_completed: true,
            wizard_version: CURRENT_WIZARD_VERSION,
        };

        save_settings_atomic(&path, &custom).unwrap();
        let loaded = load_settings_from_path(&path);
        assert_eq!(loaded, custom);

        let leftover_tmp = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().contains(".tmp."));
        assert!(!leftover_tmp);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_json_falls_back_to_defaults_and_migrates_off_refine() {
        let (path, dir) = temp_settings_path("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, b"{ invalid json :::").unwrap();

        let loaded = load_settings_from_path(&path);
        assert_eq!(loaded, AppSettings::default());

        let legacy = AppSettings {
            device: "cuda".to_string(),
            refine_level: "off".to_string(),
            background_hex: "not-hex".to_string(),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(AppSettings::default().device, "cpu");
        assert_eq!(legacy.device, "cpu");
        assert_eq!(legacy.refine_level, "none");
        assert_eq!(legacy.background_hex, "#ffffff");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn preserves_saved_auto_device_on_load() {
        let (path, dir) = temp_settings_path("preserve_auto");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, br#"{"device":"auto","tier":"fast"}"#).unwrap();

        let loaded = load_settings_from_path(&path);
        assert_eq!(loaded.device, "auto");
        assert_eq!(loaded.tier, "fast");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
