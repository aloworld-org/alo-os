//! Settings, as one place: every setting this machine has today, in one
//! window, each section reading and writing through the crate that owns it.
//!
//! `ROADMAP.md` v0.5: *Settings, as one place … not a scattering of dialogues
//! a person has to know the name of.* The sections, in the order drawn:
//!
//! 1. **what answers questions** — the four ways alo OS runs, as
//!    `alo-setting-up` offers them, and the choices the person's own
//!    `settings.toml` holds, changed through `alo_choosing::Choosing`
//!    (`crate::settings_answering`);
//! 2. **appearance** — the accent, through `alo_appearance::keeping`;
//! 3. **the dock** — its edge, through `alo_dock::keeping`;
//! 4. **shortcuts** — each action's chord, through `alo_shortcuts::keeping`;
//! 5. **what has been granted to what** — the row that gives alo a folder,
//!    then the grants and the pairings in one list, revoked with
//!    `alo_changing::Changing::revoked` (`crate::settings_granted`). The row
//!    opens `crate::a_folder_is_picked_in_settings`, which is the one way a
//!    grant is made on this machine.
//!
//! # This window decides nothing
//!
//! Every value it shows is a crate's, every change goes through that crate,
//! and every sentence is that crate's. A change is made on a copy and only a
//! copy the owning crate kept replaces what is drawn, so what a person sees
//! and what the machine enforces cannot disagree. `tests/settings_source.rs`
//! holds these files to writing no file and naming no place of their own.
//!
//! # Only what the machine has is here
//!
//! A row is something a person can act on: choose it, rebind it, revoke it.
//! Nothing is drawn disabled. Deep teal is not among the accents, because
//! `alo_appearance::Accent` cannot be deep teal. A setting the machine has
//! no road to change from a window yet — a background picture, a text scale,
//! adding a provider — is absent rather than greyed out.
//!
//! # It is a person's, and it is reached by hand
//!
//! [`SettingsWindow::opened_by_hand`] takes where things are kept and a moment,
//! and nothing else: no turn, no model and no agent is involved or reachable
//! from these files, so nothing here is a road an agent could take to change a
//! setting, grant anything or keep a grant it was about to lose.

use std::time::SystemTime;

use alo_appearance::{Accent, Appearance};
use alo_changing::{Door, Row};
use alo_dock::{Dock, Hiding};
use alo_nearby::Pairings;
use alo_remembering::NotRemembered;
use alo_shortcuts::{Action, Chord, Key, Modifiers, Shortcuts};
use alo_strings::Strings;

use crate::a_folder_is_picked_in_settings::{Folders, PickingRow};
use crate::how_code_runs_in_settings::HowCodeRunsHere;
use crate::settings_answering::{AnsweringSection, SettingsAnswered, SettingsChoice};
use crate::settings_granted::{GrantedSection, SettingsRevoked};
use crate::settings_keepers::{AppearanceKept, DockKept, ShortcutsKept};
use crate::settings_kept::{KeptSection, SettingsKept};
use crate::settings_shell_integration::TheShellIntegration;
use crate::{SettingsKey, SettingsPlaces};
use alo_choosing::MayUseAnother;

/// The sections of Settings, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsSection {
    /// What answers this person's questions.
    Answering,
    /// How the machine looks.
    Appearance,
    /// Which edge the dock is on.
    Dock,
    /// The shortcuts.
    Shortcuts,
    /// The terminal: today, whether alo may read what a person types in it.
    Terminal,
    /// How code the agent writes or fetches runs: ADR 0064's three levels.
    ///
    /// Above *what has been granted to what* because this **is** a grant — the
    /// separate one that ADR 0064 makes running code — and a person reads the
    /// grant they are giving before the list of ones they have given.
    HowCodeRuns,
    /// What has been granted to what.
    Granted,
}

