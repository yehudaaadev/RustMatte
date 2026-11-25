//! RustMatte Desktop Application Backend (Tauri 2).
//!
//! Thin Tauri command layer around `rustmatte-core`: no image processing logic lives here.

mod preview;
mod settings;
mod state;
mod validation;

#[cfg(all(not(debug_assertions), dev, not(test)))]
compile_error!(
    "Release builds of `rustmatte-app` require Tauri's `custom-protocol` feature so embedded frontend assets are served instead of `devUrl` (`http://localhost:1420`). Build with `npx tauri build` inside `crates/app`, or `cargo build --release -p rustmatte-app --features custom-protocol` after `npm run build`."
);

#[used]
static RUSTMATTE_BUILD_MODE_MARKER: &[u8] = if cfg!(dev) {
    b"RUSTMATTE_BUILD_MODE=dev-url\0"
} else {
    b"RUSTMATTE_BUILD_MODE=custom-protocol\0"
};

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State, WindowEvent};

use rustmatte_core::decode::decode_image;
use rustmatte_core::model::{
    check_model_status, default_cache_dir, spec_for_tier, verify_model_checksum, ModelStatus,
};
use rustmatte_core::pipeline::downscale_if_needed;
use rustmatte_core::{
    download_model as core_download_model, extract_source_host, ExecutionProvider, ModelTier,
    Warning,
};

use crate::settings::AppSettings;
use crate::state::{AppState, RefineLevel, SessionOptions};
use crate::validation::ExpandedInputs;

