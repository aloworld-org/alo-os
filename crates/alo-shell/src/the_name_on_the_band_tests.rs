//! The band's two names, and the refusals that keep them apart.

use std::collections::BTreeMap;

use super::*;

fn called() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("org.gnome.TextEditor".to_owned(), "Text Editor".to_owned()),
        ("org.gnome.Notes".to_owned(), "   ".to_owned()),
    ])
}

fn an(app_id: &str) -> alo_dock::AppId {
    alo_dock::AppId::named(app_id).expect("an identifier")
}

#[test]
fn an_application_with_an_entry_is_shown_by_the_name_a_person_reads() {
    assert_eq!(
        the_application_to_show(Some(&an("org.gnome.TextEditor")), &called()).as_deref(),
        Some("Text Editor"),
        "the band must not show org.gnome.TextEditor where its entry names it"
    );
}

#[test]
fn an_application_with_no_entry_is_shown_by_its_app_id() {
    assert_eq!(
        the_application_to_show(Some(&an("com.example.Started")), &called()).as_deref(),
        Some("com.example.Started")
    );
}

#[test]
fn a_blank_name_in_an_entry_is_no_name() {
    assert_eq!(
        the_application_to_show(Some(&an("org.gnome.Notes")), &called()).as_deref(),
        Some("org.gnome.Notes"),
        "a name of spaces would draw a band with nothing a person can read"
    );
}

#[test]
fn a_window_with_no_app_id_has_no_application_name() {
    assert_eq!(the_application_to_show(None, &called()), None);
}

#[test]
fn the_title_is_the_applications_title_and_never_its_class() {
    assert_eq!(
        the_title_to_show(&FrameName::Given("The next chapter".to_owned())),
        Some("The next chapter")
    );
    assert_eq!(
        the_title_to_show(&FrameName::ItsClass("org.gnome.TextEditor".to_owned())),
        None,
        "a window with no title shows its application's name once, not its app_id beside it"
    );
    assert_eq!(the_title_to_show(&FrameName::AnApplication), None);
}
