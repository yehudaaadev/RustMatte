//! Safe image decoding, EXIF orientation handling, dimension guards, and damage warnings.

use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use image::metadata::Orientation;
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader, Limits};

use crate::error::CoreError;
use crate::raw;

/// Canonical list of supported input image extensions (lowercase ASCII).
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "bmp", "cr2", "nef", "arw", "dng", "heic", "heif",
];

/// Maximum allowable image dimension (width or height) to prevent decompression bombs.
pub const MAX_IMAGE_DIMENSION: u32 = 16384;

/// Maximum allocation limit for image decoding (512 MB).
pub const MAX_ALLOCATION_BYTES: u64 = 512 * 1024 * 1024;

/// Non-fatal warnings produced during processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// The input file appears damaged, truncated, or incomplete.
    DamagedInput(String),
}

/// Decodes an image with strict safety limits, EXIF orientation correction, and truncation checks.
pub fn decode_image(path: &Path) -> Result<(DynamicImage, Vec<Warning>), CoreError> {
    let mut warnings = Vec::new();

    // 1. Verify file presence and zero-byte check
    let metadata = std::fs::metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::ImageDecode {
                path: path.to_path_buf(),
                reason: "File not found".to_string(),
            }
        } else {
            CoreError::Io(e)
        }
    })?;

    let file_len = metadata.len();
    if file_len == 0 {
        return Err(CoreError::ZeroByteInput {
            path: path.to_path_buf(),
        });
    }

    // 2. Dispatch specialized formats (Camera RAW embedded preview, HEIC/HEIF)
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    let mut file = File::open(path)?;

    if matches!(ext.as_str(), "cr2" | "nef" | "arw" | "dng") {
        return decode_raw_preview(path, &mut file, file_len);
    }

    if matches!(ext.as_str(), "heic" | "heif") {
        return decode_heic(path, &mut file, file_len);
    }

    // 3. Inspect raw file trailer for truncation detection
    let mut header_buf = [0u8; 16];
    let bytes_read = file.read(&mut header_buf)?;

    if bytes_read >= 2 && header_buf[0] == 0xFF && header_buf[1] == 0xD8 {
        // JPEG format: check for 0xFF 0xD9 End-of-Image (EOI) marker at the end
        if file_len >= 2 {
            use std::io::Seek;
            use std::io::SeekFrom;
            let mut end_buf = [0u8; 2];
            if file.seek(SeekFrom::End(-2)).is_ok()
                && file.read_exact(&mut end_buf).is_ok()
                && end_buf != [0xFF, 0xD9]
            {
                warnings.push(Warning::DamagedInput(
                    "File is damaged or incomplete: missing JPEG end-of-image marker".to_string(),
                ));
            }
        }
    } else if bytes_read >= 8
        && header_buf.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    {
        // PNG format: check for standard IEND chunk trailer
        if file_len >= 12 {
            use std::io::Seek;
            use std::io::SeekFrom;
            let mut end_buf = [0u8; 8];
            if file.seek(SeekFrom::End(-8)).is_ok()
                && file.read_exact(&mut end_buf).is_ok()
                && end_buf[0..4] != [0x49, 0x45, 0x4E, 0x44]
            {
                warnings.push(Warning::DamagedInput(
                    "File is damaged or incomplete: missing PNG IEND marker".to_string(),
                ));
            }
        }
    }

    // 3. Configure safe decoding limits against decompression bombs
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOCATION_BYTES);

    let mut reader = ImageReader::open(path).map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("Failed to open image reader: {e}"),
    })?;
    reader.limits(limits);

    let mut decoder = reader.into_decoder().map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("Failed to initialize image decoder: {e}"),
    })?;

    // Check dimensions before full allocation
    let (w, h) = decoder.dimensions();
    if w > MAX_IMAGE_DIMENSION || h > MAX_IMAGE_DIMENSION {
        return Err(CoreError::DimensionLimitExceeded {
            width: w,
            height: h,
            max: MAX_IMAGE_DIMENSION,
        });
    }

    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("Decoding failed: {e}"),
    })?;

    // Apply EXIF orientation
    img.apply_orientation(orientation);

    let (final_w, final_h) = img.dimensions();
    if final_w > MAX_IMAGE_DIMENSION || final_h > MAX_IMAGE_DIMENSION {
        return Err(CoreError::DimensionLimitExceeded {
            width: final_w,
            height: final_h,
            max: MAX_IMAGE_DIMENSION,
        });
    }

    Ok((img, warnings))
}

fn default_decode_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOCATION_BYTES);
    limits
}

