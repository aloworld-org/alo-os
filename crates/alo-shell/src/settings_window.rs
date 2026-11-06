//! Settings, as one place: every setting this machine has today, in one
//! window, each section reading and writing through the crate that owns it.
//!
//! `ROADMAP.md` v0.5: *Settings, as one place … not a scattering of dialogues
//! a person has to know the name of.*
//!
//! # A side column of categories, and a page beside it
//!
//! The owner's redesign of 2026-10-31: the sections below are drawn on eight
//! pages, chosen in a side column (`crate::settings_categories` says which
//! section is on which page). The keyboard is in one of the two at a time —
//! Up and Down choose the page in the side column, and Tab or Enter go into
//! it — and `crate::settings_page` is the one list of what each page draws,
//! which this file's [`Open::rows_on`] is held to by a test. **No choice the
//! one-list window offered is dropped**: `no_choice_the_old_window_offered_is_dropped`
//! enumerates them by kind and finds each on its page.
//!
//! The sections, in the order the old one list drew them:
//!
//! 1. **what answers questions** — the four ways alo OS runs, as
//!    `alo-setting-up` offers them, and the choices the person's own
//!    `settings.toml` holds, changed through `alo_choosing::Choosing`
//!    (`crate::settings_answering`);
//! 2. **appearance** — the accent, through `alo_appearance::keeping`;
//!    **how dates and times are written** beneath it — the person's language,
//!    or a region they chose, through `alo_formats::keeping`
//!    (`crate::settings_formats`);
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
//! # Only what the machine has works, and the rest says so
//!
//! A row is something a person can act on: choose it, rebind it, revoke it.
//! Nothing is drawn disabled. Deep teal is not among the accents, because
//! `alo_appearance::Accent` cannot be deep teal. **The redesign draws a few
//! choices this computer does not have yet** — alo's own service, saying where
//! a provider runs — and they are drawn as the design draws them; choosing one
//! says in words that it is not on this computer yet and changes nothing. A
//! setting the redesign does not draw and the machine has no road to change
//! from a window — a background picture — is absent rather than greyed out.
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
use crate::settings_formats::FormatsSection;
use crate::settings_granted::{GrantedSection, SettingsRevoked};
use crate::settings_keepers::{AppearanceKept, DockKept, ShortcutsKept};
use crate::settings_kept::{KeptSection, SettingsKept};
use crate::settings_shell_integration::TheShellIntegration;
use crate::settings_time_zone::TimeZoneSection;
use crate::the_zone_through_timedated::SetsTheZone;
use crate::{SettingsKey, SettingsPlaces};
use alo_choosing::MayUseAnother;
use alo_setting_up::Offered;

pub use crate::settings_categories::SettingsCategory;
pub use crate::settings_picker::SettingsPicker;

/// The sections of Settings, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsSection {
    /// What answers this person's questions.
    Answering,
    /// How the machine looks.
    Appearance,
    /// How dates and times are written: the person's language, or a region
    /// they chose (`crate::settings_formats`).
    Formats,
    /// The time zone: the cities of the region dates are written like
    /// (`crate::settings_time_zone`), set through systemd's own service.
    TimeZone,
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
    /// Text size and the pointer's size.
    Accessibility,
}

/// Light, dark, or following the time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightOrDark {
    /// Light, whatever the time.
    Light,
    /// Dark, whatever the time.
    Dark,
    /// Dark in the evening and light again in the morning, by the schedule
    /// `alo-appearance` ships.
    FollowTheTimeOfDay,
}

impl LightOrDark {
    /// All three, in the order the frame draws them.
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::FollowTheTimeOfDay];

    /// Which one a person's appearance follows.
    #[must_use]
    pub const fn of(following: alo_appearance::Following) -> Self {
        match following {
            alo_appearance::Following::Always(alo_appearance::Scheme::Light) => Self::Light,
            alo_appearance::Following::Always(alo_appearance::Scheme::Dark) => Self::Dark,
            alo_appearance::Following::TheClock(_) => Self::FollowTheTimeOfDay,
        }
    }
}

/// How large the pointer is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerSize {
    /// As it ships.
    Default,
    /// Larger.
    Large,
    /// Larger still.
    Larger,
}

impl PointerSize {
    /// All three, in the order the frame draws them.
    pub const ALL: [Self; 3] = [Self::Default, Self::Large, Self::Larger];
}

