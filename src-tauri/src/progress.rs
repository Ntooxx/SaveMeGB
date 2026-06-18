use crate::model::ScanProgress;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub type ProgressSink = Arc<dyn Fn(ScanProgress) + Send + Sync + 'static>;

pub fn sink_from_app(app: AppHandle) -> ProgressSink {
    Arc::new(move |p: ScanProgress| {
        if let Err(e) = app.emit("scan-progress", &p) {
            log::warn!("emit scan-progress failed: {e}");
        }
    })
}

pub fn silent_sink() -> ProgressSink {
    Arc::new(|_p: ScanProgress| {})
}

pub fn stdio_sink() -> ProgressSink {
    Arc::new(|p: ScanProgress| {
        eprintln!(
            "[scan:{}] {}/{} {}",
            p.stage, p.current, p.total, p.message
        );
    })
}

#[derive(Default)]
pub struct StageTimer {
    started: Mutex<Option<std::time::Instant>>,
}

impl StageTimer {
    pub fn start(&self) {
        *self.started.lock() = Some(std::time::Instant::now());
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started
            .lock()
            .map(|s| s.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }
}
