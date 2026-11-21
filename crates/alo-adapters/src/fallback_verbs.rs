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

use crate::choosing_words as choosing;
use crate::fallback_words as words;
use crate::keys_words as keys;
use crate::menu_words as menu;
use crate::not_loaded::NotLoaded;
use crate::opening_words as opening;
use crate::picture_words as picture;
use crate::pressable::Pressable;
use crate::pressing_a_key::the_keys_offered;
use crate::scrolling::Toward;
use crate::scrolling_words as scrolling;
use crate::setting_words as setting;
use crate::typing_without_saving_words as unsaved;
use crate::typing_words as typing;

/// The prefix both verbs are named under.
pub const PREFIX: &str = "accessible";

/// The verb that reads what an application's windows show.
pub const READ_WINDOW: &str = "accessible.read_window";

/// The verb that presses one control.
pub const ACTIVATE_CONTROL: &str = "accessible.activate_control";

/// The verb that types one approved text into one field, and saves it (ADR
/// 0153).
pub const TYPE_TEXT: &str = "accessible.type_text";

/// The verb that scrolls one named thing onto the screen (2026-11-17).
pub const SCROLL_TO: &str = "accessible.scroll_to";

/// The verb that scrolls by one page, down or up (2026-11-17).
pub const SCROLL_PAGE: &str = "accessible.scroll_page";

/// The argument naming which way a page is scrolled.
pub const TOWARD: &str = "toward";
/// The verb that presses one key of a closed list (2026-11-17).
pub const PRESS_KEY: &str = "accessible.press_key";

/// The argument naming the key.
pub const KEY: &str = "key";

/// The verb that takes one picture of an application's window that describes
/// nothing, and has it read in words by whoever the person chose (ADR 0147 §1,
/// the screen road).
pub const PICTURE_WINDOW: &str = "accessible.picture_window";

/// The argument carrying what the picture is to answer, in the person's words.
///
/// **There for the reader, and for the shape** (2026-11-17): taking the
/// application alone, `accessible.picture_window` had `focus_application`'s
/// shape, and `gpt-4o-mini` sent `accessible.open_menu` naming the photo in 9
/// of 10 tries and the verb as a command line in the tenth
/// (`docs/quirks/two-verbs-taking-the-same-arguments-and-gpt-4o-mini-sends-neither.md`).
pub const QUESTION: &str = "question";

/// The verb that types one approved text into one field and saves nothing
/// (2026-11-17): a search box, a form, an address, where `Ctrl+S` would save
/// the page instead.
pub const TYPE_WITHOUT_SAVING: &str = "accessible.type_without_saving";

/// The verb that moves a slider or a number box to a whole number
/// (2026-11-17, the owner's "Yes do all").
pub const SET_VALUE: &str = "accessible.set_value";

/// The argument carrying the number.
pub const VALUE: &str = "value";

/// The largest number either way `accessible.set_value` takes. The control's
/// own range is what decides at the moment of moving; this only keeps the
/// number one a slider could hold.
pub const LARGEST_VALUE: i64 = 1_000_000_000;
/// The argument naming the thing scrolled to.
///
/// **Not `name`** (2026-11-17), for [`THING`]'s reason: taking `application`
/// and `name` it was a call of `accessible.type_text`'s shape with one argument
/// fewer, and `gpt-4o-mini` sent `type_text` with an empty text and the
/// heading's name in 4 of 20 tries asked to scroll to it.
pub const TO: &str = "to";

/// The verb that opens one thing's right-click menu (2026-11-17).
pub const OPEN_MENU: &str = "accessible.open_menu";

/// The argument naming the thing whose menu is opened.
///
/// **Not `name`**, which `accessible.scroll_to` takes beside `application`
/// and nothing else: told both, `gpt-4o-mini` sent `type_text` in every try
/// asked to open a file's menu, so the two calls are kept apart in shape as
/// well as in name.
pub const THING: &str = "thing";

/// The argument carrying the text, whole.
pub const TEXT: &str = "text";

/// The verb that chooses one option of a drop-down (2026-11-17, the owner's
/// "Yes do all").
pub const CHOOSE_OPTION: &str = "accessible.choose_option";

/// The verb that selects one item of a list, without opening it (2026-11-17).
pub const SELECT_ITEM: &str = "accessible.select_item";

/// The argument naming the option chosen.
pub const OPTION: &str = "option";

/// The verb that opens one item of a list, as a double-click would
/// (2026-11-17, the owner's "Yes but it should be without okay on full
/// access").
pub const OPEN_ITEM: &str = "accessible.open_item";

