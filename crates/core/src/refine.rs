//! Advanced edge refinement: Fast Guided Filter, transition zone feathering, and color defringing.

use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};

const GUIDED_BAND_HEIGHT: usize = 128;

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
    box_filter_vertical_band(src, dst, width, height, radius, 0, height, 0, height);
}

/// Computes `box_filter_vertical` for global rows `out_start..out_end`, reading `src` whose
/// row 0 corresponds to `src_y_offset` in a `total_height`-row image and writing to `dst`
/// starting at `src_y_offset` (or `out_start` offset when `dst` is aligned to `src`).
#[allow(clippy::too_many_arguments)]
fn box_filter_vertical_band(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    total_height: usize,
    radius: usize,
    src_y_offset: usize,
    _src_height: usize,
    out_start: usize,
    out_end: usize,
) {
    if out_start >= out_end || width == 0 || total_height == 0 {
        return;
    }

    let r = radius as isize;
    let max_y = total_height as isize - 1;

    for x in 0..width {
        let mut sum = 0.0f32;
        for dy in -r..=r {
            let gy = (out_start as isize + dy).clamp(0, max_y) as usize;
            sum += src[(gy - src_y_offset) * width + x];
        }
        dst[(out_start - src_y_offset) * width + x] = sum;

        for y in (out_start + 1)..out_end {
            let top_y = (y as isize - r - 1).clamp(0, max_y) as usize;
            let bottom_y = (y as isize + r).clamp(0, max_y) as usize;
            sum += src[(bottom_y - src_y_offset) * width + x]
                - src[(top_y - src_y_offset) * width + x];
            dst[(y - src_y_offset) * width + x] = sum;
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

struct BandBuffers {
    guidance: Vec<f32>,
    p: Vec<f32>,
    scratch: Vec<f32>,
    temp: Vec<f32>,
    mean_i: Vec<f32>,
    mean_p: Vec<f32>,
    mean_ip: Vec<f32>,
    mean_ii: Vec<f32>,
    a: Vec<f32>,
    b: Vec<f32>,
    mean_a: Vec<f32>,
    mean_b: Vec<f32>,
}

impl BandBuffers {
    fn new(width: usize, max_in_rows: usize, max_mid_rows: usize) -> Self {
        let in_cap = width * max_in_rows;
        let mid_cap = width * max_mid_rows;
        Self {
            guidance: vec![0.0f32; in_cap],
            p: vec![0.0f32; in_cap],
            scratch: vec![0.0f32; in_cap],
            temp: vec![0.0f32; in_cap],
            mean_i: vec![0.0f32; mid_cap],
            mean_p: vec![0.0f32; mid_cap],
            mean_ip: vec![0.0f32; mid_cap],
            mean_ii: vec![0.0f32; mid_cap],
            a: vec![0.0f32; mid_cap],
            b: vec![0.0f32; mid_cap],
            mean_a: vec![0.0f32; mid_cap],
            mean_b: vec![0.0f32; mid_cap],
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_band(
        &mut self,
        width: usize,
        height: usize,
        radius: usize,
        eps: f32,
        y0: usize,
        y1: usize,
        top_a: usize,
        bot_a: usize,
        top_in: usize,
        bot_in: usize,
    ) {
        let in_rows = bot_in - top_in;
        let mid_rows = bot_a - top_a;
        let in_len = width * in_rows;
        let mid_len = width * mid_rows;
        let n_inv = 1.0f32 / ((2 * radius + 1) * (2 * radius + 1)) as f32;

        let g_slice = &self.guidance[..in_len];
        let p_slice = &self.p[..in_len];
        let scratch = &mut self.scratch[..in_len];
        let temp = &mut self.temp[..in_len];

        // Pass 1: compute mean_I, mean_p, mean_Ip, mean_II for rows top_a..bot_a
        box_filter_horizontal(g_slice, scratch, width, in_rows, radius);
        let a_off = (top_a - top_in) * width;
        let mean_i_band = &mut self.mean_i[..mid_len];
        box_filter_vertical_band_into(
            scratch,
            mean_i_band,
            width,
            height,
            radius,
            top_in,
            top_a,
            bot_a,
        );
        for v in &mut *mean_i_band {
            *v *= n_inv;
        }

        box_filter_horizontal(p_slice, scratch, width, in_rows, radius);
        let mean_p_band = &mut self.mean_p[..mid_len];
        box_filter_vertical_band_into(
            scratch,
            mean_p_band,
            width,
            height,
            radius,
            top_in,
            top_a,
            bot_a,
        );
        for v in &mut *mean_p_band {
            *v *= n_inv;
        }

        for i in 0..in_len {
            temp[i] = g_slice[i] * p_slice[i];
        }
        box_filter_horizontal(temp, scratch, width, in_rows, radius);
        let mean_ip_band = &mut self.mean_ip[..mid_len];
        box_filter_vertical_band_into(
            scratch,
            mean_ip_band,
            width,
            height,
            radius,
            top_in,
            top_a,
            bot_a,
        );
        for v in &mut *mean_ip_band {
            *v *= n_inv;
        }

        for i in 0..in_len {
            temp[i] = g_slice[i] * g_slice[i];
        }
        box_filter_horizontal(temp, scratch, width, in_rows, radius);
        let mean_ii_band = &mut self.mean_ii[..mid_len];
        box_filter_vertical_band_into(
            scratch,
            mean_ii_band,
            width,
            height,
            radius,
            top_in,
            top_a,
            bot_a,
        );
        for v in &mut *mean_ii_band {
            *v *= n_inv;
        }

        // Compute a and b on rows top_a..bot_a
        let a_band = &mut self.a[..mid_len];
        let b_band = &mut self.b[..mid_len];
        for i in 0..mid_len {
            let cov_ip = mean_ip_band[i] - mean_i_band[i] * mean_p_band[i];
            let var_i = mean_ii_band[i] - mean_i_band[i] * mean_i_band[i];
            let a_val = cov_ip / (var_i.max(0.0) + eps);
            let b_val = mean_p_band[i] - a_val * mean_i_band[i];
            a_band[i] = a_val;
            b_band[i] = b_val;
        }

        // Pass 2: box filter a and b to rows y0..y1
        let out_rows = y1 - y0;
        let out_len = width * out_rows;
        let mid_scratch = &mut self.scratch[..mid_len];

        box_filter_horizontal(a_band, mid_scratch, width, mid_rows, radius);
        let mean_a_out = &mut self.mean_a[..out_len];
        box_filter_vertical_band_into(
            mid_scratch,
            mean_a_out,
            width,
            height,
            radius,
            top_a,
            y0,
            y1,
        );
        for v in &mut *mean_a_out {
            *v *= n_inv;
        }

        box_filter_horizontal(b_band, mid_scratch, width, mid_rows, radius);
        let mean_b_out = &mut self.mean_b[..out_len];
        box_filter_vertical_band_into(
            mid_scratch,
            mean_b_out,
            width,
            height,
            radius,
            top_a,
            y0,
            y1,
        );
        for v in &mut *mean_b_out {
            *v *= n_inv;
        }

        let _ = a_off;
    }
}

#[allow(clippy::too_many_arguments)]
fn box_filter_vertical_band_into(
    src: &[f32],
    dst: &mut [f32],
    width: usize,
    total_height: usize,
    radius: usize,
    src_y_offset: usize,
    out_start: usize,
    out_end: usize,
) {
    if out_start >= out_end || width == 0 || total_height == 0 {
        return;
    }

    let r = radius as isize;
    let max_y = total_height as isize - 1;

    for x in 0..width {
        let mut sum = 0.0f32;
        for dy in -r..=r {
            let gy = (out_start as isize + dy).clamp(0, max_y) as usize;
            sum += src[(gy - src_y_offset) * width + x];
        }
        dst[x] = sum;

        for y in (out_start + 1)..out_end {
            let top_y = (y as isize - r - 1).clamp(0, max_y) as usize;
            let bottom_y = (y as isize + r).clamp(0, max_y) as usize;
            sum += src[(bottom_y - src_y_offset) * width + x]
                - src[(top_y - src_y_offset) * width + x];
            dst[(y - out_start) * width + x] = sum;
        }
    }
}

/// Applies Guided Image Filtering to refine the mask edges against the original image guidance.
///
/// Implements $O(N)$ guided filter in horizontal bands using luminance as guidance $I$ and
/// raw mask as filtering input $p$, producing identical results to full-frame execution at
/// a fraction of peak memory.
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
    let mut q = vec![0.0f32; total_pixels];
    if width == 0 || height == 0 {
        return q;
    }

    let band_h = GUIDED_BAND_HEIGHT.min(height);
    let max_mid_rows = band_h + 2 * radius;
    let max_in_rows = band_h + 4 * radius;
    let mut bufs = BandBuffers::new(width, max_in_rows, max_mid_rows);

    let mut y0 = 0;
    while y0 < height {
        let y1 = (y0 + band_h).min(height);
        let top_a = y0.saturating_sub(radius);
        let bot_a = (y1 + radius).min(height);
        let top_in = top_a.saturating_sub(radius);
        let bot_in = (bot_a + radius).min(height);

        let in_len = (bot_in - top_in) * width;
        let src_off = top_in * width;
        bufs.guidance[..in_len].copy_from_slice(&guidance_gray[src_off..src_off + in_len]);
        bufs.p[..in_len].copy_from_slice(&input_mask[src_off..src_off + in_len]);

        bufs.process_band(
            width, height, radius, eps, y0, y1, top_a, bot_a, top_in, bot_in,
        );

        let out_len = (y1 - y0) * width;
        let g_off = (y0 - top_in) * width;
        let out_off = y0 * width;
        for i in 0..out_len {
            q[out_off + i] = bufs.mean_a[i] * bufs.guidance[g_off + i] + bufs.mean_b[i];
        }

        y0 = y1;
    }

    q
}

#[inline]
fn read_pixel_rgb(orig: &DynamicImage, x: u32, y: u32) -> [u8; 3] {
    match orig {
        DynamicImage::ImageRgb8(rgb) => {
            let p = rgb.get_pixel(x, y);
            [p[0], p[1], p[2]]
        }
        DynamicImage::ImageRgba8(rgba) => {
            let p = rgba.get_pixel(x, y);
            [p[0], p[1], p[2]]
        }
        _ => {
            let p = orig.get_pixel(x, y);
            [p[0], p[1], p[2]]
        }
    }
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

    let w = width as usize;
    let h = height as usize;
    let radius = config.radius;
    let band_h = GUIDED_BAND_HEIGHT.min(h);
    let max_mid_rows = band_h + 2 * radius;
    let max_in_rows = band_h + 4 * radius;
    let mut bufs = BandBuffers::new(w, max_in_rows, max_mid_rows);

    let mut refined = GrayImage::new(width, height);
    let mask_raw = raw_mask.as_raw();
    let refined_raw = refined.as_mut();

    let mut y0 = 0;
    while y0 < h {
        let y1 = (y0 + band_h).min(h);
        let top_a = y0.saturating_sub(radius);
        let bot_a = (y1 + radius).min(h);
        let top_in = top_a.saturating_sub(radius);
        let bot_in = (bot_a + radius).min(h);

        let in_rows = bot_in - top_in;
        let in_len = w * in_rows;

        // Populate guidance luminance and normalized mask directly into band buffers (zero full-frame copies)
        match orig {
            DynamicImage::ImageRgb8(rgb) => {
                let rgb_raw = rgb.as_raw();
                let start_px = top_in * w;
                for i in 0..in_len {
                    let base = (start_px + i) * 3;
                    let r = rgb_raw[base] as f32;
                    let g = rgb_raw[base + 1] as f32;
                    let b = rgb_raw[base + 2] as f32;
                    bufs.guidance[i] = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
                    bufs.p[i] = mask_raw[start_px + i] as f32 / 255.0;
                }
            }
            DynamicImage::ImageRgba8(rgba) => {
                let rgba_raw = rgba.as_raw();
                let start_px = top_in * w;
                for i in 0..in_len {
                    let base = (start_px + i) * 4;
                    let r = rgba_raw[base] as f32;
                    let g = rgba_raw[base + 1] as f32;
                    let b = rgba_raw[base + 2] as f32;
                    bufs.guidance[i] = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0;
                    bufs.p[i] = mask_raw[start_px + i] as f32 / 255.0;
                }
            }
            _ => {
                for dy in 0..in_rows {
                    let gy = (top_in + dy) as u32;
                    let row_off = dy * w;
                    let mask_off = (top_in + dy) * w;
                    for x in 0..w {
                        let px = orig.get_pixel(x as u32, gy);
                        bufs.guidance[row_off + x] =
                            (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32)
                                / 255.0;
                        bufs.p[row_off + x] = mask_raw[mask_off + x] as f32 / 255.0;
                    }
                }
            }
        }

        bufs.process_band(
            w, h, radius, config.eps, y0, y1, top_a, bot_a, top_in, bot_in,
        );

        let out_len = (y1 - y0) * w;
        let g_off = (y0 - top_in) * w;
        let dst_off = y0 * w;

        for i in 0..out_len {
            let p_val = bufs.p[g_off + i];
            let q_val =
                (bufs.mean_a[i] * bufs.guidance[g_off + i] + bufs.mean_b[i]).clamp(0.0, 1.0);
            let final_alpha = if p_val <= 0.05 {
                0.0f32
            } else if p_val >= 0.95 {
                1.0f32
            } else {
                q_val
            };
            refined_raw[dst_off + i] = (final_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }

        y0 = y1;
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
    let mut cutout = RgbaImage::new(w, h);
    let mask_raw = mask.as_raw();
    let cutout_raw = cutout.as_mut();
    let w_usize = w as usize;

    if !enable_defringing {
        for y in 0..h {
            for x in 0..w {
                let [r, g, b] = read_pixel_rgb(orig, x, y);
                let idx = (y as usize) * w_usize + (x as usize);
                let base = idx * 4;
                cutout_raw[base] = r;
                cutout_raw[base + 1] = g;
                cutout_raw[base + 2] = b;
                cutout_raw[base + 3] = mask_raw[idx];
            }
        }
        return cutout;
    }

    let r_search: isize = 2; // 5x5 neighborhood search for solid foreground

    for y in 0..h {
        for x in 0..w {
            let idx = (y as usize) * w_usize + (x as usize);
            let alpha = mask_raw[idx];
            let base = idx * 4;

            if alpha == 0 {
                cutout_raw[base] = 0;
                cutout_raw[base + 1] = 0;
                cutout_raw[base + 2] = 0;
                cutout_raw[base + 3] = 0;
                continue;
            }

            let [mut r_out, mut g_out, mut b_out] = read_pixel_rgb(orig, x, y);

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

                        let n_alpha = mask_raw[(ny as usize) * w_usize + (nx as usize)];
                        if n_alpha >= 235 {
                            let [nr, ng, nb] = read_pixel_rgb(orig, nx as u32, ny as u32);
                            sum_r += nr as f32;
                            sum_g += ng as f32;
                            sum_b += nb as f32;
                            fg_count += 1.0;
                        }
                    }
                }

                if fg_count > 0.0 {
                    let avg_r = sum_r / fg_count;
                    let avg_g = sum_g / fg_count;
                    let avg_b = sum_b / fg_count;

                    let blend_fg = 1.0 - (alpha as f32 / 255.0);
                    r_out = ((r_out as f32) * (1.0 - blend_fg) + avg_r * blend_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    g_out = ((g_out as f32) * (1.0 - blend_fg) + avg_g * blend_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    b_out = ((b_out as f32) * (1.0 - blend_fg) + avg_b * blend_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }

            cutout_raw[base] = r_out;
            cutout_raw[base + 1] = g_out;
            cutout_raw[base + 2] = b_out;
            cutout_raw[base + 3] = alpha;
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

        let n = (2 * radius + 1) * (2 * radius + 1);
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

        for y in 0..h {
            for x in 0..w {
                if x >= 8 {
                    guidance[y * w + x] = 1.0;
                    mask[y * w + x] = 1.0;
                }
            }
        }

        let q = guided_filter(&guidance, &mask, w, h, 2, 1e-4);

        for y in 0..h {
            assert!(q[y * w + 5] < 0.1);
            assert!(q[y * w + 10] > 0.9);
        }
    }

    #[test]
    fn test_guided_filter_multi_band_matches_single_band() {
        let w = 40;
        let h = GUIDED_BAND_HEIGHT * 2 + 19;
        let mut guidance = vec![0.0f32; w * h];
        let mut mask = vec![0.0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                guidance[y * w + x] = ((x * 7 + y * 13) % 256) as f32 / 255.0;
                mask[y * w + x] = if x + y / 4 > 25 { 0.9 } else { 0.1 };
            }
        }
        let q = guided_filter(&guidance, &mask, w, h, 4, 1e-4);
        // Compare against full-frame reference calculation
        let n = w * h;
        let n_inv = 1.0f32 / 81.0f32;
        let mut scratch = vec![0.0f32; n];
        let mut temp = vec![0.0f32; n];
        let mut mean_i = vec![0.0f32; n];
        let mut mean_p = vec![0.0f32; n];
        let mut mean_ip = vec![0.0f32; n];
        let mut mean_ii = vec![0.0f32; n];
        box_filter_2d(&guidance, &mut mean_i, &mut scratch, w, h, 4);
        for v in &mut mean_i {
            *v *= n_inv;
        }
        box_filter_2d(&mask, &mut mean_p, &mut scratch, w, h, 4);
        for v in &mut mean_p {
            *v *= n_inv;
        }
        for i in 0..n {
            temp[i] = guidance[i] * mask[i];
        }
        box_filter_2d(&temp, &mut mean_ip, &mut scratch, w, h, 4);
        for v in &mut mean_ip {
            *v *= n_inv;
        }
        for i in 0..n {
            temp[i] = guidance[i] * guidance[i];
        }
        box_filter_2d(&temp, &mut mean_ii, &mut scratch, w, h, 4);
        for v in &mut mean_ii {
            *v *= n_inv;
        }
        let mut a = vec![0.0f32; n];
        let mut b = vec![0.0f32; n];
        for i in 0..n {
            let cov = mean_ip[i] - mean_i[i] * mean_p[i];
            let var = mean_ii[i] - mean_i[i] * mean_i[i];
            a[i] = cov / (var.max(0.0) + 1e-4);
            b[i] = mean_p[i] - a[i] * mean_i[i];
        }
        let mut mean_a = vec![0.0f32; n];
        let mut mean_b = vec![0.0f32; n];
        box_filter_2d(&a, &mut mean_a, &mut scratch, w, h, 4);
        for v in &mut mean_a {
            *v *= n_inv;
        }
        box_filter_2d(&b, &mut mean_b, &mut scratch, w, h, 4);
        for v in &mut mean_b {
            *v *= n_inv;
        }
        let mut max_diff = 0.0f32;
        let mut max_u8_diff = 0i32;
        for i in 0..n {
            let q_ref = mean_a[i] * guidance[i] + mean_b[i];
            max_diff = max_diff.max((q[i] - q_ref).abs());
            let b1 = (q[i].clamp(0.0, 1.0) * 255.0).round() as i32;
            let b2 = (q_ref.clamp(0.0, 1.0) * 255.0).round() as i32;
            max_u8_diff = max_u8_diff.max((b1 - b2).abs());
        }
        assert!(max_diff < 1e-4, "max_diff={max_diff}");
        assert!(max_u8_diff <= 1, "max_u8_diff={max_u8_diff}");
    }

    #[test]
    fn test_defringing_decontamination() {
        use image::Rgba;
        let w = 4;
        let h = 1;
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

        let edge_px = result.get_pixel(2, 0);
        assert_eq!(edge_px[3], 100);
        assert!(edge_px[0] > 80);
        assert!(edge_px[1] < 180);
    }
}