const SHIPPED_THIRD_PARTY_LICENSES: &str = include_str!("../../../../THIRD_PARTY_LICENSES.md");
const SHIPPED_MODELS_MD: &str = include_str!("../../../../MODELS.md");
const HUGE_IMAGE_PIXELS_THRESHOLD: u64 = 16_000_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelTierStatus {
    pub tier: String,
    pub name: String,
    pub filename: String,
    pub cached: bool,
    pub expected_size_mb: f64,
    pub license: String,
    pub primary_host: String,
    pub fallback_host: String,
    pub caveat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsStatusResponse {
    pub quality: ModelTierStatus,
    pub fast: ModelTierStatus,
    pub cache_dir: String,
}

#[derive(Debug, Serialize)]
pub struct AboutInfo {
    pub version: String,
    pub license: String,
    pub models_markdown: String,
    pub third_party_licenses_markdown: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticInfo {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub total_ram_mb: Option<u64>,
    pub available_ram_mb: Option<u64>,
    pub tier: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessImagesOptions {
    pub tier: String,
    pub device: String,
    pub max_side: Option<u32>,
    pub refine_level: String,
    pub output_folder: Option<String>,
    pub background_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressEvent {
    pub tier: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobStatusEvent {
    pub job_id: u64,
    pub phase: String,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemWarning {
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobProgressEvent {
    pub job_id: u64,
    pub index: usize,
    pub total: usize,
    pub file: String,
    pub input_path: String,
    pub status: String,
    pub timing_ms: f64,
    pub inference_ms: f64,
    pub warnings: Vec<ItemWarning>,
    pub oom_warning: bool,
    pub error: Option<String>,
    pub original_preview_path: Option<String>,
    pub cutout_preview_path: Option<String>,
    pub full_cutout_path: Option<String>,
    pub width: u32,
    pub height: u32,
    pub device: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobFinishedEvent {
    pub job_id: u64,
    pub processed: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub total_ms: f64,
    pub output_dir: Option<String>,
    pub display_output_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveAllItem {
    pub source_cutout_path: String,
    pub input_path: String,
}

#[derive(Debug, Serialize)]
pub struct SaveSingleResult {
    pub path: String,
    pub display_path: String,
}

#[derive(Debug, Serialize)]
pub struct SaveAllResult {
    pub saved_count: usize,
    pub output_dir: String,
    pub display_output_dir: String,
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

fn preview_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to resolve app cache directory: {e}"))?;
    Ok(base.join("previews"))
}

fn settings_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve app config directory: {e}"))?;
    Ok(base.join("settings.json"))
}

fn startup_log_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to resolve app cache directory: {e}"))?;
    Ok(base.join("startup.log"))
}

fn sanitize_origin(origin: &str) -> String {
    let Some((scheme, rest)) = origin.trim().split_once("://") else {
        return "unknown".to_string();
    };
    let clean_scheme: String = scheme
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '-' || *c == '.')
        .take(16)
        .collect();
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or("");
    let clean_host: String = host_port
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric()
                || *c == '.'
                || *c == '-'
                || *c == ':'
                || *c == '['
                || *c == ']'
        })
        .take(128)
        .collect();
    if clean_scheme.is_empty() || clean_host.is_empty() {
        "unknown".to_string()
    } else {
        format!("{clean_scheme}://{clean_host}")
    }
}

fn tier_status_from_spec(tier: ModelTier, caveat: Option<String>) -> ModelTierStatus {
    let spec = spec_for_tier(tier);
    let status = check_model_status(tier);
    let primary_host = spec
        .source_urls
        .first()
        .map(|u| extract_source_host(u))
        .unwrap_or_else(|| "github.com".to_string());
    let fallback_host = spec
        .source_urls
        .get(1)
        .map(|u| extract_source_host(u))
        .unwrap_or_else(|| "huggingface.co".to_string());

    ModelTierStatus {
        tier: match tier {
            ModelTier::Quality => "quality".to_string(),
            ModelTier::Fast => "fast".to_string(),
        },
        name: spec.name.to_string(),
        filename: spec.default_filename.to_string(),
        cached: matches!(status, ModelStatus::Verified { .. }),
        expected_size_mb: spec.expected_size as f64 / (1024.0 * 1024.0),
        license: spec.license.to_string(),
        primary_host,
        fallback_host,
        caveat,
    }
}

fn system_ram_mb() -> (Option<u64>, Option<u64>) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GlobalMemoryStatusEx(lp_buffer: *mut MemoryStatusEx) -> i32;
        }

        let mut mem = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            dw_memory_load: 0,
            ull_total_phys: 0,
            ull_avail_phys: 0,
            ull_total_page_file: 0,
            ull_avail_page_file: 0,
            ull_total_virtual: 0,
            ull_avail_virtual: 0,
            ull_avail_extended_virtual: 0,
        };
        // SAFETY: `mem` has exact `dw_length` set for `GlobalMemoryStatusEx`.
        let ok = unsafe { GlobalMemoryStatusEx(&mut mem) };
        if ok != 0 {
            return (
                Some(mem.ull_total_phys / (1024 * 1024)),
                Some(mem.ull_avail_phys / (1024 * 1024)),
            );
        }
        (None, None)
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            let mut total_kb = None;
            let mut avail_kb = None;
            for line in meminfo.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    total_kb = rest
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse::<u64>().ok());
                } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                    avail_kb = rest
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse::<u64>().ok());
                }
            }
            return (total_kb.map(|k| k / 1024), avail_kb.map(|k| k / 1024));
        }
        (None, None)
    }

    #[cfg(not(any(windows, target_os = "linux")))]
    {
        (None, None)
    }
}

fn export_cutout_to_dest(
    src: &std::path::Path,
    dest: &std::path::Path,
    bg_hex: Option<&str>,
) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create destination folder: {e}"))?;
    }

    if let Some(hex) = bg_hex {
        let img = image::open(src)
            .map_err(|e| format!("Failed to read processed PNG for background fill: {e}"))?
            .to_rgba8();
        let composited = preview::composite_solid_background(&img, hex)?;
        preview::write_full_png_atomic(dest, &composited)
    } else {
        let tmp = dest.with_extension("png.tmp");
        if let Err(e) = std::fs::copy(src, &tmp) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("Failed to copy cutout PNG: {e}"));
        }
        if let Err(e) = std::fs::rename(&tmp, dest) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("Failed to finalize saved PNG: {e}"));
        }
        Ok(())
    }
}

mod commands {
    use super::*;

