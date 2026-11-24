//! ONNX Runtime session creation, thread configuration, and D-015 allocator enforcement.

use std::path::Path;

use ort::session::Session;

use crate::error::CoreError;

/// Calculates the default execution thread count.
///
/// Heuristic note: On a 6-core/12-thread CPU (i5-10400), 6 intra-op threads produced
/// optimal CPU throughput (~2,985 ms vs ~3,295 ms for 4 threads).
/// We set the default thread count to half of available logical CPUs, clamped to 2..=8.
#[must_use]
pub fn default_thread_count() -> usize {
    let logical = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    (logical / 2).clamp(2, 8)
}

/// Builds an ONNX Runtime session with D-015 memory allocator configuration.
pub fn create_session(
    env: &ort::environment::Environment,
    model_path: &Path,
    threads: usize,
) -> Result<Session, CoreError> {
    let mut builder = Session::builder(env)
        .map_err(|e| CoreError::InferenceSession(format!("Failed to create session builder: {e}")))?
        .with_intra_threads(threads)
        .map_err(|e| {
            CoreError::InferenceSession(format!("Failed to set thread count to {threads}: {e}"))
        })?;

    // D-015: Critical memory stability configuration for CPU inference.
    // Disables static memory pattern and enables OS environment system allocators.
    // Saves ~1.33 GB RAM and prevents BFCArena allocation crashes on unrolled deformable convolutions.
    builder = builder
        .with_memory_pattern(false)
        .map_err(|e| CoreError::InferenceSession(format!("Failed to disable memory pattern: {e}")))?
        .with_config_entry("session.use_env_allocators", "1")
        .map_err(|e| {
            CoreError::InferenceSession(format!("Failed to enable env allocators: {e}"))
        })?;

    let session = builder.commit_from_file(model_path).map_err(|e| {
        CoreError::InferenceSession(format!(
            "Failed to commit model from '{}': {e}",
            model_path.display()
        ))
    })?;

    Ok(session)
}