/// One row a person can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsRow {
    /// A way to answer questions: Enter chooses it.
    Answer(SettingsChoice),
    /// An accent: Enter chooses it.
    Accent(Accent),
    /// What the dock does when a window needs the room: Enter chooses it.
    ///
    /// This was an edge for the dock until ADR 0076 fixed the dock to the bottom
    /// edge. The section did not become empty — it became the one question the
    /// dock actually asks, and the strings for these two rows were already
    /// declared with nothing drawing them.
    Hiding(Hiding),
    /// An action's shortcut: Enter waits for a chord, Backspace puts it back.
    Shortcut(Action),
    /// A grant or a pairing: Enter revokes it.
    Granted(Row),
    /// Give alo a folder: Enter opens the picker.
    ///
    /// **One row carrying nothing**, which is [`Self::AddAProvider`]'s shape
    /// and for its reason: there is one of these, and what it acts on is the
    /// section it is in. Until it existed this window was right in the grants
    /// section's own words — *nothing is granted from here* — and so nothing
    /// on the machine could grant alo a folder at all.
    GrantAFolder,
    /// One row of the picker, while a person is in it: Enter acts on it.
    ///
    /// The moves are rows rather than new keys, so the picker is worked with
    /// the four this window already has.
    /// `crate::a_folder_is_picked_in_settings` says why.
    Picking(PickingRow),
    /// alo's shell integration: Enter adds it to the person's shell startup
    /// file, and says so. There is one of these rows, so it carries nothing —
    /// what it would act on is the section's own answer.
    ShellIntegration,
    /// Add a provider: Enter opens the form that asks for its name, its
    /// address, its key and the model to ask for.
    ///
    /// **One row carrying nothing**, which is `ShellIntegration`'s shape and
    /// for its reason: there is one of these, and what it acts on is the
    /// section it is in. Until it existed `settings_answering` was right in
    /// its own words -- *adding a provider … is not reachable from this list*
    /// -- and a person on a fresh machine could not make it answer them.
    AddAProvider,
    /// Keep a place to return to before each command (ADR 0130).
    APlaceToReturnTo,
    /// Whether alo may use another model or API when the one the person chose
    /// is not available: Enter chooses it.
    ///
    /// In the answering section, under the four ways, because it is a question
    /// about the choice they made there and arises only when they chose a
    /// provider or another machine.
    UsingAnother(MayUseAnother),
    /// How code the agent writes or fetches runs: Enter chooses this level.
    ///
    /// One row per level of ADR 0064, in that ADR's own order, which is least
    /// to most the agent may do. **A person who has not chosen has none of the
    /// three ticked**, and `crate::how_code_runs_in_settings` holds why that is
    /// a state of its own rather than the sealed box.
    HowCodeRuns(alo_capability::HowCodeRuns),
}

impl SettingsRow {
    /// The section the row is in.
    #[must_use]
    pub const fn section(&self) -> SettingsSection {
        match self {
            Self::Answer(_) => SettingsSection::Answering,
            Self::Accent(_) => SettingsSection::Appearance,
            Self::Hiding(_) => SettingsSection::Dock,
            Self::Shortcut(_) => SettingsSection::Shortcuts,
            Self::Granted(_) | Self::GrantAFolder | Self::Picking(_) => SettingsSection::Granted,
            Self::UsingAnother(_) => SettingsSection::Answering,
            Self::HowCodeRuns(_) => SettingsSection::HowCodeRuns,
            Self::ShellIntegration | Self::APlaceToReturnTo => SettingsSection::Terminal,
            Self::AddAProvider => SettingsSection::Answering,
        }
    }
}

/// One key press as Settings receives it: what the key means, and the chord it
/// makes, for the moment Settings is waiting for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsPress {
    /// What the key means.
    pub key: SettingsKey,
    /// The modifiers held and the key pressed, when the key is one a shortcut
    /// can be made of.
    pub chord: Option<(Modifiers, Key)>,
    /// What the same press means while a person is filling in a provider.
    ///
    /// **Read off the same press rather than translated again**, which is
    /// `crate::a_key_reaches_settings`' own rule one level down: *one reading
    /// of what libinput said … so a machine cannot disagree with itself about
    /// which key a person pressed depending on what is on the screen.* Two
    /// readings taken at two moments could answer about two different keys on
    /// a keyboard whose layout changed between them.
    ///
    /// Which of these three is acted on is decided by whether a form is open,
    /// and nothing here decides it.
    pub filling: crate::a_key_fills_in_a_provider::FillingInKey,
}

