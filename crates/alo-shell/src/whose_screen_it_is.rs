//! **Whether a person is arriving at the screen**, asked of `logind` and of
//! systemd before the sign-in screen ends as a failure.
//!
//! The sign-in screen ends as a failure when nobody signed in, and its unit then
//! starts it again (#1026), because a machine left at a console is stranded.
//! **But one failure is not a failure**: the person's desktop taking the
//! display. #1029 recognised that by one error shape — the seat saying the
//! session is inactive — and a booted machine on 2026-10-31 showed a second:
//! the person's session took the card first, the sign-in screen's next frame
//! was refused with *atomic TEST_ONLY failed: Permission denied*, the screen
//! ended as a failure, systemd started it again, and as root it took the
//! display back from the desktop, which then died. Which shape arrives is
//! timing, so the question is asked of what decides it rather than read off the
//! error.
//!
//! # A person who is still arriving counts, and #1052 missed that
//!
//! #1052 asked only for a session that was settled: `Active`, class `user`,
//! state `active`. **On the next booted machine that answered *nobody* ten
//! seconds into the desktop starting and thirty before it had started** —
//! the screen came back and the desktop died the same way. A session `logind`
//! has just switched the seat to is still being opened: its state is `opening`
//! or `online` and systemd 257 gives it a passing class such as `user-early`
//! or `user-incomplete`. The settled session was what podman had measured.
//!
//! So two things each count as a person arriving, and either is enough:
//!
//! - **a session of theirs on the seat**: a person's account (a uid from
//!   [`THE_FIRST_PERSON`] up, and not `nobody`), a class that begins `user`, and a
//!   state that is opening, online or active — anything but closing;
//! - **their desktop's unit starting or running**: `alo-desktop@<uid>.service`
//!   activating or active, asked of systemd. This one does not depend on
//!   `logind`'s timing at all, because `alo-sessiond` starts the unit before
//!   the session exists.
//!
//! **It fails safe.** What cannot be asked, or cannot be read, is *nobody is
//! arriving*, which is the behaviour before any of this existed: the screen
//! comes back. A sign-in screen that wrongly stayed away would leave the
//! machine at a console, which is the fault #1026 ended.

/// One session on the seat, as `logind` describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASessionOnTheSeat {
    /// `logind`'s name for it, for the log.
    pub id: String,
    /// The account it is for.
    pub uid: u32,
    /// Whether it is the seat's foreground session.
    pub active: bool,
    /// `logind`'s class: `user` and its passing forms for a person, `greeter`
    /// for a sign-in screen.
    pub class: String,
    /// `logind`'s state: `opening`, `online`, `active`, or `closing`.
    pub state: String,
    /// The PAM service that opened it, for the log.
    pub service: String,
}

/// What can say which sessions are on the seat.
pub trait TheSeat {
    /// Every session on the seat the sign-in screen draws on.
    ///
    /// # Errors
    /// Why `logind` could not be asked, for the log.
    fn sessions(&self) -> Result<Vec<ASessionOnTheSeat>, String>;
}

/// What can say which of the person's desktops systemd has.
pub trait TheDesktops {
    /// Every desktop unit systemd has loaded, with its `ActiveState`.
    ///
    /// # Errors
    /// Why systemd could not be asked, for the log.
    fn units(&self) -> Result<Vec<(String, String)>, String>;
}

/// The first account number that is a person's rather than the machine's.
pub const THE_FIRST_PERSON: u32 = 1000;

/// The account that is nobody's.
const NOBODY: u32 = 65_534;

/// Whether `logind` describes a person's session that is not on its way out.
#[must_use]
pub fn is_a_person_arriving(session: &ASessionOnTheSeat) -> bool {
    session.uid >= THE_FIRST_PERSON
        && session.uid != NOBODY
        && session.class.starts_with("user")
        && matches!(session.state.as_str(), "opening" | "online" | "active")
}

/// Whether systemd describes a desktop that is starting or running.
#[must_use]
pub fn is_a_desktop_arriving(active_state: &str) -> bool {
    matches!(active_state, "activating" | "active" | "reloading")
}

