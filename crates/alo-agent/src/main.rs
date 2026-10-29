//! **The process that drives one request to its end**, which until now nothing
//! was.
//!
//! `crate::driving` is the loop, `crate::AtTheDaemonsSocket` is the door, and
//! `alo_turn::the_verbs_a_whole_machine_offers` is the list. All three were
//! built, tested, and **called by nothing outside a test** — measured on
//! 2026-10-14 with a control: `alo_agent::driving` appeared in two files
//! outside its own crate, one a test and one a doc comment, against 22
//! manifests naming the crate. `ROADMAP.md`'s first launch item says *a person
//! asks the agent in plain words, and the agent does it*, and nothing on a real
//! machine ran the loop.
//!
//! This is the binary that does. It is the agent's half of
//! `docs/contracts/daemon-protocol.md`, in one process that a person's shell —
//! or a person at a terminal — can start.
//!
//! # Who runs this, which is not a person
//!
//! **Run as a person, every request it makes is refused** — and that is the two
//! doors holding rather than a fault. `alo_agentd::side::Sides::which` decides
//! which door a connection is on from **the uid the kernel recorded**, not from
//! which socket it arrived at:
//!
//! ```text
//! if uid == self.agent.raw()  { return Ok(Side::Agent); }
//! if uid == self.person.raw() { return Ok(Side::Person); }
//! ```
//!
//! So a person running this lands on the person's door, where `read`, `propose`
//! and `ask` are not that side's messages — `docs/contracts/daemon-protocol.md`
//! lists *a request on the wrong door* among its nine refusals. The property
//! survives a person running the agent's own binary by hand, which is stronger
//! than it needed to be.
//!
//! What runs it is `alo-asking@.service` with `User=alo-agent` (ADR 0109), and
//! `sudo -u alo-agent alo-agent …` is the same thing by hand, for whoever is
//! debugging. **The earlier version of this file said *a person runs it*, which
//! is why it is in `bin` rather than `libexec`** — the placement is still right
//! and the reason was wrong: something other than systemd runs it, and that
//! something is the agent's own user.
//!
//! # What it is, and what it is deliberately not
//!
//! **It is a process, not a surface.** It takes a request as words and prints
//! what happened. Joining it to the composer a person types in is a separate
//! change, in `alo-shell`, and this does not pre-empt it: nothing here draws,
//! and nothing here is promised to a person who has not opened a terminal.
//!
//! **It approves nothing.** `crate::TheDoor` is typed to the agent's half of
//! the protocol and the approval lives on the person's door, in another
//! process. A change this drives ends as *a change is waiting* and stops — a
//! person answers it where they answer every other one.
//!
//! **It chooses no model and causes no egress of its own.** Which model answers
//! and whether anything leaves is the daemon's, from the person's settings (ADR
//! 0008), and law 1's indicator is lit by the daemon rather than by this.
//!
//! # Why it has to be told where the socket is
//!
//! The socket is `/run/alo/<uid>/agentd.sock` where `<uid>` is **the person's**
//! login number, and this process runs as the **agent's** — a different user
//! (ADR 0001 §5), which is the whole reason `SO_PEERCRED` can tell the two
//! doors apart. So the agent cannot work the path out from its own identity.
//!
//! It is given `--at`, or it looks: exactly one directory under `/run/alo` that
//! this process can enter and that holds an `agentd.sock`. **Two and it
//! refuses**, by name, rather than picking one — a machine with two signed-in
//! people is a machine where guessing means driving one person's request
//! against the other's grants.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use alo_agent::{AtTheDaemonsSocket, HowFar, WhatHappened, driving_until_stopped};
use alo_instructing::{Instructions, TheConversationSoFar, WhatItMayReach};

/// Where the daemon's doors live, which the image makes at boot.
///
/// `alo_agentd::place::THE_ROOT` is the same path. Spelt again here for the
/// reason `alo-shell` spells the compositor's socket again: this process must
/// not depend on the daemon's crate to learn a filename. Both are held to
/// `docs/contracts/daemon-protocol.md` and never to each other.
const THE_ROOT: &str = "/run/alo";

/// The socket's name, from the same contract.
const THE_SOCKET: &str = "agentd.sock";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(why) => {
            eprintln!("alo-agent: {why}");
            ExitCode::FAILURE
        }
    }
}

