//! The twenty-four languages as rows a person can move through and pick one
//! of, and the keyboard that comes with each.
//!
//! The whole list, a cursor in it, and a window onto it.
//!
//! # Why a scroll and not a search
//!
//! A picker in the design file shows **three** rows and there are
//! **twenty-four** languages, so something has to turn 24 into 3. The design
//! does not say which of a scroll, a page or a search it means, and on this
//! screen that is not a free choice: **a search box needs typing, and this is
//! the screen that chooses the keyboard.** A fresh machine has whatever its
//! unit file named — `us` until somebody says otherwise — and a Greek speaker
//! cannot type `Ελληνικά` on a US layout at all, because those characters are
//! not on it. A search would be unusable by exactly the people it would help
//! most.
//!
//! So: a window that moves with the cursor, and **everything reachable by the
//! arrow keys alone**. `the_window_shows_three_rows_and_the_cursor_is_always_in_it`
//! and `the_window_reaches_the_last_language_and_the_cursor_stops_there` hold
//! that, because a list that cannot reach its twenty-fourth row offers
//! twenty-one languages to nobody.
//!
//! **What the design settles and what it does not.**
//! `docs/design/the-first-start.md` specifies scrolling, and it is worth
//! being exact about which problem that answers: it is the **small-screen**
//! mechanism, measured at 1280×720 — *tighten* first, then *scroll* the
//! content in a 640-wide region. Twenty-four languages in three rows is a
//! different problem and that passage does not answer it. What rules out a
//! search is the paragraph above, which stands on its own.
//!
//! **And when a screen does take the scroll mechanism, the actions move.**
//! *The actions and `Accessibility` move outside the scrolling region* so
//! they stay reachable without scrolling, at y 812 and y 884 rather than
//! inside the column. `crate::setup_sheet_layout` puts them inside the
//! column, which is right at the reference size and wrong at the small one —
//! recorded here because a build that scrolls this list without moving them
//! would put the way forward out of reach.
//!
//! # A cursor is not a choice
//!
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! and ADR 0014 §4 say nothing is pre-selected at first start, and
//! `docs/features.md` adds that **moving focus must never silently commit a
//! choice**. So there are two positions here and they are different things:
//! the cursor is where a person has moved to, which starts somewhere
//! and means nothing, and `chosen` is what a person picked, which starts
//! [`None`] and is the only one that enables *Use this language*.
//!
use alo_keyboards::Layout;
use alo_strings::Language;

/// The order the languages are offered in, declared rather than sorted.
///
/// **Each language's own name decides its place.** `alo_strings::union::OFFICIAL`
/// is in alphabetical order of each language's name *in English*, which is the
/// order `docs/features.md` lists them in and the wrong order for this screen:
/// it would put `Ελληνικά` where *Greek* sorts, which is a fact about a
/// language the person reading may not read.
///
/// **Declared, not computed.** Rust has no Unicode collation and twenty-four
/// fixed strings do not need one; a sort nobody can read would be worse than a
/// list anybody can reorder in one line. `every_official_language_is_offered`
/// holds this to being exactly the twenty-four tags, so a language cannot be
/// dropped or doubled here without a failing build.
///
/// Within the Latin script by that script's alphabet, then Greek, then
/// Cyrillic — which is the order a single collation would produce anyway, and
/// is what every system that offers this list does.
///
/// # The cost, said rather than buried
///
/// This puts `Ελληνικά` and `Български` last, and those are exactly the
/// speakers `docs/features.md` refuses to leave out — *those are exactly the
/// member states with the least software in their own language*. It is
/// accepted because the alternatives are worse for them, not better: ordering
/// by English name hides their language behind a word they may not read, and
/// transliterating it into the Latin run would file `Ελληνικά` under a letter
/// it does not begin with.
///
/// **And ordering is not what makes this findable.** What would is the
/// firmware's own locale or keyboard hint, putting the likely language under
/// the cursor on arrival — this compositor has no source for either yet, and
/// saying so is the difference between a limit and a silence.
const IN_THE_ORDER_A_PERSON_READS: [&str; 22] = [
    // Latin, by its own alphabet.
    "cs", "da", "de", "et", "en", "es", "fr", "ga", "hr", "it", "lv", "lt", "hu", "mt", "nl", "pl",
    "pt", "ro", "sk", "sl", "fi", "sv",
];

