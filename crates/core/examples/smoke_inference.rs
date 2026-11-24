use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use image::imageops::FilterType;
use image::metadata::Orientation;
use image::{DynamicImage, GenericImageView, GrayImage, ImageDecoder, ImageReader, RgbaImage};
use ndarray::Array4;
use ort::session::Session;
use ort::value::Tensor;

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[cfg(windows)]
#[link(name = "psapi")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn GetProcessMemoryInfo(
        process: *mut std::ffi::c_void,
        ppsmem_counters: *mut ProcessMemoryCounters,
        cb: u32,
    ) -> i32;
}

#[cfg(windows)]
fn get_peak_working_set_mb() -> f64 {
    unsafe {
        let mut pmc = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            ..Default::default()
        };
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) != 0 {
            pmc.peak_working_set_size as f64 / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }
}

#[cfg(not(windows))]
fn get_peak_working_set_mb() -> f64 {
    0.0
}

#[derive(Debug, Clone, Default)]
struct RunTimings {
    decode_ms: f64,
    preprocess_ms: f64,
    inference_ms: f64,
    postprocess_ms: f64,
    total_ms: f64,
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn decode_image_with_orientation(path: &Path) -> Result<DynamicImage> {
    let reader = ImageReader::open(path)
        .with_context(|| format!("Failed to open image file: {}", path.display()))?;
    let mut decoder = reader
        .into_decoder()
        .with_context(|| format!("Failed to create decoder for: {}", path.display()))?;
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)
        .with_context(|| format!("Failed to decode image: {}", path.display()))?;
    img.apply_orientation(orientation);
    Ok(img)
}

fn preprocess_tensor(img: &DynamicImage) -> Array4<f32> {
    let rgb = img.to_rgb8();
    let resized = image::imageops::resize(&rgb, 1024, 1024, FilterType::Triangle);

    let mut tensor = Array4::<f32>::zeros((1, 3, 1024, 1024));
    let mean = [0.485_f32, 0.456_f32, 0.406_f32];
    let std = [0.229_f32, 0.224_f32, 0.225_f32];

    for y in 0..1024 {
        for x in 0..1024 {
            let px = resized.get_pixel(x, y);
            tensor[[0, 0, y as usize, x as usize]] = (px[0] as f32 / 255.0 - mean[0]) / std[0];
            tensor[[0, 1, y as usize, x as usize]] = (px[1] as f32 / 255.0 - mean[1]) / std[1];
            tensor[[0, 2, y as usize, x as usize]] = (px[2] as f32 / 255.0 - mean[2]) / std[2];
        }
    }

    tensor
}

fn postprocess_cutout(img: &DynamicImage, raw_logits: &[f32]) -> Result<(GrayImage, RgbaImage)> {
    let (orig_w, orig_h) = img.dimensions();

    // 1. Sigmoid activation mapping raw logits to [0, 255] grayscale mask
    let mut mask_1024 = Vec::with_capacity(1024 * 1024);
    for &logit in raw_logits {
        let prob = 1.0 / (1.0 + (-logit).exp());
        let val = (prob * 255.0).clamp(0.0, 255.0) as u8;
        mask_1024.push(val);
    }

    let mask_img_1024 = GrayImage::from_raw(1024, 1024, mask_1024)
        .context("Failed to create 1024x1024 grayscale mask buffer")?;

    // 2. Bilinear (Triangle) resize back to source image resolution
    let full_mask = image::imageops::resize(&mask_img_1024, orig_w, orig_h, FilterType::Triangle);

    // 3. Compose RGBA cutout
    let mut cutout = img.to_rgba8();
    for y in 0..orig_h {
        for x in 0..orig_w {
            let alpha = full_mask.get_pixel(x, y)[0];
            let px = cutout.get_pixel_mut(x, y);
            px[3] = alpha;
        }
    }

    Ok((full_mask, cutout))
}

