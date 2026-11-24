//! RustMatte CLI - Command line interface for offline background removal.

mod batch;

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use clap::{Parser, Subcommand, ValueEnum};
use rustmatte_core::decode::Warning;
use rustmatte_core::model::{
    check_model_status_with_custom, default_cache_dir, spec_for_tier, ModelSpec, ModelStatus,
};
use rustmatte_core::session::default_thread_count;
use rustmatte_core::{
    download_model, extract_source_host, Config, ModelTier, Remover, RgbaImage, StageTimings,
};

/// Model processing tier for CLI selection.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliTier {
    /// High-quality tier using BiRefNet Lite 512 (default).
    Quality,
    /// Fast preview tier using U-2-Netp: best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds.
    Fast,
}

impl From<CliTier> for ModelTier {
    fn from(tier: CliTier) -> Self {
        match tier {
            CliTier::Quality => ModelTier::Quality,
            CliTier::Fast => ModelTier::Fast,
        }
    }
}

/// Command-line argument specifications for RustMatte.
#[derive(Parser, Debug)]
#[command(
    name = "rustmatte",
    about = "Offline, privacy-first AI background remover with a Rust core",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input image file or directory containing images (JPEG, PNG, WebP, BMP)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output destination: transparent PNG path for file input, or directory for batch input (alias: --out)
    #[arg(short = 'o', long = "output", alias = "out")]
    output: Option<PathBuf>,

    /// Recursively scan subdirectories for images when processing a directory (default: off)
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,

    /// Skip processing if the target output file already exists (batch mode)
    #[arg(long = "skip-existing")]
    skip_existing: bool,

    /// Downscale image so that its longest side is at most N pixels, preserving aspect ratio (default: off)
    #[arg(long = "max-side")]
    max_side: Option<u32>,

    /// Model tier: quality (default) or fast (best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds)
    #[arg(short = 't', long = "tier", default_value = "quality")]
    tier: CliTier,

    /// Custom path to ONNX model weights file. If omitted, uses standard lookup order
    #[arg(short = 'm', long = "model-path")]
    model_path: Option<PathBuf>,

    /// Number of intra-op execution threads (defaults to half of logical CPUs, clamped to 2..=8)
    #[arg(long = "threads")]
    threads: Option<usize>,

    /// Overwrite output file if it already exists
    #[arg(long = "overwrite")]
    overwrite: bool,

    /// Automatically confirm model download without interactive prompt
    #[arg(short = 'y', long = "yes")]
    yes: bool,

    /// Disable network access completely (refuses to download missing models)
    #[arg(long = "no-download")]
    no_download: bool,

    /// Print detailed timing breakdown for each pipeline stage to stderr
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage offline AI model weights (list or pre-download)
    Models(ModelsArgs),
}

#[derive(Parser, Debug)]
struct ModelsArgs {
    #[command(subcommand)]
    action: ModelsAction,
}

