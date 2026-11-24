//! RustMatte Desktop Application Backend (Tauri 2).

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use rustmatte_core::model::{check_model_status, default_cache_dir, spec_for_tier, ModelStatus};
use rustmatte_core::{
    download_model, Config, ExecutionProvider, ModelTier, RefineConfig, Remover, Warning,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelCacheStatus {
    pub quality_cached: bool,
    pub fast_cached: bool,
    pub quality_size_mb: f64,
    pub fast_size_mb: f64,
}

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub input_path: String,
    pub tier: String,
    pub device: String,
    pub max_side: Option<u32>,
    pub no_refine: bool,
}

#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub original_data_url: String,
    pub cutout_data_url: String,
    pub width: u32,
    pub height: u32,
    pub device: String,
    pub total_ms: f64,
    pub inference_ms: f64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressEvent {
    pub tier: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchItemEvent {
    pub index: usize,
    pub total: usize,
    pub path: String,
    pub status: String,
    pub duration_ms: f64,
    pub error: Option<String>,
}

fn parse_tier(tier_str: &str) -> ModelTier {
    match tier_str.to_ascii_lowercase().as_str() {
        "fast" => ModelTier::Fast,
        _ => ModelTier::Quality,
    }
}

fn parse_device(device_str: &str) -> ExecutionProvider {
    ExecutionProvider::from_str(device_str).unwrap_or(ExecutionProvider::Cpu)
}

fn file_to_data_url(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read source file: {e}"))?;
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    Ok(format!(
        "data:{mime};base64,{}",
        BASE64_STANDARD.encode(&bytes)
    ))
}

fn rgba_to_data_url(img: &rustmatte_core::RgbaImage) -> Result<String, String> {
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode cutout PNG: {e}"))?;
    Ok(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(&buf)
    ))
}

mod commands {
    use super::*;

    #[tauri::command]
    pub fn get_model_cache_status() -> Result<ModelCacheStatus, String> {
        let quality_spec = spec_for_tier(ModelTier::Quality);
        let fast_spec = spec_for_tier(ModelTier::Fast);

        let quality_status = check_model_status(ModelTier::Quality);
        let fast_status = check_model_status(ModelTier::Fast);

        Ok(ModelCacheStatus {
            quality_cached: matches!(quality_status, ModelStatus::Verified { .. }),
            fast_cached: matches!(fast_status, ModelStatus::Verified { .. }),
            quality_size_mb: quality_spec.expected_size as f64 / (1024.0 * 1024.0),
            fast_size_mb: fast_spec.expected_size as f64 / (1024.0 * 1024.0),
        })
    }

    #[tauri::command]
    pub async fn download_model_tier(app: AppHandle, tier: String) -> Result<String, String> {
        let model_tier = parse_tier(&tier);
        let spec = spec_for_tier(model_tier);
        let cache_dir = default_cache_dir();

        let tier_clone = tier.clone();
        let app_clone = app.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let mut last_pct = 255;
            let downloaded = download_model(spec, &cache_dir, |dl, total| {
                let pct = if total > 0 {
                    (dl as f64 / total as f64 * 100.0) as u8
                } else {
                    0
                };
                if pct != last_pct {
                    last_pct = pct;
                    let _ = app_clone.emit(
                        "download_progress",
                        DownloadProgressEvent {
                            tier: tier_clone.clone(),
                            downloaded_bytes: dl,
                            total_bytes: total,
                            percentage: pct,
                        },
                    );
                }
            })
            .map_err(|e| e.to_string())?;

            Ok(downloaded.to_string_lossy().to_string())
        })
        .await
        .map_err(|e| format!("Download task join error: {e}"))?
    }

    #[tauri::command]
    pub async fn process_single_image(req: ProcessRequest) -> Result<ProcessResponse, String> {
        tauri::async_runtime::spawn_blocking(move || {
            let t_total = Instant::now();
            let path = PathBuf::from(&req.input_path);
            if !path.exists() {
                return Err(format!("Input file '{}' does not exist", req.input_path));
            }

            let tier = parse_tier(&req.tier);
            let ep = parse_device(&req.device);

            let config = Config {
                tier,
                model_path: None,
                threads: rustmatte_core::session::default_thread_count(),
                refine: RefineConfig {
                    enable_guided_filter: !req.no_refine,
                    enable_defringing: !req.no_refine,
                    ..Default::default()
                },
                execution_provider: ep,
            };

            let mut remover = Remover::new(config).map_err(|e| e.to_string())?;
            let output = remover
                .process_with_options(&path, req.max_side)
                .map_err(|e| e.to_string())?;

            let (width, height) = output.image.dimensions();
            let original_data_url = file_to_data_url(&path)?;
            let cutout_data_url = rgba_to_data_url(&output.image)?;

            let warnings = output
                .warnings
                .into_iter()
                .map(|w| match w {
                    Warning::DamagedInput(msg) => format!("Warning: {msg}"),
                })
                .collect();

            Ok(ProcessResponse {
                original_data_url,
                cutout_data_url,
                width,
                height,
                device: remover.execution_provider().to_string(),
                total_ms: t_total.elapsed().as_secs_f64() * 1000.0,
                inference_ms: output.timings.inference_ms,
                warnings,
            })
        })
        .await
        .map_err(|e| format!("Processing thread panic: {e}"))?
    }

    #[tauri::command]
    pub fn save_cutout_png(data_url: String, destination_path: String) -> Result<(), String> {
        let dest = PathBuf::from(destination_path);
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let b64 = data_url
            .split(',')
            .nth(1)
            .ok_or_else(|| "Invalid data URL format".to_string())?;

        let bytes = BASE64_STANDARD
            .decode(b64.trim())
            .map_err(|e| format!("Base64 decode error: {e}"))?;

        std::fs::write(&dest, bytes).map_err(|e| format!("Failed to write file to disk: {e}"))?;
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_model_cache_status,
            commands::download_model_tier,
            commands::process_single_image,
            commands::save_cutout_png
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
