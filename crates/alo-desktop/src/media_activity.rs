//! Live media activity, read off the compositor thread and never guessed quiet.

use alo_in_use::{InUse, Line, NotHeard, TheMediaServer};
use std::{
    sync::mpsc::{Receiver, TryRecvError, sync_channel},
    time::{Duration, Instant},
};

/// Minimum interval between complete graph requests.
const INTERVAL: Duration = Duration::from_millis(250);
/// A slower reading no longer describes what is happening now.
const STALE: Duration = Duration::from_secs(2);
/// A complete graph, or the reason it could not be read.
type Reading = Result<InUse, NotHeard>;

/// The latest complete reading and one bounded outstanding request.
pub(crate) struct MediaActivity {
    /// Rows derived from the latest successful graph.
    lines: Vec<Line>,
    /// Why that graph is not currently known.
    problem: Option<NotHeard>,
    /// The sole outstanding worker's bounded handoff.
    pending: Option<Receiver<Reading>>,
    /// When that worker started, also used to limit request frequency.
    started: Option<Instant>,
    /// Leave recovery time after completion, especially after a slow query.
    next_request: Option<Instant>,
}

impl MediaActivity {
    /// Start visibly unknown until the server answers.
    pub(crate) fn new() -> Self {
        Self {
            lines: Vec::new(),
            problem: Some(unavailable("waiting for the first media-server reading")),
            pending: None,
            started: None,
            next_request: None,
        }
    }
    /// Current rows for the renderer.
    pub(crate) fn lines(&self) -> &[Line] {
        &self.lines
    }
    /// The failure the renderer must display instead of silence.
    pub(crate) fn problem(&self) -> Option<&NotHeard> {
        self.problem.as_ref()
    }

    /// Poll without waiting for a tool, socket, or worker thread.
    pub(crate) fn refresh(&mut self) {
        self.refresh_at(Instant::now(), || {
            InUse::read_from(&mut TheMediaServer::on_this_machine())
        });
    }

    /// Explicit time and reader let regressions hold a genuinely pending query.
    fn refresh_at(&mut self, now: Instant, read: impl FnOnce() -> Reading + Send + 'static) {
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(reading) => {
                    self.pending = None;
                    let late = self
                        .started
                        .is_some_and(|at| now.saturating_duration_since(at) >= STALE);
                    let failed = late || reading.is_err();
                    self.next_request = Some(now + if failed { STALE } else { INTERVAL });
                    self.accept(if late {
                        Err(unavailable("the media-server reading timed out"))
                    } else {
                        reading
                    });
                }
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.next_request = Some(now + STALE);
                    self.accept(Err(unavailable("the media-reading worker stopped")));
                }
                Err(TryRecvError::Empty) => {
                    if self
                        .started
                        .is_some_and(|started| now.saturating_duration_since(started) >= STALE)
                    {
                        self.accept(Err(unavailable("the media-server reading timed out")));
                    }
                    return;
                }
            }
        }
        if self
            .started
            .is_some_and(|at| now.saturating_duration_since(at) >= STALE)
        {
            self.accept(Err(unavailable("the media-server reading timed out")));
        }
        if self.next_request.is_some_and(|at| now < at) {
            return;
        }
        let (sender, receiver) = sync_channel(1);
        match std::thread::Builder::new()
            .name("alo-media-activity".into())
            .spawn(move || {
                let _ = sender.send(read());
            }) {
            Ok(_) => {
                self.pending = Some(receiver);
                self.started = Some(now);
            }
            Err(error) => {
                self.started = Some(now);
                self.next_request = Some(now + STALE);
                self.accept(Err(unavailable(&error.to_string())));
            }
        }
    }
    /// Replace the complete reading; a failure cannot retain a quiet status.
    fn accept(&mut self, reading: Reading) {
        match reading {
            Ok(reading) => {
                self.lines = reading.lines();
                self.problem = None;
            }
            Err(problem) => {
                if self.problem.as_ref() != Some(&problem) {
                    eprintln!("alo-desktop: media activity unavailable: {problem}");
                }
                self.lines.clear();
                self.problem = Some(problem);
            }
        }
    }
}