#[derive(Subcommand, Debug)]
enum ModelsAction {
    /// List all model tiers, local cache status, sizes, licenses, and checksum verification
    List,
    /// Download model weights to the local OS cache directory
    Download {
        /// Model tier to download: quality (default) or fast
        #[arg(short = 't', long = "tier", default_value = "quality")]
        tier: CliTier,

        /// Automatically confirm download without interactive prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

fn prompt_user_consent(spec: &ModelSpec) -> Result<bool, String> {
    let size_mb = spec.expected_size as f64 / (1024.0 * 1024.0);
    let host = spec
        .source_urls
        .first()
        .map(|u| extract_source_host(u))
        .unwrap_or_else(|| "unknown host".to_string());

    eprintln!("\nRustMatte Offline AI Model Downloader");
    eprintln!("--------------------------------------");
    eprintln!("File:    {}", spec.default_filename);
    eprintln!("Size:    {size_mb:.1} MB");
    eprintln!("Source:  {host}");
    eprintln!("License: {}", spec.license);
    eprintln!();
    eprintln!("This is a one-time network download to obtain the model weights.");
    eprintln!("Once downloaded, all background removal operations run completely offline.");
    eprintln!("No user data, images, or telemetry will ever be transmitted.");
    eprint!("\nDownload now? [y/N]: ");
    std::io::stderr()
        .flush()
        .map_err(|e| format!("IO error: {e}"))?;

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read user input: {e}"))?;

    let trimmed = input.trim().to_ascii_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

fn ensure_model_available(
    spec: &'static ModelSpec,
    custom_path: Option<&Path>,
    auto_confirm: bool,
    no_download: bool,
) -> Result<PathBuf, String> {
    let status = check_model_status_with_custom(custom_path, spec.tier);

    match status {
        ModelStatus::Verified { path } => Ok(path),
        ModelStatus::Corrupt {
            path,
            expected,
            actual,
        } => Err(format!(
            "Model file '{}' is corrupt (checksum mismatch).\nExpected: {expected}\nActual:   {actual}\nPlease remove it or run 'rustmatte models download' to re-fetch a valid copy.",
            path.display()
        )),
        ModelStatus::Missing {
            expected_cache_path: _,
        } => {
            if no_download {
                return Err(format!(
                    "Model file '{}' not found and '--no-download' was specified.\nDownload it manually or run 'rustmatte models download --tier {}' without '--no-download'.",
                    spec.default_filename,
                    match spec.tier {
                        ModelTier::Quality => "quality",
                        ModelTier::Fast => "fast",
                    }
                ));
            }

            if !auto_confirm {
                if !std::io::stdin().is_terminal() {
                    return Err(format!(
                        "Model file '{}' is not present in local storage.\nInteractive confirmation is required. Pass '--yes' / '-y' to allow automatic download, or run 'rustmatte models download'.",
                        spec.default_filename
                    ));
                }

                let confirmed = prompt_user_consent(spec)?;
                if !confirmed {
                    return Err("Model download declined by user. Aborting.".to_string());
                }
            }

            let cache_dir = default_cache_dir();
            eprintln!("Starting model download...");

            let mut last_reported_pct: i32 = -1;
            let downloaded_path = download_model(spec, &cache_dir, |downloaded, total| {
                let pct = if total > 0 {
                    (downloaded as f64 / total as f64 * 100.0) as i32
                } else {
                    0
                };

                if pct != last_reported_pct {
                    last_reported_pct = pct;
                    let down_mb = downloaded as f64 / (1024.0 * 1024.0);
                    let total_mb = total as f64 / (1024.0 * 1024.0);
                    eprint!(
                        "\rDownloading model: {down_mb:.1} / {total_mb:.1} MB ({pct:.1}%)",
                        pct = pct as f64
                    );
                    let _ = std::io::stderr().flush();
                }
            })
            .map_err(|e| format!("Download failed: {e}"))?;

            eprintln!(
                "\nModel verified and cached at: {}",
                downloaded_path.display()
            );
            Ok(downloaded_path)
        }
    }
}

fn handle_models_command(args: ModelsArgs) -> Result<(), String> {
    match args.action {
        ModelsAction::List => {
            let tiers = [ModelTier::Quality, ModelTier::Fast];
            println!(
                "{:<8} {:<24} {:<10} {:<12} Status",
                "Tier", "File Name", "Size", "License"
            );
            println!("{}", "-".repeat(80));

            for tier in tiers {
                let spec = spec_for_tier(tier);
                let size_mb = spec.expected_size as f64 / (1024.0 * 1024.0);
                let status_str = match check_model_status_with_custom(None, tier) {
                    ModelStatus::Verified { path } => {
                        format!("Cached (Verified: {})", path.display())
                    }
                    ModelStatus::Corrupt { path, .. } => {
                        format!("Corrupt checksum ({})", path.display())
                    }
                    ModelStatus::Missing {
                        expected_cache_path,
                    } => {
                        format!("Not cached (expected: {})", expected_cache_path.display())
                    }
                };

                let tier_label = match tier {
                    ModelTier::Quality => "Quality",
                    ModelTier::Fast => "Fast",
                };

                println!(
                    "{:<8} {:<24} {:<10} {:<12} {}",
                    tier_label,
                    spec.default_filename,
                    format!("{size_mb:.1} MB"),
                    spec.license,
                    status_str
                );
            }
            Ok(())
        }
        ModelsAction::Download { tier, yes } => {
            let model_tier: ModelTier = tier.into();
            let spec = spec_for_tier(model_tier);
            ensure_model_available(spec, None, yes, false)?;
            Ok(())
        }
    }
}

fn is_same_file(path_a: &Path, path_b: &Path) -> bool {
    if path_a == path_b {
        return true;
    }
    match (path_a.canonicalize(), path_b.canonicalize()) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => false,
    }
}

fn save_image_atomic(image: &RgbaImage, output_path: &Path, overwrite: bool) -> Result<(), String> {
    let output_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| {
            format!(
                "Failed to create output directory '{}': {e}",
                output_dir.display()
            )
        })?;
    }

