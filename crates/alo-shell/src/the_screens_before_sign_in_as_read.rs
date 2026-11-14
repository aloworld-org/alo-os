//! **The screens before anybody signs in, as a screen reader is told about
//! them** ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md)).
//!
//! Until now no tree was built for them at all. The first start — language,
//! region, keyboard, the time zone, the name and the PIN — and the sign-in
//! screen were drawn, and a reader was told nothing: a blind person could not
//! set up an alo machine or sign in to one alone. The owner, 2026-11-14:
//! *"Build it"*.
//!
//! # What is read, in order
//!
//! Each screen is **one dialogue**, named by its question and in front, and in
//! it, in the order they are drawn:
//!
//! - its **heading**, and the sentence under it;
//! - its **list**: every row, not only the rows that fit on this screen — each
//!   named as it is drawn, with its second line as description, **whether it is
//!   the one selected**, and **its place in the list** (*9 of 24*), which is
//!   what the design asks of every choice before an account exists
//!   (`docs/design/the-first-start.md`, *Accessibility, before an account
//!   exists*);
//! - its **fields**, each named by its label;
//! - the **sentence under the card** — what the screen last said, announced
//!   as it appears when it is a refusal;
//! - its **two buttons**, and **Accessibility**.
//!
//! The control the keyboard is on is focused, and the screen is put in front
//! (`crate::access_sheet_in_front`), so a reader is told when a new screen
//! comes up and each time the keyboard moves (`crate::access_events`).
//!
//! **Every word is one the screen draws**, asked of the same vocabulary the
//! drawing asks — the overview's rows through the very function that letters
//! them (`crate::overview_sheet::what_the_row_says`) — so what is heard and
//! what is seen are one sentence (ADR 0089).
//!
//! # Nothing secret crosses
//!
//! A PIN or password field is a **password field**, and says nothing about
//! what is in it — not what, not how much, not even whether anything is typed:
//! the rule the password sheet keeps (`crate::the_password_sheet_as_read`).
//! Nothing on this tree answers `Text`. The name a person types is not a
//! secret, and is drawn in full under its label, so it is told as the field's
//! description, which is how somebody who cannot see the screen hears what
//! they typed.
//!
//! # What this does not do yet
//!
//! **Nothing here can be pressed by a reader.** A reader follows the keyboard,
//! which is the person's own; carrying a reader's press to the screen comes
//! with the road from the compositor to the sign-in login's bus (ADR 0155 §2),
//! where a press crosses as data and is answered as the key it stands for.
//! *Power*, on the sign-in screen, is an icon with no word, and is left out
//! rather than named here.

use alo_access::{Role, State};
use alo_strings::{Filling, Said, Strings, Word};

use crate::access_attributes::{a_heading_at, its_place};
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::first_start_words as say;
use crate::sign_in_entry::SignInField;
use crate::sign_in_screen::SignInShows;
use crate::the_first_account::{ASecret, OnTheSheet, TheAccountSheet, TheAccountShown};
use crate::the_first_start::{Row, Sheet, TheFirstStart};
use crate::the_keyboard_picker::{TheKeyboardPicker, TheRing};
use crate::the_languages_offered::WhereTheRingIs;
use crate::the_regions_offered::{Showing, TheRegionsOffered};
use crate::the_zone_picker::TheZonePicker;

/// **Which screen before sign-in is up** — what tells a reader one screen from
/// the next, so leaving the overview for the language list is heard as a new
/// screen and not as the keyboard moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScreenBeforeSignIn {
    /// Frame `09`: language, region and keyboard.
    Overview,
    /// Frame `09b`: which language.
    Language,
    /// Frame `09c`: which region.
    Region,
    /// Which time zone, in a region that has several.
    Zone,
    /// Frame `09a`: which keyboard.
    Keyboard,
    /// *What should we call you?*
    Name,
    /// *Add a PIN?*
    Pin,
    /// The password, for a person who chose *Use password only*.
    Password,
    /// The sign-in screen.
    SignIn,
    /// A sentence and nothing to type into: the accounts would not be read.
    Sentence,
}