fn run_single(
    image_path: &Path,
    session: &mut Session,
    save_outputs: bool,
    out_dir: &Path,
) -> Result<RunTimings> {
    let t_total_start = Instant::now();

    // 1. Decode
    let t_dec_start = Instant::now();
    let img = decode_image_with_orientation(image_path)?;
    let dec_dur = t_dec_start.elapsed().as_secs_f64() * 1000.0;

    // 2. Preprocess
    let t_pre_start = Instant::now();
    let tensor_data = preprocess_tensor(&img);
    let tensor = Tensor::from_array(tensor_data)?;
    let pre_dur = t_pre_start.elapsed().as_secs_f64() * 1000.0;

    // 3. Inference
    let t_inf_start = Instant::now();
    let outputs = session.run(ort::inputs!["input_image" => tensor])?;
    let inf_dur = t_inf_start.elapsed().as_secs_f64() * 1000.0;

    // 4. Postprocess
    let t_post_start = Instant::now();
    let (_, raw_slice) = outputs[0].try_extract_tensor::<f32>()?;
    let (mask, cutout) = postprocess_cutout(&img, raw_slice)?;
    let post_dur = t_post_start.elapsed().as_secs_f64() * 1000.0;

    let total_dur = t_total_start.elapsed().as_secs_f64() * 1000.0;

    if save_outputs {
        let stem = image_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let mask_path = out_dir.join(format!("{stem}_mask.png"));
        let cutout_path = out_dir.join(format!("{stem}_cutout.png"));

        mask.save(&mask_path)
            .with_context(|| format!("Failed to save mask to: {}", mask_path.display()))?;
        cutout
            .save(&cutout_path)
            .with_context(|| format!("Failed to save cutout to: {}", cutout_path.display()))?;
    }

    Ok(RunTimings {
        decode_ms: dec_dur,
        preprocess_ms: pre_dur,
        inference_ms: inf_dur,
        postprocess_ms: post_dur,
        total_ms: total_dur,
    })
}

fn main() -> Result<()> {
    let out_dir = PathBuf::from("bench/out");
    fs::create_dir_all(&out_dir)?;

    let environment = ort::init()
        .with_name("rustmatte")
        .with_telemetry(false)
        .build()?;

    let mut session = Session::builder(&environment)
        .map_err(|e| anyhow!("{e}"))?
        .with_memory_pattern(false)
        .map_err(|e| anyhow!("{e}"))?
        .with_config_entry("session.use_env_allocators", "1")
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(4)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file("models/birefnet_lite.onnx")?;

    let test_images = [
        "tests/fixtures/rambut1.jpg",     // Fine hair portrait
        "tests/fixtures/hewan1.jpg",      // Animal / fur
        "tests/fixtures/produk1.jpg",     // Product
        "tests/fixtures/kaca1.jpg",       // Glass / transparent object
        "tests/fixtures/latarramai1.jpg", // Complex / busy background
        "tests/fixtures/orang1.jpg",      // Person
    ];

    println!("================================================================================");
    println!("RustMatte Phase 1A: BiRefNet_lite CPU Inference Smoke Test (env_allocators)");
    println!("Hardware: Intel Core i5-10400 (6 Cores, 12 Threads), 16GB RAM");
    println!("ORT Settings: intra_threads = 4, with_env_allocators = 1");
    println!("================================================================================");

    for img_path_str in &test_images {
        let img_path = Path::new(img_path_str);
        if !img_path.exists() {
            eprintln!("Warning: file not found: {img_path_str}");
            continue;
        }

        println!("\nTesting: {img_path_str}");

        // 1 Warm-up run (discarded, saves outputs to disk)
        let _ = run_single(img_path, &mut session, true, &out_dir).context("Warm-up run failed")?;

        // 5 Timed runs
        let mut decode_times = Vec::new();
        let mut prep_times = Vec::new();
        let mut inf_times = Vec::new();
        let mut post_times = Vec::new();
        let mut total_times = Vec::new();

        for _ in 0..5 {
            let t = run_single(img_path, &mut session, false, &out_dir)?;
            decode_times.push(t.decode_ms);
            prep_times.push(t.preprocess_ms);
            inf_times.push(t.inference_ms);
            post_times.push(t.postprocess_ms);
            total_times.push(t.total_ms);
        }

        let med_decode = median(decode_times);
        let med_prep = median(prep_times);
        let med_inf = median(inf_times);
        let med_post = median(post_times);
        let med_total = median(total_times);

        println!(
            "  Median Timings (ms): Decode={:.1}ms | Preprocess={:.1}ms | Inference={:.1}ms | Postprocess={:.1}ms | Total={:.1}ms",
            med_decode, med_prep, med_inf, med_post, med_total
        );
    }

    let peak_mb = get_peak_working_set_mb();
    println!(
        "\nPeak Working Set Memory: {:.1} MB (measured on this machine)",
        peak_mb
    );
    println!("Outputs written to: bench/out/");

    Ok(())
}
