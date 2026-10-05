//! **What a screen reader is told about every surface this machine draws.**
//!
//! Task 2 of `docs/autonomy/access-and-language-plan.md`.
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

use alo_shortcuts::Action;

use crate::setting::Setting;
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
    /// Which access setting this control *is*, where it is one.
    ///
    /// **A reader is told which way a switch is set, and this is how it knows
    /// which switch.** A control carrying [`State::OnOrOff`] and no setting
    /// would be a switch whose value nothing can look up, which is the shape
    /// the tree had until 2026-09-30: one control named *a setting*, standing
    /// for all nine.
    ///
    /// [`None`] for everything that is not a setting — a window, a list, a
    /// button. Those have no value to be told.
    pub setting: Option<Setting>,

    /// Which action this control performs, where it performs one.
    ///
    /// **A control a person acts on is named by what it does**
    /// ([ADR 0089](../../../docs/decisions/0089-what-a-control-is-called.md)),
    /// so its [`name`](Self::name) is `Action::word` and the words a reader
    /// says are the words drawn on it. Carrying the action rather than only
    /// the word is what lets a caller ask *which* control this is without
    /// matching on a sentence.
    ///
    /// [`None`] for everything that performs no action: a window, a list, a
    /// label, and a switch, which carries a [`setting`](Self::setting)
    /// instead. **Never both** — a control is one or the other, and
    /// `a_control_is_named_by_one_thing` holds that.
    pub does: Option<Action>,
}

impl Control {
    /// One control.
    const fn of(role: Role, name: Word, state: State) -> Self {
        Self {
            role,
            name,
            state,
            setting: None,
            does: None,
        }
    }

    /// The switch for one access setting, named by the setting itself.
    ///
    /// **The name comes from [`Setting::word`] rather than from here**, so a
    /// setting added to [`Setting::ALL`] arrives in the tree already named and
    /// there is no second list to keep in step. A tenth setting nobody teaches
    /// this file about is a tenth control, not a missing one.
    #[must_use]
    pub fn for_setting(setting: Setting) -> Self {
        Self {
            role: Role::Switch,
            name: setting.word(),
            state: State::OnOrOff,
            setting: Some(setting),
            does: None,
        }
    }