/// How many rows of a list a reader is told: all of them. A short screen draws
/// a window of a long list (`crate::a_window_onto_a_list`), and a reader is
/// not looking at the screen, so it hears the whole.
const THE_WHOLE_LIST: usize = usize::MAX;

impl ReadAloudTree {
    /// **The tree for the screen showing before anybody has signed in**, for
    /// somebody reading in `strings`' language: the shell's own node, and the
    /// screen in front of it with the keyboard on the control it is on.
    ///
    /// Built from [`SignInShows`] — what the screen draws, read with no access
    /// to anything secret — rather than from the screen itself, so a tree can
    /// be built of any moment and nothing here can reach a password.
    #[must_use]
    pub fn before_sign_in(strings: &Strings, shows: SignInShows<'_>) -> Self {
        let mut tree = Self::with_nothing_on_it();
        let (screen, sheet, keyboard) = {
            let mut reading = Reading {
                tree: &mut tree,
                strings,
                keyboard: None,
            };
            let (screen, sheet) = match shows {
                SignInShows::FirstStart(first_start) => reading.the_first_start(first_start),
                SignInShows::FirstAccount { account, said } => {
                    reading.the_first_account(account, said)
                }
                SignInShows::Fields {
                    whose,
                    secret,
                    name,
                    password_typed: _,
                    field,
                    said,
                } => (
                    ScreenBeforeSignIn::SignIn,
                    reading.signing_in(whose, secret, name, field, said),
                ),
                SignInShows::Sentence(said) => {
                    (ScreenBeforeSignIn::Sentence, reading.a_sentence(said))
                }
            };
            // **Accessibility is on every screen that draws it**: every
            // first-start sheet and the sign-in screen, bottom left. The
            // sentence screen draws nothing but its sentence.
            if screen != ScreenBeforeSignIn::Sentence {
                let word = reading.said(crate::sign_in_words::ACCESSIBILITY);
                reading.a_button(sheet, word, false);
            }
            (screen, sheet, reading.keyboard.unwrap_or(sheet))
        };
        if let Some(node) = tree.node_mut(keyboard) {
            node.states = access_roles::with_the_keyboard(node.states);
        }
        tree.put_in_front(ASheetInFront {
            which: TheSheet::BeforeSignIn(screen),
            at: sheet,
            keyboard,
        });
        tree
    }
}

/// One reading of a screen in progress.
struct Reading<'t, 's> {
    /// The tree being added to.
    tree: &'t mut ReadAloudTree,
    /// The person's vocabulary.
    strings: &'s Strings,
    /// The node the keyboard is on, once it is met.
    keyboard: Option<usize>,
}

impl Reading<'_, '_> {
    /// One of the screen's words, in the person's language.
    fn said(&self, word: Word) -> String {
        self.strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    }

    /// Put `node` under `parent`, noting it as where the keyboard is when
    /// `keyboard` says so.
    fn push(&mut self, parent: usize, node: Node, keyboard: bool) -> usize {
        let at = self.tree.push(parent, node);
        if keyboard {
            self.keyboard = Some(at);
        }
        at
    }

    /// The screen's own node, as `role`, named and described, in front of
    /// everything.
    fn the_screen(&mut self, role: Role, name: String, description: String) -> usize {
        self.push(
            0,
            Node {
                role: access_roles::number_of(role),
                name,
                description,
                states: access_roles::in_front(access_roles::words_of(State::ReadOnly, None)),
                ..Node::default()
            },
            false,
        )
    }

    /// A first-start sheet: the dialogue, named by its title and described by
    /// its eyebrow, then its heading and the sentence under it.
    fn a_sheet(&mut self, eyebrow: Word, title: Word, introduction: Option<Word>) -> usize {
        let (eyebrow, title) = (self.said(eyebrow), self.said(title));
        let sheet = self.the_screen(Role::Dialogue, title.clone(), eyebrow);
        self.a_heading(sheet, title);
        if let Some(introduction) = introduction {
            let words = self.said(introduction);
            self.a_line(sheet, words, false);
        }
        sheet
    }

