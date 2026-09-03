//! One machine, two connections, one turn.
//!
//! This is the decision item 21c cut out of itself rather than take in a hurry,
//! and it is the reason this file exists at all.
//!
//! # The problem, exactly
//!
//! `alo_turn::Turning` borrows the machine mutably and there is one machine, so
//! there is one turn. A turn belongs to an **agent's** connection: it began
//! because somebody asked their agent something. But an approval belongs to the
//! **person's** connection, on the other door, because ADR 0001 §5 puts the
//! answering there and `alo-protocol` makes the two doors two.
//!
//! So a service that read one connection to the end before looking at the other
//! would stop dead at the first proposal: the agent is waiting to be told what
//! happened, and the message that would tell it can only arrive on a connection
//! nobody is reading.
//!
//! # The answer is readiness, and it is one thread
//!
//! Nothing in a turn blocks. A read answers, a proposal comes straight back as
//! a number and a sentence, an approval runs and answers. What blocks is
//! **waiting for somebody to say something**, and that is one call — `poll` —
//! over the socket, the connections and the end a stop arrives on
//! (`crate::unix::ready`).
//!
//! So there are no threads here, no channels, no lock around the machine, and
//! nothing shared between two things that run at once. The machine is a local
//! variable that one loop owns. That is worth saying plainly, because the
//! obvious shape for *two connections at once* is a thread each and a mutex,
//! and it would have put the capability model behind a lock in the one service
//! whose whole value is being small enough to read.
//!
//! # A turn is an agent's connection, and it is a scope
//!
//! It begins when an agent connects and ends when that connection closes, and
//! nothing on the wire says either — `alo-protocol` deliberately has no message
//! that begins or ends a turn, because a number for it would be a number an
//! agent could change. Here that is not a rule to remember but the shape of the
//! code: the turn is a variable that lives inside the loop that runs while an
//! agent is connected, and the grant the invocation made goes back on every
//! road out of it, including the service stopping and the record breaking.
//!
//! **One turn at a time, therefore one agent at a time.** A second agent
//! arriving while a turn is under way is refused in words and closed; the
//! person's door is one at a time for the same reason and gets a sentence of
//! its own. Both are `crate::words`.
//!
//! # What a turn is begun with, and what it is not
//!
//! Nothing. `alo_context::Context::at_invocation` offers no document, no window
//! and no selection, because what is in front of the person is answered by
//! Wayland and AT-SPI and there is no compositor here. That is the honest state
//! of this machine rather than a gap: an agent gets what it was granted and
//! nothing the person happened to have open.
//!
//! # And the clock
//!
//! Every crate in this workspace takes `now` as an argument so that expiry is
//! arithmetic rather than a wait. Something has to read a clock, and it is
//! `this_moment` — once per round, so every message in one round is answered
//! at one moment and no two answers can disagree about whether a grant had
//! expired between them.

use std::time::{Duration, SystemTime};

use alo_capability::Grants;
use alo_context::Context;
use alo_protocol::{NotUnderstood, ToAPerson, ToAnAgent};
use alo_strings::{Filling, Strings};
use alo_turn::{Machine, Turning};

use crate::answering::what_a_person_said;
use crate::doing::what_an_agent_said;
use crate::knocking::Knocking;
use crate::lines::Line;
use crate::refusing::NotServed;
use crate::side::Side;
use crate::stopping::Waking;
use crate::unix::ready;
use crate::words::{A_TURN_IS_UNDER_WAY, SOMEBODY_IS_ALREADY_ANSWERING};

/// What the service did before it stopped.
///
/// What the process reports when it ends. Showing a person what their machine
/// turned away while it is running is a surface that does not exist yet, and
/// these are numbers rather than sentences, so nothing here needs a language.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Served {
    /// How many turns were held.
    turns: u64,
    /// How many messages were read and answered, refusals included.
    messages: u64,
    /// How many connections were made by somebody who is neither the person nor
    /// the agent.
    ///
    /// The one number here that is not a measure of work done. It is what item
    /// 21c left this item: a stranger is told nothing, because answering would
    /// say there is an alo OS daemon here — and being told nothing is not the
    /// same as nothing being noticed.
    strangers: u64,
}

