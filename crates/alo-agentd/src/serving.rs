//! One machine, two doors, one port, one turn.
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
//! over the socket, the connections, the port, the discovery socket and the
//! end a stop arrives on (`crate::unix::ready`).
//!
//! So there are no threads here, no channels, no lock around the machine, and
//! nothing shared between two things that run at once. The machine is a local
//! variable that one loop owns. That is worth saying plainly, because the
//! obvious shape for *two connections at once* is a thread each and a mutex,
//! and it would have put the capability model behind a lock in the one service
//! whose whole value is being small enough to read.
//!
//! # And the port presence advertises is the same loop
//!
//! Since the daemon bound the port, the machine has a third door: the network's
//! (`alo_corridor::Doorway`), through which a proven verb from a paired machine
//! begins a remote turn on this machine's grants. The doorway borrows the
//! machine exactly as a local turn does, so **the lock over the machine is the
//! loop**, and the two kinds of turn take it in turn:
//!
//! - while no local agent is connected, the doorway holds the machine — for a
//!   remote turn, or for the next verb — and every message on the port is read
//!   and answered in the round it arrives, because nothing a remote verb does
//!   blocks either;
//! - while a local turn is under way, **the port is not polled at all**. A
//!   verb from the network waits in the kernel's backlog for the local turn to
//!   end, which is the plan's *a verb from the network waits on a local turn*
//!   made literal; the asking machine's own patience is the wire's, and a verb
//!   that outwaits it is answered to a connection that has gone.
//! - a local agent knocking while a remote turn is open **ends the remote
//!   turn** and begins its own (`Doorway::given_back`). A remote turn between
//!   verbs holds nothing but the window for the next one — every verb was
//!   answered in the round it arrived — so the person's own agent waits for
//!   nothing, and the next remote verb begins a fresh turn on a fresh proof.
//!
//! The pairings and the proposals are a different matter: the person's own
//! surface changes them from outside this loop, so they are behind one lock
//! (`crate::network`) taken for the length of one message. The proofs seen
//! (`alo_nearby::Seen`) travel with the machine between doorways, so a replay
//! across a local turn is still a replay.
//!
//! Discovery is answered in every state, because presence never says what a
//! machine is doing — and on every network the machine is on, which is
//! followed in every state too: when the kernel says a network appeared, the
//! wire joins discovery on it (`crate::joining`).
//!
//! **And the person's door pairs from any state.** The four requests about a
//! pairing (`crate::pairing`) are answered against the lock and the wire
//! whether a local turn, a remote turn or nobody holds the machine, because a
//! person pairs their machine from their own shell at any hour. A proposal
//! that arrives on the port is shown by waiting on that door, so the wire is
//! handed the surface only while a shell is connected, and *nobody to show it
//! to* otherwise — the true word for a machine with no one in front of it.
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
//! **And no agent is called by a machine's name.** `machine:` and an identity
//! is how a grant to a paired machine is spelt, so an agent called that would
//! be answered by that machine's grants. `crate::Described` refuses the name
//! where it is read; [`Serving::until_stopped`] refuses it again before any
//! turn, so that a service handed the name any other way holds no turn for it
//! (`alo_nearby::Origin::names_a_machine`).
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
//!
//! # A shortening runs between turns, and never inside one
//!
//! The other thing this service does with a clock is remove what the machine no
//! longer keeps ([`crate::ageing`] is *when*, `alo_turn::Machine::shorten` is
//! the door, `alo-keeping` is *what*). It happens in the rounds where no turn
//! is under way — local or remote — and that is not a rule anybody has to
//! remember: while a turn is under way the [`Turning`] or the doorway's turn
//! holds the machine, so there is nothing here to ask. A shortening due while
//! a remote turn is open runs at the next round in which the machine is free.
//!
//! It is the right place as well as the only one. A shortening replaces the file
//! the record is being written to, and one that ran mid-turn and was refused
//! would end an agent's turn with *nothing is written down* because the machine
//! was tidying up — a refusal about the wrong thing, told to the wrong caller.
//! Between turns there is nobody to tell and nothing to interrupt: a shortening
//! that fails leaves the record exactly as long as it was, is counted, and the
//! service goes on.

use std::time::{Duration, SystemTime};

use alo_context::Context;
use alo_corridor::Doorway;
use alo_nearby::{Origin, Seen, Surface};
use alo_protocol::{NotUnderstood, ToAPerson, ToAnAgent};
use alo_strings::{Filling, Strings};
use alo_turn::{Machine, Shortened, Turning};

use crate::ageing::Ageing;
use crate::answering::what_a_person_said;
use crate::corridor::Corridor;
use crate::doing::what_an_agent_said;
use crate::hearing::{self, Judging};
use crate::holding::Holding;
use crate::knocking::Knocking;
use crate::lines::Line;
use crate::network::TheNetwork;
use crate::pairing::Nearby;
use crate::questions::Questions;
use crate::refusing::NotServed;
use crate::rereading::WhatIsGranted;
use crate::side::Side;
use crate::stopping::Waking;
use crate::surface::NobodyToShowItTo;
use crate::terms::Terms;
use crate::unix::ready;
use crate::wire::Wire;
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
    /// How many entries were removed from the record because the machine no
    /// longer keeps them.
    ///
    /// The only number here about evidence going away, and it is here for the
    /// reason `alo-keeping` writes a mark into a record's first line: a
    /// shortening that left nothing behind would be a shortening nobody could
    /// have noticed.
    removed: u64,
    /// How many shortenings the machine refused to make.
    ///
    /// Nothing was removed in any of them — a refusal by `alo-keeping` leaves
    /// the record exactly as it was — so this is a machine keeping **more** than
    /// its rule rather than less. It is still what somebody wants to see: a
    /// record with a line nobody can read is refused every time it is asked, and
    /// a number that stays at zero is how anybody would know it never happened.
    not_shortened: u64,
    /// How many messages arrived on the port presence advertises, each
    /// answered by the wire its path named or refused as for none of them.
    heard: u64,
    /// How many times another machine asked whether this one exists, and was
    /// told.
    found: u64,
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

    /// How many entries the machine removed because it no longer keeps them.
    #[must_use]
    pub const fn entries_removed(&self) -> u64 {
        self.removed
    }

    /// How many shortenings the machine refused to make, in none of which
    /// anything was removed.
    #[must_use]
    pub const fn shortenings_refused(&self) -> u64 {
        self.not_shortened
    }

    /// How many messages arrived on the port presence advertises.
    #[must_use]
    pub const fn heard_on_the_port(&self) -> u64 {
        self.heard
    }

    /// How many times this machine answered that it exists.
    #[must_use]
    pub const fn discovery_answered(&self) -> u64 {
        self.found
    }
}