    /// A heading, at the first level: the screen's question.
    fn a_heading(&mut self, parent: usize, words: String) {
        if words.trim().is_empty() {
            return;
        }
        self.push(
            parent,
            Node {
                role: access_roles::number_of(Role::Heading),
                name: words,
                states: access_roles::words_of(State::ReadOnly, None),
                attributes: vec![a_heading_at(1)],
                ..Node::default()
            },
            false,
        );
    }

    /// A sentence the screen says; `announced` where it appeared because
    /// something was tried, so a reader says it without being asked. Answers
    /// where it went, and [`None`] for a sentence with nothing in it, which is
    /// not put on the tree.
    fn a_line(&mut self, parent: usize, words: String, announced: bool) -> Option<usize> {
        if words.trim().is_empty() {
            return None;
        }
        let state = if announced {
            State::AnnouncedWhenItChanges
        } else {
            State::ReadOnly
        };
        Some(self.push(
            parent,
            Node {
                role: access_roles::number_of(Role::Label),
                name: words,
                states: access_roles::words_of(state, None),
                announced,
                ..Node::default()
            },
            false,
        ))
    }

    /// A list, named by the question it answers.
    fn a_list(&mut self, parent: usize, name: String) -> usize {
        self.push(
            parent,
            Node {
                role: access_roles::number_of(Role::List),
                name,
                states: access_roles::words_of(State::ReadOnly, None),
                ..Node::default()
            },
            false,
        )
    }

    /// One row of a list: its name, its second line, its place, and — for a
    /// row that is one of a few choices — whether it is the one selected.
    /// `chosen` is [`None`] for a row that opens something rather than being
    /// chosen, which is every row of the overview and *Other region*.
    fn a_row(
        &mut self,
        list: usize,
        (name, description): (String, String),
        (at, of): (usize, usize),
        chosen: Option<bool>,
        keyboard: bool,
    ) {
        let state = if chosen.is_some() {
            State::OneOfAFew
        } else {
            State::CanBeUsed
        };
        self.push(
            list,
            Node {
                role: access_roles::number_of(Role::ListItem),
                name,
                description,
                states: access_roles::words_of(state, chosen),
                attributes: its_place(at, of),
                ..Node::default()
            },
            keyboard,
        );
    }

    /// A field, named by its label. `shows` is what is drawn in it — what was
    /// typed, or the invitation to type — and is said only for a field that is
    /// not a secret.
    fn a_field(&mut self, parent: usize, role: Role, name: String, shows: String, keyboard: bool) {
        let description = if matches!(role, Role::PasswordEntry) {
            String::new()
        } else {
            shows
        };
        self.push(
            parent,
            Node {
                role: access_roles::number_of(role),
                name,
                description,
                states: access_roles::words_of(State::CanBeUsed, None),
                ..Node::default()
            },
            keyboard,
        );
    }

    /// A button.
    fn a_button(&mut self, parent: usize, name: String, keyboard: bool) {
        self.push(
            parent,
            Node {
                role: access_roles::number_of(Role::Button),
                name,
                states: access_roles::words_of(State::CanBeUsed, None),
                ..Node::default()
            },
            keyboard,
        );
    }

    /// A sheet's two buttons, the way on first; `on` is which one the
    /// keyboard is on, if either.
    fn the_two_buttons(&mut self, sheet: usize, [go_on, go_back]: [Word; 2], on: Option<usize>) {
        let (go_on, go_back) = (self.said(go_on), self.said(go_back));
        self.a_button(sheet, go_on, on == Some(0));
        self.a_button(sheet, go_back, on == Some(1));
    }

    /// The first-start sheet showing, and which it is.
    fn the_first_start(&mut self, first_start: &TheFirstStart) -> (ScreenBeforeSignIn, usize) {
        match first_start.sheet() {
            Sheet::Overview => (ScreenBeforeSignIn::Overview, self.the_overview(first_start)),
            Sheet::Language(_) => (
                ScreenBeforeSignIn::Language,
                self.the_languages(first_start),
            ),
            Sheet::Region(offered) => (ScreenBeforeSignIn::Region, self.the_regions(offered)),
            Sheet::Zone(picker) => (ScreenBeforeSignIn::Zone, self.the_zones(picker)),
            Sheet::Keyboard(picker) => (
                ScreenBeforeSignIn::Keyboard,
                self.the_keyboards(first_start, picker),
            ),
        }
    }