impl Served {
    /// How many turns were held.
    #[must_use]
    pub const fn turns(&self) -> u64 {
        self.turns
    }

    /// How many messages were read and answered.
    #[must_use]
    pub const fn messages(&self) -> u64 {
        self.messages
    }

    /// How many connections came from somebody who is neither of the two.
    #[must_use]
    pub const fn strangers_turned_away(&self) -> u64 {
        self.strangers
    }
}

/// The agent service, running.
///
/// Holds what the machine was told about itself and nothing that changes: the
/// door connections arrive at, the end a stop arrives on, which agent this
/// machine has, and how long a turn and a question last. Where those come from
/// is the process's, and is queue item 21e.
pub struct Serving<'a> {
    /// Where connections come from, and which door each is on.
    knocking: &'a dyn Knocking,
    /// What a stop arrives on.
    waking: &'a Waking,
    /// The agent this machine has, as the grants name it.
    for_agent: &'a str,
    /// How long a turn's own grant lasts.
    lasting: Duration,
    /// How long a change waits for an answer.
    standing: Duration,
}

/// What a round of work found, once everything ready has been dealt with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Next {
    /// Go round again.
    GoOn,
    /// Somebody asked the service to stop.
    Stopped,
}

/// The two connections the service is holding.
///
/// At most one of each, which is what makes *one machine, one turn* a shape
/// rather than a rule: there is nowhere to put a second agent.
#[derive(Debug, Default)]
struct Held {
    /// The agent's connection, while a turn is under way.
    agent: Option<Line>,
    /// The person's shell, which outlives any one turn.
    person: Option<Line>,
}

impl<'a> Serving<'a> {
    /// The service, told what this machine is.
    #[must_use]
    pub const fn of(
        knocking: &'a dyn Knocking,
        waking: &'a Waking,
        for_agent: &'a str,
        lasting: Duration,
        standing: Duration,
    ) -> Self {
        Self {
            knocking,
            waking,
            for_agent,
            lasting,
            standing,
        }
    }

    /// Serve until somebody asks the service to stop.
    ///
    /// The machine is borrowed rather than built here, so that where the record
    /// really goes stays one decision made in one place. The words this service
    /// says are **the machine's own** — taken from it rather than passed in
    /// beside it — because the record writes down what a person was shown, and
    /// two vocabularies would be a screen in one language and a record in
    /// another.
    ///
    /// # Errors
    ///
    /// [`NotServed`], which is always the machine rather than a client: a
    /// message that is not a request, a stranger at the door and a caller that
    /// hangs up mid-message are all served and survived. In every one of them
    /// the turn that was under way has been ended and its grant given back.
    pub fn until_stopped(
        &self,
        machine: &mut Machine<'_>,
        grants: &mut Grants,
    ) -> Result<Served, NotServed> {
        let strings = machine.strings();
        let mut held = Held::default();
        let mut served = Served::default();

        loop {
            while held.agent.is_none() {
                if self.one_round(&mut held, None, grants, strings, &mut served)? == Next::Stopped {
                    return Ok(served);
                }
            }

            let mut turning = Turning::beginning(
                Context::at_invocation(this_moment()),
                self.for_agent,
                self.lasting,
                grants,
                machine,
            )
            .map_err(|why| NotServed::NoTurn { why })?;
            served.turns = served.turns.saturating_add(1);

            // Every road out of this loop ends the turn, including the ones
            // that are about to fail: a grant an invocation made is the
            // machine's until something takes it back, and a service that
            // stopped holding one would leave a folder reachable by an agent
            // whose turn is over.
            let mut over = Ok(false);
            while held.agent.is_some() {
                match self.one_round(&mut held, Some(&mut turning), grants, strings, &mut served) {
                    Ok(Next::GoOn) => {}
                    Ok(Next::Stopped) => {
                        over = Ok(true);
                        break;
                    }
                    Err(why) => {
                        over = Err(why);
                        break;
                    }
                }
                if turning.is_closed() {
                    over = Err(NotServed::NothingIsWrittenDown);
                    break;
                }
            }
            let _gave_a_grant_back = turning.ending(grants);
            held.agent = None;

            if over? {
                return Ok(served);
            }
        }
    }

