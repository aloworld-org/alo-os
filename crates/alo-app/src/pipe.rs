//! **A value given as `-` is read from the pipe, one value per line, and each
//! line is one call** (`docs/contracts/terminal-commands.md`, *Pipes*).
//!
//! Ten pictures piped in are ten calls and ten entries, never one call over
//! ten. One that fails does not stop the others. And a value that cannot be
//! read in a sentence — a control character, an escape sequence — is refused
//! on its line, exactly as it would be in an argument, because a value that
//! reaches a terminal or a record is one somebody reads.

use crate::typed::THE_PIPE;

/// Whether a value can be shown in a sentence: no control character in it,
/// which is what every escape sequence begins with.
#[must_use]
pub fn readable(value: &str) -> bool {
    !value.chars().any(char::is_control)
}

/// More than one value was given as the pipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TwoPipes;

/// Which of these values, if any, is the pipe.
///
/// # Errors
/// [`TwoPipes`]: a pipe holds one stream of values, so two of them cannot both
/// be it.
pub fn which_is_the_pipe<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<Option<usize>, TwoPipes> {
    let mut found = None;
    for (at, value) in values.into_iter().enumerate() {
        if value == THE_PIPE && found.replace(at).is_some() {
            return Err(TwoPipes);
        }
    }
    Ok(found)
}

/// The values for every call a command line makes: one set when nothing is
/// piped, and one per line of the pipe when one value is.
pub fn every_call(
    values: &[String],
    pipe: Option<usize>,
    mut next_line: impl FnMut() -> Option<String>,
) -> Vec<Vec<String>> {
    let Some(at) = pipe else {
        return vec![values.to_vec()];
    };
    let mut calls = Vec::new();
    while let Some(line) = next_line() {
        let mut call = values.to_vec();
        if let Some(value) = call.get_mut(at) {
            *value = line;
        }
        calls.push(call);
    }
    calls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_with_a_control_character_is_not_readable() {
        assert!(readable("Save"));
        assert!(readable("Latviešu — ok"));
        assert!(!readable("Save\u{1b}[31m"));
        assert!(!readable("two\nlines"));
        assert!(!readable("tab\there"));
    }

    #[test]
    fn one_pipe_is_found_and_two_are_refused() {
        assert_eq!(which_is_the_pipe(["a", "b"]), Ok(None));
        assert_eq!(which_is_the_pipe(["a", "-"]), Ok(Some(1)));
        assert_eq!(which_is_the_pipe(["-", "-"]), Err(TwoPipes));
    }

    /// **One line, one call**: each line stands in for the pipe, and nothing
    /// is read when nothing is piped.
    #[test]
    fn each_line_of_the_pipe_is_one_call() {
        let values = vec!["kept".to_owned(), "-".to_owned()];
        let mut lines = vec!["one".to_owned(), "two".to_owned()].into_iter();
        assert_eq!(
            every_call(&values, Some(1), || lines.next()),
            vec![
                vec!["kept".to_owned(), "one".to_owned()],
                vec!["kept".to_owned(), "two".to_owned()],
            ]
        );
        let mut asked = false;
        assert_eq!(
            every_call(&values, None, || {
                asked = true;
                None
            }),
            vec![values.clone()]
        );
        assert!(!asked, "the pipe was read for a command that did not name it");
        assert!(every_call(&values, Some(1), || None).is_empty());
    }
}