/// One row a person can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsRow {
    /// A way to answer questions: Enter chooses it.
    Answer(SettingsChoice),
    /// An accent: Enter chooses it.
    Accent(Accent),
    /// How dates and times are written: Enter chooses it. [`None`] is *as my
    /// language writes them*; a code is the region whose way they follow.
    WrittenAs(Option<&'static str>),
    /// A time zone, by the zone database's name: Enter sets the machine to it.
    TimeZone(String),
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
    /// See what alo did: Enter opens the record.
    ///
    /// **In the section of what alo may reach**, above giving it more, because
    /// that is where a person already looks at what alo is allowed to do; and
    /// always offered, because the record answers *what happened* whether or
    /// not anything is granted now. The owner chose this row and Super+R on
    /// 2026-10-27. `crate::a_chord_opens_the_record` is the one opener.
    SeeWhatAloDid,
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
    /// is not available: one switch, and Enter turns it over.
    ///
    /// In the answering section, under the four ways, because it is a question
    /// about the choice they made there and arises only when they chose a
    /// provider or another machine.
    ///
    /// **Two rows until 2026-10-31**, one for each answer, because a person
    /// who has never said is neither — `alo-choosing`'s *absent is stronger
    /// than defaulted*. The redesign draws a switch, and the switch keeps that
    /// third state in words: *Not set* until the person says. Enter on *Not
    /// set* or *Off* turns it on; Enter on *On* turns it off.
    UsingAnother,
    /// One of the four ways a question can be answered, as a choice of its
    /// own: Enter goes to what is under it, or says why there is nothing.
    ///
    /// *Not at all* is never one of these: it needs nothing under it, so it is
    /// [`Self::Answer`] itself.
    Way(Offered),
    /// alo's own service, under the providers: Enter says it is not on this
    /// computer yet.
    AlosOwnService,
    /// Say where a provider runs: Enter says it is not on this computer yet.
    SayWhereItRuns(String),
    /// Which edge of the screen the Dock sits on: Enter chooses it.
    ///
    /// `docs/features.md`: *the person decides where it goes — bottom, left,
    /// right or top, chosen in Settings*, the promise the owner restored after
    /// ADR 0076 withdrew it.
    Edge(alo_dock::Edge),
    /// Another page of Settings, from a row that names it: Enter shows it.
    GoTo(SettingsCategory),
    /// Light, dark, or following the time of day: Enter chooses it.
    LightOrDark(LightOrDark),
    /// *Show the title only when I point at it*: one switch, and Enter turns
    /// it over.
    ///
    /// **Off as this release ships**, which is every window's title always on
    /// its edge — the owner's answer of 2026-11-01. On is the edge as it was
    /// drawn before that day, kept as a choice (`alo_appearance::WindowTitles`).
    WindowTitles,
    /// How large text is drawn everywhere: Left and Right make it smaller and
    /// larger, a step of 25 per cent at a time, and Enter makes it larger.
    TextSize,
    /// How large the pointer is drawn: Enter says it is not on this computer
    /// yet. `docs/features.md` promises *cursor size*; nothing draws it yet.
    PointerSize(PointerSize),
    /// The list of countries and regions, or of time zones: Enter opens it.
    ///
    /// **One row where there were two hundred and fifty.** Every region is
    /// still a row — [`Self::WrittenAs`] — in the list this opens, with a
    /// search at its top (`crate::settings_picker`).
    OpenThePicker(SettingsPicker),
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
            Self::WrittenAs(_) => SettingsSection::Formats,
            Self::TimeZone(_) => SettingsSection::TimeZone,
            Self::Hiding(_) | Self::Edge(_) => SettingsSection::Dock,
            Self::GoTo(_) => SettingsSection::Terminal,
            Self::LightOrDark(_) | Self::WindowTitles => SettingsSection::Appearance,
            Self::TextSize | Self::PointerSize(_) => SettingsSection::Accessibility,
            Self::OpenThePicker(SettingsPicker::Region) => SettingsSection::Formats,
            Self::OpenThePicker(SettingsPicker::TimeZone) => SettingsSection::TimeZone,
            Self::Shortcut(_) => SettingsSection::Shortcuts,
            Self::Granted(_) | Self::GrantAFolder | Self::SeeWhatAloDid | Self::Picking(_) => {
                SettingsSection::Granted
            }
            Self::UsingAnother | Self::Way(_) | Self::AlosOwnService | Self::SayWhereItRuns(_) => {
                SettingsSection::Answering
            }
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
    /// systemd's time-zone service, which the time-zone rows ask
    /// (`crate::the_zone_through_timedated`) — a trait, so every rule about
    /// what is set is tested without a bus.
    pub zone: &'a dyn SetsTheZone,
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
    /// Another page is shown, from a row that names it.
    WentTo(SettingsCategory),
    /// A list to choose from opened, or closed without a choice.
    TheList(SettingsPicker),
    /// Something the design draws and this computer does not have yet: the
    /// page says so, and nothing changed.
    NotOnThisComputerYet,
    /// A person asked to see the record, so the record window should open.
    ///
    /// **The window says so and does not do it**, for `AddingAProvider`'s
    /// reason: the record window lives on the `Server`.
    OpeningTheRecord,
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
    /// How dates and times are written.
    pub(crate) formats: FormatsSection,
    /// The time zone.
    pub(crate) time_zone: TimeZoneSection,
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
    /// The page the window is showing, which is the category chosen in the
    /// side column.
    pub(crate) category: SettingsCategory,
    /// Whether the keyboard is in the page rather than the side column.
    pub(crate) in_the_page: bool,
    /// The list of countries or time zones, while it is open.
    pub(crate) picker: Option<crate::settings_picker::APickerOpen>,
    /// What the page says about the last thing chosen on it, for a page
    /// with no section of its own to say it: something not on this computer
    /// yet. Gone when the page changes.
    pub(crate) told: Option<String>,
    /// Which row of the page has the focus, counted from its first.
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
            formats: FormatsSection::at_sign_in(
                places.kept(alo_formats::keeping::THE_FILE),
                places.choosing(),
            ),
            // **The cities are filled in by the host**, which knows the region
            // dates are written like; see [`SettingsWindow::the_zones_are_those_of`].
            time_zone: TimeZoneSection::of(
                None,
                alo_formats::THE_SOURCE_LANGUAGE,
                jiff::tz::TimeZone::system().iana_name().map(str::to_owned),
            ),
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
            // **It opens on the first page, with the keyboard in the side
            // column**, which is where a person chooses what they came for.
            category: SettingsCategory::AloAndAi,
            in_the_page: false,
            picker: None,
            told: None,
            focus: 0,
            waiting_for_a_chord: false,
        });
        SettingsOpened { not_remembered }
    }

    /// **What answers questions is read again**, because something outside
    /// this window just wrote it: a provider the person saved from the form on
    /// this page (`alo-desktop` makes those writes).
    ///
    /// Until 2026-11-01 the section was read only when Settings opened, so a
    /// provider saved here was not on the page — and was not chosen on it —
    /// until Settings was closed and opened again. Seen on a booted machine.
    ///
    /// **The keyboard stays on the row it was on**, found again by what the
    /// row is: the new provider adds a row above *Add a provider*, and a focus
    /// kept by number would have moved to a different row under the person's
    /// hand. Nothing else in the window is read again.
    pub fn what_answers_is_read_again(&mut self, places: &SettingsPlaces, now: SystemTime) {
        let was_on = self.focused(now);
        let Some(open) = self.open.as_mut() else {
            return;
        };
        open.answering = AnsweringSection::at_sign_in(places.choosing());
        let rows = open.rows_on(open.category, now);
        if let Some(at) = was_on.and_then(|row| rows.iter().position(|it| *it == row)) {
            open.focus = at;
        } else {
            open.focus = open.focus.min(rows.len().saturating_sub(1));
        }
    }

    /// **The time zones offered are those of the region `formats` writes
    /// dates like**, in the language its person reads — asked by the host when
    /// Settings opens and again when a region is kept, because the region is
    /// the one `crate::TheirFormats` answers, which this window does not hold.
    pub fn the_zones_are_those_of(&mut self, formats: &crate::TheirFormats) {
        if let Some(open) = self.open.as_mut() {
            let region = formats
                .the_region()
                .0
                .and_then(|it| alo_formats::RegionCode::named(&it).ok());
            open.time_zone.follows(region.as_ref(), formats.reading());
        }
    }

    /// The time zone the machine is in as Settings last set or read it.
    #[must_use]
    pub fn the_zone(&self) -> Option<String> {
        self.open
            .as_ref()
            .and_then(|open| open.time_zone.now().map(str::to_owned))
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

    /// Settings, while it is open, for the pointer to move the keyboard's place
    /// in it (`crate::settings_pointed_at`).
    pub(crate) const fn open_mut(&mut self) -> Option<&mut Open> {
        self.open.as_mut()
    }

    /// How the machine looks as Settings holds it — what the compositor draws
    /// from the moment a change is kept, because the two are one process.
    #[must_use]
    pub fn appearance(&self) -> Option<&Appearance> {
        self.open.as_ref().map(|open| open.appearance.drawn())
    }

    /// How dates and times are written, as Settings holds it — what the seat
    /// is told the moment a choice is kept.
    #[must_use]
    pub fn formats(&self) -> Option<&alo_formats::Chosen> {
        self.open.as_ref().map(|open| open.formats.drawn())
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

    /// When windows' titles are shown, as Settings holds it.
    #[must_use]
    pub fn window_titles(&self) -> Option<alo_appearance::WindowTitles> {
        self.open
            .as_ref()
            .map(|open| open.appearance.drawn().window_titles())
    }

    /// Every row a person can act on, in the order drawn. Empty while closed.
    #[must_use]
    pub fn rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        self.open
            .as_ref()
            .map(|open| open.rows(now))
            .unwrap_or_default()
    }

    /// The rows on the page the window is showing, in the order drawn.
    #[must_use]
    pub fn rows_on_the_page(&self, now: SystemTime) -> Vec<SettingsRow> {
        self.open
            .as_ref()
            .map(|open| open.rows_on(open.category, now))
            .unwrap_or_default()
    }

    /// The page the window is showing, while it is open.
    #[must_use]
    pub fn category(&self) -> Option<SettingsCategory> {
        self.open.as_ref().map(|open| open.category)
    }

    /// Whether the keyboard is in the page, rather than in the side column.
    #[must_use]
    pub fn is_in_the_page(&self) -> bool {
        self.open.as_ref().is_some_and(|open| open.in_the_page)
    }

    /// The row with the focus, while the keyboard is in the page and the page
    /// has any row.
    #[must_use]
    pub fn focused(&self, now: SystemTime) -> Option<SettingsRow> {
        let open = self.open.as_ref()?;
        if let Some(picker) = &open.picker {
            return picker.shown.get(picker.focus).cloned();
        }
        if !open.in_the_page {
            return None;
        }
        let rows = open.rows_on(open.category, now);
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
        if open.picker.is_some() {
            return Some(open.pressed_in_the_list(press, doors));
        }
        if !open.in_the_page {
            let did = open.pressed_in_the_side_column(press.key, doors);
            if did == Some(SettingsDid::Closed) {
                self.open = None;
            }
            return did;
        }
        let rows = open.rows_on(open.category, doors.now);
        let last = rows.len().saturating_sub(1);
        open.focus = open.focus.min(last);
        let focused = rows.get(open.focus).cloned();
        match press.key {
            SettingsKey::Previous => open.focus = open.focus.saturating_sub(1),
            SettingsKey::Next => open.focus = (open.focus + 1).min(last),
            SettingsKey::First => open.focus = 0,
            SettingsKey::Last => open.focus = last,
            // **Tab goes to the next group of rows on the page, and from the
            // last back to the side column**; Shift+Tab the other way. The
            // groups are the page's own sections, as they are drawn.
            SettingsKey::NextSection | SettingsKey::PreviousSection => {
                let starts = open.where_sections_start(doors.strings, doors.now);
                let forward = press.key == SettingsKey::NextSection;
                match next_section(&starts, open.focus, forward) {
                    Some(at) => open.focus = at,
                    None => open.in_the_page = false,
                }
            }
            SettingsKey::Choose => return focused.map(|row| open.chosen(&row, doors)),
            // **Left and Right on the text size make it smaller and larger**;
            // anywhere else they do nothing.
            SettingsKey::Less | SettingsKey::More => {
                if focused == Some(SettingsRow::TextSize) {
                    let by = if press.key == SettingsKey::More {
                        25
                    } else {
                        -25
                    };
                    return Some(open.the_text_is(by, doors.strings));
                }
            }
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
    /// Every row, in the order drawn: page by page in the order the side
    /// column lists them, and on each page in the order it draws them.
    ///
    /// **This list and `crate::settings_page` are two readers of one
    /// question**, and `every_setting_the_machine_has_is_in_one_window` holds
    /// them to each other: *a row is acted on that is not drawn, or drawn and
    /// not acted on*. It has caught a missing line three times.
    pub(crate) fn rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        SettingsCategory::ALL
            .into_iter()
            .flat_map(|category| self.rows_on(category, now))
            .collect()
    }

    /// The rows on one page, in the order it draws them.
    pub(crate) fn rows_on(&self, category: SettingsCategory, now: SystemTime) -> Vec<SettingsRow> {
        match category {
            SettingsCategory::AloAndAi => self.the_answering_rows(now),
            SettingsCategory::FoldersAloCanUse => self.the_granted_rows(now),
            SettingsCategory::RunningCode => alo_capability::HowCodeRuns::ALL
                .into_iter()
                .map(SettingsRow::HowCodeRuns)
                .collect(),
            SettingsCategory::Terminal => {
                let mut rows = Vec::new();
                if self.shell_integration.offers_a_row() {
                    rows.push(SettingsRow::ShellIntegration);
                }
                // **Beneath the integration**, because keeping a place needs
                // it: the line this row adds calls a command that asks the
                // service, and a person who has not added the integration has
                // not added the shell's side of any of this.
                if self.a_place_to_return_to.offers_a_row() {
                    rows.push(SettingsRow::APlaceToReturnTo);
                }
                rows.push(SettingsRow::GoTo(SettingsCategory::RunningCode));
                rows
            }
            SettingsCategory::Appearance => {
                let mut rows = Vec::new();
                if self.appearance.at().is_some() {
                    rows.extend(LightOrDark::ALL.into_iter().map(SettingsRow::LightOrDark));
                    rows.extend(Accent::ALL.into_iter().map(SettingsRow::Accent));
                    rows.push(SettingsRow::WindowTitles);
                }
                if self.dock.at().is_some() {
                    rows.extend(alo_dock::Edge::EVERY.into_iter().map(SettingsRow::Edge));
                    rows.extend(Hiding::ALL.into_iter().map(SettingsRow::Hiding));
                }
                rows
            }
            // **Two rows, each opening its list**; every region and zone is a
            // row of the list it opens.
            SettingsCategory::RegionAndTime => {
                let mut rows = Vec::new();
                if self.formats.is_offered() {
                    rows.push(SettingsRow::OpenThePicker(SettingsPicker::Region));
                }
                if !self.time_zone.choices().is_empty() {
                    rows.push(SettingsRow::OpenThePicker(SettingsPicker::TimeZone));
                }
                rows
            }
            SettingsCategory::KeyboardAndShortcuts => {
                if self.shortcuts.at().is_none() {
                    return Vec::new();
                }
                self.shortcuts
                    .drawn()
                    .bindings()
                    .map(|binding| SettingsRow::Shortcut(binding.action))
                    .collect()
            }
            // **Nothing yet.** The text size and the pointer's size the design
            // draws here are not reachable from this window; the page says so
            // in words rather than drawing controls that do nothing.
            SettingsCategory::Accessibility => {
                let mut rows = Vec::new();
                if self.appearance.at().is_some() {
                    rows.push(SettingsRow::TextSize);
                }
                rows.extend(PointerSize::ALL.into_iter().map(SettingsRow::PointerSize));
                rows
            }
        }
    }

    /// The alo and AI page's rows: each of the four ways and what is under it,
    /// then the switch about using another.
    fn the_answering_rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        let mut rows = Vec::new();
        let pairings = self.pairings();
        for (way, choices) in self.answering.every_way(&pairings, now) {
            if way == Offered::NotAtAll {
                rows.push(SettingsRow::Answer(SettingsChoice::NotAtAll));
                continue;
            }
            rows.push(SettingsRow::Way(way));
            for choice in choices {
                // **A provider that has not said where it runs has a button**,
                // and the keyboard reaches it straight after the provider.
                let unsaid = match &choice {
                    SettingsChoice::Provider { provider, .. } => self
                        .answering
                        .the_provider(provider)
                        .is_none_or(|it| matches!(it.region, alo_models::Region::Unknown))
                        .then(|| provider.clone()),
                    _ => None,
                };
                rows.push(SettingsRow::Answer(choice));
                rows.extend(unsaid.map(SettingsRow::SayWhereItRuns));
            }
            if way == Offered::FromAProvider {
                rows.push(SettingsRow::AlosOwnService);
                // **Always offered, and that is the point of it.** Every other
                // row here exists only for something the person already has; a
                // row that appeared only once somebody had a provider would be a
                // door that opens after you are through it.
                if self.answering.a_provider_can_be_added() {
                    rows.push(SettingsRow::AddAProvider);
                }
            }
        }
        if self.answering.another_is_a_question_at_all() {
            rows.push(SettingsRow::UsingAnother);
        }
        rows
    }

    /// The folders page's rows: what is given, the row that gives more, the
    /// record, and the computers paired with this one.
    fn the_granted_rows(&self, now: SystemTime) -> Vec<SettingsRow> {
        // **The picker takes the page over while it is open**, rather than
        // appearing above the list of grants: the key that opens a folder and
        // the key that revokes a grant would otherwise be one key a row apart.
        if let Some(picking) = self.granted.picking() {
            return picking
                .rows()
                .into_iter()
                .map(SettingsRow::Picking)
                .collect();
        }
        let (paired, given): (Vec<Row>, Vec<Row>) = self
            .granted
            .rows(now)
            .into_iter()
            .partition(|row| matches!(row, Row::Pairing(_)));
        let mut rows: Vec<SettingsRow> = given.into_iter().map(SettingsRow::Granted).collect();
        // **Always offered when it can be**: a row that appeared only once
        // somebody had a grant would be a door that opens after you are
        // through it.
        if self.granted.a_folder_can_be_given() {
            rows.push(SettingsRow::GrantAFolder);
        }
        rows.push(SettingsRow::SeeWhatAloDid);
        rows.extend(paired.into_iter().map(SettingsRow::Granted));
        rows
    }

    /// A key while a list is open: letters search, Backspace takes one back,
    /// Up and Down move, Enter chooses and closes it, Escape closes it.
    fn pressed_in_the_list(
        &mut self,
        press: SettingsPress,
        doors: SettingsDoors<'_>,
    ) -> SettingsDid {
        use crate::a_key_fills_in_a_provider::FillingInKey;
        let Some(picker) = self.picker.clone() else {
            return SettingsDid::StoppedWaiting;
        };
        match press.filling {
            FillingInKey::Typed(letter) => {
                let mut typed = picker.typed;
                typed.push(letter);
                self.the_search_is(typed, doors.strings);
                return SettingsDid::TheList(picker.which);
            }
            FillingInKey::RubOut => {
                let mut typed = picker.typed;
                typed.pop();
                self.the_search_is(typed, doors.strings);
                return SettingsDid::TheList(picker.which);
            }
            _ => {}
        }
        let last = picker.shown.len().saturating_sub(1);
        let focus = match press.key {
            SettingsKey::Previous => picker.focus.saturating_sub(1),
            SettingsKey::Next => (picker.focus + 1).min(last),
            SettingsKey::First => 0,
            SettingsKey::Last => last,
            SettingsKey::Choose => {
                self.picker = None;
                return match picker.shown.get(picker.focus) {
                    Some(row) => self.chosen(row, doors),
                    None => SettingsDid::TheList(picker.which),
                };
            }
            SettingsKey::Close => {
                self.picker = None;
                return SettingsDid::TheList(picker.which);
            }
            SettingsKey::NextSection
            | SettingsKey::PreviousSection
            | SettingsKey::PutBackThisOne
            | SettingsKey::PutBackAsShipped
            | SettingsKey::Less
            | SettingsKey::More
            | SettingsKey::Nothing => picker.focus,
        };
        if let Some(open) = self.picker.as_mut() {
            open.focus = focus;
        }
        SettingsDid::TheList(picker.which)
    }

    /// The text made `by` per cent larger, or smaller, within the range
    /// `alo-appearance` allows, through its keeper.
    pub(crate) fn the_text_is(&mut self, by: i32, strings: &Strings) -> SettingsDid {
        let (least, most) = alo_appearance::TextScale::range();
        let now = i32::from(self.appearance.drawn().text().as_percent());
        let to = (now + by).clamp(i32::from(least), i32::from(most));
        let to = u16::try_from(to).unwrap_or(least);
        let kept = self.appearance.changed(
            |appearance| {
                let scale = alo_appearance::TextScale::percent(to)
                    .map_err(|refused| refused.said(strings).into_text())?;
                appearance.set_text(scale);
                Ok(())
            },
            strings,
        );
        SettingsDid::Kept(SettingsSection::Appearance, kept)
    }

    /// Where each of the page's groups of rows begins, counted in its rows.
    fn where_sections_start(&self, strings: &Strings, now: SystemTime) -> Vec<usize> {
        let page = crate::settings_page::the_page(self, self.category, strings, now);
        let mut starts = Vec::new();
        let mut counted = 0;
        for block in &page.blocks {
            let here: usize = match block {
                crate::settings_page::ABlock::Section(section) => {
                    section.rows.iter().map(|row| row.targets().count()).sum()
                }
                crate::settings_page::ABlock::AnAction(_) => 1,
                crate::settings_page::ABlock::Said(_) => 0,
            };
            if here > 0 {
                starts.push(counted);
            }
            counted += here;
        }
        starts
    }

    /// A key while the keyboard is in the side column: Up and Down choose
    /// the page, Home and End the first and last, and Tab or Enter go into it.
    fn pressed_in_the_side_column(
        &mut self,
        key: SettingsKey,
        doors: SettingsDoors<'_>,
    ) -> Option<SettingsDid> {
        let before = self.category;
        match key {
            SettingsKey::Previous => self.category = self.category.previous(),
            SettingsKey::Next => self.category = self.category.next(),
            SettingsKey::First => self.category = SettingsCategory::AloAndAi,
            SettingsKey::Last => self.category = SettingsCategory::Accessibility,
            SettingsKey::NextSection | SettingsKey::Choose => {
                if !self.rows_on(self.category, doors.now).is_empty() {
                    self.in_the_page = true;
                    self.focus = 0;
                }
            }
            // **Shift+Tab from the side column goes to the page's last group**,
            // so Tab and Shift+Tab walk one ring in both directions.
            SettingsKey::PreviousSection => {
                if let Some(last) = self
                    .where_sections_start(doors.strings, doors.now)
                    .last()
                    .copied()
                {
                    self.in_the_page = true;
                    self.focus = last;
                }
            }
            SettingsKey::Close => {
                if self.granted.picking().is_some() {
                    self.granted.that_folder_is_not_being_given();
                    return Some(SettingsDid::Gave(
                        crate::settings_granted::SettingsGave::ItChanged,
                    ));
                }
                return Some(SettingsDid::Closed);
            }
            SettingsKey::PutBackThisOne
            | SettingsKey::PutBackAsShipped
            | SettingsKey::Less
            | SettingsKey::More
            | SettingsKey::Nothing => {}
        }
        if self.category != before {
            self.focus = 0;
            self.told = None;
        }
        None
    }

    /// The pairings as read, or none when they did not read.
    pub(crate) fn pairings(&self) -> Pairings {
        self.granted.pairings()
    }

    /// Enter on one of the four ways: **the keyboard goes to what is under
    /// it**, the choice the person has made there or the first; and a way with
    /// nothing under it says why, in words, and changes nothing.
    ///
    /// Choosing the way itself writes nothing, because a way is not an answer:
    /// *on this machine* is a model on this machine, and which one is the
    /// person's to say.
    fn a_way_chosen(&mut self, way: Offered, doors: SettingsDoors<'_>) -> SettingsDid {
        let rows = self.rows_on(self.category, doors.now);
        let here = rows.iter().position(|row| *row == SettingsRow::Way(way));
        let chosen = self.answering.chosen().map(SettingsRow::Answer);
        let under: Vec<usize> = here
            .map(|here| {
                rows.iter()
                    .enumerate()
                    .skip(here + 1)
                    .take_while(|(_, row)| {
                        matches!(
                            row,
                            SettingsRow::Answer(
                                SettingsChoice::Provider { .. }
                                    | SettingsChoice::Brought(_)
                                    | SettingsChoice::Paired(_)
                            ) | SettingsRow::SayWhereItRuns(_)
                        )
                    })
                    .filter(|(_, row)| matches!(row, SettingsRow::Answer(_)))
                    .map(|(at, _)| at)
                    .collect()
            })
            .unwrap_or_default();
        if let Some(first) = under.first() {
            self.focus = under
                .iter()
                .copied()
                .find(|at| rows.get(*at) == chosen.as_ref())
                .unwrap_or(*first);
            return SettingsDid::Answered(SettingsAnswered::NothingChanged);
        }
        use crate::the_settings_words as w;
        let why = match way {
            Offered::OnThisMachine => w::NO_MODEL_HERE_YET,
            Offered::OnAMachineOnThisNetwork => w::NOTHING_PAIRED_YET,
            Offered::FromAProvider | Offered::NotAtAll => w::NO_PROVIDER_YET,
        };
        self.answering
            .tell(crate::settings_page::said(doors.strings, why));
        SettingsDid::Answered(SettingsAnswered::NothingChanged)
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
            // **One switch, turned over.** *Not set* and *Off* turn it on; *On*
            // turns it off. A person who has never said is never written as
            // having said anything until they press it.
            SettingsRow::UsingAnother => {
                let said = match self.answering.may_use_another() {
                    Some(MayUseAnother::Yes) => MayUseAnother::No,
                    Some(MayUseAnother::No) | None => MayUseAnother::Yes,
                };
                SettingsDid::Answered(
                    self.answering
                        .say_whether_another_may_be_used(said, doors.strings),
                )
            }
            SettingsRow::Way(way) => self.a_way_chosen(*way, doors),
            SettingsRow::AlosOwnService | SettingsRow::SayWhereItRuns(_) => {
                self.answering.tell(crate::settings_page::said(
                    doors.strings,
                    crate::the_settings_words::NOT_ON_THIS_COMPUTER_YET,
                ));
                SettingsDid::Answered(SettingsAnswered::NothingChanged)
            }
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
            SettingsRow::WrittenAs(choice) => SettingsDid::Kept(
                SettingsSection::Formats,
                self.formats.choose(*choice, doors.strings),
            ),
            // **The service is asked, and the tick moves only on its yes.**
            SettingsRow::TimeZone(zone) => SettingsDid::Kept(
                SettingsSection::TimeZone,
                self.time_zone.choose(zone, doors.zone, doors.strings),
            ),
            SettingsRow::LightOrDark(which) => {
                let following = match which {
                    LightOrDark::Light => {
                        alo_appearance::Following::Always(alo_appearance::Scheme::Light)
                    }
                    LightOrDark::Dark => {
                        alo_appearance::Following::Always(alo_appearance::Scheme::Dark)
                    }
                    LightOrDark::FollowTheTimeOfDay => alo_appearance::Following::TheClock(
                        alo_appearance::Shipped::the_evening_schedule(),
                    ),
                };
                let kept = self.appearance.changed(
                    |appearance| {
                        appearance.follow(following);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Appearance, kept)
            }
            // **One switch, turned over**, and kept with the rest of the
            // appearance; the seat is told by `crate::a_key_reaches_settings`.
            SettingsRow::WindowTitles => {
                let turned = match self.appearance.drawn().window_titles() {
                    alo_appearance::WindowTitles::Always => {
                        alo_appearance::WindowTitles::WhenPointedAt
                    }
                    alo_appearance::WindowTitles::WhenPointedAt => {
                        alo_appearance::WindowTitles::Always
                    }
                };
                let kept = self.appearance.changed(
                    |appearance| {
                        appearance.set_window_titles(turned);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Appearance, kept)
            }
            SettingsRow::TextSize => self.the_text_is(25, doors.strings),
            SettingsRow::PointerSize(_) => {
                self.told = Some(crate::settings_page::said(
                    doors.strings,
                    crate::the_settings_words::NOT_ON_THIS_COMPUTER_YET,
                ));
                SettingsDid::NotOnThisComputerYet
            }
            SettingsRow::OpenThePicker(which) => {
                self.the_list_opens(*which, doors.strings);
                SettingsDid::TheList(*which)
            }
            SettingsRow::Edge(edge) => {
                let edge = *edge;
                let kept = self.dock.changed(
                    |dock| {
                        dock.set_edge(edge);
                        Ok(())
                    },
                    doors.strings,
                );
                SettingsDid::Kept(SettingsSection::Dock, kept)
            }
            // **The page it names is shown, with the keyboard on its first
            // row**, which is where a person reaches for it.
            SettingsRow::GoTo(category) => {
                self.category = *category;
                self.told = None;
                self.in_the_page = !self.rows_on(*category, doors.now).is_empty();
                self.focus = 0;
                SettingsDid::WentTo(*category)
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
            // **Nothing is read by pressing this**: it asks for the record
            // window, which reads the record when it opens.
            SettingsRow::SeeWhatAloDid => SettingsDid::OpeningTheRecord,
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
            SettingsSection::Formats => self.formats.put_back_as_shipped(strings),
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
            // **Not offered for the time zone**: it is the machine's, kept by
            // systemd rather than in a file of this person's, and there is no
            // release default to put back — a fresh disk keeps none.
            SettingsSection::Answering
            | SettingsSection::Granted
            | SettingsSection::Terminal
            | SettingsSection::HowCodeRuns
            | SettingsSection::Accessibility
            | SettingsSection::TimeZone => {
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
        let Some(SettingsRow::Shortcut(action)) = self
            .rows_on(self.category, doors.now)
            .get(self.focus)
            .cloned()
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

/// The first row of the group after the one `from` is in, or of the one
/// before it — [`None`] past either end, which is the side column.
///
/// `starts` is where each group begins, in order. Backwards from a row inside
/// a group goes to that group's own first row first, the way Shift+Tab in any
/// list returns to the start of the thing it is in.
fn next_section(starts: &[usize], from: usize, forward: bool) -> Option<usize> {
    if forward {
        starts.iter().copied().find(|start| *start > from)
    } else {
        starts.iter().rev().copied().find(|start| *start < from)
    }
}

#[cfg(test)]
#[path = "settings_window_tests.rs"]
mod tests;