    /// Wait until something has happened, and deal with all of it.
    ///
    /// The order is the person, then the agent, then the door: somebody already
    /// connected is answered before somebody new is let in, and the person is
    /// answered before the agent because an approval that has already arrived
    /// should not wait behind the next thing an agent thought of.
    ///
    /// **A round that ended a turn lets nobody in**, and that is the one piece
    /// of ordering here that is load-bearing rather than tidy. An agent hanging
    /// up and the next one knocking are two things that can be noticed in the
    /// same wake-up, and if the door were answered afterwards the newcomer
    /// would land in a slot that had just been emptied — inside the turn the
    /// first agent's invocation made, holding a grant that was never for it.
    /// So the round returns as soon as the agent has gone. Nothing is lost by
    /// it: `poll` reports what is *there* rather than what has changed, so
    /// whoever is knocking is still knocking when the next round asks, and they
    /// get a turn of their own instead of the remains of somebody else's.
    fn one_round(
        &self,
        held: &mut Held,
        mut turning: Option<&mut Turning<'_, '_>>,
        grants: &Grants,
        strings: &Strings,
        served: &mut Served,
    ) -> Result<Next, NotServed> {
        let (stopped, person, agent, knocked) = {
            let waiting_on = [
                Some(self.waking.waiting_on()),
                held.person.as_ref().map(Line::waiting_on),
                held.agent.as_ref().map(Line::waiting_on),
                Some(self.knocking.waiting_on()),
            ];
            let [stopped, person, agent, knocked] =
                ready(&waiting_on).map_err(|why| NotServed::NotWaiting { why })?;
            (stopped, person, agent, knocked)
        };

        if stopped {
            return Ok(Next::Stopped);
        }
        let now = this_moment();

        if person {
            let answered = held.person.as_mut().map(|line| {
                one_message(
                    line,
                    |said| {
                        what_a_person_said(said, turning.as_deref_mut(), grants, strings, now)
                            .written()
                            .ok()
                    },
                    |why| ToAPerson::refused(&why.said(strings)).written().ok(),
                )
            });
            if answered == Some(Message::Ended) {
                held.person = None;
            }
            if answered == Some(Message::Answered) {
                served.messages = served.messages.saturating_add(1);
            }
        }

        if agent {
            let answered = match (held.agent.as_mut(), turning.as_deref_mut()) {
                // A connection with no turn behind it cannot be served and
                // cannot be left waiting either: it would be ready for ever and
                // read by nobody. It is the end of the connection, which for an
                // agent is the end of what it came for. Unreachable while the
                // two loops above are the only callers, and answered here
                // rather than assumed away.
                (Some(_) | None, None) | (None, Some(_)) => Message::Ended,
                (Some(line), Some(turning)) => one_message(
                    line,
                    |said| {
                        what_an_agent_said(said, turning, grants, strings, self.standing, now)
                            .written()
                            .ok()
                    },
                    |why| ToAnAgent::refused(&why.said(strings)).written().ok(),
                ),
            };
            if answered == Message::Answered {
                served.messages = served.messages.saturating_add(1);
            }
            if answered == Message::Ended {
                held.agent = None;
                return Ok(Next::GoOn);
            }
        }

        if knocked {
            match self.knocking.next() {
                Ok((side, connection)) => {
                    self.let_in(held, side, connection, turning.is_some(), strings);
                }
                Err(why) if why.is_only_this_connection() => {
                    served.strangers = served.strangers.saturating_add(1);
                }
                Err(why) => return Err(NotServed::NotTaken(why)),
            }
        }

        Ok(Next::GoOn)
    }

