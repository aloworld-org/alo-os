//! **The kernel's counter**, as the counting in `alo-shell` asks for it
//! (ADR 0148).
//!
//! `alo_counting::Counter` opened by the pin `alo-boundaryd` made at boot. A
//! machine where it cannot be opened — no boundary loaded, a boundary loaded
//! before ADR 0148, a login not in the agent's group — has no counter, and the
//! desktop says so once on its log and answers every count *cannot count*:
//! never *nothing was sent*.

use alo_counting::{Counter, ON_THIS_MACHINE};
use alo_shell::counting_during_their_commands::CountsWhatIsSent;

/// The kernel's counter.
pub struct TheKernels(Counter);

impl TheKernels {
    /// The counter on this machine, or [`None`] with the reason said.
    #[must_use]
    pub fn on_this_machine() -> Option<Box<dyn CountsWhatIsSent>> {
        match Counter::opened(std::path::Path::new(ON_THIS_MACHINE)) {
            Ok(counter) => Some(Box::new(Self(counter))),
            Err(why) => {
                eprintln!(
                    "alo-desktop: what an application sends during a person's command cannot be \
                     counted here, so every such count is refused: {why}"
                );
                None
            }
        }
    }
}

impl CountsWhatIsSent for TheKernels {
    fn begun(&mut self, group: u64) -> bool {
        self.0.begun(group).is_ok()
    }

    fn so_far(&self, group: u64) -> Option<u64> {
        self.0.so_far(group).ok().flatten()
    }

    fn ended(&mut self, group: u64) {
        if let Err(why) = self.0.ended(group) {
            // Nothing here asks again, and the entry is the kernel's until the
            // boundary is loaded afresh at the next boot: said plainly, because
            // a count that outlives its command is the one thing ADR 0148
            // forbids, and it must be findable.
            eprintln!(
                "alo-desktop: a count could not be ended, so the kernel keeps counting control \
                 group {group} until this machine restarts: {why}"
            );
        }
    }
}
