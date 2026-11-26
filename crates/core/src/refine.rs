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

#[derive(Clone, Copy)]
struct BandWindow {
    width: usize,
    total_height: usize,
    radius: usize,
    src_y_offset: usize,
    src_rows: usize,
    out_start: usize,
    out_end: usize,
    n_inv: f32,
}

#[inline]
fn box_filter_mean_band(src: &[f32], scratch: &mut [f32], dst: &mut [f32], win: BandWindow) {
    let BandWindow {
        width,
        total_height,
        radius,
        src_y_offset,
        src_rows,
        out_start,
        out_end,
        n_inv,
    } = win;
    let in_len = width * src_rows;
    let out_len = width * (out_end - out_start);
    box_filter_horizontal(
        &src[..in_len],
        &mut scratch[..in_len],
        width,
        src_rows,
        radius,
    );
    box_filter_vertical_band_into(
        &scratch[..in_len],
        &mut dst[..out_len],
        width,
        total_height,
        radius,
        src_y_offset,
        out_start,
        out_end,
    );
    for v in &mut dst[..out_len] {
        *v *= n_inv;
    }
}

struct ColorBandBuffers {
    r: Vec<f32>,
    g: Vec<f32>,
    b: Vec<f32>,
    p: Vec<f32>,
    scratch: Vec<f32>,
    temp: Vec<f32>,
    mean_r: Vec<f32>,
    mean_g: Vec<f32>,
    mean_b: Vec<f32>,
    mean_p: Vec<f32>,
    var_rr: Vec<f32>,
    var_rg: Vec<f32>,
    var_rb: Vec<f32>,
    var_gg: Vec<f32>,
    var_gb: Vec<f32>,
    var_bb: Vec<f32>,
    cov_rp: Vec<f32>,
    cov_gp: Vec<f32>,
    cov_bp: Vec<f32>,
}