    /// Put a new connection on its door, or turn it away in words.
    ///
    /// A connection that cannot be read from or answered on is closed without a
    /// word, because there is no way to say anything to it — that is the same
    /// answer [`crate::Line::over`] gives and for the same reason.
    ///
    /// **The agent's door asks two questions and the person's asks one.** A
    /// second shell is refused because one is already connected; a second agent
    /// is refused because one is connected *or* because a turn is under way at
    /// all. The second half is what makes *an agent never acts under a grant
    /// another agent's invocation made* a property of this method rather than a
    /// property of the order [`Serving::one_round`] happens to do things in —
    /// the ordering is still right, and this is what would hold if it were not.
    fn let_in(
        &self,
        held: &mut Held,
        side: Side,
        connection: std::os::unix::net::UnixStream,
        a_turn_is_under_way: bool,
        strings: &Strings,
    ) {
        let Ok(mut line) = Line::over(connection) else {
            return;
        };
        let (taken, free) = match side {
            Side::Agent => (&mut held.agent, !a_turn_is_under_way),
            Side::Person => (&mut held.person, true),
        };
        if free && taken.is_none() {
            *taken = Some(line);
            return;
        }

        let word = match side {
            Side::Agent => A_TURN_IS_UNDER_WAY,
            Side::Person => SOMEBODY_IS_ALREADY_ANSWERING,
        };
        let said = strings.say(&word.key(), &Filling::nothing());
        let written = match side {
            Side::Agent => ToAnAgent::refused(&said).written().ok(),
            Side::Person => ToAPerson::refused(&said).written().ok(),
        };
        if let Some(written) = written {
            drop(line.say(&written));
        }
    }
}

impl std::fmt::Debug for Serving<'_> {
    /// Written by hand because where connections come from is a trait object,
    /// and what a reader wants here is which agent this service is holding a
    /// turn for rather than the address of one.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Serving")
            .field("for_agent", &self.for_agent)
            .field("lasting", &self.lasting)
            .field("standing", &self.standing)
            .finish_non_exhaustive()
    }
}

/// What became of one message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Message {
    /// It was read and answered.
    Answered,
    /// The connection is over: the caller has gone, or what arrived cannot be
    /// gone on from.
    Ended,
}

/// Read one message, answer it, and say whether the connection survives.
///
/// The two ways a connection ends here are the caller closing it and a line
/// this service cannot go on reading — a line with no ending inside the bound,
/// or bytes that are not text. Both of the second kind are **answered first**,
/// so the contract's *refused in words and never dropped* holds right up to the
/// moment the connection has to go. The sentence is `alo-protocol`'s own; what
/// the caller of this passes in is which of the two doors it goes back on,
/// because an answer to an agent and an answer to a person are two types and
/// this file will not choose between them for a caller.
///
/// An answer this machine could not write is the one thing that is nobody's
/// fault but ours: `alo-protocol` proves nothing an alo OS verb can produce is
/// too long for the wire, so there is nothing to say about it to a client and
/// the connection is closed.
fn one_message(
    line: &mut Line,
    answering: impl FnOnce(&str) -> Option<String>,
    refusing: impl FnOnce(NotUnderstood) -> Option<String>,
) -> Message {
    match line.heard() {
        Ok(Some(said)) => match answering(&said) {
            Some(written) if line.say(&written).is_ok() => Message::Answered,
            _ => Message::Ended,
        },
        Ok(None) => Message::Ended,
        Err(why) => {
            if let Some(written) = why.what_to_say().and_then(refusing) {
                drop(line.say(&written));
            }
            // Asked rather than assumed. Every way a line can be unreadable
            // leaves the next byte in the middle of something, so the answer is
            // always the same one — and `alo-agentd`'s reader is where somebody
            // adding a fourth way has to decide whether that is still true.
            if why.is_the_end_of_the_connection() {
                Message::Ended
            } else {
                Message::Answered
            }
        }
    }
}

