//! What closing the lid does.
//!
//! **It sleeps the machine**, unless another display is attached and the person
//! chose that it should not. Two choices and no third: a lid closed on a laptop
//! with nothing plugged in always sleeps whatever was chosen, because a machine
//! running in a bag is the failure this rule exists to prevent, and a person
//! who wants that has the setting that keeps a machine awake with its lid open.
//!
//! **Keepers do not hold a closed lid open.** An application holding the inhibit
//! portal or a turn that is running keeps the machine from sleeping *on its
//! own*; closing the lid is the person's own act, and it is theirs
//! (`crate::deciding`).

use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// What the person chose closing the lid does.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lid {
    /// Closing the lid always sleeps the machine. What alo OS ships.
    #[default]
    Sleeps,
    /// Closing the lid sleeps the machine unless another display is attached.
    StaysAwakeWithADisplay,
}

/// Whether a display other than the machine's own is attached at the moment
/// the lid closes.
///
/// A value handed in: which displays there are is `alo-displays`' to know, and
/// this crate asks nothing more of it than this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Displays {
    /// Only the machine's own display, or none.
    OnlyItsOwn,
    /// At least one other display is attached.
    AnotherAttached,
}

/// What a closed lid does, now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LidClosed {
    /// The machine sleeps.
    Sleeps,
    /// The machine stays awake, as the person chose.
    StaysAwake,
}

impl Lid {
    /// What closing the lid does with these displays attached.
    #[must_use]
    pub const fn closed(self, displays: Displays) -> LidClosed {
        match (self, displays) {
            (Self::StaysAwakeWithADisplay, Displays::AnotherAttached) => LidClosed::StaysAwake,
            (Self::Sleeps, _) | (Self::StaysAwakeWithADisplay, Displays::OnlyItsOwn) => {
                LidClosed::Sleeps
            }
        }
    }

    /// The string this crate declares for this choice, as Settings lists it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Sleeps => words::LID_SLEEPS,
            Self::StaysAwakeWithADisplay => words::LID_STAYS_AWAKE_WITH_A_DISPLAY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Closing the lid sleeps the machine unless a display is attached and
    /// the person chose otherwise** — all four cases, and three of them sleep.
    #[test]
    fn a_closed_lid_sleeps_unless_a_display_is_attached_and_the_person_chose_otherwise() {
        assert_eq!(Lid::default(), Lid::Sleeps);
        assert_eq!(Lid::Sleeps.closed(Displays::OnlyItsOwn), LidClosed::Sleeps);
        assert_eq!(
            Lid::Sleeps.closed(Displays::AnotherAttached),
            LidClosed::Sleeps
        );
        assert_eq!(
            Lid::StaysAwakeWithADisplay.closed(Displays::OnlyItsOwn),
            LidClosed::Sleeps,
            "a laptop in a bag sleeps whatever was chosen"
        );
        assert_eq!(
            Lid::StaysAwakeWithADisplay.closed(Displays::AnotherAttached),
            LidClosed::StaysAwake
        );
    }
}
