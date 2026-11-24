//! RustMatte CLI - Command line interface for offline background removal.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use rustmatte_core::decode::Warning;
use rustmatte_core::session::default_thread_count;
use rustmatte_core::{Config, ModelTier, Remover};

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
    /// Input image path (JPEG, PNG, WebP, BMP)
    input: PathBuf,

    /// Output transparent image path (PNG)
    #[arg(short = 'o', long = "output")]
    output: PathBuf,

    /// Model tier: quality (default) or fast (best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds)
    #[arg(short = 't', long = "tier", default_value = "quality")]
    tier: CliTier,

    /// Custom path to ONNX model weights file. If omitted, looks in ./models/
    #[arg(short = 'm', long = "model-path")]
    model_path: Option<PathBuf>,

    /// Number of intra-op execution threads (defaults to half of logical CPUs, clamped to 2..=8)
    #[arg(long = "threads")]
    threads: Option<usize>,

    /// Overwrite output file if it already exists
    #[arg(long = "overwrite")]
    overwrite: bool,
}

fn run_cli(cli: Cli) -> Result<(), String> {
    // 1. Verify input existence
    if !cli.input.exists() {
        return Err(format!(
            "Input file '{}' does not exist",
            cli.input.display()
        ));
    }

    // 2. Guard against unwanted file overwrites
    if cli.output.exists() && !cli.overwrite {
        return Err(format!(
            "Output file '{}' already exists. Pass '--overwrite' to allow replacing it.",
            cli.output.display()
        ));
    }

    // 3. Configure core engine
    let tier: ModelTier = cli.tier.into();
    let thread_count = cli.threads.unwrap_or_else(default_thread_count);

    let config = Config {
        tier,
        model_path: cli.model_path,
        threads: thread_count,
    };

    let mut remover = Remover::new(config).map_err(|e| e.to_string())?;

    // 4. Process image
    let output = remover.process(&cli.input).map_err(|e| e.to_string())?;

    // 5. Emit non-fatal warnings to stderr
    for warning in output.warnings {
        match warning {
            Warning::DamagedInput(msg) => {
                eprintln!("Warning: {msg}");
            }
        }
    }

    // 6. Atomically save output image
    let output_dir = cli
        .output
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(|e| {
            format!(
                "Failed to create output directory '{}': {e}",
                output_dir.display()
            )
        })?;
    }

    let file_stem = cli
        .output
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

    // On Windows, rename fails if target exists even when overwriting, so remove target first if overwrite was permitted
    if cli.overwrite && cli.output.exists() {
        let _ = std::fs::remove_file(&cli.output);
    }

    std::fs::rename(&temp_path, &cli.output).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!(
            "Failed to finalize output file '{}': {e}",
            cli.output.display()
        )
    })?;

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
