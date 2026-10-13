//! One line in: an act and the application it is about.
//!
//! Two words separated by one space, and nothing else:
//!
//! ```text
//! <act> <application>
//! ```
//!
//! `alo-broker`'s is five because a root service needs an approval number, a
//! moment and a proof. **This one needs none of those**, and ADR 0108 is the
//! argument: the compositor is not privileged, the caller is the person by the
//! kernel's account, and the authority is the turn's — already checked before a
//! line is written.
//!
//! # Why two words and not JSON
//!
//! Because there is nothing to nest. An act is one of three words and an
//! application is one identifier held to its own shape, so a line that parses
//! is a line that is complete — there is no field to forget and no field to
//! smuggle. `alo-broker` chose the same for the same reason one level up.
//!
//! A line that is not exactly two words is not a request, rather than a request
//! with something missing: *read around nothing* is ADR 0031's rule for a body
//! a machine reads, and it is cheaper to keep here than to recover from.

use alo_applications::Application;

use crate::acting::Acting;

/// One thing asked of a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// What is asked.
    acting: Acting,
}

impl Request {
    /// The most a line may be, in bytes.
    ///
    /// An act is at most six characters and an application identifier is bounded
    /// by `alo_applications::Application` itself, so this is generous twice
    /// over. It is here so that a caller cannot make the compositor hold a line
    /// while it reads it — the same reason `alo-protocol` bounds a message, and
    /// a window system is the worst place to be made to wait.
    pub const THE_MOST: usize = 512;

    /// This act, as a request.
    #[must_use]
    pub const fn to(acting: Acting) -> Self {
        Self { acting }
    }

    /// What is asked.
    #[must_use]
    pub const fn acting(&self) -> &Acting {
        &self.acting
    }

    /// The line this crosses as.
    #[must_use]
    pub fn written(&self) -> String {
        format!(
            "{} {}",
            self.acting.on_the_wire(),
            self.acting.application().identifier()
        )
    }

    /// One line, read.
    ///
    /// # Errors
    /// [`NotRead`], which says which of the three things was wrong rather than
    /// *not a request*: whoever is fixing a caller needs to know whether the
    /// shape, the act or the application was the problem.
    pub fn read(line: &str) -> Result<Self, NotRead> {
        if line.len() > Self::THE_MOST {
            return Err(NotRead::TooLong {
                most: Self::THE_MOST,
                was: line.len(),
            });
        }
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        let (Some(act), Some(named), None) = (words.next(), words.next(), words.next()) else {
            return Err(NotRead::NotTwoWords);
        };
        let which = Application::identified(named).map_err(|_| NotRead::NotAnApplication {
            named: named.to_owned(),
        })?;
        let acting = Acting::off_the_wire(act, which).map_err(|why| NotRead::NotAnAct {
            word: why.word().to_owned(),
        })?;
        Ok(Self { acting })
    }
}

/// Why a line is not a request this door can act on.
///
/// English and a `Display`, like [`crate::acting::NotAnAct`]: read by whoever is
/// fixing the caller. **A person never reads one of these** — what they read
/// about an application verb is `alo-applications`' own sentence, decided long
/// before a line reaches this door.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotRead {
    /// Longer than the door will hold.
    #[error("a request is at most {most} bytes and this was {was}")]
    TooLong {
        /// The most a line may be.
        most: usize,
        /// How long it was.
        was: usize,
    },
    /// Not exactly two words.
    #[error("a request is an act and an application, separated by one space")]
    NotTwoWords,
    /// The first word is not one of the three.
    #[error("{word} is not something this door does to a window")]
    NotAnAct {
        /// The word that arrived.
        word: String,
    },
    /// The second word is not an application identifier.
    #[error("{named} is not an application's identifier")]
    NotAnApplication {
        /// What arrived in its place.
        named: String,
    },
}

#[cfg(test)]
// Counted: five `unwrap` and one `panic!`, and no `expect` — so neither more
// nor fewer than the two it uses.
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]
mod tests {
    use super::{NotRead, Request};
    use crate::acting::Acting;
    use alo_applications::Application;

    fn mail() -> Application {
        Application::identified("org.gnome.Geary").unwrap()
    }

    /// **Every act is written and read back as itself**, which is the one
    /// property a door needs before anything else is worth testing.
    #[test]
    fn every_act_is_written_and_read_back_as_itself() {
        for act in [
            Acting::Open(mail()),
            Acting::Focus(mail()),
            Acting::Close(mail()),
        ] {
            let asked = Request::to(act.clone());
            let line = asked.written();
            assert!(!line.contains('\n'), "a request is one line: {line}");
            assert_eq!(Request::read(&line).unwrap(), asked);
            assert_eq!(Request::read(&line).unwrap().acting(), &act);
        }
    }

