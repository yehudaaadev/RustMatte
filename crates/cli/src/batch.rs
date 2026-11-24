//! Batch image discovery, deterministic collision resolution, and job scheduling.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum recommended image count before emitting a batch volume warning.
pub const BATCH_WARNING_LIMIT: usize = 1000;

/// Supported image extensions for batch discovery (case-insensitive ASCII).
pub const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp"];

/// Discovered image job with input path, resolved output path, and relative display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchJob {
    /// Path to the source image file.
    pub input_path: PathBuf,
    /// Deterministic destination path for the output transparent PNG.
    pub output_path: PathBuf,
    /// Display name relative to the input root for progress logging.
    pub display_name: String,
}

/// Determines the default output folder next to the input directory.
#[must_use]
pub fn default_batch_output_dir(input_dir: &Path) -> PathBuf {
    if let Some(parent) = input_dir.parent() {
        if !parent.as_os_str().is_empty() {
            return parent.join("rustmatte-output");
        }
    }
    PathBuf::from("rustmatte-output")
}

/// Checks whether a file path has a supported image extension.
#[must_use]
pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            SUPPORTED_EXTENSIONS.iter().any(|&sup| sup == lower)
        })
        .unwrap_or(false)
}

/// Discovers supported image files in `input_dir` and computes deterministic output paths.
pub fn discover_batch_jobs(
    input_dir: &Path,
    output_dir: &Path,
    recursive: bool,
) -> Result<(Vec<BatchJob>, Vec<String>), String> {
    let mut warnings = Vec::new();
    let mut raw_files = Vec::new();

    let canonical_input = input_dir.canonicalize().map_err(|e| {
        format!(
            "Failed to resolve input directory '{}': {e}",
            input_dir.display()
        )
    })?;

    collect_files_recursive(
        input_dir,
        &canonical_input,
        input_dir,
        recursive,
        &mut raw_files,
        &mut warnings,
    )?;

    if raw_files.len() > BATCH_WARNING_LIMIT {
        warnings.push(format!(
            "Found {} images (exceeds recommended batch limit of {}). Processing will continue.",
            raw_files.len(),
            BATCH_WARNING_LIMIT
        ));
    }

    raw_files.sort_by(|a, b| a.0.cmp(&b.0));

    let jobs = resolve_output_paths(&raw_files, output_dir);
    Ok((jobs, warnings))
}

fn collect_files_recursive(
    current_dir: &Path,
    canonical_input: &Path,
    base_input: &Path,
    recursive: bool,
    collected: &mut Vec<(PathBuf, PathBuf)>,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let entries = fs::read_dir(current_dir)
        .map_err(|e| format!("Failed to read directory '{}': {e}", current_dir.display()))?;

    let mut subdirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Directory read error: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to read file type: {e}"))?;

        if file_type.is_symlink() {
            match path.canonicalize() {
                Ok(target) => {
                    if !target.starts_with(canonical_input) {
                        warnings.push(format!(
                            "Skipping symlink '{}' pointing outside input directory",
                            path.display()
                        ));
                        continue;
                    }
                    if target.is_dir() && recursive {
                        subdirs.push(path);
                    } else if target.is_file() && is_supported_image(&path) {
                        if let Ok(rel) = path.strip_prefix(base_input) {
                            collected.push((rel.to_path_buf(), path));
                        }
                    }
                }
                Err(e) => {
                    warnings.push(format!(
                        "Skipping unresolvable symlink '{}': {e}",
                        path.display()
                    ));
                }
            }
        } else if file_type.is_dir() {
            if recursive {
                subdirs.push(path);
            }
        } else if file_type.is_file() && is_supported_image(&path) {
            if let Ok(rel) = path.strip_prefix(base_input) {
                collected.push((rel.to_path_buf(), path));
            }
        }
    }

    if recursive {
        for subdir in subdirs {
            collect_files_recursive(
                &subdir,
                canonical_input,
                base_input,
                recursive,
                collected,
                warnings,
            )?;
        }
    }

    Ok(())
}

