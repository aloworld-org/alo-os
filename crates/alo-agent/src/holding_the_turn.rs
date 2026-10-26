//! Holding the turn open while the person answers, and no longer.
//!
//! **A waiting change lives inside the turn, and the turn lasts as long as
//! this process is connected.** `alo-agentd`'s own words, in `answering.rs`: *a
//! number that is not waiting is a number that is not waiting, whether the turn
//! ended a moment ago or never began.* So an agent that printed *a change is
//! waiting* and exited — which this one did until 2026-10-26 — ended its turn
//! with its question in it, and by the time the shell asked the daemon what was
//! waiting the truthful answer was nothing. Walking a booted machine that day,
//! the proposal appeared and the card under it never did.
//!
//! # How long, and why not the whole time the question stands
//!
//! The daemon serves **one turn at a time** (`alo-agentd`'s `serving.rs`): a
//! second agent arriving while one is under way is refused. So an agent that
//! held its turn for the whole five minutes a question stands would refuse the
//! person's next request for whatever was left of them — approve an archive,
//! ask to rename it, and be turned away.
//!
//! So the turn is held **until the person has answered**, and the shell says
//! when by closing the socket it opened for the request: it keeps that socket
//! while the question is on the screen and lets it go once the daemon has
//! carried the answer out. This process sees the close and leaves, and the turn
//! ends with nothing waiting in it. The question's own standing time is only
//! the bound, for a person who never answers.
//!
//! # The answer reaches the shell first
//!
//! The shell reads what this process wrote **until the socket's writing side
//! closes**, and only then asks the daemon what is waiting. So the writing side
//! is closed first — the reply is complete, the card can be drawn — and the
//! reading side is what this waits on. A half-closed socket is the whole
//! mechanism, which is why it needs no change to the daemon or the protocol.

use std::io::Read;
use std::os::fd::AsFd;
use std::time::Duration;

/// How holding the turn ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held {
    /// The shell closed the request's socket: the person has answered.
    TheyAnswered,
    /// The question's standing time passed with no answer.
    ItLapsed,
    /// This was not started on a socket — run by hand from a terminal — so
    /// there is nobody to tell and nobody who will close anything. Nothing is
    /// held, which is what this process did before holding existed.
    NotOnASocket,
}

/// Close the writing side of `told`, so whoever asked has the whole reply, and
/// then wait until `asking` is closed by the other side or `stands_for` passes.
///
/// `told` and `asking` are this process's standard output and input, which
/// `alo-asking@.service` makes the request's socket; a test hands both ends of
/// a socket pair instead.
pub fn held_until_answered(
    told: impl AsFd,
    mut asking: impl AsFd + Read,
    stands_for: Duration,
) -> Held {
    if rustix::net::shutdown(told.as_fd(), rustix::net::Shutdown::Write).is_err() {
        return Held::NotOnASocket;
    }
    // A zero timeout means *wait for ever* to the kernel, which is the one
    // thing a bound must never become.
    let bound = stands_for.max(Duration::from_millis(1));
    if rustix::net::sockopt::set_socket_timeout(
        asking.as_fd(),
        rustix::net::sockopt::Timeout::Recv,
        Some(bound),
    )
    .is_err()
    {
        return Held::NotOnASocket;
    }
    let mut ignored = [0_u8; 64];
    loop {
        match asking.read(&mut ignored) {
            Ok(0) => return Held::TheyAnswered,
            // Nothing is said on this socket after the request, and anything
            // that is said is not an answer: the answer goes to the daemon.
            Ok(_) => {}
            Err(why) if why.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Held::ItLapsed,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on a socket pair that cannot be made is the failure being reported"
)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;
    use std::time::Instant;

    /// **The person answering lets the agent go at once**, and not when the
    /// question would have lapsed — which is what keeps their next request
    /// from being refused.
    #[test]
    fn the_shell_closing_the_socket_ends_the_hold_at_once() {
        let (agents, shells) = UnixStream::pair().unwrap();
        let closing = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            drop(shells);
        });

        let started = Instant::now();
        let held = held_until_answered(&agents, &agents, Duration::from_secs(30));

        assert_eq!(held, Held::TheyAnswered);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the hold outlasted the answer"
        );
        closing.join().unwrap();
    }

    /// **The reply is complete before anybody answers**: the shell reads to the
    /// end of what was written, and only then can it draw the card. If the
    /// writing side stayed open, the shell would wait for the hold and the card
    /// would appear after its question had lapsed.
    #[test]
    fn the_shell_has_the_whole_reply_while_the_turn_is_held() {
        let (agents, shells) = UnixStream::pair().unwrap();
        let holding = std::thread::spawn(move || {
            held_until_answered(&agents, &agents, Duration::from_secs(30))
        });

        shells
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut read = Vec::new();
        let ended = (&shells).read_to_end(&mut read);
        assert!(
            ended.is_ok(),
            "the shell never saw the reply end: {ended:?}"
        );

        drop(shells);
        assert_eq!(holding.join().unwrap(), Held::TheyAnswered);
    }

    /// **A person who never answers does not hold the machine for ever**: the
    /// question's standing time bounds the hold.
    #[test]
    fn with_no_answer_the_hold_ends_when_the_question_lapses() {
        let (agents, shells) = UnixStream::pair().unwrap();

        let started = Instant::now();
        let held = held_until_answered(&agents, &agents, Duration::from_millis(200));

        assert_eq!(held, Held::ItLapsed);
        assert!(
            started.elapsed() >= Duration::from_millis(150),
            "it did not wait"
        );
        drop(shells);
    }

    /// **Run by hand from a terminal, nothing is held** — there is no socket to
    /// close and nobody who would close it, so waiting would only hang.
    #[test]
    fn not_on_a_socket_holds_nothing() {
        let not_a_socket = std::fs::File::open("/dev/null").unwrap();
        let started = Instant::now();

        let held = held_until_answered(&not_a_socket, &not_a_socket, Duration::from_secs(30));

        assert_eq!(held, Held::NotOnASocket);
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