/// Drive one request, and say what happened.
fn run() -> Result<ExitCode, String> {
    let asked = what_was_asked()?;
    // **Three ways to know whose daemon to ask, in this order.**
    //
    // `--at` is whoever started this by hand saying so. Then the kernel: when
    // this process was started by `alo-asking.socket` (ADR 0109), its input
    // *is* the person's connection, and `SO_PEERCRED` says which person — not a
    // path this was told and not a directory it guessed. Only when there is
    // neither does it go looking, and that road refuses rather than choosing
    // when it finds two.
    let at = match asked.at {
        Some(at) => at,
        None => match whoever_is_asking() {
            Some(person) => the_door_of(Path::new(THE_ROOT), person),
            None => the_one_door_we_can_reach(Path::new(THE_ROOT))?,
        },
    };

    // The list this machine offers, from the one place that builds it. An agent
    // cannot ask the daemon for it — the protocol has three messages and none
    // is *what may I ask for* — because `docs/contracts/agent-verbs.md` is the
    // answer for whoever writes an adapter, and this function is the answer for
    // anything in this workspace.
    let verbs = alo_turn::the_verbs_a_whole_machine_offers()
        .map_err(|why| format!("the verbs this machine offers could not be declared: {why}"))?;

    let mut door = AtTheDaemonsSocket::opened(&at).map_err(|why| {
        format!(
            "there is no turn to be had at {}: {why}. A machine with no daemon running has no \
             socket there, and a process that is not the agent's user is closed with nothing \
             written on it",
            at.display()
        )
    })?;

    // **The measured instructions**, which is what a caller with no opinion of
    // its own passes: `crate::driving` takes them rather than choosing, because
    // a grade is a statement about a set of them and a loop that chose would be
    // a second place deciding the one thing every grade is about.
    //
    // **And a follow-up is shown what came before it**, inside the question
    // this turn asks the daemon — the only road anything takes off this
    // machine, so the indicator and the record see it (ADR 0138).
    //
    // **And the person may stop it** (ADR 0139): a watcher hears the shell
    // shut its side, and tells the daemon by shutting this side's.
    let stop = match (asked.may_stop, door.a_second_handle(), the_input()) {
        (true, Some(to_the_daemon), Some(input)) => {
            alo_agent::ThePersonMayStop::watching(input, to_the_daemon)
        }
        _ => alo_agent::ThePersonMayStop::cannot(),
    };
    let happened = driving_until_stopped(
        &asked.earlier,
        &asked.request,
        &verbs,
        &asked.may_reach,
        &mut door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
        &|| stop.has_stopped(),
    );
    // Ended before anything else reads the input: holding below reads it.
    let _ = stop.ended();
    let code = said(&happened);
    // **And the turn is held open while the person answers**, because the
    // change waiting is inside it: exiting here ended the turn with the
    // question in it, and the person's shell then found nothing to put on the
    // card. `alo_agent::holding_the_turn` has the whole account.
    if let WhatHappened::AChangeIsWaiting {
        stands_for: Some(stands_for),
        ..
    } = &happened
    {
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let held = alo_agent::held_until_answered(std::io::stdout(), std::io::stdin(), *stands_for);
        eprintln!("alo-agent: the turn was held, and ended: {held:?}");
    }
    // The turn ends here, as the daemon's connection closes.
    drop(door);
    Ok(code)
}

/// How much of one step is worth putting in a service log.
///
/// A verb's answer can be `AStep::THE_MOST_OF_AN_ANSWER` — two thousand
/// bytes — and eight of those is a reason nobody finds. This is a line a
/// person reads, not the record: `alo-record` keeps the whole of it.
const A_LINE_WORTH_READING: usize = 160;

