//! Desktop entries written to real directories, read back as a list.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// Write these entries into a directory of their own, and return it.
///
/// **Named from the contents, not from a count.** The first version keyed the
/// directory on the number of entries and the first filename, so the two calls
/// in `the_persons_own_directory_wins` — one entry each, both
/// `browser.desktop` — produced the **same** directory and the second
/// overwrote the first. The test then failed for a reason that had nothing to
/// do with the code it was about.
fn a_place_holding(entries: &[(&str, &str)]) -> std::path::PathBuf {
    let mut fingerprint: u64 = 1469598103934665603;
    for (named, text) in entries {
        for byte in named.bytes().chain(text.bytes()) {
            fingerprint ^= u64::from(byte);
            fingerprint = fingerprint.wrapping_mul(1099511628211);
        }
    }
    let at = std::env::temp_dir().join(format!(
        "alo-installed-{}-{fingerprint:x}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        let path = at.join(named);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, text).unwrap();
    }
    at
}

const A_BROWSER: &str = "[Desktop Entry]\nType=Application\nName=Browser\nExec=browser %u\n";

/// **An entry on the machine is an application in the list.**
#[test]
fn what_is_declared_is_what_is_held() {
    let at = a_place_holding(&[("browser.desktop", A_BROWSER)]);
    let installed = read(std::slice::from_ref(&at));
    assert_eq!(
        installed.how_many(),
        1,
        "one entry did not make one application"
    );
    assert!(
        installed.has("browser"),
        "the identifier is not the file's own stem"
    );
    let _ = std::fs::remove_dir_all(at);
}

/// **`NoDisplay` and `Hidden` keep an entry out, and `Type` other than
/// `Application` does too.**
///
/// Three refusals rather than one test each, because what matters is that the
/// list a person sees is the list the entries asked to be in.
#[test]
fn what_asks_not_to_be_listed_is_not_listed() {
    let at = a_place_holding(&[
        ("shown.desktop", A_BROWSER),
        (
            "helper.desktop",
            "[Desktop Entry]\nType=Application\nName=Helper\nNoDisplay=true\n",
        ),
        (
            "masked.desktop",
            "[Desktop Entry]\nType=Application\nName=Masked\nHidden=true\n",
        ),
        (
            "a-link.desktop",
            "[Desktop Entry]\nType=Link\nName=Somewhere\nURL=https://example.invalid\n",
        ),
    ]);
    let installed = read(std::slice::from_ref(&at));
    assert_eq!(
        installed.how_many(),
        1,
        "something that asked not to be listed was listed: {:?}",
        installed
            .all()
            .map(Application::identifier)
            .collect::<Vec<_>>()
    );
    assert!(installed.has("shown"));
    let _ = std::fs::remove_dir_all(at);
}

/// **An earlier directory wins**, which is what lets a person override the
/// system's copy of an application.
///
/// The reverse would silently ignore their own entry, which is the failure
/// this ordering exists to prevent.
#[test]
fn the_persons_own_directory_wins() {
    let theirs = a_place_holding(&[(
        "browser.desktop",
        "[Desktop Entry]\nType=Application\nName=Their Browser\n",
    )]);
    let system = a_place_holding(&[(
        "browser.desktop",
        "[Desktop Entry]\nType=Application\nName=System Browser\n",
    )]);
    let installed = read(&[theirs.clone(), system.clone()]);
    assert_eq!(
        installed.how_many(),
        1,
        "one identifier became two applications"
    );
    let held = installed.knows("browser").unwrap();
    assert!(
        format!("{held:?}").contains("Their Browser"),
        "the system's entry won over the person's own: {held:?}"
    );
    let _ = std::fs::remove_dir_all(theirs);
    let _ = std::fs::remove_dir_all(system);
}

/// **An entry in a subdirectory is identified with a hyphen**, which is the
/// specification's desktop file ID and not a convenience.
#[test]
fn a_subdirectory_becomes_part_of_the_identifier() {
    let at = a_place_holding(&[("kde4/konsole.desktop", A_BROWSER)]);
    let installed = read(std::slice::from_ref(&at));
    assert!(
        installed.has("kde4-konsole"),
        "a nested entry was not identified by its path: {:?}",
        installed
            .all()
            .map(Application::identifier)
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(at);
}

/// **A `Name` belonging to an action is not the application's name.**
///
/// A desktop file carries one group per action, each with its own `Name`.
/// Reading the first `Name` anywhere would call an application after one of
/// its right-click items.
#[test]
fn an_actions_name_is_not_the_applications() {
    let at = a_place_holding(&[(
        "player.desktop",
        "[Desktop Entry]\nType=Application\nName=Player\n\n[Desktop Action Next]\nName=Next Track\n",
    )]);
    let installed = read(std::slice::from_ref(&at));
    let held = installed.knows("player").unwrap();
    assert!(
        format!("{held:?}").contains("Player") && !format!("{held:?}").contains("Next Track"),
        "an action's name reached the application: {held:?}"
    );
    let _ = std::fs::remove_dir_all(at);
}

/// **A directory that is not there is not an error**, because a machine with
/// nothing of its own installed is ordinary.
#[test]
fn a_place_that_does_not_exist_is_empty_rather_than_a_failure() {
    let installed = read(&[std::path::PathBuf::from(
        "/this/does/not/exist/applications",
    )]);
    assert!(installed.is_empty());
}

/// **An empty variable means unset**, which the specification says and which a
/// shell produces by exporting nothing.
///
/// Treating `""` as a directory would read entries out of the process's
/// working directory, which is how a launcher ends up listing whatever
/// happened to be beside the binary.
#[test]
fn an_empty_variable_is_not_a_directory() {
    let places = where_entries_live(
        Some(std::path::Path::new("/home/somebody")),
        Some(std::ffi::OsStr::new("")),
        Some(std::ffi::OsStr::new("")),
    );
    assert!(
        places.iter().all(|place| place.is_absolute()),
        "a relative directory reached the list: {places:?}"
    );
    assert_eq!(
        places.first().unwrap(),
        &std::path::PathBuf::from("/home/somebody/.local/share/applications"),
        "an empty XDG_DATA_HOME did not fall back to the specification's default"
    );
}

/// **The person's own directory is asked first**, which is the whole of why
/// `the_persons_own_directory_wins` can be true.
#[test]
fn the_persons_directory_comes_before_the_systems() {
    let places = where_entries_live(
        Some(std::path::Path::new("/home/somebody")),
        None,
        Some(std::ffi::OsStr::new("/opt/share:/usr/share")),
    );
    assert_eq!(
        places,
        vec![
            std::path::PathBuf::from("/home/somebody/.local/share/applications"),
            std::path::PathBuf::from("/opt/share/applications"),
            std::path::PathBuf::from("/usr/share/applications"),
        ]
    );
}

/// **An application's declared media types are read off the same entry its name
/// is.**
#[test]
fn what_an_application_opens_is_read_from_its_entry() {
    let at = a_place_holding(&[(
        "viewer.desktop",
        "[Desktop Entry]\nType=Application\nName=Viewer\nMimeType=image/png;image/jpeg;\n",
    )]);
    let (installed, declared) = read_with_what_they_open(std::slice::from_ref(&at));
    assert_eq!(installed.how_many(), 1);
    let opens = declared
        .declaring(alo_opening::Kind::PngImage)
        .collect::<Vec<_>>();
    assert_eq!(opens, ["viewer"], "{opens:?}");
    let _ = std::fs::remove_dir_all(&at);
}

/// **An entry with no `MimeType` is installed and opens nothing**, which is
/// most of them.
///
/// A terminal and a settings panel declare none, and leaving them out of what
/// is installed because they open nothing would make the Dock's list disagree
/// with the machine.
#[test]
fn an_application_that_opens_nothing_is_still_installed() {
    let at = a_place_holding(&[(
        "terminal.desktop",
        "[Desktop Entry]\nType=Application\nName=Terminal\n",
    )]);
    let (installed, declared) = read_with_what_they_open(std::slice::from_ref(&at));
    assert_eq!(installed.how_many(), 1, "it is installed");
    for kind in [alo_opening::Kind::Text, alo_opening::Kind::PngImage] {
        assert_eq!(
            declared.declaring(kind).count(),
            0,
            "it declared nothing, so it opens no {kind:?}"
        );
    }
    let _ = std::fs::remove_dir_all(&at);
}

/// **A trailing separator declares nothing extra**, which nearly every real
/// entry has.
///
/// **This holds the outcome and does not guard the filter that produces it, and
/// that was measured rather than assumed.** Deleting
/// `.filter(|it| !it.is_empty())` from the parser fails **no test**, this one
/// included: `alo_applications::media_types::kinds_declared_by` answers no
/// kinds for a media type it does not know, and the empty string is one of
/// those. So the filter is belt and braces over a thing already true one layer
/// down, and the honest shape of this test is a characterisation — a trailing
/// separator is harmless — rather than a guard over the line that makes it so.
///
/// Said here because a test whose name sounds like a guard and is not is worse
/// than no test: the next person deletes the filter, sees green, and concludes
/// the filter was dead rather than redundant.
#[test]
fn a_trailing_separator_declares_nothing_extra() {
    let with = a_place_holding(&[(
        "a.desktop",
        "[Desktop Entry]\nType=Application\nName=A\nMimeType=text/plain;\n",
    )]);
    let without = a_place_holding(&[(
        "b.desktop",
        "[Desktop Entry]\nType=Application\nName=B\nMimeType=text/plain\n",
    )]);
    let (_, one) = read_with_what_they_open(std::slice::from_ref(&with));
    let (_, two) = read_with_what_they_open(std::slice::from_ref(&without));
    assert_eq!(
        one.declaring(alo_opening::Kind::Text).count(),
        two.declaring(alo_opening::Kind::Text).count(),
        "the separator changed what was declared"
    );
    let _ = std::fs::remove_dir_all(&with);
    let _ = std::fs::remove_dir_all(&without);
}

/// **A person's own copy declares what their copy declares.**
///
/// `read`'s rule is that an earlier place wins, and this holds the declarations
/// to the same rule: an application replaced in `~/.local/share/applications`
/// must not keep declaring what the image's copy declared, or *what opens this*
/// answers from a file the person replaced.
#[test]
fn the_persons_own_copy_declares_what_theirs_declares() {
    let theirs = a_place_holding(&[(
        "editor.desktop",
        "[Desktop Entry]\nType=Application\nName=Editor\nMimeType=text/plain;\n",
    )]);
    let shipped = a_place_holding(&[(
        "editor.desktop",
        "[Desktop Entry]\nType=Application\nName=Editor\nMimeType=image/png;\n",
    )]);
    let (installed, declared) = read_with_what_they_open(&[theirs.clone(), shipped.clone()]);
    assert_eq!(installed.how_many(), 1, "one application, not two");
    assert_eq!(
        declared
            .declaring(alo_opening::Kind::Text)
            .collect::<Vec<_>>(),
        ["editor"],
        "their copy opens text"
    );
    assert_eq!(
        declared.declaring(alo_opening::Kind::PngImage).count(),
        0,
        "the shipped copy's declaration must not survive the person replacing it"
    );
    let _ = std::fs::remove_dir_all(&theirs);
    let _ = std::fs::remove_dir_all(&shipped);
}

/// What an entry calls its icon, read off the same walk as everything else.
///
/// **Every one of these failed before the `Icon` arm existed**, which is the
/// only reason to trust them: the walk already carried the field and the
/// parser left it [`None`], so each assertion below ran against a reader that
/// compiled, found the entry, and reported no icon for it.
mod what_an_entry_calls_its_icon {
    use super::{a_place_holding, read_with_their_icons};

    #[test]
    fn an_application_that_names_one_has_it_read_back_by_identifier() {
        let place = a_place_holding(&[(
            "org.gnome.Nautilus.desktop",
            "[Desktop Entry]\nType=Application\nName=Files\nIcon=org.gnome.Nautilus\n",
        )]);
        let (installed, icons) = read_with_their_icons(&[place]);
        assert!(
            installed.has("org.gnome.Nautilus"),
            "the control: the entry counted as installed, so a missing icon below is the \
             icon reading and not the walk"
        );
        assert_eq!(
            icons.named("org.gnome.Nautilus"),
            Some("org.gnome.Nautilus")
        );
    }

    #[test]
    fn an_icon_name_need_not_look_like_an_identifier() {
        // The two shapes the specification allows, and the Dock has to carry
        // both: a bare theme name, and an absolute path to a file.
        let place = a_place_holding(&[
            (
                "firefox.desktop",
                "[Desktop Entry]\nType=Application\nName=Firefox\nIcon=firefox\n",
            ),
            (
                "weather.desktop",
                "[Desktop Entry]\nType=Application\nName=Weather\nIcon=/usr/share/pixmaps/w.png\n",
            ),
        ]);
        let (_installed, icons) = read_with_their_icons(&[place]);
        assert_eq!(icons.named("firefox"), Some("firefox"));
        assert_eq!(icons.named("weather"), Some("/usr/share/pixmaps/w.png"));
    }

    #[test]
    fn an_application_with_no_icon_line_is_installed_and_absent_from_the_icons() {
        let place = a_place_holding(&[(
            "terminal.desktop",
            "[Desktop Entry]\nType=Application\nName=Terminal\n",
        )]);
        let (installed, icons) = read_with_their_icons(&[place]);
        assert!(
            installed.has("terminal"),
            "no icon is not a reason to leave an application out of what is installed"
        );
        assert_eq!(
            icons.named("terminal"),
            None,
            "and it is absent rather than present-and-empty, which is what lets the Dock \
             tell *named nothing* from *named something unreadable*"
        );
    }

    #[test]
    fn an_empty_icon_line_is_no_icon_rather_than_an_icon_called_nothing() {
        let place = a_place_holding(&[(
            "packaged.desktop",
            "[Desktop Entry]\nType=Application\nName=Packaged\nIcon=\n",
        )]);
        let (installed, icons) = read_with_their_icons(&[place]);
        assert!(installed.has("packaged"), "the control: the entry counted");
        assert_eq!(
            icons.named("packaged"),
            None,
            "an empty name would send the lookup hunting for a file called nothing, and \
             report *not found* for what is really *never named*"
        );
    }

    #[test]
    fn an_icon_belonging_to_an_action_is_not_the_applications() {
        // A desktop file carries one group per right-click action, and each
        // may name its own icon. Reading the first Icon anywhere in the file
        // would draw the action's picture on the Dock.
        let place = a_place_holding(&[(
            "browser.desktop",
            "[Desktop Entry]\nType=Application\nName=Browser\n\
             [Desktop Action NewWindow]\nName=New Window\nIcon=window-new\n",
        )]);
        let (installed, icons) = read_with_their_icons(&[place]);
        assert!(installed.has("browser"), "the control: the entry counted");
        assert_eq!(
            icons.named("browser"),
            None,
            "the application named no icon; window-new belongs to one of its actions"
        );
    }

    #[test]
    fn the_applications_own_icon_is_kept_when_an_action_names_one_after_it() {
        // The same trap from the other side: an application that *does* name
        // an icon must keep its own, not take the action's.
        let place = a_place_holding(&[(
            "editor.desktop",
            "[Desktop Entry]\nType=Application\nName=Editor\nIcon=text-editor\n\
             [Desktop Action NewWindow]\nName=New Window\nIcon=window-new\n",
        )]);
        let (_installed, icons) = read_with_their_icons(&[place]);
        assert_eq!(icons.named("editor"), Some("text-editor"));
    }

    #[test]
    fn the_persons_own_directory_names_the_icon() {
        let theirs = a_place_holding(&[(
            "browser.desktop",
            "[Desktop Entry]\nType=Application\nName=Their Browser\nIcon=the-one-they-chose\n",
        )]);
        let shipped = a_place_holding(&[(
            "browser.desktop",
            "[Desktop Entry]\nType=Application\nName=Shipped Browser\nIcon=what-we-shipped\n",
        )]);
        let (_installed, icons) = read_with_their_icons(&[theirs, shipped]);
        assert_eq!(
            icons.named("browser"),
            Some("the-one-they-chose"),
            "earlier directories win, as they already do for the name and the media types"
        );
    }

    #[test]
    fn an_entry_a_person_should_not_see_contributes_no_icon() {
        // The three refusals are the name's and are unchanged; what this adds
        // is that none of them leaks an icon for an application that is not
        // listed. A hidden entry with an icon would put artwork in the map
        // under an identifier nothing on the Dock can match.
        let place = a_place_holding(&[
            (
                "helper.desktop",
                "[Desktop Entry]\nType=Application\nName=Helper\nIcon=helper\nNoDisplay=true\n",
            ),
            (
                "masked.desktop",
                "[Desktop Entry]\nType=Application\nName=Masked\nIcon=masked\nHidden=true\n",
            ),
            (
                "pointer.desktop",
                "[Desktop Entry]\nType=Link\nName=Pointer\nIcon=pointer\n",
            ),
            (
                "real.desktop",
                "[Desktop Entry]\nType=Application\nName=Real\nIcon=real\n",
            ),
        ]);
        let (_installed, icons) = read_with_their_icons(&[place]);
        assert_eq!(
            icons.named("real"),
            Some("real"),
            "the control: a listable entry in the same directory did get its icon, so the \
             three absences below are the refusals and not an empty walk"
        );
        assert_eq!(icons.named("helper"), None);
        assert_eq!(icons.named("masked"), None);
        assert_eq!(icons.named("pointer"), None);
        assert_eq!(icons.how_many(), 1);
    }

    #[test]
    fn a_machine_with_no_entries_names_no_icons() {
        let place = a_place_holding(&[]);
        let (installed, icons) = read_with_their_icons(&[place]);
        assert_eq!(installed.how_many(), 0);
        assert!(icons.is_empty());
    }
}

/// **The refusal path: an entry that cannot be read is skipped and said.**
///
/// Four broken entries beside one good one. The good one is the control — it
/// is still read, so nothing here was fatal — and each broken one is named
/// with its own reason. The entry that asked not to be listed is the second
/// control: it is the entry working, and it is not a refusal.
#[test]
fn an_entry_that_will_not_read_is_skipped_and_said() {
    let at = a_place_holding(&[
        ("browser.desktop", A_BROWSER),
        ("no-group.desktop", "Type=Application\nName=Nowhere\n"),
        ("no-name.desktop", "[Desktop Entry]\nType=Application\n"),
        (
            "helper.desktop",
            "[Desktop Entry]\nType=Application\nName=Helper\nNoDisplay=true\n",
        ),
    ]);
    // Not text at all: a desktop entry is UTF-8, and this is not.
    std::fs::write(at.join("not-text.desktop"), [0xff_u8, 0xfe, 0x00, 0x9f]).unwrap();
    // A flatpak export left behind by an application half removed: a link to
    // a file that is gone.
    #[cfg(unix)]
    std::os::unix::fs::symlink(at.join("gone"), at.join("dangling.desktop")).unwrap();

    let (installed, _icons, not_read) =
        read_with_their_icons_saying_what_did_not(std::slice::from_ref(&at));

    assert!(
        installed.has("browser"),
        "the good entry beside the broken ones was lost, so a broken one was fatal"
    );
    assert_eq!(installed.how_many(), 1);
    let why = |name: &str| {
        not_read
            .iter()
            .find(|refused| refused.path() == at.join(name))
            .map(NotRead::why)
    };
    assert_eq!(why("no-group.desktop"), Some(WhyNotRead::NoEntryInIt));
    assert_eq!(why("no-name.desktop"), Some(WhyNotRead::NoName));
    assert_eq!(
        why("not-text.desktop"),
        Some(WhyNotRead::WouldNotOpen(std::io::ErrorKind::InvalidData))
    );
    #[cfg(unix)]
    assert_eq!(
        why("dangling.desktop"),
        Some(WhyNotRead::WouldNotOpen(std::io::ErrorKind::NotFound))
    );
    assert_eq!(
        why("helper.desktop"),
        None,
        "an entry asking not to be listed is the entry working, not a refusal"
    );
    let expected = if cfg!(unix) { 4 } else { 3 };
    assert_eq!(not_read.len(), expected, "{not_read:?}");
    let _ = std::fs::remove_dir_all(at);
}

/// **A place that is not there says nothing**: a machine with no flatpak
/// installed yet has no export directory, and that is not a refusal.
#[test]
fn a_place_that_is_not_there_is_not_a_refusal() {
    let nowhere =
        std::env::temp_dir().join(format!("alo-installed-{}-not-there", std::process::id()));
    let _ = std::fs::remove_dir_all(&nowhere);
    let (installed, _icons, not_read) = read_with_their_icons_saying_what_did_not(&[nowhere]);
    assert!(installed.is_empty());
    assert!(not_read.is_empty(), "{not_read:?}");
}
