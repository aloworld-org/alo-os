//! **What an application sends while a person's command runs**, shown on the
//! indicator this desktop draws
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! A person's command (`alo-app`) asks the compositor's door to `count` one
//! application and keeps that connection open while it works. The door hands
//! the connection here ([`AskedToCount`]), and from then on, once a frame:
//!
//! 1. the application's control groups are found again, so one that started
//!    during the command is counted from the moment its unit appears;
//! 2. what they have sent since this command began counting them is added up;
//! 3. **from the first byte, the indicator says so** — *Thunderbird is sending,
//!    during your command* — and keeps saying it until the command ends;
//! 4. when the command says `done`, it is answered with the total and the count
//!    ends; when it hangs up, the count ends with nothing said; and **after
//!    [`THE_LONGEST`] the desktop ends it whatever the command does**, because a
//!    count held open for ever would be the watching ADR 0148 exists to prevent.
//!
//! A line that would come off the indicator less than [`SEEN_FOR_AT_LEAST`]
//! after it went up stays until then, so a short burst is still something a
//! person can see.
//!
//! # Nothing here waits
//!
//! This runs in the compositor's frame loop, like the door. Every connection is
//! non-blocking, and a command that writes nothing costs a read that would
//! block, once a frame.
//!
//! # What it cannot see
//!
//! What another process sends for the application, and anything sent before a
//! newly started unit is found — ADR 0148 §5 says both where a person reads.

use std::collections::BTreeMap;
use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant, SystemTime};

use alo_acting::{Begun, Counted, ToCount, counting::DONE};
use alo_egress::{DuringTheirCommand, Indicator, WhileItRuns};

/// The longest a count is held, whatever the command does.
pub const THE_LONGEST: Duration = Duration::from_secs(120);

/// The least time a line is on the indicator, once it is there.
pub const SEEN_FOR_AT_LEAST: Duration = Duration::from_secs(2);

/// How many commands may be counting at once.
///
/// With [`THE_MOST_GROUPS`] each, the counter's sixty-four entries
/// (`alo_bounding_map::THE_MOST_COUNTED`) are enough for all of them.
pub const THE_MOST_AT_ONCE: usize = 4;

/// How many control groups one command may count.
pub const THE_MOST_GROUPS: usize = 16;

/// The longest line a command may say on a count's connection.
const LONGEST_LINE: usize = 64;

/// The kernel's counter, as the desktop reaches it.
///
/// A trait so this file is tested without a kernel; the desktop's own is
/// `alo_bounding::Counter`. Every answer is a fact or a failure, and a failure
/// is never read as *nothing was sent*.
pub trait CountsWhatIsSent {
    /// Begin counting `group`, or leave a count already running as it is.
    /// `false` if the kernel would not.
    fn begun(&mut self, group: u64) -> bool;
    /// What `group` has sent since its count began, or [`None`] if it is not
    /// counted or could not be read.
    fn so_far(&self, group: u64) -> Option<u64>;
    /// Stop counting `group`.
    fn ended(&mut self, group: u64);
}

/// Where an application runs, as this machine's control groups.
pub trait FindsWhereItRuns {
    /// The control groups of the application with this identifier, by the
    /// kernel's number for each.
    fn groups_of(&self, application: &str) -> Vec<u64>;
}

/// A connection the door was asked to count on, handed over whole.
#[derive(Debug)]
pub struct AskedToCount {
    /// What was asked.
    asked: ToCount,
    /// The connection, not blocking.
    talking: UnixStream,
    /// Whatever the command said after its first line, which is the start of
    /// its next.
    heard: Vec<u8>,
}

impl AskedToCount {
    /// What the door read, the connection, and whatever followed the line.
    #[must_use]
    pub fn of(asked: ToCount, talking: UnixStream, heard: Vec<u8>) -> Self {
        Self {
            asked,
            talking,
            heard,
        }
    }

    /// The application the command asked to count.
    #[must_use]
    pub fn application(&self) -> &str {
        self.asked.application().identifier()
    }

    /// Answer *cannot count* and let the connection go — what a desktop with
    /// no indicator says, because a count nobody can see is not what the
    /// command asked for.
    pub fn refused(mut self) {
        say(&mut self.talking, Begun::CannotCount.written());
    }
}