/// What happened, in words, and what the shell should make of it.
///
/// **A change waiting is a success and not a failure**, which is the one of
/// these worth being careful about: the turn did everything an agent may do and
/// stopped where it must. A non-zero exit there would teach whoever wrapped this
/// that a waiting approval is a fault to be retried.
fn said(happened: &WhatHappened) -> ExitCode {
    match happened {
        WhatHappened::ItWasAnsweredInWords { said, after } => {
            println!("{said}");
            if !after.is_empty() {
                eprintln!("alo-agent: after {} step(s)", after.len());
            }
            ExitCode::SUCCESS
        }
        WhatHappened::AChangeIsWaiting {
            number,
            sentence,
            after,
            ..
        } => {
            println!("a change is waiting for you to answer, as number {number}:");
            println!("  {sentence}");
            eprintln!("alo-agent: after {} step(s)", after.len());
            ExitCode::SUCCESS
        }
        WhatHappened::ItWasRefused { said, after } => {
            // **And to the person**, on the output the shell puts in their
            // window. The sentence was worded for them by the daemon, in their
            // language — *this conversation is now longer than the model there
            // can read* among them (ADR 0138) — and until 2026-10-28 it went
            // only to the service log, so the window could say no more than
            // that the request stopped.
            println!("{said}");
            eprintln!("alo-agent: refused after {} step(s): {said}", after.len());
            ExitCode::FAILURE
        }
        // **Nothing is said to the person**: their shell asked for the Stop,
        // and what it reports is this process going, which is the one thing
        // that shows the turn really ended (ADR 0139).
        WhatHappened::ItWasStopped { after } => {
            eprintln!(
                "alo-agent: the person stopped this after {} step(s)",
                after.len()
            );
            ExitCode::SUCCESS
        }
        WhatHappened::ItRanOutOfRounds { after } => {
            eprintln!(
                "alo-agent: it got nowhere in {} step(s), so it stopped rather than going round \
                 again",
                after.len()
            );
            // **And what it tried, which this said nothing about.** Measured on
            // a booted machine on 2026-10-24: a real question reached a real
            // provider, eight rounds ran, and the only thing the machine said
            // was the sentence above. The reason each round failed was built in
            // `crate::driving` — `not a request: …` and the error — pushed onto
            // the steps, and then reduced to a count.
            //
            // **The machine's side of each step, never the model's.** `asked`
            // carries the person's words and whatever a verb read for them;
            // this is what the daemon answered, which is a diagnosis.
            // `AStep::what_came_back` carries the same argument.
            //
            // Cut at `A_LINE_WORTH_READING`, because a verb's answer can be
            // two thousand bytes and eight of those in a service log is a
            // reason nobody finds.
            for (which, step) in after.iter().enumerate() {
                let line = step.what_came_back();
                let shown = line
                    .char_indices()
                    .nth(A_LINE_WORTH_READING)
                    .map_or(line, |(at, _)| &line[..at]);
                let cut = if shown.len() < line.len() { " …" } else { "" };
                eprintln!("alo-agent:   step {}: {shown}{cut}", which + 1);
            }
            ExitCode::FAILURE
        }
        WhatHappened::TheMachineCouldNotBeAsked { why, after } => {
            eprintln!(
                "alo-agent: this machine could not be asked after {} step(s): {why}",
                after.len()
            );
            ExitCode::FAILURE
        }
    }
}

/// What this process was told to do.
struct Asked {
    /// The request, in the person's own words.
    request: String,
    /// Where the daemon's socket is, when it was given.
    at: Option<PathBuf>,
    /// The folders the asking side says this person has granted.
    ///
    /// **A hint for the model, never a capability**, which is what makes it
    /// safe for the asking side to set — unlike *who is asking*, which the
    /// kernel answers and a request may not carry. Nothing here permits
    /// anything: every verb is still decided by `alo-agentd` against the
    /// grants it reads for itself. An asking side that named a folder nobody
    /// granted would only send the model to a path the daemon refuses, which
    /// is the behaviour this change exists to stop rather than a way round it.
    ///
    /// Empty where nothing said, which is every caller that predates this —
    /// `alo-ask` writes one line and still does.
    may_reach: WhatItMayReach,
    /// What was said before, when this is a follow-up typed in a task window
    /// (ADR 0138); nothing for a first request and for a request typed here.
    earlier: TheConversationSoFar,
    /// Whether the person can stop this turn from their shell (ADR 0139).
    may_stop: bool,
}