/// The two languages this screen does not offer yet, and what they are owed.
///
/// **Not dropped, and not a tier.** `CLAUDE.md` is blunt about the shape this
/// would otherwise take — *"English plus the big five" is the same bug wearing
/// a business case: it serves some people and not others, and the ones it
/// skips are those with the least software in their own language already.* So
/// the reason is a measurement rather than a priority:
///
/// **An account name must be ASCII.** `alo_accounts` requires a first
/// character that is `is_ascii_lowercase`, then lowercase, digits, `-` or `_`
/// — a name a Unix login can have. The `gr` and `bg` layouts put Greek and
/// Cyrillic on the letter keys, so a person who chose one of these and reached
/// the next screen **could not type their own account name**. Offering them
/// today would be offering a door that locks behind them.
///
/// **What they are owed is a second group, not a translation.** The machine
/// needs to hold `[gr, us]` or `[bg, us]` with a key that switches between
/// them, which `alo_keyboards::These::of` and `Keyboards::switch` already
/// support — what is missing is the offer carrying two layouts and this sheet
/// saying which key switches. Every Latin-script layout needs none of that:
/// QWERTY, AZERTY and QWERTZ all put the ASCII letters somewhere, which is
/// exactly what choosing a layout fixes.
///
/// `the_declared_order_and_the_owed_are_exactly_the_official_languages` holds
/// these two lists together to all 24, so a language cannot leave this file by
/// being quietly forgotten — only by being moved from one list to the other.
pub const OWED_A_SECOND_GROUP: [&str; 2] = ["el", "bg"];

/// How many rows of the list a picker shows at once.
///
/// Three, which is what the design file draws on `09b · Choose language`:
/// `Choice / English`, `Choice / Nederlands` and `Choice / Français` at 0, 97
/// and 192 inside a `Choices` frame 277 tall.
pub const ROWS_AT_A_TIME: usize = 3;

/// Where the focus ring is on the language sheet.
///
/// **Two places, because the design draws two controls**: a list of choices
/// and a row of actions. A person with no pointer reaches both with the same
/// keys, and the ring is what tells them which one they are on — without it,
/// the second press of Enter is a mode nobody can see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereTheRingIs {
    /// On the row the cursor is on.
    OnTheList,
    /// On *Use this language*, which is where Enter puts it after a choice.
    OnTheWayOn,
}

/// One language as a row on a picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferedLanguage {
    /// The tag, which is what a [`Language`] is made from.
    pub(crate) tag: &'static str,
    /// What the language calls itself, which the row's **first** line reads.
    ///
    /// The prominent line: Body/Large, 16/25, in `text/primary`.
    pub(crate) calls_itself: &'static str,
    /// What it is called in English, which the row's **second** line reads.
    ///
    /// **The design shows both, and that is better than either alone.** This
    /// file first carried only `calls_itself`, with a test asserting that no
    /// row reads an English name — on the reasoning that a person who does not
    /// read English should not be shown it. Frame `09b · Choose language`
    /// reads `Nederlands` over `Dutch` and `Français` over `French`: the
    /// language's own name identifies the row at 16/25 in `text/primary`, and
    /// the English name sits under it at 14/22 in `text/secondary`. So it
    /// helps somebody who knows only the English name and costs the endonym
    /// reader nothing, because it is never the line that names the row.
    ///
    /// `alo_strings::union::Official::in_english` says of itself that it is
    /// how `docs/features.md` lists them *and is used for nothing else*. It
    /// is used for this.
    pub(crate) in_english: &'static str,
    /// The keyboard offered with it, from `alo_keyboards::offered_with`.
    ///
    /// The whole point of choosing a language here: *choosing Greek and then
    /// hunting for a Greek keyboard is the same bug twice* (`docs/features.md`).
    pub(crate) keyboard: Layout,
}

