//! **What an application sends while alo works in it**, shown on the indicator
//! this desktop draws
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6 and the owner's answers of 2026-11-10; the count is
//! [ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)'s,
//! of one copy rather than of every window).
//!
//! alo asks the compositor's door to work in an application and holds the
//! connection while it does (`alo_acting::working_in`). The door hands the
//! connection here ([`AskedToWork`](crate::alo_at_work::AskedToWork)), and from then on, once a frame:
//!
//! - **at the default — alo's own copy** — the copy is started, or found
//!   running, by the desktop ([`StartsAlosCopy`](crate::alo_at_work::StartsAlosCopy)); its control groups, and only
//!   its, are counted; and **from the first byte the indicator says so**. A
//!   machine with no counter, a copy whose groups cannot be found within
//!   [`FOUND_WITHIN`](crate::alo_at_work::FOUND_WITHIN), or one part of it that cannot be counted, puts the line
//!   *this machine cannot count what it sends* up at once instead, for the whole
//!   time: **never silence** (law 1);
//! - **turned up — the person's own window** — the line *may be sending, and
//!   alo cannot count what* is up for the whole time, and **nothing is counted**,
//!   because counting the person's window would be watching them;
//! - an application that cannot open a copy of its own is refused —
//!   `no-copy-of-its-own`, the owner's answer A — and the turn says which
//!   setting lets alo work in the person's window instead, in
//!   `alo_capability::words::NO_COPY_OF_ITS_OWN`.
//!
//! **Which setting it is, is the person's, read from their own file at each
//! ask** (`alo_capability::AloInApplications`, chosen in Settings → Folders
//! alo can use) and handed to
//! [`taken_in`](crate::alo_at_work::WorkingInApplications::taken_in) — never
//! the word alo sent. At the default, alo asking for the person's window is
//! answered `not-chosen`. Turned up, the person's window is allowed, and an
//! application with no copy of its own is worked in there rather than refused.
//!
//! When alo says `done` it is answered with what the copy sent; when it hangs
//! up the line comes off; and after [`THE_LONGEST`](crate::alo_at_work::THE_LONGEST) the desktop ends it,
//! because a line can only be trusted while somebody is still holding it. A
//! line stays up at least [`SEEN_FOR_AT_LEAST`](crate::alo_at_work::SEEN_FOR_AT_LEAST).
//!
//! **alo's copies are not the person's application**: their groups are handed
//! to [`CountingForCommands`](crate::counting_during_their_commands::CountingForCommands) so that a person's own command never counts them.
//!
//! # Nothing calls this yet
//!
//! **No turn drives an application until stage 4 of the Demo 2 plan**, which
//! offers the accessibility verbs. Until then nothing asks the door to work in
//! an application, and a person can see none of this. It is built first so
//! that nothing that publishes is ever driven before what it sends is shown.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant, SystemTime};

use alo_acting::{Opened, ToWorkIn, Where, Worked, Working, counting::DONE};
use alo_capability::AloInApplications;
use alo_egress::{AloAtWork, HowItIsShown, Indicator, WhileAloWorks};

use crate::counting_during_their_commands::{CountingForCommands, THE_MOST_GROUPS};

/// The longest alo may work in an application on one connection; a turn that
/// works longer asks again.
pub const THE_LONGEST: Duration = Duration::from_secs(15 * 60);

/// The least time a line is on the indicator, once it is there.
pub const SEEN_FOR_AT_LEAST: Duration = Duration::from_secs(2);

/// How long alo's copy may take to be found before the line says this machine
/// cannot count it.
pub const FOUND_WITHIN: Duration = Duration::from_secs(2);

/// How many applications alo may be working in at once.
pub const THE_MOST_AT_ONCE: usize = 4;

/// The longest line alo may say on the connection.
const LONGEST_LINE: usize = 64;

/// Why alo's copy of an application is not running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoCopy {
    /// The application cannot open a separate copy for alo (answer A).
    OfItsOwn,
    /// It can, and starting it failed.
    CouldNotStart,
}