    /// Frame `09`: the three rows, each its label with what it is set to.
    fn the_overview(&mut self, first_start: &TheFirstStart) -> usize {
        let sheet = self.a_sheet(
            say::LANGUAGE_EYEBROW,
            say::OVERVIEW_TITLE,
            Some(say::OVERVIEW_INTRODUCTION),
        );
        let on_the_list = first_start.ring() == WhereTheRingIs::OnTheList;
        let question = self.said(say::OVERVIEW_TITLE);
        let list = self.a_list(sheet, question);
        for (at, row) in Row::ALL.into_iter().enumerate() {
            let says = crate::overview_sheet::what_the_row_says(first_start, row, self.strings);
            self.a_row(
                list,
                says,
                (at, Row::ALL.len()),
                None,
                on_the_list && first_start.row() == row,
            );
        }
        let note = self.said(say::OVERVIEW_NOTE);
        self.a_line(sheet, note, false);
        self.the_two_buttons(
            sheet,
            [say::CONTINUE, say::BACK],
            (!on_the_list).then_some(0),
        );
        sheet
    }

    /// Frame `09b`: every language, by its own name over its English one.
    fn the_languages(&mut self, first_start: &TheFirstStart) -> usize {
        let sheet = self.a_sheet(say::LANGUAGE_EYEBROW, say::LANGUAGE_TITLE, None);
        let offered = first_start.languages();
        let on_the_list = offered.ring() == WhereTheRingIs::OnTheList;
        let question = self.said(say::LANGUAGE_TITLE);
        let list = self.a_list(sheet, question);
        let all = offered.window_of(THE_WHOLE_LIST);
        let of = all.rows.len();
        for (at, language) in all.rows.iter().enumerate() {
            self.a_row(
                list,
                (
                    language.calls_itself.to_owned(),
                    language.in_english.to_owned(),
                ),
                (all.first + at, of),
                Some(all.chosen == Some(at)),
                on_the_list && all.cursor == Some(at),
            );
        }
        self.the_two_buttons(
            sheet,
            [say::USE_THIS_LANGUAGE, say::BACK],
            (!on_the_list).then_some(0),
        );
        sheet
    }

    /// Frame `09c`: the region chosen and *Other region*, or every region.
    fn the_regions(&mut self, offered: &TheRegionsOffered) -> usize {
        let sheet = self.a_sheet(
            say::REGION_EYEBROW,
            say::REGION_TITLE,
            Some(say::REGION_INTRODUCTION),
        );
        let on_the_list = offered.ring() == WhereTheRingIs::OnTheList;
        let question = self.said(say::REGION_TITLE);
        let list = self.a_list(sheet, question);
        let all = offered.window_of(THE_WHOLE_LIST);
        let of = all.rows.len();
        for (at, row) in all.rows.iter().enumerate() {
            let chosen = all.chosen == Some(at);
            let (says, chosen) = match row {
                Some(name) => {
                    let under = match (offered.showing(), chosen) {
                        (Showing::TheTwo, true) => self.said(say::SELECTED_REGION),
                        (Showing::TheTwo | Showing::EveryRegion, _) => String::new(),
                    };
                    (((*name).to_owned(), under), Some(chosen))
                }
                None => (
                    (
                        self.said(say::OTHER_REGION),
                        self.said(say::OTHER_REGION_HINT),
                    ),
                    None,
                ),
            };
            self.a_row(
                list,
                says,
                (all.first + at, of),
                chosen,
                on_the_list && all.cursor == Some(at),
            );
        }
        self.the_two_buttons(
            sheet,
            [say::USE_THIS_REGION, say::BACK],
            (!on_the_list).then_some(0),
        );
        sheet
    }