/// The list, a cursor in it, and what has been chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheLanguagesOffered {
    /// All twenty-four, in `OFFICIAL`'s order.
    all: Vec<OfferedLanguage>,
    /// Where the cursor is. Means nothing until something is chosen.
    on: usize,
    /// What a person picked, and [`None`] until they do.
    chosen: Option<usize>,
    /// Where the focus ring is.
    ring: WhereTheRingIs,
}

impl TheLanguagesOffered {
    /// Every official language, with the cursor on the one this machine
    /// already speaks and nothing chosen.
    ///
    /// A language whose offer `alo_keyboards` has no line for is **left out
    /// rather than offered without a keyboard**: a row a person can pick that
    /// leaves them unable to type is worse than a row that is not there, and
    /// that crate's own tests hold it to a line per official language, so an
    /// empty list here would be that crate failing rather than this one
    /// filtering.
    pub(crate) fn every_official() -> Self {
        let all = IN_THE_ORDER_A_PERSON_READS
            .iter()
            .filter_map(|tag| {
                let language = Language::written(tag).ok()?;
                let offer = alo_keyboards::offered_with(&language)?;
                let official = alo_strings::union::OFFICIAL
                    .iter()
                    .find(|official| official.tag == *tag)?;
                Some(OfferedLanguage {
                    tag,
                    calls_itself: official.calls_itself,
                    in_english: official.in_english,
                    keyboard: offer.first(),
                })
            })
            .collect::<Vec<_>>();
        let on = all
            .iter()
            .position(|it| it.tag == alo_strings::union::THE_SOURCE)
            .unwrap_or_default();
        Self {
            all,
            on,
            chosen: None,
            // The ring opens on the list, on the language this machine already
            // speaks -- so the first thing a person sees marked is the row
            // they are most likely to want.
            ring: WhereTheRingIs::OnTheList,
        }
    }

    /// How many languages are offered.
    pub fn how_many(&self) -> usize {
        self.all.len()
    }

    /// Where the cursor is.
    pub fn cursor(&self) -> usize {
        self.on
    }

    /// Where the focus ring is.
    pub fn ring(&self) -> WhereTheRingIs {
        self.ring
    }

    /// Put the ring back on the list, which every move does.
    ///
    /// **Moving always returns to the list**, because an arrow key means *a
    /// different row* and there is nothing to move through among two actions
    /// that the same key should reach. Tab is what crosses between them.
    pub(crate) fn ring_to_the_list(&mut self) {
        self.ring = WhereTheRingIs::OnTheList;
    }

    /// Put the ring on *Use this language*.
    ///
    /// Where Enter leaves it after choosing a row: the second press then has
    /// something visible under it rather than being a mode.
    pub(crate) fn ring_to_the_way_on(&mut self) {
        self.ring = WhereTheRingIs::OnTheWayOn;
    }

    /// Move the ring between the list and the way on.
    ///
    /// Tab, which is `SignInKey::OtherField` — the key that moves between the
    /// two fields on the sign-in screen does the same job here, so a person
    /// who learnt it on one screen has it on the other. **Only when something
    /// is chosen**: a way on that cannot be taken is not a place to put a
    /// ring, and a person who tabbed onto it and pressed Enter to no effect
    /// would be told nothing by a screen that had invited them there.
    pub(crate) fn ring_crosses(&mut self) {
        self.ring = match (self.ring, self.chosen) {
            (WhereTheRingIs::OnTheList, Some(_)) => WhereTheRingIs::OnTheWayOn,
            (WhereTheRingIs::OnTheList, None) | (WhereTheRingIs::OnTheWayOn, _) => {
                WhereTheRingIs::OnTheList
            }
        };
    }