/// Preserve a diagnostic behind the existing translated failure vocabulary.
fn unavailable(said: &str) -> NotHeard {
    NotHeard::NoAnswer {
        said: said.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_in_use::{By, Streams, Use, UseId, Used};

    struct Graph(Vec<Use>);
    impl Streams for Graph {
        fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
            Ok(self.0.clone())
        }
    }
    #[test]
    fn an_unread_or_failed_server_never_becomes_a_quiet_indicator() {
        let mut activity = MediaActivity::new();
        assert!(activity.problem().is_some());
        let uses = vec![
            Use::of(
                UseId::recorded(41),
                Used::Screen,
                By::something_on_this_machine(),
            ),
            Use::of(
                UseId::recorded(42),
                Used::Microphone,
                By::something_on_this_machine(),
            ),
        ];
        activity.accept(InUse::read_from(&mut Graph(uses)));
        assert_eq!(activity.lines().len(), 2);
        assert!(activity.problem().is_none());
        activity.accept(Err(unavailable("server disconnected")));
        assert!(activity.problem().is_some());
        assert!(activity.lines().is_empty());
        activity.accept(InUse::read_from(&mut Graph(vec![])));
        assert!(activity.problem().is_none());
        assert!(activity.lines().is_empty());
    }
    #[test]
    fn a_pending_worker_does_not_block_the_frame_or_spawn_more_readers() {
        let mut activity = MediaActivity::new();
        let (_sender, receiver) = sync_channel(1);
        activity.pending = Some(receiver);
        activity.started = Instant::now().checked_sub(STALE);
        let began = Instant::now();
        for _ in 0..100 {
            activity.refresh();
        }
        assert!(began.elapsed() < Duration::from_secs(1));
        assert!(activity.pending.is_some());
        assert!(activity.problem().is_some());
    }

    #[test]
    fn a_late_quiet_result_stays_unknown_and_does_not_restart_immediately() -> Result<(), String> {
        let now = Instant::now();
        let mut activity = MediaActivity::new();
        let (sender, receiver) = sync_channel(1);
        sender
            .send(InUse::read_from(&mut Graph(vec![])))
            .map_err(|e| e.to_string())?;
        activity.started = Some(now);
        activity.pending = Some(receiver);
        activity.refresh_at(now + STALE, || InUse::read_from(&mut Graph(vec![])));
        assert!(activity.problem().is_some());
        assert!(activity.pending.is_none());
        activity.refresh_at(now + STALE + INTERVAL, || {
            InUse::read_from(&mut Graph(vec![]))
        });
        assert!(activity.problem().is_some());
        assert!(activity.pending.is_none());
        activity.refresh_at(now + STALE + STALE, || InUse::read_from(&mut Graph(vec![])));
        assert!(activity.pending.is_some());
        Ok(())
    }

    #[test]
    fn a_slow_valid_result_leaves_an_interval_before_the_next_query() -> Result<(), String> {
        let now = Instant::now();
        let mut activity = MediaActivity::new();
        let (sender, receiver) = sync_channel(1);
        sender
            .send(InUse::read_from(&mut Graph(vec![])))
            .map_err(|e| e.to_string())?;
        activity.started = Some(now);
        activity.pending = Some(receiver);
        let completed = now + Duration::from_secs(1);
        activity.refresh_at(completed, || InUse::read_from(&mut Graph(vec![])));
        assert!(activity.problem().is_none());
        assert!(activity.pending.is_none());
        activity.refresh_at(completed + INTERVAL, || {
            InUse::read_from(&mut Graph(vec![]))
        });
        assert!(activity.pending.is_some());
        Ok(())
    }
}
