//! The door itself: who is at it, what they asked, and what they are told.
//!
//! `docs/contracts/application-door.md` is the published surface.
//! [ADR 0108](../../../docs/decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md)
//! is why there is a door and why no token crosses it.
//! [`crate::the_application_doors_place`] is where it lives and what it
//! refuses to be served at; this file is what happens once it is.
//!
//! # Nothing here waits, and that is the whole shape of the file
//!
//! This is read from a compositor's frame loop. A door that blocked — on an
//! `accept`, on a read, or on a write — would be a door any process of the
//! person's could use to stop every window on the machine from drawing, by
//! connecting and saying nothing. A read timeout does not fix that; it only
//! decides how long the freeze is, and a tenth of a frame is still a stutter
//! somebody can cause on purpose.
//!
//! So **a connection is held across frames**: everything is non-blocking,
//! [`TheApplicationDoor::answered`] takes whatever bytes have arrived and
//! returns, and a caller that has written half a line is read again next
//! frame. A caller that
//! writes nothing is dropped when its patience runs out, and the number held at
//! once is bounded, so a process opening connections in a loop costs this door
//! a fixed amount of memory and no time at all.
//!
//! # Who is asking is the kernel's answer, not the caller's
//!
//! `SO_PEERCRED` is the user the kernel recorded when `connect` was called. It
//! is not a field in the request and a caller cannot set it. Anybody who is not
//! the person is **closed with nothing written on it** — which is
//! `daemon-protocol.md`'s rule for a stranger and its reason: an answer tells
//! whoever knocked that there is an alo OS here.
//!
//! The check happens **before a byte is read**, so a stranger's line is never
//! parsed, never acted on and never recorded.

use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::{UnixListener, UnixStream};

use alo_acting::{Answer, Request};

use crate::the_application_doors_place::{NoDoor, ThePlace};

/// Whatever carries an act out against this machine's windows.
///
/// **A trait rather than a direct call into [`crate::Server`]**, for the reason
/// `alo_shell::an_application_is_started`'s `AsksAnApplicationToStart` is a
/// function type: it makes the door testable without a compositor, and a door
/// is the half worth testing on its own. The compositor's implementation lives
/// in `crate::an_application_verb_is_carried_out`.
///
/// It answers [`Answer`] and never a refusal of its own, because *may this run*
/// was decided by the turn before the line was written (ADR 0108). A compositor
/// that could refuse for permission would be a second place deciding it.
pub trait CarriesOutAnApplicationVerb {
    /// Do this, and say what happened.
    fn carrying_out(&mut self, acting: &alo_acting::Acting) -> Answer;
}

/// How many callers may be part-way through a line at once.
///
/// One is the daemon. The rest of the room is for a daemon restarting while an
/// old connection has not been noticed yet. **A bound rather than a queue**: a
/// process of the person's own opening connections in a loop then costs a fixed
/// amount of memory, and the alternative is a compositor whose memory is set by
/// whoever is rudest.
pub(crate) const AS_MANY_AS_WAIT: usize = 8;

/// How many frames a caller may hold a connection without finishing a line.
///
/// The daemon writes its line in the same breath as connecting, so one frame is
/// the ordinary case and this is sixty of them — about a second at 60Hz.
/// Counted in **visits rather than seconds** because that is what this file can
/// see: it has no clock, and a compositor that is not drawing is not one a
/// caller is waiting on either.
pub(crate) const PATIENCE: u8 = 60;

/// One caller, part-way through saying something.
struct Waiting {
    /// Their connection, not blocking.
    talking: UnixStream,
    /// What has arrived so far, which may not be a whole line.
    heard: Vec<u8>,
    /// How many visits are left before this is dropped.
    patience: u8,
}

/// The compositor's application door.
pub struct TheApplicationDoor {
    /// Where it lives, so it can be taken away.
    place: ThePlace,
    /// The socket, not blocking.
    listening: UnixListener,
    /// The person this machine is signed in as, by the kernel's account.
    person: u32,
    /// Callers part-way through a line.
    waiting: Vec<Waiting>,
}

impl TheApplicationDoor {
    /// Serve a door at `place` for `person`.
    ///
    /// # Errors
    ///
    /// [`NoDoor`] — and a compositor reads every one of them as *serve no door
    /// and keep running*, which is `the_application_doors_place`'s own
    /// documentation. Nothing is bound when one is returned.
    pub fn served(place: ThePlace, person: u32) -> Result<Self, NoDoor> {
        let listening = place.prepared(person)?;
        Ok(Self {
            place,
            listening,
            person,
            waiting: Vec::new(),
        })
    }

    /// A door admitting a uid that is not the one the directory belongs to.
    ///
    /// **Two checks, varied apart.** On a real machine the person who owns
    /// `/run/alo/<uid>` and the person admitted at the door are the same, and
    /// [`Self::served`] takes one number for both. They are still two
    /// different checks, and a test that cannot vary them cannot test the
    /// second at all: *the stranger is closed with nothing written* is the one
    /// thing standing between the agent and this door, and it has to be
    /// provable without being two users.
    #[cfg(test)]
    pub(crate) fn admitting(place: ThePlace, listening: UnixListener, person: u32) -> Self {
        Self {
            place,
            listening,
            person,
            waiting: Vec::new(),
        }
    }

    /// How many callers are part-way through a line, for a test about the bound.
    #[cfg(test)]
    pub(crate) fn how_many_are_waiting(&self) -> usize {
        self.waiting.len()
    }

    /// The socket this door is on, for a caller to be told where to knock.
    #[must_use]
    pub fn socket(&self) -> &std::path::Path {
        self.place.socket()
    }

