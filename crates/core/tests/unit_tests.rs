//! Unit and integration tests for RustMatte Core.

use std::path::PathBuf;

use image::{DynamicImage, GenericImageView, GrayImage, Rgb, RgbImage, Rgba};
use rustmatte_core::decode::{decode_image, Warning, MAX_IMAGE_DIMENSION};
use rustmatte_core::error::CoreError;
use rustmatte_core::model::{
    spec_for_tier, verify_model_checksum, ModelTier, SPEC_FAST, SPEC_QUALITY,
};
use rustmatte_core::pipeline::{apply_mask, postprocess_mask, preprocess_tensor};
use rustmatte_core::{Config, RefineConfig, Remover};

#[test]
fn test_dimension_limits_constant() {
    assert_eq!(MAX_IMAGE_DIMENSION, 16384);
}

#[test]
fn test_normalization_math_known_pixels() {
    let spec = &SPEC_QUALITY;
    let mut rgb_img = RgbImage::new(1, 1);
    // Known pixel: R=128, G=64, B=32
    rgb_img.put_pixel(0, 0, Rgb([128, 64, 32]));
    let dyn_img = DynamicImage::ImageRgb8(rgb_img);

    let tensor = preprocess_tensor(&dyn_img, spec);

    // Verify shape is [1, 3, 512, 512]
    assert_eq!(tensor.shape(), &[1, 3, 512, 512]);

    // Expected normalized values
    let expected_r = (128.0_f32 / 255.0 - 0.485) / 0.229;
    let expected_g = (64.0_f32 / 255.0 - 0.456) / 0.224;
    let expected_b = (32.0_f32 / 255.0 - 0.406) / 0.225;

    let actual_r = tensor[[0, 0, 0, 0]];
    let actual_g = tensor[[0, 1, 0, 0]];
    let actual_b = tensor[[0, 2, 0, 0]];

    assert!(
        (actual_r - expected_r).abs() < 1e-4,
        "Red normalization mismatch: actual={actual_r}, expected={expected_r}"
    );
    assert!(
        (actual_g - expected_g).abs() < 1e-4,
        "Green normalization mismatch: actual={actual_g}, expected={expected_g}"
    );
    assert!(
        (actual_b - expected_b).abs() < 1e-4,
        "Blue normalization mismatch: actual={actual_b}, expected={expected_b}"
    );
}

#[test]
fn test_nchw_tensor_layout() {
    let spec = &SPEC_FAST;
    let mut rgb_img = RgbImage::new(4, 4);
    // Fill top-left pixel with Pure Red: [255, 0, 0]
    rgb_img.put_pixel(0, 0, Rgb([255, 0, 0]));
    let dyn_img = DynamicImage::ImageRgb8(rgb_img);

    let tensor = preprocess_tensor(&dyn_img, spec);

    // Check dimensions match NCHW [1, 3, 320, 320]
    assert_eq!(tensor.shape(), &[1, 3, 320, 320]);

    // Red channel at [0, 0, 0, 0] should be normalized (1.0 - 0.485)/0.229 > 0
    assert!(tensor[[0, 0, 0, 0]] > 2.0);
    // Green channel at [0, 1, 0, 0] should be negative (0.0 - 0.456)/0.224 < 0
    assert!(tensor[[0, 1, 0, 0]] < -1.0);
    // Blue channel at [0, 2, 0, 0] should be negative (0.0 - 0.406)/0.225 < 0
    assert!(tensor[[0, 2, 0, 0]] < -1.0);
}

#[test]
fn test_mask_resizing_and_postprocess() {
    let spec = &SPEC_FAST; // 320x320
    let count = (spec.width * spec.height) as usize;
    // Uniform mock raw output slice
    let raw_slice = vec![1.0_f32; count];

    let mask = postprocess_mask(&raw_slice, spec, 640, 480).expect("Postprocess should succeed");
    assert_eq!(mask.dimensions(), (640, 480));
}

#[test]
fn test_apply_mask_alpha_blending() {
    let mut rgb_img = RgbImage::new(2, 2);
    rgb_img.put_pixel(0, 0, Rgb([200, 100, 50]));
    rgb_img.put_pixel(1, 0, Rgb([10, 20, 30]));
    rgb_img.put_pixel(0, 1, Rgb([50, 150, 250]));
    rgb_img.put_pixel(1, 1, Rgb([255, 255, 255]));
    let dyn_img = DynamicImage::ImageRgb8(rgb_img);

    let mut mask = GrayImage::new(2, 2);
    mask.put_pixel(0, 0, image::Luma([0]));
    mask.put_pixel(1, 0, image::Luma([64]));
    mask.put_pixel(0, 1, image::Luma([128]));
    mask.put_pixel(1, 1, image::Luma([255]));

    let cutout = apply_mask(&dyn_img, &mask);
    assert_eq!(cutout.dimensions(), (2, 2));

    // Verify RGB is strictly preserved and Alpha is set exactly from mask
    assert_eq!(cutout.get_pixel(0, 0), &Rgba([200, 100, 50, 0]));
    assert_eq!(cutout.get_pixel(1, 0), &Rgba([10, 20, 30, 64]));
    assert_eq!(cutout.get_pixel(0, 1), &Rgba([50, 150, 250, 128]));
    assert_eq!(cutout.get_pixel(1, 1), &Rgba([255, 255, 255, 255]));
}