/// What a key press reaches outside the window, handed in for that press.
#[derive(Clone, Copy)]
pub struct SettingsDoors<'a> {
    /// The vocabulary the person reads, which words every refusal.
    pub strings: &'a Strings,
    /// The daemon's door, which hears a grant's revocation and revokes a
    /// pairing.
    pub daemon: &'a dyn Door,
    /// The one road to a disk, which the picker walks.
    ///
    /// **A trait rather than the filesystem**, which is `alo-picking`'s own
    /// division and is why every rule about what a person may pick is tested
    /// against a folder tree written down in a test. It is also the only thing
    /// in this window that reads anything outside the person's own settings.
    pub folders: &'a dyn Folders,
    /// The moment of the press, for the grants and pairings in force.
    pub now: SystemTime,
}

/// What a key press did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsDid {
    /// A section kept in the person's folder was changed, or was not.
    Kept(SettingsSection, SettingsKept),
    /// What answers questions was chosen, or was not.
    Answered(SettingsAnswered),
    /// A row of the one list was revoked, or was not.
    Revoked(SettingsRevoked),
    /// Settings waits for the chord to bind the focused action to.
    WaitingForAChord,
    /// Settings stopped waiting for a chord without binding one.
    StoppedWaiting,
    /// The window closed.
    Closed,
    /// A person moved through the picker, or gave alo a folder.
    Gave(crate::settings_granted::SettingsGave),
    /// A person asked to add a provider, so a form should open.
    ///
    /// **The window says so and does not do it**: the form lives on the
    /// `Server` because the keys that fill it in reach the seat, not this
    /// window. See `crate::a_provider_is_added_in_settings`.
    AddingAProvider,
}

/// What opening Settings read, for the host's log: every file of the
/// machine's own that did not read. A section whose own file did not read says
/// so in the window instead.
#[derive(Debug)]
pub struct SettingsOpened {
    /// The grants and pairings files that are there and did not read.
    pub not_remembered: Vec<NotRemembered>,
}

/// Settings, while it is open.
#[derive(Debug)]
pub(crate) struct Open {
    /// What answers questions.
    pub(crate) answering: AnsweringSection,
    /// Appearance.
    pub(crate) appearance: KeptSection<AppearanceKept>,
    /// The dock.
    pub(crate) dock: KeptSection<DockKept>,
    /// Shortcuts.
    pub(crate) shortcuts: KeptSection<ShortcutsKept>,
    /// Grants and pairings.
    pub(crate) granted: GrantedSection,
    /// Whether alo's shell integration is in the person's startup file.
    pub(crate) shell_integration: TheShellIntegration,
    /// What Settings found about keeping a place to return to.
    pub(crate) a_place_to_return_to: TheShellIntegration,
    /// How code the agent writes or fetches runs, as this machine's setting
    /// says — including *nobody has chosen*, which is why this is not a
    /// `KeptSection`. `crate::how_code_runs_in_settings` holds that argument.
    pub(crate) how_code_runs: HowCodeRunsHere,
    /// Which row has the focus, counted from the first.
    pub(crate) focus: usize,
    /// Whether Settings waits for a chord for the focused shortcut.
    pub(crate) waiting_for_a_chord: bool,
}

/// The Settings window.
#[derive(Debug, Default)]
pub struct SettingsWindow {
    /// The sections, while the window is open.
    open: Option<Open>,
}

