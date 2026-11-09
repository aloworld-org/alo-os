//! What a command is, and the three shapes refused before anybody is asked.

use super::*;
use crate::testing::in_english;

/// A command is kept exactly as it was written, and the person's sentence
/// carries it whole.
#[test]
fn a_command_is_kept_as_written_and_shown_whole() {
    let command = ACommand::of("mkdir -p /var/home/alo/Documents/demo").unwrap();
    assert_eq!(command.text(), "mkdir -p /var/home/alo/Documents/demo");
    assert_eq!(command.program(), "mkdir");
    assert_eq!(
        command.arguments(),
        [
            "-c".to_owned(),
            "mkdir -p /var/home/alo/Documents/demo".to_owned()
        ]
    );
    let said = command.sentence(&in_english());
    assert_eq!(
        said.text(),
        "run this command: mkdir -p /var/home/alo/Documents/demo"
    );
    assert!(!said.is_a_bug(), "{said:?}");
}

/// **Empty, too long and more than one line are refused**, each in its own
/// words, and none of them is a sentence this crate did not declare.
#[test]
fn an_empty_a_long_and_a_two_line_command_are_refused() {
    let strings = in_english();
    for (line, refused) in [
        ("", NotACommand::Empty),
        ("   \t ", NotACommand::Empty),
        ("echo one\necho two", NotACommand::NotOneLine),
        ("echo one\recho two", NotACommand::NotOneLine),
        ("echo \0", NotACommand::NotOneLine),
    ] {
        assert_eq!(ACommand::of(line), Err(refused), "{line:?}");
        assert!(!refused.said(&strings).is_a_bug(), "{refused:?}");
    }
    let long = "x".repeat(THE_LONGEST_COMMAND + 1);
    assert_eq!(ACommand::of(&long), Err(NotACommand::TooLong));
    let said = NotACommand::TooLong.said(&strings);
    assert!(said.text().contains("4096"), "{said:?}");
    assert!(ACommand::of(&"x".repeat(THE_LONGEST_COMMAND)).is_ok());
}
