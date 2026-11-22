use std::fs;
use std::io;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Jpg,
    Png,
    Webp,
    Avif,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Jpg => "jpg",
            OutputFormat::Png => "png",
            OutputFormat::Webp => "webp",
            OutputFormat::Avif => "avif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageMode {
    #[default]
    Auto,
    Ru,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolvedLanguage {
    Ru,
    En,
}

#[cfg(windows)]
pub fn detect_system_language() -> ResolvedLanguage {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }

    let lang_id = unsafe { GetUserDefaultUILanguage() };
    if (lang_id & 0x03FF) == 0x19 {
        ResolvedLanguage::Ru
    } else {
        ResolvedLanguage::En
    }
}

#[cfg(not(windows))]
pub fn detect_system_language() -> ResolvedLanguage {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(key)
            && !val.is_empty()
        {
            return if val.to_ascii_lowercase().starts_with("ru") {
                ResolvedLanguage::Ru
            } else {
                ResolvedLanguage::En
            };
        }
    }
    ResolvedLanguage::En
}

impl LanguageMode {
    pub fn resolve(self) -> ResolvedLanguage {
        match self {
            LanguageMode::Auto => detect_system_language(),
            LanguageMode::Ru => ResolvedLanguage::Ru,
            LanguageMode::En => ResolvedLanguage::En,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub format: OutputFormat,
    pub target_width: u32,
    pub no_upscale: bool,
    pub quality: u8,
    pub delete_originals: bool,
    pub theme: ThemeMode,
    pub language: LanguageMode,
    pub show_notifications: bool,
    pub hint_dismissed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Jpg,
            target_width: 1024,
            no_upscale: true,
            quality: 90,
            delete_originals: true,
            theme: ThemeMode::System,
            language: LanguageMode::Auto,
            show_notifications: true,
            hint_dismissed: false,
        }
    }
}

impl AppConfig {
    pub fn resolved_language(&self) -> ResolvedLanguage {
        self.language.resolve()
    }

    pub fn config_path() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("", "", "ImageResizerJ") {
            dirs.config_dir().join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn webview_data_dir(subdir: &str) -> PathBuf {
        let base = if let Some(dirs) = ProjectDirs::from("", "", "ImageResizerJ") {
            dirs.data_local_dir().to_path_buf()
        } else {
            std::env::temp_dir().join("ImageResizerJ")
        };
        let dir = base.join(subdir);
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn load_or_create() -> Self {
        let path = Self::config_path();
        if let Ok(text) = fs::read_to_string(&path)
            && let Ok(cfg) = serde_json::from_str::<AppConfig>(&text)
        {
            return cfg;
        }

        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }
}