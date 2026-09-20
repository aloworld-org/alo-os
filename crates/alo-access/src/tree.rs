//! **What a screen reader is told about every surface this machine draws.**
//!
//! Task 2 of `docs/autonomy/v0-5-access-and-language-plan.md`.
//! `docs/contracts/app-adapters.md` says an agent reads an application through
//! its accessibility tree where no adapter exists; a screen reader reads the
//! same tree. **If it is good enough for one it is good enough for the other**,
//! so this is one description, used by both, rather than an agent's view of the
//! machine and a blind person's view of it drifting apart.
//!
//! # What is decided here, and what is not
//!
//! Decided here: the role, the name and the state of every control the shell
//! draws. Not here: the drawing, the tree's wire format, and any reading aloud —
//! the shell exposes this through AT-SPI and Orca speaks it, both rented and
//! never written by us (ADR 0011).
//!
//! # A name is a sentence, not a label
//!
//! Every name is an `alo_strings::Word`, because a control named in English to
//! somebody using the machine in Latvian is a control they cannot find. The
//! words are `alo-access`'s own ([`crate::words`]) or the ones the shell's own
//! crates already declare for what they draw.

use crate::words::{self, Word};

/// **What kind of thing a control is**, in the vocabulary AT-SPI uses.
///
/// A short list rather than that specification's hundred: these are the kinds
/// alo OS's own surfaces are made of, and a kind nothing draws would be a kind
/// nobody tested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A window or screen a person is in.
    Window,
    /// A thing that is pressed and does something.
    Button,
    /// Words that are read and not acted on.
    Label,
    /// Where text is typed.
    Entry,
    /// Where text is typed and never shown back.
    PasswordEntry,
    /// A strip of state along an edge.
    StatusBar,
    /// A thing that is on or off.
    Switch,
    /// A list of things.
    List,
    /// One thing in a list.
    ListItem,
    /// A question that must be answered before anything else happens.
    Dialogue,
}

/// **Whether a control can be acted on, and whether it already carries a
/// state** — what a reader says after the name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Read and nothing more.
    ReadOnly,
    /// A person can act on it.
    CanBeUsed,
    /// On or off, and which it is now is read from the machine.
    OnOrOff,
    /// Announced when it changes, once, whether or not anybody is looking at it.
    AnnouncedWhenItChanges,
}

/// One control, as a reader is told about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Control {
    /// What kind of thing it is.
    pub role: Role,
    /// What it is called, in the person's own language.
    pub name: Word,
    /// What can be done with it, or what it announces.
    pub state: State,
}

impl Control {
    /// One control.
    const fn of(role: Role, name: Word, state: State) -> Self {
        Self { role, name, state }
    }

    /// **Whether a person can act on this**, which is what decides whether the
    /// keyboard stops on it (`crate::reaching`).
    ///
    /// A switch is acted on and a thing that announces itself is not: somebody
    /// hears that something is leaving the machine, and there is nothing to
    /// press about it.
    #[must_use]
    pub const fn can_be_used(&self) -> bool {
        matches!(self.state, State::CanBeUsed | State::OnOrOff)
    }
}

/// **Every surface this machine draws**, as the shell's own frames name them.
///
/// The list is held to `alo-shell`'s exports by
/// `tests/every_surface_the_shell_draws_is_read_aloud.rs`, which reads that
/// crate's own source: a surface added there with no entry here fails, because
/// a screen a person cannot be told about is a screen they cannot use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// The screen a person reaches when the desktop will not start, and the one
    /// road out of it: going back to the version of this machine before.
    ///
    /// **Before sign-in**, because this is the surface that exists precisely
    /// when the workspace does not: somebody who cannot see the screen must
    /// reach it without an account, which is what *reachable when the workspace
    /// is not* means for a reader.
    Recovery,
    /// Where somebody signs in — drawn before any account is chosen.
    SignIn,
    /// The desktop itself, and the windows on it.
    Desktop,
    /// The dock.
    Dock,
    /// The status area, where what is running and what is leaving are shown.
    StatusArea,
    /// The approval surface: one sentence, two answers (ADR 0001).
    Approval,
    /// The record: what has happened on this machine.
    Record,
    /// Settings.
    Settings,
    /// The controls that close, move and arrange a window.
    WindowControls,
}

impl Surface {
    /// All nine, in the order a person meets them — recovery first, because a
    /// machine that will not start is met before anything a person signs into.
    pub const ALL: [Self; 9] = [
        Self::Recovery,
        Self::SignIn,
        Self::Desktop,
        Self::Dock,
        Self::StatusArea,
        Self::Approval,
        Self::Record,
        Self::Settings,
        Self::WindowControls,
    ];

