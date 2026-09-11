//! Whether this person has been asked what their machine should do, and
//! answered.
//!
//! One bit, and it exists because two states of a settings file are otherwise
//! the same file.
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)
//! gave setup a fourth choice — *no model, no provider, no agent* — with the
//! same weight as the other three, and
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! settled that until somebody chooses, nothing answers. Both are true of a
//! machine nobody has configured **and** of a machine whose owner said *not at
//! all*: `crate::Settings::chosen` is [`None`] either way, and nothing answers
//! either way.
//!
//! They are not the same machine. One is waiting to be asked; the other has
//! been asked, has answered, and is finished. A machine that could not tell them
//! apart would put setup in front of somebody who has already declined it —
//! which is ADR 0009's *no nagging* broken by the one mechanism that is
//! guaranteed to meet everybody who declined.
//!
//! # It is not a second copy of the choice
//!
//! [`Setup::Answered`] says only that the question was put and answered. **What
//! was answered is `crate::Settings::chosen`**, exactly as it always was: a
//! source, or nothing at all. So there is no way for this and the choice to
//! disagree, and no rule anywhere that has to keep them in step — which is the
//! whole reason it is a bit rather than a copy of the four choices. A file
//! saying *I chose nothing* beside a choice would be a settings file with two
//! answers in it, and something would have to decide which one the person meant.
//!
//! # Nothing here is written by anybody but the person
//!
//! A machine that shipped with this set would be ADR 0025's Option B — us
//! writing in the one file ADR 0016 keeps for the person — with the mechanism
//! moved somewhere nobody would look for it. It is absent on a machine nobody
//! has configured, `crate::Choosing::at` writes nothing for it, and the only
//! door that sets it is `crate::Choosing::setting_up`.

/// Whether the person whose file this is has answered setup.
///
/// Absent from a settings file is [`Setup::NotAnswered`], which is the ordinary
/// state of a machine on its first morning and of every machine configured
/// before this key existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Setup {
    /// Nobody has answered it.
    ///
    /// The state a machine arrives in. It says nothing about whether anything
    /// answers questions: a person may have configured their machine through
    /// Settings without ever being taken through setup, and their file says so
    /// honestly rather than claiming they were asked.
    #[default]
    NotAnswered,

    /// It was put to them, and they answered.
    ///
    /// What they answered is `crate::Settings::chosen`. [`None`] there is
    /// ADR 0009's fourth choice — a finished setup on a machine with no agent —
    /// and not a person who has yet to be asked.
    Answered,
}

impl Setup {
    /// Whether setup is finished.
    ///
    /// True of a machine whose owner chose a source **and** of one whose owner
    /// declined, because both were asked and both answered. Whoever wants to
    /// know which of the two it was asks `crate::Settings::chosen`.
    #[must_use]
    pub const fn is_answered(self) -> bool {
        matches!(self, Self::Answered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A machine nobody has configured has not answered setup**, which is the
    /// state the first morning is in and the state every file written before
    /// this key existed reads as.
    #[test]
    fn a_machine_nobody_has_configured_has_not_answered() {
        assert_eq!(Setup::default(), Setup::NotAnswered);
        assert!(!Setup::NotAnswered.is_answered());
    }

    /// **An answered setup is finished**, whichever of the four was answered.
    #[test]
    fn an_answered_setup_is_finished() {
        assert!(Setup::Answered.is_answered());
    }
}
