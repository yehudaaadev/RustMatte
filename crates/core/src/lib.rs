//! RustMatte Core - Offline, privacy-first AI background remover engine.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};
use ort::session::Session;
use ort::value::Tensor;

pub mod decode;
pub mod download;
pub mod error;
pub mod manifest;
pub mod model;
pub mod pipeline;
pub mod recommend;
pub mod refine;
pub mod session;
pub mod sysinfo;

pub use decode::Warning;
pub use download::{
    delete_partial_download, download_model, download_model_with_options, extract_source_host,
    partial_download_info, DownloadOptions, PartialDownloadInfo,
};
pub use error::CoreError;
pub use manifest::{
    find_manifest_entry, manifest_entries, manifest_entry_for_tier, validate_manifest,
    ActivationKind, MeasuredSpec, ModelInputSpec, ModelManifestEntry, ModelSource, ModelStatusKind,
    ProfileTags, MANIFEST_BALANCED, MANIFEST_FAST,
};
pub use model::{
    check_model_status, check_model_status_with_custom, default_cache_dir, delete_cached_model,
    spec_for_tier, ModelSpec, ModelStatus, ModelTier,
};
pub use recommend::{
    recommend_model, recommend_model_with_probe, ModelRecommendation, PhotoSubject, RamAdvisory,
    RecommendationReason, UsageFrequency, LOW_RAM_THRESHOLD_MB, MID_RAM_THRESHOLD_MB,
};
pub use refine::{apply_mask_with_refinement, refine_mask_guided, RefineConfig};
pub use session::{
    create_session, create_session_with_provider, default_thread_count, ActiveExecutionProvider,
    ExecutionProvider,
};
pub use sysinfo::{
    check_disk_space, check_disk_space_with, detect_available_disk_bytes,
    detect_available_memory_mb, detect_memory_info, required_space_with_margin, DiskSpaceCheck,
    MemoryInfo,
};

/// Configuration options for initializing the background remover engine.
#[derive(Debug, Clone)]
pub struct Config {
    /// Model tier to run (Quality or Fast).
    pub tier: ModelTier,
    /// Path to the ONNX model file. If None, hierarchical lookup order is used.
    pub model_path: Option<PathBuf>,
    /// Number of intra-op execution threads.
    pub threads: usize,
    /// Edge refinement and color defringing settings.
    pub refine: RefineConfig,
    /// Target hardware execution provider.
    pub execution_provider: ExecutionProvider,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tier: ModelTier::Quality,
            model_path: None,
            threads: session::default_thread_count(),
            refine: RefineConfig::default(),
            execution_provider: ExecutionProvider::Cpu,
        }
    }
}

/// Elapsed time measurements for each pipeline stage (in milliseconds).
#[derive(Debug, Clone, Default)]
pub struct StageTimings {
    /// Time spent resolving model and initializing ONNX Runtime session.
    pub model_load_ms: f64,
    /// Time spent decoding image file and checking dimensions/safety.
    pub decode_ms: f64,
    /// Time spent resizing and normalizing pixel values to input tensor.
    pub preprocess_ms: f64,
    /// Time spent executing neural network inference on CPU.
    pub inference_ms: f64,
    /// Time spent converting logits to mask, applying refinement, and alpha blending.
    pub postprocess_ms: f64,
}

/// Output product of background removal processing.
#[derive(Debug, Clone)]
pub struct Output {
    /// 32-bit RGBA image with transparent background at original image dimensions.
    pub image: RgbaImage,
    /// 8-bit grayscale probability mask at original image dimensions.
    pub mask: GrayImage,
    /// Non-fatal warnings encountered during decoding and processing.
    pub warnings: Vec<Warning>,
    /// Execution timing metrics per stage.
    pub timings: StageTimings,
}

