//! A name the model sends is the identifier of the application it is told by
//! that name — at full access, and only where it is told that name.

use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};
use alo_capability::Given;
use alo_instructing::{TheApplicationsInstalled, WhatItMayReach};

use super::AsTheMachineNames;
use crate::testing::the_verbs;

const EDITOR: &str = "com.vscodium.codium";

fn told(name: Option<&str>, at_full_access: bool) -> WhatItMayReach {
    WhatItMayReach::nothing()
        .at_full_access(at_full_access)
        .with_these_applications(Some(TheApplicationsInstalled::of([(EDITOR, name)])))
}

fn named(machine: &WhatItMayReach, said: &str) -> Vec<(&'static str, Given)> {
    AsTheMachineNames::of(machine).naming(
        &the_verbs(),
        OPEN_APPLICATION,
        &[(APPLICATION, Given::text(said))],
    )
}

#[test]
fn a_name_the_model_is_told_is_the_applications_identifier_at_full_access() {
    assert_eq!(
        named(&told(Some("VSCodium"), true), "VSCodium"),
        [(APPLICATION, Given::text(EDITOR))]
    );
}

/// **Not at full access, nothing granted**: the name stays, and the step is
/// held to it.
#[test]
fn a_name_is_left_where_nothing_permits_the_application() {
    assert_eq!(
        named(&told(Some("VSCodium"), false), "VSCodium"),
        [(APPLICATION, Given::text("VSCodium"))]
    );
}

/// **Told by identifier only, the name means nothing**: the machine knows no
/// name it was not told.
#[test]
fn a_name_the_model_was_not_told_is_left() {
    assert_eq!(
        named(&told(None, true), "VSCodium"),
        [(APPLICATION, Given::text("VSCodium"))]
    );
}

#[test]
fn an_identifier_is_itself() {
    assert_eq!(
        named(&told(Some("VSCodium"), true), EDITOR),
        [(APPLICATION, Given::text(EDITOR))]
    );
}

/// **The name in any case**, as the machine matches it.
#[test]
fn a_name_in_another_case_is_the_applications_identifier() {
    assert_eq!(
        named(&told(Some("VSCodium"), true), "vscodium"),
        [(APPLICATION, Given::text(EDITOR))]
    );
}

/// **Two told by one name, neither is named**, as the machine leaves it.
#[test]
fn a_name_two_applications_are_told_by_is_left() {
    let two = WhatItMayReach::nothing()
        .at_full_access(true)
        .with_these_applications(Some(TheApplicationsInstalled::of([
            ("org.gnome.TextEditor", Some("Text Editor")),
            ("org.kde.kwrite", Some("Text Editor")),
        ])));
    assert_eq!(
        named(&two, "Text Editor"),
        [(APPLICATION, Given::text("Text Editor"))]
    );
}