    #[tauri::command]
    pub fn get_models_status() -> Result<ModelsStatusResponse, String> {
        Ok(ModelsStatusResponse {
            quality: tier_status_from_spec(ModelTier::Quality, None),
            fast: tier_status_from_spec(
                ModelTier::Fast,
                Some(
                    "Best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds."
                        .to_string(),
                ),
            ),
            cache_dir: default_cache_dir().to_string_lossy().to_string(),
        })
    }

    #[tauri::command]
    pub fn expand_inputs(paths: Vec<String>) -> Result<ExpandedInputs, String> {
        Ok(validation::expand_input_paths(&paths))
    }

    #[tauri::command]
    pub async fn download_model(
        app: AppHandle,
        tier: String,
        consent: bool,
    ) -> Result<String, String> {
        if !consent {
            return Err("Model download requires explicit user consent".to_string());
        }

        let settings_path = settings_file_path(&app)?;
        let current_settings = settings::load_settings_from_path(&settings_path);
        if current_settings.never_use_network {
            return Err(
                "Offline lock is enabled ('Never use the network'). Disable it in Settings or choose a local .onnx file."
                    .to_string(),
            );
        }

        let model_tier = parse_tier(&tier);
        let spec = spec_for_tier(model_tier);
        let cache_dir = default_cache_dir();

        let tier_clone = tier.clone();
        let app_clone = app.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let mut last_pct = 255;
            let downloaded = core_download_model(spec, &cache_dir, |dl, total| {
                let pct = if total > 0 {
                    ((dl as f64 / total as f64) * 100.0) as u8
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
        .map_err(|e| format!("Download task failed: {e}"))?
    }

    #[tauri::command]
    pub async fn import_model_file(tier: String, file_path: String) -> Result<String, String> {
        let model_tier = parse_tier(&tier);
        let spec = spec_for_tier(model_tier);
        let cache_dir = default_cache_dir();

        tauri::async_runtime::spawn_blocking(move || {
            let src = std::fs::canonicalize(file_path.trim())
                .map_err(|e| format!("Selected model file could not be accessed: {e}"))?;
            if !src.is_file() {
                return Err("Selected model path is not a file".to_string());
            }

            verify_model_checksum(&src, spec.expected_sha256).map_err(|e| {
                format!(
                    "Selected file does not match {} (expected SHA-256 {}): {e}",
                    spec.default_filename, spec.expected_sha256
                )
            })?;

            std::fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to create model cache directory: {e}"))?;
            let dest = cache_dir.join(spec.default_filename);
            let tmp = cache_dir.join(format!(".{}.import.tmp", spec.default_filename));
            if let Err(e) = std::fs::copy(&src, &tmp) {
                let _ = std::fs::remove_file(&tmp);
                return Err(format!("Failed to copy model into local cache: {e}"));
            }
            if let Err(e) = std::fs::rename(&tmp, &dest) {
                let _ = std::fs::remove_file(&tmp);
                return Err(format!("Failed to finalize imported model: {e}"));
            }

            Ok(dest.to_string_lossy().to_string())
        })
        .await
        .map_err(|e| format!("Model import failed: {e}"))?
    }

    #[tauri::command]
    pub async fn process_images(
        app: AppHandle,
        state: State<'_, AppState>,
        paths: Vec<String>,
        options: ProcessImagesOptions,
    ) -> Result<u64, String> {
        if paths.is_empty() {
            return Err("No input images provided".to_string());
        }

        let mut validated_inputs = Vec::with_capacity(paths.len());
        for raw in &paths {
            validated_inputs.push(validation::validate_input_path(raw)?);
        }

        let resolved_out_dir = match options.output_folder.as_deref() {
            Some(folder) if !folder.trim().is_empty() => {
                Some(validation::validate_output_dir(folder)?)
            }
            _ => {
                if validated_inputs.len() > 1 {
                    let parent = validated_inputs[0].parent().ok_or_else(|| {
                        "Cannot determine parent directory for output".to_string()
                    })?;
                    Some(parent.join("rustmatte-output"))
                } else {
                    None
                }
            }
        };

        let batch_outputs = if let Some(ref out_dir) = resolved_out_dir {
            Some(validation::resolve_batch_output_paths(
                &validated_inputs,
                out_dir,
            )?)
        } else {
            None
        };

        if let Some(ref hex) = options.background_hex {
            let _ = preview::parse_hex_rgb(hex)?;
        }

        let (job_id, cancel_flag) = {
            let mut coord = state
                .job_coordinator
                .lock()
                .map_err(|_| "Job coordinator lock poisoned".to_string())?;
            coord.try_start()?
        };

        let preview_dir = preview_cache_dir(&app)?;
        let tier = parse_tier(&options.tier);
        let ep = parse_device(&options.device);
        let refine_level = RefineLevel::parse(&options.refine_level);
        let max_side = options.max_side;
        let bg_hex = options.background_hex.clone();
        let app_clone = app.clone();

        tauri::async_runtime::spawn_blocking(move || {
            let t_job = Instant::now();
            let total = validated_inputs.len();
            let mut processed = 0usize;
            let mut failed = 0usize;
            let mut cancelled = false;

            let session_opts = SessionOptions {
                execution_provider: ep,
                refine_level,
                threads: rustmatte_core::session::default_thread_count(),
            };

            let app_state = app_clone.state::<AppState>();
            let needs_model_load = app_state
                .session_cache
                .lock()
                .map(|c| !c.is_loaded(tier, &session_opts))
                .unwrap_or(true);

            if needs_model_load {
                let _ = app_clone.emit(
                    "job_status",
                    JobStatusEvent {
                        job_id,
                        phase: "loading_model".to_string(),
                        tier: options.tier.clone(),
                    },
                );
            }

            for (idx, input_path) in validated_inputs.iter().enumerate() {
                if cancel_flag.load(Ordering::SeqCst) {
                    cancelled = true;
                    break;
                }

                let t_item = Instant::now();
                let file_name = input_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("image")
                    .to_string();

                let mut item_warnings: Vec<ItemWarning> = Vec::new();
                let oom_warning = image::image_dimensions(input_path)
                    .map(|(w, h)| {
                        (w as u64) * (h as u64) >= HUGE_IMAGE_PIXELS_THRESHOLD && max_side.is_none()
                    })
                    .unwrap_or(false);

                let original_preview_path = match decode_image(input_path) {
                    Ok((decoded, decode_warnings)) => {
                        for w in decode_warnings {
                            let Warning::DamagedInput(msg) = w;
                            item_warnings.push(ItemWarning {
                                code: "damaged_input".to_string(),
                                detail: msg,
                            });
                        }
                        let preview_img = downscale_if_needed(decoded, max_side);
                        let res = preview::write_preview_image(&preview_dir, "orig", &preview_img);
                        drop(preview_img);
                        res.ok()
                    }
                    Err(_) => None,
                };

                let result = {
                    let mut cache_guard = match app_state.session_cache.lock() {
                        Ok(g) => g,
                        Err(_) => {
                            failed += 1;
                            let _ = app_clone.emit(
                                "job_progress",
                                JobProgressEvent {
                                    job_id,
                                    index: idx + 1,
                                    total,
                                    file: file_name,
                                    input_path: input_path.to_string_lossy().to_string(),
                                    status: "failed".to_string(),
                                    timing_ms: t_item.elapsed().as_secs_f64() * 1000.0,
                                    inference_ms: 0.0,
                                    warnings: item_warnings,
                                    oom_warning,
                                    error: Some("Internal session lock error".to_string()),
                                    original_preview_path: None,
                                    cutout_preview_path: None,
                                    full_cutout_path: None,
                                    width: 0,
                                    height: 0,
                                    device: String::new(),
                                },
                            );
                            continue;
                        }
                    };

                    let loaded = cache_guard.get_or_load(tier, session_opts.clone());
                    let _ = app_clone.emit(
                        "job_status",
                        JobStatusEvent {
                            job_id,
                            phase: "processing".to_string(),
                            tier: options.tier.clone(),
                        },
                    );
                    match loaded {
                        Ok(remover) => remover
                            .process_with_options(input_path, max_side)
                            .map(|out| (out, remover.execution_provider().to_string()))
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(e),
                    }
                };

                match result {
                    Ok((output, active_device)) => {
                        for w in output.warnings {
                            let Warning::DamagedInput(msg) = w;
                            if !item_warnings.iter().any(|existing| {
                                existing.code == "damaged_input" && existing.detail == msg
                            }) {
                                item_warnings.push(ItemWarning {
                                    code: "damaged_input".to_string(),
                                    detail: msg,
                                });
                            }
                        }

                        let (width, height) = output.image.dimensions();

                        let full_dest = if let Some(ref outs) = batch_outputs {
                            outs[idx].clone()
                        } else {
                            preview_dir.join(format!("full_{}_{}.png", job_id, idx + 1))
                        };

                        let full_write = if batch_outputs.is_some() {
                            if let Some(ref hex) = bg_hex {
                                match preview::composite_solid_background(&output.image, hex) {
                                    Ok(filled) => {
                                        preview::write_full_png_atomic(&full_dest, &filled)
                                    }
                                    Err(e) => Err(e),
                                }
                            } else {
                                preview::write_full_png_atomic(&full_dest, &output.image)
                            }
                        } else {
                            preview::write_full_png_atomic(&full_dest, &output.image)
                        };

                        let preview_dyn = image::DynamicImage::ImageRgba8(output.image);
                        let cutout_preview =
                            preview::write_preview_image(&preview_dir, "cutout", &preview_dyn);
                        drop(preview_dyn);

                        match (full_write, cutout_preview) {
                            (Ok(()), Ok(cutout_prev_path)) => {
                                processed += 1;
                                let status = if item_warnings.is_empty() {
                                    "done"
                                } else {
                                    "warning"
                                };
                                let _ = app_clone.emit(
                                    "job_progress",
                                    JobProgressEvent {
                                        job_id,
                                        index: idx + 1,
                                        total,
                                        file: file_name,
                                        input_path: input_path.to_string_lossy().to_string(),
                                        status: status.to_string(),
                                        timing_ms: t_item.elapsed().as_secs_f64() * 1000.0,
                                        inference_ms: output.timings.inference_ms,
                                        warnings: item_warnings,
                                        oom_warning,
                                        error: None,
                                        original_preview_path: original_preview_path
                                            .map(|p| p.to_string_lossy().to_string()),
                                        cutout_preview_path: Some(
                                            cutout_prev_path.to_string_lossy().to_string(),
                                        ),
                                        full_cutout_path: Some(
                                            full_dest.to_string_lossy().to_string(),
                                        ),
                                        width,
                                        height,
                                        device: active_device,
                                    },
                                );
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                failed += 1;
                                let _ = app_clone.emit(
                                    "job_progress",
                                    JobProgressEvent {
                                        job_id,
                                        index: idx + 1,
                                        total,
                                        file: file_name,
                                        input_path: input_path.to_string_lossy().to_string(),
                                        status: "failed".to_string(),
                                        timing_ms: t_item.elapsed().as_secs_f64() * 1000.0,
                                        inference_ms: output.timings.inference_ms,
                                        warnings: item_warnings,
                                        oom_warning,
                                        error: Some(e),
                                        original_preview_path: None,
                                        cutout_preview_path: None,
                                        full_cutout_path: None,
                                        width: 0,
                                        height: 0,
                                        device: active_device,
                                    },
                                );
                            }
                        }
                    }
                    Err(err_msg) => {
                        failed += 1;
                        let _ = app_clone.emit(
                            "job_progress",
                            JobProgressEvent {
                                job_id,
                                index: idx + 1,
                                total,
                                file: file_name,
                                input_path: input_path.to_string_lossy().to_string(),
                                status: "failed".to_string(),
                                timing_ms: t_item.elapsed().as_secs_f64() * 1000.0,
                                inference_ms: 0.0,
                                warnings: item_warnings,
                                oom_warning,
                                error: Some(err_msg),
                                original_preview_path: None,
                                cutout_preview_path: None,
                                full_cutout_path: None,
                                width: 0,
                                height: 0,
                                device: String::new(),
                            },
                        );
                    }
                }
            }

            if let Ok(mut coord) = app_state.job_coordinator.lock() {
                coord.finish(job_id);
            }

            let out_dir_str = resolved_out_dir.map(|p| p.to_string_lossy().to_string());
            let display_out_dir_str = out_dir_str.as_deref().map(validation::format_display_path);
            let _ = app_clone.emit(
                "job_finished",
                JobFinishedEvent {
                    job_id,
                    processed,
                    failed,
                    cancelled,
                    total_ms: t_job.elapsed().as_secs_f64() * 1000.0,
                    output_dir: out_dir_str,
                    display_output_dir: display_out_dir_str,
                },
            );
        });

        Ok(job_id)
    }

    #[tauri::command]
    pub fn cancel_job(state: State<'_, AppState>) -> Result<bool, String> {
        let coord = state
            .job_coordinator
            .lock()
            .map_err(|_| "Job coordinator lock poisoned".to_string())?;
        Ok(coord.request_cancel())
    }

    #[tauri::command]
    pub async fn save_cutout_png(
        source_cutout_path: String,
        destination_path: String,
        input_paths: Vec<String>,
        background_hex: Option<String>,
    ) -> Result<SaveSingleResult, String> {
        tauri::async_runtime::spawn_blocking(move || {
            let src = std::fs::canonicalize(source_cutout_path.trim())
                .map_err(|e| format!("Cutout file is missing or inaccessible: {e}"))?;
            if !src.is_file() {
                return Err("Cutout source is not a file".to_string());
            }

            let input_bufs: Vec<PathBuf> = input_paths.into_iter().map(PathBuf::from).collect();
            let dest = validation::validate_save_destination(&destination_path, &input_bufs)?;

            export_cutout_to_dest(&src, &dest, background_hex.as_deref())?;
            let path = dest.to_string_lossy().to_string();
            let display_path = validation::format_display_path(&path);
            Ok(SaveSingleResult { path, display_path })
        })
        .await
        .map_err(|e| format!("Save task failed: {e}"))?
    }

    #[tauri::command]
    pub async fn save_all_cutouts(
        items: Vec<SaveAllItem>,
        output_dir: String,
        background_hex: Option<String>,
    ) -> Result<SaveAllResult, String> {
        if items.is_empty() {
            return Err("No completed cutouts to save".to_string());
        }

        tauri::async_runtime::spawn_blocking(move || {
            let dir = validation::validate_output_dir(&output_dir)?;
            let mut input_bufs = Vec::with_capacity(items.len());
            let mut sources = Vec::with_capacity(items.len());

            for item in &items {
                let input_canon = validation::validate_input_path(&item.input_path)?;
                let src_canon = std::fs::canonicalize(item.source_cutout_path.trim())
                    .map_err(|e| format!("Cutout file is missing or inaccessible: {e}"))?;
                input_bufs.push(input_canon);
                sources.push(src_canon);
            }

            let destinations = validation::resolve_batch_output_paths(&input_bufs, &dir)?;
            let mut saved_count = 0usize;
            for (src, dest) in sources.iter().zip(destinations.iter()) {
                export_cutout_to_dest(src, dest, background_hex.as_deref())?;
                saved_count += 1;
            }

            let output_dir_str = dir.to_string_lossy().to_string();
            let display_output_dir = validation::format_display_path(&output_dir_str);
            Ok(SaveAllResult {
                saved_count,
                output_dir: output_dir_str,
                display_output_dir,
            })
        })
        .await
        .map_err(|e| format!("Batch save task failed: {e}"))?
    }

    #[tauri::command]
    pub fn open_output_folder(path: String) -> Result<(), String> {
        let dir = validation::validate_existing_directory(&path)?;

        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = std::process::Command::new("explorer");
            c.arg(&dir);
            c
        };

        #[cfg(target_os = "macos")]
        let mut cmd = {
            let mut c = std::process::Command::new("open");
            c.arg(&dir);
            c
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        let mut cmd = {
            let mut c = std::process::Command::new("xdg-open");
            c.arg(&dir);
            c
        };

        cmd.spawn()
            .map_err(|e| format!("Failed to open output folder: {e}"))?;
        Ok(())
    }

    #[tauri::command]
    pub fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
        let path = settings_file_path(&app)?;
        Ok(settings::load_settings_from_path(&path))
    }

    #[tauri::command]
    pub fn set_settings(app: AppHandle, new_settings: AppSettings) -> Result<AppSettings, String> {
        let path = settings_file_path(&app)?;
        let sanitized = new_settings.sanitized();
        settings::save_settings_atomic(&path, &sanitized)?;
        Ok(sanitized)
    }

    #[tauri::command]
    pub fn get_about_info() -> AboutInfo {
        AboutInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            license: env!("CARGO_PKG_LICENSE").to_string(),
            models_markdown: SHIPPED_MODELS_MD.to_string(),
            third_party_licenses_markdown: SHIPPED_THIRD_PARTY_LICENSES.to_string(),
        }
    }

    #[tauri::command]
    pub fn get_diagnostic_info(tier: String) -> DiagnosticInfo {
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let (total_ram_mb, available_ram_mb) = system_ram_mb();
        DiagnosticInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores,
            total_ram_mb,
            available_ram_mb,
            tier,
        }
    }

