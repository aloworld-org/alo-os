//! Live media activity, read off the compositor thread and never guessed quiet.

use alo_in_use::{InUse, Line, NotHeard, TheMediaServer};
use std::{
    sync::mpsc::{Receiver, TryRecvError, sync_channel},
    time::{Duration, Instant},
};

const INTERVAL: Duration = Duration::from_millis(250);
const STALE: Duration = Duration::from_secs(2);
type Reading = Result<InUse, NotHeard>;

/// The latest complete reading and one bounded outstanding request.
pub(crate) struct MediaActivity {
    lines: Vec<Line>,
    problem: Option<NotHeard>,
    pending: Option<Receiver<Reading>>,
    started: Option<Instant>,
}

impl MediaActivity {
    pub(crate) fn new() -> Self {
        Self {
            lines: Vec::new(),
            problem: Some(unavailable("waiting for the first media-server reading")),
            pending: None,
            started: None,
        }
    }
    pub(crate) fn lines(&self) -> &[Line] {
        &self.lines
    }
    pub(crate) fn problem(&self) -> Option<&NotHeard> {
        self.problem.as_ref()
    }

    /// Poll without waiting for a tool, socket, or worker thread.
    pub(crate) fn refresh(&mut self) {
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(reading) => {
                    self.pending = None;
                    self.accept(reading);
                }
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    self.accept(Err(unavailable("the media-reading worker stopped")));
                }
                Err(TryRecvError::Empty) => {
                    if self
                        .started
                        .is_some_and(|started| started.elapsed() >= STALE)
                    {
                        self.accept(Err(unavailable("the media-server reading timed out")));
                    }
                    return;
                }
            }
        }
        if self
            .started
            .is_some_and(|started| started.elapsed() < INTERVAL)
        {
            return;
        }
        let (sender, receiver) = sync_channel(1);
        match std::thread::Builder::new()
            .name("alo-media-activity".into())
            .spawn(move || {
                let _ = sender.send(InUse::read_from(&mut TheMediaServer::on_this_machine()));
            }) {
            Ok(_) => {
                self.pending = Some(receiver);
                self.started = Some(Instant::now());
            }
            Err(error) => {
                self.started = Some(Instant::now());
                self.accept(Err(unavailable(&error.to_string())));
            }
        }
    }
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
}
