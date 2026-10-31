//! `Exec` lines as the applications on a fresh alo machine write them, and the
//! ones the specification calls invalid.

use super::*;

/// Nothing to fill a code with.
const NOTHING: Filling<'static> = Filling {
    icon: None,
    name: None,
    location: None,
};

/// The words, for an assertion that reads like the line.
fn words(line: &[&str]) -> Result<Vec<String>, WhyNotACommand> {
    Ok(line.iter().map(|it| (*it).to_owned()).collect())
}

#[test]
fn a_plain_line_with_a_file_code_starts_with_no_file() {
    assert_eq!(the_command_in("dolphin %u", &NOTHING), words(&["dolphin"]));
    assert_eq!(
        the_command_in("libreoffice --writer %U", &NOTHING),
        words(&["libreoffice", "--writer"])
    );
}

/// Flatpak's own export: `@@u` and `@@` are its markers for forwarded files,
/// and with no file between them they are passed through untouched — flatpak
/// reads them, not this.
#[test]
fn a_flatpak_line_becomes_flatpak_run() {
    assert_eq!(
        the_command_in(
            "/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=firefox \
             --file-forwarding org.mozilla.firefox @@u %u @@",
            &NOTHING
        ),
        words(&[
            "/usr/bin/flatpak",
            "run",
            "--branch=stable",
            "--arch=x86_64",
            "--command=firefox",
            "--file-forwarding",
            "org.mozilla.firefox",
            "@@u",
            "@@",
        ])
    );
}

#[test]
fn the_codes_that_are_not_files_are_filled() {
    let filling = Filling {
        icon: Some("org.kde.dolphin"),
        name: Some("Dolphin"),
        location: Some("/usr/share/applications/org.kde.dolphin.desktop"),
    };
    assert_eq!(
        the_command_in("app %i --title=%c %k %%done", &filling),
        words(&[
            "app",
            "--icon",
            "org.kde.dolphin",
            "--title=Dolphin",
            "/usr/share/applications/org.kde.dolphin.desktop",
            "%done",
        ])
    );
    assert_eq!(
        the_command_in("app %i %c %k", &NOTHING),
        words(&["app"]),
        "a code with nothing to fill it adds no argument"
    );
}

#[test]
fn the_deprecated_codes_are_removed() {
    assert_eq!(
        the_command_in("app %d %D %n %N %v %m", &NOTHING),
        words(&["app"])
    );
}

#[test]
fn quotes_hold_reserved_characters_and_their_own_escapes() {
    assert_eq!(
        the_command_in(r#"sh -c "echo \"hi\" \$HOME > /dev/null""#, &NOTHING),
        words(&["sh", "-c", r#"echo "hi" $HOME > /dev/null"#])
    );
    assert_eq!(
        the_command_in(r#"app """#, &NOTHING),
        words(&["app", ""]),
        "an empty quoted argument is an argument"
    );
}

/// The key file's escapes come first: `\s` is a space before the quoting rule
/// sees the line, so unquoted it separates two arguments, and four backslashes
/// in the file are one in a quoted argument.
#[test]
fn the_key_files_escapes_are_applied_first() {
    assert_eq!(
        the_command_in(r"app a\sb", &NOTHING),
        words(&["app", "a", "b"])
    );
    assert_eq!(
        the_command_in(r#"app "a\sb""#, &NOTHING),
        words(&["app", "a b"])
    );
    assert_eq!(
        the_command_in(r#"app "a\\\\b""#, &NOTHING),
        words(&["app", r"a\b"])
    );
}

#[test]
fn an_unquoted_reserved_character_is_refused() {
    for (line, letter) in [
        ("app ; rm -rf ~", ';'),
        ("app | other", '|'),
        ("app $HOME", '$'),
        ("app `id`", '`'),
        ("app 'one'", '\''),
        ("app a&b", '&'),
        ("app > out", '>'),
    ] {
        assert_eq!(
            the_command_in(line, &NOTHING),
            Err(WhyNotACommand::ReservedOutsideQuotes(letter)),
            "{line}"
        );
    }
}

#[test]
fn a_quote_never_closed_is_refused() {
    assert_eq!(
        the_command_in(r#"app "never closed"#, &NOTHING),
        Err(WhyNotACommand::UnterminatedQuote)
    );
    assert_eq!(
        the_command_in(r#"app "ends in \"#, &NOTHING),
        Err(WhyNotACommand::UnterminatedQuote)
    );
}

#[test]
fn a_backslash_before_an_unnamed_character_is_refused() {
    assert_eq!(
        the_command_in(r#"app "a\xb""#, &NOTHING),
        Err(WhyNotACommand::BadEscapeInQuotes('x'))
    );
}

#[test]
fn a_field_code_inside_quotes_is_refused() {
    assert_eq!(
        the_command_in(r#"sh -c "open %u""#, &NOTHING),
        Err(WhyNotACommand::FieldCodeInQuotes('u'))
    );
}

#[test]
fn a_whole_argument_code_inside_a_longer_one_is_refused() {
    for code in ['F', 'U', 'i'] {
        assert_eq!(
            the_command_in(&format!("app --files=%{code}"), &NOTHING),
            Err(WhyNotACommand::ListCodeInsideAnArgument(code))
        );
    }
}

#[test]
fn an_undefined_field_code_is_refused() {
    assert_eq!(
        the_command_in("app %x", &NOTHING),
        Err(WhyNotACommand::UnknownFieldCode('x'))
    );
    assert_eq!(
        the_command_in("app --x=%z", &NOTHING),
        Err(WhyNotACommand::UnknownFieldCode('z'))
    );
    assert_eq!(
        the_command_in("app %", &NOTHING),
        Err(WhyNotACommand::LoneFieldCodeMark)
    );
}

#[test]
fn a_line_that_names_nothing_is_refused() {
    assert_eq!(the_command_in("", &NOTHING), Err(WhyNotACommand::Empty));
    assert_eq!(the_command_in("   ", &NOTHING), Err(WhyNotACommand::Empty));
    assert_eq!(
        the_command_in("%u %F", &NOTHING),
        Err(WhyNotACommand::Empty)
    );
    assert_eq!(
        the_command_in(r#""" app"#, &NOTHING),
        Err(WhyNotACommand::Empty),
        "an empty program is not a program"
    );
}
