//! **Whether a person's session has the screen**, asked of `logind` before the
//! sign-in screen ends as a failure.
//!
//! The sign-in screen ends as a failure when nobody signed in, and its unit then
//! starts it again (#1026), because a machine left at a console is stranded.
//! **But one failure is not a failure**: the person's desktop taking the
//! display. #1029 recognised that by one error shape — the seat saying the
//! session is inactive — and a booted machine on 2026-10-31 showed a second:
//! the person's session took the card first, the sign-in screen's next frame
//! was refused with *atomic TEST_ONLY failed: Permission denied*, the screen
//! ended as a failure, systemd started it again, and as root it took the
//! display back from the desktop, which then died with *display session is
//! inactive*. Which of the two shapes arrives depends on timing.
//!
//! So the question is asked of what decides it rather than read off how the
//! display ended: **is the seat's active session now a person's?** If it is,
//! the end is the hand-over and the screen steps aside. `logind` is the one
//! that knows, whatever error the display produced.
//!
//! **It fails safe.** A `logind` that cannot be asked, or an answer that cannot
//! be read, is *no person has the screen*, which is today's behaviour: the
//! screen comes back. A sign-in screen that wrongly stayed away would leave the
//! machine at a console, which is the fault #1026 ended.

/// One session on the seat, as `logind` describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASessionOnTheSeat {
    /// Whether it is the seat's foreground session.
    pub active: bool,
    /// `logind`'s class: `user` for a person, `greeter` for a sign-in screen.
    pub class: String,
    /// `logind`'s state: `active`, `online`, or `closing`.
    pub state: String,
}

/// What can say which sessions are on the seat.
pub trait TheSeat {
    /// Every session on the seat the sign-in screen draws on.
    ///
    /// # Errors
    /// Why `logind` could not be asked, for the log.
    fn sessions(&self) -> Result<Vec<ASessionOnTheSeat>, String>;
}

/// The class a person's session has, and a sign-in screen's does not.
const A_PERSONS: &str = "user";

/// The state of a session in front of somebody, not one on its way out.
const IN_FRONT: &str = "active";

/// **Whether a person's session has the screen**, failing safe.
///
/// Only a session that is the seat's foreground, of a person's class, and not
/// closing counts. Everything else — a greeter, a session on its way out after
/// a sign-out, no answer at all — is not a hand-over, so the screen comes back.
#[must_use]
pub fn a_person_has_the_screen(seat: &dyn TheSeat) -> bool {
    match seat.sessions() {
        Ok(sessions) => sessions
            .iter()
            .any(|it| it.active && it.class == A_PERSONS && it.state == IN_FRONT),
        Err(why) => {
            eprintln!(
                "alo-compositor: logind could not say whose session has the screen, so the \
                 sign-in screen comes back: {why}"
            );
            false
        }
    }
}

/// `logind`'s `seat0`, on this machine's system bus.
#[derive(Debug, Clone, Copy, Default)]
pub struct Seat0;

/// The seat the sign-in screen draws on, which is the one its unit names.
#[cfg(target_os = "linux")]
const THE_SEAT: &str = "seat0";

impl TheSeat for Seat0 {
    #[cfg(target_os = "linux")]
    fn sessions(&self) -> Result<Vec<ASessionOnTheSeat>, String> {
        use zbus::zvariant::{OwnedObjectPath, OwnedValue};

        let bus = zbus::blocking::Connection::system().map_err(|why| why.to_string())?;
        let listed: Vec<(String, u32, String, String, OwnedObjectPath)> = bus
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1",
                Some("org.freedesktop.login1.Manager"),
                "ListSessions",
                &(),
            )
            .and_then(|reply| reply.body().deserialize())
            .map_err(|why| why.to_string())?;

        let property = |at: &OwnedObjectPath, named: &str| -> Result<OwnedValue, String> {
            bus.call_method(
                Some("org.freedesktop.login1"),
                at.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.login1.Session", named),
            )
            .and_then(|reply| reply.body().deserialize())
            .map_err(|why| format!("{named} of {}: {why}", at.as_str()))
        };

        let mut on_the_seat = Vec::new();
        for (_id, _uid, _user, seat, at) in listed {
            if seat != THE_SEAT {
                continue;
            }
            let active = bool::try_from(property(&at, "Active")?)
                .map_err(|why| format!("Active of {}: {why}", at.as_str()))?;
            let class = String::try_from(property(&at, "Class")?)
                .map_err(|why| format!("Class of {}: {why}", at.as_str()))?;
            let state = String::try_from(property(&at, "State")?)
                .map_err(|why| format!("State of {}: {why}", at.as_str()))?;
            on_the_seat.push(ASessionOnTheSeat {
                active,
                class,
                state,
            });
        }
        Ok(on_the_seat)
    }

    #[cfg(not(target_os = "linux"))]
    fn sessions(&self) -> Result<Vec<ASessionOnTheSeat>, String> {
        Err("there is no logind on this host".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A seat that answers with these sessions, or not at all.
    struct Answering(Result<Vec<ASessionOnTheSeat>, String>);

    impl TheSeat for Answering {
        fn sessions(&self) -> Result<Vec<ASessionOnTheSeat>, String> {
            self.0.clone()
        }
    }

    fn a_session(active: bool, class: &str, state: &str) -> ASessionOnTheSeat {
        ASessionOnTheSeat {
            active,
            class: class.to_owned(),
            state: state.to_owned(),
        }
    }

    /// **The booted machine's case: the person's session is in front, so the
    /// sign-in screen's refused frame is the hand-over.**
    #[test]
    fn a_persons_session_in_front_is_the_hand_over() {
        let seat = Answering(Ok(vec![
            a_session(false, "greeter", "online"),
            a_session(true, "user", "active"),
        ]));
        assert!(a_person_has_the_screen(&seat));
    }

    /// **Nobody's session in front, so the screen comes back** (#1026).
    #[test]
    fn with_only_the_greeter_the_screen_comes_back() {
        for sessions in [
            vec![],
            vec![a_session(true, "greeter", "active")],
            vec![a_session(false, "user", "online")],
        ] {
            assert!(
                !a_person_has_the_screen(&Answering(Ok(sessions.clone()))),
                "{sessions:?} was read as a person having the screen"
            );
        }
    }

    /// **A session on its way out after a sign-out does not keep the sign-in
    /// screen away**, or signing out would leave a black screen.
    #[test]
    fn a_closing_session_does_not_keep_the_screen_away() {
        let seat = Answering(Ok(vec![a_session(true, "user", "closing")]));
        assert!(!a_person_has_the_screen(&seat));
    }

    /// **A logind that cannot be asked means the screen comes back**, which is
    /// the behaviour before this check existed.
    #[test]
    fn a_logind_that_cannot_be_asked_brings_the_screen_back() {
        let seat = Answering(Err("the bus is not there".to_owned()));
        assert!(!a_person_has_the_screen(&seat));
    }
}