/// alo's own copies of applications, as the desktop starts and finds them.
///
/// A trait so this file is tested without a session; the desktop's own is
/// `alo-desktop`'s `AlosCopies`.
pub trait StartsAlosCopy {
    /// Start alo's copy of the application with this identifier, or find the
    /// one already running.
    ///
    /// # Errors
    /// [`NoCopy`] when there is no copy and none can be started.
    fn started(&mut self, application: &str) -> Result<(), NoCopy>;

    /// The control groups alo's copy of this application runs in now, by the
    /// kernel's number for each — **never a group of the person's own**.
    fn groups_of(&mut self, application: &str) -> Vec<u64>;

    /// Every control group any of alo's copies runs in now.
    fn every_copys_groups(&mut self) -> BTreeSet<u64>;

    /// Whether the process `pid` runs in alo's copy of this application —
    /// the one question typing into a window asks (ADR 0153). **No** unless
    /// the copies can say yes.
    fn runs_as_alos_copy(&mut self, _application: &str, _pid: u32) -> bool {
        false
    }

    /// Open `file` in alo's copy of this application — starting the copy with
    /// it, or handing it to the copy already running — and say where it opened
    /// or why it did not (`open_file_in_application`, 2026-11-14).
    ///
    /// **Could not start by default**, so copies that cannot hand a file over
    /// open nothing and say so.
    fn started_with_a_file(&mut self, _application: &str, _file: &std::path::Path) -> Opened {
        Opened::CouldNotStart
    }
}

/// A connection the door was asked to work on, handed over whole.
#[derive(Debug)]
pub struct AskedToWork {
    /// What was asked.
    asked: ToWorkIn,
    /// The connection, not blocking.
    talking: UnixStream,
    /// Whatever followed the first line.
    heard: Vec<u8>,
}

impl AskedToWork {
    /// What the door read, the connection, and whatever followed the line.
    #[must_use]
    pub fn of(asked: ToWorkIn, talking: UnixStream, heard: Vec<u8>) -> Self {
        Self {
            asked,
            talking,
            heard,
        }
    }

    /// The application alo asked to work in.
    #[must_use]
    pub fn application(&self) -> &str {
        self.asked.application().identifier()
    }

    /// Answer *cannot show* and let the connection go — what a desktop with no
    /// indicator says, because alo may not work where nothing can be shown.
    pub fn refused(mut self) {
        say(&mut self.talking, Working::CannotShow.written());
    }
}

/// alo, working in one application.
struct OneWork {
    /// The connection.
    talking: UnixStream,
    /// What it has said so far of its next line.
    heard: Vec<u8>,
    /// The application, by its identifier.
    application: String,
    /// What a person calls it.
    called: String,
    /// When the work began.
    began: Instant,
    /// Whether this is counted — alo's copy on a machine that can count it.
    counted: bool,
    /// Each control group of alo's copy held, with what it had sent then.
    groups: BTreeMap<u64, u64>,
    /// The line on the indicator, and when it went up.
    line: Option<(WhileAloWorks, Instant)>,
}

/// A line staying on past its work until it has been seen.
struct Lingering {
    /// The line.
    line: WhileAloWorks,
    /// When it may come off.
    until: Instant,
}

/// How a connection stands this frame.
enum Said {
    /// Nothing new.
    Nothing,
    /// `done`.
    Done,
    /// It hung up, or said something that is not `done`.
    Gone,
}

/// Every application alo is working in.
pub struct WorkingInApplications {
    /// alo's copies.
    copies: Box<dyn StartsAlosCopy>,
    /// The work going on now.
    works: Vec<OneWork>,
    /// Lines staying on until they have been seen.
    lingering: Vec<Lingering>,
}

impl WorkingInApplications {
    /// Working with these copies.
    #[must_use]
    pub fn with(copies: Box<dyn StartsAlosCopy>) -> Self {
        Self {
            copies,
            works: Vec::new(),
            lingering: Vec::new(),
        }
    }

    /// Whether the process `pid` runs in alo's own copy of `application`
    /// (ADR 0153), as the copies say.
    pub fn runs_as_alos_copy(&mut self, application: &str, pid: u32) -> bool {
        self.copies.runs_as_alos_copy(application, pid)
    }