fn resolve_output_paths(raw_files: &[(PathBuf, PathBuf)], output_dir: &Path) -> Vec<BatchJob> {
    // Map of (relative_parent, stem) -> count of files sharing the same stem
    let mut stem_counts: HashMap<(PathBuf, String), usize> = HashMap::new();

    for (rel_path, _) in raw_files {
        let rel_parent = rel_path.parent().unwrap_or(Path::new("")).to_path_buf();
        let stem = rel_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string();
        *stem_counts.entry((rel_parent, stem)).or_insert(0) += 1;
    }

    let mut jobs = Vec::with_capacity(raw_files.len());

    for (rel_path, full_path) in raw_files {
        let rel_parent = rel_path.parent().unwrap_or(Path::new("")).to_path_buf();
        let stem = rel_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let ext = rel_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let count = stem_counts
            .get(&(rel_parent.clone(), stem.to_string()))
            .copied()
            .unwrap_or(1);

        let final_filename = if count > 1 && !ext.is_empty() {
            format!("{stem}_{ext}.png")
        } else {
            format!("{stem}.png")
        };

        let output_path = output_dir.join(&rel_parent).join(final_filename);
        let display_name = rel_path.to_string_lossy().replace('\\', "/");

        jobs.push(BatchJob {
            input_path: full_path.clone(),
            output_path,
            display_name,
        });
    }

    jobs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image(Path::new("photo.jpg")));
        assert!(is_supported_image(Path::new("photo.JPEG")));
        assert!(is_supported_image(Path::new("photo.png")));
        assert!(is_supported_image(Path::new("photo.webp")));
        assert!(is_supported_image(Path::new("photo.bmp")));
        assert!(!is_supported_image(Path::new("photo.gif")));
        assert!(!is_supported_image(Path::new("photo.txt")));
        assert!(!is_supported_image(Path::new("photo")));
    }

    #[test]
    fn test_collision_resolution() {
        let raw = vec![
            (PathBuf::from("photo.jpg"), PathBuf::from("/in/photo.jpg")),
            (PathBuf::from("photo.png"), PathBuf::from("/in/photo.png")),
            (PathBuf::from("solo.webp"), PathBuf::from("/in/solo.webp")),
        ];
        let out_dir = Path::new("/out");
        let jobs = resolve_output_paths(&raw, out_dir);

        assert_eq!(jobs.len(), 3);
        assert_eq!(jobs[0].output_path, PathBuf::from("/out/photo_jpg.png"));
        assert_eq!(jobs[1].output_path, PathBuf::from("/out/photo_png.png"));
        assert_eq!(jobs[2].output_path, PathBuf::from("/out/solo.png"));
    }

    #[test]
    fn test_default_batch_output_dir() {
        assert_eq!(
            default_batch_output_dir(Path::new("photos")),
            PathBuf::from("rustmatte-output")
        );
        assert_eq!(
            default_batch_output_dir(Path::new("path/to/photos")),
            PathBuf::from("path/to/rustmatte-output")
        );
    }

    #[test]
    fn test_discover_batch_jobs_traversal_and_filtering() {
        let temp_root =
            std::env::temp_dir().join(format!("rustmatte_batch_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_root);
        fs::create_dir_all(temp_root.join("sub")).unwrap();

        fs::write(temp_root.join("image1.jpg"), b"fake_jpg").unwrap();
        fs::write(temp_root.join("image2.PNG"), b"fake_png").unwrap();
        fs::write(temp_root.join("document.txt"), b"fake_txt").unwrap();
        fs::write(temp_root.join("sub").join("nested.webp"), b"fake_webp").unwrap();
        fs::write(temp_root.join("sub").join("data.csv"), b"fake_csv").unwrap();

        let out_dir = temp_root.join("output");

        // Non-recursive: discovers top-level images only
        let (jobs_flat, warnings_flat) = discover_batch_jobs(&temp_root, &out_dir, false).unwrap();
        assert!(warnings_flat.is_empty());
        assert_eq!(jobs_flat.len(), 2);
        assert_eq!(jobs_flat[0].output_path, out_dir.join("image1.png"));
        assert_eq!(jobs_flat[1].output_path, out_dir.join("image2.png"));

        // Recursive: discovers nested images with relative hierarchy
        let (jobs_rec, warnings_rec) = discover_batch_jobs(&temp_root, &out_dir, true).unwrap();
        assert!(warnings_rec.is_empty());
        assert_eq!(jobs_rec.len(), 3);
        assert_eq!(jobs_rec[0].output_path, out_dir.join("image1.png"));
        assert_eq!(jobs_rec[1].output_path, out_dir.join("image2.png"));
        assert_eq!(
            jobs_rec[2].output_path,
            out_dir.join("sub").join("nested.png")
        );

        let _ = fs::remove_dir_all(&temp_root);
    }
}
