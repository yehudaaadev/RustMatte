//! Safe image decoding, EXIF orientation handling, dimension guards, and damage warnings.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use image::metadata::Orientation;
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageReader, Limits};

use crate::error::CoreError;

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

    // 2. Inspect raw file trailer for truncation detection
    let mut file = File::open(path)?;
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
