//! Image preprocessing, tensor normalization, mask postprocessing, and alpha blending.

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};
use ndarray::Array4;

use crate::error::CoreError;
use crate::model::{ModelKind, ModelSpec};
use crate::refine::{apply_mask_with_refinement, refine_mask_guided, RefineConfig};

/// Downscales an image early if its longest dimension exceeds `max_side`, preserving aspect ratio.
///
/// If `max_side` is `None` or 0, or if both dimensions are already <= `max_side`, the image is returned unchanged.
#[must_use]
pub fn downscale_if_needed(img: DynamicImage, max_side: Option<u32>) -> DynamicImage {
    if let Some(max) = max_side {
        if max > 0 {
            let (w, h) = img.dimensions();
            let longest = w.max(h);
            if longest > max {
                let scale = max as f64 / longest as f64;
                let new_w = ((w as f64 * scale).round() as u32).max(1);
                let new_h = ((h as f64 * scale).round() as u32).max(1);
                return img.resize_exact(new_w, new_h, FilterType::Triangle);
            }
        }
    }
    img
}

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
        ModelKind::U2Netp | ModelKind::Ben2 => {
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

/// Refines the raw segmentation mask against the original image using Guided Filter and transition feathering.
#[must_use]
pub fn refine_mask(mask: GrayImage, orig: &DynamicImage, config: &RefineConfig) -> GrayImage {
    refine_mask_guided(mask, orig, config)
}

/// Blends the original color channels with the alpha mask, applying optional color defringing.
#[must_use]
pub fn apply_mask_with_options(orig: &DynamicImage, mask: &GrayImage, defringe: bool) -> RgbaImage {
    apply_mask_with_refinement(orig, mask, defringe)
}

/// Blends the original color channels with the alpha mask to produce a 32-bit RGBA image.
#[must_use]
pub fn apply_mask(orig: &DynamicImage, mask: &GrayImage) -> RgbaImage {
    apply_mask_with_options(orig, mask, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_downscale_if_needed() {
        let img =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(2000, 1000, Rgba([255, 0, 0, 255])));

        // None keeps original
        let res = downscale_if_needed(img.clone(), None);
        assert_eq!(res.dimensions(), (2000, 1000));

        // Larger max_side keeps original
        let res = downscale_if_needed(img.clone(), Some(3000));
        assert_eq!(res.dimensions(), (2000, 1000));

        // Downscale landscape
        let res = downscale_if_needed(img.clone(), Some(1000));
        assert_eq!(res.dimensions(), (1000, 500));

        // Downscale portrait
        let portrait =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(800, 1600, Rgba([0, 255, 0, 255])));
        let res = downscale_if_needed(portrait, Some(800));
        assert_eq!(res.dimensions(), (400, 800));
    }
}