    /// **Which of the shell's frames or screens draw this**, as that crate
    /// spells them — what the test matches its exports against.
    ///
    /// More than one where a surface has both: the approval is an
    /// `ApprovalScreen` that a nested `ApprovalFrame` presents. None where a
    /// surface is drawn inside another's frame, as the dock is inside the
    /// desktop's — it is still a surface a person meets and must still be read
    /// aloud, which is why it is here with nothing beside it rather than left
    /// out.
    #[must_use]
    pub const fn drawn_by(self) -> &'static [&'static str] {
        match self {
            Self::Recovery => &["RecoveryFrame", "RecoveryScreen"],
            Self::SignIn => &["SignInScreen"],
            Self::Desktop => &["DesktopFrame"],
            Self::Dock => &[],
            Self::StatusArea => &["EgressStatusFrame"],
            Self::Approval => &["ApprovalFrame", "ApprovalScreen"],
            Self::Record => &["RecordFrame"],
            Self::Settings => &["SettingsFrame"],
            Self::WindowControls => &["WindowControlFrame"],
        }
    }

    /// **What a reader is told about this surface**, in the order it is read.
    #[must_use]
    pub fn read_aloud(self) -> Vec<Control> {
        match self {
            // The screen that is there when the desktop is not. This crate
            // names the controls and none of the sentences: the offer is
            // `alo_keeping_up::GoingBack::said`, the refusal is
            // `CannotGoBack::said`, and the two moments are `when_word` — each
            // read after the name, the way an application's own name is read
            // after `AN_APPLICATION`.
            //
            // **Nothing is preselected.** One of the two moments restarts the
            // machine, and an Enter held down from whatever just failed a
            // moment ago would be that. So both moments are controls a person
            // moves to, and neither carries a state that reads as chosen.
            Self::Recovery => vec![
                Control::of(Role::Window, words::THE_RECOVERY_SCREEN, State::ReadOnly),
                // What is running and what it replaced are **named here and
                // said by nobody yet**: `alo_keeping_up::Deployments` has no
                // `said` and `Since` has no words, so each line has a name and
                // no sentence until that crate writes one. Naming the control
                // anyway is what lets a reader announce the line at all, and
                // the gap is written down as a finding in
                // `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`.
                Control::of(Role::Label, words::WHAT_IS_RUNNING, State::ReadOnly),
                Control::of(Role::Label, words::WHAT_IT_REPLACED, State::ReadOnly),
                Control::of(Role::List, words::THE_CHOICES, State::CanBeUsed),
                Control::of(
                    Role::Button,
                    words::GO_BACK_AT_THE_NEXT_RESTART,
                    State::CanBeUsed,
                ),
                Control::of(Role::Button, words::GO_BACK_NOW, State::CanBeUsed),
            ],
            Self::SignIn => vec![
                Control::of(Role::Window, words::SIGN_IN, State::ReadOnly),
                Control::of(Role::List, words::WHO_IS_SIGNING_IN, State::CanBeUsed),
                Control::of(Role::PasswordEntry, words::THE_PASSWORD, State::CanBeUsed),
                Control::of(Role::Button, words::SIGN_IN_NOW, State::CanBeUsed),
                // The whole of task 1 is reachable here, before an account
                // exists, so a reader must be able to find it here too.
                Control::of(Role::Button, words::ACCESS_SETTINGS, State::CanBeUsed),
            ],
            Self::Desktop => vec![
                Control::of(Role::Window, words::THE_DESKTOP, State::ReadOnly),
                Control::of(Role::List, words::THE_WINDOWS_OPEN, State::CanBeUsed),
                // The agent answers to a chord, and a chord is not a road for
                // somebody who has not been told it. `crate::reaching` holds
                // that every action this machine offers is also a place the
                // keyboard arrives at by pressing Tab; this is the agent's.
                Control::of(Role::Button, words::ASK_THE_AGENT, State::CanBeUsed),
            ],
            Self::Dock => vec![
                Control::of(Role::List, words::THE_DOCK, State::CanBeUsed),
                Control::of(Role::ListItem, words::AN_APPLICATION, State::CanBeUsed),
                Control::of(Role::Button, words::THE_LAUNCHER, State::CanBeUsed),
            ],
            Self::StatusArea => vec![
                Control::of(Role::StatusBar, words::THE_STATUS_AREA, State::ReadOnly),
                // The two things this machine promises to make visible, and
                // both are announced rather than waited for: law 1's indicator
                // is not a diagnostic somebody goes looking for.
                Control::of(
                    Role::Label,
                    words::SOMETHING_IS_LEAVING,
                    State::AnnouncedWhenItChanges,
                ),
                Control::of(
                    Role::Label,
                    words::THE_AGENT_IS_WORKING,
                    State::AnnouncedWhenItChanges,
                ),
            ],
            // ADR 0001: what a person approves is the sentence the turn wrote,
            // and the two answers follow it with nothing chosen for them.
            Self::Approval => vec![
                Control::of(Role::Dialogue, words::SOMETHING_IS_ASKED, State::ReadOnly),
                Control::of(Role::Label, words::WHAT_THE_TURN_WROTE, State::ReadOnly),
                Control::of(Role::Button, words::SAY_NO, State::CanBeUsed),
                Control::of(Role::Button, words::APPROVE_IT, State::CanBeUsed),
            ],
            Self::Record => vec![
                Control::of(Role::Window, words::THE_RECORD, State::ReadOnly),
                Control::of(Role::List, words::WHAT_HAPPENED, State::CanBeUsed),
                Control::of(
                    Role::ListItem,
                    words::ONE_THING_THAT_HAPPENED,
                    State::ReadOnly,
                ),
            ],
            Self::Settings => vec![
                Control::of(Role::Window, words::SETTINGS, State::ReadOnly),
                Control::of(Role::List, words::WHAT_CAN_BE_CHANGED, State::CanBeUsed),
                Control::of(Role::Switch, words::A_SETTING, State::OnOrOff),
            ],
            Self::WindowControls => vec![
                Control::of(Role::Button, words::CLOSE_THIS_WINDOW, State::CanBeUsed),
                Control::of(Role::Button, words::ARRANGE_THIS_WINDOW, State::CanBeUsed),
            ],
        }
    }
}

