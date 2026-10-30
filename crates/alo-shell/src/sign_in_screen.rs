//! The sign-in screen: what a person types, what it is lent to, and what the
//! screen says afterwards.
//!
//! # It composes nothing
//!
//! `alo_greeting::Greeting` is the whole of signing in — authenticate, agree
//! about the number, knock once — and this screen calls it and nothing else.
//! It never names `alo_accounts::Accounts` or `alo_sessiond`: the order those
//! are asked in is the composition's, and a screen that re-implemented it is
//! the bug that crate exists to prevent. `tests/sign_in_source.rs` reads this
//! crate's sign-in files and holds them to it.
//!
//! # It says nothing of its own
//!
//! Every sentence it can show is one `alo-greeting` hands it —
//! `Greeted::said`, `Standing::said`, `NotReadable::said` — which are that
//! crate's words and the words of the two crates whose refusals it carries. A
//! wrong password and a name nobody has are one sentence, the same one each
//! time, and nothing here counts attempts or adds a clause, because anything
//! added here is a second account of the moment and the obvious place to leak
//! which names exist.
//!
//! # It decides nothing
//!
//! Who may sign in, what a password is worth and what a session is are
//! decided below this screen. No account is made here — a machine with nobody
//! on it shows `alo-greeting`'s *make an account* sentence and takes no keys,
//! because making one is `alo-setting-up`'s.
//!
//! # It stops when a session opens
//!
//! [`SignInScreen::pressed`] takes the screen **by value**. When the knock
//! opens a session the answer is [`Signing::HandedOver`], which holds the
//! session and no screen — so there is nothing left to draw and nothing left
//! to type at, and two things cannot own the display at once.

use alo_accounts::Session;
use alo_greeting::{Greeted, Greeting, Knocking, NotAnswered, NotReadable, Standing};
use alo_strings::{Said, Strings};

use crate::a_press_on_a_setup_sheet::OnASheet;
use crate::sign_in_entry::{SignInEntry, SignInField};
use crate::sign_in_keys::SignInKey;
use crate::the_first_start::{TheFirstStart, Went};

/// The sign-in screen, before anybody is signed in.
///
/// No `Debug`: it holds what is being typed, password included.
pub struct SignInScreen<K: Knocking> {
    /// What this machine's accounts let the screen stand at.
    stands: Stands<K>,
    /// The person's language, for every sentence shown.
    strings: Strings,
    /// What has been typed.
    entry: SignInEntry,
    /// The sentence the last sign-in answered with, if it answered one.
    said: Option<Said>,
    /// What the machine said when the conversation with the opener did not
    /// happen, in English, for whoever maintains the machine.
    trouble: Option<NotAnswered>,
}

/// What the screen stands at, derived once from the greeting.
enum Stands<K: Knocking> {
    /// A name and a password are asked for.
    SignIn(Greeting<K>),
    /// **Nobody has an account yet, and the language comes first.**
    ///
    /// Before the first account can be made, a person has to be able to type
    /// their own name — and until they have chosen, this machine types through
    /// whatever its unit file named. `ALO_KEYBOARD=gb` has been that on every
    /// machine ever written, so somebody on a French or German keyboard
    /// reached the name sheet and could not type their password through a
    /// British layout. That is the bug this state exists to close, and it is
    /// why the language sheet comes **before** `MakeAnAccount` rather than
    /// after: a choice made afterwards is a choice made in the wrong keyboard.
    ///
    /// Since 2026-10-28 the language is one row of three, on the overview of
    /// frame `09` — language, region and keyboard — and every one of them is
    /// chosen before the name is asked (`crate::the_first_start`).
    ///
    /// The `Said` and the `Greeting` are carried through untouched, so the
    /// account sheet this becomes is the one `of` would have built.
    FirstStart(Box<TheFirstStart>, Said, Greeting<K>),

    /// Nobody has an account yet: this is the sentence, **and the greeting
    /// that can make the first one**.
    ///
    /// The greeting was dropped here until 2026-10-13, so this state could
    /// say *make an account* and had no way for anybody to make one — the
    /// screen took no keys at all. `alo_greeting::makes_an_account` has been
    /// able to make a machine's first account since 2026-10-10, refusing
    /// when the store is not empty and writing to the disk before signing
    /// anybody in, and **nothing had ever called it**. A fresh machine had
    /// nobody to sign in as, which is what stood between alo OS and a usable
    /// computer.
    MakeAnAccount(Said, Greeting<K>),
    /// The accounts would not be read; this is the sentence and the reason,
    /// and no keys are taken.
    Unreadable(Said, NotReadable),
}

