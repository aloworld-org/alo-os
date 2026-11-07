//! The words on the compositor's door for **`front` — the window a person was
//! working in just before the terminal they are typing in**
//! (`docs/contracts/terminal-commands.md`, *front*).
//!
//! ```text
//! → front
//! ← front 17 org.gnome.TextEditor today.txt   (or: none, unnamed)
//! ```
//!
//! The number is the compositor's own for that window, stable for its life and
//! never reused, so a command that names it a moment later names that window or
//! none — never another. The title is the rest of the line and may be empty.

use alo_applications::Application;

use crate::request::Request;

/// The act, alone on its line.
pub const FRONT: &str = "front";

/// The longest title an answer carries, in characters: the line must fit
/// [`Request::THE_MOST`] with the number and an identifier beside it.
pub const LONGEST_TITLE: usize = 200;

/// Whether a line is the `front` act.
#[must_use]
pub fn is_front(line: &str) -> bool {
    line.trim_end_matches(['\r', '\n']) == FRONT
}

/// What the compositor answers to `front`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InFront {
    /// The window the person was working in before this terminal.
    Window {
        /// The compositor's number for it.
        number: u64,
        /// The application that owns it, by the identifier it gave itself.
        application: String,
        /// Its title, with nothing a line cannot hold.
        title: String,
    },
    /// There is none: nothing had the keyboard before this terminal, or what
    /// has it now is not a terminal at all — a script a timer started, say.
    None,
    /// There is one, and it cannot be named: it gave no application
    /// identifier this machine can read. Refused rather than acted on as
    /// *something*.
    Unnamed,
}

impl InFront {
    /// A window, its title cleaned of control characters and cut to
    /// [`LONGEST_TITLE`]. [`InFront::Unnamed`] when the identifier is not one a
    /// verb could name, because that window cannot be named either.
    #[must_use]
    pub fn window(number: u64, application: &str, title: &str) -> Self {
        if Application::identified(application).is_err() {
            return Self::Unnamed;
        }
        let title: String = title
            .chars()
            .filter(|character| !character.is_control())
            .take(LONGEST_TITLE)
            .collect();
        Self::Window {
            number,
            application: application.to_owned(),
            title: title.trim().to_owned(),
        }
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Window {
                number,
                application,
                title,
            } if title.is_empty() => format!("{FRONT} {number} {application}"),
            Self::Window {
                number,
                application,
                title,
            } => format!("{FRONT} {number} {application} {title}"),
            Self::None => "none".to_owned(),
            Self::Unnamed => "unnamed".to_owned(),
        }
    }

    /// One line, read; [`None`] for anything else, which a caller must never
    /// read as a window.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.len() > Request::THE_MOST {
            return None;
        }
        match line {
            "none" => return Some(Self::None),
            "unnamed" => return Some(Self::Unnamed),
            _ => {}
        }
        let rest = line.strip_prefix(FRONT)?.strip_prefix(' ')?;
        let (number, rest) = rest.split_once(' ').unwrap_or((rest, ""));
        let number = number.parse().ok()?;
        let (application, title) = rest.split_once(' ').unwrap_or((rest, ""));
        match Self::window(number, application, title) {
            Self::Window {
                number,
                application,
                title: cleaned,
            } if cleaned == title => Some(Self::Window {
                number,
                application,
                title: cleaned,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_is_written_and_read_back_whole() {
        let window = InFront::window(17, "org.gnome.TextEditor", "today.txt — Notes");
        assert_eq!(
            window.written(),
            "front 17 org.gnome.TextEditor today.txt — Notes"
        );
        assert_eq!(InFront::read(&window.written()), Some(window));
        let untitled = InFront::window(3, "org.gnome.Loupe", "");
        assert_eq!(untitled.written(), "front 3 org.gnome.Loupe");
        assert_eq!(InFront::read("front 3 org.gnome.Loupe\n"), Some(untitled));
    }

    #[test]
    fn none_and_unnamed_read_back_and_are_never_a_window() {
        assert_eq!(InFront::read("none"), Some(InFront::None));
        assert_eq!(InFront::read("unnamed"), Some(InFront::Unnamed));
        assert_eq!(InFront::read(&InFront::None.written()), Some(InFront::None));
    }

    /// **A title is cleaned, not trusted**: a window that puts an escape
    /// sequence or a line break in its title cannot make an answer say
    /// something else on a terminal.
    #[test]
    fn a_title_loses_what_a_line_cannot_hold() {
        let window = InFront::window(1, "org.example.App", "a\u{1b}[31mb\nc");
        assert_eq!(
            window,
            InFront::Window {
                number: 1,
                application: "org.example.App".to_owned(),
                title: "a[31mbc".to_owned(),
            }
        );
        let long = InFront::window(1, "org.example.App", &"x".repeat(LONGEST_TITLE + 50));
        assert!(matches!(long, InFront::Window { title, .. } if title.chars().count() == LONGEST_TITLE));
    }

    /// **An identifier no verb could name is `unnamed`**, never a window.
    #[test]
    fn an_identifier_that_names_nothing_is_unnamed() {
        assert_eq!(InFront::window(1, "", "x"), InFront::Unnamed);
        assert_eq!(InFront::window(1, "a/b", "x"), InFront::Unnamed);
    }

    /// **Anything else is not an answer**: a caller reading it as a window
    /// would act on one nobody named.
    #[test]
    fn anything_else_is_not_an_answer() {
        for not in [
            "",
            "front",
            "front x org.a.B",
            "front 1",
            "front -1 org.a.B",
            "front 1 a/b title",
            "acted",
            "front 1 org.a.B a\u{1b}b",
        ] {
            assert_eq!(InFront::read(not), None, "{not:?}");
        }
        assert!(is_front("front\n"));
        assert!(!is_front("front org.a.B"));
    }
}