    /// Open `file` in alo's own copy of `application`, as the copies say
    /// (`open_file_in_application`, 2026-11-14).
    pub fn opened_in_alos_copy(&mut self, application: &str, file: &std::path::Path) -> Opened {
        self.copies.started_with_a_file(application, file)
    }

    /// How many applications alo is working in now.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.works.len()
    }

    /// Take work the door was asked for, and answer whether alo may begin.
    ///
    /// `called` is what a person calls the application, or [`None`] when it is
    /// not installed. `chosen` is where the person lets alo work, read from
    /// their settings for this ask.
    pub fn taken_in(
        &mut self,
        asked: AskedToWork,
        called: Option<&str>,
        chosen: AloInApplications,
        counts: &mut CountingForCommands,
        indicator: &mut Indicator,
        now: (Instant, SystemTime),
    ) {
        let AskedToWork {
            asked,
            mut talking,
            heard,
        } = asked;
        let Some(called) = called else {
            say(&mut talking, Working::NotInstalled.written());
            return;
        };
        let application = asked.application().identifier().to_owned();
        // A name that cannot be shown on one line cannot be the line that has
        // to be up, so alo may not work there.
        if AloAtWork::of(&application, called, HowItIsShown::TheirWindow).is_err()
            || self.works.len() >= THE_MOST_AT_ONCE
        {
            say(&mut talking, Working::CannotShow.written());
            return;
        }
        let mut work = OneWork {
            talking,
            heard,
            application,
            called: called.to_owned(),
            began: now.0,
            counted: false,
            groups: BTreeMap::new(),
            line: None,
        };
        let answer = match asked.place() {
            // **The person's window only where the person chose it**: the word
            // alo sent is a request, and the setting is the answer.
            Where::InTheirWindow if !chosen.may_work_in_their_window() => {
                say(&mut work.talking, Working::NotChosen.written());
                return;
            }
            Where::InTheirWindow => {
                put_up(&mut work, HowItIsShown::TheirWindow, indicator, now);
                Working::Showing
            }
            Where::InAlosCopy => match self.copies.started(&work.application) {
                // **Turned up, an application with no copy of its own is worked
                // in where the person is**, shown and never counted.
                Err(NoCopy::OfItsOwn) if chosen.may_work_in_their_window() => {
                    put_up(&mut work, HowItIsShown::TheirWindow, indicator, now);
                    Working::Showing
                }
                Err(NoCopy::OfItsOwn) => {
                    say(&mut work.talking, Working::NoCopyOfItsOwn.written());
                    return;
                }
                Err(NoCopy::CouldNotStart) => {
                    say(&mut work.talking, Working::CouldNotStart.written());
                    return;
                }
                Ok(()) if counts.can_count() => {
                    work.counted = true;
                    if !self.joined(&mut work, counts) {
                        uncounted(&mut work, counts, indicator, now);
                    }
                    if work.counted {
                        Working::Counting
                    } else {
                        Working::CannotCount
                    }
                }
                Ok(()) => {
                    put_up(&mut work, HowItIsShown::Uncounted, indicator, now);
                    Working::CannotCount
                }
            },
        };
        if say(&mut work.talking, answer.written()) {
            self.works.push(work);
        } else {
            self.finished(work, counts, indicator, now.0);
        }
    }

    /// Look at all of alo's work once: find the copies' groups, put a line up
    /// for a copy seen sending, answer work that is done, end what is over.
    pub fn looked_at(
        &mut self,
        counts: &mut CountingForCommands,
        indicator: &mut Indicator,
        now: (Instant, SystemTime),
    ) {
        counts.alos_copies_run_in(self.copies.every_copys_groups());
        let mut still = Vec::with_capacity(self.works.len());
        for mut work in std::mem::take(&mut self.works) {
            if work.counted {
                let whole = self.joined(&mut work, counts);
                let unfound = work.groups.is_empty()
                    && now.0.saturating_duration_since(work.began) >= FOUND_WITHIN;
                if !whole || unfound {
                    uncounted(&mut work, counts, indicator, now);
                } else if work.line.is_none() && sent_by(&work, counts) > 0 {
                    put_up(&mut work, HowItIsShown::Counted, indicator, now);
                }
            }
            let sent = sent_by(&work, counts);
            let at_the_end = if work.counted {
                Worked::Sent(sent)
            } else {
                Worked::Uncounted
            };
            let too_long = now.0.saturating_duration_since(work.began) >= THE_LONGEST;
            match heard_from(&mut work.talking, &mut work.heard) {
                Said::Done => {
                    say(&mut work.talking, &at_the_end.written());
                    self.finished(work, counts, indicator, now.0);
                }
                Said::Gone => self.finished(work, counts, indicator, now.0),
                Said::Nothing if too_long => {
                    let stopped = if work.counted {
                        Worked::Stopped(sent)
                    } else {
                        Worked::Uncounted
                    };
                    say(&mut work.talking, &stopped.written());
                    self.finished(work, counts, indicator, now.0);
                }
                Said::Nothing => still.push(work),
            }
        }
        self.works = still;
        let (over, staying): (Vec<_>, Vec<_>) = std::mem::take(&mut self.lingering)
            .into_iter()
            .partition(|lingering| now.0 >= lingering.until);
        self.lingering = staying;
        for lingering in over {
            indicator.ended_alo_at_work(lingering.line);
        }
    }

    /// Hold every group of the copy not yet held; `false` if one could not be.
    fn joined(&mut self, work: &mut OneWork, counts: &mut CountingForCommands) -> bool {
        for group in self.copies.groups_of(&work.application) {
            if work.groups.contains_key(&group) {
                continue;
            }
            if work.groups.len() >= THE_MOST_GROUPS {
                return false;
            }
            let Some(from) = counts.held(group) else {
                return false;
            };
            work.groups.insert(group, from);
        }
        true
    }

    /// The work is over: its holds are let go, and its line comes off — or
    /// stays until it has been seen.
    fn finished(
        &mut self,
        work: OneWork,
        counts: &mut CountingForCommands,
        indicator: &mut Indicator,
        now: Instant,
    ) {
        for group in work.groups.keys() {
            counts.let_go(*group);
        }
        if let Some((line, up)) = work.line {
            let until = up + SEEN_FOR_AT_LEAST;
            if now >= until {
                indicator.ended_alo_at_work(line);
            } else {
                self.lingering.push(Lingering { line, until });
            }
        }
    }
}

