//! RustMatte Core - Offline, privacy-first AI background remover engine.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};
use ort::session::Session;
use ort::value::Tensor;

pub mod decode;
pub mod error;
pub mod model;
pub mod pipeline;
pub mod session;

pub use decode::Warning;
pub use error::CoreError;
pub use model::{spec_for_tier, ModelSpec, ModelTier};

/// Configuration options for initializing the background remover engine.
#[derive(Debug, Clone)]
pub struct Config {
    /// Model tier to run (Quality or Fast).
    pub tier: ModelTier,
    /// Path to the ONNX model file. If None, default `./models/<filename>` is used.
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

/// Output product of background removal processing.
#[derive(Debug, Clone)]
pub struct Output {
    /// 32-bit RGBA image with transparent background at original image dimensions.
    pub image: RgbaImage,
    /// 8-bit grayscale probability mask at original image dimensions.
    pub mask: GrayImage,
    /// Non-fatal warnings encountered during decoding and processing.
    pub warnings: Vec<Warning>,
}

/// Core background removal engine.
pub struct Remover {
    spec: &'static ModelSpec,
    session: Session,
    _env: Arc<ort::environment::Environment>,
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
    // Ignore error if another thread initialized it simultaneously
    let _ = GLOBAL_ENV.set(Arc::clone(&arc));
    Ok(GLOBAL_ENV.get().map(Arc::clone).unwrap_or(arc))
}

impl Remover {
    /// Initializes a new remover engine with the provided configuration.
    ///
    /// Validates model presence, verifies SHA-256 integrity, and configures an ORT session
    /// with the D-015 memory allocator settings.
    pub fn new(config: Config) -> Result<Self, CoreError> {
        let spec = spec_for_tier(config.tier);
        let model_path = model::resolve_model_path(config.model_path.as_deref(), config.tier)?;

        // Verify SHA-256 checksum against pinned reference
        model::verify_model_checksum(&model_path, spec.expected_sha256)?;

        let env = get_or_init_ort_env()?;
        let session = session::create_session(&env, &model_path, config.threads)?;

        Ok(Self {
            spec,
            session,
            _env: env,
        })
    }

    /// Processes an image from a file path, returning the transparent cutout and any warnings.
    pub fn process(&mut self, path: &Path) -> Result<Output, CoreError> {
        let (img, warnings) = decode::decode_image(path)?;
        self.process_image(&img, warnings)
    }

    /// Processes an in-memory dynamic image, applying preprocessing, inference, and mask blending.
    pub fn process_image(
        &mut self,
        img: &DynamicImage,
        warnings: Vec<Warning>,
    ) -> Result<Output, CoreError> {
        let (orig_w, orig_h) = img.dimensions();

        // 1. Preprocess normalized tensor
        let tensor = pipeline::preprocess_tensor(img, self.spec);
        let tensor_val = Tensor::from_array(tensor).map_err(|e| {
            CoreError::InferenceSession(format!("Failed to construct input tensor: {e}"))
        })?;

        // 2. Run inference
        let outputs = self
            .session
            .run(ort::inputs![self.spec.input_name => tensor_val])
            .map_err(|e| CoreError::InferenceSession(format!("Inference run failed: {e}")))?;

        let (_shape, raw_slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
            CoreError::TensorExtraction(format!("Failed to extract output tensor: {e}"))
        })?;

        // 3. Postprocess raw logits into probability mask
        let raw_mask = pipeline::postprocess_mask(raw_slice, self.spec, orig_w, orig_h)?;

        // 4. Refine mask (Phase 1C: identity passthrough; Phase 1D: guided filter)
        let mask = pipeline::refine_mask(raw_mask, img);

        // 5. Apply mask as alpha
        let image = pipeline::apply_mask(img, &mask);

        Ok(Output {
            image,
            mask,
            warnings,
        })
    }

    /// Returns the active model specification.
    #[must_use]
    pub fn spec(&self) -> &'static ModelSpec {
        self.spec
    }
}