impl SettingsWindow {
    /// The window, closed.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// A person opened Settings, without asking any agent anything.
    ///
    /// Every section is read once, here, from where `places` says it is kept.
    /// Reopening an open window reads everything again.
    pub fn opened_by_hand(&mut self, places: &SettingsPlaces, now: SystemTime) -> SettingsOpened {
        let (granted, not_remembered) = GrantedSection::at_sign_in(
            places.grants(),
            places.pairings(),
            places.home(),
            places.agent(),
            now,
        );
        self.open = Some(Open {
            answering: AnsweringSection::at_sign_in(places.choosing()),
            appearance: KeptSection::at_sign_in(places.kept(alo_appearance::keeping::THE_FILE)),
            dock: KeptSection::at_sign_in(places.kept(alo_dock::keeping::THE_FILE)),
            shortcuts: KeptSection::at_sign_in(places.kept(alo_shortcuts::keeping::THE_FILE)),
            granted,
            shell_integration: TheShellIntegration::at_sign_in(places.shell(), places.folder()),
            a_place_to_return_to: TheShellIntegration::about(
                alo_terminal::where_the_integration_stands::WhichOne::APlaceToReturnTo,
                places.shell(),
                places.folder(),
            ),
            how_code_runs: HowCodeRunsHere::at_sign_in(places.how_code_runs()),
            focus: 0,
            waiting_for_a_chord: false,
        });
        SettingsOpened { not_remembered }
    }

    /// Close the window. Nothing is written by closing it.
    pub fn close(&mut self) {
        self.open = None;
    }

    /// Whether the window is open, so the host routes keys here.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Settings, while it is open.
    pub(crate) const fn open(&self) -> Option<&Open> {
        self.open.as_ref()
    }

    /// How the machine looks as Settings holds it — what the compositor draws
    /// from the moment a change is kept, because the two are one process.
    #[must_use]
    pub fn appearance(&self) -> Option<&Appearance> {
        self.open.as_ref().map(|open| open.appearance.drawn())
    }

    /// Where the dock is as Settings holds it.
    #[must_use]
    pub fn dock(&self) -> Option<&Dock> {
        self.open.as_ref().map(|open| open.dock.drawn())
    }

    /// The shortcuts as Settings holds them.
    #[must_use]
    pub fn shortcuts(&self) -> Option<&Shortcuts> {
        self.open.as_ref().map(|open| open.shortcuts.drawn())
    }

    /// Every row a person can act on, in the order drawn. Empty while closed.
    #[must_use]
    pub fn rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        self.open
            .as_ref()
            .map(|open| open.rows(now))
            .unwrap_or_default()
    }

    /// The row with the focus, while the window is open and has any row.
    #[must_use]
    pub fn focused(&self, now: SystemTime) -> Option<SettingsRow> {
        let open = self.open.as_ref()?;
        let rows = open.rows(now);
        rows.get(open.focus.min(rows.len().saturating_sub(1)))
            .cloned()
    }

    /// Whether Settings waits for a chord.
    #[must_use]
    pub fn is_waiting_for_a_chord(&self) -> bool {
        self.open
            .as_ref()
            .is_some_and(|open| open.waiting_for_a_chord)
    }

    /// One key press. Nothing happens while the window is closed.
    pub fn pressed(
        &mut self,
        press: SettingsPress,
        doors: SettingsDoors<'_>,
    ) -> Option<SettingsDid> {
        let open = self.open.as_mut()?;
        if open.waiting_for_a_chord {
            return Some(open.chord_pressed(press, doors));
        }
        let rows = open.rows(doors.now);
        let last = rows.len().saturating_sub(1);
        open.focus = open.focus.min(last);
        let focused = rows.get(open.focus).cloned();
        match press.key {
            SettingsKey::Previous => open.focus = open.focus.saturating_sub(1),
            SettingsKey::Next => open.focus = (open.focus + 1).min(last),
            SettingsKey::First => open.focus = 0,
            SettingsKey::Last => open.focus = last,
            SettingsKey::NextSection => open.focus = next_section(&rows, open.focus, true),
            SettingsKey::PreviousSection => open.focus = next_section(&rows, open.focus, false),
            SettingsKey::Choose => return focused.map(|row| open.chosen(&row, doors)),
            SettingsKey::PutBackAsShipped => {
                return focused.and_then(|row| open.put_back_as_shipped(&row, doors.strings));
            }
            SettingsKey::PutBackThisOne => {
                if let Some(SettingsRow::Shortcut(action)) = focused {
                    let kept = open.shortcuts.changed(
                        |shortcuts| {
                            shortcuts.reset(action);
                            Ok(())
                        },
                        doors.strings,
                    );
                    return Some(SettingsDid::Kept(SettingsSection::Shortcuts, kept));
                }
            }
            SettingsKey::Close => {
                // **Escape gives up on the picker and leaves Settings open.**
                // `crate::a_key_fills_in_a_provider` keeps the same rule for
                // the same reason: a person pressing Escape to get out of a
                // folder chooser has not asked to throw away everything else
                // they came here for.
                if let Some(open) = &mut self.open
                    && open.granted.picking().is_some()
                {
                    open.granted.that_folder_is_not_being_given();
                    return Some(SettingsDid::Gave(
                        crate::settings_granted::SettingsGave::ItChanged,
                    ));
                }
                self.open = None;
                return Some(SettingsDid::Closed);
            }
            SettingsKey::Nothing => {}
        }
        None
    }
}

