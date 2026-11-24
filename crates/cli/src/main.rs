//! RustMatte CLI - Command line interface for offline background removal.

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
    download_model, extract_source_host, Config, ModelTier, Remover, StageTimings,
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

    /// Input image path (JPEG, PNG, WebP, BMP)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output transparent image path (PNG)
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

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

        /// Automatically confirm model download without interactive prompt
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
}

fn print_progress(downloaded: u64, total: u64) {
    let pct = if total > 0 {
        (downloaded as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let downloaded_mb = downloaded as f64 / (1024.0 * 1024.0);
    let total_mb = total as f64 / (1024.0 * 1024.0);
    eprint!("\rDownloading model: {downloaded_mb:.1} / {total_mb:.1} MB ({pct:.1}%)");
    let _ = std::io::stderr().flush();
}

fn ensure_model_available(
    spec: &ModelSpec,
    custom_path: Option<&Path>,
    yes: bool,
    no_download: bool,
) -> Result<PathBuf, String> {
    match check_model_status_with_custom(custom_path, spec.tier) {
        ModelStatus::Verified { path } => Ok(path),
        ModelStatus::Corrupt {
            path,
            expected,
            actual,
        } => Err(format!(
            "Model file at '{}' is corrupt. Expected SHA-256 '{expected}', found '{actual}'",
            path.display()
        )),
        ModelStatus::Missing {
            expected_cache_path,
        } => {
            if no_download {
                return Err(format!(
                    "Model '{}' is not present locally and network downloads are disabled by '--no-download'. Place '{}' in '{}' or pass '--model-path'.",
                    spec.id,
                    spec.default_filename,
                    expected_cache_path.display()
                ));
            }

            let size_mb = spec.expected_size as f64 / (1024.0 * 1024.0);
            let first_url = spec
                .source_urls
                .iter()
                .find(|u| !u.starts_with("TODO"))
                .copied()
                .unwrap_or("unknown");
            let host = extract_source_host(first_url);

            if !yes {
                if !std::io::stdin().is_terminal() {
                    return Err(format!(
                        "Model '{}' needs to be downloaded ({size_mb:.1} MB from {host}, license {}).\nRun with '--yes' to confirm download, or run 'rustmatte models download', or place '{}' in '{}'.",
                        spec.id,
                        spec.license,
                        spec.default_filename,
                        expected_cache_path.display()
                    ));
                }

                eprintln!("RustMatte needs to download model weights on first run:");
                eprintln!("  Model:   {}", spec.name);
                eprintln!("  File:    {}", spec.default_filename);
                eprintln!("  Size:    {size_mb:.1} MB");
                eprintln!("  License: {}", spec.license);
                eprintln!("  Source:  {host}");
                eprintln!("  Cache:   {}", expected_cache_path.display());
                eprint!("Proceed with download? [y/N]: ");
                let _ = std::io::stderr().flush();

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| format!("Failed to read user input: {e}"))?;
                let trimmed = input.trim();
                if !trimmed.eq_ignore_ascii_case("y") && !trimmed.eq_ignore_ascii_case("yes") {
                    return Err("Download cancelled by user. Model was not downloaded.".to_string());
                }
            }

            eprintln!("Starting model download...");
            let cache_dir = default_cache_dir();
            let downloaded_path =
                download_model(spec, &cache_dir, print_progress).map_err(|e| {
                    eprintln!();
                    format!("Model download failed: {e}")
                })?;
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

fn run_cli(cli: Cli) -> Result<(), String> {
    // 1. Handle subcommands if provided
    if let Some(Commands::Models(models_args)) = cli.command {
        return handle_models_command(models_args);
    }

    // 2. Positional input check
    let input = match cli.input {
        Some(p) => p,
        None => {
            return Err(
                "Missing required argument <INPUT>. Run 'rustmatte --help' for usage instructions."
                    .to_string(),
            );
        }
    };

    let output_path = match cli.output {
        Some(p) => p,
        None => {
            return Err("Missing required option '-o, --output <OUTPUT>'. Run 'rustmatte --help' for usage instructions.".to_string());
        }
    };

    // 3. Verify input existence
    if !input.exists() {
        return Err(format!("Input file '{}' does not exist", input.display()));
    }

    // 4. Guard against unwanted file overwrites
    if output_path.exists() && !cli.overwrite {
        return Err(format!(
            "Output file '{}' already exists. Pass '--overwrite' to allow replacing it.",
            output_path.display()
        ));
    }

    // 5. Ensure model is available (resolve / download with consent)
    let tier: ModelTier = cli.tier.into();
    let spec = spec_for_tier(tier);
    let resolved_model_path =
        ensure_model_available(spec, cli.model_path.as_deref(), cli.yes, cli.no_download)?;

    // 6. Configure core engine
    let thread_count = cli.threads.unwrap_or_else(default_thread_count);
    let config = Config {
        tier,
        model_path: Some(resolved_model_path),
        threads: thread_count,
    };

    let mut remover = Remover::new(config).map_err(|e| e.to_string())?;

    // 7. Process image
    let output = remover.process(&input).map_err(|e| e.to_string())?;

    // 8. Emit non-fatal warnings to stderr
    for warning in output.warnings {
        match warning {
            Warning::DamagedInput(msg) => {
                eprintln!("Warning: {msg}");
            }
        }
    }

    // 9. Atomically save output image
    let t_write = Instant::now();
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

    // Save to temp file
    output
        .image
        .save(&temp_path)
        .map_err(|e| format!("Failed to encode output PNG to temporary file: {e}"))?;

    // On Windows, rename fails if target exists even when overwriting, so remove target first
    if cli.overwrite && output_path.exists() {
        let _ = std::fs::remove_file(&output_path);
    }

    std::fs::rename(&temp_path, &output_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!(
            "Failed to finalize output file '{}': {e}",
            output_path.display()
        )
    })?;
    let write_ms = t_write.elapsed().as_secs_f64() * 1000.0;

    // 10. Timing breakdown if --verbose is active
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

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run_cli(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::from(1)
        }
    }
}