/// One command, counting one application.
struct OneCommand {
    /// The command's connection.
    talking: UnixStream,
    /// What it has said so far of its next line.
    heard: Vec<u8>,
    /// The application, and what a person calls it.
    during: DuringTheirCommand,
    /// When the count began.
    began: Instant,
    /// Each control group counted, with what it had sent when this command
    /// began counting it — another command may have been counting it already.
    groups: BTreeMap<u64, u64>,
    /// Whether a control group of the application could not be counted, so
    /// the total is less than everything it sent.
    missed: bool,
    /// The line on the indicator, and when it went up.
    line: Option<(WhileItRuns, Instant)>,
}

/// A line that has stayed on past its command, until it has been seen.
struct Lingering {
    /// The line.
    line: WhileItRuns,
    /// When it may come off.
    until: Instant,
}

/// How a command's connection stands this frame.
enum Said {
    /// Nothing new.
    Nothing,
    /// `done`.
    Done,
    /// It hung up, or said something that is not `done`.
    Gone,
}

/// Every count being held for a person's commands.
pub struct CountingForCommands {
    /// The kernel's counter, if this machine has one.
    counter: Option<Box<dyn CountsWhatIsSent>>,
    /// How an application's control groups are found.
    finding: Box<dyn FindsWhereItRuns>,
    /// The commands counting now.
    commands: Vec<OneCommand>,
    /// Lines staying on until they have been seen.
    lingering: Vec<Lingering>,
    /// How many commands count each control group, so its entry ends with the
    /// last of them.
    in_use: BTreeMap<u64, usize>,
}

impl CountingForCommands {
    /// Counting with this counter — [`None`] on a machine whose boundary was
    /// loaded without one — and this way of finding an application.
    #[must_use]
    pub fn with(
        counter: Option<Box<dyn CountsWhatIsSent>>,
        finding: Box<dyn FindsWhereItRuns>,
    ) -> Self {
        Self {
            counter,
            finding,
            commands: Vec::new(),
            lingering: Vec::new(),
            in_use: BTreeMap::new(),
        }
    }

    /// How many commands are counting now.
    #[must_use]
    pub fn how_many_are_counting(&self) -> usize {
        self.commands.len()
    }

    /// Take a count the door was asked for, and answer whether it has begun.
    ///
    /// `called` is what a person calls the application, or [`None`] when it is
    /// not installed — which is answered `not-installed`, because there is
    /// nothing a count of it could mean.
    pub fn taken_in(&mut self, asked: AskedToCount, called: Option<&str>, now: Instant) {
        let AskedToCount {
            asked,
            mut talking,
            heard,
        } = asked;
        let Some(called) = called else {
            say(&mut talking, Begun::NotInstalled.written());
            return;
        };
        let during = DuringTheirCommand::of(asked.application().identifier(), called);
        let (Some(_), Ok(during)) = (self.counter.as_ref(), during) else {
            say(&mut talking, Begun::CannotCount.written());
            return;
        };
        if self.commands.len() >= THE_MOST_AT_ONCE {
            say(&mut talking, Begun::CannotCount.written());
            return;
        }
        let mut command = OneCommand {
            talking,
            heard,
            during,
            began: now,
            groups: BTreeMap::new(),
            missed: false,
            line: None,
        };
        self.joined(&mut command);
        if command.missed && command.groups.is_empty() {
            // Not one part of it could be counted, so a count would be a
            // number that means nothing.
            say(&mut command.talking, Begun::CannotCount.written());
            return;
        }
        if say(&mut command.talking, Begun::Counting.written()) {
            self.commands.push(command);
        } else {
            self.released(&command);
        }
    }

