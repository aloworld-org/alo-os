//! Keep slow hardware queries out of the compositor's frame and input loop.

use alo_shell::{StatusItems, TheirFormats};
use std::{
    sync::mpsc::{Receiver, TryRecvError, sync_channel},
    time::{Duration, Instant},
};

/// Hardware does not need to be queried more than once a second.
const INTERVAL: Duration = Duration::from_secs(1);
/// An older request cannot establish the machine's current hardware state.
const STALE: Duration = Duration::from_secs(2);
/// One complete status reading, or why it could not be taken.
type Reading = Result<StatusItems, String>;

/// One outstanding reader; late and failed results cannot freeze or mislead UI.
pub(crate) struct StatusReadings {
    /// The single worker's bounded handoff, never waited on by the compositor.
    pending: Option<Receiver<Reading>>,
    /// When that worker started, also used to limit request frequency.
    started: Option<Instant>,
    /// The request boundary of the last usable hardware reading.
    fresh_at: Option<Instant>,
    /// Last reported failure, avoiding an identical message on every frame.
    problem: Option<String>,
}

impl StatusReadings {
    /// The startup hardware query began at this monotonic boundary.
    pub(crate) fn new(started: Instant) -> Self {
        Self::at(started)
    }

    /// Seed the initial reading's freshness at an explicit boundary.
    fn at(now: Instant) -> Self {
        Self {
            pending: None,
            started: None,
            fresh_at: Some(now),
            problem: None,
        }
    }

    /// Poll a bounded handoff; no device, subprocess or thread join runs here.
    pub(crate) fn refresh(&mut self, status: &mut StatusItems, formats: &TheirFormats) {
        self.refresh_with(Instant::now(), status, formats, |formats| {
            let at = crate::readings::At::now()?;
            crate::readings::taken(&at, &formats).map(|(status, _)| status)
        });
    }

    /// Inject only the reader in tests; production always uses the fixed query.
    fn refresh_with(
        &mut self,
        now: Instant,
        status: &mut StatusItems,
        formats: &TheirFormats,
        read: impl FnOnce(TheirFormats) -> Reading + Send + 'static,
    ) {
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    match result {
                        Ok(mut reading)
                            if self.started.is_some_and(|started| {
                                now.saturating_duration_since(started) < STALE
                            }) =>
                        {
                            // A result started under an old region must never
                            // replace the clock already reformatted by Settings.
                            reading.set_clock(status.clock().to_owned());
                            *status = reading;
                            self.fresh_at = self.started;
                            self.problem = None;
                        }
                        Ok(_) => self.failed(status, "the hardware reading timed out".into()),
                        Err(problem) => self.failed(status, problem),
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.failed(status, "the hardware-reading worker stopped".into());
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if self
            .fresh_at
            .is_some_and(|at| now.saturating_duration_since(at) >= STALE)
        {
            self.failed(status, "the hardware reading timed out".into());
        }
        if self.pending.is_some()
            || self
                .started
                .is_some_and(|at| now.saturating_duration_since(at) < INTERVAL)
        {
            return;
        }
        self.started = Some(now);
        let formats = formats.clone();
        let (sender, receiver) = sync_channel(1);
        match std::thread::Builder::new()
            .name("alo-status-readings".into())
            .spawn(move || {
                let _ = sender.send(read(formats));
            }) {
            Ok(_) => self.pending = Some(receiver),
            Err(problem) => self.failed(status, problem.to_string()),
        }
    }

    /// An unavailable reading is unknown, never a fabricated zero or silence.
    fn failed(&mut self, status: &mut StatusItems, problem: String) {
        if self.problem.as_ref() != Some(&problem) {
            eprintln!("alo-desktop: status readings unavailable: {problem}");
            self.problem = Some(problem);
        }
        if self.fresh_at.take().is_some() {
            *status = StatusItems::shown(
                status.clock().to_owned(),
                None,
                alo_networks::Reaching::reported(
                    alo_networks::HowFar::NotSaid,
                    alo_networks::Metered::NotSaid,
                ),
                None,
            );
        }
    }
}

#[cfg(test)]
#[path = "status_readings_tests.rs"]
mod tests;