fn fixture_path(name: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let in_workspace = manifest_dir.join("../../tests/fixtures").join(name);
    if in_workspace.exists() {
        in_workspace
    } else {
        PathBuf::from("tests/fixtures").join(name)
    }
}

#[test]
fn test_decode_safety_zero_byte() {
    let res = decode_image(&fixture_path("corrupt_zerobyte.jpg"));
    match res {
        Err(CoreError::ZeroByteInput { path }) => {
            assert!(path.to_string_lossy().contains("corrupt_zerobyte.jpg"));
        }
        other => panic!("Expected ZeroByteInput error, got {other:?}"),
    }
}

#[test]
fn test_decode_truncated_jpeg_warning() {
    let path = fixture_path("corrupt_truncated.jpg");
    let res = decode_image(&path);
    // If the image crate is able to decode lines, it returns Ok with warning;
    // if the truncation is too severe to extract headers, it returns CoreError::ImageDecode.
    // Either way it must not panic and must handle cleanly.
    match res {
        Ok((_img, warnings)) => {
            assert!(
                warnings
                    .iter()
                    .any(|w| matches!(w, Warning::DamagedInput(_))),
                "Expected DamagedInput warning for truncated file"
            );
        }
        Err(CoreError::ImageDecode { .. }) => {
            // Clean typed error handled safely
        }
        other => panic!("Unexpected result for truncated image: {other:?}"),
    }
}

fn model_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let in_workspace = manifest_dir.join("../../models");
    if in_workspace.exists() {
        in_workspace
    } else {
        PathBuf::from("models")
    }
}

#[test]
fn test_model_checksum_verification() {
    let spec = spec_for_tier(ModelTier::Quality);
    let model_path = model_dir().join(spec.default_filename);

    if model_path.exists() {
        // Valid checksum test
        let res = verify_model_checksum(&model_path, spec.expected_sha256);
        assert!(
            res.is_ok(),
            "Valid model checksum verification failed: {res:?}"
        );

        // Invalid checksum test
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let res_bad = verify_model_checksum(&model_path, wrong_hash);
        match res_bad {
            Err(CoreError::ModelChecksumMismatch {
                expected, actual, ..
            }) => {
                assert_eq!(expected, wrong_hash);
                assert_eq!(actual, spec.expected_sha256);
            }
            other => panic!("Expected ModelChecksumMismatch, got {other:?}"),
        }
    }
}

// Model inference integration tests (ignored in CI because CI runners do not have model weights)
#[test]
#[ignore = "requires local model files not present in CI"]
fn test_remover_end_to_end_quality() {
    let spec = spec_for_tier(ModelTier::Quality);
    let path = model_dir().join(spec.default_filename);
    let config = Config {
        tier: ModelTier::Quality,
        model_path: Some(path),
        threads: 4,
        refine: RefineConfig::default(),
        execution_provider: rustmatte_core::ExecutionProvider::Cpu,
    };
    let mut remover = Remover::new(config).expect("Failed to initialize Remover for Quality tier");
    let input_path = fixture_path("baju1.jpg");
    let input_img = image::open(&input_path).expect("Failed to open baju1.jpg");
    let (expected_w, expected_h) = input_img.dimensions();

    let output = remover.process(&input_path).expect("Processing failed");

    assert_eq!(output.image.dimensions(), (expected_w, expected_h));
    assert_eq!(output.mask.dimensions(), (expected_w, expected_h));
}

#[test]
#[ignore = "requires local model files not present in CI"]
fn test_remover_end_to_end_fast() {
    let spec = spec_for_tier(ModelTier::Fast);
    let path = model_dir().join(spec.default_filename);
    let config = Config {
        tier: ModelTier::Fast,
        model_path: Some(path),
        threads: 4,
        refine: RefineConfig::default(),
        execution_provider: rustmatte_core::ExecutionProvider::Cpu,
    };
    let mut remover = Remover::new(config).expect("Failed to initialize Remover for Fast tier");
    let input_path = fixture_path("baju1.jpg");
    let input_img = image::open(&input_path).expect("Failed to open baju1.jpg");
    let (expected_w, expected_h) = input_img.dimensions();

    let output = remover.process(&input_path).expect("Processing failed");

    assert_eq!(output.image.dimensions(), (expected_w, expected_h));
    assert_eq!(output.mask.dimensions(), (expected_w, expected_h));
}