/// The agent service, running.
///
/// Holds what the machine was told about itself and nothing that changes: the
/// door connections arrive at, the end a stop arrives on, the port and the
/// identity presence advertises, the one lock over the pairings, and the
/// terms — which agent this machine has, how long a turn and a change last,
/// what may leave. Where those come from is the process's.
pub struct Serving<'a> {
    /// Where connections come from, and which door each is on.
    knocking: &'a dyn Knocking,
    /// What a stop arrives on.
    waking: &'a Waking,
    /// The port presence advertises, bound, and discovery answered.
    wire: &'a Wire,
    /// The pairings and the proposals, behind one lock.
    network: &'a TheNetwork,
    /// What this machine was told about itself.
    terms: Terms<'a>,
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
    ///
    /// Every one of the terms is `crate::Described`'s, read off the file
    /// whoever stands the machine up wrote; the wire is the port bound and the
    /// identity kept; the network is the one lock over the pairings. The rule
    /// the record is kept under is here rather than inside the machine because
    /// *when* a shortening runs is this service's — it is the thing that is
    /// really running while time passes — and *what* one removes is
    /// `alo-keeping`'s.
    #[must_use]
    pub const fn of(
        knocking: &'a dyn Knocking,
        waking: &'a Waking,
        wire: &'a Wire,
        network: &'a TheNetwork,
        terms: Terms<'a>,
    ) -> Self {
        Self {
            knocking,
            waking,
            wire,
            network,
            terms,
        }
    }

    /// Serve until somebody asks the service to stop.
    ///
    /// The machine is borrowed rather than built here, so that where the record
    /// really goes stays one decision made in one place. The words this service
    /// says are **the machine's own** — taken from it rather than passed in
    /// beside it — because the record writes down what a person was shown, and
    /// two vocabularies would be a screen in one language and a record in
    /// another. `surface` is what shows a proposal from another machine to the
    /// person here, and is the shell's.
    ///
    /// # Errors
    ///
    /// [`NotServed`], which is always the machine rather than a client: a
    /// message that is not a request, a stranger at the door, a caller that
    /// hangs up mid-message and a message on the port that could not be
    /// answered are all served and survived. In every one of them the turn that
    /// was under way has been ended and its grant given back.
    pub fn until_stopped(
        &self,
        machine: &mut Machine<'_>,
        granted: &mut WhatIsGranted<'_>,
        questions: &mut Questions,
        surface: &mut dyn Surface,
    ) -> Result<Served, NotServed> {
        if Origin::names_a_machine(self.terms.for_agent) {
            return Err(NotServed::AnAgentNamedAMachine {
                named: self.terms.for_agent.trim().to_owned(),
            });
        }
        let strings = machine.strings();
        let mut held = Held::default();
        let mut served = Served::default();
        let mut ageing = Ageing::under(self.terms.keeping);
        let mut seen = Seen::nothing();

        loop {
            // Between local turns the network's door holds the machine, with
            // every proof seen so far, so a verb from a paired machine is
            // judged and answered in the round it arrives.
            let mut doorway = Doorway::keeping(
                self.wire.here().clone(),
                &mut *machine,
                seen,
                self.terms.lasting,
                self.terms.standing,
            )?;
            while held.agent.is_none() {
                // Before the wait rather than after it, so that the first
                // shortening happens before this machine serves anything: it is
                // the one catching up on however long the machine was switched
                // off for. The moment is read here and again inside the round,
                // because a wait sits between them.
                let now = this_moment();
                if ageing.due(now)
                    && let Some(free) = doorway.machine()
                {
                    shortening(free, &mut ageing, &mut served, now);
                }
                if self.one_round(
                    &mut held,
                    &mut Holding::TheNetwork {
                        doorway: &mut doorway,
                        network: self.network,
                        questions: &mut *questions,
                    },
                    granted,
                    strings,
                    &mut served,
                    ageing.before(now),
                    surface,
                )? == Next::Stopped
                {
                    return Ok(served);
                }
            }

            // A local agent knocked: the network's door gives the machine
            // back, ending any remote turn, and hands over what it has seen.
            let Some((machine, seen_so_far)) = doorway.given_back(granted.holding_mut()) else {
                return Err(NotServed::TheMachineWasLost);
            };
            seen = seen_so_far;

            // A turn is where what answers a question is looked for, so a turn
            // beginning is where the last one is forgotten. Here rather than at
            // the end of a turn, because a service that stopped mid-turn would
            // otherwise leave the runtime it found for the next one.
            questions.a_new_turn();

            let mut turning = Turning::beginning(
                Context::at_invocation(this_moment()),
                self.terms.for_agent,
                self.terms.lasting,
                granted.holding_mut(),
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
                // No timeout: a turn is not interrupted by the clock, and there
                // is nothing this service could do with the wake-up while the
                // machine is held by the turn.
                match self.one_round(
                    &mut held,
                    &mut Holding::ATurn {
                        turning: &mut turning,
                        questions: &mut *questions,
                    },
                    granted,
                    strings,
                    &mut served,
                    None,
                    surface,
                ) {
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
                // Asked before the record, because a service that has lost a
                // thread has not stopped keeping evidence and must not report
                // that it has: the two are different things wrong with the
                // machine and send whoever reads the log to two different
                // places.
                if turning.a_thread_is_lost() {
                    over = Err(NotServed::AThreadIsInsideATurn);
                    break;
                }
                if turning.is_closed() {
                    over = Err(NotServed::NothingIsWrittenDown);
                    break;
                }
            }
            let _gave_a_grant_back = turning.ending(granted.holding_mut());
            held.agent = None;

            if over? {
                return Ok(served);
            }
        }
    }

    /// Wait until something has happened, and deal with all of it.
    ///
    /// The order is the person, then the agent, then the door, then the port,
    /// then discovery: somebody already connected is answered before somebody
    /// new is let in, and the person is answered before the agent because an
    /// approval that has already arrived should not wait behind the next thing
    /// an agent thought of.
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
    ///
    /// **The port is waited on only while the network's door has the
    /// machine.** During a local turn it is left out of the wait, and a verb
    /// from a paired machine waits in the backlog for the turn to end.
    ///
    /// **`for_at_most` is how long this round may sleep**, and `None` is until
    /// somebody says something. A round that slept the whole of it finds nothing
    /// ready, does nothing, and answers [`Next::GoOn`] — which is what brings
    /// the caller back round to a shortening that has come due.
    #[expect(
        clippy::too_many_arguments,
        reason = "one round of one service, and every one of these is a thing the round has to reach; grouping them would be a struct that exists for one call"
    )]
    fn one_round(
        &self,
        held: &mut Held,
        holding: &mut Holding<'_, '_, '_>,
        granted: &mut WhatIsGranted<'_>,
        strings: &Strings,
        served: &mut Served,
        for_at_most: Option<Duration>,
        surface: &mut dyn Surface,
    ) -> Result<Next, NotServed> {
        let the_network_has_the_machine = matches!(holding, Holding::TheNetwork { .. });
        let (stopped, person, agent, knocked, on_the_port, asked_who_is_here, networks_changed) = {
            let waiting_on = [
                Some(self.waking.waiting_on()),
                held.person.as_ref().map(Line::waiting_on),
                held.agent.as_ref().map(Line::waiting_on),
                Some(self.knocking.waiting_on()),
                the_network_has_the_machine.then(|| self.wire.waiting_on()),
                Some(self.wire.discovery_waiting_on()),
                self.wire.networks_waiting_on(),
            ];
            let [
                stopped,
                person,
                agent,
                knocked,
                on_the_port,
                asked_who_is_here,
                networks_changed,
            ] = ready(&waiting_on, for_at_most).map_err(|why| NotServed::NotWaiting { why })?;
            (
                stopped,
                person,
                agent,
                knocked,
                on_the_port,
                asked_who_is_here,
                networks_changed,
            )
        };

        if stopped {
            return Ok(Next::Stopped);
        }
        let now = this_moment();

        if person {
            // A record that broke while the person was being answered ends the
            // service rather than the connection, so it is carried out of the
            // closure rather than turned into an answer: what is missing is
            // evidence, and there is nothing to say to a caller about it.
            let mut nothing_written_down = false;
            // What the four requests about a pairing are answered against:
            // the one lock, and the link this wire is bound to — and what this
            // wire tells that link about the machine, as it holds it now.
            let advertising = self.wire.advertising();
            let nearby = Nearby {
                network: self.network,
                looking: self.wire,
                advertising: &advertising,
            };
            let answered = held.person.as_mut().map(|line| {
                one_message(
                    line,
                    |said| match what_a_person_said(said, holding, granted, &nearby, strings, now) {
                        Ok(told) => told.written().ok(),
                        Err(_) => {
                            nothing_written_down = true;
                            None
                        }
                    },
                    |why| ToAPerson::refused(&why.said(strings)).written().ok(),
                )
            });
            if nothing_written_down {
                return Err(NotServed::NothingIsWrittenDown);
            }
            if answered == Some(Message::Ended) {
                held.person = None;
            }
            if answered == Some(Message::Answered) {
                served.messages = served.messages.saturating_add(1);
            }
        }

        if agent {
            // What a question down the corridor is asked against: the one lock
            // over the pairings, the link this wire is bound to, and what the
            // person here called the machines they paired with.
            let corridor = Corridor {
                network: self.network,
                looking: self.wire,
                naming: self.terms.naming,
            };
            let answered = match (held.agent.as_mut(), holding.underway()) {
                // A connection with no turn behind it cannot be served and
                // cannot be left waiting either: it would be ready for ever and
                // read by nobody. It is the end of the connection, which for an
                // agent is the end of what it came for. Unreachable while the
                // two loops above are the only callers, and answered here
                // rather than assumed away.
                (None, _) | (Some(_), None) => Message::Ended,
                (Some(line), Some((turning, questions))) => one_message(
                    line,
                    |said| {
                        what_an_agent_said(
                            said,
                            turning,
                            questions,
                            Some(&corridor),
                            granted.holding(),
                            strings,
                            self.terms.standing,
                            now,
                        )
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
                    self.let_in(held, side, connection, holding.turning().is_some(), strings);
                }
                Err(why) if why.is_only_this_connection() => {
                    served.strangers = served.strangers.saturating_add(1);
                }
                Err(why) => return Err(NotServed::NotTaken(why)),
            }
        }

        if on_the_port
            && let Holding::TheNetwork {
                doorway, questions, ..
            } = holding
        {
            let knocked = self.wire.accept_one().map_err(NotServed::TheWire)?;
            // A proposal is shown by waiting on the person's door, and there
            // is a person's door to wait on only while a shell is connected;
            // with none, nobody can be shown it and it is refused as such.
            let mut nobody = NobodyToShowItTo;
            let mut judging = Judging {
                network: self.network,
                surface: if held.person.is_some() {
                    surface
                } else {
                    &mut nobody
                },
                naming: self.terms.naming,
                policy: &self.terms.policy,
                asking_at: self.wire.asking_at(),
                questions,
            };
            hearing::heard(knocked, doorway, granted.holding_mut(), &mut judging, now)?;
            served.heard = served.heard.saturating_add(1);
        }

        if networks_changed {
            // A network appeared, changed or went: discovery is joined on
            // every network the machine is on now. Nothing it says moves, and
            // a network that will not join is a line in the log, not a stop.
            self.wire.networks_changed();
        }

        if asked_who_is_here {
            // A question that was not one for this service — a printer's, or
            // this machine's own answer coming back round — is nothing to
            // count; a socket that will not read is the machine's.
            if self
                .wire
                .answer_discovery()
                .map_err(NotServed::TheWire)?
                .is_some()
            {
                served.found = served.found.saturating_add(1);
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
    /// is refused because one is connected *or* because a local turn is under
    /// way at all. The second half is what makes *an agent never acts under a
    /// grant another agent's invocation made* a property of this method rather
    /// than a property of the order [`Serving::one_round`] happens to do things
    /// in — the ordering is still right, and this is what would hold if it were
    /// not. A remote turn is not a reason to refuse: the caller ends it and
    /// begins the local one, as this file's header says.
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
            .field("wire", &self.wire)
            .field("terms", &self.terms)
            .finish_non_exhaustive()
    }
}

/// Remove what this machine no longer keeps, and count what that came to.
///
/// A free function rather than a method, because it touches nothing on
/// [`Serving`] — the rule travels in the [`Ageing`], the moment is passed in,
/// and what is removed is decided by `alo-keeping` from those two alone. There
/// is deliberately no way to say *remove this*.
///
/// **The attempt is recorded before the answer is looked at**, so a machine
/// whose record cannot be read tries again at the next interval rather than on
/// every round for as long as it is switched on. And a refusal is counted and
/// survived: nothing was removed in it, so the machine is keeping more than its
/// rule rather than less, and stopping the service over it would take away the
/// only thing that still writes evidence down.
fn shortening(
    machine: &mut Machine<'_>,
    ageing: &mut Ageing,
    served: &mut Served,
    now: SystemTime,
) {
    ageing.ran(now);
    match machine.shorten(ageing.keeping(), now) {
        Ok(Shortened::Ran(pruned)) => {
            let removed = u64::try_from(pruned.removed()).unwrap_or(u64::MAX);
            served.removed = served.removed.saturating_add(removed);
        }
        Ok(Shortened::NotOnADisk) => ageing.nothing_to_shorten(),
        Err(_) => served.not_shortened = served.not_shortened.saturating_add(1),
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
        NothingIsRemembered, Pretending, a_folder_with_an_invoice, a_message, granting, hour,
        in_english, nothing_has_been_chosen,
    };
    use alo_capability::Grants;
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_protocol::Standing;
    use alo_record::Record;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
    use std::os::unix::fs::PermissionsExt as _;
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use crate::network::{KeepingPairings, NothingKeepsPairings, TheNetwork};
    use crate::terms::Terms;
    use crate::testing::{paired_between, reception, the_studio};
    use crate::wire::Wire;
    use alo_egress::EgressPolicy;
    use alo_keeping::Keeping;

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

    /// Nothing here is shortened: these tests are about a turn that cannot
    /// write, and a machine with no room on it has nothing to remove either.
    impl alo_turn::Shortening for ANoSpaceLeftDisk {
        fn shorten(
            &mut self,
            _keeping: Keeping,
            _now: SystemTime,
        ) -> Result<Shortened, alo_keeping::NotKept> {
            Ok(Shortened::NotOnADisk)
        }
    }

    /// A record on a disk that refuses every shortening, which is what a line
    /// nobody can read looks like from inside the service.
    ///
    /// It keeps what it is handed, because a machine that will not shorten its
    /// record is still a machine that writes one — that is the whole difference
    /// between a refusal to remove and a failure to keep.
    #[derive(Default)]
    struct ARecordNothingWillShorten {
        /// What it was handed.
        kept: Record,
        /// How many shortenings it refused.
        refused: usize,
    }

    impl alo_turn::Kept for ARecordNothingWillShorten {
        fn keep(&mut self, entry: alo_record::Entry) -> Result<(), alo_keeping::NotKept> {
            self.kept.keep(entry);
            Ok(())
        }
    }

    impl alo_turn::Shortening for ARecordNothingWillShorten {
        fn shorten(
            &mut self,
            _keeping: Keeping,
            _now: SystemTime,
        ) -> Result<Shortened, alo_keeping::NotKept> {
            self.refused += 1;
            Err(alo_keeping::NotKept::Damaged {
                path: "/var/lib/alo/record.jsonl".to_owned(),
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
        /// The port presence advertises, on this host.
        port: u16,
        /// The pairings and the proposals, behind the one lock — what the
        /// person's surface reaches from outside the loop.
        network: Arc<TheNetwork>,
        /// The socket the service asks reception's discovery at, for a test
        /// that wants reception to be found.
        discovery: UdpSocket,
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

        /// Read what the service said without saying anything first.
        ///
        /// A connection the service will not serve is answered the moment it is
        /// accepted and then closed, so there is nothing to ask it. A test that
        /// sent a request anyway would be writing to a socket the service has
        /// already finished with, and would fail with a broken pipe whenever the
        /// close won the race — which is what it did, about once in eight runs,
        /// until this method existed.
        fn what_it_said(&mut self) -> String {
            let mut back = String::new();
            self.reading.read_line(&mut back).unwrap();
            back
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
        kept: &mut dyn alo_turn::Shortening,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        while_it_runs_for("@files", Keeping::Forever, what, sides, kept, talking)
    }

    /// The same, on a machine an organisation has set a retention rule on.
    ///
    /// Which is what makes the shortening happen at all: a machine that keeps
    /// everything has nothing to wake up for, and these are the tests about the
    /// machine that has.
    fn while_it_runs_under(
        keeping: Keeping,
        what: &str,
        sides: &[Option<Side>],
        kept: &mut dyn alo_turn::Shortening,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        while_it_runs_for("@files", keeping, what, sides, kept, talking)
    }

    /// The same again, for a machine that says its agent is something else.
    ///
    /// Which agent this service holds turns for is what a machine says about
    /// itself, and item 21e is what reads it — so a name it could get wrong is
    /// a name these tests have to be able to give.
    fn while_it_runs_for(
        agent: &str,
        keeping: Keeping,
        what: &str,
        sides: &[Option<Side>],
        kept: &mut dyn alo_turn::Shortening,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        while_it_runs_remembering(
            agent,
            keeping,
            what,
            sides,
            kept,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            granting,
            talking,
        )
    }

    /// The same again, on a machine whose grants file a test has written, and
    /// whose person has chosen what answers questions.
    ///
    /// The one thing the person's knock reaches, handed in rather than named
    /// here for `crate::starting`'s reason: the file is `src/main.rs`'s, and a
    /// service that could find it for itself would be a service with a road from
    /// the socket to a path. What answers a question is handed in for the
    /// same reason.
    #[expect(
        clippy::too_many_arguments,
        reason = "a fixture standing a whole service up, and every one of these is a thing about \
                  the machine a test has to be able to choose"
    )]
    fn while_it_runs_remembering(
        agent: &str,
        keeping: Keeping,
        what: &str,
        sides: &[Option<Side>],
        kept: &mut dyn alo_turn::Shortening,
        remembering: &dyn crate::rereading::Remembering,
        questions: Questions,
        starting: impl FnOnce(&Path, SystemTime) -> Grants,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        while_it_runs_holding(
            agent,
            keeping,
            what,
            sides,
            kept,
            remembering,
            questions,
            (alo_nearby::Pairings::none(), Box::new(NothingKeepsPairings)),
            starting,
            talking,
        )
    }

    /// The same again, on a machine that starts holding these pairings and
    /// writes every change to this keeper — what a restart hands the service,
    /// as `src/main.rs` hands it.
    #[expect(
        clippy::too_many_arguments,
        reason = "a fixture standing a whole service up, and every one of these is a thing about \
                  the machine a test has to be able to choose"
    )]
    fn while_it_runs_holding(
        agent: &str,
        keeping: Keeping,
        what: &str,
        sides: &[Option<Side>],
        kept: &mut dyn alo_turn::Shortening,
        remembering: &dyn crate::rereading::Remembering,
        mut questions: Questions,
        paired: (alo_nearby::Pairings, Box<dyn KeepingPairings>),
        starting: impl FnOnce(&Path, SystemTime) -> Grants,
        talking: impl FnOnce(Told) + Send + 'static,
    ) -> Result<(Served, PathBuf), NotServed> {
        let strings = in_english();
        let (folder, invoice) = a_folder_with_an_invoice(what);
        let (waking, stop) = Waking::made().unwrap();
        let knocking = Pretending::handing_out(what, sides);
        // The port and the discovery socket, on this host; and a socket of
        // reception's own, at which this service is told to look for it.
        let receptions_discovery = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let wire = Wire::on(
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            the_studio(),
            receptions_discovery.local_addr().unwrap().port(),
        )
        .unwrap();
        let (pairings, keeping_pairings) = paired;
        let network = Arc::new(TheNetwork::remembering(
            the_studio(),
            pairings,
            keeping_pairings,
        ));
        let told = Told {
            at: knocking.at(),
            invoice: invoice.clone(),
            stop,
            port: wire.port(),
            network: Arc::clone(&network),
            discovery: receptions_discovery,
        };

        let mut indicator = Indicator::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            kept,
        )
        .unwrap();
        let mut grants = starting(&folder, this_moment());

        let client = std::thread::spawn(move || talking(told));
        // Every proposal is shown: the surface that really shows one is the
        // shell's, and what these tests hold is the road on either side of it.
        let mut shown = |_: &alo_nearby::Waiting| true;
        let terms = Terms {
            for_agent: agent,
            lasting: hour(),
            standing: hour(),
            keeping,
            policy: EgressPolicy::InTheBuilding,
            // What a running machine answers with: the names given on the
            // person's door, held beside the pairings.
            naming: network.names(),
        };
        let served = Serving::of(&knocking, &waking, &wire, &network, terms).until_stopped(
            &mut machine,
            &mut WhatIsGranted::of(&mut grants, remembering),
            &mut questions,
            &mut shown,
        );
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

    /// **A grant made while the service is running is honoured in the same
    /// session, and a revocation takes effect on the next question asked.**
    ///
    /// The whole of task 4, over a real socket, with an agent's turn open the
    /// entire time and nothing restarted. The person's side does what a folder
    /// picker does — it writes the file and then knocks — and the read the agent
    /// was refused a moment earlier is carried out. Then the same again in
    /// reverse: the file is rewritten with nothing in it, the person knocks, and
    /// the next question the agent asks is refused.
    ///
    /// The knock carries no grant, no path and no duration
    /// (`alo_protocol::FromAPerson::Granted` has no field for one), so
    /// re-reading the person's own file is the only thing it can cause.
    #[test]
    fn a_grant_made_while_the_service_runs_reaches_it_and_a_revocation_does_too() {
        let at = crate::testing::a_directory_of_our_own("live-grants").join("grants.toml");
        let file = crate::ThePersonsFile::at(&at);
        let writing = at.clone();

        let (served, _invoice) = while_it_runs_remembering(
            "@files",
            Keeping::Forever,
            "live-grants-service",
            &[Some(Side::Agent), Some(Side::Person)],
            &mut Record::default(),
            &file,
            nothing_has_been_chosen(),
            // The service signs in having been granted nothing, which is the
            // machine this task is about: the folder is picked afterwards.
            |_folder, _now| Grants::default(),
            move |told| {
                let folder = told.invoice.parent().unwrap().to_path_buf();
                let mut agent = Talking::to(&told.at);
                let mut person = Talking::to(&told.at);

                let refused = agent.asking(&listing(&folder));
                assert!(
                    refused.contains("refused"),
                    "something was granted before the person picked anything: {refused}"
                );

                // What a folder picker does: write the file, then knock.
                alo_remembering::kept(&writing, &granting(&folder, this_moment()), this_moment())
                    .unwrap();
                let knocked = person.asking(r#"{"granted":{}}"#);
                assert!(knocked.contains(r#""holding":1"#), "{knocked}");

                let listed = agent.asking(&listing(&folder));
                assert!(
                    listed.contains("listed"),
                    "the grant made while the service was running was not honoured: {listed}"
                );

                // And the other way: what the surface that revokes does.
                alo_remembering::kept(&writing, &Grants::default(), this_moment()).unwrap();
                let knocked = person.asking(r#"{"granted":{}}"#);
                assert!(knocked.contains(r#""holding":0"#), "{knocked}");

                let after = agent.asking(&listing(&folder));
                assert!(
                    after.contains("refused"),
                    "a revoked grant was still honoured on the next question: {after}"
                );
                told.stop.stop();
            },
        )
        .unwrap();

        assert_eq!(served.turns(), 1, "the turn was never interrupted");
        assert_eq!(served.messages(), 5);
    }

    /// **An agent knocking is refused in words, and the person's list is not
    /// read.** The same request as the test above, on the other door.
    #[test]
    fn the_same_request_on_the_agents_door_is_refused() {
        let at = crate::testing::a_directory_of_our_own("agent-knock").join("grants.toml");
        let file = crate::ThePersonsFile::at(&at);
        let writing = at.clone();
        let mut record = Record::default();

        let (served, _invoice) = while_it_runs_remembering(
            "@files",
            Keeping::Forever,
            "agent-knock-service",
            &[Some(Side::Agent)],
            &mut record,
            &file,
            nothing_has_been_chosen(),
            |_folder, _now| Grants::default(),
            move |told| {
                let folder = told.invoice.parent().unwrap().to_path_buf();
                // The file really does grant the folder, so a service that read
                // it at the agent's asking would answer the read below.
                alo_remembering::kept(&writing, &granting(&folder, this_moment()), this_moment())
                    .unwrap();

                let mut agent = Talking::to(&told.at);
                let refused = agent.asking(r#"{"granted":{}}"#);
                assert!(refused.contains("refused"), "{refused}");
                assert!(
                    refused.contains("an agent cannot say"),
                    "the agent was refused in somebody else's words: {refused}"
                );

                let after = agent.asking(&listing(&folder));
                assert!(
                    after.contains("refused"),
                    "an agent's knock made the service read the person's grants: {after}"
                );
                told.stop.stop();
            },
        )
        .unwrap();

        assert_eq!(served.messages(), 2);
        // Two entries: the knock refused, and the read the grants then refused
        // — which is the second half of the claim, because a service that had
        // read the file would have carried that read out.
        assert_eq!(record.len(), 2, "the refusal was not written down");
        let mut written = record.everything();
        assert!(
            matches!(
                written.next().unwrap().happened(),
                alo_record::Happened::GrantsNotReadAgain { .. }
            ),
            "the knock was written down as something else"
        );
        assert!(
            written.next().unwrap().happened().was_stopped(),
            "the read after the knock was not refused"
        );
    }

    /// One read of a folder, as an agent asks for it.
    fn listing(folder: &Path) -> String {
        format!(
            r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
            folder.display()
        )
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
                let refused = second.what_it_said();
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
                let refused = second.what_it_said();
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
        let stopped = while_it_runs_for(
            "",
            Keeping::Forever,
            "no-agent",
            &[Some(Side::Agent)],
            &mut record,
            |told| {
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
            },
        )
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

    /// A record on a disk with a fortnight of afternoons in it, one entry a
    /// day, and where it is.
    ///
    /// Written through `alo-keeping` rather than by hand, so what these tests
    /// shorten is a real record in the format `docs/contracts/record-file.md`
    /// fixes rather than a file that looks like one.
    /// Each entry is put at the **middle** of its day rather than at the start
    /// of it, so no entry sits on the boundary a rule counted in whole days
    /// draws: the service prunes at a real moment a few instructions after this
    /// runs, and an entry exactly seven days old would fall on one side of that
    /// or the other depending on how long the fixture took.
    fn a_fortnight_on_a_disk(what: &str) -> (alo_keeping::Writing, PathBuf) {
        let path = crate::testing::a_directory_of_our_own(what).join("record.jsonl");
        let mut writing = alo_keeping::Writing::opening(&path).unwrap();
        let day = Duration::from_secs(24 * 60 * 60);
        let now = this_moment();
        for days_ago in (0..14_u32).rev() {
            alo_turn::Kept::keep(
                &mut writing,
                alo_record::Entry::answered_here(
                    &alo_capability::Grantee::named("@files"),
                    now - day * days_ago - day / 2,
                ),
            )
            .unwrap();
        }
        (writing, path)
    }

    /// **A machine an organisation set a retention rule on shortens its record,
    /// and it does it before it serves anything.** The one that matters most is
    /// the first: a machine switched off for six months comes back with six
    /// months of a rule to catch up on, and nothing has run yet.
    #[test]
    fn a_machine_under_a_rule_removes_what_it_no_longer_keeps() {
        let (mut writing, path) = a_fortnight_on_a_disk("shortened-while-serving");
        let (served, _invoice) = while_it_runs_under(
            Keeping::for_days(7).unwrap(),
            "shortening",
            &[Some(Side::Person)],
            &mut writing,
            |told| {
                let mut person = Talking::to(&told.at);
                assert!(person.asking(r#"{"waiting":{}}"#).contains("waiting"));
                told.stop.stop();
            },
        )
        .unwrap();

        assert_eq!(
            served.entries_removed(),
            7,
            "the week before last week is what a seven-day rule removes"
        );
        assert_eq!(served.shortenings_refused(), 0);

        let reading = alo_keeping::Reading::at(&path).unwrap();
        assert_eq!(reading.record().len(), 7, "the record on the disk");
        assert!(
            !reading.head().is_whole(),
            "a shortened record does not say where it now starts"
        );
    }

    /// **A machine that keeps everything is not shortened at all**, which is
    /// what one ships with and is the reason the wait still has no timeout on
    /// it. Nothing is removed and nothing is refused.
    #[test]
    fn a_machine_that_keeps_everything_removes_nothing() {
        let (mut writing, path) = a_fortnight_on_a_disk("kept-while-serving");
        let (served, _invoice) = while_it_runs_keeping(
            "keeping-everything",
            &[Some(Side::Person)],
            &mut writing,
            |told| {
                let mut person = Talking::to(&told.at);
                assert!(person.asking(r#"{"waiting":{}}"#).contains("waiting"));
                told.stop.stop();
            },
        )
        .unwrap();

        assert_eq!(served.entries_removed(), 0);
        assert_eq!(served.shortenings_refused(), 0);

        let reading = alo_keeping::Reading::at(&path).unwrap();
        assert_eq!(reading.record().len(), 14);
        assert!(reading.head().is_whole(), "a record nobody shortened");
    }

    /// **A shortening the machine refuses is counted, and the service goes
    /// on.** Nothing was removed in it — `alo-keeping` will not rewrite a record
    /// it cannot read all of — so the machine is keeping more than its rule
    /// rather than less, and stopping over it would take away the one thing
    /// still writing evidence down.
    ///
    /// This is deliberately the opposite answer to a machine that cannot
    /// *write*, which stops: what is missing there is evidence, and what is
    /// missing here is a tidy-up.
    #[test]
    fn a_shortening_the_machine_refuses_is_counted_and_the_service_goes_on() {
        let mut record = ARecordNothingWillShorten::default();
        let (served, invoice) = while_it_runs_under(
            Keeping::for_days(7).unwrap(),
            "refused-shortening",
            &[Some(Side::Agent)],
            &mut record,
            |told| {
                let mut agent = Talking::to(&told.at);
                let read = agent.asking(&format!(
                    r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{}"}}]}}}}"#,
                    told.invoice.display()
                ));
                assert!(
                    read.contains("4180.00"),
                    "the service stopped over a shortening it refused: {read}"
                );
                told.stop.stop();
            },
        )
        .unwrap();

        assert_eq!(served.entries_removed(), 0);
        assert!(served.shortenings_refused() >= 1);
        assert_eq!(served.messages(), 1, "the read was served");
        assert!(invoice.is_file());
        assert_eq!(record.kept.len(), 1, "and it was written down");
    }

    /// **A record that is not on a disk stops being asked**, rather than
    /// leaving a machine under a rule waking up every hour to be told the same
    /// thing. It is the answer a record held only in memory gives, and it costs
    /// exactly one asking.
    #[test]
    fn a_record_that_is_not_on_a_disk_is_asked_once_and_left_alone() {
        let mut record = Record::default();
        let (served, _invoice) = while_it_runs_under(
            Keeping::for_days(7).unwrap(),
            "not-on-a-disk",
            &[Some(Side::Person)],
            &mut record,
            |told| {
                let mut person = Talking::to(&told.at);
                assert!(person.asking(r#"{"waiting":{}}"#).contains("waiting"));
                told.stop.stop();
            },
        )
        .unwrap();

        assert_eq!(served.entries_removed(), 0);
        assert_eq!(served.shortenings_refused(), 0);
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
    /// One request on the port, as another machine puts it: the reply's
    /// status line and body.
    fn on_the_port(port: u16, path: &str, headers: &[(&str, &str)], body: &str) -> (u16, String) {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        stream
            .write_all(alo_nearby::http::a_request_carrying(path, "h", headers, body).as_bytes())
            .unwrap();
        let reply = alo_nearby::http::read_message_of_at_most(&stream, 64 * 1024).unwrap();
        (
            alo_nearby::http::status_of(&reply.first).unwrap(),
            reply.body,
        )
    }

    /// The same, with a `GET` rather than a `POST`.
    fn getting_on_the_port(port: u16, path: &str) -> (u16, String) {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        stream
            .write_all(
                format!("GET {path} HTTP/1.1\r\nhost: h\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                    .as_bytes(),
            )
            .unwrap();
        let reply = alo_nearby::http::read_message(&stream).unwrap();
        (
            alo_nearby::http::status_of(&reply.first).unwrap(),
            reply.body,
        )
    }

    /// A verb on the wire, proven by reception at this moment.
    fn a_proven_verb(
        on_reception: &alo_nearby::Pairing,
        verb: &str,
        given: &[(&str, alo_capability::Given)],
        at: SystemTime,
    ) -> (String, String) {
        let body = alo_corridor::Carried::of(verb, given).said();
        let proof = alo_nearby::Proof::made(on_reception, &reception(), body.as_bytes(), at);
        (proof.said(), body)
    }

    /// A question on the wire, proven by reception at this moment.
    fn a_proven_question(on_reception: &alo_nearby::Pairing, at: SystemTime) -> (String, String) {
        let body = r#"{"model":"m","messages":[{"role":"user","content":"hello"}]}"#.to_owned();
        let proof = alo_nearby::Proof::made(on_reception, &reception(), body.as_bytes(), at);
        (proof.said(), body)
    }

    /// **Every path on the port reaches its door, and any other reaches
    /// nothing.** Each of the six is told apart and answered by the crate
    /// that decides it — with that crate's own word for a message it refuses
    /// — and a seventh is answered *not for this wire* by the daemon itself.
    #[test]
    fn each_path_on_the_port_reaches_its_door_and_any_other_reaches_nothing() {
        let (served, record, _invoice) = while_it_runs("six-paths", &[], |told| {
            let port = told.port;
            // A proposal from a machine discovery here never measured.
            let proposal = alo_nearby::Proposal::checked(
                reception(),
                the_studio(),
                &[alo_nearby::MayAskIts::Models],
                Duration::from_secs(3_600),
                alo_nearby::Keying::fresh().unwrap().offer().clone(),
            )
            .unwrap();
            let (status, said) = on_the_port(
                port,
                alo_nearby::THE_PROPOSAL_PATH,
                &[],
                &format!("{}\n", proposal.said()),
            );
            assert_eq!(status, 400, "{said}");
            assert_eq!(
                alo_nearby::NotProposed::off_the_wire(said.trim()),
                alo_nearby::NotProposed::NotFromWhereItWasFound
            );
            // A confirmation for nothing that is waiting.
            let (status, said) = on_the_port(
                port,
                alo_nearby::THE_CONFIRMATION_PATH,
                &[],
                "not a confirmation\n",
            );
            assert_eq!(status, 400, "{said}");
            assert!(
                matches!(
                    alo_nearby::NotProposed::off_the_wire(said.trim()),
                    alo_nearby::NotProposed::Underneath(_)
                ),
                "{said}"
            );
            // A verb, an outcome and a question with no proof: each reaches a
            // door that judges the proof first and says so.
            for path in [
                alo_corridor::THE_READ_PATH,
                alo_corridor::THE_CHANGE_PATH,
                alo_corridor::THE_OUTCOME_PATH,
                alo_asking::THE_QUESTION_PATH,
            ] {
                let (status, said) = on_the_port(port, path, &[], "{}");
                assert_eq!(
                    status,
                    alo_corridor::AtTheDoor::NoProof.status().0,
                    "{path}: {said}"
                );
                assert_eq!(
                    alo_corridor::AtTheDoor::off_the_wire(said.trim()),
                    Some(alo_corridor::AtTheDoor::NoProof),
                    "{path}"
                );
            }
            // None of the six.
            let (status, said) = on_the_port(port, "/alo-os/1/something-else", &[], "{}");
            assert_eq!(status, 404);
            assert_eq!(said.trim(), crate::hearing::NOT_FOR_THIS_WIRE);
            let (status, _) = getting_on_the_port(port, alo_corridor::THE_READ_PATH);
            assert_eq!(
                status,
                alo_corridor::AtTheDoor::NotForThisWire.status().0,
                "a GET reached a door"
            );
            // Not a message at all.
            let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
            stream.write_all(b"\r\n").unwrap();
            let reply = alo_nearby::http::read_message(&stream).unwrap();
            assert_eq!(alo_nearby::http::status_of(&reply.first).unwrap(), 400);
            assert_eq!(reply.body.trim(), crate::hearing::NOT_A_MESSAGE);
            told.stop.stop();
        });
        assert_eq!(served.heard_on_the_port(), 9);
        assert_eq!(
            served.turns(),
            0,
            "a message on the port began a local turn"
        );
        assert_eq!(
            record.len(),
            0,
            "a refusal before the door was written down"
        );
    }

    /// **The pairings are one, behind one lock: a pairing revoked on the
    /// person's side refuses the next verb and the next question alike**,
    /// with nothing more written. Before the revocation the same verb ran
    /// and the same question was proven.
    #[test]
    fn a_pairing_revoked_on_the_persons_surface_refuses_the_next_verb_and_the_next_question() {
        let mut record = Record::default();
        let (served, _invoice) = while_it_runs_remembering(
            "@files",
            Keeping::Forever,
            "revoked-between-two-paths",
            &[],
            &mut record,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            |folder, now| {
                let mut grants = granting(folder, now);
                grants.grant(
                    alo_capability::Grant::checked(
                        &format!("machine:{}", reception().as_str()),
                        alo_capability::Reach::Folder(folder.to_path_buf()),
                        now,
                        hour(),
                    )
                    .unwrap(),
                );
                grants
            },
            |told| {
                let now = this_moment();
                let (on_reception, on_studio) = paired_between(
                    reception(),
                    the_studio(),
                    &[alo_nearby::MayAskIts::Models],
                    now,
                );
                told.network.locked().pairings_mut().keep(on_studio);
                let folder = told.invoice.parent().unwrap().to_path_buf();
                let listing = [(
                    "folder",
                    alo_capability::Given::text(folder.to_string_lossy().into_owned()),
                )];

                let (proof, body) = a_proven_verb(&on_reception, "list_folder", &listing, now);
                let (status, said) = on_the_port(
                    told.port,
                    alo_corridor::THE_READ_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(status, 200, "{said}");
                assert!(
                    alo_corridor::Answered::read(&said).unwrap().did().is_some(),
                    "{said}"
                );
                let later = now + Duration::from_secs(1);
                let (proof, body) = a_proven_question(&on_reception, later);
                let (status, said) = on_the_port(
                    told.port,
                    alo_asking::THE_QUESTION_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(status, 503, "{said}");
                assert_eq!(said.trim(), crate::questioned::NOT_ANSWERED_HERE);

                // The person here revokes it, on their own side of the lock.
                assert!(told.network.locked().pairings_mut().revoke(&reception()));

                let later = now + Duration::from_secs(2);
                let (proof, body) = a_proven_verb(&on_reception, "list_folder", &listing, later);
                let (_, said) = on_the_port(
                    told.port,
                    alo_corridor::THE_READ_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(
                    alo_corridor::AtTheDoor::off_the_wire(said.trim()),
                    Some(alo_corridor::AtTheDoor::NotProven(
                        alo_nearby::NotProven::NotWithThatMachine
                    )),
                    "{said}"
                );
                let later = now + Duration::from_secs(3);
                let (proof, body) = a_proven_question(&on_reception, later);
                let (_, said) = on_the_port(
                    told.port,
                    alo_asking::THE_QUESTION_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(
                    alo_corridor::AtTheDoor::off_the_wire(said.trim()),
                    Some(alo_corridor::AtTheDoor::NotProven(
                        alo_nearby::NotProven::NotWithThatMachine
                    )),
                    "{said}"
                );
                told.stop.stop();
            },
        )
        .unwrap();
        assert_eq!(served.heard_on_the_port(), 4);
        // The read ran and its answer left; the question and the two
        // refusals after the revocation wrote nothing.
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is(reception().as_str()))
        }));
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(alo_record::Only::Refusals))
                .count(),
            0
        );
    }

    /// A grant to reception's principal over the fixture's folder, beside
    /// `@files`'s.
    fn granting_reception_too(folder: &Path, now: SystemTime) -> Grants {
        let mut grants = granting(folder, now);
        grants.grant(
            alo_capability::Grant::checked(
                &format!("machine:{}", reception().as_str()),
                alo_capability::Reach::Folder(folder.to_path_buf()),
                now,
                hour(),
            )
            .unwrap(),
        );
        grants
    }

    /// **A question from a paired machine, put down the corridor as
    /// reception's own machine puts one, is answered on the port by this
    /// machine's own model** — the model the person here chose, named in the
    /// answer — with reception's indicator firing for the corridor and this
    /// machine's record saying a question was answered for reception and its
    /// answer left.
    #[test]
    fn a_question_from_a_paired_machine_is_answered_on_the_port_by_this_machines_own_model() {
        use alo_answering::Answering;
        use alo_asking::{Asking, DownTheCorridor, Question};
        use alo_capability::Grantee;
        use alo_choosing::{Chosen, Which};
        use alo_models::{InferenceSource, SourcePolicy};

        let mut record = Record::default();
        let (served, _invoice) = while_it_runs_remembering(
            "@files",
            Keeping::Forever,
            "answered-on-the-port",
            &[],
            &mut record,
            &NothingIsRemembered,
            Questions::already_found(
                Chosen::of(Which::Catalogue, "the-studios-model").unwrap(),
                crate::testing::a_runtime_saying(Ok("Three are unpaid.".to_owned())),
                crate::questions::TheBound::Nobodys,
            ),
            granting,
            |told| {
                let now = this_moment();
                let (on_reception, on_studio) = paired_between(
                    reception(),
                    the_studio(),
                    &[alo_nearby::MayAskIts::Models],
                    now,
                );
                told.network.locked().pairings_mut().keep(on_studio);

                // Reception's side, exactly as its own machine has it: its
                // row of the pairing, the corridor made from it, and the
                // question leaving under its own indicator.
                let mut receptions_pairings = alo_nearby::Pairings::none();
                receptions_pairings.keep(on_reception);
                let studio_at = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), told.port);
                let corridor = DownTheCorridor::paired(
                    &receptions_pairings,
                    &reception(),
                    &the_studio(),
                    "the studio machine",
                    studio_at,
                    None,
                    now,
                )
                .unwrap();
                let question = Question::asked("how many are unpaid?", "m").unwrap();
                let files = Grantee::named("@files");
                let permitted =
                    Answering::chosen(corridor.source(), &SourcePolicy::InTheBuilding).unwrap();
                let mut receptions_indicator = Indicator::default();
                let asked = Asking::by(&files, permitted, &[], &SourcePolicy::InTheBuilding)
                    .to_a_paired_machine(
                        &question,
                        &corridor,
                        &mut receptions_indicator,
                        now,
                        &[studio_at],
                    )
                    .unwrap();
                assert_eq!(asked.answer().text(), "Three are unpaid.");
                // What reception's answer names is what reception asked for:
                // `alo_asking::Answer` carries the model as it was named when
                // it was asked and reads nothing off the reply, for the
                // reason its constructor gives. Which model really answered
                // is in the studio's reply and the studio's record, and
                // `crate::questioned`'s tests hold that it is the one the
                // studio's person chose.
                assert_eq!(asked.answer().model(), "m");
                assert_eq!(
                    asked.answer().source(),
                    &InferenceSource::PairedMachine {
                        machine: "the studio machine".to_owned()
                    }
                );
                assert!(
                    !receptions_indicator.is_quiet(),
                    "the question went down the corridor with reception's indicator quiet"
                );
                let answer = asked.ended(&mut receptions_indicator);
                assert!(receptions_indicator.is_quiet());
                assert_eq!(answer.text(), "Three are unpaid.");
                told.stop.stop();
            },
        )
        .unwrap();
        assert_eq!(served.heard_on_the_port(), 1);
        assert_eq!(served.turns(), 0, "a question began a local turn");
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(record.everything().any(|entry| matches!(
            entry.happened(),
            alo_record::Happened::AnsweredForAnotherMachine { .. }
        )));
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(alo_record::Only::Egress))
                .count(),
            1
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is(reception().as_str()))
        }));
    }

    /// **The person's door reaches a remote turn on the running service**: a
    /// change reception's agent proposed on the port is listed for the
    /// person here with reception named, approved from their shell, runs
    /// once, and a second approval finds nothing waiting.
    #[test]
    fn a_change_a_paired_machine_proposed_is_approved_from_the_persons_shell() {
        let mut record = Record::default();
        let (served, invoice) = while_it_runs_remembering(
            "@files",
            Keeping::Forever,
            "approved-from-the-shell",
            &[Some(Side::Person)],
            &mut record,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            granting_reception_too,
            |told| {
                let now = this_moment();
                let (on_reception, on_studio) = paired_between(
                    reception(),
                    the_studio(),
                    &[alo_nearby::MayAskIts::Models],
                    now,
                );
                told.network.locked().pairings_mut().keep(on_studio);
                let (proof, body) = a_proven_verb(
                    &on_reception,
                    "rename_file",
                    &[
                        (
                            "file",
                            alo_capability::Given::text(
                                told.invoice.to_string_lossy().into_owned(),
                            ),
                        ),
                        ("name", alo_capability::Given::text("march-final.pdf")),
                    ],
                    now,
                );
                let (status, said) = on_the_port(
                    told.port,
                    alo_corridor::THE_CHANGE_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(status, 200, "{said}");
                let number = alo_corridor::Answered::read(&said)
                    .unwrap()
                    .waits()
                    .unwrap();
                assert!(told.invoice.is_file(), "a proposal moved a file");

                let mut person = Talking::to(&told.at);
                let said = person.asking(r#"{"waiting":{}}"#);
                let told_back = ToAPerson::read(said.trim_end()).unwrap();
                let changes = told_back.changes().unwrap();
                assert_eq!(changes.len(), 1, "{changes:?}");
                assert_eq!(changes.first().unwrap().number(), number);
                assert_eq!(
                    changes.first().unwrap().from(),
                    Some(reception().as_str()),
                    "{changes:?}"
                );

                let approve = format!(r#"{{"approve":{{"number":{number}}}}}"#);
                let said = person.asking(&approve);
                assert!(
                    ToAPerson::read(said.trim_end()).unwrap().done().is_some(),
                    "{said}"
                );
                assert!(!told.invoice.is_file(), "the file did not move on the disk");
                let again = person.asking(&approve);
                assert!(
                    ToAPerson::read(again.trim_end())
                        .unwrap()
                        .refusal()
                        .is_some(),
                    "{again}"
                );
                assert_eq!(person.what_is_waiting(), Vec::<u64>::new());
                told.stop.stop();
            },
        )
        .unwrap();
        assert!(!invoice.is_file());
        assert_eq!(served.heard_on_the_port(), 1);
        assert_eq!(served.turns(), 0, "a remote change began a local turn");
        assert_eq!(
            record
                .answering(&alo_record::Asking::anything().only(alo_record::Only::Executions))
                .count(),
            1,
            "{record:?}"
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is(reception().as_str()))
        }));
    }

    /// **A pairing kept is written to the record at the one moment there is
    /// one value to write it from, and a proposal refused writes nothing.**
    /// Reception proposes over the wire, the studio's person is shown it and
    /// confirms first, reception confirms second, and the second
    /// confirmation is the one entry — after a proposal from an address
    /// discovery never measured and a second proposal while one waits were
    /// each refused with nothing written.
    #[test]
    fn a_pairing_kept_is_written_down_once_and_a_proposal_refused_writes_nothing() {
        let (served, record, _invoice) =
            while_it_runs("pairing-kept", &[Some(Side::Person)], |told| {
                // A shell is connected, so a proposal has a door to wait on.
                let _person = Talking::to(&told.at);
                let now = this_moment();
                let day = Duration::from_secs(86_400);
                let studio_at = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), told.port);
                let found = alo_nearby::Found::seen(the_studio(), told.port, studio_at.ip());

                // Reception, before it answers discovery: not found, refused.
                let mut receptions_proposals = alo_nearby::Proposals::on(reception());
                let refused = alo_nearby::crossing::propose(
                    &mut receptions_proposals,
                    &found,
                    &[alo_nearby::MayAskIts::Models],
                    day,
                    now,
                )
                .unwrap_err();
                assert_eq!(refused, alo_nearby::NotProposed::NotFromWhereItWasFound);
                assert!(told.network.locked().proposals().every().is_empty());

                // Reception's own wire, where the studio's confirmation arrives.
                let receptions_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
                let receptions_port = receptions_listener.local_addr().unwrap().port();
                // Reception answers discovery when the studio looks for it.
                let answering = alo_nearby::Answering::on(
                    told.discovery,
                    alo_nearby::Presence::of(reception(), receptions_port),
                );
                // Three questions reach it: the one the refused proposal caused,
                // which waited unanswered in the socket, and one for each of the
                // two proposals below.
                let answered = std::thread::spawn(move || {
                    (0..3).map(|_| answering.answer_one()).collect::<Vec<_>>()
                });

                let waiting = alo_nearby::crossing::propose(
                    &mut receptions_proposals,
                    &found,
                    &[alo_nearby::MayAskIts::Models],
                    day,
                    now,
                )
                .unwrap();
                let code_at_reception = waiting.code().unwrap();
                assert_eq!(
                    told.network
                        .locked()
                        .proposals()
                        .with(&reception())
                        .unwrap()
                        .code()
                        .unwrap(),
                    code_at_reception,
                    "the two people were shown different codes"
                );

                // A second proposal while the first waits is refused, and still
                // nothing is written.
                let mut again = alo_nearby::Proposals::on(reception());
                let refused = alo_nearby::crossing::propose(
                    &mut again,
                    &found,
                    &[alo_nearby::MayAskIts::Models],
                    day,
                    now,
                )
                .unwrap_err();
                assert_eq!(refused, alo_nearby::NotProposed::AlreadyWaiting);
                drop(answered.join().unwrap());

                // The studio's person confirms first, on their own side of the
                // lock; reception hears it on its own wire and holds nothing yet.
                let mut receptions_pairings = alo_nearby::Pairings::none();
                let heard_at_reception = std::thread::spawn(move || {
                    let receiving = alo_nearby::Receiving::on(receptions_listener);
                    let arrived = receiving.accept_one().unwrap();
                    let mut shown = |_: &alo_nearby::Waiting| true;
                    let heard = arrived
                        .considered(
                            &mut receptions_proposals,
                            &mut receptions_pairings,
                            &[],
                            &mut shown,
                            this_moment(),
                        )
                        .unwrap();
                    (heard, receptions_proposals, receptions_pairings)
                });
                let kept_at_the_studio = alo_nearby::crossing::confirm(
                    told.network.locked().proposals_mut(),
                    &reception(),
                    this_moment(),
                )
                .unwrap();
                assert!(
                    kept_at_the_studio.is_none(),
                    "one confirmation kept a pairing"
                );
                let (heard, mut receptions_proposals, mut receptions_pairings) =
                    heard_at_reception.join().unwrap();
                assert!(matches!(
                    heard,
                    alo_nearby::Heard::AConfirmation { kept: None, .. }
                ));
                assert!(receptions_pairings.every().is_empty());

                // Reception confirms second, over the wire: the studio keeps the
                // pairing and writes it down; reception keeps its own.
                let kept_at_reception = alo_nearby::crossing::confirm(
                    &mut receptions_proposals,
                    &the_studio(),
                    this_moment(),
                )
                .unwrap()
                .unwrap();
                receptions_pairings.keep(kept_at_reception);
                assert!(
                    told.network
                        .locked()
                        .pairings()
                        .paired_with(&reception(), this_moment())
                );
                told.stop.stop();
            });
        assert_eq!(served.heard_on_the_port(), 4, "{served:?}");
        assert_eq!(record.len(), 1, "{record:?}");
        assert!(
            matches!(
                record.everything().next().unwrap().happened(),
                alo_record::Happened::Paired { with } if with.as_str() == reception().as_str()
            ),
            "{record:?}"
        );
    }

    /// **A local agent that calls itself by a machine's name is refused at
    /// the daemon's door**: no turn is begun for it, nothing is written, and
    /// the service stops saying so rather than holding a turn that a paired
    /// machine's grants would answer.
    #[test]
    fn a_local_agent_called_by_a_machines_name_is_refused_at_the_door() {
        let mut record = Record::default();
        let stopped = while_it_runs_for(
            "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            Keeping::Forever,
            "agent-named-a-machine",
            &[Some(Side::Agent)],
            &mut record,
            |told| {
                // The service refuses before it listens for anybody, so the
                // socket may already be gone; either answer is the refusal.
                drop(UnixStream::connect(&told.at));
                drop(told.stop);
            },
        )
        .unwrap_err();
        assert!(
            matches!(stopped, NotServed::AnAgentNamedAMachine { ref named } if named.starts_with("machine:")),
            "{stopped:?}"
        );
        assert_eq!(record.len(), 0);
    }

    /// Reception, as the machine down the corridor: it answers discovery
    /// once, is shown the studio's proposal on its own wire and answers it,
    /// hears the studio's confirmation, confirms in turn to the studio's port,
    /// and hands back its own row of the pairing.
    fn reception_pairs_with_the_studio(
        told_port: u16,
        discovery: UdpSocket,
    ) -> (u16, std::thread::JoinHandle<alo_nearby::Pairing>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let answering =
            alo_nearby::Answering::on(discovery, alo_nearby::Presence::of(reception(), port));
        let heard = std::thread::spawn(move || {
            answering.answer_one().unwrap();
            let receiving = alo_nearby::Receiving::on(listener);
            let mut proposals = alo_nearby::Proposals::on(reception());
            let mut pairings = alo_nearby::Pairings::none();
            let studio =
                alo_nearby::Found::seen(the_studio(), told_port, Ipv4Addr::LOCALHOST.into());
            let mut shown = |waiting: &alo_nearby::Waiting| waiting.code().is_some();
            let heard = receiving
                .accept_one()
                .unwrap()
                .considered(
                    &mut proposals,
                    &mut pairings,
                    &[studio],
                    &mut shown,
                    this_moment(),
                )
                .unwrap();
            assert!(
                matches!(heard, alo_nearby::Heard::AProposal { .. }),
                "{heard:?}"
            );
            let heard = receiving
                .accept_one()
                .unwrap()
                .considered(
                    &mut proposals,
                    &mut pairings,
                    &[],
                    &mut shown,
                    this_moment(),
                )
                .unwrap();
            assert!(
                matches!(heard, alo_nearby::Heard::AConfirmation { kept: None, .. }),
                "{heard:?}"
            );
            assert!(
                pairings.every().is_empty(),
                "one confirmation kept a pairing"
            );
            let kept = alo_nearby::crossing::confirm(&mut proposals, &the_studio(), this_moment())
                .unwrap()
                .unwrap();
            pairings.keep(kept.clone());
            kept
        });
        (port, heard)
    }

    /// Where a test keeps the pairings file, and the keeper the service
    /// writes it through.
    fn a_pairings_file_of_our_own(what: &str) -> (PathBuf, Box<dyn KeepingPairings>) {
        let at = crate::testing::a_directory_of_our_own(what).join("pairings.toml");
        (at.clone(), Box::new(crate::ThePairingsFile::at(&at)))
    }

    /// One proven verb from reception on the port, and its status and body.
    fn a_verb_from_reception(
        on_reception: &alo_nearby::Pairing,
        port: u16,
        folder: &Path,
        at: SystemTime,
    ) -> (u16, String) {
        let listing = [(
            "folder",
            alo_capability::Given::text(folder.to_string_lossy().into_owned()),
        )];
        let (proof, body) = a_proven_verb(on_reception, "list_folder", &listing, at);
        on_the_port(
            port,
            alo_corridor::THE_READ_PATH,
            &[(alo_asking::THE_PROOF_HEADER, &proof)],
            &body,
        )
    }

    /// **The person's door proposes, confirms and lists a pairing, and the
    /// pairing outlives a restart — and its expiry survives with it.** The
    /// studio's person proposes to reception by identity from their shell;
    /// reception is found on the network, shown, and answers; the shell is
    /// shown the code and confirms with it; reception confirms; the shell
    /// lists it paired, once, and the record says so once. The service is
    /// then stopped and started again from nothing but the file, and a
    /// proven verb from reception is carried out; started a third time after
    /// the moment the pairing ends, the same verb is refused as not paired.
    #[test]
    fn a_pairing_made_on_the_persons_door_outlives_a_restart_and_its_expiry_survives_with_it() {
        let (file, keeping) = a_pairings_file_of_our_own("pairing-door-file");
        let mut record = Record::default();
        let paired_from_the_door = Arc::new(std::sync::Mutex::new(None));
        let handed_back = Arc::clone(&paired_from_the_door);
        let (served, _invoice) = while_it_runs_holding(
            "@files",
            Keeping::Forever,
            "pairing-door",
            &[Some(Side::Person)],
            &mut record,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            (alo_nearby::Pairings::none(), keeping),
            granting,
            move |told| {
                let mut person = Talking::to(&told.at);
                let (_, reception_pairs) =
                    reception_pairs_with_the_studio(told.port, told.discovery);

                let said = person.asking(&format!(
                    r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":6}}}}"#,
                    reception().as_str()
                ));
                let proposed = ToAPerson::read(said.trim_end()).unwrap();
                assert!(
                    proposed.proposed_pairing().is_some(),
                    "the proposal did not come back with the code: {said}"
                );
                let waiting = proposed.proposed_pairing().unwrap();
                let code = waiting.code().unwrap().to_owned();
                assert_eq!(waiting.machine(), reception().as_str());
                assert_eq!(waiting.side(), alo_protocol::SideOf::Asking);
                assert!(!waiting.confirmed().here);

                let said = person.asking(r#"{"pairings":{}}"#);
                let listed = ToAPerson::read(said.trim_end()).unwrap();
                assert!(listed.paired().unwrap().is_empty());
                assert_eq!(listed.waiting_to_pair().unwrap().len(), 1);
                assert_eq!(
                    listed.waiting_to_pair().unwrap().first().unwrap().code(),
                    Some(code.as_str())
                );

                let said = person.asking(&format!(
                    r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
                    reception().as_str()
                ));
                assert_eq!(
                    ToAPerson::read(said.trim_end())
                        .unwrap()
                        .became_of_confirming(),
                    Some(alo_protocol::AfterConfirming::WaitingForTheOtherPerson),
                    "{said}"
                );
                // Reception confirms in turn, to the studio's port.
                let on_reception = reception_pairs.join().unwrap();

                let said = person.asking(r#"{"pairings":{}}"#);
                let listed = ToAPerson::read(said.trim_end()).unwrap();
                assert_eq!(listed.paired().unwrap().len(), 1, "{said}");
                assert_eq!(
                    listed.paired().unwrap().first().unwrap().machine(),
                    reception().as_str()
                );
                assert!(listed.paired().unwrap().first().unwrap().ends_in() <= 6);
                assert!(listed.waiting_to_pair().unwrap().is_empty());
                *handed_back.lock().unwrap() = Some(on_reception);
                told.stop.stop();
            },
        )
        .unwrap();
        assert_eq!(served.heard_on_the_port(), 1, "reception's confirmation");
        assert_eq!(record.len(), 1, "{record:?}");
        assert!(matches!(
            record.everything().next().unwrap().happened(),
            alo_record::Happened::Paired { with } if with.as_str() == reception().as_str()
        ));
        let on_reception = paired_from_the_door.lock().unwrap().take().unwrap();
        let made = this_moment();
        assert_eq!(
            std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600,
            "the key went down where somebody else could read it"
        );

        // The restart: nothing but the file, as `src/main.rs` reads it.
        let read_back = alo_remembering::pairings_remembered(&file, this_moment()).unwrap();
        assert!(read_back.paired_with(&reception(), this_moment()));
        let (_, keeping) = a_pairings_file_of_our_own("pairing-door-file-again");
        let mut record = Record::default();
        let after = on_reception.clone();
        let (served, _invoice) = while_it_runs_holding(
            "@files",
            Keeping::Forever,
            "pairing-door-restarted",
            &[],
            &mut record,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            (read_back, keeping),
            granting_reception_too,
            move |told| {
                let folder = told.invoice.parent().unwrap().to_path_buf();
                let (status, said) =
                    a_verb_from_reception(&after, told.port, &folder, this_moment());
                assert_eq!(
                    status, 200,
                    "the pairing did not survive the restart: {said}"
                );
                told.stop.stop();
            },
        )
        .unwrap();
        assert_eq!(served.heard_on_the_port(), 1);
        assert_eq!(
            record.len(),
            2,
            "the read and its answer leaving: {record:?}"
        );

        // And across the moment it ends: the file still holds the row, and
        // the row is gone as the list is read.
        let until = made + Duration::from_secs(7);
        if let Ok(left) = until.duration_since(this_moment()) {
            std::thread::sleep(left);
        }
        let ended = alo_remembering::pairings_remembered(&file, this_moment()).unwrap();
        assert!(ended.every().is_empty(), "a pairing that ended came back");
        let (_, keeping) = a_pairings_file_of_our_own("pairing-door-file-ended");
        let mut record = Record::default();
        let (served, _invoice) = while_it_runs_holding(
            "@files",
            Keeping::Forever,
            "pairing-door-ended",
            &[],
            &mut record,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            (ended, keeping),
            granting_reception_too,
            move |told| {
                let folder = told.invoice.parent().unwrap().to_path_buf();
                let (_, said) =
                    a_verb_from_reception(&on_reception, told.port, &folder, this_moment());
                assert_eq!(
                    alo_corridor::AtTheDoor::off_the_wire(said.trim()),
                    Some(alo_corridor::AtTheDoor::NotProven(
                        alo_nearby::NotProven::NotWithThatMachine
                    )),
                    "{said}"
                );
                told.stop.stop();
            },
        )
        .unwrap();
        assert_eq!(served.heard_on_the_port(), 1);
        assert_eq!(
            record.len(),
            0,
            "a refusal before the door was written down"
        );
    }

    /// **A revocation from the person's door takes effect on the next verb
    /// and the next question**, tested through the door rather than the
    /// lock: before it the verb ran and the question was proven, after it
    /// both are refused as not paired, and the file no longer holds the row.
    #[test]
    fn a_pairing_revoked_from_the_persons_door_refuses_the_next_verb_and_the_next_question() {
        let (file, keeping) = a_pairings_file_of_our_own("revoked-from-the-door");
        let mut record = Record::default();
        let (on_reception, on_studio) = paired_between(
            reception(),
            the_studio(),
            &[alo_nearby::MayAskIts::Models],
            this_moment(),
        );
        let mut pairings = alo_nearby::Pairings::none();
        pairings.keep(on_studio);
        alo_remembering::pairings_kept(&file, &pairings, this_moment()).unwrap();
        let (served, _invoice) = while_it_runs_holding(
            "@files",
            Keeping::Forever,
            "revoked-from-the-door-service",
            &[Some(Side::Person)],
            &mut record,
            &NothingIsRemembered,
            nothing_has_been_chosen(),
            (pairings, keeping),
            granting_reception_too,
            move |told| {
                let now = this_moment();
                let folder = told.invoice.parent().unwrap().to_path_buf();
                let (status, said) = a_verb_from_reception(&on_reception, told.port, &folder, now);
                assert_eq!(status, 200, "{said}");
                let (proof, body) = a_proven_question(&on_reception, now + Duration::from_secs(1));
                let (status, said) = on_the_port(
                    told.port,
                    alo_asking::THE_QUESTION_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(status, 503, "{said}");
                assert_eq!(said.trim(), crate::questioned::NOT_ANSWERED_HERE);

                // The person here revokes it, from their shell.
                let mut person = Talking::to(&told.at);
                let said = person.asking(&format!(
                    r#"{{"revoke-pairing":{{"machine":"{}"}}}}"#,
                    reception().as_str()
                ));
                assert_eq!(
                    ToAPerson::read(said.trim_end())
                        .unwrap()
                        .became_of_revoking(),
                    Some(alo_protocol::AfterRevoking::Revoked),
                    "{said}"
                );
                let said = person.asking(r#"{"pairings":{}}"#);
                assert!(
                    ToAPerson::read(said.trim_end())
                        .unwrap()
                        .paired()
                        .unwrap()
                        .is_empty()
                );

                let later = now + Duration::from_secs(2);
                let (_, said) = a_verb_from_reception(&on_reception, told.port, &folder, later);
                assert_eq!(
                    alo_corridor::AtTheDoor::off_the_wire(said.trim()),
                    Some(alo_corridor::AtTheDoor::NotProven(
                        alo_nearby::NotProven::NotWithThatMachine
                    )),
                    "{said}"
                );
                let (proof, body) = a_proven_question(&on_reception, now + Duration::from_secs(3));
                let (_, said) = on_the_port(
                    told.port,
                    alo_asking::THE_QUESTION_PATH,
                    &[(alo_asking::THE_PROOF_HEADER, &proof)],
                    &body,
                );
                assert_eq!(
                    alo_corridor::AtTheDoor::off_the_wire(said.trim()),
                    Some(alo_corridor::AtTheDoor::NotProven(
                        alo_nearby::NotProven::NotWithThatMachine
                    )),
                    "{said}"
                );
                told.stop.stop();
            },
        )
        .unwrap();
        assert_eq!(served.heard_on_the_port(), 4);
        assert_eq!(served.messages(), 2);
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(
            alo_remembering::pairings_remembered(&file, this_moment())
                .unwrap()
                .every()
                .is_empty(),
            "a revoked pairing was still in the file"
        );
    }

    /// **The four about a pairing on the agent's door are refused in the
    /// words an agent approving something gets**, nothing waits or is paired
    /// afterwards, and nothing is written down.
    #[test]
    fn the_four_about_pairing_on_the_agents_door_are_refused_as_an_approval_would_be() {
        let (served, record, _invoice) = while_it_runs(
            "agent-pairing",
            &[Some(Side::Agent), Some(Side::Person)],
            |told| {
                let mut agent = Talking::to(&told.at);
                for asks in [
                    format!(
                        r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":3600}}}}"#,
                        reception().as_str()
                    ),
                    format!(
                        r#"{{"confirm-pairing":{{"machine":"{}","code":"000000"}}}}"#,
                        reception().as_str()
                    ),
                    format!(
                        r#"{{"revoke-pairing":{{"machine":"{}"}}}}"#,
                        reception().as_str()
                    ),
                    r#"{"pairings":{}}"#.to_owned(),
                ] {
                    let refused = agent.asking(&asks);
                    assert!(
                        refused
                            .contains("an agent cannot answer a question that was put to a person"),
                        "{refused}"
                    );
                }
                let mut person = Talking::to(&told.at);
                let said = person.asking(r#"{"pairings":{}}"#);
                let listed = ToAPerson::read(said.trim_end()).unwrap();
                assert!(listed.paired().unwrap().is_empty());
                assert!(listed.waiting_to_pair().unwrap().is_empty());
                told.stop.stop();
            },
        );
        assert_eq!(served.messages(), 5);
        assert_eq!(
            record.len(),
            0,
            "an agent's refused request was written down: {record:?}"
        );
    }
}