/// What time it is.
///
/// The one place in this workspace that reads a clock. Every crate underneath
/// takes `now` as an argument — item 1's rule, so that expiry is arithmetic
/// rather than a wait and the daemon and the settings panel cannot disagree
/// about the moment. The rule needs somewhere to end, and a service is the
/// honest place: it is the thing that is really running while time passes.
fn this_moment() -> SystemTime {
    SystemTime::now()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::stopping::Stop;
    use crate::testing::{
        Pretending, a_folder_with_an_invoice, a_message, granting, hour, in_english,
    };
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_protocol::Standing;
    use alo_record::Record;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};

    /// A record that cannot be written to, which is what a full disk looks like
    /// from inside a turn.
    #[derive(Default)]
    struct ANoSpaceLeftDisk;

    impl alo_turn::Kept for ANoSpaceLeftDisk {
        fn keep(&mut self, _entry: alo_record::Entry) -> Result<(), alo_keeping::NotKept> {
            Err(alo_keeping::NotKept::NotAddedTo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "no space left on device".to_owned(),
            })
        }
    }

    /// What a client thread is handed: where to connect, which file the fixture
    /// made, and the only thing that ends the service.
    struct Told {
        /// The socket.
        at: PathBuf,
        /// The file in the granted folder.
        invoice: PathBuf,
        /// What stops the service.
        stop: Stop,
    }

    /// One client, talking on one connection.
    ///
    /// The service runs in the thread that called it — it is a loop that sleeps
    /// in `poll` — so everything a test does to it happens from another thread.
    struct Talking {
        /// What is written to.
        writing: UnixStream,
        /// What is read back, a line at a time.
        reading: BufReader<UnixStream>,
    }

    impl Talking {
        /// Connect to the service.
        fn to(at: &Path) -> Self {
            let connection = UnixStream::connect(at).unwrap();
            Self {
                reading: BufReader::new(connection.try_clone().unwrap()),
                writing: connection,
            }
        }

        /// Say one thing and read what comes back.
        fn asking(&mut self, asks: &str) -> String {
            self.saying(&a_message(asks))
        }

        /// Ask what is waiting, and answer with the numbers the person is
        /// shown beside the sentences.
        ///
        /// Read back through `alo-protocol` rather than searched for in the
        /// text, because what these tests are about is that the number a person
        /// answers with is the number they were shown — a literal written here
        /// would be this test agreeing with itself about where the capability
        /// model starts counting.
        fn what_is_waiting(&mut self) -> Vec<u64> {
            let said = self.asking(r#"{"waiting":{}}"#);
            ToAPerson::read(said.trim_end())
                .unwrap()
                .changes()
                .unwrap()
                .iter()
                .map(Standing::number)
                .collect()
        }

        /// Put one line on the socket exactly as written, and read the answer.
        fn saying(&mut self, line: &str) -> String {
            self.writing
                .write_all(line.as_bytes())
                .and_then(|()| self.writing.write_all(b"\n"))
                .unwrap();
            let mut back = String::new();
            self.reading.read_line(&mut back).unwrap();
            back
        }
    }

    /// Propose renaming the fixture's file, as an agent would ask for it.
    fn renaming(invoice: &Path) -> String {
        format!(
            r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
            invoice.display()
        )
    }

    /// Run the service against a machine of its own, with a client thread
    /// driving it and stopping it when it is done.
    ///
    /// The client's thread is where the test really is: it connects, talks, and
    /// then asks the service to stop, which is the only thing that ends the
    /// loop. The service runs here, in the thread that called this, because it
    /// is what sleeps in `poll`.
    fn while_it_runs(
        what: &str,
        sides: &[Option<Side>],
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> (Served, Record, PathBuf) {
        let mut record = Record::default();
        let (served, invoice) = while_it_runs_keeping(what, sides, &mut record, talking).unwrap();
        (served, record, invoice)
    }

    /// The same, with somewhere else to write the record and the service's own
    /// answer handed back rather than unwrapped.
    fn while_it_runs_keeping(
        what: &str,
        sides: &[Option<Side>],
        kept: &mut dyn alo_turn::Kept,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        while_it_runs_for("@files", what, sides, kept, talking)
    }

    /// The same again, for a machine that says its agent is something else.
    ///
    /// Which agent this service holds turns for is what a machine says about
    /// itself, and item 21e is what reads it — so a name it could get wrong is
    /// a name these tests have to be able to give.
    fn while_it_runs_for(
        agent: &str,
        what: &str,
        sides: &[Option<Side>],
        kept: &mut dyn alo_turn::Kept,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        let strings = in_english();
        let (folder, invoice) = a_folder_with_an_invoice(what);
        let (waking, stop) = Waking::made().unwrap();
        let knocking = Pretending::handing_out(what, sides);
        let told = Told {
            at: knocking.at(),
            invoice: invoice.clone(),
            stop,
        };

        let mut indicator = Indicator::default();
        let mut machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut indicator, kept)
                .unwrap();
        let mut grants = granting(&folder, this_moment());

        let client = std::thread::spawn(move || talking(told));
        let served = Serving::of(&knocking, &waking, agent, hour(), hour())
            .until_stopped(&mut machine, &mut grants);
        client.join().unwrap();
        served.map(|served| (served, invoice))
    }

    /// **A change proposed on one door is approved on the other, while both are
    /// open.** The test this whole file exists for: a service that read one
    /// connection to the end would never reach the second message, because the
    /// agent is waiting for an answer only the person can cause.
    #[test]
    fn a_change_is_proposed_by_an_agent_and_approved_by_a_person() {
        let (served, record, invoice) = while_it_runs(
            "two-doors",
            &[Some(Side::Agent), Some(Side::Person)],
            |told| {
                let mut agent = Talking::to(&told.at);
                let mut person = Talking::to(&told.at);

                let proposed = agent.asking(&renaming(&told.invoice));
                assert!(proposed.contains("proposed"), "{proposed}");

                let waiting = person.what_is_waiting();
                assert_eq!(waiting.len(), 1, "one change was proposed: {waiting:?}");
                let number = waiting.first().unwrap();

                let approved = person.asking(&format!(r#"{{"approve":{{"number":{number}}}}}"#));
                assert!(approved.contains("renamed"), "{approved}");
                told.stop.stop();
            },
        );

        assert_eq!(served.turns(), 1);
        assert_eq!(served.messages(), 3);
        assert!(!invoice.is_file(), "the file did not move on the disk");
        assert!(
            invoice.with_file_name("march-final.pdf").is_file(),
            "the change was approved and nothing happened"
        );
        assert_eq!(record.len(), 1, "one approval, one execution, one entry");
    }

    /// **A turn is an agent's connection.** It ends when the connection does,
    /// and the next agent gets a turn of its own — which is how a machine that
    /// serves one turn at a time serves more than one in a day.
    #[test]
    fn the_turn_ends_with_the_connection_and_the_next_one_gets_its_own() {
        let (served, _record, _invoice) = while_it_runs(
            "one-at-a-time",
            &[Some(Side::Agent), Some(Side::Agent)],
            |told| {
                let mut first = Talking::to(&told.at);
                assert!(
                    first
                        .asking(&format!(
                            r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
                            told.invoice.parent().unwrap().display()
                        ))
                        .contains("listed")
                );
                drop(first);

                // The second connection is refused if the first turn is still
                // held, so an answer at all is the assertion.
                let mut second = Talking::to(&told.at);
                let read = second.asking(&format!(
                    r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{}"}}]}}}}"#,
                    told.invoice.display()
                ));
                assert!(read.contains("4180.00"), "{read}");
                told.stop.stop();
            },
        );

        assert_eq!(served.turns(), 2);
    }

    /// **A change nobody answered goes away with the turn that proposed it.**
    /// The person's shell outlives the turn and is asked afterwards: what is
    /// waiting is nothing, because a question belongs to the turn it was asked
    /// in.
    #[test]
    fn a_change_nobody_answered_goes_away_with_the_turn() {
        let (_served, _record, invoice) = while_it_runs(
            "unanswered",
            &[Some(Side::Person), Some(Side::Agent)],
            |told| {
                let mut person = Talking::to(&told.at);
                let mut agent = Talking::to(&told.at);

                assert!(
                    agent
                        .asking(&renaming(&told.invoice))
                        .contains("march-final.pdf")
                );
                assert!(
                    person.asking(r#"{"waiting":{}}"#).contains("march-final"),
                    "the change was not waiting while the turn was open"
                );

                drop(agent);
                // The service notices the agent has gone when it next wakes,
                // and it wakes because the person says something.
                let mut waiting = person.asking(r#"{"waiting":{}}"#);
                if waiting.contains("march-final") {
                    waiting = person.asking(r#"{"waiting":{}}"#);
                }
                assert!(
                    !waiting.contains("march-final"),
                    "a question outlived the turn that asked it: {waiting}"
                );
                told.stop.stop();
            },
        );

        assert!(invoice.is_file(), "an unanswered change ran anyway");
    }

    /// **A stranger is told nothing and counted.** Item 21c decided the
    /// silence — answering would say there is an alo OS daemon here — and this
    /// is what it left this item: being told nothing is not the same as nothing
    /// being noticed.
    #[test]
    fn a_stranger_is_told_nothing_and_counted() {
        let (served, _record, _invoice) =
            while_it_runs("stranger", &[None, Some(Side::Person)], |told| {
                let stranger = UnixStream::connect(&told.at).unwrap();
                let mut back = String::new();
                // The service closes it, so the read ends rather than blocking.
                drop(BufReader::new(stranger).read_line(&mut back));
                assert!(back.is_empty(), "a stranger was told something: {back}");

                let mut person = Talking::to(&told.at);
                assert!(person.asking(r#"{"waiting":{}}"#).contains("waiting"));
                told.stop.stop();
            });

        assert_eq!(served.strangers_turned_away(), 1);
        assert_eq!(served.turns(), 0, "a stranger began a turn");
    }

    /// **A second agent is refused in words and the turn goes on.** One machine
    /// has one turn, and the one that is already running is not interrupted by
    /// somebody else's arrival.
    #[test]
    fn a_second_agent_is_refused_in_words_and_the_first_turn_goes_on() {
        let (served, _record, _invoice) = while_it_runs(
            "second-agent",
            &[Some(Side::Agent), Some(Side::Agent)],
            |told| {
                let mut first = Talking::to(&told.at);
                assert!(first.asking(&renaming(&told.invoice)).contains("proposed"));

                let mut second = Talking::to(&told.at);
                let refused = second.asking(r#"{"waiting":{}}"#);
                assert!(
                    refused.contains("already in a turn"),
                    "a second agent was served: {refused}"
                );

                // And the first is still being served.
                assert!(
                    first
                        .asking(&format!(
                            r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{}"}}]}}}}"#,
                            told.invoice.display()
                        ))
                        .contains("4180.00")
                );
                told.stop.stop();
            },
        );

        assert_eq!(served.turns(), 1);
    }

    /// **A second shell is refused in words too**, and told which of the two
    /// things in front of the person to close.
    #[test]
    fn a_second_shell_is_told_something_else_is_already_answering() {
        while_it_runs(
            "second-shell",
            &[Some(Side::Person), Some(Side::Person)],
            |told| {
                let mut first = Talking::to(&told.at);
                assert!(first.asking(r#"{"waiting":{}}"#).contains("waiting"));

                let mut second = Talking::to(&told.at);
                let refused = second.asking(r#"{"waiting":{}}"#);
                assert!(
                    refused.contains("already answering"),
                    "a second shell was served: {refused}"
                );
                told.stop.stop();
            },
        );
    }

    /// **A message that is not a request is answered and the connection goes
    /// on.** `docs/contracts/daemon-protocol.md`'s *refused in words and never
    /// dropped*, from the socket rather than from a unit test of the reader.
    #[test]
    fn a_message_that_is_not_a_request_is_answered_and_the_caller_stays() {
        let (served, _record, _invoice) =
            while_it_runs("gibberish", &[Some(Side::Person)], |told| {
                let mut person = Talking::to(&told.at);
                assert!(person.saying("not json at all").contains("refused"));
                assert!(
                    person.asking(r#"{"waiting":{}}"#).contains("waiting"),
                    "the connection was closed on a message it could have answered"
                );
                told.stop.stop();
            });

        assert_eq!(served.messages(), 2);
    }

    /// **A line with no end to it is answered and then closed.** There is no
    /// way to find the start of the next message, so the connection goes — but
    /// not before whoever sent it has been told why.
    #[test]
    fn a_line_with_no_end_is_answered_and_then_closed() {
        while_it_runs("flood", &[Some(Side::Person)], |told| {
            let mut person = Talking::to(&told.at);
            let flood = "a".repeat(64 * 1024);
            let mut refused = String::new();
            for _ in 0..64 {
                if person.writing.write_all(flood.as_bytes()).is_err() {
                    break;
                }
            }
            drop(person.reading.read_line(&mut refused));
            assert!(refused.contains("shorter"), "{refused}");

            // And the connection is gone: nothing more comes off it.
            let mut nothing = String::new();
            drop(person.reading.read_line(&mut nothing));
            assert!(nothing.is_empty(), "the connection was kept: {nothing}");
            told.stop.stop();
        });
    }

    /// **A machine that cannot write down what it did stops serving.** What is
    /// missing is evidence, and a service that went on acting without it would
    /// be doing exactly what the gate's *every execution leaves a record* is
    /// there to prevent.
    #[test]
    fn a_service_that_cannot_write_down_what_it_did_stops() {
        let mut disk = ANoSpaceLeftDisk;
        let stopped = while_it_runs_keeping("no-space", &[Some(Side::Agent)], &mut disk, |told| {
            let mut agent = Talking::to(&told.at);
            let said = agent.asking(&format!(
                r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{}"}}]}}}}"#,
                told.invoice.display()
            ));
            assert!(
                said.contains("refused"),
                "a read was answered on a machine that could not write it down: {said}"
            );
            // Nothing stops the service here: it stops itself.
            drop(told.stop);
        })
        .unwrap_err();

        assert!(matches!(stopped, NotServed::NothingIsWrittenDown));
    }

    /// **A machine that named no agent serves nobody, and says so.** The name
    /// is what a machine says about itself and item 21e is what reads it, so it
    /// is a thing that can be got wrong — and the answer is the service
    /// stopping with the capability model's own reason rather than holding a
    /// turn that belongs to nobody.
    ///
    /// Nothing is written down and nothing is refused to the agent, because
    /// nothing about it was ever asked: the turn failed before its first
    /// message was read.
    #[test]
    fn a_machine_that_named_no_agent_stops_rather_than_holding_a_nameless_turn() {
        let mut record = Record::default();
        let stopped =
            while_it_runs_for("", "no-agent", &[Some(Side::Agent)], &mut record, |told| {
                let mut agent = Talking::to(&told.at);
                // Nothing comes back: the service stops as the connection is taken.
                let mut back = String::new();
                drop(agent.writing.write_all(b"{}\n"));
                drop(agent.reading.read_line(&mut back));
                assert!(
                    back.is_empty(),
                    "a nameless turn answered something: {back}"
                );
                drop(told.stop);
            })
            .unwrap_err();

        assert!(
            matches!(
                stopped,
                NotServed::NoTurn {
                    why: alo_capability::GrantError::Anonymous
                }
            ),
            "{stopped:?}"
        );
        assert_eq!(record.len(), 0, "a turn that never began was written down");
    }

    /// A service nobody has said anything to stops when it is asked to, which
    /// is the whole of what makes it stoppable while it is asleep.
    #[test]
    fn a_service_that_has_done_nothing_still_stops() {
        let (served, _record, _invoice) = while_it_runs("quiet", &[], |told| {
            told.stop.stop();
        });

        assert_eq!(served, Served::default());
    }

    /// **Nothing a verb can answer with is too long to put on the wire**, which
    /// is what makes *an answer this machine could not write* a bug in this
    /// machine rather than something a client can arrange by asking for a big
    /// file. `alo-protocol` derives the bound from `alo-files`' own; this is
    /// the service reading that derivation rather than trusting it.
    #[test]
    fn nothing_a_verb_answers_with_is_too_long_for_the_wire() {
        assert!(
            u64::try_from(alo_protocol::LONGEST_ANSWER).unwrap_or(u64::MAX) > alo_files::MOST_READ
        );
    }
}
