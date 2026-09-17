//! **Why the media server could not be asked** — and the four facts that are
//! four different facts.
//!
//! This is the reason this crate exists as a crate. Three of these used to be
//! two, in three places, and the two that read the same were the two that matter
//! most:
//!
//! | What is true of the machine | What it used to say |
//! |---|---|
//! | there is no such tool on it | *nothing here handles sound and video* |
//! | the tool is there and **no session is running** | *the server would not answer* |
//! | the server is there and failed | *the server would not answer* |
//! | the server answered something unreadable | *it answered something unreadable* |
//!
//! **An ordinary build host is not a broken machine.** A machine with the
//! package installed and nobody signed in has no media server running, which is
//! not a fault and not something to fix; reading it as a server that would not
//! answer sent somebody looking for a broken service that was never started.
//!
//! **And a server that answered nonsense must never read as a quiet machine.**
//! That is the other half, and it is why [`NotAsked::ItAnsweredSomethingUnreadable`]
//! is its own fact rather than an empty list: an indicator that showed nothing
//! because a record would not parse looks exactly like an indicator on a machine
//! where no camera is on, and that is the wrong answer given confidently about
//! the one thing an indicator is for.
//!
//! # What a crate reading this does with them
//!
//! Turns them into its own sentences. Nothing here is shown to a person: this
//! crate has no vocabulary and wants none, because *what is watching* and *which
//! speaker is this* are two different things to be told and belong to the crates
//! that know which is being asked.

/// Why the media server could not be asked, or its answer not read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAsked {
    /// There is no such tool on this machine: nothing here handles sound and
    /// video at all.
    #[error("nothing on this machine handles sound and video: {said}")]
    NothingHandlesIt {
        /// What trying to run it said. English, for whoever is fixing the
        /// machine, and never shown to a person.
        said: String,
    },
    /// The tool is there and no server is listening — an ordinary machine whose
    /// session has not started, not a broken one.
    #[error("this machine has no media server running: {said}")]
    NoServerIsRunning {
        /// What the tool said.
        said: String,
    },
    /// The server is there and the asking failed.
    #[error("this machine's media server would not answer: {said}")]
    ItWouldNotAnswer {
        /// What it said instead.
        said: String,
    },
    /// It answered something this crate cannot read.
    #[error("this machine's media server answered something unreadable: {said}")]
    ItAnsweredSomethingUnreadable {
        /// What was wrong with the answer.
        said: String,
    },
}

impl NotAsked {
    /// What to put in front of whoever is fixing the machine.
    #[must_use]
    pub fn diagnosis(&self) -> String {
        self.to_string()
    }

    /// **Whether this machine simply has no media server at work** — either
    /// because nothing here handles sound and video, or because nothing is
    /// running.
    ///
    /// The question a test asks before it skips itself, and the question a
    /// surface asks before it says *nothing here handles sound and video*. The
    /// other two are failures; these two are machines.
    #[must_use]
    pub const fn is_a_machine_without_one(&self) -> bool {
        matches!(
            self,
            Self::NothingHandlesIt { .. } | Self::NoServerIsRunning { .. }
        )
    }

    /// What the tool said, whichever fact this is.
    #[must_use]
    pub fn said(&self) -> &str {
        match self {
            Self::NothingHandlesIt { said }
            | Self::NoServerIsRunning { said }
            | Self::ItWouldNotAnswer { said }
            | Self::ItAnsweredSomethingUnreadable { said } => said,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Four facts, and the two that mean *there is no server here* say so
    /// together** — which is the question a test or a surface actually asks.
    #[test]
    fn a_machine_without_a_server_is_told_apart_from_a_machine_with_a_broken_one() {
        let said = || "something".to_owned();
        assert!(NotAsked::NothingHandlesIt { said: said() }.is_a_machine_without_one());
        assert!(NotAsked::NoServerIsRunning { said: said() }.is_a_machine_without_one());
        assert!(!NotAsked::ItWouldNotAnswer { said: said() }.is_a_machine_without_one());
        assert!(
            !NotAsked::ItAnsweredSomethingUnreadable { said: said() }.is_a_machine_without_one(),
            "a record that would not parse is not a machine without a server, and an indicator \
             that treated it as one would show nothing while a camera was on"
        );
    }

    /// **Each of the four says a different thing**, so a log names which.
    #[test]
    fn the_four_read_differently() {
        let said = || "because".to_owned();
        let every = [
            NotAsked::NothingHandlesIt { said: said() },
            NotAsked::NoServerIsRunning { said: said() },
            NotAsked::ItWouldNotAnswer { said: said() },
            NotAsked::ItAnsweredSomethingUnreadable { said: said() },
        ];
        let mut sentences: Vec<String> = every.iter().map(NotAsked::diagnosis).collect();
        sentences.sort_unstable();
        sentences.dedup();
        assert_eq!(sentences.len(), every.len());
        assert!(every.iter().all(|why| why.said() == "because"));
    }
}