    /// **The button for one action, named by the action itself.**
    ///
    /// The name comes from [`Action::word`] rather than from this crate's own
    /// vocabulary, for the reason [`Control::for_setting`] gives one line up
    /// and [ADR 0089](../../../docs/decisions/0089-what-a-control-is-called.md)
    /// gives at length: a second list of names is a second thing to keep in
    /// step, and the two lists this crate had **disagreed about which controls
    /// exist** rather than merely about their wording.
    ///
    /// It also satisfies EN 301 549 clause 11.2.5.3 by construction: the words
    /// a reader says *are* the words drawn on the control, in every language,
    /// because there is one string rather than two that must agree.
    #[must_use]
    pub fn for_action(action: Action) -> Self {
        Self {
            role: Role::Button,
            name: action.word(),
            state: State::CanBeUsed,
            setting: None,
            does: Some(action),
        }
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
                // `docs/autonomy/the-machine-keeps-itself-plan.md`.
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
            // **The two answers are named by the words they are drawn with**,
            // which is ADR 0089's rule arriving at the surface it matters most
            // on. Until 2026-10-05 this crate announced `access.say-no` and
            // `access.approve-it` while `alo-approving` drew `approving.no`
            // and `approving.approve` — two vocabularies for two buttons,
            // translated independently into 24 languages, with nothing
            // comparing them. In German the drawn words are *Nein* and
            // *Genehmigen*; what a reader was told was whatever this crate's
            // own keys happened to become.
            //
            // Clause 11.2.5.3 asks the programmatic name to contain the
            // visible label. One string rather than two makes that true by
            // construction, in every language, and makes it impossible for a
            // translator to move one without the other.
            Self::Approval => vec![
                Control::of(Role::Dialogue, words::SOMETHING_IS_ASKED, State::ReadOnly),
                Control::of(Role::Label, words::WHAT_THE_TURN_WROTE, State::ReadOnly),
                Control::of(Role::Button, alo_approving::words::NO, State::CanBeUsed),
                Control::of(
                    Role::Button,
                    alo_approving::words::APPROVE,
                    State::CanBeUsed,
                ),
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
            // **Every setting by name, from the closed list itself.** This
            // read one switch called *a setting* until 2026-09-30 — a
            // placeholder standing for all nine, so a reader told its value
            // would have announced *"a setting, on"* and named nothing. The
            // names were already here, in `Setting::word`; only the tree was
            // not using them.
            Self::Settings => {
                let mut read = vec![
                    Control::of(Role::Window, words::SETTINGS, State::ReadOnly),
                    Control::of(Role::List, words::WHAT_CAN_BE_CHANGED, State::CanBeUsed),
                ];
                read.extend(Setting::ALL.map(Control::for_setting));
                read
            }
            // **The three buttons a window is drawn with, in the order they
            // are drawn in** — `alo_shell::WindowControls` lays them out at
            // x-offsets 0, 36 and 72, and this list is the same list in the
            // same order. Until 2026-10-04 it was two controls of this crate's
            // own naming, *close this window* and *move this window*: minimise
            // and maximise were drawn and never announced, and arranging was
            // announced and never drawn, because snapping is a chord with no
            // button. Nothing compared the two lists (ADR 0089).
            Self::WindowControls => vec![
                Control::for_action(Action::MinimiseWindow),
                Control::for_action(Action::MaximiseWindow),
                Control::for_action(Action::CloseWindow),
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

/// How many of this crate's **own** words name something a reader says — which,
/// since 2026-09-30, is all of them.
///
/// *This used to count the words that were names rather than settings, and the
/// two were disjoint: the tree named one placeholder switch called "a setting"
/// and the nine settings' own words appeared only in `Setting::word`. Now the
/// tree names every switch by its setting, so every word this crate declares is
/// a name a reader says and `words.rs` holds the two lists to being one.*
///
/// **It stopped being every name the tree says on 2026-10-04.** Three controls
/// are named by `alo_shortcuts::Action::word` instead, because a control a
/// person acts on is named by what it does
/// ([ADR 0089](../../../docs/decisions/0089-what-a-control-is-called.md)). The
/// identity this constant holds is still worth having — a word declared here
/// and never said is still a fault — but it is now one side of the tree rather
/// than the whole of it, and
/// [`EVERY_NAME_A_READER_SAYS_IN_ANOTHER_CRATES_WORDS`] is the other.
pub const EVERY_NAME_A_READER_SAYS: usize = 36;

/// How many names a reader says come from another crate's vocabulary.
///
/// The three buttons a window is drawn with, and the two answers on the
/// approval surface (ADR 0089, and the same rule applied to `alo-approving`'s
/// own words on 2026-10-05). Counted separately
/// rather than folded into the number above, because the two have different
/// failure modes: a word of ours that nothing says is a word to retire, and a
/// name of somebody else's that nothing draws is the fault this ADR was
/// written about.
pub const EVERY_NAME_A_READER_SAYS_IN_ANOTHER_CRATES_WORDS: usize = 5;

/// The words this tree borrows that are **not** an action's.
///
/// Three of the five come from `alo_shortcuts::Action::word` and are reached
/// through `Control::does`, so a test can find them without a list. These two
/// cannot: the approval answers are drawn by `alo-approving` and perform no
/// `Action`, so the only thing saying they are deliberately somebody else's is
/// this constant.
///
/// **A list rather than a prefix test**, because *any word from another crate
/// is fine* is not the rule. The rule is that a control is named by the thing
/// that draws it, and each borrowing is a decision somebody took — see
/// `Surface::Approval`'s own note. A word appearing here that nothing draws is
/// the fault ADR 0089 was written about, arriving from the other side.
/// **`cfg(test)` because the count above is the public statement** and this is
/// how it is checked. A second public list would be a second answer to *which
/// words does this tree borrow*, kept in step by hand.
#[cfg(test)]
const BORROWED_AND_NOT_AN_ACTION: [Word; 2] =
    [alo_approving::words::NO, alo_approving::words::APPROVE];

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every name a reader says is declared by somebody, and which somebody
    /// is a property of the control rather than of the word.**
    ///
    /// A control that performs an action is named by the action; everything
    /// else is named by one of this crate's own words. Nothing is named by
    /// both, and nothing is named by neither (ADR 0089).
    #[test]
    fn every_name_in_the_tree_is_a_word_somebody_declares() {
        let every_control: Vec<Control> = Surface::ALL
            .into_iter()
            .flat_map(Surface::read_aloud)
            .collect();

        let mut ours: Vec<String> = Vec::new();
        let mut theirs: Vec<String> = Vec::new();
        for control in &every_control {
            let key = control.name.key().to_string();
            match control.does {
                Some(action) => {
                    assert_eq!(
                        control.name.key().to_string(),
                        action.word().key().to_string(),
                        "{key} performs {action:?} and is named by something else"
                    );
                    theirs.push(key);
                }
                None => {
                    // **Borrowed first**, because a word this tree takes from
                    // the crate that draws it is somebody else's however it
                    // reaches the control — through `does` for an action, and
                    // through a deliberate list for the two that perform none.
                    if BORROWED_AND_NOT_AN_ACTION
                        .iter()
                        .any(|word| word.key().to_string() == key)
                    {
                        theirs.push(key);
                        continue;
                    }
                    assert!(
                        crate::words::EVERY_WORD
                            .iter()
                            .any(|word| word.key().to_string() == key),
                        "{key} is read aloud and nobody declares it"
                    );
                    ours.push(key);
                }
            }
        }
        for named in [&mut ours, &mut theirs] {
            named.sort();
            named.dedup();
        }
        assert_eq!(
            ours.len(),
            EVERY_NAME_A_READER_SAYS,
            "the tree says {} of this crate's words and the count says {EVERY_NAME_A_READER_SAYS}",
            ours.len()
        );
        assert_eq!(
            theirs.len(),
            EVERY_NAME_A_READER_SAYS_IN_ANOTHER_CRATES_WORDS,
            "the tree says {} of somebody else's words and the count says \
             {EVERY_NAME_A_READER_SAYS_IN_ANOTHER_CRATES_WORDS}",
            theirs.len()
        );
    }

    /// **A control is named by one thing**: the action it performs, the
    /// setting it is, or a word of this crate's — never two of them.
    ///
    /// The same shape `alo_applications::Called` took the same day, for the
    /// same reason: an exception that is visible in the type cannot be
    /// forgotten, and nothing may guess which kind a name is.
    #[test]
    fn a_control_is_named_by_one_thing() {
        for control in Surface::ALL.into_iter().flat_map(Surface::read_aloud) {
            assert!(
                !(control.does.is_some() && control.setting.is_some()),
                "{:?} both performs an action and is a setting",
                control.name.key()
            );
            if let Some(setting) = control.setting {
                assert_eq!(
                    control.name.key().to_string(),
                    setting.word().key().to_string(),
                    "a switch is named by its setting"
                );
            }
        }
    }

    /// **The window's buttons are the three a window is drawn with, in the
    /// order they are drawn in** — the set comparison that was missing.
    ///
    /// `alo-shell` draws `[minimise, maximise, close]` at x-offsets 0, 36 and
    /// 72 and this crate cannot see that list from here, so the comparison
    /// against the real strip lives in `alo-shell`'s own tests. What is held
    /// here is the half this crate can hold alone: the actions, and that each
    /// is a button a person can use.
    #[test]
    fn the_windows_buttons_are_the_three_that_are_drawn() {
        let controls = Surface::WindowControls.read_aloud();
        assert_eq!(
            controls
                .iter()
                .map(|control| control.does)
                .collect::<Vec<_>>(),
            vec![
                Some(Action::MinimiseWindow),
                Some(Action::MaximiseWindow),
                Some(Action::CloseWindow),
            ],
        );
        for control in controls {
            assert_eq!(control.role, Role::Button);
            assert_eq!(control.state, State::CanBeUsed);
        }
    }

    /// **Every surface is read as something, and every control is named.**
    #[test]
    fn every_surface_is_read_as_something_rather_than_nothing() {
        for surface in Surface::ALL {
            let controls = surface.read_aloud();
            assert!(!controls.is_empty(), "{surface:?} is read as nothing");
        }
    }

    /// **Anything a person has to fill in says what it wants** — clause 11.3.3.2.
    ///
    /// Asked of the fill-in roles by name rather than of every control, because
    /// the clause is about them: a list or a label being named says nothing about
    /// whether a person knows what to type. Today that is `Surface::SignIn`'s
    /// password, and the loop finds it rather than naming it, so a second field
    /// added anywhere is held to the same thing without this test being touched.
    #[test]
    fn everything_a_person_fills_in_says_what_it_wants() {
        let mut found = 0;
        for surface in Surface::ALL {
            for control in surface.read_aloud() {
                if !matches!(control.role, Role::Entry | Role::PasswordEntry) {
                    continue;
                }
                found += 1;
                assert!(
                    !control.name.says().trim().is_empty(),
                    "{surface:?} asks a person to fill something in and does not say what it wants"
                );
            }
        }
        assert!(
            found > 0,
            "no surface has a field to fill in, so this test proves nothing and the \
             clause it is cited by should be not-applicable instead"
        );
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
                // **The words the screen draws**, not this crate's. Until
                // 2026-10-05 these read `access.say-no` and
                // `access.approve-it`, and the two vocabularies were
                // translated independently with nothing comparing them.
                alo_approving::words::NO.key().to_string(),
                alo_approving::words::APPROVE.key().to_string(),
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