/// What became of one key press.
///
/// No `Debug`: [`Signing::Still`] holds the screen and what is typed at it.
pub enum Signing<K: Knocking> {
    /// Nobody is signed in yet; this is the screen to draw.
    Still(Box<SignInScreen<K>>),
    /// A session is open. There is no screen in here, on purpose: whatever the
    /// session shows owns the display from now on.
    HandedOver(Session),
}

/// What a screen draws, read without any access to the password.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInShows<'a> {
    /// A name field and a password field.
    Fields {
        /// **Whose screen this is**: the display name of the person this
        /// machine signs in, read from their own account. Empty when the
        /// machine has nobody with that number yet.
        ///
        /// Separate from `name`, because they are different facts and were
        /// confused once. The full-screen design greets the person it is
        /// authenticating — *alo, disan* above the entry — and that greeting
        /// was drawn from `name`, which is **what has been typed** and is
        /// empty when the screen appears. So the screen shaped an empty
        /// string, drew a box with no glyphs in it, and a booted machine
        /// showed a blank bar where somebody's name belongs.
        whose: &'a str,
        /// The name as typed.
        name: &'a str,
        /// Whether anything is typed in the password field. Not how much.
        password_typed: bool,
        /// Where the next letter goes.
        field: SignInField,
        /// What the last attempt answered, if it answered anything.
        said: Option<&'a Said>,
    },
    /// **The first account on this machine**: a name and a password, on the
    /// first-start sheet rather than the sign-in screen.
    ///
    /// Told apart from [`Self::Fields`] because the two are different
    /// surfaces for different moments. `Fields` authenticates somebody this
    /// machine already knows and greets them by name; this one is asked of a
    /// machine that knows nobody, so there is no name to greet and no *other
    /// ways to sign in* to offer.
    FirstAccount {
        /// The name as typed.
        name: &'a str,
        /// Whether anything is typed in the password field. Not how much.
        password_typed: bool,
        /// Where the next letter goes.
        field: SignInField,
        /// What this machine last said about making an account — the
        /// invitation before anybody has tried, and the refusal after.
        said: Option<&'a Said>,
    },
    /// **The first-start sheets**, before any account exists on this machine:
    /// the overview of frame `09` and the three pickers behind it. Carries the
    /// whole state rather than the rows drawn, because the drawing needs the
    /// cursor, the ring and the choice as well, and separate fields that must
    /// agree is the shape that lets them disagree.
    FirstStart(&'a TheFirstStart),
    /// A sentence and nothing to type into.
    Sentence(&'a Said),
}

impl<K: Knocking> SignInScreen<K> {
    /// The screen for a greeting — or for the accounts that would not be read
    /// into one — in the person's language.
    ///
    /// Takes the result of `alo_greeting::Greeting::at` as it is, so that the
    /// screen for a machine whose accounts are refused is decided by that
    /// crate rather than by a second reading of the file here.
    #[must_use]
    pub fn of(greeting: Result<Greeting<K>, NotReadable>, strings: Strings) -> Self {
        let stands = match greeting {
            Err(why) => Stands::Unreadable(why.said(&strings), why),
            Ok(greeting) => match greeting.standing() {
                Standing::MakeAnAccount => match Standing::MakeAnAccount.said(&strings) {
                    Some(said) => {
                        Stands::FirstStart(Box::new(TheFirstStart::arriving(None)), said, greeting)
                    }
                    // `alo-greeting` always words this standing; were it ever
                    // not to, asking for a name is the honest fallback —
                    // `alo-accounts` refuses every one on an empty machine.
                    None => Stands::SignIn(greeting),
                },
                Standing::SignIn => Stands::SignIn(greeting),
            },
        };
        // **The field a person can see is the field their keys go into.**
        // The returning screen draws one field, `Enter your PIN`, above a name
        // it already knows; starting on the name there would send every
        // keystroke somewhere invisible. The account screen asks for a name
        // and starts on one.
        let entry = match &stands {
            // **Only when the machine can say whose it is.** Then it draws one
            // field over a name it already knows, and that is the field keys
            // must go into.
            //
            // When `whose_name` is `None` the machine has accounts and cannot
            // name its own person — `agentd.toml` and `accounts.toml`
            // disagreeing — and there is a name to be asked for. Starting on
            // the password there would leave nobody able to supply it.
            Stands::SignIn(greeting) if greeting.whose_name().is_some() => {
                SignInEntry::waiting_for_a_password()
            }
            Stands::SignIn(_)
            | Stands::FirstStart(_, _, _)
            | Stands::MakeAnAccount(_, _)
            | Stands::Unreadable(_, _) => SignInEntry::empty(),
        };
        Self {
            stands,
            strings,
            entry,
            said: None,
            trouble: None,
        }
    }

    /// **The region this machine's time zone is in**, for the region sheet to
    /// arrive with — or [`None`] where it keeps universal time or none.
    #[must_use]
    pub fn where_the_machine_is(mut self, region: Option<alo_formats::RegionCode>) -> Self {
        if let Stands::FirstStart(first_start, _, _) = &mut self.stands {
            first_start.on_a_machine_in(region);
        }
        self
    }

    /// **What each keyboard is called**, in every language this machine has
    /// `xkeyboard-config`'s catalogue for, for the keyboard sheet.
    #[must_use]
    pub fn naming_keyboards_with(mut self, names: alo_keyboards::TheKeyboardNames) -> Self {
        if let Stands::FirstStart(first_start, _, _) = &mut self.stands {
            first_start.naming_keyboards_with(names);
        }
        self
    }

    /// Whether this screen draws a single field.
    ///
    /// True for the screen a person comes back to: it knows whose machine it
    /// is, says so, and asks only for a password. `sign_in_raster` draws that
    /// screen with `let _ = field;` — it has no second field to put a cursor
    /// in — and this is the same condition, named once so the two cannot
    /// drift apart again.
    fn draws_one_field(&self) -> bool {
        match &self.stands {
            Stands::SignIn(greeting) => greeting.whose_name().is_some(),
            Stands::FirstStart(_, _, _)
            | Stands::MakeAnAccount(_, _)
            | Stands::Unreadable(_, _) => false,
        }
    }
    /// One key press, and what became of the screen.
    ///
    /// Letters, erasing and moving between the two fields edit what is typed.
    /// Enter in the name field moves to the password; Enter in the password
    /// field lends both to the greeting, **forgets both at once whatever it
    /// answered**, and either hands over to the session that opened or shows
    /// the sentence the greeting chose. A screen standing at a sentence takes
    /// no keys at all.
    #[must_use]
    pub fn pressed(mut self, key: SignInKey) -> Signing<K> {
        // **The language sheet answers first, and takes the keys itself.**
        // It is a list rather than two fields, so `SignInEntry` has nothing to
        // say about it — handing an arrow to a password field would be the
        // wrong question asked of the wrong screen.
        if matches!(self.stands, Stands::FirstStart(_, _, _)) {
            return self.on_the_first_start(key);
        }
        // **A screen with one field has no other field to move to.** The
        // returning screen draws a single `Enter your PIN` over a name it
        // already knows, and `sign_in_raster` discards `field` entirely for
        // it. A Tab that moved the waiting field to a name nobody can see
        // would put every key after it somewhere invisible -- which is the
        // fault this whole change is about, reachable one keystroke later.
        if matches!(key, SignInKey::OtherField) && self.draws_one_field() {
            return Signing::Still(Box::new(self));
        }
        if !self.entry.pressed(key) {
            // **Typing starts a new attempt, so the last one stops speaking.**
            // A refusal describes the attempt that produced it and nothing
            // else. Measured on a booted machine on 2026-10-18: an empty
            // secret was refused, the person typed their name again and went
            // on to the PIN row, and *an account needs a password, and an
            // empty one is not one* was still sitting under it — a sentence
            // about a sheet they had already left.
            self.said = None;
            return Signing::Still(Box::new(self));
        }
        // **The secret is forgotten the moment either road is taken**, and
        // before the answer is looked at: a refusal that left a password in
        // memory would be a refusal that kept it.
        //
        // **The name goes with it when somebody is signing in, and stays when
        // this is the machine's first account.** `Entry::the_secret_is_forgotten`
        // carries the argument: clearing the name protects the *last* person
        // at a shared screen, and the first-account sheet is reached only when
        // there is no last person.
        let name = self.entry.name().to_owned();
        let password = self.entry.password().to_owned();
        if matches!(self.stands, Stands::MakeAnAccount(_, _)) {
            self.entry.the_secret_is_forgotten();
        } else {
            self.entry.forgotten();
        }
        let greeted = match &mut self.stands {
            Stands::SignIn(greeting) => {
                // **The name is the one this machine already knows.** The
                // returning screen draws no name field -- it says *alo,
                // disan* above a single `Enter your PIN` -- so there is
                // nobody to type one, and the entry starts on the password
                // for that reason. `whose_name` is what the screen is
                // already showing.
                //
                // `name` is the fallback for a machine that has accounts and
                // cannot say whose, which `alo-greeting` answers `None` for:
                // asking is then the honest thing, and the field is one Tab
                // away.
                let whose = greeting.whose_name().unwrap_or(&name).to_owned();
                greeting.signs_in(&whose, &password)
            }
            // **The first account on this machine.** `makes_an_account`
            // refuses when the store is not empty, writes before it signs
            // anybody in, and signs in through `signs_in` rather than opening
            // a second road to a knock — so there is nothing to repeat here.
            Stands::MakeAnAccount(_, greeting) => greeting.makes_an_account(&name, &password),
            // The language sheet never reaches here: `pressed` hands its keys
            // to `on_the_first_start` before an entry sees them. Named
            // rather than caught, so adding a state is a compiler error here.
            Stands::FirstStart(_, _, _) | Stands::Unreadable(_, _) => {
                return Signing::Still(Box::new(self));
            }
        };
        self.answered(greeted)
    }

    /// **One key press while the first-start sheets are up** —
    /// `crate::the_first_start` decides what it does.
    ///
    /// **The screen speaks the language from the press that chose it**, the
    /// owner's decision of 2026-10-28 (*from the first minute*), so every sheet
    /// after it and the name sheet are in it wherever this machine has a
    /// translation; a language unchosen gives the screen back its source.
    ///
    /// **Nothing here writes anything.** What was chosen leaves through
    /// [`Self::the_language_chosen`], [`Self::the_keyboard_chosen`] and
    /// [`Self::the_region_chosen`], and the compositor keeps and applies it.
    fn on_the_first_start(self, key: SignInKey) -> Signing<K> {
        self.the_first_start_does(|first_start| first_start.pressed(key))
    }

    /// **One press of the pointer on a first-start or first-account sheet.**
    ///
    /// On the first-start sheets `crate::the_first_start` answers it. On the
    /// first-account sheets — *Your name* and *Add a PIN* — the way on is the
    /// Enter it is lettered for (*Continue*, then *Save*), so a click and the
    /// key take one road to making the account; the field row is where the
    /// keyboard already is, and nothing else on them acts. On every other
    /// screen a sheet's press means nothing: `crate::sign_in_pointing` answers
    /// for the sign-in screen's own controls.
    #[must_use]
    pub(crate) fn pressed_on_a_sheet(self, on: OnASheet) -> Signing<K> {
        match (&self.stands, on) {
            (Stands::FirstStart(_, _, _), _) => {
                self.the_first_start_does(|first_start| first_start.pressed_on(on))
            }
            (Stands::MakeAnAccount(_, _), OnASheet::WayOn) => self.pressed(SignInKey::Enter),
            (Stands::MakeAnAccount(_, _), _)
            | (Stands::SignIn(_) | Stands::Unreadable(_, _), _) => Signing::Still(Box::new(self)),
        }
    }

    /// `then` done to the first-start sheets, and the screen it leaves.
    fn the_first_start_does(mut self, then: impl FnOnce(&mut TheFirstStart) -> Went) -> Signing<K> {
        let Stands::FirstStart(mut first_start, said, greeting) = self.stands else {
            return Signing::Still(Box::new(self));
        };
        let went = then(&mut first_start);
        let chosen: Vec<alo_strings::Language> = first_start
            .the_language_chosen()
            .and_then(|tag| alo_strings::Language::written(tag).ok())
            .into_iter()
            .collect();
        self.strings.prefers(&chosen);
        self.stands = match went {
            Went::Onward => Stands::MakeAnAccount(said, greeting),
            Went::Stayed => Stands::FirstStart(first_start, said, greeting),
        };
        Signing::Still(Box::new(self))
    }

    /// **The region the person took** with *Use this region*, for whoever
    /// keeps it — [`None`] until they take one, and on every other screen.
    #[must_use]
    pub fn the_region_chosen(&self) -> Option<&str> {
        match &self.stands {
            Stands::FirstStart(first_start, _, _) => first_start
                .the_region_chosen()
                .map(alo_formats::RegionCode::code),
            Stands::SignIn(_) | Stands::MakeAnAccount(_, _) | Stands::Unreadable(_, _) => None,
        }
    }

    /// **The time zone the region taken sets**, by its database name, for
    /// whoever sets it — [`None`] until one is, and on every other screen.
    #[must_use]
    pub fn the_zone_chosen(&self) -> Option<&str> {
        match &self.stands {
            Stands::FirstStart(first_start, _, _) => first_start.the_zone_chosen(),
            Stands::SignIn(_) | Stands::MakeAnAccount(_, _) | Stands::Unreadable(_, _) => None,
        }
    }

    /// **The keyboard to type on now**, for whoever keeps and applies it —
    /// chosen on the keyboard sheet, or the chosen language's, and [`None`]
    /// until either is chosen and on every other screen.
    #[must_use]
    pub fn the_keyboard_chosen(&self) -> Option<alo_keyboards::Layout> {
        match &self.stands {
            Stands::FirstStart(first_start, _, _) => first_start.the_keyboard_chosen(),
            Stands::SignIn(_) | Stands::MakeAnAccount(_, _) | Stands::Unreadable(_, _) => None,
        }
    }

    /// The language the person chose, for whoever keeps and applies it.
    ///
    /// [`None`] until they choose one, and on every screen that is not the
    /// language sheet. The compositor asks after each press: a layout that
    /// changed is written to `alo_keyboards::THE_MACHINES_KEYBOARDS` and
    /// handed to `Server::the_keyboard_is_now`, so the next key a person
    /// presses is already in their own layout — which is the whole point of
    /// the sheet coming before the name.
    #[must_use]
    pub fn the_language_chosen(&self) -> Option<&str> {
        match &self.stands {
            Stands::FirstStart(first_start, _, _) => first_start.the_language_chosen(),
            Stands::SignIn(_) | Stands::MakeAnAccount(_, _) | Stands::Unreadable(_, _) => None,
        }
    }

    #[must_use]
    /// The words this machine can say, for a surface that draws this screen.
    ///
    /// **Lent, never handed over.** The labels on the sign-in screen are said
    /// in the person's own language, and the drawing has no business holding a
    /// vocabulary of its own — it asks this one at the moment it letters a
    /// line, the way it already takes a worded refusal rather than composing
    /// an English sentence from a reason.
    pub const fn words(&self) -> &Strings {
        &self.strings
    }

    /// What the screen draws now.
    pub fn shows(&self) -> SignInShows<'_> {
        match &self.stands {
            Stands::SignIn(greeting) => SignInShows::Fields {
                whose: greeting.whose_name().unwrap_or_default(),
                name: self.entry.name(),
                password_typed: self.entry.has_password(),
                field: self.entry.field(),
                said: self.said.as_ref(),
            },
            // **Fields, not a sentence.** This answered `Sentence` until
            // 2026-10-13, which is why a machine with no accounts showed
            // *make an account* and offered nowhere to make one.
            Stands::MakeAnAccount(_said, _) => SignInShows::FirstAccount {
                name: self.entry.name(),
                password_typed: self.entry.has_password(),
                field: self.entry.field(),
                // **Only what an attempt answered.** This also passed the
                // standing sentence — *nobody has an account yet* — which is
                // advice rather than a refusal, and drawing it where the
                // sheet's own consequence goes wrapped it into the button
                // below. The sheet says what making an account means; the
                // greeting speaks only when something was tried.
                said: self.said.as_ref(),
            },
            Stands::FirstStart(first_start, _, _) => SignInShows::FirstStart(first_start),
            Stands::Unreadable(said, _) => SignInShows::Sentence(said),
        }
    }

    /// What went wrong with the machine, in English, for the process that
    /// drew the screen to write in its own log — or [`None`] when nothing did.
    ///
    /// The door and what the machine said about it, or the accounts file and
    /// why it was refused. Never anything typed: neither `alo-greeting` type
    /// has anywhere to put it.
    pub fn for_the_maintainer(&self) -> Option<String> {
        match &self.stands {
            Stands::Unreadable(_, why) => Some(why.to_string()),
            Stands::FirstStart(_, _, _) | Stands::SignIn(_) | Stands::MakeAnAccount(_, _) => {
                self.trouble.as_ref().map(ToString::to_string)
            }
        }
    }

    /// The screen after a sign-in answered.
    fn answered(mut self, greeted: Greeted) -> Signing<K> {
        self.said = greeted.said(&self.strings);
        match greeted {
            Greeted::SignedIn(session) => Signing::HandedOver(session),
            Greeted::Refused(_) => {
                self.trouble = None;
                Signing::Still(Box::new(self))
            }
            Greeted::NotAnswered(why) => {
                self.trouble = Some(why);
                Signing::Still(Box::new(self))
            }
        }
    }
}

#[cfg(test)]
#[path = "sign_in_screen_tests.rs"]
mod tests;
