//! Advanced edge refinement: Fast Guided Filter, transition zone feathering, and color defringing.

use image::{DynamicImage, GenericImageView, GrayImage, Rgba, RgbaImage};

/// Edge refinement and defringing configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefineConfig {
    /// Filter window radius for the guided filter (default: 4).
    pub radius: usize,
    /// Regularization parameter preventing division by zero and controlling edge sensitivity (default: 1e-4).
    pub eps: f32,
    /// Whether to apply guided filter edge alignment (default: true).
    pub enable_guided_filter: bool,
    /// Whether to apply color defringing on semi-transparent edge pixels (default: true).
    pub enable_defringing: bool,
}

impl Default for RefineConfig {
    fn default() -> Self {
        Self {
            radius: 4,
            eps: 1e-4,
            enable_guided_filter: true,
            enable_defringing: true,
        }
    }
}

/// Applies a 1D horizontal box filter using a sliding window accumulator with border clamping.
pub fn box_filter_horizontal(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
) {
    let r = radius as isize;
    for y in 0..height {
        let row_offset = y * width;
        let mut sum = 0.0f32;

        for dx in -r..=r {
            let clamped_x = dx.clamp(0, width as isize - 1) as usize;
            sum += src[row_offset + clamped_x];
        }
        dst[row_offset] = sum;

        for x in 1..width {
            let left_x = (x as isize - r - 1).clamp(0, width as isize - 1) as usize;
            let right_x = (x as isize + r).clamp(0, width as isize - 1) as usize;
            sum += src[row_offset + right_x] - src[row_offset + left_x];
            dst[row_offset + x] = sum;
        }
    }
}

/// Applies a 1D vertical box filter using a sliding window accumulator with border clamping.
pub fn box_filter_vertical(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
) {
    let r = radius as isize;
    for x in 0..width {
        let mut sum = 0.0f32;

        for dy in -r..=r {
            let clamped_y = dy.clamp(0, height as isize - 1) as usize;
            sum += src[clamped_y * width + x];
        }
        dst[x] = sum;

        for y in 1..height {
            let top_y = (y as isize - r - 1).clamp(0, height as isize - 1) as usize;
            let bottom_y = (y as isize + r).clamp(0, height as isize - 1) as usize;
            sum += src[bottom_y * width + x] - src[top_y * width + x];
            dst[y * width + x] = sum;
        }
    }
}

/// Applies a 2D separable box filter: horizontal pass followed by vertical pass.
pub fn box_filter_2d(
    src: &[f32],
    dst: &mut [f32],
    scratch: &mut [f32],
    width: usize,
    height: usize,
    radius: usize,
) {
    box_filter_horizontal(src, scratch, width, height, radius);
    box_filter_vertical(scratch, dst, width, height, radius);
}

/// Applies Guided Image Filtering to refine the mask edges against the original image guidance.
///
/// Implements $O(N)$ guided filter using luminance as guidance $I$ and raw mask as filtering input $p$.
#[must_use]
pub fn guided_filter(
    guidance_gray: &[f32],
    input_mask: &[f32],
    width: usize,
    height: usize,
    radius: usize,
    eps: f32,
) -> Vec<f32> {
    let total_pixels = width * height;
    let n_inv = 1.0f32 / ((2 * radius + 1) * (2 * radius + 1)) as f32;

    let mut scratch = vec![0.0f32; total_pixels];
    let mut temp = vec![0.0f32; total_pixels];

    // 1. mean_I
    let mut mean_i = vec![0.0f32; total_pixels];
    box_filter_2d(
        guidance_gray,
        &mut mean_i,
        &mut scratch,
        width,
        height,
        radius,
    );
    for v in &mut mean_i {
        *v *= n_inv;
    }

    // 2. mean_p
    let mut mean_p = vec![0.0f32; total_pixels];
    box_filter_2d(input_mask, &mut mean_p, &mut scratch, width, height, radius);
    for v in &mut mean_p {
        *v *= n_inv;
    }

    // 3. mean_Ip
    for i in 0..total_pixels {
        temp[i] = guidance_gray[i] * input_mask[i];
    }
    let mut mean_ip = vec![0.0f32; total_pixels];
    box_filter_2d(&temp, &mut mean_ip, &mut scratch, width, height, radius);
    for v in &mut mean_ip {
        *v *= n_inv;
    }

    // 4. mean_II
    for i in 0..total_pixels {
        temp[i] = guidance_gray[i] * guidance_gray[i];
    }
    let mut mean_ii = vec![0.0f32; total_pixels];
    box_filter_2d(&temp, &mut mean_ii, &mut scratch, width, height, radius);
    for v in &mut mean_ii {
        *v *= n_inv;
    }

    // 5. a = cov_Ip / (var_I + eps), b = mean_p - a * mean_I
    let mut a = vec![0.0f32; total_pixels];
    let mut b = vec![0.0f32; total_pixels];

    for i in 0..total_pixels {
        let cov_ip = mean_ip[i] - mean_i[i] * mean_p[i];
        let var_i = mean_ii[i] - mean_i[i] * mean_i[i];
        let a_val = cov_ip / (var_i.max(0.0) + eps);
        let b_val = mean_p[i] - a_val * mean_i[i];
        a[i] = a_val;
        b[i] = b_val;
    }

    // 6. mean_a and mean_b
    let mut mean_a = vec![0.0f32; total_pixels];
    box_filter_2d(&a, &mut mean_a, &mut scratch, width, height, radius);
    for v in &mut mean_a {
        *v *= n_inv;
    }

    let mut mean_b = vec![0.0f32; total_pixels];
    box_filter_2d(&b, &mut mean_b, &mut scratch, width, height, radius);
    for v in &mut mean_b {
        *v *= n_inv;
    }

    // 7. q = mean_a * I + mean_b
    let mut q = vec![0.0f32; total_pixels];
    for i in 0..total_pixels {
        q[i] = mean_a[i] * guidance_gray[i] + mean_b[i];
    }

    q
}

