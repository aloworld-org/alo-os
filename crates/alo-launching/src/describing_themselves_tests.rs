use super::*;

/// **On, every toolkit is asked**: GTK's two are left to their default, Qt is
/// told always, Firefox and Chromium's window are told yes — and Qt's goes to
/// the manager only.
#[test]
fn on_every_toolkit_is_asked() {
    let told = what_the_session_is_told(true);
    assert_eq!(
        told.everywhere,
        [
            "NO_AT_BRIDGE=0",
            "GTK_A11Y=0",
            "GNOME_ACCESSIBILITY=1",
            "ACCESSIBILITY_ENABLED=1"
        ]
    );
    assert_eq!(told.the_manager_only, ["QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1"]);
    assert!(told.forgotten.is_empty());
}

/// **Off, every toolkit is told no** — including the two that describe
/// themselves unasked, because the switch says *applications*, not *the ones
/// that would not have* — and Qt's switch is forgotten rather than emptied,
/// because Qt reads an empty one as on.
#[test]
fn off_every_toolkit_is_told_no() {
    let told = what_the_session_is_told(false);
    assert_eq!(
        told.everywhere,
        [
            "NO_AT_BRIDGE=1",
            "GTK_A11Y=none",
            "GNOME_ACCESSIBILITY=0",
            "ACCESSIBILITY_ENABLED=0"
        ]
    );
    assert!(told.the_manager_only.is_empty());
    assert_eq!(told.forgotten, [QT_ALWAYS_ON]);
    assert!(
        !told
            .everywhere
            .iter()
            .any(|it| it.starts_with(QT_ALWAYS_ON)),
        "Qt's switch was handed a value, which Qt reads as on"
    );
}

/// **The session bus is never told Qt's switch**, on or off: it could be told
/// a value and never told to forget it, so turning the switch off would leave
/// every application the bus starts asked for ever.
#[test]
fn the_bus_is_never_told_what_it_cannot_forget() {
    for describe in [true, false] {
        assert!(
            !what_the_session_is_told(describe)
                .everywhere
                .iter()
                .any(|it| it.starts_with(QT_ALWAYS_ON)),
            "describe = {describe}"
        );
    }
}

/// **A start is handed every value**, and Qt's only when on.
#[test]
fn a_start_is_handed_what_the_session_is_told() {
    let on = what_a_start_is_handed(true);
    assert!(on.contains(&"QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1".to_owned()));
    assert!(on.contains(&"ACCESSIBILITY_ENABLED=1".to_owned()));
    let off = what_a_start_is_handed(false);
    assert!(off.contains(&"NO_AT_BRIDGE=1".to_owned()));
    assert!(off.contains(&"GTK_A11Y=none".to_owned()));
    assert!(!off.iter().any(|it| it.starts_with(QT_ALWAYS_ON)));
}

/// **Only an application drawn by Chromium gains the switch, and only on.**
/// Handed to anything else it would be read as a file to open.
#[test]
fn only_chromium_gains_the_switch_and_only_on() {
    for identifier in DRAWN_BY_CHROMIUM {
        assert_eq!(what_its_command_gains(identifier, true), [THE_PAGE_TOO]);
        assert!(what_its_command_gains(identifier, false).is_empty());
    }
    for not_chromium in [
        "org.mozilla.firefox",
        "org.libreoffice.LibreOffice",
        "org.gnome.TextEditor",
        "org.kde.dolphin",
        "codium",
    ] {
        assert!(
            what_its_command_gains(not_chromium, true).is_empty(),
            "{not_chromium}"
        );
    }
}

/// **The switch never claims a screen reader is running.** The bare switch
/// does, and VSCodium answers it by changing how its editor behaves.
#[test]
fn the_switch_names_a_mode_and_not_a_screen_reader() {
    let (switch, mode) = THE_PAGE_TOO
        .split_once('=')
        .unwrap_or((THE_PAGE_TOO, ""));
    assert_eq!(switch, "--force-renderer-accessibility");
    assert!(!mode.is_empty(), "the bare switch says a screen reader is running");
    assert_ne!(mode, "screen-reader");
}