/// Core background removal engine.
pub struct Remover {
    spec: &'static ModelSpec,
    session: Option<Session>,
    _env: Arc<ort::environment::Environment>,
    model_load_ms: f64,
    refine: RefineConfig,
    active_ep: ActiveExecutionProvider,
    model_path: PathBuf,
    threads: usize,
}

static GLOBAL_ENV: std::sync::OnceLock<Arc<ort::environment::Environment>> =
    std::sync::OnceLock::new();

fn get_or_init_ort_env() -> Result<Arc<ort::environment::Environment>, CoreError> {
    if let Some(env) = GLOBAL_ENV.get() {
        return Ok(Arc::clone(env));
    }

    let env = ort::init()
        .with_name("rustmatte")
        .with_telemetry(false)
        .build()
        .map_err(|e| {
            CoreError::InferenceSession(format!("Failed to initialize ORT environment: {e}"))
        })?;

    let arc = Arc::new(env);
    let _ = GLOBAL_ENV.set(Arc::clone(&arc));
    Ok(GLOBAL_ENV.get().map(Arc::clone).unwrap_or(arc))
}

impl Remover {
    /// Initializes a new remover engine with the provided configuration.
    ///
    /// Validates model presence via hierarchical lookup, verifies SHA-256 integrity,
    /// and configures an ORT session with requested execution provider and fallback.
    pub fn new(config: Config) -> Result<Self, CoreError> {
        let t_start = Instant::now();
        let spec = spec_for_tier(config.tier);
        let model_path = model::resolve_model_path(config.model_path.as_deref(), config.tier)?;

        let env = get_or_init_ort_env()?;
        let (session, active_ep) = session::create_session_with_provider(
            &env,
            &model_path,
            config.threads,
            config.execution_provider,
        )?;
        let model_load_ms = t_start.elapsed().as_secs_f64() * 1000.0;

        Ok(Self {
            spec,
            session: Some(session),
            _env: env,
            model_load_ms,
            refine: config.refine,
            active_ep,
            model_path,
            threads: config.threads,
        })
    }

    /// Returns the active execution provider bound to the session.
    #[must_use]
    pub fn execution_provider(&self) -> ActiveExecutionProvider {
        self.active_ep
    }

    /// Processes an image from a file path with optional early downscaling, returning the transparent cutout, warnings, and timings.
    pub fn process_with_options(
        &mut self,
        path: &Path,
        max_side: Option<u32>,
    ) -> Result<Output, CoreError> {
        let t_decode = Instant::now();
        let (mut img, warnings) = decode::decode_image(path)?;
        if let Some(max) = max_side {
            img = pipeline::downscale_if_needed(img, Some(max));
        }
        let decode_ms = t_decode.elapsed().as_secs_f64() * 1000.0;

        let mut output = self.process_image(&img, warnings)?;
        output.timings.decode_ms = decode_ms;
        Ok(output)
    }

    /// Processes an image from a file path, returning the transparent cutout, warnings, and timings.
    pub fn process(&mut self, path: &Path) -> Result<Output, CoreError> {
        self.process_with_options(path, None)
    }

    /// Processes an in-memory dynamic image, applying preprocessing, inference, and mask blending.
    pub fn process_image(
        &mut self,
        img: &DynamicImage,
        warnings: Vec<Warning>,
    ) -> Result<Output, CoreError> {
        let (orig_w, orig_h) = img.dimensions();

        // 1. Preprocess normalized tensor
        let t_pre = Instant::now();
        let tensor = pipeline::preprocess_tensor(img, self.spec);
        let tensor_val = Tensor::from_array(tensor).map_err(|e| {
            CoreError::InferenceSession(format!("Failed to construct input tensor: {e}"))
        })?;
        let preprocess_ms = t_pre.elapsed().as_secs_f64() * 1000.0;

        // 2. Run inference with automatic fallback to CPU if hardware execution fails
        let (raw_mask, inference_ms) = if self.active_ep != ActiveExecutionProvider::Cpu {
            let t_inf = Instant::now();
            let mut hardware_err = None;
            let mut result_mask = None;

            {
                let sess = self.session.as_mut().expect("Session must be active");
                match sess.run(ort::inputs![self.spec.input_name => tensor_val]) {
                    Ok(outputs) => {
                        let (_shape, raw_slice) =
                            outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                                CoreError::TensorExtraction(format!(
                                    "Failed to extract output tensor: {e}"
                                ))
                            })?;
                        result_mask = Some(pipeline::postprocess_mask(
                            raw_slice, self.spec, orig_w, orig_h,
                        )?);
                    }
                    Err(e) => {
                        hardware_err = Some(e);
                    }
                }
            }

