pub mod config;

use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageReader, Rgb, RgbImage, RgbaImage};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub use config::{AppConfig, OutputFormat, ThemeMode};

static TEMP_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct BatchSummary {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub elapsed: Duration,
    pub errors: Vec<String>,
    pub log_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastPayload {
    pub succeeded: usize,
    pub failed: usize,
    pub elapsed_ms: u64,
    pub format: OutputFormat,
    pub theme: ThemeMode,
    pub log_path: Option<String>,
}

fn normalize_key(path: &Path) -> String {
    #[cfg(windows)]
    {
        path.to_string_lossy().replace('/', "\\").to_lowercase()
    }
    #[cfg(not(windows))]
    {
        path.to_string_lossy().into_owned()
    }
}

fn is_same_extension(path: &Path, target_ext: &str) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(target_ext))
}

fn reserve_output_path(
    file_path: &Path,
    config: &AppConfig,
    reserved: &Mutex<HashSet<String>>,
) -> PathBuf {
    let ext = config.format.extension();

    // Перезаписываем на месте только если включено удаление оригиналов и расширение уже совпадает
    if config.delete_originals && is_same_extension(file_path, ext) {
        return file_path.to_path_buf();
    }

    let parent = file_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_string());

    let mut guard = reserved.lock().unwrap_or_else(|e| e.into_inner());
    let mut candidate = parent.join(format!("{stem}.{ext}"));
    let mut counter = 1u32;

    loop {
        let key = normalize_key(&candidate);
        if !candidate.exists() && !guard.contains(&key) {
            guard.insert(key);
            return candidate;
        }
        candidate = parent.join(format!("{stem} ({counter}).{ext}"));
        counter += 1;
    }
}

fn open_standard_image(file_path: &Path) -> Result<DynamicImage, Box<dyn Error>> {
    let mut decoder = ImageReader::open(file_path)?
        .with_guessed_format()?
        .into_decoder()?;

    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)?;
    img.apply_orientation(orientation);

    Ok(img)
}

fn open_heic_image(file_path: &Path) -> Result<DynamicImage, Box<dyn Error>> {
    let bytes = fs::read(file_path)?;
    let ctx = HeifContext::read_from_bytes(&bytes)?;
    let handle = ctx.primary_image_handle()?;

    let has_alpha = handle.has_alpha_channel();
    let (chroma, channels) = if has_alpha {
        (RgbChroma::Rgba, 4usize)
    } else {
        (RgbChroma::Rgb, 3usize)
    };

    let lib_heif = LibHeif::new();
    let heif_img = lib_heif.decode(&handle, ColorSpace::Rgb(chroma), None)?;

    let planes = heif_img.planes();
    let interleaved = planes
        .interleaved
        .ok_or("Не удалось получить пиксельные данные из HEIC/AVIF")?;

    let width = interleaved.width;
    let height = interleaved.height;
    let stride = interleaved.stride;
    let data = interleaved.data;

    let row_bytes = (width as usize)
        .checked_mul(channels)
        .ok_or("Переполнение размера строки изображения")?;
    let total_bytes = row_bytes
        .checked_mul(height as usize)
        .ok_or("Переполнение размера буфера изображения")?;

    let mut buf = Vec::with_capacity(total_bytes);
    for y in 0..(height as usize) {
        let start = y * stride;
        let row = data
            .get(start..start + row_bytes)
            .ok_or("Некорректный размер буфера HEIC/AVIF")?;
        buf.extend_from_slice(row);
    }

    if has_alpha {
        let rgba_img = RgbaImage::from_raw(width, height, buf)
            .ok_or("Не удалось собрать RGBA-изображение из буфера HEIC/AVIF")?;
        Ok(DynamicImage::ImageRgba8(rgba_img))
    } else {
        let rgb_img = RgbImage::from_raw(width, height, buf)
            .ok_or("Не удалось собрать RGB-изображение из буфера HEIC/AVIF")?;
        Ok(DynamicImage::ImageRgb8(rgb_img))
    }
}

fn open_any_image(file_path: &Path) -> Result<DynamicImage, Box<dyn Error>> {
    match open_standard_image(file_path) {
        Ok(img) => Ok(img),
        Err(std_err) => open_heic_image(file_path).map_err(|heic_err| {
            format!("Не удалось открыть файл ({std_err}; HEIC: {heic_err})").into()
        }),
    }
}

/// Накладывает прозрачное изображение на белый фон (только для JPEG)
fn flatten_to_rgb_white(img: &DynamicImage) -> RgbImage {
    if !img.color().has_alpha() {
        return img.to_rgb8();
    }

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut rgb = RgbImage::new(w, h);

    for (src, dst) in rgba.pixels().zip(rgb.pixels_mut()) {
        let a = src[3] as u16;
        let inv_a = 255 - a;
        let r = ((src[0] as u16 * a + 255 * inv_a + 127) / 255) as u8;
        let g = ((src[1] as u16 * a + 255 * inv_a + 127) / 255) as u8;
        let b = ((src[2] as u16 * a + 255 * inv_a + 127) / 255) as u8;
        *dst = Rgb([r, g, b]);
    }

    rgb
}

