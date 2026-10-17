//! **The join**: a person asked for an application and this machine installs
//! it.
//!
//! `MarketWindow` has recorded what somebody asked for since it was written and
//! **nothing answered it.** The window held an identifier, drew *installing
//! it*, and waited to be told what became of it by a caller that did not exist
//! — so a person could press Install and watch a row say *installing it* for
//! ever.
//!
//! This is the caller. `crate::what_a_person_asked_goes_to_the_agent` is the
//! same shape one road along and its header's argument holds here word for
//! word: **nothing here waits.** An installation fetches an application over a
//! network and takes minutes; this is read from a compositor's frame loop, so
//! waiting here would stop every window on the machine from drawing until a
//! download finished.
//!
//! # Where the work is cut, and why exactly there
//!
//! `alo_turn::installing_an_application` writes down the order an installation
//! happens in, and this file keeps it. What is new is the **seam**, placed at
//! the one line where the machine starts reaching somewhere else:
//!
//! 1. `Enabled::read` — which places this machine may install from, under the
//!    bound an organisation set;
//! 2. `alo_software::installing` — may this place be used, and is the
//!    application already here;
//! 3. the line goes up on the indicator;
//!
//! — all three on the **frame loop's own thread**, because `alo-software`
//! promises in as many words that *steps 1 to 3 refuse without anything
//! leaving*. They run the local tool and reach no network, so the cost is a
//! stutter of tens of milliseconds rather than a wait on somebody else's
//! server.
//!
//! 4. `alo_software::install` — **on a thread of its own**, because this is the
//!    step that fetches;
//! 5. the line comes down, back on the frame loop, when the answer is drained.
//!
//! **Law 1 is satisfied by where the cut is rather than by remembering to do
//! it.** `alo_software::install` cannot run without the `alo_egress::Underway`
//! that
//! lighting the line produced, and the thread cannot have one until it is sent:
//! it begins by **waiting** for the authority, which the frame loop sends only
//! after the line is up. An installation that somehow started without the line
//! lit would refuse itself.
//!
//! ## The thread waits to be handed the authority, and that is not a flourish
//!
//! The first version of this file moved the `Underway` **into** the closure and
//! lit the line before spawning. `std::thread::Builder::spawn` drops the
//! closure when it fails, which drops the authority — so a machine that could
//! not start a thread would have had a line on its indicator **for the rest of
//! the session**, saying something was leaving when nothing was, with nothing
//! able to take it off. Law 1's surface lying is a worse fault than the
//! installation not happening.
//!
//! Handing it over after the spawn succeeds means nothing is lit on the one
//! road where nothing can take it down.
//!
//! # One thread per installation, and at most one installation
//!
//! `MarketWindow` holds one identifier, because *a person who asked for two
//! things before either finished would be shown two rows doing something and
//! could not tell which answer belonged to which.* The window enforces that,
//! which is where the rule belongs; this file would happily run two.
//!
//! # What this does not do
//!
//! **It does not decide whether a person may install anything.** The places and
//! the organisation's bound are `alo-software`'s, read fresh on every ask, so a
//! rule changed between two asks takes effect on the second.
//!
//! **It words nothing.** Every refusal is an `alo_software::Stopped`, worded by
//! the crate that made it, and the window renders it where a person reads.
//! There is not a sentence in this file.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::SystemTime;

use alo_egress::{Indicator, Underway};
use alo_software::{Bound, Enabled, Installed, Listed, NotDone, Stopped, Tool, Wanted};

/// What became of one installation a person asked for.
///
/// **No `Underway` on it.** The line is taken off inside
/// [`TheInstallingThatIsAsked::whatever_became_of_them`], which is the only
/// place it can be, so a caller is handed what it has to tell the window and no
/// authority it could misuse.
#[derive(Debug)]
pub struct WhatBecameOfIt {
    /// Which application, by the identifier the person's window knows it by.
    pub identifier: String,
    /// It arrived, or it was stopped and this is what to say about it.
    pub outcome: Result<Installed, Stopped>,
}

/// One installation's answer and the authority for its line, as the thread
/// posts it.
///
/// The authority is carried **in the message** rather than kept here, so that
/// one answer takes off exactly the line its own installation put on. A map
/// from identifier to line would be a second place that has to agree.
struct Finished {
    /// What the window is told.
    what: WhatBecameOfIt,
    /// The line to take off.
    underway: Underway,
}

/// Everything one thread is handed when the line is up.
///
/// Sent rather than moved into the closure, for the reason this module's header
/// gives at length: a closure that is dropped takes the authority with it.
struct TheAuthority {
    /// The line this installation is on.
    underway: Underway,
    /// The places this machine may install from, as they were read.
    enabled: Enabled,
    /// What is being installed.
    wanted: Wanted,
}

/// The road from *a person asked for this* to *it is here*.
///
/// **Holds the tool rather than taking one per call**, because it is the same
/// tool all session and an `Arc` is what lets a thread of its own use it.
/// `alo_software::TheRentedTool` is the one that ships and a test passes its
/// own, which is the whole reason this is a trait object rather than that type.
pub struct TheInstallingThatIsAsked {
    /// What installs applications on this machine.
    tool: Arc<dyn Tool + Send + Sync>,
    /// The organisation's rule about which places may be used.
    ///
    /// **Taken rather than read**, which is `alo_turn::Machine`'s own rule for
    /// this very value: it is somebody else's rule, off a file a root wrote,
    /// and a compositor that opened that file would be a compositor deciding
    /// what an organisation permits.
    bound: Bound,
    /// Threads post here; the frame loop drains.
    posting: Sender<Finished>,
    /// What has come back and not been read.
    arriving: Receiver<Finished>,
}