fn decode_raw_preview(
    path: &Path,
    file: &mut File,
    file_len: u64,
) -> Result<(DynamicImage, Vec<Warning>), CoreError> {
    let mut warnings = Vec::new();
    let preview = raw::extract_raw_jpeg_preview(path, file, file_len)?;

    let len = preview.jpeg_bytes.len();
    if len >= 2 && preview.jpeg_bytes[len - 2..len] != [0xFF, 0xD9] {
        warnings.push(Warning::DamagedInput(
            "File is damaged or incomplete: missing JPEG end-of-image marker".to_string(),
        ));
    }

    let mut reader = ImageReader::with_format(Cursor::new(&preview.jpeg_bytes), ImageFormat::Jpeg);
    reader.limits(default_decode_limits());

    let mut decoder = reader.into_decoder().map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("Failed to initialize RAW JPEG preview decoder: {e}"),
    })?;

    let (w, h) = decoder.dimensions();
    if w > MAX_IMAGE_DIMENSION || h > MAX_IMAGE_DIMENSION {
        return Err(CoreError::DimensionLimitExceeded {
            width: w,
            height: h,
            max: MAX_IMAGE_DIMENSION,
        });
    }

    let jpeg_orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let orientation = if jpeg_orientation != Orientation::NoTransforms {
        jpeg_orientation
    } else {
        preview.orientation
    };

    let mut img = DynamicImage::from_decoder(decoder).map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("RAW preview decoding failed: {e}"),
    })?;

    img.apply_orientation(orientation);

    let (final_w, final_h) = img.dimensions();
    if final_w > MAX_IMAGE_DIMENSION || final_h > MAX_IMAGE_DIMENSION {
        return Err(CoreError::DimensionLimitExceeded {
            width: final_w,
            height: final_h,
            max: MAX_IMAGE_DIMENSION,
        });
    }

    Ok((img, warnings))
}

fn decode_heic(
    path: &Path,
    file: &mut File,
    file_len: u64,
) -> Result<(DynamicImage, Vec<Warning>), CoreError> {
    if file_len > MAX_ALLOCATION_BYTES {
        return Err(CoreError::ImageDecode {
            path: path.to_path_buf(),
            reason: format!(
                "HEIC/HEIF file size ({file_len} bytes) exceeds maximum limit ({MAX_ALLOCATION_BYTES} bytes)"
            ),
        });
    }

    let capacity = usize::try_from(file_len).map_err(|_| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: "HEIC/HEIF file size exceeds addressable memory".to_string(),
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    file.read_to_end(&mut bytes)?;

    let info = heic_rs::probe(&bytes).map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("Failed to probe HEIC/HEIF metadata: {e}"),
    })?;

    if info.width > MAX_IMAGE_DIMENSION || info.height > MAX_IMAGE_DIMENSION {
        return Err(CoreError::DimensionLimitExceeded {
            width: info.width,
            height: info.height,
            max: MAX_IMAGE_DIMENSION,
        });
    }

    let layout = if info.has_alpha {
        heic_rs::PixelLayout::Rgba8
    } else {
        heic_rs::PixelLayout::Rgb8
    };

    let options = heic_rs::DecodeOptions::default()
        .with_layout(layout)
        .with_max_pixels(Some(MAX_ALLOCATION_BYTES / 4))
        .with_transforms(true);

    let decoded = heic_rs::decode(&bytes, &options).map_err(|e| CoreError::ImageDecode {
        path: path.to_path_buf(),
        reason: format!("HEIC/HEIF decoding failed: {e}"),
    })?;

    if decoded.width > MAX_IMAGE_DIMENSION || decoded.height > MAX_IMAGE_DIMENSION {
        return Err(CoreError::DimensionLimitExceeded {
            width: decoded.width,
            height: decoded.height,
            max: MAX_IMAGE_DIMENSION,
        });
    }

    let img = match decoded.layout {
        heic_rs::PixelLayout::Rgba8 => {
            image::RgbaImage::from_raw(decoded.width, decoded.height, decoded.data)
                .map(DynamicImage::ImageRgba8)
                .ok_or_else(|| CoreError::ImageDecode {
                    path: path.to_path_buf(),
                    reason: "HEIC/HEIF RGBA8 pixel buffer size mismatch".to_string(),
                })?
        }
        heic_rs::PixelLayout::Rgb8 => {
            image::RgbImage::from_raw(decoded.width, decoded.height, decoded.data)
                .map(DynamicImage::ImageRgb8)
                .ok_or_else(|| CoreError::ImageDecode {
                    path: path.to_path_buf(),
                    reason: "HEIC/HEIF RGB8 pixel buffer size mismatch".to_string(),
                })?
        }
        other => {
            return Err(CoreError::ImageDecode {
                path: path.to_path_buf(),
                reason: format!("Unexpected HEIC/HEIF pixel layout: {other:?}"),
            });
        }
    };

    Ok((img, Vec::new()))
}