/// What the copy has sent since alo began working in it.
fn sent_by(work: &OneWork, counts: &CountingForCommands) -> u64 {
    work.groups
        .iter()
        .map(|(group, from)| counts.so_far(*group).unwrap_or(*from).saturating_sub(*from))
        .fold(0_u64, u64::saturating_add)
}

/// This copy cannot be counted whole: stop counting it, and make sure the line
/// that says so is up.
fn uncounted(
    work: &mut OneWork,
    counts: &mut CountingForCommands,
    indicator: &mut Indicator,
    now: (Instant, SystemTime),
) {
    work.counted = false;
    for group in std::mem::take(&mut work.groups).keys() {
        counts.let_go(*group);
    }
    if let Some((line, _)) = work.line.take() {
        indicator.ended_alo_at_work(line);
    }
    put_up(work, HowItIsShown::Uncounted, indicator, now);
}

/// Put this work's line up.
fn put_up(
    work: &mut OneWork,
    how: HowItIsShown,
    indicator: &mut Indicator,
    now: (Instant, SystemTime),
) {
    // Checked when the work was taken in, by the same rule.
    if let Ok(at_work) = AloAtWork::of(&work.application, &work.called, how) {
        let line = indicator.beginning_alo_at_work(at_work, now.1);
        work.line = Some((line, now.0));
    }
}

/// Say one line on the connection; `false` if it could not be said.
fn say(talking: &mut UnixStream, line: &str) -> bool {
    talking.write_all(line.as_bytes()).is_ok()
        && talking.write_all(b"\n").is_ok()
        && talking.flush().is_ok()
}

/// Whatever alo has said since the last look.
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
#[path = "alo_at_work_tests.rs"]
mod tests;