/// The argument naming the application, in both verbs.
pub const APPLICATION: &str = "application";

/// The argument naming the kind of control.
pub const KIND: &str = "kind";

/// The argument naming the control.
pub const NAME: &str = "name";

/// The argument naming the field `accessible.type_without_saving` types into.
///
/// **Not `name`** (2026-11-17), for [`THING`]'s reason: taking `type_text`'s
/// three arguments exactly, `gpt-4o-mini` sent `type_text` in 10 of 20 tries
/// asked to type into a search box, even told which of the two saves.
pub const FIELD: &str = "field";

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
        )
        // **What it changes and leaves alone**, on the approval card — held to
        // what a press does by `tests/the_accessibility_fallback.rs`.
        .and_then(|verb| verb.changing(words::PRESS_CHANGES, Some(words::PRESS_LEAVES))),
    )?)?;
    verbs.declare(checked(
        TYPE_TEXT,
        Verb::checked(
            TYPE_TEXT,
            typing::TYPE_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, typing::TYPE_APPLICATION, Takes::Application),
                Arg::taking(NAME, typing::TYPE_NAME, Takes::name(LONGEST_NAME)),
                Arg::taking(TEXT, typing::TYPE_THE_TEXT, Takes::Text),
            ],
            Requires::grants_over([APPLICATION]),
            typing::TYPE_SENTENCE,
        )
        .and_then(|verb| verb.changing(typing::TYPE_CHANGES, Some(typing::TYPE_LEAVES))),
    )?)?;
    // **Scrolling, 2026-11-17** (`crate::scrolling`): a change, like a press —
    // it moves what the person's window shows. Neither verb takes a position.
    verbs.declare(checked(
        SCROLL_TO,
        Verb::checked(
            SCROLL_TO,
            scrolling::SCROLL_TO_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(
                    APPLICATION,
                    scrolling::SCROLL_TO_APPLICATION,
                    Takes::Application,
                ),
                // `name` until 2026-11-17, still accepted and deprecated: the
                // verb had landed under it (`docs/contracts/agent-verbs.md`).
                Arg::taking(TO, scrolling::SCROLL_TO_NAME, Takes::name(LONGEST_NAME))
                    .formerly_called(NAME),
            ],
            Requires::grants_over([APPLICATION]),
            scrolling::SCROLL_TO_SENTENCE,
        )
        .and_then(|verb| verb.changing(scrolling::SCROLL_CHANGES, Some(scrolling::SCROLL_LEAVES))),
    )?)?;
    verbs.declare(checked(
        SCROLL_PAGE,
        Verb::checked(
            SCROLL_PAGE,
            scrolling::SCROLL_PAGE_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(
                    APPLICATION,
                    scrolling::SCROLL_PAGE_APPLICATION,
                    Takes::Application,
                ),
                Arg::taking(TOWARD, scrolling::SCROLL_PAGE_TOWARD, Toward::offered()),
            ],
            Requires::grants_over([APPLICATION]),
            scrolling::SCROLL_PAGE_SENTENCE,
        )
        .and_then(|verb| verb.changing(scrolling::SCROLL_CHANGES, Some(scrolling::SCROLL_LEAVES))),
    )?)?;
    verbs.declare(checked(
        SET_VALUE,
        Verb::checked(
            SET_VALUE,
            setting::SET_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, setting::SET_APPLICATION, Takes::Application),
                Arg::taking(NAME, setting::SET_NAME, Takes::name(LONGEST_NAME)),
                Arg::taking(
                    VALUE,
                    setting::SET_VALUE_WORD,
                    Takes::count(-LARGEST_VALUE, LARGEST_VALUE),
                ),
            ],
            Requires::grants_over([APPLICATION]),
            setting::SET_SENTENCE,
        )
        .and_then(|verb| verb.changing(setting::SET_CHANGES, Some(setting::SET_LEAVES))),
    )?)?;
    verbs.declare(checked(
        CHOOSE_OPTION,
        Verb::checked(
            CHOOSE_OPTION,
            choosing::CHOOSE_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(
                    APPLICATION,
                    choosing::CHOOSE_APPLICATION,
                    Takes::Application,
                ),
                Arg::taking(NAME, choosing::CHOOSE_NAME, Takes::name(LONGEST_NAME)),
                Arg::taking(
                    OPTION,
                    choosing::CHOOSE_OPTION_WORD,
                    Takes::name(LONGEST_NAME),
                ),
            ],
            Requires::grants_over([APPLICATION]),
            choosing::CHOOSE_SENTENCE,
        )
        .and_then(|verb| verb.changing(choosing::CHOOSE_CHANGES, Some(choosing::CHOOSE_LEAVES))),
    )?)?;
    verbs.declare(checked(
        SELECT_ITEM,
        Verb::checked(
            SELECT_ITEM,
            choosing::SELECT_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(
                    APPLICATION,
                    choosing::SELECT_APPLICATION,
                    Takes::Application,
                ),
                Arg::taking(NAME, choosing::SELECT_NAME, Takes::name(LONGEST_NAME)),
            ],
            Requires::grants_over([APPLICATION]),
            choosing::SELECT_SENTENCE,
        )
        .and_then(|verb| verb.changing(choosing::SELECT_CHANGES, Some(choosing::SELECT_LEAVES))),
    )?)?;
    verbs.declare(checked(
        OPEN_ITEM,
        Verb::checked(
            OPEN_ITEM,
            opening::OPEN_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, opening::OPEN_APPLICATION, Takes::Application),
                Arg::taking(NAME, opening::OPEN_NAME, Takes::name(LONGEST_NAME)),
            ],
            Requires::grants_over([APPLICATION]),
            opening::OPEN_SENTENCE,
        )
        .and_then(|verb| verb.changing(opening::OPEN_CHANGES, Some(opening::OPEN_LEAVES))),
    )?)?;
    // **A thing's own menu, 2026-11-17** (`crate::opening_a_menu`): a change,
    // by the application's own action, never by a click at a place.
    verbs.declare(checked(
        OPEN_MENU,
        Verb::checked(
            OPEN_MENU,
            menu::MENU_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, menu::MENU_APPLICATION, Takes::Application),
                Arg::taking(THING, menu::MENU_NAME, Takes::name(LONGEST_NAME)),
            ],
            Requires::grants_over([APPLICATION]),
            menu::MENU_SENTENCE,
        )
        .and_then(|verb| verb.changing(menu::MENU_CHANGES, Some(menu::MENU_LEAVES))),
    )?)?;
    // **One key, 2026-11-17** (`crate::pressing_a_key`): a change, one of seven
    // keys offered as a choice, so no combination or letter can arrive.
    verbs.declare(checked(
        PRESS_KEY,
        Verb::checked(
            PRESS_KEY,
            keys::KEY_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, keys::KEY_APPLICATION, Takes::Application),
                Arg::taking(KEY, keys::KEY_KEY, the_keys_offered()),
            ],
            Requires::grants_over([APPLICATION]),
            keys::KEY_SENTENCE,
        )
        .and_then(|verb| verb.changing(keys::KEY_CHANGES, Some(keys::KEY_LEAVES))),
    )?)?;
    // **Typing that saves nothing, 2026-11-17**: `accessible.type_text`'s
    // arguments and checks, without its `Ctrl+S` (`crate::typing_text`).
    verbs.declare(checked(
        TYPE_WITHOUT_SAVING,
        Verb::checked(
            TYPE_WITHOUT_SAVING,
            unsaved::UNSAVED_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(APPLICATION, typing::TYPE_APPLICATION, Takes::Application),
                Arg::taking(FIELD, typing::TYPE_NAME, Takes::name(LONGEST_NAME)),
                Arg::taking(TEXT, typing::TYPE_THE_TEXT, Takes::Text),
            ],
            Requires::grants_over([APPLICATION]),
            unsaved::UNSAVED_SENTENCE,
        )
        .and_then(|verb| verb.changing(unsaved::UNSAVED_CHANGES, Some(typing::TYPE_LEAVES))),
    )?)?;
    // **One picture of a window that describes nothing** (ADR 0147 §1,
    // `crate::picturing_a_window`): a change, so it is one sentence and one OK
    // at *ask each time*, and it takes the application and nothing else — no
    // place, no region, no second window.
    verbs.declare(checked(
        PICTURE_WINDOW,
        Verb::checked(
            PICTURE_WINDOW,
            picture::PICTURE_PURPOSE,
            Effect::Change,
            vec![
                Arg::taking(
                    APPLICATION,
                    picture::PICTURE_APPLICATION,
                    Takes::Application,
                ),
                Arg::taking(QUESTION, picture::PICTURE_QUESTION, Takes::Text),
            ],
            Requires::grants_over([APPLICATION]),
            picture::PICTURE_SENTENCE,
        )
        .and_then(|verb| verb.changing(picture::PICTURE_CHANGES, Some(picture::PICTURE_LEAVES))),
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
