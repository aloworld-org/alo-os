//! A person's keyboards: which ones they have, which one they are typing on,
//! which key composes, and how they type a script no keyboard has.
//!
//! This is the value the rest of the machine reads. It is built from what the
//! release ships and what the person changed ([`Keyboards::shipped`] and
//! [`Keyboards::with`], which is ADR 0038's shape), and every change to it is
//! decided before anything moves.
//!
//! # What is a setting and what is not
//!
//! The keyboards, their order, the compose key and the input methods are the
//! person's settings and are kept ([`crate::keeping`]). **Which of them is in
//! use right now is not**: it is where somebody is in a working day, it goes
//! back to the first keyboard at the next sign-in, and writing it to a disk
//! every time a person pressed the switch would be this crate writing a file
//! forty times an hour to remember something nobody asked it to remember.
//!
//! # There is always a keyboard
//!
//! Not as a rule that is checked in several places but as [`crate::These`], a
//! list that cannot be empty. So [`Keyboards::in_use`] answers a keyboard
//! rather than a maybe, and the refusal a person meets when they try to remove
//! their last one ([`Refused::TheLastKeyboard`]) is the type saying no rather
//! than a condition somebody remembered to write.

use alo_strings::Language;

use crate::changes::Changes;
use crate::compose_key::ComposeKey;
use crate::layout::Layout;
use crate::methods::{IT_IS_STARTED_BY, Writing};
use crate::offering;
use crate::refusing::Refused;
use crate::rented::Rented;
use crate::these::These;

/// The keyboard a machine that has been told nothing types on.
///
/// Not a claim about the person: setup asks for a language and
/// [`Keyboards::offered_with`] answers with theirs. This is what is true before
/// anybody has said anything at all.
const BEFORE_ANYBODY_SAID: &str = "us";

/// A person's keyboards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyboards {
    /// The keyboards, in the order they are switched through.
    layouts: These,
    /// Which one is being typed on.
    on: usize,
    /// Which key composes.
    compose: ComposeKey,
    /// The ways of writing a keyboard cannot type.
    methods: Vec<Writing>,
}

impl Keyboards {
    /// What the release ships: one keyboard, no compose key, no input method.
    #[must_use]
    pub fn shipped() -> Self {
        Self {
            layouts: These::just(Layout::offered(BEFORE_ANYBODY_SAID, None)),
            on: 0,
            compose: ComposeKey::None,
            methods: Vec::new(),
        }
    }

    /// What is offered to somebody who has just chosen this language.
    ///
    /// The keyboard their language is typed on, and [`Self::shipped`]'s for a
    /// language alo OS has no offer for — a machine that offered no keyboard at
    /// all would be a machine nobody could finish setting up.
    #[must_use]
    pub fn offered_with(language: &Language) -> Self {
        let mut keyboards = Self::shipped();
        if let Some(offer) = offering::offered_with(language) {
            keyboards.layouts = These::just(offer.first());
        }
        keyboards
    }

    /// The keyboards, in the order they are switched through.
    #[must_use]
    pub const fn all(&self) -> &These {
        &self.layouts
    }

    /// The one being typed on.
    #[must_use]
    pub fn in_use(&self) -> &Layout {
        self.layouts.at(self.on)
    }

    /// The mark shown in the status area, or `None` when there is only one
    /// keyboard and nothing to switch to.
    ///
    /// A square with `DE` in it beside a person who types in one language is a
    /// square that says nothing and takes the room of something that would.
    #[must_use]
    pub fn in_the_status_area(&self) -> Option<String> {
        (self.layouts.how_many() > 1).then(|| self.in_use().badge())
    }

    /// Switch to the next keyboard, and to the first again after the last.
    ///
    /// What [`crate::switching`]'s one shortcut does.
    pub fn switch(&mut self) -> &Layout {
        self.on = self.on.saturating_add(1) % self.layouts.how_many();
        self.in_use()
    }