    /// Take whatever has arrived, answer every whole line, and return.
    ///
    /// Answers how many acts were carried out, which is what a caller logs or
    /// counts. **Never waits**: see this file's own documentation.
    pub fn answered(&mut self, carrying_out: &mut dyn CarriesOutAnApplicationVerb) -> usize {
        self.let_them_in();
        self.hear_them_out(carrying_out)
    }

    /// Take the door away. The socket goes; the daemon's directory stays.
    pub fn taken_away(&self) {
        self.place.taken_away();
    }

    /// Accept everybody waiting, refusing anyone who is not the person.
    ///
    /// A stranger is closed **with nothing written on it**, and before a byte
    /// of theirs is read.
    fn let_them_in(&mut self) {
        loop {
            if self.waiting.len() >= AS_MANY_AS_WAIT {
                return;
            }
            let talking = match self.listening.accept() {
                Ok((talking, _)) => talking,
                // Nobody else is waiting, which is the ordinary answer.
                Err(why) if why.kind() == ErrorKind::WouldBlock => return,
                Err(why) if why.kind() == ErrorKind::Interrupted => continue,
                // The socket itself has gone wrong. Nothing here can mend it,
                // and a door that cannot accept is a door that answers nothing
                // — which is the same as no door, and the verbs say
                // `could-not-ask`.
                Err(_) => return,
            };
            if !self.is_the_person(&talking) {
                drop(talking);
                continue;
            }
            if talking.set_nonblocking(true).is_err() {
                drop(talking);
                continue;
            }
            self.waiting.push(Waiting {
                talking,
                heard: Vec::new(),
                patience: PATIENCE,
            });
        }
    }

    /// Whether the kernel says the person is at the other end.
    ///
    /// **Asked of the connection and not of the socket.** `SO_PEERCRED` on a
    /// socket with no peer answers with process zero, and a listening socket is
    /// not a connection — so the type is what keeps this honest, exactly as in
    /// `alo_agentd::unix`.
    fn is_the_person(&self, talking: &UnixStream) -> bool {
        rustix::net::sockopt::socket_peercred(talking)
            .is_ok_and(|peer| peer.uid.as_raw() == self.person)
    }

    /// Read what has arrived from everybody, answer whole lines, drop the rest.
    fn hear_them_out(&mut self, carrying_out: &mut dyn CarriesOutAnApplicationVerb) -> usize {
        let mut acted = 0;
        let mut still_waiting = Vec::with_capacity(self.waiting.len());
        for mut caller in std::mem::take(&mut self.waiting) {
            match caller.read_a_little() {
                // A whole line, so this connection is finished either way.
                HowFarAlong::ALine(line) => {
                    let answer = match Request::read(&line) {
                        Ok(request) => {
                            let answer = carrying_out.carrying_out(request.acting());
                            if matches!(answer, Answer::Acted) {
                                acted += 1;
                            }
                            answer.written()
                        }
                        // **Not a request is not an answer either.** The three
                        // words `NotRead` tells apart are for whoever is fixing
                        // a caller, and this wire has one word out. A caller
                        // that cannot write a request is a caller with a bug,
                        // and `could-not-ask` would blame the machine for it.
                        Err(_) => {
                            drop(caller.talking);
                            continue;
                        }
                    };
                    let _ = caller.talking.write_all(answer.as_bytes());
                    let _ = caller.talking.write_all(b"\n");
                    let _ = caller.talking.flush();
                    drop(caller.talking);
                }
                // Part of one, or nothing yet. Next frame.
                HowFarAlong::NotYet => {
                    caller.patience = caller.patience.saturating_sub(1);
                    if caller.patience > 0 {
                        still_waiting.push(caller);
                    }
                }
                // Gone, or said more than a request may be.
                HowFarAlong::Finished => drop(caller.talking),
            }
        }
        self.waiting = still_waiting;
        acted
    }
}

/// How far one caller has got.
enum HowFarAlong {
    /// A whole line, without its newline.
    ALine(String),
    /// Nothing, or part of a line. Ask again.
    NotYet,
    /// The caller has gone, or has said too much to be saying a request.
    Finished,
}

impl Waiting {
    /// Take whatever bytes have arrived, and say how far along this caller is.
    ///
    /// **Bounded by `Request::THE_MOST`**, so a caller cannot make the
    /// compositor hold a line while it writes one — which is that constant's
    /// own reason. A caller that has written more than a request may be is
    /// finished with rather than truncated: a truncated line could be a
    /// different, valid request.
    fn read_a_little(&mut self) -> HowFarAlong {
        let mut arrived = [0_u8; Request::THE_MOST];
        loop {
            match self.talking.read(&mut arrived) {
                // The caller has gone without finishing a line.
                Ok(0) => return HowFarAlong::Finished,
                Ok(read) => {
                    let Some(said) = arrived.get(..read) else {
                        return HowFarAlong::Finished;
                    };
                    self.heard.extend_from_slice(said);
                    if self.heard.len() > Request::THE_MOST {
                        return HowFarAlong::Finished;
                    }
                    if let Some(end) = self.heard.iter().position(|byte| *byte == b'\n') {
                        let Some(line) = self.heard.get(..end) else {
                            return HowFarAlong::Finished;
                        };
                        return match std::str::from_utf8(line) {
                            Ok(line) => HowFarAlong::ALine(line.to_owned()),
                            // Not text at all, so not a request. Said here
                            // rather than left to `Request::read`, which takes
                            // a `&str` and would never see it.
                            Err(_) => HowFarAlong::Finished,
                        };
                    }
                }
                Err(why) if why.kind() == ErrorKind::WouldBlock => return HowFarAlong::NotYet,
                Err(why) if why.kind() == ErrorKind::Interrupted => {}
                Err(_) => return HowFarAlong::Finished,
            }
        }
    }
}