    /// The time zone, in a region that has several.
    fn the_zones(&mut self, picker: &TheZonePicker) -> usize {
        let sheet = self.a_sheet(
            say::REGION_EYEBROW,
            say::ZONE_TITLE,
            Some(say::ZONE_INTRODUCTION),
        );
        let on_the_list = picker.ring() == WhereTheRingIs::OnTheList;
        let question = self.said(say::ZONE_TITLE);
        let list = self.a_list(sheet, question);
        let all = picker.window_of(THE_WHOLE_LIST);
        let of = all.rows.len();
        for (at, zone) in all.rows.iter().enumerate() {
            self.a_row(
                list,
                (zone.name.clone(), String::new()),
                (all.first + at, of),
                Some(all.chosen == Some(at)),
                on_the_list && all.cursor == Some(at),
            );
        }
        self.the_two_buttons(
            sheet,
            [say::USE_THIS_ZONE, say::BACK],
            (!on_the_list).then_some(0),
        );
        sheet
    }

    /// Frame `09a`: every keyboard offered, by name and country, then the row
    /// a person types into to test the one chosen.
    fn the_keyboards(&mut self, first_start: &TheFirstStart, picker: &TheKeyboardPicker) -> usize {
        let sheet = self.a_sheet(
            say::KEYBOARD_EYEBROW,
            say::KEYBOARD_TITLE,
            Some(say::KEYBOARD_INTRODUCTION),
        );
        let ring = picker.ring();
        let question = self.said(say::KEYBOARD_TITLE);
        let list = self.a_list(sheet, question);
        let all = picker.window_of(THE_WHOLE_LIST);
        let of = all.rows.len();
        let language = first_start.the_language();
        for (at, keyboard) in all.rows.iter().enumerate() {
            let country = alo_keyboards::TheKeyboardNames::its_country(keyboard)
                .and_then(|code| alo_formats::region_names::the_name_in(&code, language))
                .unwrap_or_default();
            self.a_row(
                list,
                (first_start.the_name_of(keyboard), country),
                (all.first + at, of),
                Some(all.chosen == Some(at)),
                ring == TheRing::OnTheKeyboards && all.cursor == Some(at),
            );
        }
        let test = self.said(say::TEST_YOUR_KEYBOARD);
        let typed = if picker.typed().is_empty() {
            self.said(say::TEST_HINT)
        } else {
            picker.typed().to_owned()
        };
        self.a_field(sheet, Role::Entry, test, typed, ring == TheRing::OnTheTest);
        self.the_two_buttons(
            sheet,
            [say::USE_THIS_KEYBOARD, say::BACK],
            (ring == TheRing::OnTheWayOn).then_some(0),
        );
        sheet
    }

