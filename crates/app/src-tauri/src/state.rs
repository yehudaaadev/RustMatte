//! Thread-safe job coordinator and per-tier cached inference session manager.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rustmatte_core::{Config, ExecutionProvider, ModelTier, RefineConfig, Remover};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineLevel {
    None,
    Light,
    Standard,
}

impl RefineLevel {
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "none" | "off" => Self::None,
            "light" => Self::Light,
            _ => Self::Standard,
        }
    }

    #[must_use]
    pub fn to_refine_config(self) -> RefineConfig {
        match self {
            Self::None => RefineConfig {
                enable_guided_filter: false,
                enable_defringing: false,
                ..Default::default()
            },
            Self::Light => RefineConfig {
                enable_guided_filter: true,
                enable_defringing: false,
                ..Default::default()
            },
            Self::Standard => RefineConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionOptions {
    pub execution_provider: ExecutionProvider,
    pub refine_level: RefineLevel,
    pub threads: usize,
}

/// Holds at most one loaded ORT `Remover` session per `ModelTier`, reloading only on option change.
pub struct SessionCache {
    quality: Option<(SessionOptions, Remover)>,
    fast: Option<(SessionOptions, Remover)>,
}

impl SessionCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            quality: None,
            fast: None,
        }
    }

    fn slot_mut(&mut self, tier: ModelTier) -> &mut Option<(SessionOptions, Remover)> {
        match tier {
            ModelTier::Quality => &mut self.quality,
            ModelTier::Fast => &mut self.fast,
        }
    }

    /// Returns whether a cached session matching `tier` and `options` already exists.
    #[must_use]
    pub fn is_loaded(&self, tier: ModelTier, options: &SessionOptions) -> bool {
        let slot = match tier {
            ModelTier::Quality => &self.quality,
            ModelTier::Fast => &self.fast,
        };
        matches!(slot.as_ref(), Some((cached_opts, _)) if cached_opts == options)
    }

    /// Returns a mutable reference to the cached `Remover` for `tier`, loading or reloading if options changed.
    pub fn get_or_load(
        &mut self,
        tier: ModelTier,
        options: SessionOptions,
    ) -> Result<&mut Remover, String> {
        let needs_reload = !self.is_loaded(tier, &options);

        if needs_reload {
            // Drop old session for this tier first so memory is released before reallocating.
            *self.slot_mut(tier) = None;

            let config = Config {
                tier,
                model_path: None,
                threads: options.threads,
                refine: options.refine_level.to_refine_config(),
                execution_provider: options.execution_provider,
            };

            let remover = Remover::new(config).map_err(|e| e.to_string())?;
            *self.slot_mut(tier) = Some((options, remover));
        }

        self.slot_mut(tier)
            .as_mut()
            .map(|(_, r)| r)
            .ok_or_else(|| "Session cache lookup failed".to_string())
    }
}

impl Default for SessionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct ActiveJob {
    id: u64,
    cancel_flag: Arc<AtomicBool>,
}

/// Coordinates single active processing job and cancellation requests.
#[derive(Debug, Default)]
pub struct JobCoordinator {
    next_id: u64,
    active: Option<ActiveJob>,
}

impl JobCoordinator {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_start(&mut self) -> Result<(u64, Arc<AtomicBool>), String> {
        if self.active.is_some() {
            return Err("A background removal job is already running".to_string());
        }
        self.next_id += 1;
        let id = self.next_id;
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.active = Some(ActiveJob {
            id,
            cancel_flag: Arc::clone(&cancel_flag),
        });
        Ok((id, cancel_flag))
    }

    pub fn request_cancel(&self) -> bool {
        if let Some(ref active) = self.active {
            active.cancel_flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn finish(&mut self, job_id: u64) {
        if let Some(ref active) = self.active {
            if active.id == job_id {
                self.active = None;
            }
        }
    }

    #[cfg(test)]
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.active.is_some()
    }
}

/// Coordinates single active model download and cancellation requests.
#[derive(Debug, Default)]
pub struct DownloadCoordinator {
    active: Option<(String, Arc<AtomicBool>)>,
}

impl DownloadCoordinator {
    pub fn try_start(&mut self, tier: &str) -> Result<Arc<AtomicBool>, String> {
        if self.active.is_some() {
            return Err("A model download is already in progress".to_string());
        }
        let flag = Arc::new(AtomicBool::new(false));
        self.active = Some((tier.to_string(), Arc::clone(&flag)));
        Ok(flag)
    }

    pub fn request_cancel(&self) -> bool {
        if let Some((_, ref flag)) = self.active {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn finish(&mut self, tier: &str) {
        if let Some((ref active_tier, _)) = self.active {
            if active_tier == tier {
                self.active = None;
            }
        }
    }
}

pub struct AppState {
    pub session_cache: Mutex<SessionCache>,
    pub job_coordinator: Mutex<JobCoordinator>,
    pub download_coordinator: Mutex<DownloadCoordinator>,
}

impl AppState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            session_cache: Mutex::new(SessionCache::new()),
            job_coordinator: Mutex::new(JobCoordinator::new()),
            download_coordinator: Mutex::new(DownloadCoordinator::default()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refine_levels_map_accurately() {
        assert_eq!(RefineLevel::parse("none"), RefineLevel::None);
        assert_eq!(RefineLevel::parse("off"), RefineLevel::None);
        assert_eq!(RefineLevel::parse("light"), RefineLevel::Light);
        assert_eq!(RefineLevel::parse("standard"), RefineLevel::Standard);

        let light = RefineLevel::Light.to_refine_config();
        assert!(light.enable_guided_filter);
        assert!(!light.enable_defringing);

        let standard = RefineLevel::Standard.to_refine_config();
        assert!(standard.enable_guided_filter);
        assert!(standard.enable_defringing);
    }

    #[test]
    fn job_coordinator_prevents_concurrent_jobs_and_tracks_cancel() {
        let mut coord = JobCoordinator::new();
        assert!(!coord.is_running());
        assert!(!coord.request_cancel());

        let (job1, flag1) = coord.try_start().expect("start first job");
        assert!(coord.is_running());
        assert!(!flag1.load(Ordering::SeqCst));
        assert!(coord.try_start().is_err());

        assert!(coord.request_cancel());
        assert!(flag1.load(Ordering::SeqCst));

        coord.finish(job1 + 999);
        assert!(coord.is_running());

        coord.finish(job1);
        assert!(!coord.is_running());

        let (job2, flag2) = coord.try_start().expect("start second job");
        assert_ne!(job1, job2);
        assert!(!flag2.load(Ordering::SeqCst));
    }
}
