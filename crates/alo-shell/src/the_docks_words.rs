//! Everything on the Dock that a person reads, said once a frame.
//!
//! **The drawing path may not word a sentence.** `tests/desktop_source.rs` holds
//! that no file named `desktop_*` or `dock_*` names `Word::`, `.say(` or
//! `Filling::`: every sentence a person meets comes from `alo-saying`, and a
//! raster that looked one up would be a second place deciding what a person
//! reads.
//!
//! So the lane says them and hands the results over, exactly as it does for a
//! window's control labels.
//!
//! # One bundle rather than five fields
//!
//! `DesktopFrame` carried `the_overflows_heading` alone for a day, and the Dock
//! then gained a composer with two strings in it and two command labels. Five
//! borrowed `&str` fields on a frame is five chances to set four of them, and
//! the one that is forgotten is a blank where a word should be — which draws
//! fine and says nothing.
//!
//! A struct cannot be half-filled: this type's one constructor builds
//! all of them or none, from one `Strings`, in one place.

use alo_strings::{Filling, Strings};

/// What the Dock says, already said.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheDocksWords {
    /// The heading over *More apps*.
    pub heading: String,
    /// What the search field at the top of *More apps* invites.
    pub search: String,
    /// The heading over the applications a person pinned.
    pub pinned: String,
    /// The heading over everything else that is open.
    pub others: String,
    /// The line under the list, which says what is on it and how to leave.
    pub footer: String,
    /// The line that replaces it when the list has more below.
    ///
    /// **Both are said, and the drawing chooses.** Whether a list scrolls is a
    /// fact about this frame's layout, which only the drawing knows — and the
    /// drawing may not word a sentence. So the lane says both and hands them
    /// over, which is the same shape as every other word here.
    pub scrolling: String,
    /// What the composer invites a person to do.
    pub prompt: String,
    /// The key that opens the composer, shown at its end.
    pub key: String,
    /// The control that lays every open window out at once.
    pub show_all: String,
    /// The control that opens what a person did before.
    pub history: String,
    /// What the person has typed into the search field, or [`None`] when they
    /// have typed nothing.
    ///
    /// **Not a word, and here anyway.** Everything else in this type is said
    /// from the vocabulary; this is the person's own text, which is never
    /// translated. It is carried here because the drawing may not reach into
    /// the `Server` — the same rule that puts every other string in this
    /// struct — and because *what is in the field* and *what the field says
    /// when it is empty* are one decision at the point of drawing.
    pub typed: Option<String>,
    /// What stands where the grid would be when the search matches nothing.
    ///
    /// [`None`] whenever there is a grid to draw, which is every frame but
    /// one. **An empty field is never this**: it shows everything, and the
    /// sentence is for a search that found nothing — two states that look
    /// alike in a list of zero rows.
    pub nothing_matches: Option<String>,
    /// The name of the Settings entry, when a person has one on their Dock.
    ///
    /// **Said whether or not it is pinned**, unlike the picker below. It is one
    /// fixed label with nothing about this moment in it, so saying it per frame
    /// would buy nothing — and a drawing that had to ask whether it was pinned
    /// before it could have a name is a drawing that can be handed a row it
    /// cannot label.
    ///
    /// **It is translated, which no other Dock icon's name is.** Every other
    /// label there is an application's own name as its source reports it; this
    /// one is alo OS's word for its own surface.
    pub settings: String,
    /// What the Applications entry on the Dock is called.
    pub applications: String,
    /// What the control at the end of the Dock is called: the name its
    /// tooltip shows and a screen reader speaks, since it draws only a mark.
    pub show_more_apps: String,
    /// What one application's window list says, when one is open.
    ///
    /// [`None`] when no picker is open, which is almost every frame. **Not
    /// empty strings**: a heading of nothing drawn over a list that is not
    /// there is a different bug from no list at all, and only one of them
    /// should be possible.
    pub picker: Option<ThePickersWords>,
}

