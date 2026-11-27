//! Input, destination, folder expansion, and batch output path validation helpers.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

const SUPPORTED_EXTENSIONS: &[&str] = rustmatte_core::decode::SUPPORTED_EXTENSIONS;

/// Strips Windows extended-length verbatim prefixes (`\\?\C:\...`, `\\?\UNC\...`) for UI display only.
/// Leaves internal canonical paths untouched for validation and filesystem calls.
#[must_use]
pub fn format_display_path(raw: &str) -> String {
    if raw.len() >= 8 && raw[..4].eq(r"\\?\") && raw[4..8].eq_ignore_ascii_case(r"UNC\") {
        return format!(r"\\{}", &raw[8..]);
    }
    if let Some(rest) = raw.strip_prefix(r"\\?\") {
        let bytes = rest.as_bytes();
        if bytes.len() >= 2
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes.len() == 2 || bytes[2] == b'\\' || bytes[2] == b'/')
        {
            return rest.to_string();
        }
    }
    raw.to_string()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RejectedInput {
    pub path: String,
    pub name: String,
    pub reason_code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpandedInputs {
    pub valid_paths: Vec<String>,
    pub rejected: Vec<RejectedInput>,
}

/// Returns true if the path has a supported image extension (case-insensitive).
#[must_use]
pub fn is_supported_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Validates and canonicalizes a single input image path received from the frontend.
pub fn validate_input_path(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Input path must not be empty".to_string());
    }

    let path = PathBuf::from(trimmed);
    let canonical = std::fs::canonicalize(&path)
        .map_err(|e| format!("Input file '{trimmed}' is not accessible: {e}"))?;

    if !canonical.is_file() {
        return Err(format!("Input path '{trimmed}' is not a regular file"));
    }

    if !is_supported_image_extension(&canonical) {
        return Err(format!(
            "Unsupported image format for '{}'. Supported: jpg, jpeg, png, webp, bmp, cr2, nef, arw, dng, heic, heif",
            canonical
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(trimmed)
        ));
    }

    Ok(canonical)
}

/// Expands files and top-level folder entries into deduplicated valid image paths and rejected items.
pub fn expand_input_paths(raw_paths: &[String]) -> ExpandedInputs {
    let mut seen = HashSet::new();
    let mut valid_paths = Vec::new();
    let mut rejected = Vec::new();

    for raw in raw_paths {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let path = PathBuf::from(trimmed);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(trimmed)
            .to_string();

        let Ok(canonical) = std::fs::canonicalize(&path) else {
            rejected.push(RejectedInput {
                path: trimmed.to_string(),
                name,
                reason_code: "inaccessible".to_string(),
                reason: "File or folder could not be accessed".to_string(),
            });
            continue;
        };

        if canonical.is_dir() {
            match std::fs::read_dir(&canonical) {
                Ok(entries) => {
                    let mut child_paths: Vec<PathBuf> =
                        entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                    child_paths.sort();

                    for child in child_paths {
                        let Ok(child_canon) = std::fs::canonicalize(&child) else {
                            continue;
                        };
                        if !child_canon.starts_with(&canonical) || !child_canon.is_file() {
                            continue;
                        }
                        if is_supported_image_extension(&child_canon) {
                            let s = child_canon.to_string_lossy().to_string();
                            if seen.insert(s.clone()) {
                                valid_paths.push(s);
                            }
                        }
                    }
                }
                Err(e) => {
                    rejected.push(RejectedInput {
                        path: canonical.to_string_lossy().to_string(),
                        name,
                        reason_code: "read_dir_failed".to_string(),
                        reason: format!("Failed to read folder: {e}"),
                    });
                }
            }
            continue;
        }

        if !canonical.is_file() {
            rejected.push(RejectedInput {
                path: canonical.to_string_lossy().to_string(),
                name,
                reason_code: "not_file_or_dir".to_string(),
                reason: "Item is not a regular file or folder".to_string(),
            });
            continue;
        }

        if !is_supported_image_extension(&canonical) {
            rejected.push(RejectedInput {
                path: canonical.to_string_lossy().to_string(),
                name,
                reason_code: "unsupported_format".to_string(),
                reason: "Unsupported image format (supported: JPG, JPEG, PNG, WebP, BMP, CR2, NEF, ARW, DNG, HEIC, HEIF)"
                    .to_string(),
            });
            continue;
        }

        let s = canonical.to_string_lossy().to_string();
        if seen.insert(s.clone()) {
            valid_paths.push(s);
        }
    }

    ExpandedInputs {
        valid_paths,
        rejected,
    }
}