/// **Whether a person is arriving at the screen**, failing safe.
///
/// Either a session of theirs on the seat that is not closing, or their
/// desktop's unit starting or running. What could not be asked counts as
/// nobody, and what each said is written to the log on the way to a restart,
/// so the next machine shows which question answered what.
#[must_use]
pub fn a_person_is_arriving(seat: &dyn TheSeat, desktops: &dyn TheDesktops) -> bool {
    let sessions = seat.sessions();
    let units = desktops.units();

    let by_session = sessions
        .as_ref()
        .is_ok_and(|all| all.iter().any(is_a_person_arriving));
    let by_desktop = units
        .as_ref()
        .is_ok_and(|all| all.iter().any(|(_, state)| is_a_desktop_arriving(state)));
    if by_session || by_desktop {
        return true;
    }

    eprintln!(
        "alo-compositor: nobody is arriving at the screen — logind says {}; systemd says {}",
        match &sessions {
            Ok(all) if all.is_empty() => "no session on the seat".to_owned(),
            Ok(all) => all
                .iter()
                .map(|it| {
                    format!(
                        "{} (uid {}, class {}, state {}, {}, service {})",
                        it.id,
                        it.uid,
                        it.class,
                        it.state,
                        if it.active { "in front" } else { "behind" },
                        it.service
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
            Err(why) => format!("nothing, because it could not be asked: {why}"),
        },
        match &units {
            Ok(all) if all.is_empty() => "no desktop unit".to_owned(),
            Ok(all) => all
                .iter()
                .map(|(name, state)| format!("{name} {state}"))
                .collect::<Vec<_>>()
                .join(", "),
            Err(why) => format!("nothing, because it could not be asked: {why}"),
        }
    );
    false
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
        let text = |at: &OwnedObjectPath, named: &str| -> Result<String, String> {
            String::try_from(property(at, named)?)
                .map_err(|why| format!("{named} of {}: {why}", at.as_str()))
        };

        let mut on_the_seat = Vec::new();
        for (id, uid, _user, seat, at) in listed {
            if seat != THE_SEAT {
                continue;
            }
            let active = bool::try_from(property(&at, "Active")?)
                .map_err(|why| format!("Active of {}: {why}", at.as_str()))?;
            on_the_seat.push(ASessionOnTheSeat {
                id,
                uid,
                active,
                class: text(&at, "Class")?,
                state: text(&at, "State")?,
                service: text(&at, "Service")?,
            });
        }
        Ok(on_the_seat)
    }

    #[cfg(not(target_os = "linux"))]
    fn sessions(&self) -> Result<Vec<ASessionOnTheSeat>, String> {
        Err("there is no logind on this host".to_owned())
    }
}

/// systemd's own manager, asked about the person's desktop units.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheDesktopUnits;

/// Every instance of the person's desktop unit (ADR 0141).
#[cfg(target_os = "linux")]
const EVERY_DESKTOP: &str = "alo-desktop@*.service";

impl TheDesktops for TheDesktopUnits {
    #[cfg(target_os = "linux")]
    fn units(&self) -> Result<Vec<(String, String)>, String> {
        use zbus::zvariant::OwnedObjectPath;

        /// What `ListUnitsByPatterns` answers with, one per unit.
        type AUnit = (
            String,
            String,
            String,
            String,
            String,
            String,
            OwnedObjectPath,
            u32,
            String,
            OwnedObjectPath,
        );

        let bus = zbus::blocking::Connection::system().map_err(|why| why.to_string())?;
        let every_state: Vec<&str> = Vec::new();
        let listed: Vec<AUnit> = bus
            .call_method(
                Some("org.freedesktop.systemd1"),
                "/org/freedesktop/systemd1",
                Some("org.freedesktop.systemd1.Manager"),
                "ListUnitsByPatterns",
                &(every_state, vec![EVERY_DESKTOP]),
            )
            .and_then(|reply| reply.body().deserialize())
            .map_err(|why| why.to_string())?;
        Ok(listed
            .into_iter()
            .map(|(name, _, _, active, ..)| (name, active))
            .collect())
    }

    #[cfg(not(target_os = "linux"))]
    fn units(&self) -> Result<Vec<(String, String)>, String> {
        Err("there is no systemd on this host".to_owned())
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

    /// A systemd that answers with these units, or not at all.
    struct Units(Result<Vec<(String, String)>, String>);

    impl TheDesktops for Units {
        fn units(&self) -> Result<Vec<(String, String)>, String> {
            self.0.clone()
        }
    }

    fn a_session(uid: u32, class: &str, state: &str) -> ASessionOnTheSeat {
        ASessionOnTheSeat {
            id: "3".to_owned(),
            uid,
            active: true,
            class: class.to_owned(),
            state: state.to_owned(),
            service: "alo-desktop".to_owned(),
        }
    }

    fn no_units() -> Units {
        Units(Ok(Vec::new()))
    }

    /// **A session still being opened is the person arriving**, in every form
    /// systemd 257 gives one on its way in.
    #[test]
    fn a_session_still_opening_is_the_hand_over() {
        for (class, state) in [
            ("user", "opening"),
            ("user-early", "opening"),
            ("user-incomplete", "online"),
            ("user", "online"),
            ("user", "active"),
        ] {
            let seat = Answering(Ok(vec![a_session(1000, class, state)]));
            assert!(
                a_person_is_arriving(&seat, &no_units()),
                "class {class}, state {state} was not read as a person arriving"
            );
        }
    }

    /// **A desktop unit starting is the person arriving**, whatever `logind`
    /// said — the booted machine's case on the second walk.
    #[test]
    fn a_desktop_unit_starting_is_the_hand_over() {
        let seat = Answering(Ok(vec![a_session(60990, "greeter", "active")]));
        for state in ["activating", "active"] {
            let units = Units(Ok(vec![(
                "alo-desktop@1000.service".to_owned(),
                state.to_owned(),
            )]));
            assert!(
                a_person_is_arriving(&seat, &units),
                "a desktop {state} was not read as a person arriving"
            );
        }
    }

    /// **A session on its way out does not keep the sign-in screen away**, or
    /// signing out would leave a black screen.
    #[test]
    fn a_closing_session_does_not_count() {
        let seat = Answering(Ok(vec![a_session(1000, "user", "closing")]));
        let units = Units(Ok(vec![(
            "alo-desktop@1000.service".to_owned(),
            "inactive".to_owned(),
        )]));
        assert!(!a_person_is_arriving(&seat, &units));
    }

    /// **Neither a person's session nor a desktop starting means the screen
    /// comes back** (#1026).
    #[test]
    fn neither_present_brings_the_screen_back() {
        for sessions in [
            vec![],
            vec![a_session(60990, "greeter", "active")],
            vec![a_session(0, "user", "active")],
            vec![a_session(65_534, "user", "active")],
            vec![a_session(1000, "background", "active")],
        ] {
            assert!(
                !a_person_is_arriving(&Answering(Ok(sessions.clone())), &no_units()),
                "{sessions:?} was read as a person arriving"
            );
        }
        let failed = Units(Ok(vec![(
            "alo-desktop@1000.service".to_owned(),
            "failed".to_owned(),
        )]));
        assert!(!a_person_is_arriving(&Answering(Ok(vec![])), &failed));
    }

    /// **Nothing that can be asked means the screen comes back**, which is the
    /// behaviour before any of this existed.
    #[test]
    fn an_unreachable_logind_and_systemd_bring_the_screen_back() {
        let seat = Answering(Err("the bus is not there".to_owned()));
        let units = Units(Err("the bus is not there".to_owned()));
        assert!(!a_person_is_arriving(&seat, &units));
    }

    /// **And either one answering yes is enough**, so a systemd that cannot be
    /// asked does not hide a session `logind` can see, nor the other way round.
    #[test]
    fn either_question_answering_yes_is_enough() {
        assert!(a_person_is_arriving(
            &Answering(Ok(vec![a_session(1000, "user", "opening")])),
            &Units(Err("no systemd".to_owned()))
        ));
        assert!(a_person_is_arriving(
            &Answering(Err("no logind".to_owned())),
            &Units(Ok(vec![(
                "alo-desktop@1000.service".to_owned(),
                "activating".to_owned()
            )]))
        ));
    }
}