/// Refines a mask using Guided Filter with selective transition zone feathering.
///
/// Solid foreground ($\ge 0.95$) and definite background ($\le 0.05$) remain sharp, while
/// boundary pixels receive edge-preserving filter alignment.
#[must_use]
pub fn refine_mask_guided(
    raw_mask: GrayImage,
    orig: &DynamicImage,
    config: &RefineConfig,
) -> GrayImage {
    if !config.enable_guided_filter {
        return raw_mask;
    }

    let (width, height) = orig.dimensions();
    let (mw, mh) = raw_mask.dimensions();

    if width != mw || height != mh || width == 0 || height == 0 {
        return raw_mask;
    }

    let total = (width * height) as usize;
    let rgb = orig.to_rgb8();

    // Guidance luminance channel [0.0, 1.0]
    let mut guidance = Vec::with_capacity(total);
    for pixel in rgb.pixels() {
        let lum =
            (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) / 255.0;
        guidance.push(lum);
    }

    // Normalized raw mask [0.0, 1.0]
    let mut p = Vec::with_capacity(total);
    for pixel in raw_mask.pixels() {
        p.push(pixel[0] as f32 / 255.0);
    }

    let q = guided_filter(
        &guidance,
        &p,
        width as usize,
        height as usize,
        config.radius,
        config.eps,
    );

    let mut refined = GrayImage::new(width, height);

    for (i, pixel) in refined.pixels_mut().enumerate() {
        let p_val = p[i];
        let q_val = q[i].clamp(0.0, 1.0);

        // Selective feathering in transition zone (0.05..0.95)
        let final_alpha = if p_val <= 0.05 {
            0.0f32
        } else if p_val >= 0.95 {
            1.0f32
        } else {
            // Smoothly blend between raw mask and guided filter output
            q_val
        };

        pixel[0] = (final_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    refined
}

/// Blends the original color channels with the alpha mask, applying optional defringing
/// to decontaminate background color bleed on semi-transparent edge pixels.
#[must_use]
pub fn apply_mask_with_refinement(
    orig: &DynamicImage,
    mask: &GrayImage,
    enable_defringing: bool,
) -> RgbaImage {
    let (w, h) = orig.dimensions();
    let rgb = orig.to_rgba8();
    let mut cutout = RgbaImage::new(w, h);

    if !enable_defringing {
        for y in 0..h {
            for x in 0..w {
                let mut px = *rgb.get_pixel(x, y);
                px[3] = mask.get_pixel(x, y)[0];
                cutout.put_pixel(x, y, px);
            }
        }
        return cutout;
    }

    let r_search: isize = 2; // 5x5 neighborhood search for solid foreground

    for y in 0..h {
        for x in 0..w {
            let alpha = mask.get_pixel(x, y)[0];

            if alpha == 0 {
                cutout.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }

            let mut px = *rgb.get_pixel(x, y);
            px[3] = alpha;

            // Semi-transparent boundary pixel: search for solid foreground neighbor (alpha >= 235)
            if alpha < 235 {
                let mut sum_r = 0.0f32;
                let mut sum_g = 0.0f32;
                let mut sum_b = 0.0f32;
                let mut fg_count = 0.0f32;

                for dy in -r_search..=r_search {
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= h as isize {
                        continue;
                    }

                    for dx in -r_search..=r_search {
                        let nx = x as isize + dx;
                        if nx < 0 || nx >= w as isize {
                            continue;
                        }

                        let n_alpha = mask.get_pixel(nx as u32, ny as u32)[0];
                        if n_alpha >= 235 {
                            let n_px = rgb.get_pixel(nx as u32, ny as u32);
                            sum_r += n_px[0] as f32;
                            sum_g += n_px[1] as f32;
                            sum_b += n_px[2] as f32;
                            fg_count += 1.0;
                        }
                    }
                }

                if fg_count > 0.0 {
                    let avg_r = sum_r / fg_count;
                    let avg_g = sum_g / fg_count;
                    let avg_b = sum_b / fg_count;

                    // Blend proportion based on alpha transparency
                    let blend_fg = 1.0 - (alpha as f32 / 255.0);
                    px[0] = ((px[0] as f32) * (1.0 - blend_fg) + avg_r * blend_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    px[1] = ((px[1] as f32) * (1.0 - blend_fg) + avg_g * blend_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    px[2] = ((px[2] as f32) * (1.0 - blend_fg) + avg_b * blend_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }

            cutout.put_pixel(x, y, px);
        }
    }

    cutout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_filter_1d_flat() {
        let src = vec![1.0f32; 10];
        let mut dst = vec![0.0f32; 10];
        box_filter_horizontal(&src, &mut dst, 10, 1, 2);
        for v in dst {
            // radius 2 -> window of 5 elements
            assert!((v - 5.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_box_filter_2d_flat() {
        let w = 8;
        let h = 8;
        let src = vec![2.0f32; w * h];
        let mut dst = vec![0.0f32; w * h];
        let mut scratch = vec![0.0f32; w * h];
        let radius = 1;
        box_filter_2d(&src, &mut dst, &mut scratch, w, h, radius);

        let n = (2 * radius + 1) * (2 * radius + 1); // 9
        for v in dst {
            assert!((v - 2.0 * n as f32).abs() < 1e-4);
        }
    }

    #[test]
    fn test_guided_filter_edge_preservation() {
        let w = 16;
        let h = 16;
        let mut guidance = vec![0.0f32; w * h];
        let mut mask = vec![0.0f32; w * h];

        // Sharp vertical edge at x = 8
        for y in 0..h {
            for x in 0..w {
                if x >= 8 {
                    guidance[y * w + x] = 1.0;
                    mask[y * w + x] = 1.0;
                }
            }
        }

        let q = guided_filter(&guidance, &mask, w, h, 2, 1e-4);

        // Verify edge sharpness: x <= 6 must be close to 0, x >= 9 must be close to 1
        for y in 0..h {
            assert!(q[y * w + 5] < 0.1);
            assert!(q[y * w + 10] > 0.9);
        }
    }

    #[test]
    fn test_defringing_decontamination() {
        let w = 4;
        let h = 1;
        // Pixel 0, 1: solid foreground (Brown: 120, 60, 20) with alpha 255
        // Pixel 2: edge pixel contaminated with Green background (80, 180, 60) with alpha 100
        // Pixel 3: background (Green: 20, 220, 30) with alpha 0
        let mut orig = RgbaImage::new(w, h);
        orig.put_pixel(0, 0, Rgba([120, 60, 20, 255]));
        orig.put_pixel(1, 0, Rgba([120, 60, 20, 255]));
        orig.put_pixel(2, 0, Rgba([80, 180, 60, 255]));
        orig.put_pixel(3, 0, Rgba([20, 220, 30, 255]));

        let mut mask = GrayImage::new(w, h);
        mask.put_pixel(0, 0, image::Luma([255]));
        mask.put_pixel(1, 0, image::Luma([255]));
        mask.put_pixel(2, 0, image::Luma([100]));
        mask.put_pixel(3, 0, image::Luma([0]));

        let dyn_orig = DynamicImage::ImageRgba8(orig);
        let result = apply_mask_with_refinement(&dyn_orig, &mask, true);

        // Pixel 2 should be pulled toward brown (120, 60, 20), away from green (180)
        let edge_px = result.get_pixel(2, 0);
        assert_eq!(edge_px[3], 100); // alpha preserved
        assert!(edge_px[0] > 80); // Red increased toward 120
        assert!(edge_px[1] < 180); // Green reduced from 180
    }
}