    /// A line with a newline on it reads the same, because a socket hands one
    /// over with the newline it was sent with.
    #[test]
    fn a_line_off_a_socket_reads_with_its_newline() {
        let asked = Request::to(Acting::Open(mail()));
        assert_eq!(
            Request::read(&format!("{}\n", asked.written())).unwrap(),
            asked
        );
        assert_eq!(
            Request::read(&format!("{}\r\n", asked.written())).unwrap(),
            asked
        );
    }

    /// **Each way a line can be wrong says which way**, rather than *not a
    /// request*: the four are told apart because whoever is fixing a caller
    /// needs to know which.
    #[test]
    fn each_way_a_line_is_wrong_says_which_way() {
        assert_eq!(Request::read("open").unwrap_err(), NotRead::NotTwoWords);
        assert_eq!(
            Request::read("open org.gnome.Geary extra").unwrap_err(),
            NotRead::NotTwoWords
        );
        assert_eq!(Request::read("").unwrap_err(), NotRead::NotTwoWords);
        assert!(matches!(
            Request::read("arrange org.gnome.Geary").unwrap_err(),
            NotRead::NotAnAct { .. }
        ));
        // A second word that `alo_applications::Application` refuses. It
        // refuses whitespace, so a space cannot get this far — what reaches
        // `NotAnApplication` is a path separator or a control character.
        assert!(matches!(
            Request::read("open org.gnome/Geary").unwrap_err(),
            NotRead::NotAnApplication { .. }
        ));
        assert!(matches!(
            Request::read("open ..\\Geary").unwrap_err(),
            NotRead::NotAnApplication { .. }
        ));
    }

    /// **A long line is refused before it is parsed**, so a caller cannot make
    /// a window system hold one while it reads it.
    #[test]
    fn a_long_line_is_refused_before_it_is_parsed() {
        let long = format!("open {}", "x".repeat(Request::THE_MOST));
        match Request::read(&long).unwrap_err() {
            NotRead::TooLong { most, was } => {
                assert_eq!(most, Request::THE_MOST);
                assert!(was > most);
            }
            other => panic!("{other:?}"),
        }
    }

    /// **Nothing in a request carries a path or a window to look up**, which
    /// is law 2 at the one place a caller can reach — and this test says
    /// exactly how far that goes rather than implying more.
    ///
    /// # What `Application` does and does not refuse
    ///
    /// `alo_applications::Application::identified` refuses an empty
    /// identifier, any control character, any whitespace, `/` and `\`. It
    /// **does not** refuse shell punctuation: `org.gnome.Geary;rm` is a valid
    /// identifier by its rules, and this test asserted otherwise until it was
    /// run.
    ///
    /// **That is not reachable as an injection today, and the reason is law 2
    /// rather than this type.** Every consumer hands the identifier over as one
    /// argument of an argument vector (`alo_software::asked`) or as a D-Bus bus
    /// name (`alo_portals::activated`), and neither composes a shell line —
    /// *there is no field for a command to arrive in* is the protocol's rule
    /// and it holds here too. A `;` in a bus name is refused by the bus.
    ///
    /// **But it is a soft spot and it is named rather than tidied over.** The
    /// type's own job is *an identifier no verb could ever name*, and a real
    /// application identifier is reverse-DNS — letters, digits, dots, hyphens,
    /// underscores — so nothing in the wild needs a semicolon. Tightening
    /// `checked` would be defence in depth on a **published contract value**
    /// (`docs/contracts/agent-verbs.md` types this argument), so it is
    /// `alo-applications`' decision and not this door's to take quietly.
    #[test]
    fn nothing_in_a_request_can_carry_a_path_or_a_window_to_look_up() {
        for line in [
            "open /usr/bin/sh",
            "open ../../etc/passwd",
            "focus $(whoami) extra",
            "open org.gnome.Geary and.another",
        ] {
            assert!(
                Request::read(line).is_err(),
                "`{line}` was read as a request"
            );
        }
        // And what it does admit, said out loud so the boundary is visible
        // rather than inferred from the four above.
        assert!(
            Request::read("open org.gnome.Geary;rm").is_ok(),
            "the boundary moved: see this test's documentation"
        );
    }
}