    /// Move the cursor down one, stopping at the last row.
    ///
    /// **It stops rather than wraps.** A list that jumped from Swedish back to
    /// Bulgarian would leave somebody holding a key unable to tell whether
    /// they had reached the end or gone round it.
    pub(crate) fn down(&mut self) {
        self.ring_to_the_list();
        self.on = self
            .on
            .saturating_add(1)
            .min(self.all.len().saturating_sub(1));
    }

    /// Move the cursor up one, stopping at the first row.
    pub(crate) fn up(&mut self) {
        self.ring_to_the_list();
        self.on = self.on.saturating_sub(1);
    }

    /// Take back the choice, leaving the cursor where it is.
    ///
    /// Escape on this sheet, which `alo_access::leaving` says clears what was
    /// entered on the screen a machine starts at. It leaves the **cursor**
    /// alone on purpose: somebody who chose the wrong row and pressed Escape
    /// is still looking at the row they meant to be near, and moving them back
    /// to the top would be a second thing they did not ask for.
    pub(crate) fn unchoose(&mut self) {
        self.chosen = None;
    }

    /// Choose whatever the cursor is on.
    pub(crate) fn choose(&mut self) {
        if !self.all.is_empty() {
            self.chosen = Some(self.on);
        }
    }

    /// What has been chosen, and [`None`] until something is.
    pub(crate) fn chosen(&self) -> Option<&OfferedLanguage> {
        self.all.get(self.chosen?)
    }

    /// **What the overview shows before anything is chosen**: the chosen
    /// language, or the one the cursor arrives on — the language this machine
    /// already speaks, which is what it will go on speaking if nobody chooses.
    pub(crate) fn chosen_or_already_spoken(&self) -> Option<&OfferedLanguage> {
        self.chosen().or_else(|| self.all.get(self.on))
    }

    /// The first row of the window onto the list.
    ///
    /// Derived from the cursor rather than kept beside it, so there is one
    /// answer to *where is the list scrolled to* and it cannot disagree with
    /// the cursor. The cursor sits in the middle of the window where there is
    /// room on both sides, and the window stops at each end rather than
    /// scrolling past it into blank rows.
    fn first_in_the_window(&self) -> usize {
        let last_start = self.all.len().saturating_sub(ROWS_AT_A_TIME);
        self.on.saturating_sub(ROWS_AT_A_TIME / 2).min(last_start)
    }

    /// The rows a picker draws now.
    pub(crate) fn window(&self) -> &[OfferedLanguage] {
        let from = self.first_in_the_window();
        let to = from.saturating_add(ROWS_AT_A_TIME).min(self.all.len());
        self.all.get(from..to).unwrap_or_default()
    }

    /// Where in the window the cursor is, for drawing the focus ring.
    ///
    /// Always [`Some`] while the list has anything in it — the window is
    /// derived from the cursor, so the cursor is in it by construction — and
    /// this returns an [`Option`] rather than asserting that, because a screen
    /// that drew a ring at row 0 of an empty list would be drawing a lie.
    pub fn cursor_in_the_window(&self) -> Option<usize> {
        self.on.checked_sub(self.first_in_the_window())
    }

    /// Where in the window the chosen row is, for drawing its border, and
    /// [`None`] when nothing is chosen **or when what was chosen has scrolled
    /// out of sight**.
    ///
    /// Those two are deliberately one answer: both mean *do not draw a
    /// selected border on any visible row*, and a caller that had to tell them
    /// apart would be a caller drawing the border in the wrong place.
    pub(crate) fn chosen_in_the_window(&self) -> Option<usize> {
        let chosen = self.chosen?;
        let from = self.first_in_the_window();
        let within = chosen.checked_sub(from)?;
        (within < ROWS_AT_A_TIME).then_some(within)
    }
}

#[cfg(test)]
#[path = "the_languages_offered_tests.rs"]
mod tests;