    /// Add a keyboard this machine has.
    ///
    /// # Errors
    /// [`Refused::NotAKeyboardWeHave`] when the rented data does not name it,
    /// and [`Refused::AlreadyOneOfYours`] when it is already in the list.
    /// Nothing is added in either case.
    pub fn add(&mut self, layout: Layout, rented: &Rented) -> Result<(), Refused> {
        if !rented.has(&layout) {
            return Err(Refused::NotAKeyboardWeHave(layout));
        }
        if self.layouts.holds(&layout) {
            return Err(Refused::AlreadyOneOfYours(layout));
        }
        self.layouts.add(layout);
        Ok(())
    }

    /// Add the keyboard this language is typed on, without anybody having to
    /// know what it is called.
    ///
    /// # Errors
    /// [`Refused::NoKeyboardForThisLanguage`] when alo OS has no offer for it,
    /// and whatever [`Self::add`] refuses.
    pub fn add_for(&mut self, language: &Language, rented: &Rented) -> Result<Layout, Refused> {
        let offer = offering::offered_with(language)
            .ok_or_else(|| Refused::NoKeyboardForThisLanguage(language.clone()))?;
        let layout = offer.first();
        self.add(layout.clone(), rented)?;
        Ok(layout)
    }

    /// Remove a keyboard.
    ///
    /// # Errors
    /// [`Refused::NotOneOfYours`] when it is not in the list, and
    /// **[`Refused::TheLastKeyboard`] rather than leaving a person with no way
    /// to type at all**.
    pub fn remove(&mut self, layout: &Layout) -> Result<(), Refused> {
        let Some(place) = self.layouts.where_is(layout) else {
            return Err(Refused::NotOneOfYours(layout.clone()));
        };
        if !self.layouts.take_out(place) {
            return Err(Refused::TheLastKeyboard);
        }
        self.on = self.on.min(self.layouts.how_many().saturating_sub(1));
        Ok(())
    }

    /// Which key composes.
    #[must_use]
    pub const fn compose_key(&self) -> ComposeKey {
        self.compose
    }

    /// Make this key the compose key, or [`ComposeKey::None`] for no compose
    /// key at all.
    pub fn composes_with(&mut self, key: ComposeKey) {
        self.compose = key;
    }

    /// The ways of writing this person has added.
    #[must_use]
    pub fn input_methods(&self) -> &[Writing] {
        &self.methods
    }

    /// Add a way of typing this language, without anybody having to know what
    /// the engine is called.
    ///
    /// # Errors
    /// **[`Refused::TypedOnAKeyboard`] when the language is typed on a
    /// keyboard** — which is the answer for every one of the 24, and is a
    /// refusal rather than a silent keyboard because somebody looking for Greek
    /// in this list needs to be sent to the right place.
    /// [`Refused::NoKeyboardForThisLanguage`] when alo OS knows neither, and
    /// [`Refused::AlreadyTyping`] when it is already set up.
    pub fn type_language(&mut self, language: &Language) -> Result<Writing, Refused> {
        if let Some(offer) = offering::offered_with(language) {
            return Err(Refused::TypedOnAKeyboard {
                language: language.clone(),
                keyboard: offer.first(),
            });
        }
        let Some(writing) = Writing::for_language(language) else {
            return Err(Refused::NoKeyboardForThisLanguage(language.clone()));
        };
        if self.methods.contains(&writing) {
            return Err(Refused::AlreadyTyping(writing));
        }
        self.methods.push(writing);
        Ok(writing)
    }

    /// Stop typing that way.
    ///
    /// Answers whether it was one of the person's; there is no last input
    /// method to protect, because a person who removes the last one still has
    /// a keyboard.
    pub fn stop_typing(&mut self, writing: Writing) -> bool {
        let before = self.methods.len();
        self.methods.retain(|one| *one != writing);
        self.methods.len() != before
    }

