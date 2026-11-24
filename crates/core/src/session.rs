//! ONNX Runtime session creation, execution provider selection, and automatic CPU fallback.

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use ort::session::Session;

use crate::error::CoreError;

/// Target execution provider requested by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionProvider {
    /// Standard CPU inference (default).
    #[default]
    Cpu,
    /// DirectML execution provider for DirectX 12 hardware on Windows.
    DirectML,
    /// CoreML execution provider for Apple Silicon / Neural Engine on macOS.
    CoreML,
    /// Automatically selects available hardware acceleration with graceful CPU fallback.
    Auto,
}

impl FromStr for ExecutionProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "cpu" => Ok(Self::Cpu),
            "directml" | "dml" | "gpu" => Ok(Self::DirectML),
            "coreml" | "ane" => Ok(Self::CoreML),
            "auto" => Ok(Self::Auto),
            other => Err(format!(
                "Unknown execution provider '{other}'. Valid options: cpu, directml, coreml, auto"
            )),
        }
    }
}

impl fmt::Display for ExecutionProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cpu => write!(f, "cpu"),
            Self::DirectML => write!(f, "directml"),
            Self::CoreML => write!(f, "coreml"),
            Self::Auto => write!(f, "auto"),
        }
    }
}

/// Active execution provider successfully initialized and active for the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveExecutionProvider {
    Cpu,
    DirectML,
    CoreML,
}

impl fmt::Display for ActiveExecutionProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cpu => write!(f, "CPU"),
            Self::DirectML => write!(f, "DirectML"),
            Self::CoreML => write!(f, "CoreML"),
        }
    }
}

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

/// Builds an ONNX Runtime session with D-015 memory allocator configuration for pure CPU inference.
fn create_cpu_session(
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
            "Failed to commit CPU session from '{}': {e}",
            model_path.display()
        ))
    })?;

    Ok(session)
}

/// Builds an ONNX Runtime session targeting the requested execution provider with automatic CPU fallback.
pub fn create_session_with_provider(
    env: &ort::environment::Environment,
    model_path: &Path,
    threads: usize,
    provider: ExecutionProvider,
) -> Result<(Session, ActiveExecutionProvider), CoreError> {
    match provider {
        ExecutionProvider::Cpu => {
            let session = create_cpu_session(env, model_path, threads)?;
            Ok((session, ActiveExecutionProvider::Cpu))
        }
        ExecutionProvider::DirectML | ExecutionProvider::Auto => {
            #[cfg(all(target_os = "windows", feature = "directml"))]
            {
                use ort::ep::ExecutionProvider as _;
                if let Ok(true) = ort::ep::DirectML::is_available() {
                    match Session::builder(env) {
                        Ok(builder) => {
                            let ep = ort::ep::DirectML::default().build();
                            let configured_builder = builder
                                .with_execution_providers([ep])
                                .and_then(|b| b.with_intra_threads(threads))
                                .and_then(|b| b.with_memory_pattern(false));

                            match configured_builder {
                                Ok(mut b) => match b.commit_from_file(model_path) {
                                    Ok(session) => {
                                        return Ok((session, ActiveExecutionProvider::DirectML));
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: DirectML session commit failed ({e}); falling back to CPU."
                                        );
                                    }
                                },
                                Err(e) => {
                                    eprintln!(
                                        "Warning: DirectML configuration failed ({e}); falling back to CPU."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Session builder initialization failed ({e}); falling back to CPU."
                            );
                        }
                    }
                } else if provider == ExecutionProvider::DirectML {
                    eprintln!(
                        "Warning: DirectML is not available on this device; falling back to CPU."
                    );
                }
            }

            #[cfg(not(all(target_os = "windows", feature = "directml")))]
            if provider == ExecutionProvider::DirectML {
                eprintln!(
                    "Warning: DirectML feature is not compiled or unsupported on this OS; falling back to CPU."
                );
            }

            let session = create_cpu_session(env, model_path, threads)?;
            Ok((session, ActiveExecutionProvider::Cpu))
        }
        ExecutionProvider::CoreML => {
            #[cfg(all(target_os = "macos", feature = "coreml"))]
            {
                use ort::ep::ExecutionProvider as _;
                if let Ok(true) = ort::ep::CoreML::is_available() {
                    match Session::builder(env) {
                        Ok(builder) => {
                            let ep = ort::ep::CoreML::default().build();
                            match builder.with_execution_providers([ep]) {
                                Ok(mut builder_with_ep) => {
                                    match builder_with_ep.commit_from_file(model_path) {
                                        Ok(session) => {
                                            return Ok((session, ActiveExecutionProvider::CoreML));
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "Warning: CoreML session commit failed ({e}); falling back to CPU."
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Warning: CoreML execution provider registration failed ({e}); falling back to CPU."
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Session builder initialization failed ({e}); falling back to CPU."
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "Warning: CoreML is not available on this device; falling back to CPU."
                    );
                }
            }

            #[cfg(not(all(target_os = "macos", feature = "coreml")))]
            {
                eprintln!(
                    "Warning: CoreML feature is not compiled or unsupported on this OS; falling back to CPU."
                );
            }

            let session = create_cpu_session(env, model_path, threads)?;
            Ok((session, ActiveExecutionProvider::Cpu))
        }
    }
}

/// Legacy wrapper preserving backward compatibility.
pub fn create_session(
    env: &ort::environment::Environment,
    model_path: &Path,
    threads: usize,
) -> Result<Session, CoreError> {
    create_cpu_session(env, model_path, threads)
}