    #[tauri::command]
    pub fn notify_frontend_ready(app: AppHandle, origin: String) -> Result<String, String> {
        let mode = if tauri::is_dev() {
            "dev-url"
        } else {
            "custom-protocol"
        };
        let safe_origin = crate::sanitize_origin(&origin);
        let line = format!("ui_ready mode={mode} origin={safe_origin}");
        eprintln!("{line}");
        if let Ok(path) = crate::startup_log_path(&app) {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, format!("{line}\n"));
        }
        Ok(line)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .setup(|app| {
            if let Ok(log_path) = startup_log_path(app.handle()) {
                let _ = std::fs::remove_file(&log_path);
            }
            if let Ok(cache_dir) = preview_cache_dir(app.handle()) {
                preview::cleanup_preview_cache(&cache_dir);
                let _ = std::fs::create_dir_all(&cache_dir);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_models_status,
            commands::expand_inputs,
            commands::download_model,
            commands::import_model_file,
            commands::process_images,
            commands::cancel_job,
            commands::save_cutout_png,
            commands::save_all_cutouts,
            commands::open_output_folder,
            commands::get_settings,
            commands::set_settings,
            commands::get_about_info,
            commands::get_diagnostic_info,
            commands::notify_frontend_ready
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        RunEvent::WindowEvent {
            event: WindowEvent::CloseRequested { .. },
            ..
        } => {
            let state = app_handle.state::<AppState>();
            if let Ok(coord) = state.job_coordinator.lock() {
                coord.request_cancel();
            };
        }
        RunEvent::Exit => {
            let state = app_handle.state::<AppState>();
            if let Ok(coord) = state.job_coordinator.lock() {
                coord.request_cancel();
            };
            if let Ok(cache_dir) = preview_cache_dir(app_handle) {
                preview::cleanup_preview_cache(&cache_dir);
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::sanitize_origin;

    #[test]
    fn sanitize_origin_strips_paths_queries_and_credentials() {
        assert_eq!(
            sanitize_origin("http://tauri.localhost"),
            "http://tauri.localhost"
        );
        assert_eq!(
            sanitize_origin("tauri://localhost/secret/path?user=alice#hash"),
            "tauri://localhost"
        );
        assert_eq!(
            sanitize_origin("http://user:pass@localhost:1420/app"),
            "http://localhost:1420"
        );
        assert_eq!(sanitize_origin("C:\\Users\\Alice\\file.html"), "unknown");
    }
}