    /// The rented framework's command, when this person's keyboards need it
    /// started with their session, and `None` when they do not.
    #[must_use]
    pub fn what_the_session_starts(&self) -> Option<&'static str> {
        (!self.methods.is_empty()).then_some(IT_IS_STARTED_BY)
    }

    /// Only what this person changed, for keeping.
    #[must_use]
    pub fn changes(&self) -> Changes {
        let shipped = Self::shipped();
        Changes {
            layouts: (self.layouts != shipped.layouts).then(|| self.layouts.to_vec()),
            compose: (self.compose != shipped.compose).then_some(self.compose),
            methods: (!self.methods.is_empty()).then(|| self.methods.clone()),
        }
    }

    /// What the release ships, with what this person changed over it.
    ///
    /// A change naming no keyboards at all is the one thing that cannot be
    /// honoured — a person with no keyboard could not undo it — so it leaves
    /// the shipped keyboard in place.
    #[must_use]
    pub fn with(mut self, changes: Changes) -> Self {
        if let Some(layouts) = changes.layouts.and_then(These::of) {
            self.layouts = layouts;
        }
        if let Some(compose) = changes.compose {
            self.compose = compose;
        }
        if let Some(methods) = changes.methods {
            self.methods = methods;
        }
        self.on = 0;
        self
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_language, rented};

    fn de() -> Layout {
        Layout::named("de").unwrap()
    }

    fn gr() -> Layout {
        Layout::named("gr").unwrap()
    }

    /// A machine nobody has told anything has one keyboard and no compose key.
    #[test]
    fn a_machine_that_was_told_nothing_has_one_keyboard() {
        let keyboards = Keyboards::shipped();
        assert_eq!(keyboards.all().how_many(), 1);
        assert_eq!(keyboards.compose_key(), ComposeKey::None);
        assert_eq!(keyboards.in_the_status_area(), None);
        assert_eq!(keyboards.what_the_session_starts(), None);
    }

    /// Choosing a language chooses the keyboard it is typed on.
    #[test]
    fn choosing_a_language_chooses_its_keyboard() {
        assert_eq!(Keyboards::offered_with(&a_language("el")).in_use(), &gr());
        assert_eq!(Keyboards::offered_with(&a_language("de")).in_use(), &de());
        assert_eq!(
            Keyboards::offered_with(&a_language("is")).in_use(),
            &Layout::named("us").unwrap()
        );
    }

    /// Switching goes round, and the mark appears once there is somewhere to
    /// switch to.
    #[test]
    fn switching_goes_round_and_the_mark_appears_with_the_second_keyboard() {
        let rented = rented();
        let mut keyboards = Keyboards::offered_with(&a_language("de"));
        assert_eq!(keyboards.in_the_status_area(), None);
        keyboards.add(gr(), &rented).unwrap();
        assert_eq!(keyboards.in_the_status_area().as_deref(), Some("DE"));
        assert_eq!(keyboards.switch(), &gr());
        assert_eq!(keyboards.in_the_status_area().as_deref(), Some("GR"));
        assert_eq!(keyboards.switch(), &de());
    }

    /// **A keyboard this machine does not have is refused**, and one already in
    /// the list is not added twice.
    #[test]
    fn a_keyboard_this_machine_does_not_have_is_refused() {
        let rented = rented();
        let mut keyboards = Keyboards::offered_with(&a_language("de"));
        let made_up = Layout::named("qwertz").unwrap();
        assert_eq!(
            keyboards.add(made_up.clone(), &rented),
            Err(Refused::NotAKeyboardWeHave(made_up))
        );
        assert_eq!(
            keyboards.add(de(), &rented),
            Err(Refused::AlreadyOneOfYours(de()))
        );
        assert_eq!(keyboards.all().to_vec(), vec![de()]);
    }

    /// **The last keyboard is not removed**, and a keyboard that is not the
    /// person's is not removed either.
    #[test]
    fn the_last_keyboard_stays() {
        let rented = rented();
        let mut keyboards = Keyboards::offered_with(&a_language("de"));
        assert_eq!(keyboards.remove(&de()), Err(Refused::TheLastKeyboard));
        assert_eq!(keyboards.remove(&gr()), Err(Refused::NotOneOfYours(gr())));
        keyboards.add(gr(), &rented).unwrap();
        keyboards.switch();
        assert_eq!(keyboards.remove(&gr()), Ok(()));
        assert_eq!(keyboards.all().to_vec(), vec![de()]);
        assert_eq!(keyboards.in_use(), &de());
    }

    /// Adding by language never asks a person for a layout's name.
    #[test]
    fn a_keyboard_is_added_by_naming_a_language() {
        let rented = rented();
        let mut keyboards = Keyboards::offered_with(&a_language("de"));
        assert_eq!(keyboards.add_for(&a_language("el"), &rented), Ok(gr()));
        assert_eq!(
            keyboards.add_for(&a_language("is"), &rented),
            Err(Refused::NoKeyboardForThisLanguage(a_language("is")))
        );
    }

    /// An input method is added by naming a language, and nothing is started
    /// until one is.
    #[test]
    fn an_input_method_is_added_by_naming_a_language() {
        let mut keyboards = Keyboards::shipped();
        assert_eq!(keyboards.what_the_session_starts(), None);
        assert_eq!(
            keyboards.type_language(&a_language("ja")),
            Ok(Writing::Japanese)
        );
        assert_eq!(keyboards.input_methods(), &[Writing::Japanese]);
        assert_eq!(keyboards.what_the_session_starts(), Some(IT_IS_STARTED_BY));
        assert_eq!(
            keyboards.type_language(&a_language("ja")),
            Err(Refused::AlreadyTyping(Writing::Japanese))
        );
        assert!(keyboards.stop_typing(Writing::Japanese));
        assert!(!keyboards.stop_typing(Writing::Japanese));
        assert_eq!(keyboards.what_the_session_starts(), None);
    }

    /// **A language typed on a keyboard is sent to the keyboard**, not given an
    /// input method it does not need.
    #[test]
    fn a_language_typed_on_a_keyboard_is_sent_to_the_keyboard() {
        let mut keyboards = Keyboards::shipped();
        assert_eq!(
            keyboards.type_language(&a_language("el")),
            Err(Refused::TypedOnAKeyboard {
                language: a_language("el"),
                keyboard: gr(),
            })
        );
        assert!(keyboards.input_methods().is_empty());
        assert_eq!(
            keyboards.type_language(&a_language("is")),
            Err(Refused::NoKeyboardForThisLanguage(a_language("is")))
        );
    }

    /// Only the difference is kept, and what the release ships keeps nothing.
    #[test]
    fn only_the_difference_is_kept() {
        let rented = rented();
        assert_eq!(Keyboards::shipped().changes(), Changes::default());
        let mut keyboards = Keyboards::offered_with(&a_language("de"));
        keyboards.add(gr(), &rented).unwrap();
        keyboards.composes_with(ComposeKey::Menu);
        keyboards.type_language(&a_language("ko")).unwrap();
        let changes = keyboards.changes();
        assert_eq!(changes.layouts, Some(vec![de(), gr()]));
        assert_eq!(changes.compose, Some(ComposeKey::Menu));
        assert_eq!(changes.methods, Some(vec![Writing::Korean]));
        let again = Keyboards::shipped().with(changes);
        assert_eq!(again.all(), keyboards.all());
        assert_eq!(again.compose_key(), ComposeKey::Menu);
        assert_eq!(again.input_methods(), keyboards.input_methods());
        assert_eq!(again.in_use(), &de());
    }

    /// **A change naming no keyboards leaves the shipped one**, because a
    /// person with none could not undo it.
    #[test]
    fn a_change_with_no_keyboards_in_it_leaves_one() {
        let none = Changes {
            layouts: Some(Vec::new()),
            ..Changes::default()
        };
        assert_eq!(Keyboards::shipped().with(none).all().how_many(), 1);
    }
}