/// Validates a batch output folder path, canonicalizing existing paths or verifying parent access.
pub fn validate_output_dir(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Output folder path must not be empty".to_string());
    }

    let path = PathBuf::from(trimmed);
    if path.exists() {
        let canonical = std::fs::canonicalize(&path)
            .map_err(|e| format!("Failed to access output folder '{trimmed}': {e}"))?;
        if !canonical.is_dir() {
            return Err(format!(
                "Output path '{trimmed}' exists and is not a directory"
            ));
        }
        return Ok(canonical);
    }

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| format!("Output folder '{trimmed}' has no valid parent directory"))?;

    if parent.exists() {
        let canonical_parent = std::fs::canonicalize(parent)
            .map_err(|e| format!("Output parent folder is not accessible: {e}"))?;
        let name = path
            .file_name()
            .ok_or_else(|| "Output folder name is missing".to_string())?;
        return Ok(canonical_parent.join(name));
    }

    Ok(path)
}

/// Validates a save destination PNG path and ensures it never overwrites any input file.
pub fn validate_save_destination(raw: &str, input_paths: &[PathBuf]) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Destination path must not be empty".to_string());
    }

    let mut dest = PathBuf::from(trimmed);
    if dest
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
        != Some("png")
    {
        dest.set_extension("png");
    }

    let normalized_dest = normalize_target_path(&dest)?;

    for input in input_paths {
        let canonical_input = std::fs::canonicalize(input).unwrap_or_else(|_| input.clone());
        if paths_equal(&normalized_dest, &canonical_input) {
            return Err("Refusing to overwrite an input image file".to_string());
        }
    }

    Ok(normalized_dest)
}

/// Computes deterministic PNG output paths for a batch of canonical input files.
/// Colliding stems (e.g. `photo.jpg` and `photo.png`) resolve to `<stem>_<ext>.png`.
pub fn resolve_batch_output_paths(
    inputs: &[PathBuf],
    out_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut stem_counts: HashMap<String, usize> = HashMap::new();
    for input in inputs {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_ascii_lowercase();
        *stem_counts.entry(stem).or_insert(0) += 1;
    }

    let mut outputs = Vec::with_capacity(inputs.len());
    for input in inputs {
        let stem = input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = input
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("img")
            .to_ascii_lowercase();

        let filename = if stem_counts
            .get(&stem.to_ascii_lowercase())
            .copied()
            .unwrap_or(0)
            > 1
        {
            format!("{stem}_{ext}.png")
        } else {
            format!("{stem}.png")
        };

        let candidate = out_dir.join(filename);
        let normalized = normalize_target_path(&candidate)?;
        if paths_equal(&normalized, input) {
            return Err(format!(
                "Refusing to write output '{}' on top of input file",
                normalized.display()
            ));
        }
        outputs.push(normalized);
    }

    Ok(outputs)
}

/// Validates an existing directory before opening it in the OS file manager.
pub fn validate_existing_directory(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Folder path must not be empty".to_string());
    }
    let canonical = std::fs::canonicalize(trimmed)
        .map_err(|e| format!("Folder '{trimmed}' does not exist or is inaccessible: {e}"))?;
    if !canonical.is_dir() {
        return Err(format!("Path '{trimmed}' is not a directory"));
    }
    Ok(canonical)
}

fn normalize_target_path(target: &Path) -> Result<PathBuf, String> {
    if target.exists() {
        return std::fs::canonicalize(target)
            .map_err(|e| format!("Failed to resolve target path '{}': {e}", target.display()));
    }

    let file_name = target
        .file_name()
        .ok_or_else(|| "Target path must include a file name".to_string())?;
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| "Target path has no parent directory".to_string())?;

    if parent.exists() {
        let canonical_parent = std::fs::canonicalize(parent)
            .map_err(|e| format!("Parent directory is inaccessible: {e}"))?;
        return Ok(canonical_parent.join(file_name));
    }

    Ok(target.to_path_buf())
}

