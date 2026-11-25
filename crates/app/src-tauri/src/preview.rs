//! Bounded preview cache manager and solid-background PNG export compositing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};

pub const MAX_PREVIEW_SIDE: u32 = 1600;
pub const DEFAULT_CACHE_MAX_BYTES: u64 = 128 * 1024 * 1024;
pub const DEFAULT_CACHE_MAX_FILES: usize = 120;

static PREVIEW_SEQ: AtomicU64 = AtomicU64::new(1);

/// Parses a `#RRGGBB` hex string into `[r, g, b]`.
pub fn parse_hex_rgb(hex: &str) -> Result<[u8; 3], String> {
    let trimmed = hex.trim();
    if trimmed.len() != 7 || !trimmed.starts_with('#') {
        return Err(format!(
            "Invalid background color '{trimmed}' (expected #RRGGBB)"
        ));
    }
    let r = u8::from_str_radix(&trimmed[1..3], 16)
        .map_err(|_| format!("Invalid red channel in '{trimmed}'"))?;
    let g = u8::from_str_radix(&trimmed[3..5], 16)
        .map_err(|_| format!("Invalid green channel in '{trimmed}'"))?;
    let b = u8::from_str_radix(&trimmed[5..7], 16)
        .map_err(|_| format!("Invalid blue channel in '{trimmed}'"))?;
    Ok([r, g, b])
}

/// Composites an RGBA cutout over a solid `#RRGGBB` background color.
pub fn composite_solid_background(rgba: &RgbaImage, hex: &str) -> Result<RgbaImage, String> {
    let [bg_r, bg_g, bg_b] = parse_hex_rgb(hex)?;
    let (w, h) = rgba.dimensions();
    let mut out = RgbaImage::new(w, h);
    for (x, y, px) in rgba.enumerate_pixels() {
        let a = px[3] as f32 / 255.0;
        let inv = 1.0 - a;
        let r = (px[0] as f32 * a + bg_r as f32 * inv)
            .round()
            .clamp(0.0, 255.0) as u8;
        let g = (px[1] as f32 * a + bg_g as f32 * inv)
            .round()
            .clamp(0.0, 255.0) as u8;
        let b = (px[2] as f32 * a + bg_b as f32 * inv)
            .round()
            .clamp(0.0, 255.0) as u8;
        out.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
    Ok(out)
}

/// Downscales an image if either side exceeds `max_side`, preserving aspect ratio.
#[must_use]
pub fn downscale_for_preview(img: &DynamicImage, max_side: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let longest = w.max(h);
    if longest <= max_side || max_side == 0 {
        return img.clone();
    }
    let scale = max_side as f64 / longest as f64;
    let new_w = ((w as f64 * scale).round() as u32).max(1);
    let new_h = ((h as f64 * scale).round() as u32).max(1);
    img.resize_exact(new_w, new_h, FilterType::Triangle)
}

/// Writes a preview PNG (max 1600px side) into `cache_dir` and enforces cache caps.
pub fn write_preview_image(
    cache_dir: &Path,
    prefix: &str,
    img: &DynamicImage,
) -> Result<PathBuf, String> {
    write_preview_image_with_limits(
        cache_dir,
        prefix,
        img,
        MAX_PREVIEW_SIDE,
        DEFAULT_CACHE_MAX_BYTES,
        DEFAULT_CACHE_MAX_FILES,
    )
}

pub fn write_preview_image_with_limits(
    cache_dir: &Path,
    prefix: &str,
    img: &DynamicImage,
    max_side: u32,
    max_bytes: u64,
    max_files: usize,
) -> Result<PathBuf, String> {
    fs::create_dir_all(cache_dir)
        .map_err(|e| format!("Failed to create preview cache directory: {e}"))?;

    let preview = downscale_for_preview(img, max_side);
    let seq = PREVIEW_SEQ.fetch_add(1, Ordering::Relaxed);
    let file_name = format!("{prefix}_{}_{}.png", std::process::id(), seq);
    let target = cache_dir.join(&file_name);
    let tmp = cache_dir.join(format!(".{file_name}.tmp"));

    if let Err(e) = preview.save_with_format(&tmp, ImageFormat::Png) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("Failed to write preview image: {e}"));
    }

    if let Err(e) = fs::rename(&tmp, &target) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("Failed to finalize preview image: {e}"));
    }

    enforce_cache_limits(cache_dir, max_bytes, max_files);
    Ok(target)
}

