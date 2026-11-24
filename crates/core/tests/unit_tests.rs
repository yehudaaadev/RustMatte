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