fn encode_image_to_path(
    img: &DynamicImage,
    path: &Path,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let quality = config.quality.clamp(1, 100);
    let has_alpha = img.color().has_alpha();

    match config.format {
        OutputFormat::Jpg => {
            let rgb = flatten_to_rgb_white(img);
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            let encoder = JpegEncoder::new_with_quality(&mut writer, quality);
            rgb.write_with_encoder(encoder)?;
            writer.flush()?;
        }
        OutputFormat::Png => {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            let encoder = PngEncoder::new(&mut writer);
            if has_alpha {
                img.to_rgba8().write_with_encoder(encoder)?;
            } else {
                img.to_rgb8().write_with_encoder(encoder)?;
            }
            writer.flush()?;
        }
        OutputFormat::Webp => {
            let webp_data = if has_alpha {
                let rgba = img.to_rgba8();
                webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height())
                    .encode(quality as f32)
            } else {
                let rgb = img.to_rgb8();
                webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height())
                    .encode(quality as f32)
            };
            fs::write(path, &*webp_data)?;
        }
        OutputFormat::Avif => {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            // speed = 6: оптимальный баланс скорости и степени сжатия
            let encoder = AvifEncoder::new_with_speed_quality(&mut writer, 6, quality);
            if has_alpha {
                img.to_rgba8().write_with_encoder(encoder)?;
            } else {
                img.to_rgb8().write_with_encoder(encoder)?;
            }
            writer.flush()?;
        }
    }

    Ok(())
}

/// Сохраняет изображение через временный файл в той же папке и атомарно заменяет цель,
/// чтобы при сбое записи оригинал не был повреждён.
fn save_image_atomic(
    img: &DynamicImage,
    target_path: &Path,
    config: &AppConfig,
) -> Result<(), Box<dyn Error>> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new(""));
    let seq = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = parent.join(format!(".imageresizej_tmp_{}_{}", std::process::id(), seq));

    if let Err(err) = encode_image_to_path(img, &tmp_path, config) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }

    if let Err(err) = fs::rename(&tmp_path, target_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err.into());
    }

    Ok(())
}

fn process_single(
    file_path: &Path,
    config: &AppConfig,
    reserved: &Mutex<HashSet<String>>,
) -> Result<PathBuf, Box<dyn Error>> {
    if !file_path.is_file() {
        return Err("Файл не найден".into());
    }

    let img = open_any_image(file_path)?;

    let w = img.width();
    let h = img.height();
    if w == 0 || h == 0 {
        return Err("Изображение имеет нулевой размер".into());
    }

    let target_w = config.target_width.max(1);
    let resized = if config.no_upscale && w <= target_w {
        img
    } else {
        let new_height = (((target_w as f64 / w as f64) * h as f64).round() as u32).max(1);
        img.resize_exact(target_w, new_height, FilterType::Lanczos3)
    };

    let new_path = reserve_output_path(file_path, config, reserved);
    save_image_atomic(&resized, &new_path, config)?;

    if config.delete_originals && normalize_key(file_path) != normalize_key(&new_path) {
        let _ = fs::remove_file(file_path);
    }

    Ok(new_path)
}

fn append_error_log(file_path: &Path, err_msg: &str) -> Option<PathBuf> {
    let mut candidates = Vec::with_capacity(2);
    if let Some(parent) = file_path.parent() {
        candidates.push(parent.join("log.txt"));
    }
    candidates.push(AppConfig::webview_data_dir("logs").join("log.txt"));

    let line = format!("Ошибка с файлом {}: {}", file_path.display(), err_msg);
    for log_path in candidates {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path)
            && writeln!(file, "{line}").is_ok()
        {
            return Some(log_path);
        }
    }

    None
}

/// Многопоточно обрабатывает список файлов согласно настройкам
pub fn process_batch(paths: &[PathBuf], config: &AppConfig) -> BatchSummary {
    let start = Instant::now();

    // Убираем дубликаты путей, чтобы два потока не перезаписывали один файл одновременно
    let mut seen = HashSet::with_capacity(paths.len());
    let unique_paths: Vec<&PathBuf> = paths
        .iter()
        .filter(|p| seen.insert(normalize_key(p)))
        .collect();

    let mut initial_reserved = HashSet::new();
    if config.delete_originals {
        let ext = config.format.extension();
        for path in &unique_paths {
            if is_same_extension(path, ext) {
                initial_reserved.insert(normalize_key(path));
            }
        }
    }
    let reserved = Mutex::new(initial_reserved);

    let results: Vec<Result<PathBuf, (&Path, String)>> = unique_paths
        .par_iter()
        .map(|path| {
            process_single(path, config, &reserved).map_err(|e| (path.as_path(), e.to_string()))
        })
        .collect();

    let mut succeeded = 0;
    let mut errors = Vec::new();
    let mut log_path = None;

    for res in results {
        match res {
            Ok(_) => succeeded += 1,
            Err((path, err_msg)) => {
                let written = append_error_log(path, &err_msg);
                if log_path.is_none() {
                    log_path = written;
                }
                errors.push(format!("{}: {}", path.display(), err_msg));
            }
        }
    }

    BatchSummary {
        total: unique_paths.len(),
        succeeded,
        failed: errors.len(),
        elapsed: start.elapsed(),
        errors,
        log_path,
    }
}
