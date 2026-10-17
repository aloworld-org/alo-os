//! The two read-only copies of a person's home a changing turn is bracketed
//! by, and the writing-down that goes with them.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) chose the
//! base's own read-only snapshots, one taken as the turn found the home and one
//! as it left it, and put **the taking in this crate**. Its amendment of
//! 2026-10-16 measured that taking one needs no capability at all:
//! `btrfs subvolume snapshot -r` of a subvolume the person owns, run as the
//! person with no `sudo`, exits 0.
//!
//! Everything around this was already built. `alo-giving-a-folder` makes the
//! folder at first sign-in, `alo-kept-undo` holds its layout,
//! `alo-keeping-up::WhatWasKept` says what each outcome means to a person, and
//! `alo-letting-go` removes an expired snapshot under a capability this crate
//! must never have. **This was the missing middle**, in ADR 0045's own words:
//! *both ends of this feature are built and the middle is absent.*
//!
//! # A turn that could not be bracketed still runs, and that is structural here
//!
//! ADR 0045's third accepted term:
//!
//! > **A machine too full to snapshot still runs the turn.** The bracket is
//! > attempted; when there is no room it is not taken, the turn proceeds, and
//! > the record says plainly that this one cannot be undone. A verb that refused
//! > to run for want of an undo would be a machine that stopped working to
//! > protect a feature.
//!
//! So [`ASideOfTheTurn`] is **not a `Result`**, and that is the whole reason:
//! a `Result` invites `?`, and one `?` on this road would turn *there was no
//! room* into *the turn failed* — which is the behaviour that term exists to
//! forbid. There is no error to propagate because nothing here can fail the
//! turn. What a caller must do instead is say which [`WhyNot`] it was, and
//! `alo_keeping_up::WhatWasKept::NotThisTurn` is the sentence a person then
//! reads.
//!
//! # Read-only, because of what an agent may not do
//!
//! `-r` is not a precaution, it is [ADR 0045's amendment recorded by #752]: the
//! agent can take a bracket and can **neither alter nor destroy one**. A
//! writable snapshot would make the first of those false, and a read-only
//! subvolume cannot be removed without `CAP_SYS_ADMIN` — measured as the person
//! on a real filesystem — which makes the second true without this crate
//! holding any capability.
//!
//! # The base's own program, and exactly one of them
//!
//! `btrfs` does the work, as it does for the remover. *Never a second
//! implementation* is ADR 0045's constraint and `CLAUDE.md`'s *engines are
//! configured, never written in*: there is no second road to a snapshot that we
//! would then have to keep correct. `the_bracket_a_turn_takes_tests.rs` reads
//! this file's own source and refuses a second `Command::new`, which is the
//! shape `alo-letting-go`'s remover already has.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// The program that makes a snapshot. The base's, never ours.
const THE_PROGRAM: &str = "btrfs";

/// What it is asked to do.
///
/// `-r` is read-only and is load-bearing — see this module's header.
const THE_ARGUMENTS: [&str; 3] = ["subvolume", "snapshot", "-r"];

/// The longest a turn's directory name may be.
///
/// The contract draws one as `4f1c8a…`, a hex name. This is generous for any
/// identity a turn could have and short enough that a name cannot be used to
/// exhaust a path.
const THE_LONGEST_NAME: usize = 64;