fn paths_equal(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "rustmatte_app_val_{label}_{}_{}",
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).expect("create temp test dir");
        dir
    }

    #[test]
    fn rejects_empty_and_missing_paths() {
        assert!(validate_input_path("").is_err());
        assert!(validate_input_path("   ").is_err());
        assert!(validate_input_path("nonexistent_image_12345.png").is_err());
    }

    #[test]
    fn rejects_unsupported_extension_and_directories() {
        let dir = make_temp_dir("ext");
        let txt = dir.join("notes.txt");
        let jpg = dir.join("photo.JPG");
        let cr2 = dir.join("raw.CR2");
        let heic = dir.join("apple.heic");
        std::fs::write(&txt, b"hello").unwrap();
        std::fs::write(&jpg, b"fakejpg").unwrap();
        std::fs::write(&cr2, b"fakecr2").unwrap();
        std::fs::write(&heic, b"fakeheic").unwrap();

        assert!(validate_input_path(dir.to_str().unwrap()).is_err());
        assert!(validate_input_path(txt.to_str().unwrap()).is_err());
        assert!(validate_input_path(jpg.to_str().unwrap()).is_ok());
        assert!(validate_input_path(cr2.to_str().unwrap()).is_ok());
        assert!(validate_input_path(heic.to_str().unwrap()).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn expands_folders_and_collects_unsupported_items() {
        let dir = make_temp_dir("expand");
        let img1 = dir.join("one.jpg");
        let img2 = dir.join("two.PNG");
        let doc = dir.join("readme.md");
        std::fs::write(&img1, b"a").unwrap();
        std::fs::write(&img2, b"b").unwrap();
        std::fs::write(&doc, b"c").unwrap();

        let expanded = expand_input_paths(&[
            dir.to_string_lossy().to_string(),
            doc.to_string_lossy().to_string(),
        ]);
        assert_eq!(expanded.valid_paths.len(), 2);
        assert_eq!(expanded.rejected.len(), 1);
        assert_eq!(expanded.rejected[0].name, "readme.md");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_to_overwrite_input_file() {
        let dir = make_temp_dir("overwrite_guard");
        let input = dir.join("sample.png");
        std::fs::write(&input, b"png").unwrap();
        let canonical = std::fs::canonicalize(&input).unwrap();

        let err =
            validate_save_destination(input.to_str().unwrap(), std::slice::from_ref(&canonical));
        assert!(err.is_err());

        let safe = dir.join("sample_cutout.png");
        assert!(validate_save_destination(safe.to_str().unwrap(), &[canonical]).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolves_batch_stem_collisions_deterministically() {
        let dir = make_temp_dir("collisions");
        let a = dir.join("hero.jpg");
        let b = dir.join("hero.png");
        let c = dir.join("unique.webp");
        std::fs::write(&a, b"a").unwrap();
        std::fs::write(&b, b"b").unwrap();
        std::fs::write(&c, b"c").unwrap();

        let inputs = vec![
            std::fs::canonicalize(&a).unwrap(),
            std::fs::canonicalize(&b).unwrap(),
            std::fs::canonicalize(&c).unwrap(),
        ];
        let out_dir = dir.join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let outs = resolve_batch_output_paths(&inputs, &out_dir).unwrap();
        assert_eq!(outs[0].file_name().unwrap(), "hero_jpg.png");
        assert_eq!(outs[1].file_name().unwrap(), "hero_png.png");
        assert_eq!(outs[2].file_name().unwrap(), "unique.png");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn formats_windows_extended_paths_for_display_cross_platform() {
        assert_eq!(
            format_display_path(r"\\?\C:\Users\Admin\Downloads"),
            r"C:\Users\Admin\Downloads"
        );
        assert_eq!(
            format_display_path(r"\\?\d:\photos\cutout.png"),
            r"d:\photos\cutout.png"
        );
        assert_eq!(format_display_path(r"\\?\E:"), r"E:");
        assert_eq!(
            format_display_path(r"\\?\UNC\fileserver\share\cutouts"),
            r"\\fileserver\share\cutouts"
        );
        assert_eq!(format_display_path(r"\\?\unc\nas\photos"), r"\\nas\photos");
        assert_eq!(
            format_display_path(r"\\?\Volume{12345678-1234-1234-1234-1234567890ab}\dir"),
            r"\\?\Volume{12345678-1234-1234-1234-1234567890ab}\dir"
        );
        assert_eq!(
            format_display_path("/home/alice/pictures"),
            "/home/alice/pictures"
        );
    }
}
