//! Telling `flatpak run` to say its instance, and reading what it says.

use super::*;

/// VSCodium's entry, as Flathub writes its `Exec`.
fn vscodium() -> Vec<String> {
    [
        "/usr/bin/flatpak",
        "run",
        "--branch=stable",
        "--arch=x86_64",
        "--command=/app/bin/codium",
        "--file-forwarding",
        "com.vscodium.codium",
        "@@",
        "/var/home/alo/hello.txt",
        "@@",
    ]
    .map(str::to_owned)
    .to_vec()
}

/// **The option goes straight after `run`**, where `flatpak run` reads its
/// own options, and nothing else of the command moves.
///
/// What would have to be true for this to pass while broken: Flatpak reading
/// the option somewhere other than among `run`'s own, which its source
/// (`flatpak-builtins-run.c:181`) does not.
#[test]
fn flatpak_run_is_told_to_say_its_instance_straight_after_run() {
    let told = told_its_instance(&vscodium(), 9).unwrap_or_default();
    let mut expected = vscodium();
    expected.insert(2, "--instance-id-fd=9".to_owned());
    assert_eq!(told, expected);
}

/// **A command that is not `flatpak run` is told nothing**: it has no
/// instance to say, and an option it does not know could be read as a file.
#[test]
fn a_command_that_is_not_flatpak_run_is_told_nothing() {
    for command in [
        vec!["/usr/bin/gnome-text-editor".to_owned()],
        vec!["/usr/bin/flatpak".to_owned(), "info".to_owned()],
        vec!["/usr/bin/notflatpak".to_owned(), "run".to_owned()],
        Vec::new(),
    ] {
        assert_eq!(told_its_instance(&command, 9), None, "{command:?}");
    }
    // Flatpak refuses 0, 1 and 2; the first it takes is 3.
    for refused in [-1, 0, 2] {
        assert_eq!(told_its_instance(&vscodium(), refused), None, "{refused}");
    }
    assert!(told_its_instance(&vscodium(), 3).is_some());
}

/// **Only a number is an instance**, as Flatpak writes one.
#[test]
fn only_a_number_is_an_instance() {
    assert_eq!(
        an_instance_number(b"2042859277"),
        Some("2042859277".to_owned())
    );
    assert_eq!(
        an_instance_number(b"2042859277\n"),
        Some("2042859277".to_owned())
    );
    let not_ones: [&[u8]; 6] = [
        b"",
        b"12a",
        b"../x",
        b"1 2",
        b"123456789012345678901",
        b"\xff",
    ];
    for not_one in not_ones {
        assert_eq!(an_instance_number(not_one), None, "{not_one:?}");
    }
}

/// **The scope is the one Flatpak names, and the one the person's own
/// counting already knows as that application's** — so naming it here cannot
/// drift from reading it there.
#[test]
fn the_scope_is_the_one_flatpak_names() {
    assert_eq!(
        the_scope_of_an_instance("com.vscodium.codium", "2042859277").as_deref(),
        Some("app-flatpak-com.vscodium.codium-2042859277.scope")
    );
    assert_eq!(
        the_scope_of_an_instance("io.gitlab.librewolf-community", "7").as_deref(),
        Some("app-flatpak-io.gitlab.librewolf\\x2dcommunity-7.scope")
    );
    let scope = the_scope_of_an_instance("com.vscodium.codium", "2042859277").unwrap_or_default();
    assert!(
        crate::is_a_unit_of(&scope, "com.vscodium.codium"),
        "{scope}"
    );
    assert_eq!(
        the_scope_of_an_instance("com.vscodium.codium", "../../x"),
        None
    );
}