#[test]
fn test_execution_provider_parsing_and_display() {
    use rustmatte_core::{ActiveExecutionProvider, ExecutionProvider};
    use std::str::FromStr;

    assert_eq!(
        ExecutionProvider::from_str("cpu").unwrap(),
        ExecutionProvider::Cpu
    );
    assert_eq!(
        ExecutionProvider::from_str("directml").unwrap(),
        ExecutionProvider::DirectML
    );
    assert_eq!(
        ExecutionProvider::from_str("dml").unwrap(),
        ExecutionProvider::DirectML
    );
    assert_eq!(
        ExecutionProvider::from_str("gpu").unwrap(),
        ExecutionProvider::DirectML
    );
    assert_eq!(
        ExecutionProvider::from_str("coreml").unwrap(),
        ExecutionProvider::CoreML
    );
    assert_eq!(
        ExecutionProvider::from_str("ane").unwrap(),
        ExecutionProvider::CoreML
    );
    assert_eq!(
        ExecutionProvider::from_str("auto").unwrap(),
        ExecutionProvider::Auto
    );
    assert!(ExecutionProvider::from_str("cuda").is_err());

    assert_eq!(format!("{}", ExecutionProvider::Cpu), "cpu");
    assert_eq!(format!("{}", ExecutionProvider::DirectML), "directml");
    assert_eq!(format!("{}", ExecutionProvider::CoreML), "coreml");
    assert_eq!(format!("{}", ExecutionProvider::Auto), "auto");

    assert_eq!(format!("{}", ActiveExecutionProvider::Cpu), "CPU");
    assert_eq!(format!("{}", ActiveExecutionProvider::DirectML), "DirectML");
    assert_eq!(format!("{}", ActiveExecutionProvider::CoreML), "CoreML");
}

fn encode_jpeg_bytes(width: u32, height: u32) -> Vec<u8> {
    let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(width, height, |x, y| {
        image::Rgb([(x % 255) as u8, (y % 255) as u8, 128])
    }));
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Jpeg)
        .expect("Failed to encode synthetic JPEG");
    buf.into_inner()
}

fn build_synthetic_raw_tiff(
    little_endian: bool,
    orientation: u16,
    small_jpeg: &[u8],
    large_jpeg: &[u8],
) -> Vec<u8> {
    let mut out = Vec::new();
    let put_u16 = |buf: &mut Vec<u8>, v: u16| {
        if little_endian {
            buf.extend_from_slice(&v.to_le_bytes());
        } else {
            buf.extend_from_slice(&v.to_be_bytes());
        }
    };
    let put_u32 = |buf: &mut Vec<u8>, v: u32| {
        if little_endian {
            buf.extend_from_slice(&v.to_le_bytes());
        } else {
            buf.extend_from_slice(&v.to_be_bytes());
        }
    };
    let put_entry = |buf: &mut Vec<u8>, tag: u16, ftype: u16, count: u32, val_u32: u32| {
        put_u16(buf, tag);
        put_u16(buf, ftype);
        put_u32(buf, count);
        if ftype == 3 && count == 1 {
            put_u16(buf, val_u32 as u16);
            put_u16(buf, 0);
        } else {
            put_u32(buf, val_u32);
        }
    };

    // TIFF Header (8 bytes): IFD0 starts at offset 8
    if little_endian {
        out.extend_from_slice(b"II");
    } else {
        out.extend_from_slice(b"MM");
    }
    put_u16(&mut out, 42);
    put_u32(&mut out, 8);

    // IFD0: 4 entries (Orientation, JPEGInterchangeFormat, JPEGInterchangeFormatLength, SubIFDs)
    let ifd0_entries = 4u16;
    let subifd_offset = 8 + 2 + u32::from(ifd0_entries) * 12 + 4;
    let subifd_entries = 2u16;
    let small_offset = subifd_offset + 2 + u32::from(subifd_entries) * 12 + 4;
    let large_offset = small_offset + small_jpeg.len() as u32;

    put_u16(&mut out, ifd0_entries);
    put_entry(&mut out, 0x0112, 3, 1, u32::from(orientation));
    put_entry(&mut out, 0x014A, 4, 1, subifd_offset);
    put_entry(&mut out, 0x0201, 4, 1, small_offset);
    put_entry(&mut out, 0x0202, 4, 1, small_jpeg.len() as u32);
    put_u32(&mut out, 0); // next IFD = 0

    // SubIFD: 2 entries (StripOffsets, StripByteCounts) pointing to large JPEG
    assert_eq!(out.len() as u32, subifd_offset);
    put_u16(&mut out, subifd_entries);
    put_entry(&mut out, 0x0111, 4, 1, large_offset);
    put_entry(&mut out, 0x0117, 4, 1, large_jpeg.len() as u32);
    put_u32(&mut out, 0);

    assert_eq!(out.len() as u32, small_offset);
    out.extend_from_slice(small_jpeg);
    assert_eq!(out.len() as u32, large_offset);
    out.extend_from_slice(large_jpeg);
    out
}