/// What one application's window list says, already said.
///
/// **Said per frame, because every line of it is about this moment**: the
/// heading carries the application's name, the count carries how many windows
/// it has, and a row carries the Place its window is on. None of them can be
/// said once at sign-in the way the Dock's fixed labels are.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThePickersWords {
    /// The heading — the application's name and the word *windows*.
    pub heading: String,
    /// How many there are and what choosing one does.
    pub count: String,
    /// The line underneath, which says Escape goes back rather than out.
    pub footer: String,
    /// What each row says about its window's state, in the rows' own order.
    ///
    /// **One per row and in the same order**, so a drawing takes the `n`th of
    /// these for the `n`th row rather than deciding again which state a window
    /// is in. Two places deciding that is two places that can disagree about
    /// whether a window is put aside.
    pub states: Vec<String>,
}

impl TheDocksWords {
    /// Add what the person typed, and the sentence for a search that matched
    /// nothing.
    ///
    /// **Both or neither of the second**, which is why this takes the list
    /// length rather than a `bool`: *nothing matched* is `typed` having text
    /// in it **and** no rows left, and a caller passing one without the other
    /// is how a surface comes to say *nothing here matches* over a full grid.
    #[must_use]
    pub fn and_what_they_typed(
        mut self,
        strings: &Strings,
        typed: &alo_dock::WhatTheyTyped,
        how_many_are_left: usize,
    ) -> Self {
        if typed.is_nothing() {
            return self;
        }
        let text = typed.as_text().to_owned();
        if how_many_are_left == 0 {
            self.nothing_matches = Some(
                strings
                    .say(
                        &alo_dock::words::NOTHING_MATCHES.key(),
                        &Filling::of("typed", text.clone()),
                    )
                    .into_text(),
            );
        }
        self.typed = Some(text);
        self
    }

    /// Say all of them, in this person's language.
    ///
    /// **All of them or none**, which is the whole reason this is a type: a
    /// frame with four of them set and one blank draws a control with no label
    /// on it and refuses nothing.
    #[must_use]
    pub fn said(strings: &Strings, open: &crate::WhatTheDockHasOpen) -> Self {
        let say =
            |word: &alo_strings::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
        let fill =
            |word: &alo_strings::Word, with: Filling| strings.say(&word.key(), &with).into_text();
        Self {
            heading: say(&alo_dock::words::MORE_OPEN_APPS),
            search: say(&alo_dock::words::SEARCH_APPS),
            pinned: say(&alo_dock::words::PINNED),
            others: say(&alo_dock::words::OTHER_OPEN_APPS),
            footer: say(&alo_dock::words::PINNED_AND_OPEN_ESCAPE_CLOSES),
            scrolling: say(&alo_dock::words::SCROLL_FOR_MORE),
            prompt: say(&alo_dock::words::FIND_OPEN_OR_ASK),
            key: say(&alo_dock::words::THE_COMPOSERS_KEY),
            show_all: say(&alo_dock::words::SHOW_ALL),
            history: say(&alo_dock::words::HISTORY),
            settings: say(&alo_dock::words::SETTINGS_ON_THE_DOCK),
            applications: say(&alo_dock::words::APPLICATIONS_ON_THE_DOCK),
            show_more_apps: say(&alo_dock::words::SHOW_MORE_OPEN_APPS),
            // Said by the caller, which is the only place that holds both the
            // text and whether anything matched it.
            typed: None,
            nothing_matches: None,
            picker: open.the_windows_of().map(|picker| {
                let rows = picker.rows();
                ThePickersWords {
                    heading: fill(
                        &alo_dock::words::WINDOWS_OF,
                        Filling::of("app", picker.app().name().to_owned()),
                    ),
                    count: fill(
                        &alo_dock::words::CHOOSE_ONE_TO_RETURN,
                        Filling::of("windows", rows.len().to_string()),
                    ),
                    footer: say(&alo_dock::words::CHOOSE_A_WINDOW_ESCAPE_RETURNS),
                    states: rows
                        .iter()
                        .map(|row| {
                            let word = if row.sits().was_put_aside() {
                                &alo_dock::words::A_WINDOW_IS_PUT_ASIDE
                            } else {
                                &alo_dock::words::A_WINDOW_IS_OPEN
                            };
                            // The Place a window is on. `Patch` knows where it
                            // sits on the plane and not which Place that is, so
                            // this is the one thing the picker cannot say yet —
                            // named rather than left as an empty fill.
                            fill(word, Filling::of("place", String::new()))
                        })
                        .collect(),
                }
            }),
        }
    }
}