/// Why a side of the turn was not kept.
///
/// **Every one of these means the same thing to a person** — this turn cannot
/// be undone — and they are told apart because the record says which, not
/// because anything behaves differently. ADR 0045's second term: *what a person
/// is told names the turns that lost it rather than a number.*
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyNot {
    /// The turn's name could not be a directory under the person's.
    ///
    /// Refused before anything is made: a name with a separator in it, an empty
    /// one, or one longer than sixty-four characters. **This is the one refusal
    /// that is about safety rather than about room** — a caller handing `..`
    /// would otherwise be handed a path outside the person's own directory.
    ThatIsNotATurnsName {
        /// What was offered, for the record to say.
        offered: String,
    },
    /// The directory for this turn could not be made.
    NoDirectory {
        /// Where it would have gone.
        at: PathBuf,
        /// What the system said.
        why: String,
    },
    /// The program could not be started at all.
    ///
    /// A machine with no `btrfs` program is a machine alo OS did not install —
    /// `alo-image` makes the one filesystem — so this is the ordinary answer on
    /// a developer's machine and not a fault there.
    TheProgramWouldNotRun {
        /// What the system said.
        why: String,
    },
    /// The program ran and refused.
    ///
    /// A full disk, a destination that exists, or a source that is not a
    /// subvolume. **Its own words are kept** rather than reworded, because the
    /// program knows why and this crate does not.
    ItRefused {
        /// What it said, trimmed of trailing newlines and nothing else.
        said: String,
    },
    /// The turn's note could not be written down.
    NothingWrittenDown {
        /// What the system said.
        why: String,
    },
}

/// One side of a changing turn: kept, or not kept and why.
///
/// **Deliberately not a `Result`.** See this module's header: a `?` on this road
/// would turn *there was no room* into *the turn failed*, which ADR 0045's third
/// term forbids by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASideOfTheTurn {
    /// It was kept, here.
    Kept(PathBuf),
    /// It was not, and this is why.
    NotKept(WhyNot),
}

impl ASideOfTheTurn {
    /// Where it was kept, if it was.
    #[must_use]
    pub fn kept_at(&self) -> Option<&Path> {
        match self {
            Self::Kept(at) => Some(at),
            Self::NotKept(_) => None,
        }
    }

    /// Why it was not kept, if it was not.
    #[must_use]
    pub const fn why_not(&self) -> Option<&WhyNot> {
        match self {
            Self::Kept(_) => None,
            Self::NotKept(why) => Some(why),
        }
    }
}

/// Whether this could be a directory under a person's own.
///
/// ASCII letters, digits and `-`, bounded, and not empty. **A separator, a dot
/// or an empty name is refused**, which is what keeps a caller's turn identity
/// from reaching outside the person's directory — and refusing dots is what
/// refuses `..`.
///
/// **Deliberately the safe subset and not a format**, because
/// `docs/contracts/kept-undo-folder.md` says *a kept turn's name is whatever the
/// process that took the bracket chose*. A stricter rule here — hex only, which
/// this was first — would refuse a caller whose turn identity had a dash or a
/// capital in it, and refuse **every** bracket on that machine without anything
/// saying why. A validator narrower than the contract it serves is a feature that
/// never works and cannot be seen not working.
fn a_turns_name(offered: &str) -> bool {
    !offered.is_empty()
        && offered.len() <= THE_LONGEST_NAME
        && offered
            .chars()
            .all(|it| it.is_ascii_alphanumeric() || it == '-')
}

/// One changing turn's place in a person's own undo directory.
///
/// Made by the caller that knows **which person** — the daemon, from
/// `SO_PEERCRED` on its socket — because nothing in this crate learns that and
/// nothing in it should.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereThisTurnIsKept {
    /// The turn's own directory, inside the person's.
    at: PathBuf,
}

impl WhereThisTurnIsKept {
    /// Make the directory this turn's bracket goes in.
    ///
    /// `whose` is the person's own directory — `/var/lib/alo-undo/<person>`,
    /// which `alo-giving-a-folder` made at their first sign-in and which is
    /// theirs at `0700`. `named` is the turn's identity.
    ///
    /// # Errors
    /// [`WhyNot::ThatIsNotATurnsName`] for a name that could reach out of the
    /// person's own directory, and [`WhyNot::NoDirectory`] when it could not be
    /// made.
    ///
    /// **This one is a `Result` and [`Self::keeping`] is not**, which is not an
    /// inconsistency: a turn whose directory could not even be made has nowhere
    /// to record that it was not bracketed, so the caller has to know. A turn
    /// whose *snapshot* was not taken has somewhere to say so, which is why that
    /// road cannot be `?`-ed into a failed turn.
    pub fn made(whose: &Path, named: &str) -> Result<Self, WhyNot> {
        if !a_turns_name(named) {
            return Err(WhyNot::ThatIsNotATurnsName {
                offered: named.to_owned(),
            });
        }
        let at = whose.join(named);
        std::fs::create_dir_all(&at).map_err(|why| WhyNot::NoDirectory {
            at: at.clone(),
            why: why.to_string(),
        })?;
        Ok(Self { at })
    }

