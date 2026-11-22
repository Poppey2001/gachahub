//! Progress events emitted by long-running core tasks.
//!
//! The old Tauri build pushed these to the web view with `AppHandle::emit`.
//! The core is now UI-agnostic: callers hand in an [`EventSink`] closure and
//! decide themselves how to forward events (channel, repaint request, log, ...).

use std::sync::Arc;

use crate::{install::InstallJob, runners::RunnerProgress, xxmi::XxmiProgress};

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Download(InstallJob),
    Xxmi(XxmiProgress),
    Runner(RunnerProgress),
}

pub type EventSink = Arc<dyn Fn(CoreEvent) + Send + Sync>;