/// Atomically writes a full-resolution RGBA cutout PNG to `dest` via a temporary file.
pub fn write_full_png_atomic(dest: &Path, rgba: &RgbaImage) -> Result<(), String> {
    let parent = dest
        .parent()
        .ok_or_else(|| "Destination path has no parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create output directory: {e}"))?;

    let seq = PREVIEW_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = parent.join(format!(
        ".{}.{}.{}.tmp",
        dest.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("cutout"),
        std::process::id(),
        seq
    ));

    if let Err(e) = rgba.save_with_format(&tmp, ImageFormat::Png) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("Failed to encode PNG cutout: {e}"));
    }

    if let Err(e) = fs::rename(&tmp, dest) {
        let _ = fs::remove_file(&tmp);
        return Err(format!("Failed to write PNG cutout: {e}"));
    }

    Ok(())
}

/// Evicts oldest cached files when file count or total byte budget is exceeded.
pub fn enforce_cache_limits(cache_dir: &Path, max_bytes: u64, max_files: usize) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };

    let mut files: Vec<(PathBuf, u64, SystemTime)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            Some((path, meta.len(), modified))
        })
        .collect();

    files.sort_by_key(|(_, _, modified)| *modified);

    let mut total_bytes: u64 = files.iter().map(|(_, len, _)| *len).sum();
    let mut total_files = files.len();

    for (path, len, _) in files {
        if total_bytes <= max_bytes && total_files <= max_files {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total_bytes = total_bytes.saturating_sub(len);
            total_files = total_files.saturating_sub(1);
        }
    }
}

/// Deletes all preview/temp files inside `cache_dir` on startup or shutdown.
pub fn cleanup_preview_cache(cache_dir: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    fn temp_cache_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "rustmatte_app_preview_{label}_{}_{}",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn downscales_large_images_to_max_preview_side() {
        let large = DynamicImage::ImageRgba8(RgbaImage::new(3200, 1600));
        let resized = downscale_for_preview(&large, MAX_PREVIEW_SIDE);
        assert_eq!(resized.dimensions(), (1600, 800));

        let small = DynamicImage::ImageRgba8(RgbaImage::new(800, 600));
        let unchanged = downscale_for_preview(&small, MAX_PREVIEW_SIDE);
        assert_eq!(unchanged.dimensions(), (800, 600));
    }

    #[test]
    fn composites_solid_background_color_accurately() {
        let mut img = RgbaImage::new(2, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 0]));
        img.put_pixel(1, 0, Rgba([200, 100, 50, 255]));
        let out = composite_solid_background(&img, "#ff0080").unwrap();
        assert_eq!(out.get_pixel(0, 0), &Rgba([255, 0, 128, 255]));
        assert_eq!(out.get_pixel(1, 0), &Rgba([200, 100, 50, 255]));
        assert!(composite_solid_background(&img, "bad").is_err());
    }

    #[test]
    fn enforces_cache_cap_and_cleans_up_on_exit() {
        let dir = temp_cache_dir("cap");
        let img = DynamicImage::ImageRgba8(RgbaImage::new(64, 64));

        for _ in 0..5 {
            write_preview_image_with_limits(&dir, "test", &img, 1600, 10_000_000, 2).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let remaining = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();
        assert!(remaining <= 2);

        cleanup_preview_cache(&dir);
        let after_cleanup = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();
        assert_eq!(after_cleanup, 0);

        let _ = fs::remove_dir_all(&dir);
    }
}