    /// Look at every count once: find new control groups, put a line up for
    /// any application seen sending, answer commands that are done, and end
    /// what has finished or run too long.
    pub fn looked_at(&mut self, indicator: &mut Indicator, now: Instant, wall: SystemTime) {
        let mut still = Vec::with_capacity(self.commands.len());
        for mut command in std::mem::take(&mut self.commands) {
            self.joined(&mut command);
            let sent = self.sent_by(&command);
            if sent > 0 && command.line.is_none() {
                let line = indicator.beginning_during_their_command(command.during.clone(), wall);
                command.line = Some((line, now));
            }
            let too_long = now.saturating_duration_since(command.began) >= THE_LONGEST;
            match heard_from(&mut command.talking, &mut command.heard) {
                Said::Done => {
                    let counted = if command.missed {
                        Counted::Stopped(sent)
                    } else {
                        Counted::Sent(sent)
                    };
                    say(&mut command.talking, &counted.written());
                    self.finished(command, indicator, now);
                }
                Said::Gone => self.finished(command, indicator, now),
                Said::Nothing if too_long => {
                    say(&mut command.talking, &Counted::Stopped(sent).written());
                    self.finished(command, indicator, now);
                }
                Said::Nothing => still.push(command),
            }
        }
        self.commands = still;
        let (over, staying): (Vec<_>, Vec<_>) = std::mem::take(&mut self.lingering)
            .into_iter()
            .partition(|lingering| now >= lingering.until);
        self.lingering = staying;
        for lingering in over {
            indicator.ended_during_their_command(lingering.line);
        }
    }

    /// Every control group of the command's application that is not yet
    /// counted, counted from now.
    fn joined(&mut self, command: &mut OneCommand) {
        let Some(counter) = self.counter.as_mut() else {
            command.missed = true;
            return;
        };
        for group in self.finding.groups_of(command.during.application()) {
            if command.groups.contains_key(&group) {
                continue;
            }
            if command.groups.len() >= THE_MOST_GROUPS || !counter.begun(group) {
                command.missed = true;
                continue;
            }
            let from = counter.so_far(group).unwrap_or(0);
            command.groups.insert(group, from);
            *self.in_use.entry(group).or_insert(0) += 1;
        }
    }

    /// What the command's application has sent since this command began
    /// counting each part of it.
    fn sent_by(&self, command: &OneCommand) -> u64 {
        let Some(counter) = self.counter.as_ref() else {
            return 0;
        };
        command
            .groups
            .iter()
            .map(|(group, from)| counter.so_far(*group).unwrap_or(*from).saturating_sub(*from))
            .fold(0_u64, u64::saturating_add)
    }

    /// The command is over: its control groups are let go of, and its line
    /// comes off — or stays until it has been seen.
    fn finished(&mut self, command: OneCommand, indicator: &mut Indicator, now: Instant) {
        self.released(&command);
        if let Some((line, up)) = command.line {
            let until = up + SEEN_FOR_AT_LEAST;
            if now >= until {
                indicator.ended_during_their_command(line);
            } else {
                self.lingering.push(Lingering { line, until });
            }
        }
    }

    /// Let go of the command's control groups, ending each count nobody else
    /// is holding.
    fn released(&mut self, command: &OneCommand) {
        for group in command.groups.keys() {
            let last = match self.in_use.get_mut(group) {
                Some(holding) if *holding > 1 => {
                    *holding -= 1;
                    false
                }
                _ => true,
            };
            if last {
                self.in_use.remove(group);
                if let Some(counter) = self.counter.as_mut() {
                    counter.ended(*group);
                }
            }
        }
    }
}

/// Say one line on a count's connection; `false` if it could not be said.
fn say(talking: &mut UnixStream, line: &str) -> bool {
    talking.write_all(line.as_bytes()).is_ok()
        && talking.write_all(b"\n").is_ok()
        && talking.flush().is_ok()
}

/// Whatever the command has said since the last look.
fn heard_from(talking: &mut UnixStream, heard: &mut Vec<u8>) -> Said {
    let mut arrived = [0_u8; LONGEST_LINE];
    loop {
        if let Some(end) = heard.iter().position(|byte| *byte == b'\n') {
            return match heard.get(..end) {
                Some(line) if line == DONE.as_bytes() => Said::Done,
                _ => Said::Gone,
            };
        }
        if heard.len() > LONGEST_LINE {
            return Said::Gone;
        }
        match talking.read(&mut arrived) {
            Ok(0) => return Said::Gone,
            Ok(read) => match arrived.get(..read) {
                Some(said) => heard.extend_from_slice(said),
                None => return Said::Gone,
            },
            Err(why) if why.kind() == ErrorKind::WouldBlock => return Said::Nothing,
            Err(why) if why.kind() == ErrorKind::Interrupted => {}
            Err(_) => return Said::Gone,
        }
    }
}

#[cfg(test)]
#[path = "counting_during_their_commands_tests.rs"]
mod tests;