            if let Some(e) = hardware_err {
                eprintln!(
                    "Warning: Inference execution failed on {} ({e}); falling back to CPU session...",
                    self.active_ep
                );
                // Drop the old hardware session first to free all VRAM/RAM buffers!
                self.session = None;

                let cpu_session =
                    session::create_session(&self._env, &self.model_path, self.threads)?;
                self.session = Some(cpu_session);
                self.active_ep = ActiveExecutionProvider::Cpu;

                let retry_tensor = pipeline::preprocess_tensor(img, self.spec);
                let retry_tensor_val = Tensor::from_array(retry_tensor).map_err(|e| {
                    CoreError::InferenceSession(format!("Failed to reconstruct input tensor: {e}"))
                })?;

                let t_retry = Instant::now();
                let sess = self.session.as_mut().unwrap();
                let fallback_outputs = sess
                    .run(ort::inputs![self.spec.input_name => retry_tensor_val])
                    .map_err(|e| {
                        CoreError::InferenceSession(format!("CPU fallback inference failed: {e}"))
                    })?;

                let (_shape, raw_slice) =
                    fallback_outputs[0]
                        .try_extract_tensor::<f32>()
                        .map_err(|e| {
                            CoreError::TensorExtraction(format!(
                                "Failed to extract output tensor: {e}"
                            ))
                        })?;
                let mask = pipeline::postprocess_mask(raw_slice, self.spec, orig_w, orig_h)?;
                (mask, t_retry.elapsed().as_secs_f64() * 1000.0)
            } else {
                (
                    result_mask.expect("Mask must be present on success"),
                    t_inf.elapsed().as_secs_f64() * 1000.0,
                )
            }
        } else {
            let t_inf = Instant::now();
            let sess = self.session.as_mut().expect("Session must be active");
            let outputs = sess
                .run(ort::inputs![self.spec.input_name => tensor_val])
                .map_err(|e| CoreError::InferenceSession(format!("Inference run failed: {e}")))?;

            let (_shape, raw_slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                CoreError::TensorExtraction(format!("Failed to extract output tensor: {e}"))
            })?;
            let mask = pipeline::postprocess_mask(raw_slice, self.spec, orig_w, orig_h)?;
            (mask, t_inf.elapsed().as_secs_f64() * 1000.0)
        };

        // 3. Refine mask (Phase 1D: Guided filter & selective transition feathering)
        let t_post = Instant::now();
        let mask = pipeline::refine_mask(raw_mask, img, &self.refine);

        // 4. Apply mask as alpha with defringing
        let image = pipeline::apply_mask_with_options(img, &mask, self.refine.enable_defringing);
        let postprocess_ms = t_post.elapsed().as_secs_f64() * 1000.0;

        let timings = StageTimings {
            model_load_ms: self.model_load_ms,
            decode_ms: 0.0,
            preprocess_ms,
            inference_ms,
            postprocess_ms,
        };

        Ok(Output {
            image,
            mask,
            warnings,
            timings,
        })
    }

    /// Returns the active model specification.
    #[must_use]
    pub fn spec(&self) -> &'static ModelSpec {
        self.spec
    }

    /// Returns the recorded model load duration in milliseconds.
    #[must_use]
    pub fn model_load_ms(&self) -> f64 {
        self.model_load_ms
    }
}