impl Open {
    /// Every row, in the order drawn.
    pub(crate) fn rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        let mut rows = Vec::new();
        let pairings = self.pairings();
        for (_, choices) in self.answering.offered(&pairings, now) {
            rows.extend(choices.into_iter().map(SettingsRow::Answer));
        }
        // **Always offered, and that is the point of it.** Every other row in
        // this section exists only for something the person already has, which
        // is why a fresh machine offered *not at all* and nothing else — and
        // why `settings_answering` could say truthfully that adding a provider
        // was reachable from nowhere. A row that appeared only once somebody
        // had a provider would be a door that opens after you are through it.
        if self.answering.a_provider_can_be_added() {
            rows.push(SettingsRow::AddAProvider);
        }
        // **Beneath the four, and in the same order they are drawn in.** This
        // list and `crate::settings_lines` are two readers of one question, and
        // `every_setting_the_machine_has_is_in_one_window` holds them to each
        // other: *a row is acted on that is not drawn, or drawn and not acted
        // on*. It caught this line being missing.
        if self.answering.another_is_a_question_at_all() {
            rows.extend(
                MayUseAnother::BOTH
                    .into_iter()
                    .map(SettingsRow::UsingAnother),
            );
        }
        if self.appearance.at().is_some() {
            rows.extend(Accent::ALL.into_iter().map(SettingsRow::Accent));
        }
        if self.dock.at().is_some() {
            rows.extend(Hiding::ALL.into_iter().map(SettingsRow::Hiding));
        }
        if self.shortcuts.at().is_some() {
            rows.extend(
                self.shortcuts
                    .drawn()
                    .bindings()
                    .map(|binding| SettingsRow::Shortcut(binding.action)),
            );
        }
        // **Beneath the shortcuts and above the grants**, which is where
        // `crate::settings_lines` draws the section. The two are held to each
        // other by `every_setting_the_machine_has_is_in_one_window`.
        if self.shell_integration.offers_a_row() {
            rows.push(SettingsRow::ShellIntegration);
        }
        // **Beneath the integration**, because keeping a place needs it: the
        // line this row adds calls a command that asks the service, and a
        // person who has not added the integration above has not added the
        // shell's side of any of this. The order says so without a sentence.
        if self.a_place_to_return_to.offers_a_row() {
            rows.push(SettingsRow::APlaceToReturnTo);
        }
        // **The three levels, always all three.** Unlike every section above,
        // this one has no file to be missing: a person who has not chosen is
        // offered the choice, which is the whole point of the section, and the
        // file that would not read is the same three rows with none marked.
        //
        // **This was drawn and not acted on for one commit**, and
        // `every_setting_the_machine_has_is_in_one_window` caught it in under a
        // second — *a row is acted on that is not drawn, or drawn and not acted
        // on*. A person would have seen three levels and pressing one would
        // have done nothing. The comment two blocks up records the same test
        // catching the same class before; this is the third time it has paid
        // for itself.
        rows.extend(
            alo_capability::HowCodeRuns::ALL
                .into_iter()
                .map(SettingsRow::HowCodeRuns),
        );
        // **The picker takes the section over while it is open**, rather than
        // appearing above the list of grants. A person walking their own
        // folders with the grants they already have underneath would be moving
        // the ring through two unrelated lists, and the key that opens a folder
        // and the key that revokes a grant would be the same key a row apart.
        if let Some(picking) = self.granted.picking() {
            rows.extend(picking.rows().into_iter().map(SettingsRow::Picking));
            return rows;
        }
        // **Above the list, and always offered when it can be.** It is the one
        // row in this section that is not something the person already has —
        // the same argument `AddAProvider` makes a hundred lines up, and the
        // same fault it fixes: a row that appeared only once somebody had a
        // grant would be a door that opens after you are through it.
        if self.granted.a_folder_can_be_given() {
            rows.push(SettingsRow::GrantAFolder);
        }
        rows.extend(self.granted.rows(now).into_iter().map(SettingsRow::Granted));
        rows
    }

    /// The pairings as read, or none when they did not read.
    pub(crate) fn pairings(&self) -> Pairings {
        self.granted.pairings()
    }

    /// Enter on a row.
    fn chosen(&mut self, row: &SettingsRow, doors: SettingsDoors<'_>) -> SettingsDid {
        match row {
            SettingsRow::Answer(choice) => {
                let pairings = self.pairings();
                SettingsDid::Answered(self.answering.choose(
                    choice,
                    &pairings,
                    doors.now,
                    doors.strings,
                ))
            }
            SettingsRow::UsingAnother(said) => SettingsDid::Answered(
                self.answering
                    .say_whether_another_may_be_used(*said, doors.strings),
            ),
            SettingsRow::Accent(accent) => {
                let accent = *accent;
                let kept = self.appearance.changed(
                    |appearance| {
                        appearance.set_accent(accent);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Appearance, kept)
            }
            SettingsRow::Hiding(hiding) => {
                let hiding = *hiding;
                let kept = self.dock.changed(
                    |dock| {
                        dock.set_hiding(hiding);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Dock, kept)
            }
            SettingsRow::Shortcut(_) => {
                self.waiting_for_a_chord = true;
                SettingsDid::WaitingForAChord
            }
            SettingsRow::Granted(row) => SettingsDid::Revoked(self.granted.revoked(
                row,
                doors.daemon,
                doors.now,
                doors.strings,
            )),
            // **Answered rather than kept.** Nothing is written by pressing
            // this: it opens a form, and what a person types in it is saved
            // when they press Enter again. A `Kept` here would tell the
            // section a change had landed that has not been made yet.
            SettingsRow::AddAProvider => SettingsDid::AddingAProvider,
            // **Nothing is granted by pressing this**: it opens the picker.
            // The grant is made further in, when the person has chosen a
            // folder *and* said how long — two answers, and this row is
            // neither of them.
            SettingsRow::GrantAFolder => {
                SettingsDid::Gave(self.granted.give_alo_a_folder(doors.folders, doors.strings))
            }
            SettingsRow::Picking(picking) => SettingsDid::Gave(self.granted.act_on(
                picking,
                doors.folders,
                doors.daemon,
                doors.now,
                doors.strings,
            )),
            SettingsRow::ShellIntegration => SettingsDid::Kept(
                SettingsSection::Terminal,
                self.shell_integration.add(doors.strings),
            ),
            SettingsRow::APlaceToReturnTo => SettingsDid::Kept(
                SettingsSection::Terminal,
                self.a_place_to_return_to.add(doors.strings),
            ),
            // **The level is written before the tick moves.** A tick that moved
            // first would tell somebody their machine runs code at a level it
            // does not, which on this setting is the difference between a
            // sealed box and full trust.
            SettingsRow::HowCodeRuns(level) => SettingsDid::Kept(
                SettingsSection::HowCodeRuns,
                self.how_code_runs.picked(*level),
            ),
        }
    }

    /// Delete on a row: its section put back as shipped, when its file did
    /// not read.
    fn put_back_as_shipped(&mut self, row: &SettingsRow, strings: &Strings) -> Option<SettingsDid> {
        let section = row.section();
        let kept = match section {
            SettingsSection::Appearance => self.appearance.put_back_as_shipped(strings),
            SettingsSection::Dock => self.dock.put_back_as_shipped(strings),
            SettingsSection::Shortcuts => self.shortcuts.put_back_as_shipped(strings),
            // **Not offered for the terminal, and never will be.** The file
            // this section appends to is the person's own shell startup, not a
            // file of alo's: putting it back as alo ships it would throw away
            // whatever else is in it. `settings_shell_integration`'s header
            // says the same thing where somebody would look for it.
            // **Not offered for how code runs either, and for a sharper
            // reason.** Putting this back as shipped would mean writing a file
            // saying the sealed box — which is not what the release ships. The
            // release ships *no file*, because nobody has chosen, and there is
            // no way to put a person back into *not having chosen* that is
            // honest once they have: deleting their file would make the machine
            // claim they never answered. `crate::how_code_runs_in_settings`
            // carries the whole argument.
            SettingsSection::Answering
            | SettingsSection::Granted
            | SettingsSection::Terminal
            | SettingsSection::HowCodeRuns => {
                return None;
            }
        };
        Some(SettingsDid::Kept(section, kept))
    }

    /// A key while Settings waits for a chord: Escape stops waiting, Backspace
    /// on its own leaves the action with no shortcut, and anything else is the
    /// chord — refused in `alo-shortcuts`' words when it cannot be one, or
    /// when another action has it.
    fn chord_pressed(&mut self, press: SettingsPress, doors: SettingsDoors<'_>) -> SettingsDid {
        let Some(SettingsRow::Shortcut(action)) = self.rows(doors.now).get(self.focus).cloned()
        else {
            self.waiting_for_a_chord = false;
            return SettingsDid::StoppedWaiting;
        };
        let Some((held, key)) = press.chord else {
            if press.key == SettingsKey::Close {
                self.waiting_for_a_chord = false;
                return SettingsDid::StoppedWaiting;
            }
            return SettingsDid::WaitingForAChord;
        };
        if held.is_empty() && key == Key::Escape {
            self.waiting_for_a_chord = false;
            return SettingsDid::StoppedWaiting;
        }
        self.waiting_for_a_chord = false;
        let strings = doors.strings;
        let kept = if held.is_empty() && key == Key::Backspace {
            self.shortcuts.changed(
                |shortcuts| {
                    shortcuts.unbind(action);
                    Ok(())
                },
                strings,
            )
        } else {
            self.shortcuts.changed(
                |shortcuts| {
                    let chord = Chord::checked(held, key)
                        .map_err(|refused| refused.said(strings).into_text())?;
                    shortcuts
                        .bind(action, chord)
                        .map_err(|taken| taken.said(strings).into_text())
                },
                strings,
            )
        };
        SettingsDid::Kept(SettingsSection::Shortcuts, kept)
    }
}

/// The first row of the next section after `from`, or of the one before.
fn next_section(rows: &[SettingsRow], from: usize, forward: bool) -> usize {
    let Some(here) = rows.get(from).map(SettingsRow::section) else {
        return from;
    };
    if forward {
        rows.iter()
            .enumerate()
            .skip(from)
            .find(|(_, row)| row.section() > here)
            .map_or(from, |(at, _)| at)
    } else {
        let Some(before) = rows
            .iter()
            .take(from)
            .rev()
            .map(SettingsRow::section)
            .find(|section| *section < here)
        else {
            return 0;
        };
        rows.iter()
            .position(|row| row.section() == before)
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "settings_window_tests.rs"]
mod tests;
