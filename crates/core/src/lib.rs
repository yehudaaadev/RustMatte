//! RustMatte Core - Offline, privacy-first AI background remover engine.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};
use ort::session::Session;
use ort::value::Tensor;

pub mod decode;
pub mod download;
pub mod error;
pub mod model;
pub mod pipeline;
pub mod session;

pub use decode::Warning;
pub use download::{download_model, extract_source_host};
pub use error::CoreError;
pub use model::{
    check_model_status, check_model_status_with_custom, default_cache_dir, spec_for_tier,
    ModelSpec, ModelStatus, ModelTier,
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tier: ModelTier::Quality,
            model_path: None,
            threads: session::default_thread_count(),
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
    session: Session,
    _env: Arc<ort::environment::Environment>,
    model_load_ms: f64,
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
    /// and configures an ORT session with D-015 memory allocator settings.
    pub fn new(config: Config) -> Result<Self, CoreError> {
        let t_start = Instant::now();
        let spec = spec_for_tier(config.tier);
        let model_path = model::resolve_model_path(config.model_path.as_deref(), config.tier)?;

        let env = get_or_init_ort_env()?;
        let session = session::create_session(&env, &model_path, config.threads)?;
        let model_load_ms = t_start.elapsed().as_secs_f64() * 1000.0;

        Ok(Self {
            spec,
            session,
            _env: env,
            model_load_ms,
        })
    }

    /// Processes an image from a file path, returning the transparent cutout, warnings, and timings.
    pub fn process(&mut self, path: &Path) -> Result<Output, CoreError> {
        let t_decode = Instant::now();
        let (img, warnings) = decode::decode_image(path)?;
        let decode_ms = t_decode.elapsed().as_secs_f64() * 1000.0;

        let mut output = self.process_image(&img, warnings)?;
        output.timings.decode_ms = decode_ms;
        Ok(output)
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

        // 2. Run inference
        let t_inf = Instant::now();
        let outputs = self
            .session
            .run(ort::inputs![self.spec.input_name => tensor_val])
            .map_err(|e| CoreError::InferenceSession(format!("Inference run failed: {e}")))?;

        let (_shape, raw_slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
            CoreError::TensorExtraction(format!("Failed to extract output tensor: {e}"))
        })?;
        let inference_ms = t_inf.elapsed().as_secs_f64() * 1000.0;

        // 3. Postprocess raw logits into probability mask
        let t_post = Instant::now();
        let raw_mask = pipeline::postprocess_mask(raw_slice, self.spec, orig_w, orig_h)?;

        // 4. Refine mask (Phase 1C: identity passthrough; Phase 1D: guided filter)
        let mask = pipeline::refine_mask(raw_mask, img);

        // 5. Apply mask as alpha
        let image = pipeline::apply_mask(img, &mask);
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
