use alo_capability::Given;

use super::*;
use crate::fallback_verbs::{KIND, fallback_verbs};

const GAME: &str = "net.supertuxkart.SuperTuxKart";
const BROWSER: &str = "org.mozilla.firefox";

fn in_english() -> Strings {
    Strings::of(crate::words::adapter_words().unwrap())
}

fn a_press_of(application: &str, name: &str) -> Call {
    fallback_verbs()
        .unwrap()
        .call(
            ACTIVATE_CONTROL,
            &[
                (APPLICATION, Given::text(application)),
                (KIND, Given::text("button")),
                (NAME, Given::text(name)),
            ],
        )
        .unwrap()
}

/// **Named where nothing is named: refused, in words that send alo to a
/// picture**, naming the application and the name it gave.
#[test]
fn a_control_named_where_nothing_is_named_is_refused_with_the_road_to_a_picture() {
    let said = named_where_nothing_is_named(
        &a_press_of(GAME, "Start Race"),
        &[GAME.to_owned()],
        &in_english(),
    )
    .unwrap();
    let expected = in_english()
        .say(
            &words::NAMES_NOTHING.key(),
            &Filling::of("application", GAME.to_owned()).and("name", "Start Race".to_owned()),
        )
        .into_text();
    assert_eq!(said.text(), expected);
    assert!(said.text().contains("Take a picture"), "{said}");
}

/// **Every other call is left alone**: a window that names its controls,
/// another application, and a verb that names no control.
#[test]
fn every_other_call_is_left_alone() {
    let strings = in_english();
    assert!(named_where_nothing_is_named(&a_press_of(GAME, "Start Race"), &[], &strings).is_none());
    assert!(
        named_where_nothing_is_named(&a_press_of(BROWSER, "Reload"), &[GAME.to_owned()], &strings)
            .is_none()
    );
    let a_read = fallback_verbs()
        .unwrap()
        .call(
            crate::fallback_verbs::READ_WINDOW,
            &[(APPLICATION, Given::text(GAME))],
        )
        .unwrap();
    assert!(named_where_nothing_is_named(&a_read, &[GAME.to_owned()], &strings).is_none());
}