/// Read the arguments.
///
/// # Errors
/// A sentence, when there is no request to drive — which is the one thing this
/// cannot invent.
fn what_was_asked() -> Result<Asked, String> {
    let mut at = None;
    let mut words: Vec<String> = Vec::new();
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--at" => {
                at = Some(PathBuf::from(arguments.next().ok_or(
                    "--at wants the path of the daemon's socket, and nothing followed it",
                )?));
            }
            "--help" | "-h" => {
                println!(
                    "alo-agent: drive one request to its end.\n\n  alo-agent <request…>\n  \
                     alo-agent --at /run/alo/1000/agentd.sock <request…>\n\nWith no request on \
                     the command line, one line is read from the input.\n\nRun this as the \
                     AGENT, not as yourself. The daemon decides which of its two doors a \
                     connection is on from the uid the kernel recorded, so run by a person it \
                     lands on the person's door and every request is refused as being on the \
                     wrong one. That is the two doors holding, not a fault.\n\n  sudo -u \
                     alo-agent alo-agent <request…>\n\nOrdinarily nothing types this: \
                     alo-asking.socket hands a person's words to one of these, running as the \
                     agent (ADR 0109)."
                );
                std::process::exit(0);
            }
            _ => words.push(argument),
        }
    }
    if !words.is_empty() {
        let request = words.join(" ").trim().to_owned();
        if request.is_empty() {
            return Err(
                "there is no request to drive. Say what to do, or pipe one line in; `--help` \
                 shows how"
                    .to_owned(),
            );
        }
        return Ok(Asked {
            request,
            at,
            may_reach: WhatItMayReach::nothing(),
            earlier: TheConversationSoFar::nothing(),
            may_stop: false,
        });
    }
    // **What the shell sent, from the input**: the folders, the request, and
    // for a follow-up the conversation so far. `alo_agent::what_the_person_sent`
    // holds the order and why an older caller still works.
    let stdin = std::io::stdin();
    let socket = std::os::fd::AsFd::as_fd(&stdin);
    let sent = alo_agent::read_what_the_person_sent(&mut stdin.lock(), Some(socket))?;
    Ok(Asked {
        request: sent.request,
        at,
        may_reach: sent.may_reach,
        earlier: sent.earlier,
        may_stop: sent.may_stop,
    })
}

/// A second handle on this process's input, for the watcher that hears a Stop
/// (ADR 0139). [`None`] when it cannot be had, which leaves a turn nobody can
/// stop rather than refusing one.
fn the_input() -> Option<std::os::fd::OwnedFd> {
    std::os::fd::AsFd::as_fd(&std::io::stdin())
        .try_clone_to_owned()
        .ok()
}

/// The person at the other end of this process's input, when there is one.
///
/// **The question ADR 0109 turns on, and the kernel answers it.** Started by
/// `alo-asking.socket` with `Accept=yes`, this process's input is the
/// connection a person's shell opened — so `SO_PEERCRED` says which person
/// asked, as the kernel recorded them when they called `connect`. It is not a
/// field in the request and the asking side cannot set it.
///
/// That matters because a machine can have two signed-in people and the request
/// must run against the **asking** person's grants. Getting it wrong is the one
/// mistake here nobody could see from the outcome: the turn would succeed, the
/// record would be the other person's, and the answer would look right.
///
/// [`None`] when the input is not a connected socket — somebody at a terminal,
/// or a pipe — which is not a failure and is the ordinary case for
/// `alo-agent <request…>`.
fn whoever_is_asking() -> Option<u32> {
    use std::os::fd::AsFd as _;

    rustix::net::sockopt::socket_peercred(std::io::stdin().as_fd())
        .ok()
        .map(|peer| peer.uid.as_raw())
}

/// This person's door, by the contract's own spelling.
fn the_door_of(root: &Path, person: u32) -> PathBuf {
    root.join(person.to_string()).join(THE_SOCKET)
}

