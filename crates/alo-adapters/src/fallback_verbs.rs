//! The accessibility fallback's two verbs, declared.
//!
//! `docs/features.md` [v0.5]: *the accessibility fallback: any application with
//! no adapter is still readable and operable through its AT-SPI tree*, and the
//! contract's table puts the tree just under an application's own automation
//! interface. Two verbs and no more:
//!
//! - **`accessible.read_window`**, a **read**: what an application's windows
//!   show — words, controls, fields — at that moment, for that turn. It answers
//!   inside the turn as every read does (ADR 0001 §5), and like every read it
//!   reaches only what is granted: it takes the application, and a grant over
//!   that application is required;
//! - **`accessible.activate_control`**, a **change**: press one control, named
//!   by its kind and the name it shows, in one application. It waits for one
//!   approval of a sentence that names the kind, the name and the application —
//!   *press the button named “Sign in” in org.example.Mail* — and one approval
//!   presses once.
//!
//! **Neither takes a position, and neither takes text to type.** The kind is one
//! of [`Pressable`]'s seven, offered as a choice; the name is one name, matched
//! exactly against what the application shows. There is no argument in which a
//! place on the screen, a key or a string to enter could arrive, so the
//! fallback cannot be made into synthetic input by what an agent sends.
//!
//! The verbs are named under `accessible`, which no adapter can be called:
//! [`crate::load`] refuses an adapter of that name
//! ([`NotLoaded::TheFallbacksName`]), so no adapter verb can ever sit beside
//! these under the same prefix.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, Verbs};

use crate::fallback_words as words;
use crate::not_loaded::NotLoaded;
use crate::pressable::Pressable;

/// The prefix both verbs are named under.
pub const PREFIX: &str = "accessible";

/// The verb that reads what an application's windows show.
pub const READ_WINDOW: &str = "accessible.read_window";

/// The verb that presses one control.
pub const ACTIVATE_CONTROL: &str = "accessible.activate_control";

/// The argument naming the application, in both verbs.
pub const APPLICATION: &str = "application";

/// The argument naming the kind of control.
pub const KIND: &str = "kind";

/// The argument naming the control.
pub const NAME: &str = "name";

/// The longest name a control is asked for by.
pub const LONGEST_NAME: usize = 200;

/// The two verbs, as a list of their own.
///
/// # Errors
/// [`NotLoaded`], which the declarations as written cannot cause — a test at
/// the bottom of this file declares them.
pub fn fallback_verbs() -> Result<Verbs, NotLoaded> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the two verbs on an existing list.
///
/// # Errors
/// [`NotLoaded::Taken`] if the list already has a verb of one of these names;
/// [`NotLoaded::Verb`] if a declaration broke the verb contract.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), NotLoaded> {
    let checked = |name: &str, result: Result<Verb, _>| {
        result.map_err(|why| NotLoaded::Verb {
            verb: name.to_owned(),
            why,
        })
    };
    verbs.declare(checked(
        READ_WINDOW,
        Verb::checked(
            READ_WINDOW,
            words::READ_PURPOSE,
            Effect::Read,
            vec![Arg::taking(
                APPLICATION,
                words::READ_APPLICATION,
                Takes::Application,
            )],
            Requires::grants_over([APPLICATION]),
            words::READ_SENTENCE,
        ),
    )?)?;
    verbs.declare(checked(
        ACTIVATE_CONTROL,
        Verb::checked(
            ACTIVATE_CONTROL,
            words::PRESS_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, words::PRESS_APPLICATION, Takes::Application),
                Arg::taking(KIND, words::PRESS_KIND, Pressable::offered()),
                Arg::taking(NAME, words::PRESS_NAME, Takes::name(LONGEST_NAME)),
            ],
            Requires::grants_over([APPLICATION]),
            words::PRESS_SENTENCE,
        ),
    )?)?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_read_and_a_change_each_over_a_granted_application() {
        let verbs = fallback_verbs().unwrap();
        let read = verbs.of(READ_WINDOW).unwrap();
        assert_eq!(read.effect(), Effect::Read);
        let press = verbs.of(ACTIVATE_CONTROL).unwrap();
        assert_eq!(press.effect(), Effect::Change);
        for verb in [read, press] {
            assert!(matches!(
                verb.requires(),
                Requires::Grants(over) if over == &[APPLICATION.to_owned()]
            ));
        }
        assert!(matches!(
            declare_into(&mut fallback_verbs().unwrap()),
            Err(NotLoaded::Taken(_))
        ));
    }
}