    /// The first account's sheets — *What should we call you?*, then a PIN
    /// or a password typed twice — and which is showing.
    fn the_first_account(
        &mut self,
        account: TheAccountShown<'_>,
        said: Option<&Said>,
    ) -> (ScreenBeforeSignIn, usize) {
        let field = |at: usize| account.on == OnTheSheet::Field(at);
        let (screen, sheet, means, buttons) = match account.sheet {
            TheAccountSheet::Name => {
                let sheet = self.a_sheet(
                    say::NAME_EYEBROW,
                    say::NAME_TITLE,
                    Some(say::NAME_INTRODUCTION),
                );
                let label = self.said(say::NAME_CHOICE);
                let shows = if account.name.is_empty() {
                    self.said(say::NAME_HINT)
                } else {
                    account.name.to_owned()
                };
                self.a_field(sheet, Role::Entry, label, shows, field(0));
                (
                    ScreenBeforeSignIn::Name,
                    sheet,
                    say::NAME_CONSEQUENCE,
                    [say::CONTINUE, say::BACK],
                )
            }
            TheAccountSheet::Secret(secret) => {
                let (screen, words, buttons) = match secret {
                    ASecret::Pin => (
                        ScreenBeforeSignIn::Pin,
                        [
                            say::PIN_EYEBROW,
                            say::PIN_TITLE,
                            say::PIN_INTRODUCTION,
                            say::PIN_CHOICE,
                            say::PIN_AGAIN,
                            say::PIN_CONSEQUENCE,
                        ],
                        [say::SAVE, say::USE_PASSWORD_ONLY],
                    ),
                    ASecret::Password => (
                        ScreenBeforeSignIn::Password,
                        [
                            say::PASSWORD_EYEBROW,
                            say::PASSWORD_TITLE,
                            say::PASSWORD_INTRODUCTION,
                            say::PASSWORD_CHOICE,
                            say::PASSWORD_AGAIN,
                            say::PASSWORD_CONSEQUENCE,
                        ],
                        [say::SAVE_PASSWORD, say::BACK],
                    ),
                };
                let [eyebrow, title, introduction, choice, again, means] = words;
                let sheet = self.a_sheet(eyebrow, title, Some(introduction));
                for (at, label) in [choice, again].into_iter().enumerate() {
                    let label = self.said(label);
                    self.a_field(sheet, Role::PasswordEntry, label, String::new(), field(at));
                }
                (screen, sheet, means, buttons)
            }
        };
        // **What the machine last said, or that the two differed, before what
        // the sheet means** — the one line under the card, as it is drawn
        // (`crate::first_start_sheet`). A refusal is announced as it appears.
        let (under, announced) = match said {
            Some(said) => (said.text().to_owned(), true),
            None if account.they_differed => (self.said(say::THEY_DIFFER), true),
            None => (self.said(means), false),
        };
        self.a_line(sheet, under, announced);
        let on = match account.on {
            OnTheSheet::Field(_) => None,
            OnTheSheet::WayOn => Some(0),
            OnTheSheet::WayBack => Some(1),
        };
        self.the_two_buttons(sheet, buttons, on);
        (screen, sheet)
    }

    /// The sign-in screen: the greeting and one field, for a machine that
    /// knows whose it is; a name and a password, for one that cannot say.
    fn signing_in(
        &mut self,
        whose: &str,
        secret: alo_greeting::SecretKind,
        name: &str,
        field: SignInField,
        said: Option<&Said>,
    ) -> usize {
        let called = self.said(alo_access::words::SIGN_IN);
        let screen = self.the_screen(Role::Window, called, String::new());
        if whose.is_empty() {
            let (who, password) = (
                self.said(alo_access::words::WHO_IS_SIGNING_IN),
                self.said(alo_access::words::THE_PASSWORD),
            );
            self.a_field(
                screen,
                Role::Entry,
                who,
                name.to_owned(),
                field == SignInField::Name,
            );
            self.a_field(
                screen,
                Role::PasswordEntry,
                password,
                String::new(),
                field == SignInField::Password,
            );
        } else {
            // **The greeting, in the screen's own words**, and the one field
            // the screen draws under it, which every key goes into.
            let hello = self
                .strings
                .say(
                    &crate::sign_in_words::GREETING.key(),
                    &Filling::of("name", whose),
                )
                .text()
                .to_owned();
            self.a_heading(screen, hello);
            // **Named for the secret this person chose**, by the same choice
            // the drawing makes (`crate::sign_in_words::enter_your`), so what
            // is heard and what is seen cannot disagree.
            let asked = self.said(crate::sign_in_words::enter_your(secret));
            self.a_field(screen, Role::PasswordEntry, asked, String::new(), true);
        }
        if let Some(said) = said {
            self.a_line(screen, said.text().to_owned(), true);
        }
        // **No *Other ways to sign in***: the screen no longer draws it,
        // because no account has another way in, and a reader that offered a
        // button the eye cannot find would be the two disagreeing.
        let switch = self.said(crate::sign_in_words::SWITCH_PERSON);
        self.a_button(screen, switch, false);
        screen
    }

    /// A sentence and nothing to type into: the screen, and the sentence, which
    /// is where the keyboard is said to be because there is nowhere else.
    fn a_sentence(&mut self, said: &Said) -> usize {
        let called = self.said(alo_access::words::SIGN_IN);
        let screen = self.the_screen(Role::Window, called, String::new());
        if let Some(line) = self.a_line(screen, said.text().to_owned(), true) {
            self.keyboard = Some(line);
        }
        screen
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_screens_before_sign_in_as_read_tests.rs"]
mod tests;