/// The one door under `root` this process can reach.
///
/// **Two is a refusal and not a choice.** A machine with two signed-in people
/// has two doors, and picking one would drive this request against the other
/// person's grants — the one mistake here that a person could not detect from
/// the outcome. So it says both and stops.
///
/// # Errors
/// A sentence naming what it found: nothing, or more than one.
fn the_one_door_we_can_reach(root: &Path) -> Result<PathBuf, String> {
    let mut found = Vec::new();
    let reading = std::fs::read_dir(root).map_err(|why| {
        format!(
            "{} could not be read: {why}. The image makes it at boot, so a machine without it is \
             a machine something is wrong with rather than one nobody has signed in to",
            root.display()
        )
    })?;
    for entry in reading.flatten() {
        let socket = entry.path().join(THE_SOCKET);
        // `exists` follows a link and answers false for a path this process
        // may not look at, which is exactly the question being asked: can this
        // process reach a door there.
        if socket.exists() {
            found.push(socket);
        }
    }
    found.sort();
    match found.as_slice() {
        [] => Err(format!(
            "no daemon is serving an agent at {}. One is there while a person is signed in, and \
             only a process in the agent's group can see it",
            root.display()
        )),
        [one] => Ok(one.clone()),
        several => Err(format!(
            "there are {} doors under {} and this will not pick one: {}. Name it with `--at` — \
             driving a request against the wrong person's grants is the one mistake here that \
             nobody could see from the outcome",
            several.len(),
            root.display(),
            several
                .iter()
                .map(|at| at.display().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )),
    }
}

#[cfg(test)]
// Counted: 7 `unwrap` and no `expect`, `panic!` or `unreachable!`.
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::{THE_SOCKET, the_one_door_we_can_reach};
    use std::path::PathBuf;

    /// A root of this test's own, holding a door for each of these people.
    ///
    /// A real file rather than a socket: `the_one_door_we_can_reach` asks
    /// whether this process can **see** something at that path, which is the
    /// question a door being there answers, and binding real sockets would add
    /// a dependency on the path length a kernel allows for nothing.
    ///
    /// **Named from `what`, not from the number of people.** The first version
    /// keyed it on `people.len()`, so the two tests that pass one person got
    /// the **same** directory and interfered — one of them failed in the whole
    /// suite and passed when run alone, which is the *a suite green on one
    /// machine* fault in miniature. `alo-installed`'s own fixture carries a
    /// comment about making exactly this mistake; I made it again anyway, which
    /// is why the comment is worth keeping in both places.
    fn a_root_with_doors_for(what: &str, people: &[u32]) -> PathBuf {
        let at =
            std::env::temp_dir().join(format!("alo-agent-doors-{}-{what}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        for person in people {
            let theirs = at.join(person.to_string());
            std::fs::create_dir_all(&theirs).unwrap();
            std::fs::write(theirs.join(THE_SOCKET), b"").unwrap();
        }
        at
    }

    /// **One door is the one it uses.**
    #[test]
    fn the_one_door_there_is_is_the_one_it_takes() {
        let root = a_root_with_doors_for("just-one", &[1000]);
        let found = the_one_door_we_can_reach(&root).unwrap();
        assert_eq!(found, root.join("1000").join(THE_SOCKET));
        let _ = std::fs::remove_dir_all(root);
    }

    /// **No door is a sentence about a daemon, not about this process.** A
    /// machine with nobody signed in has no socket there, and that is ordinary.
    #[test]
    fn no_door_says_no_daemon_is_serving_an_agent() {
        let root = a_root_with_doors_for("none-at-all", &[]);
        let why = the_one_door_we_can_reach(&root).unwrap_err();
        assert!(why.contains("no daemon is serving an agent"), "{why}");
        let _ = std::fs::remove_dir_all(root);
    }

    /// **Two doors is a refusal and never a choice**, which is the assertion
    /// this file exists to make.
    ///
    /// A machine with two signed-in people has two doors. Picking one would
    /// drive the request against the other person's grants — and that is the
    /// one mistake here **nobody could detect from the outcome**, because the
    /// turn would succeed, the record would be the other person's, and the
    /// answer would look right. So it names both and stops.
    #[test]
    fn two_doors_is_refused_by_name_rather_than_chosen_between() {
        let root = a_root_with_doors_for("two-people", &[1000, 1001]);
        let why = the_one_door_we_can_reach(&root).unwrap_err();
        assert!(why.contains("will not pick one"), "{why}");
        // **Both are named**, so whoever reads it can pass `--at` without
        // going looking.
        assert!(why.contains("1000"), "{why}");
        assert!(why.contains("1001"), "{why}");
        let _ = std::fs::remove_dir_all(root);
    }

    /// A directory with no socket in it is not a door.
    ///
    /// The daemon takes its socket away when a session ends and leaves the
    /// directory if anything else is in it, so an empty one is a person who has
    /// signed out — not a door to knock at.
    #[test]
    fn a_directory_with_no_socket_in_it_is_not_a_door() {
        let root = a_root_with_doors_for("one-signed-out", &[1000]);
        std::fs::create_dir_all(root.join("1001")).unwrap();
        let found = the_one_door_we_can_reach(&root).unwrap();
        assert_eq!(found, root.join("1000").join(THE_SOCKET));
        let _ = std::fs::remove_dir_all(root);
    }

    /// A root the image never made is said to be the machine's fault, not a
    /// machine nobody has signed in to.
    #[test]
    fn a_missing_root_is_the_machines_fault_and_says_so() {
        let nowhere =
            std::env::temp_dir().join(format!("alo-agent-no-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&nowhere);
        let why = the_one_door_we_can_reach(&nowhere).unwrap_err();
        assert!(why.contains("something is wrong with"), "{why}");
    }
}
