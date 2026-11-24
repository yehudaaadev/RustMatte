//! Image preprocessing, tensor normalization, mask postprocessing, and alpha blending.

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};
use ndarray::Array4;

use crate::error::CoreError;
use crate::model::{ModelKind, ModelSpec};

/// Preprocesses a dynamic image into a normalized NCHW floating-point tensor array.
///
/// Converts to RGB8, resizes to the model's exact target dimensions with triangle interpolation,
/// and applies per-channel mean and standard deviation normalization.
#[must_use]
pub fn preprocess_tensor(img: &DynamicImage, spec: &ModelSpec) -> Array4<f32> {
    let rgb = img.to_rgb8();
    let resized = image::imageops::resize(&rgb, spec.width, spec.height, FilterType::Triangle);

    let mut tensor = Array4::<f32>::zeros((1, 3, spec.height as usize, spec.width as usize));

    for y in 0..spec.height {
        for x in 0..spec.width {
            let px = resized.get_pixel(x, y);
            tensor[[0, 0, y as usize, x as usize]] =
                (px[0] as f32 / 255.0 - spec.mean[0]) / spec.std[0];
            tensor[[0, 1, y as usize, x as usize]] =
                (px[1] as f32 / 255.0 - spec.mean[1]) / spec.std[1];
            tensor[[0, 2, y as usize, x as usize]] =
                (px[2] as f32 / 255.0 - spec.mean[2]) / spec.std[2];
        }
    }

    tensor
}

/// Converts raw model output logits into an 8-bit probability mask resized to original dimensions.
pub fn postprocess_mask(
    raw_slice: &[f32],
    spec: &ModelSpec,
    orig_w: u32,
    orig_h: u32,
) -> Result<GrayImage, CoreError> {
    let expected_len = (spec.width * spec.height) as usize;
    if raw_slice.len() != expected_len {
        return Err(CoreError::TensorExtraction(format!(
            "Output slice length {} does not match expected size {} ({}x{})",
            raw_slice.len(),
            expected_len,
            spec.width,
            spec.height
        )));
    }

    let mut model_mask = Vec::with_capacity(expected_len);

    match spec.kind {
        ModelKind::BiRefNet => {
            // Raw logits requiring sigmoid activation: prob = 1 / (1 + exp(-logit))
            for &logit in raw_slice {
                let prob = 1.0 / (1.0 + (-logit).exp());
                let val = (prob * 255.0).clamp(0.0, 255.0) as u8;
                model_mask.push(val);
            }
        }
        ModelKind::U2Netp => {
            // Min-max normalization: (val - min) / (max - min)
            let mut min_val = f32::INFINITY;
            let mut max_val = f32::NEG_INFINITY;
            for &v in raw_slice {
                if v < min_val {
                    min_val = v;
                }
                if v > max_val {
                    max_val = v;
                }
            }
            let range = max_val - min_val;
            for &v in raw_slice {
                let norm = if range > 1e-8 {
                    (v - min_val) / range
                } else {
                    0.0
                };
                let val = (norm * 255.0).clamp(0.0, 255.0) as u8;
                model_mask.push(val);
            }
        }
    }

    let mask_img = GrayImage::from_raw(spec.width, spec.height, model_mask).ok_or_else(|| {
        CoreError::TensorExtraction("Failed to construct GrayImage from mask buffer".to_string())
    })?;

    // Basic Phase 1C postprocessing: resize mask back to original resolution with bilinear/triangle interpolation
    let resized_mask = image::imageops::resize(&mask_img, orig_w, orig_h, FilterType::Triangle);
    Ok(resized_mask)
}

/// Refines the raw segmentation mask against the original image.
///
/// Phase 1C: Basic mask resizing only (identity passthrough).
/// Phase 1D: Guided filter edge refinement, alpha feathering, and defringing will plug in here.
#[must_use]
pub fn refine_mask(mask: GrayImage, _orig: &DynamicImage) -> GrayImage {
    mask
}

/// Blends the original color channels with the alpha mask to produce a 32-bit RGBA image.
#[must_use]
pub fn apply_mask(orig: &DynamicImage, mask: &GrayImage) -> RgbaImage {
    let (w, h) = orig.dimensions();
    let rgb = orig.to_rgba8();
    let mut cutout = RgbaImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let mut px = *rgb.get_pixel(x, y);
            px[3] = mask.get_pixel(x, y)[0];
            cutout.put_pixel(x, y, px);
        }
    }

    cutout
}