impl TheInstallingThatIsAsked {
    /// This road, with the tool that installs and the rule it installs under.
    #[must_use]
    pub fn with(tool: Arc<dyn Tool + Send + Sync>, bound: Bound) -> Self {
        let (posting, arriving) = channel();
        Self {
            tool,
            bound,
            posting,
            arriving,
        }
    }

    /// Put this installation's errand on the indicator and hand the fetching to
    /// a thread.
    ///
    /// Answers `Ok(())` when the thread has the authority and the line is up.
    ///
    /// # Errors
    /// [`Stopped`], worded by whichever crate refused it: a place the
    /// organisation does not permit, a place switched off, the application
    /// already here, or a tool that would not say. **The indicator stays quiet
    /// on every one of those**, which is `alo-software`'s own promise kept by
    /// asking them before the line goes up rather than after.
    pub fn asked_for(
        &self,
        listed: &Listed,
        indicator: &mut Indicator,
        now: SystemTime,
    ) -> Result<(), Stopped> {
        // Steps 1 and 2, in `alo_turn::installing_an_application`'s order.
        // Both reach the local tool and no network, and both refuse with the
        // indicator untouched.
        // **The rule is cloned rather than held by reference**, because
        // `Enabled::read` takes it by value: it is somebody else's rule and
        // reading the places under it consumes the statement of it, which is
        // `alo-software`'s shape and not something to work around here.
        let enabled = Enabled::read(&*self.tool, self.bound.clone())?;
        let wanted = listed.wanted();
        let errand = alo_software::installing(&enabled, &wanted, &*self.tool)?;

        // The thread, started **before** the line goes up and waiting for the
        // authority. See this module's header for what moving the authority in
        // instead cost.
        let (handing, taking) = channel::<TheAuthority>();
        let identifier = listed.application().identifier().to_owned();
        let tool = Arc::clone(&self.tool);
        let posting = self.posting.clone();
        let for_the_answer = identifier.clone();
        let started = std::thread::Builder::new()
            .name("alo-installing".to_owned())
            .spawn(move || {
                // **A sender dropped without sending means the line never went
                // up**, so there is nothing to install and nothing to take
                // off. Returning is the whole of the right behaviour.
                let Ok(TheAuthority {
                    underway,
                    enabled,
                    wanted,
                }) = taking.recv()
                else {
                    return;
                };
                let outcome = alo_software::install(&underway, &enabled, wanted, &*tool);
                drop(posting.send(Finished {
                    what: WhatBecameOfIt {
                        identifier: for_the_answer,
                        outcome,
                    },
                    underway,
                }));
            });
        if let Err(why) = started {
            // **Nothing is lit, so there is nothing to take down.** This is the
            // one road out of here on which no answer will ever arrive, and it
            // is exactly why the line is not up yet.
            //
            // **`DidNotAnswer` for a thread, which is a stretch worth naming.**
            // Its own words are *the rented tool did not respond*, and this is
            // not the tool. What a person reads for it is right —
            // `alo-software` words it as *the part of this machine that
            // installs applications* failing, which a thread that would not
            // start is — and the English kept beside it for whoever
            // administers the machine says what really happened. The
            // alternative was a variant in `alo-software`'s vocabulary for a
            // compositor's thread, which would put this crate's problem in
            // that crate's words.
            return Err(Stopped::Refused(NotDone::DidNotAnswer {
                said: format!("the thread that installs applications would not start: {why}"),
            }));
        }

        // Step 3, and then the authority goes across. The thread is already
        // waiting, so the order here is the order on the machine: the line is
        // up before anything can fetch.
        let underway = indicator.beginning_on_its_own(errand, now);
        // **A send that fails means the thread has gone**, which it can only
        // have done by returning — and it returns only when the authority was
        // never sent. Nothing has fetched, so the line comes straight off
        // again and the person is told the machine could not.
        if let Err(returned) = handing.send(TheAuthority {
            underway,
            enabled,
            wanted,
        }) {
            let ended = indicator.ended_on_its_own(returned.0.underway);
            debug_assert!(
                ended,
                "the line this errand put up was not on the indicator"
            );
            return Err(Stopped::Refused(NotDone::DidNotAnswer {
                said: "the thread that installs applications ended before it was asked to"
                    .to_owned(),
            }));
        }
        Ok(())
    }

    /// Everything that has come back since this was last asked, with each
    /// installation's line taken off the indicator.
    ///
    /// **Drained rather than read**, so one answer reaches one window once —
    /// `crate::what_a_person_asked_goes_to_the_agent`'s rule at the other end
    /// of the same kind of road.
    ///
    /// **The line comes down here and nowhere else.** Taking the indicator
    /// mutably is what says so in the signature: a caller that wanted the
    /// answers without the indicator could not have them, so there is no way to
    /// read an answer and leave its line lit.
    pub fn whatever_became_of_them(&self, indicator: &mut Indicator) -> Vec<WhatBecameOfIt> {
        let mut came = Vec::new();
        // **Empty and disconnected end the loop the same way**, which is what
        // `while let` says and a `match` on `TryRecvError` only spelt out at
        // length: this holds the sender, so the only way to see `Disconnected`
        // is a thread that panicked, and *no more answers now* is the right
        // reading of either.
        while let Ok(Finished { what, underway }) = self.arriving.try_recv() {
            // **Whatever happened.** A refused install that left its line lit
            // would be law 1's surface saying something is leaving when nothing
            // is, for the rest of the session —
            // `alo_turn::installing_an_application` says the same of its own
            // road.
            let ended = indicator.ended_on_its_own(underway);
            debug_assert!(ended, "an answer carried a line no indicator had");
            came.push(what);
        }
        came
    }
}

#[cfg(test)]
#[path = "what_a_person_installs_tests.rs"]
mod tests;