    let file_stem = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let pid = std::process::id();
    let temp_path = output_dir.join(format!(".{file_stem}.tmp.{pid}.png"));

    image
        .save(&temp_path)
        .map_err(|e| format!("Failed to encode output PNG to temporary file: {e}"))?;

    if overwrite && output_path.exists() {
        let _ = std::fs::remove_file(output_path);
    }

    std::fs::rename(&temp_path, output_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!(
            "Failed to finalize output file '{}': {e}",
            output_path.display()
        )
    })?;

    Ok(())
}

fn run_single(cli: &Cli, input_path: &Path, output_path: &Path) -> Result<(), String> {
    if is_same_file(input_path, output_path) {
        return Err(format!(
            "Output path '{}' cannot be identical to input file.",
            output_path.display()
        ));
    }

    if output_path.exists() && !cli.overwrite {
        return Err(format!(
            "Output file '{}' already exists. Pass '--overwrite' to allow replacing it.",
            output_path.display()
        ));
    }

    let tier: ModelTier = cli.tier.into();
    let spec = spec_for_tier(tier);
    let resolved_model_path =
        ensure_model_available(spec, cli.model_path.as_deref(), cli.yes, cli.no_download)?;

    let thread_count = cli.threads.unwrap_or_else(default_thread_count);
    let config = Config {
        tier,
        model_path: Some(resolved_model_path),
        threads: thread_count,
    };

    let mut remover = Remover::new(config).map_err(|e| e.to_string())?;

    let output = remover
        .process_with_options(input_path, cli.max_side)
        .map_err(|e| e.to_string())?;

    for warning in &output.warnings {
        match warning {
            Warning::DamagedInput(msg) => {
                eprintln!("Warning: {msg}");
            }
        }
    }

    let t_write = Instant::now();
    save_image_atomic(&output.image, output_path, cli.overwrite)?;
    let write_ms = t_write.elapsed().as_secs_f64() * 1000.0;

    if cli.verbose {
        let StageTimings {
            model_load_ms,
            decode_ms,
            preprocess_ms,
            inference_ms,
            postprocess_ms,
        } = output.timings;

        let total_ms =
            model_load_ms + decode_ms + preprocess_ms + inference_ms + postprocess_ms + write_ms;

        eprintln!("\nExecution Timing Breakdown:");
        eprintln!("  [timing] Model load:  {:>8.2} ms", model_load_ms);
        eprintln!("  [timing] Decode:      {:>8.2} ms", decode_ms);
        eprintln!("  [timing] Preprocess:  {:>8.2} ms", preprocess_ms);
        eprintln!("  [timing] Inference:   {:>8.2} ms", inference_ms);
        eprintln!("  [timing] Postprocess: {:>8.2} ms", postprocess_ms);
        eprintln!("  [timing] Write:       {:>8.2} ms", write_ms);
        eprintln!("  --------------------------------");
        eprintln!("  [timing] Total:       {:>8.2} ms", total_ms);
    }

    Ok(())
}

