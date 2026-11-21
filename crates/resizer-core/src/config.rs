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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub format: OutputFormat,
    pub target_width: u32,
    pub no_upscale: bool,
    pub quality: u8,
    pub delete_originals: bool,
    pub theme: ThemeMode,
    pub show_notifications: bool,
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
            show_notifications: true,
        }
    }
}

impl AppConfig {
    /// Возвращает стандартный путь к config.json для текущей ОС:
    /// Windows: %APPDATA%\ImageResizerJ\config\config.json
    /// Linux:   ~/.config/imageresizerJ/config.json
    pub fn config_path() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("", "", "ImageResizerJ") {
            dirs.config_dir().join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    /// Возвращает папку с правами на запись для кэша WebView2 / WebKitGTK:
    /// Windows: %LOCALAPPDATA%\ImageResizerJ\data\<subdir>
    /// Linux:   ~/.local/share/imageresizerj/<subdir>
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

    /// Загружает настройки или создаёт файл с параметрами по умолчанию
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

    /// Сохраняет настройки в форматированный JSON
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
