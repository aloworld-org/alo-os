use super::*;

/// A game: an application that draws its own window and describes nothing.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

/// **A machine arrives with it off for every application.**
#[test]
fn a_machine_arrives_with_pictures_and_keys_off_everywhere() {
    let arrives = PicturesAndKeys::default();
    assert!(!arrives.allows(A_GAME));
    assert!(!arrives.allows("org.mozilla.firefox"));
    assert_eq!(arrives.applications().count(), 0);
}

/// **Switched on for one application, it is on for that one alone** — not
/// another, not the same name in another case, not one with space around it.
#[test]
fn switched_on_for_one_application_reaches_that_one_alone() {
    let on = PicturesAndKeys::default().switched_on(A_GAME);
    assert!(on.allows(A_GAME));
    assert!(!on.allows("org.mozilla.firefox"));
    assert!(!on.allows("net.supertuxkart.supertuxkart"));
    assert!(!on.allows(" net.supertuxkart.SuperTuxKart"));
    assert!(!on.allows(""));
    assert_eq!(on.applications().collect::<Vec<_>>(), [A_GAME]);
}

/// **Switched off, it is off again**, and the others stay as they were.
#[test]
fn switched_off_takes_that_one_away_and_keeps_the_rest() {
    let on = PicturesAndKeys::default()
        .switched_on(A_GAME)
        .switched_on("org.mozilla.firefox");
    let off = on.switched_off(A_GAME);
    assert!(!off.allows(A_GAME));
    assert!(off.allows("org.mozilla.firefox"));
}

/// **A terminal never**, whichever way it arrives: switched on, or in a list
/// read from a file.
#[test]
fn a_terminal_is_never_allowed_pictures_and_keys() {
    for terminal in crate::A_PERSONS_OWN {
        let switched = PicturesAndKeys::default().switched_on(terminal);
        assert!(!switched.allows(terminal), "{terminal} was switched on");
        let listed = PicturesAndKeys::of([terminal, A_GAME]);
        assert!(!listed.allows(terminal), "{terminal} was kept from a list");
        assert!(listed.allows(A_GAME));
        assert_eq!(listed.applications().collect::<Vec<_>>(), [A_GAME]);
    }
}

/// **An identifier nothing could be is not kept**: empty, or with space around
/// it, which trimming would turn into a different application.
#[test]
fn an_identifier_nothing_could_be_is_not_kept() {
    let listed = PicturesAndKeys::of(["", " org.mozilla.firefox", "org.gnome.Loupe\n"]);
    assert_eq!(listed.applications().count(), 0);
    let switched = PicturesAndKeys::default().switched_on(" org.mozilla.firefox");
    assert_eq!(switched.applications().count(), 0);
}

/// **A file that will not read lets alo use pictures and keys nowhere.**
#[test]
fn a_file_that_will_not_read_switches_it_on_nowhere() {
    let read: Result<PicturesAndKeys, ()> = Err(());
    assert_eq!(PicturesAndKeys::as_read(read), PicturesAndKeys::default());
    let read: Result<PicturesAndKeys, ()> = Ok(PicturesAndKeys::of([A_GAME]));
    assert!(PicturesAndKeys::as_read(read).allows(A_GAME));
}

/// **What it costs is ADR 0143's own sentence**, kept as it was written.
#[test]
fn what_it_costs_is_the_decisions_own_sentence() {
    assert_eq!(
        PicturesAndKeys::what_it_costs().says(),
        "alo will act on what it sees in pictures of this window, and may press the wrong thing"
    );
}
