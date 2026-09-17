//! **What is draining the battery, and naming the model when it is the model.**
//!
//! *Why is my battery gone* is a question this machine can answer, and on a
//! machine whose first-class workload is a model the customer owns, the answer
//! is often *the model you are running*. A machine that would not say so would
//! be leaving its owner to guess at the one process it knows the most about.
//!
//! # Nothing here watches anything
//!
//! This is handed a reading of what is running — `alo-measuring`'s, taken when
//! somebody asked — and picks out of it. There is no loop, no history and
//! nothing kept: the question is asked when a person asks it or when the battery
//! gets low, and answered from what the machine is doing at that moment.
//!
//! # And it names one thing, or nothing
//!
//! A list of eleven processes is not an answer to *why*. One process over
//! [`ENOUGH_TO_NAME`] of the machine is named; where nothing is over it, the
//! honest answer is that nothing in particular is — a machine can be flat
//! because it is old, and saying so is better than blaming whatever happened to
//! sort first.

use alo_measuring::Process;

use crate::words::{self, Word};

/// How much of the whole machine a process has to be using, in thousandths,
/// before it is worth naming. A quarter of the machine, sustained, is what a
/// person notices as heat and hears as a fan.
pub const ENOUGH_TO_NAME: u64 = 250;

/// **The model this person is running**, so it can be named as itself.
///
/// alo OS starts the model, so it knows which process it is; nothing here
/// guesses from a name, because a process called `llama` might be anything and
/// the model might be called anything.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheModel {
    /// The processes it runs as, where it is running at all.
    pids: Vec<u32>,
}

impl TheModel {
    /// Nothing is running.
    #[must_use]
    pub fn none_running() -> Self {
        Self::default()
    }

    /// The model, running as these.
    #[must_use]
    pub const fn running_as(pids: Vec<u32>) -> Self {
        Self { pids }
    }

    /// Whether this process is it.
    #[must_use]
    pub fn is_it(&self, pid: u32) -> bool {
        self.pids.contains(&pid)
    }
}

/// **What is draining the battery.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatIsDrainingIt {
    /// The person's own model, which this machine knows by name because it
    /// started it.
    TheModel {
        /// What it calls itself.
        called: String,
        /// Its share of the machine, in thousandths.
        share: u64,
    },
    /// Something else, named.
    Something {
        /// What it calls itself.
        called: String,
        /// Its share of the machine, in thousandths.
        share: u64,
    },
    /// Nothing in particular, which is an answer and often the true one.
    NothingInParticular,
}

impl WhatIsDrainingIt {
    /// What a person is told, where there is something to tell them.
    #[must_use]
    pub const fn word(&self) -> Option<Word> {
        match self {
            Self::TheModel { .. } => Some(words::THE_MODEL_IS_WHY),
            Self::Something { .. } => Some(words::SOMETHING_IS_WHY),
            Self::NothingInParticular => None,
        }
    }

    /// What it calls itself, where something was named.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        match self {
            Self::TheModel { called, .. } | Self::Something { called, .. } => Some(called),
            Self::NothingInParticular => None,
        }
    }
}

/// **What is draining it**, out of what is running.
///
/// Handed `alo_measuring::Running::processes`, which is what a reading of this
/// machine holds. It is a slice rather than the reading itself because a
/// `Running` is `alo-measuring`'s to make and this crate only reads one — and
/// because a test here can then be a handful of processes rather than a whole
/// machine.
#[must_use]
pub fn what_is_draining_it(processes: &[Process], model: &TheModel) -> WhatIsDrainingIt {
    let Some(worst) = processes
        .iter()
        .max_by_key(|process| process.processor.value().unwrap_or_default())
    else {
        return WhatIsDrainingIt::NothingInParticular;
    };
    let share = worst.processor.value().unwrap_or_default();
    if share < ENOUGH_TO_NAME {
        return WhatIsDrainingIt::NothingInParticular;
    }
    let called = a_name_for(worst);
    if model.is_it(worst.pid) {
        return WhatIsDrainingIt::TheModel { called, share };
    }
    WhatIsDrainingIt::Something { called, share }
}

/// What to call a process in front of a person: what it calls itself, or an
/// honest nothing rather than a number.
fn a_name_for(process: &Process) -> String {
    let called = process.name.trim();
    if called.is_empty() {
        return String::new();
    }
    called.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_measuring::{Number, Source};

    /// **The model is named as the model**, not as one more process.
    #[test]
    fn the_model_is_named_when_it_is_the_model() {
        let running = [
            a_process(11, "a-browser", 100),
            a_process(42, "the-model", 800),
        ];
        let draining = what_is_draining_it(&running, &TheModel::running_as(vec![42]));
        assert_eq!(
            draining,
            WhatIsDrainingIt::TheModel {
                called: "the-model".to_owned(),
                share: 800
            }
        );
        assert_eq!(draining.called(), Some("the-model"));
        assert_eq!(
            draining.word().map(|word| word.named()),
            Some(words::THE_MODEL_IS_WHY.named())
        );
    }

    /// **Something else is named as something else.**
    #[test]
    fn something_that_is_not_the_model_is_named_as_itself() {
        let running = [a_process(11, "a-browser", 900)];
        assert_eq!(
            what_is_draining_it(&running, &TheModel::none_running()),
            WhatIsDrainingIt::Something {
                called: "a-browser".to_owned(),
                share: 900
            }
        );
    }

    /// **A quiet machine is not blamed on whatever sorted first**, which is the
    /// failure this rule exists to prevent.
    #[test]
    fn a_machine_with_nothing_busy_names_nothing() {
        let running = [
            a_process(11, "a-browser", 20),
            a_process(42, "the-model", 30),
        ];
        assert_eq!(
            what_is_draining_it(&running, &TheModel::running_as(vec![42])),
            WhatIsDrainingIt::NothingInParticular
        );
        assert!(
            what_is_draining_it(&running, &TheModel::none_running())
                .word()
                .is_none()
        );
        assert_eq!(
            what_is_draining_it(&[], &TheModel::none_running()),
            WhatIsDrainingIt::NothingInParticular
        );
    }

    /// One process, as a reading of this machine holds them.
    fn a_process(pid: u32, name: &str, processor: u64) -> Process {
        let from = || Source::of("/proc/stat", "cpu");
        Process {
            pid,
            name: name.to_owned(),
            memory: Number::known(0, from()),
            processor: Number::known(processor, from()),
            read: Number::known(0, from()),
            written: Number::known(0, from()),
            received: Number::known(0, from()),
            sent: Number::known(0, from()),
            network: alo_measuring::Network {
                namespace: None,
                shared_with: 0,
            },
        }
    }
}