    /// Where it is.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// Take one side of the turn: a read-only snapshot of `of`, called `side`.
    ///
    /// `side` is `alo_kept_undo::BEFORE` or `alo_kept_undo::AFTER`; `of` is the
    /// person's home subvolume. The snapshot is read-only, for the reason in
    /// this module's header.
    #[must_use]
    pub fn keeping(&self, side: &str, of: &Path) -> ASideOfTheTurn {
        let into = self.at.join(side);
        let said = Command::new(THE_PROGRAM)
            .args(THE_ARGUMENTS)
            .arg(of)
            .arg(&into)
            .output();
        match said {
            Err(why) => ASideOfTheTurn::NotKept(WhyNot::TheProgramWouldNotRun {
                why: why.to_string(),
            }),
            Ok(done) if done.status.success() => ASideOfTheTurn::Kept(into),
            Ok(done) => ASideOfTheTurn::NotKept(WhyNot::ItRefused {
                said: String::from_utf8_lossy(&done.stderr).trim_end().to_owned(),
            }),
        }
    }

    /// Write down when this turn ran and what it did, in the person's words.
    ///
    /// `did` is the sentence the person approved at the time, which is what
    /// `alo_kept_undo::TheTurn` keeps and what a person reads months later. The
    /// contract's own words: *the session that takes the bracket, which knows,
    /// writes it down.*
    ///
    /// # Errors
    /// [`WhyNot::NothingWrittenDown`] when the file could not be written, and
    /// whatever `alo_kept_undo` refuses the note itself for.
    pub fn what_it_did(&self, taken: SystemTime, did: &str) -> Result<(), WhyNot> {
        let at = self.at.join(alo_kept_undo::THE_TURN);
        let said = |why: alo_kept_undo::NotWhatItSays| WhyNot::NothingWrittenDown {
            why: why.to_string(),
        };
        let note = alo_kept_undo::TheTurn::done(taken, did, &at).map_err(said)?;
        let text = note.as_text(&at).map_err(said)?;
        std::fs::write(&at, text).map_err(|why| WhyNot::NothingWrittenDown {
            why: why.to_string(),
        })
    }
}

/// Say where this person's settings folder is, if nothing has yet.
///
/// `whose` is the person's own undo directory; `settings` is their
/// `$XDG_CONFIG_HOME/alo`, absolute.
///
/// **Without this the disk fills.** `docs/contracts/kept-undo-folder.md`: *a
/// person's directory whose `theirs.json` will not read is stepped over whole*,
/// and one that has none at all is the same to a reader — so the privileged
/// remover would never take an expired snapshot of theirs, for ever. The
/// contract names who writes it: *the session that takes the bracket, which
/// knows, writes it down.*
///
/// **Written once and never rewritten.** A person's settings folder does not
/// move while they are signed in, and rewriting it every turn would make the
/// one file the remover depends on the file most often half-written.
///
/// # Errors
/// [`WhyNot::NothingWrittenDown`], and whatever `alo-kept-undo` refuses a
/// settings path for — a relative one, which it will not record.
pub fn where_their_settings_are(whose: &Path, settings: &Path) -> Result<(), WhyNot> {
    let at = whose.join(alo_kept_undo::THEIRS);
    if at.exists() {
        return Ok(());
    }
    let said = |why: alo_kept_undo::NotWhatItSays| WhyNot::NothingWrittenDown {
        why: why.to_string(),
    };
    let theirs = alo_kept_undo::Theirs::at(settings, &at).map_err(said)?;
    let text = theirs.as_text(&at).map_err(said)?;
    std::fs::write(&at, text).map_err(|why| WhyNot::NothingWrittenDown {
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "the_bracket_a_turn_takes_tests.rs"]
mod tests;