impl ColorBandBuffers {
    fn new(width: usize, max_in_rows: usize, max_mid_rows: usize) -> Self {
        let in_cap = width * max_in_rows;
        let mid_cap = width * max_mid_rows;
        Self {
            r: vec![0.0f32; in_cap],
            g: vec![0.0f32; in_cap],
            b: vec![0.0f32; in_cap],
            p: vec![0.0f32; in_cap],
            scratch: vec![0.0f32; in_cap],
            temp: vec![0.0f32; in_cap],
            mean_r: vec![0.0f32; mid_cap],
            mean_g: vec![0.0f32; mid_cap],
            mean_b: vec![0.0f32; mid_cap],
            mean_p: vec![0.0f32; mid_cap],
            var_rr: vec![0.0f32; mid_cap],
            var_rg: vec![0.0f32; mid_cap],
            var_rb: vec![0.0f32; mid_cap],
            var_gg: vec![0.0f32; mid_cap],
            var_gb: vec![0.0f32; mid_cap],
            var_bb: vec![0.0f32; mid_cap],
            cov_rp: vec![0.0f32; mid_cap],
            cov_gp: vec![0.0f32; mid_cap],
            cov_bp: vec![0.0f32; mid_cap],
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
        let out_len = width * (y1 - y0);
        let n_inv = 1.0f32 / ((2 * radius + 1) * (2 * radius + 1)) as f32;

        // Pass 1: means of R, G, B, p
        let mean_win = BandWindow {
            width,
            total_height: height,
            radius,
            src_y_offset: top_in,
            src_rows: in_rows,
            out_start: top_a,
            out_end: bot_a,
            n_inv,
        };
        box_filter_mean_band(&self.r, &mut self.scratch, &mut self.mean_r, mean_win);
        box_filter_mean_band(&self.g, &mut self.scratch, &mut self.mean_g, mean_win);
        box_filter_mean_band(&self.b, &mut self.scratch, &mut self.mean_b, mean_win);
        box_filter_mean_band(&self.p, &mut self.scratch, &mut self.mean_p, mean_win);

        // Pass 1b: 3x3 covariance of RGB guidance and cross-covariance with p
        let pairs: [(&[f32], &[f32], &mut [f32]); 9] = [
            (&self.r, &self.r, &mut self.var_rr),
            (&self.r, &self.g, &mut self.var_rg),
            (&self.r, &self.b, &mut self.var_rb),
            (&self.g, &self.g, &mut self.var_gg),
            (&self.g, &self.b, &mut self.var_gb),
            (&self.b, &self.b, &mut self.var_bb),
            (&self.r, &self.p, &mut self.cov_rp),
            (&self.g, &self.p, &mut self.cov_gp),
            (&self.b, &self.p, &mut self.cov_bp),
        ];
        for (c1, c2, dst) in pairs {
            for i in 0..in_len {
                self.temp[i] = c1[i] * c2[i];
            }
            box_filter_mean_band(&self.temp, &mut self.scratch, dst, mean_win);
        }

        // Solve 3x3 linear system (Sigma + eps*I) * a = cov_Ip in-place (storing a_r,a_g,a_b,b into cov_rp,cov_gp,cov_bp,mean_p).
        for i in 0..mid_len {
            let mr = self.mean_r[i];
            let mg = self.mean_g[i];
            let mb = self.mean_b[i];
            let mp = self.mean_p[i];

            let c00 = (self.var_rr[i] - mr * mr).max(0.0) + eps;
            let c01 = self.var_rg[i] - mr * mg;
            let c02 = self.var_rb[i] - mr * mb;
            let c11 = (self.var_gg[i] - mg * mg).max(0.0) + eps;
            let c12 = self.var_gb[i] - mg * mb;
            let c22 = (self.var_bb[i] - mb * mb).max(0.0) + eps;

            let crp = self.cov_rp[i] - mr * mp;
            let cgp = self.cov_gp[i] - mg * mp;
            let cbp = self.cov_bp[i] - mb * mp;

            let cof00 = c11 * c22 - c12 * c12;
            let cof01 = c02 * c12 - c01 * c22;
            let cof02 = c01 * c12 - c02 * c11;
            let cof11 = c00 * c22 - c02 * c02;
            let cof12 = c01 * c02 - c00 * c12;
            let cof22 = c00 * c11 - c01 * c01;

            let det = c00 * cof00 + c01 * cof01 + c02 * cof02;
            let (ar, ag, ab) = if det > 1e-12 {
                let inv_det = 1.0 / det;
                (
                    (cof00 * crp + cof01 * cgp + cof02 * cbp) * inv_det,
                    (cof01 * crp + cof11 * cgp + cof12 * cbp) * inv_det,
                    (cof02 * crp + cof12 * cgp + cof22 * cbp) * inv_det,
                )
            } else {
                (0.0, 0.0, 0.0)
            };

            self.cov_rp[i] = ar;
            self.cov_gp[i] = ag;
            self.cov_bp[i] = ab;
            self.mean_p[i] = mp - (ar * mr + ag * mg + ab * mb);
        }

        // Pass 2: average (a_r, a_g, a_b, b) into var_rr, var_rg, var_rb, var_gg for output rows y0..y1
        let out_win = BandWindow {
            width,
            total_height: height,
            radius,
            src_y_offset: top_a,
            src_rows: mid_rows,
            out_start: y0,
            out_end: y1,
            n_inv,
        };
        box_filter_mean_band(&self.cov_rp, &mut self.scratch, &mut self.var_rr, out_win);
        box_filter_mean_band(&self.cov_gp, &mut self.scratch, &mut self.var_rg, out_win);
        box_filter_mean_band(&self.cov_bp, &mut self.scratch, &mut self.var_rb, out_win);
        box_filter_mean_band(&self.mean_p, &mut self.scratch, &mut self.var_gg, out_win);
        let _ = out_len;
    }
}

/// Refines a mask using 3-channel RGB Color Guided Filter with transition haze suppression.
///
/// Solid foreground ($\ge 0.95$) and definite background ($\le 0.06$) remain sharp, while
/// boundary pixels receive RGB edge-preserving alignment that distinguishes chromatic edges
/// even when foreground and background share similar luminance.
#[must_use]
pub fn refine_mask_guided(
    raw_mask: GrayImage,
    orig: &DynamicImage,
    config: &RefineConfig,
) -> GrayImage {
    refine_mask_guided_banded(raw_mask, orig, config, GUIDED_BAND_HEIGHT)
}

fn refine_mask_guided_banded(
    raw_mask: GrayImage,
    orig: &DynamicImage,
    config: &RefineConfig,
    band_height: usize,
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
    let band_h = band_height.max(1).min(h);
    let max_mid_rows = band_h + 2 * radius;
    let max_in_rows = band_h + 4 * radius;
    let mut bufs = ColorBandBuffers::new(w, max_in_rows, max_mid_rows);

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
        let start_px = top_in * w;

        // Skip expensive guided math if every pixel in the extended band is solidly background or foreground.
        let mut all_bg = true;
        let mut all_fg = true;
        for &m in &mask_raw[start_px..start_px + in_len] {
            if m > 15 {
                all_bg = false;
            }
            if m < 243 {
                all_fg = false;
            }
            if !all_bg && !all_fg {
                break;
            }
        }
        if all_bg || all_fg {
            let out_len = (y1 - y0) * w;
            let dst_off = y0 * w;
            let fill = if all_fg { 255u8 } else { 0u8 };
            refined_raw[dst_off..dst_off + out_len].fill(fill);
            y0 = y1;
            continue;
        }

        match orig {
            DynamicImage::ImageRgb8(rgb) => {
                let rgb_raw = rgb.as_raw();
                for i in 0..in_len {
                    let base = (start_px + i) * 3;
                    bufs.r[i] = rgb_raw[base] as f32 / 255.0;
                    bufs.g[i] = rgb_raw[base + 1] as f32 / 255.0;
                    bufs.b[i] = rgb_raw[base + 2] as f32 / 255.0;
                    bufs.p[i] = mask_raw[start_px + i] as f32 / 255.0;
                }
            }
            DynamicImage::ImageRgba8(rgba) => {
                let rgba_raw = rgba.as_raw();
                for i in 0..in_len {
                    let base = (start_px + i) * 4;
                    bufs.r[i] = rgba_raw[base] as f32 / 255.0;
                    bufs.g[i] = rgba_raw[base + 1] as f32 / 255.0;
                    bufs.b[i] = rgba_raw[base + 2] as f32 / 255.0;
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
                        bufs.r[row_off + x] = px[0] as f32 / 255.0;
                        bufs.g[row_off + x] = px[1] as f32 / 255.0;
                        bufs.b[row_off + x] = px[2] as f32 / 255.0;
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
            let final_alpha = if p_val <= 0.06 {
                0.0f32
            } else if p_val >= 0.95 {
                1.0f32
            } else {
                let q_val = (bufs.var_rr[i] * bufs.r[g_off + i]
                    + bufs.var_rg[i] * bufs.g[g_off + i]
                    + bufs.var_rb[i] * bufs.b[g_off + i]
                    + bufs.var_gg[i])
                    .clamp(0.0, 1.0);
                // Suppress faint low-confidence smear haze while leaving true guided hair strands intact.
                if q_val <= 0.06 {
                    0.0
                } else if q_val < 0.14 {
                    q_val * ((q_val - 0.06) / 0.08)
                } else {
                    q_val
                }
            };
            refined_raw[dst_off + i] = (final_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
        }

        y0 = y1;
    }

    refined
}

/// Blends the original color channels with the alpha mask, applying banded multi-pass
/// boundary color decontamination to eliminate background bleed halos on edge pixels.
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
    let h_usize = h as usize;

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

    if w_usize == 0 || h_usize == 0 {
        return cutout;
    }

    const SOLID_ALPHA_MIN: u8 = 235;
    const DEFRINGE_PASSES: usize = 6;
    let band_h = GUIDED_BAND_HEIGHT.min(h_usize);
    let max_rows = band_h + 2 * DEFRINGE_PASSES;
    let cap = w_usize * max_rows;

    let mut fg_r = vec![0u8; cap];
    let mut fg_g = vec![0u8; cap];
    let mut fg_b = vec![0u8; cap];
    let mut has_fg = vec![0u8; cap];
    let mut next_r = vec![0u8; cap];
    let mut next_g = vec![0u8; cap];
    let mut next_b = vec![0u8; cap];
    let mut next_has = vec![0u8; cap];

    let mut y0 = 0usize;
    while y0 < h_usize {
        let y1 = (y0 + band_h).min(h_usize);
        let top = y0.saturating_sub(DEFRINGE_PASSES);
        let bot = (y1 + DEFRINGE_PASSES).min(h_usize);
        let rows = bot - top;
        let len = rows * w_usize;

        for by in 0..rows {
            let gy = (top + by) as u32;
            let row_off = by * w_usize;
            let global_row = (top + by) * w_usize;
            for x in 0..w_usize {
                let idx = row_off + x;
                let a = mask_raw[global_row + x];
                if a >= SOLID_ALPHA_MIN {
                    let [r, g, b] = read_pixel_rgb(orig, x as u32, gy);
                    fg_r[idx] = r;
                    fg_g[idx] = g;
                    fg_b[idx] = b;
                    has_fg[idx] = 1;
                } else {
                    has_fg[idx] = 0;
                }
            }
        }

        // Propagate nearest solid foreground color outward up to DEFRINGE_PASSES pixels.
        for _ in 0..DEFRINGE_PASSES {
            next_r[..len].copy_from_slice(&fg_r[..len]);
            next_g[..len].copy_from_slice(&fg_g[..len]);
            next_b[..len].copy_from_slice(&fg_b[..len]);
            next_has[..len].copy_from_slice(&has_fg[..len]);
            let mut changed = false;

            for by in 0..rows {
                let row_off = by * w_usize;
                let global_row = (top + by) * w_usize;
                for x in 0..w_usize {
                    let idx = row_off + x;
                    let a = mask_raw[global_row + x];
                    if a == 0 || has_fg[idx] != 0 {
                        continue;
                    }

                    let mut sr = 0u32;
                    let mut sg = 0u32;
                    let mut sb = 0u32;
                    let mut count = 0u32;

                    let y_min = by.saturating_sub(1);
                    let y_max = (by + 1).min(rows - 1);
                    let x_min = x.saturating_sub(1);
                    let x_max = (x + 1).min(w_usize - 1);
                    for ny in y_min..=y_max {
                        let n_off = ny * w_usize;
                        for nx in x_min..=x_max {
                            let n_idx = n_off + nx;
                            if has_fg[n_idx] != 0 {
                                sr += fg_r[n_idx] as u32;
                                sg += fg_g[n_idx] as u32;
                                sb += fg_b[n_idx] as u32;
                                count += 1;
                            }
                        }
                    }

                    let half = count / 2;
                    if let (Some(r), Some(g), Some(b)) = (
                        (sr + half).checked_div(count),
                        (sg + half).checked_div(count),
                        (sb + half).checked_div(count),
                    ) {
                        next_r[idx] = r as u8;
                        next_g[idx] = g as u8;
                        next_b[idx] = b as u8;
                        next_has[idx] = 1;
                        changed = true;
                    }
                }
            }

            fg_r[..len].copy_from_slice(&next_r[..len]);
            fg_g[..len].copy_from_slice(&next_g[..len]);
            fg_b[..len].copy_from_slice(&next_b[..len]);
            has_fg[..len].copy_from_slice(&next_has[..len]);
            if !changed {
                break;
            }
        }

        for gy in y0..y1 {
            let by = gy - top;
            let band_row = by * w_usize;
            let global_row = gy * w_usize;
            for x in 0..w_usize {
                let g_idx = global_row + x;
                let b_idx = band_row + x;
                let alpha = mask_raw[g_idx];
                let base = g_idx * 4;

                if alpha == 0 {
                    cutout_raw[base] = 0;
                    cutout_raw[base + 1] = 0;
                    cutout_raw[base + 2] = 0;
                    cutout_raw[base + 3] = 0;
                    continue;
                }

                let [mut r_out, mut g_out, mut b_out] = read_pixel_rgb(orig, x as u32, gy as u32);
                if alpha < SOLID_ALPHA_MIN && has_fg[b_idx] != 0 {
                    // Replace contaminated edge color strongly on low/mid alpha so old background bleed disappears.
                    let w_fg = ((SOLID_ALPHA_MIN as f32 - alpha as f32) / 75.0).clamp(0.0, 1.0);
                    r_out = ((r_out as f32) * (1.0 - w_fg) + (fg_r[b_idx] as f32) * w_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    g_out = ((g_out as f32) * (1.0 - w_fg) + (fg_g[b_idx] as f32) * w_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    b_out = ((b_out as f32) * (1.0 - w_fg) + (fg_b[b_idx] as f32) * w_fg)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }

                cutout_raw[base] = r_out;
                cutout_raw[base + 1] = g_out;
                cutout_raw[base + 2] = b_out;
                cutout_raw[base + 3] = alpha;
            }
        }

        y0 = y1;
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

    #[test]
    fn test_color_guided_filter_preserves_isoluminant_chroma_edge() {
        use image::{Rgb, RgbImage};
        // Foreground (right) and background (left) have virtually identical luminance (~170)
        // but differ sharply in RGB chroma (yellowish vs cool purple-blue).
        let w = 24u32;
        let h = 24u32;
        let mut rgb = RgbImage::new(w, h);
        let mut mask = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if x < 12 {
                    rgb.put_pixel(x, y, Rgb([200, 180, 40]));
                    mask.put_pixel(x, y, image::Luma([if x >= 10 { 80 } else { 0 }]));
                } else {
                    rgb.put_pixel(x, y, Rgb([148, 180, 210]));
                    mask.put_pixel(x, y, image::Luma([if x <= 13 { 180 } else { 255 }]));
                }
            }
        }

        let dyn_orig = DynamicImage::ImageRgb8(rgb);
        let refined = refine_mask_guided(mask, &dyn_orig, &RefineConfig::default());

        assert!(refined.get_pixel(9, 12)[0] < 30);
        assert!(refined.get_pixel(14, 12)[0] > 220);
        assert!(refined.get_pixel(11, 12)[0] < refined.get_pixel(12, 12)[0]);
    }

    #[test]
    fn test_defringing_reaches_outer_edge_pixels_across_band() {
        use image::Rgba;
        // Solid foreground at x=0..=1, contaminated yellow transition at x=2..=5 (up to 4px out).
        let w = 8u32;
        let h = (GUIDED_BAND_HEIGHT + 4) as u32;
        let mut orig = RgbaImage::new(w, h);
        let mut mask = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if x <= 1 {
                    orig.put_pixel(x, y, Rgba([120, 60, 40, 255]));
                    mask.put_pixel(x, y, image::Luma([255]));
                } else if x <= 5 {
                    orig.put_pixel(x, y, Rgba([230, 220, 20, 255]));
                    let a = (200 - (x - 2) * 40) as u8;
                    mask.put_pixel(x, y, image::Luma([a]));
                } else {
                    orig.put_pixel(x, y, Rgba([240, 230, 10, 255]));
                    mask.put_pixel(x, y, image::Luma([0]));
                }
            }
        }

        let dyn_orig = DynamicImage::ImageRgba8(orig);
        let result = apply_mask_with_refinement(&dyn_orig, &mask, true);

        // Even 4 pixels from the solid foreground (x=5, alpha=80), yellow bleed should be replaced.
        let far_edge = result.get_pixel(5, GUIDED_BAND_HEIGHT as u32);
        assert_eq!(far_edge[3], 80);
        assert!(
            far_edge[1] < 100,
            "Expected green channel defringed, got {:?}",
            far_edge
        );
        assert!(
            far_edge[0] < 160,
            "Expected red channel defringed, got {:?}",
            far_edge
        );
        assert_eq!(result.get_pixel(6, 0), &Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn test_color_guided_filter_multi_band_no_seam() {
        use image::{Rgb, RgbImage};
        let w = 32u32;
        let h = (GUIDED_BAND_HEIGHT * 2 + 19) as u32;
        let mut rgb = RgbImage::new(w, h);
        let mut mask = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                if x < 16 {
                    rgb.put_pixel(x, y, Rgb([200, 180, 40]));
                    mask.put_pixel(x, y, image::Luma([if x >= 14 { 80 } else { 0 }]));
                } else {
                    rgb.put_pixel(x, y, Rgb([148, 180, 210]));
                    mask.put_pixel(x, y, image::Luma([if x <= 17 { 180 } else { 255 }]));
                }
            }
        }

        let dyn_orig = DynamicImage::ImageRgb8(rgb);
        let refined = refine_mask_guided(mask, &dyn_orig, &RefineConfig::default());

        for x in 0..w {
            let ref_px = refined.get_pixel(x, 64)[0];
            for y in [
                (GUIDED_BAND_HEIGHT - 1) as u32,
                GUIDED_BAND_HEIGHT as u32,
                (GUIDED_BAND_HEIGHT + 1) as u32,
                (GUIDED_BAND_HEIGHT * 2 - 1) as u32,
                (GUIDED_BAND_HEIGHT * 2) as u32,
                (GUIDED_BAND_HEIGHT * 2 + 1) as u32,
            ] {
                assert_eq!(
                    refined.get_pixel(x, y)[0],
                    ref_px,
                    "Band seam detected at x={x}, y={y}"
                );
            }
        }
    }

    #[test]
    fn test_color_guided_filter_early_exit_threshold_matches_float_boundary() {
        use image::{Rgb, RgbImage};
        // 242/255 = 0.9490 < 0.95, so an image filled with 242 must not skip to 255 unless >= 243.
        let w = 16u32;
        let h = 16u32;
        let rgb = RgbImage::from_pixel(w, h, Rgb([120, 120, 120]));
        let mask_242 = GrayImage::from_pixel(w, h, image::Luma([242]));
        let mask_243 = GrayImage::from_pixel(w, h, image::Luma([243]));
        let dyn_orig = DynamicImage::ImageRgb8(rgb);

        let out_242 = refine_mask_guided(mask_242, &dyn_orig, &RefineConfig::default());
        let out_243 = refine_mask_guided(mask_243, &dyn_orig, &RefineConfig::default());

        assert_eq!(out_242.get_pixel(8, 8)[0], 242);
        assert_eq!(out_243.get_pixel(8, 8)[0], 255);
    }

    #[test]
    fn test_color_guided_filter_multi_band_matches_full_frame() {
        use image::{Rgb, RgbImage};
        let w = 64u32;
        let h = (GUIDED_BAND_HEIGHT * 3 + 37) as u32;
        let mut rgb = RgbImage::new(w, h);
        let mut mask = GrayImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let r = ((x * 17 + y * 31) % 256) as u8;
                let g = ((x * 41 + y * 13 + 70) % 256) as u8;
                let b = ((x * 23 + y * 53 + 140) % 256) as u8;
                rgb.put_pixel(x, y, Rgb([r, g, b]));
                let m = if x + (y / 8) > 45 {
                    245u8
                } else if x + (y / 8) > 35 {
                    ((x + y / 8 - 35) * 20 + 20) as u8
                } else {
                    10u8
                };
                mask.put_pixel(x, y, image::Luma([m]));
            }
        }
        let dyn_orig = DynamicImage::ImageRgb8(rgb);
        let banded = refine_mask_guided_banded(
            mask.clone(),
            &dyn_orig,
            &RefineConfig::default(),
            GUIDED_BAND_HEIGHT,
        );
        let full = refine_mask_guided_banded(mask, &dyn_orig, &RefineConfig::default(), h as usize);

        let mut max_diff = 0i32;
        for y in 0..h {
            for x in 0..w {
                let d = (banded.get_pixel(x, y)[0] as i32 - full.get_pixel(x, y)[0] as i32).abs();
                max_diff = max_diff.max(d);
            }
        }
        assert!(max_diff <= 1, "Banded vs full-frame max diff={max_diff}");
    }
}