fn run_batch(cli: &Cli, input_dir: &Path, output_dir: &Path) -> Result<ExitCode, String> {
    let (jobs, discovery_warnings) =
        batch::discover_batch_jobs(input_dir, output_dir, cli.recursive)?;

    for warning in discovery_warnings {
        eprintln!("Warning: {warning}");
    }

    let total = jobs.len();
    if total == 0 {
        return Err(format!(
            "No supported image files found in '{}'",
            input_dir.display()
        ));
    }

    let tier: ModelTier = cli.tier.into();
    let spec = spec_for_tier(tier);
    let resolved_model_path =
        ensure_model_available(spec, cli.model_path.as_deref(), cli.yes, cli.no_download)?;

    let thread_count = cli.threads.unwrap_or_else(default_thread_count);
    let config = Config {
        tier,
        model_path: Some(resolved_model_path),
        threads: thread_count,
    };

    let mut remover = Remover::new(config).map_err(|e| e.to_string())?;

    let mut processed: usize = 0;
    let mut skipped: usize = 0;
    let mut failed: usize = 0;
    let mut total_warnings: usize = 0;
    let mut total_image_duration = std::time::Duration::ZERO;
    let batch_start = Instant::now();

    for (idx, job) in jobs.iter().enumerate() {
        let step = idx + 1;

        if is_same_file(&job.input_path, &job.output_path) {
            eprintln!(
                "[{step}/{total}] {}: failed (refusing to write output on top of input file)",
                job.display_name
            );
            failed += 1;
            continue;
        }

        if job.output_path.exists() {
            if cli.skip_existing {
                eprintln!(
                    "[{step}/{total}] {}: skipped (already exists)",
                    job.display_name
                );
                skipped += 1;
                continue;
            }
            if !cli.overwrite {
                eprintln!(
                    "[{step}/{total}] {}: failed (destination exists, use --overwrite or --skip-existing)",
                    job.display_name
                );
                failed += 1;
                continue;
            }
        }

        let t_img = Instant::now();
        match remover.process_with_options(&job.input_path, cli.max_side) {
            Ok(output) => {
                if let Err(e) = save_image_atomic(&output.image, &job.output_path, cli.overwrite) {
                    eprintln!(
                        "[{step}/{total}] {}: failed (save error: {e})",
                        job.display_name
                    );
                    failed += 1;
                    continue;
                }

                let elapsed = t_img.elapsed();
                let elapsed_ms = elapsed.as_millis();
                total_image_duration += elapsed;
                processed += 1;

                let warn_count = output.warnings.len();
                total_warnings += warn_count;

                if warn_count > 0 {
                    let warn_msgs: Vec<String> = output
                        .warnings
                        .into_iter()
                        .map(|w| match w {
                            Warning::DamagedInput(m) => m,
                        })
                        .collect();
                    eprintln!(
                        "[{step}/{total}] {}: ok with warning ({}) ({}ms)",
                        job.display_name,
                        warn_msgs.join("; "),
                        elapsed_ms
                    );
                } else {
                    eprintln!(
                        "[{step}/{total}] {}: ok ({}ms)",
                        job.display_name, elapsed_ms
                    );
                }
            }
            Err(e) => {
                eprintln!("[{step}/{total}] {}: failed ({e})", job.display_name);
                failed += 1;
            }
        }
    }

    let total_batch_secs = batch_start.elapsed().as_secs_f64();
    let avg_ms = if processed > 0 {
        (total_image_duration.as_secs_f64() * 1000.0) / (processed as f64)
    } else {
        0.0
    };

    eprintln!("\nBatch Summary:");
    eprintln!("  Processed:    {processed}");
    eprintln!("  Skipped:      {skipped}");
    eprintln!("  Failed:       {failed}");
    eprintln!("  Warnings:     {total_warnings}");
    eprintln!("  Total time:   {total_batch_secs:.2}s");
    eprintln!("  Average time: {avg_ms:.1}ms / image (excluding model load)");

    if failed > 0 && (processed > 0 || skipped > 0) {
        Ok(ExitCode::from(2))
    } else if failed > 0 && processed == 0 && skipped == 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn run_app(cli: Cli) -> Result<ExitCode, String> {
    if let Some(Commands::Models(models_args)) = cli.command {
        handle_models_command(models_args)?;
        return Ok(ExitCode::SUCCESS);
    }

    let input = match &cli.input {
        Some(p) => p,
        None => {
            return Err(
                "Missing required argument <INPUT>. Run 'rustmatte --help' for usage instructions."
                    .to_string(),
            );
        }
    };

    if !input.exists() {
        return Err(format!("Input path '{}' does not exist", input.display()));
    }

    if input.is_dir() {
        let output_dir = cli
            .output
            .clone()
            .unwrap_or_else(|| batch::default_batch_output_dir(input));
        run_batch(&cli, input, &output_dir)
    } else {
        let output_path = match &cli.output {
            Some(p) => p.clone(),
            None => {
                return Err(
                    "Missing required option '-o, --output <OUTPUT>' for single file input. Run 'rustmatte --help' for usage instructions."
                        .to_string(),
                );
            }
        };
        run_single(&cli, input, &output_path)?;
        Ok(ExitCode::SUCCESS)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run_app(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::from(1)
        }
    }
}