/// **What the approval surface reads as**, in order: the sentence the turn
/// wrote, then the two answers — and nothing is chosen for the person.
///
/// ADR 0001: *what a person approves is that sentence*. A reader that announced
/// a preselected answer would be telling somebody who cannot see the screen that
/// a choice had already been made for them.
#[must_use]
pub fn the_approval_in_reading_order() -> Vec<Control> {
    Surface::Approval.read_aloud()
}

/// How many of this crate's words name something a reader says, rather than a
/// setting a person turns on — the count `words.rs` holds its own list to.
pub const EVERY_NAME_A_READER_SAYS: usize = 32;

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every name a reader says is one of this crate's words, and the count
    /// `words.rs` checks itself against is the truth.**
    #[test]
    fn every_name_in_the_tree_is_a_word_this_crate_declares() {
        let mut named: Vec<String> = Surface::ALL
            .into_iter()
            .flat_map(|surface| surface.read_aloud())
            .map(|control| control.name.key().to_string())
            .collect();
        named.sort();
        named.dedup();
        assert_eq!(
            named.len(),
            EVERY_NAME_A_READER_SAYS,
            "the tree names {} things and the count says {EVERY_NAME_A_READER_SAYS}",
            named.len()
        );
        for key in named {
            assert!(
                crate::words::EVERY_WORD
                    .iter()
                    .any(|word| word.key().to_string() == key),
                "{key} is read aloud and not declared"
            );
        }
    }

    /// **Every surface is read as something, and every control is named.**
    #[test]
    fn every_surface_says_what_it_is_and_what_is_in_it() {
        for surface in Surface::ALL {
            let controls = surface.read_aloud();
            assert!(!controls.is_empty(), "{surface:?} is read as nothing");
        }
    }

    /// **The approval surface: the sentence, then no, then approve — and
    /// nothing preselected** (ADR 0001).
    #[test]
    fn the_approval_is_read_as_the_sentence_then_its_two_answers() {
        let read = the_approval_in_reading_order();
        let roles: Vec<Role> = read.iter().map(|control| control.role).collect();
        assert_eq!(
            roles,
            vec![Role::Dialogue, Role::Label, Role::Button, Role::Button],
            "the approval surface is read in another order"
        );
        let keys: Vec<String> = read
            .iter()
            .map(|control| control.name.key().to_string())
            .collect();
        assert_eq!(
            keys,
            vec![
                "access.something-is-asked".to_owned(),
                "access.what-the-turn-wrote".to_owned(),
                "access.say-no".to_owned(),
                "access.approve-it".to_owned(),
            ]
        );
        // Nothing carries a state that would read as *chosen*.
        for control in &read {
            assert_ne!(
                control.state,
                State::OnOrOff,
                "an answer on the approval surface reads as already chosen"
            );
        }
    }

    /// **What must be announced is announced**: the egress indicator and the
    /// agent's, both without anybody looking for them (law 1).
    #[test]
    fn what_is_leaving_and_what_the_agent_is_doing_are_announced_rather_than_found() {
        let announced: Vec<&Control> = Surface::StatusArea
            .read_aloud()
            .iter()
            .filter(|control| control.state == State::AnnouncedWhenItChanges)
            .count()
            .eq(&2)
            .then(Vec::new)
            .unwrap_or_default();
        assert!(announced.is_empty());
        let status = Surface::StatusArea.read_aloud();
        let announced = status
            .iter()
            .filter(|control| control.state == State::AnnouncedWhenItChanges)
            .count();
        assert_eq!(announced, 2, "the two indicators are not both announced");
    }

    /// **Access settings are reachable at sign-in**, in the tree as well as in
    /// the file — the same promise task 1 made, read aloud.
    #[test]
    fn the_access_settings_are_in_the_sign_in_surface() {
        assert!(
            Surface::SignIn
                .read_aloud()
                .iter()
                .any(|control| control.name.key().to_string() == "access.settings-here"),
            "a person who needs a reader cannot find the settings at sign-in"
        );
    }
}