#[test]
fn test_decode_raw_selects_largest_preview_and_applies_orientation() {
    let small = encode_jpeg_bytes(4, 2);
    let large = encode_jpeg_bytes(16, 8);
    assert!(large.len() > small.len());

    let raw_le = build_synthetic_raw_tiff(true, 6, &small, &large);
    let raw_be = build_synthetic_raw_tiff(false, 6, &small, &large);

    let temp_dir = std::env::temp_dir().join(format!("rustmatte_raw_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    for (name, bytes) in [
        ("sample.cr2", &raw_le),
        ("sample.NEF", &raw_be),
        ("sample.arw", &raw_le),
        ("sample.DNG", &raw_be),
    ] {
        let path = temp_dir.join(name);
        std::fs::write(&path, bytes).unwrap();

        let (img, warnings) = decode_image(&path).expect("RAW preview should decode");
        assert!(warnings.is_empty(), "Unexpected warnings for {name}");
        // Orientation 6 (Rotate90) swaps 16x8 into 8x16
        assert_eq!(img.width(), 8, "Expected rotated width 8 for {name}");
        assert_eq!(img.height(), 16, "Expected rotated height 16 for {name}");
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_decode_raw_truncated_preview_and_missing_preview_errors() {
    let small = encode_jpeg_bytes(4, 4);
    let mut truncated_large = encode_jpeg_bytes(16, 12);
    // Strip EOI marker (0xFF 0xD9)
    truncated_large.truncate(truncated_large.len() - 2);

    let temp_dir = std::env::temp_dir().join(format!("rustmatte_raw_err_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let trunc_path = temp_dir.join("truncated.CR2");
    let raw_trunc = build_synthetic_raw_tiff(true, 1, &small, &truncated_large);
    std::fs::write(&trunc_path, &raw_trunc).unwrap();
    match decode_image(&trunc_path) {
        Ok((_img, warnings)) => {
            assert!(warnings
                .iter()
                .any(|w| matches!(w, Warning::DamagedInput(_))));
        }
        Err(CoreError::ImageDecode { .. }) => {}
        other => panic!("Unexpected result for truncated RAW preview: {other:?}"),
    }

    // Malformed / short RAW file
    let bad_path = temp_dir.join("malformed.nef");
    std::fs::write(&bad_path, b"NOT_A_TIFF").unwrap();
    assert!(matches!(
        decode_image(&bad_path),
        Err(CoreError::ImageDecode { .. })
    ));

    // Valid TIFF header but no embedded JPEG preview
    let no_preview_path = temp_dir.join("empty_ifd.dng");
    let mut no_preview = Vec::new();
    no_preview.extend_from_slice(b"II\x2A\x00\x08\x00\x00\x00");
    no_preview.extend_from_slice(&0u16.to_le_bytes()); // 0 entries
    no_preview.extend_from_slice(&0u32.to_le_bytes()); // next IFD = 0
    std::fs::write(&no_preview_path, &no_preview).unwrap();
    assert!(matches!(
        decode_image(&no_preview_path),
        Err(CoreError::ImageDecode { .. })
    ));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_decode_heic_handles_zerobyte_and_malformed_safely() {
    let temp_dir = std::env::temp_dir().join(format!("rustmatte_heic_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let zero_path = temp_dir.join("empty.HEIC");
    std::fs::write(&zero_path, b"").unwrap();
    assert!(matches!(
        decode_image(&zero_path),
        Err(CoreError::ZeroByteInput { .. })
    ));

    let bad_heic = temp_dir.join("corrupt.heic");
    std::fs::write(&bad_heic, b"ftypheic_corrupt_payload").unwrap();
    assert!(matches!(
        decode_image(&bad_heic),
        Err(CoreError::ImageDecode { .. })
    ));

    let bad_heif = temp_dir.join("corrupt.HEIF");
    std::fs::write(&bad_heif, b"not_a_valid_heif_box").unwrap();
    assert!(matches!(
        decode_image(&bad_heif),
        Err(CoreError::ImageDecode { .. })
    ));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
