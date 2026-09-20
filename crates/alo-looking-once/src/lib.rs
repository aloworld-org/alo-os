//! The one look this machine takes on its way up.
//!
//! `alo-looking` is the act — ask the place this machine's builds come from
//! whether there is a newer version, show it on the indicator for the whole of
//! it, keep the answer. `alo_looking::Because` is the whole list of occasions
//! and it is two, *the person asked* and *this machine started*. The first is a
//! surface's and belongs to the shell. **The second was nobody's**: every piece
//! of *updates that never interrupt* was built and nothing on any alo OS
//! machine had ever looked, which from where the person sits is
//! indistinguishable from a machine that has no updates.
//!
//! This crate is that second occasion, and it is the **only** caller of it on a
//! booted machine.
//!
//! # Once per start, and never again
//!
//! [`AtAStart::look_once`] asks once and answers. There is no thread here, no
//! timer, no retry and no loop, and
//! `tests/a_machine_that_has_never_looked.rs` reads every file of this crate to
//! keep it that way — as task 6's tests read `alo-looking`'s. The repetition a
//! systemd unit could add is refused in the unit as well: `RemainAfterExit=yes`,
//! so a machine that has looked stays looked and starting the unit again does
//! nothing at all.
//!
//! **And it is `Type=exec` rather than `Type=oneshot`, which was measured
//! rather than chosen.** A unit wanted by a target is implicitly ordered before
//! it, and a `oneshot`'s start job is not finished until the process has
//! exited — so a `oneshot` here held `multi-user.target` open for the whole
//! check, and `graphical.target` behind it. On a booted machine on 2026-09-19
//! that was 4.246 s against 1.9 s, and on a machine whose network answers
//! slowly it would have been the twenty seconds a request waits. A check at a
//! start never delays a person's sign-in, and `exec` is where that stops being
//! a sentence.
//!
//! # Where it runs, and under whose privilege
//!
//! **A system unit, started once at `multi-user.target`, running as the
//! person's own login, holding no capability at all.**
//! [`THE_UNIT`] is the file, handed to the lane that owns the
//! image the way task 4 handed over the filesystem it needs; every reason is in
//! its own comments and the two that decided it are these.
//!
//! **Why a system unit rather than a hook in the person's session.** What a
//! check keeps is one answer per machine
//! (`alo_looking::THE_ANSWER`, `/var/lib/alo/an-update-was-found`) and a
//! session is not one per machine. A session hook would make two sign-ins two
//! checks — two departures for one question, twice on somebody's indicator —
//! and would leave a machine nobody signs into never looking at all. *This
//! machine started* is a fact about the machine, so the thing that says it is
//! the machine's.
//!
//! **Why the person's login.** Both files a check writes are inside
//! `/var/lib/alo`, which the image makes `0700 alo alo` because what an agent
//! did on somebody's machine is theirs. A login of its own — the shape
//! `alo-modeld` and `alo-convertd` take — could not write there unless that
//! folder's mode were widened, and no worker may widen the folder the record
//! lives in to make a check convenient. Root is refused on its own merits: this
//! process reads what a public registry sent it, and ADR 0018's argument is
//! that one privileged component is acceptable because of how little it is
//! trusted with. So it runs as the person, with nothing the person does not
//! have themselves (ADR 0001 §2), writing two files that are already theirs.
//!
//! **Nothing here needed a new privileged component or a widened grant**, which
//! is why this crate exists rather than an ADR (ADR 0001 §2 asked for one if it
//! had).
//!
//! # Its own record file, for the reason the broker's is its own
//!
//! [`THE_RECORD`] is `/var/lib/alo/checking-for-updates.jsonl`, beside the
//! person's record rather than inside it. `alo_keeping::Writing` has exactly
//! one writer per file, because two processes appending to one interleave, and
//! the person's record already has one: `alo-agentd`. `alo-brokerd` met this
//! first and answered it the same way — its entries go in a file of its own —
//! and a check at a start cannot be ordered against a sign-in tightly enough to
//! promise the two never overlap.
//!
//! **The record is opened before the check, not after it.** A machine that
//! cannot write down what it is about to do does not do it: there is no road
//! through this crate that reaches the network without somewhere to account for
//! it afterwards, which is the half of law 1 that is not the indicator.
//!
//! # What is not here
//!
//! **Nothing downloads a build and nothing stages one.** A check fetches an
//! answer; applying an update is the person's choice and `alo_updating::apply`'s
//! work, decided by `alo_keeping_up::Staging`.
//!
//! **No setting that turns checking off**, and no member meaning *urgent*.
//! `alo-keeping-up`'s rule since task 1 of the plan: the choice a person has is
//! *when an update applies*, never *whether to be told there is one*.
//!
//! **Nothing a person reads is worded here.** What this program prints goes to
//! the journal, for whoever administers the machine, exactly as `alo-boundaryd`
//! and `alo-agentd` print theirs. The sentence a *person* reads is drawn by a
//! surface from the answer this kept, in their own language.
//!
//! # Why every test is in `tests/`
//!
//! Unusually for this workspace, there is no `#[cfg(test)] mod tests` in any
//! file here. The acceptance of this crate is a test that **reads this crate's
//! own source** and counts the occasions a check is made for; a test module
//! inside the source would be a second mention of the very thing being counted,
//! and a source-reading test that has to except its own test modules is a test
//! with a hole in exactly the shape of the mistake it is looking for.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod once;
pub mod record;
pub mod refusing;
pub mod road;

pub use once::{AtAStart, THE_ANSWER, THE_RECORD};
pub use record::IntoTheRecord;
pub use refusing::DidNotLook;
pub use road::{NoRoad, the_road_out};

/// The unit that runs it, as this crate hands it over.
///
/// **The installation is not this crate's.** The plan that owns this work reads
/// `image/` and never edits it; the lane that owns the image installs this file
/// at `/usr/lib/systemd/system/alo-looking-once.service`, builds the program
/// into `/usr/libexec/alo-looking-once` and enables the unit. What is here is
/// the decision written down where a test can hold it — where it runs, as whom,
/// and with what — rather than a paragraph somebody has to be trusted to have
/// read.
pub const THE_UNIT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/alo-looking-once.service"
));

/// What the unit is called once it is installed.
pub const THE_UNITS_NAME: &str = "alo-looking-once.service";

/// Where the program goes on a machine.
///
/// In `libexec` beside the loader and the opener, and for their reason: nobody
/// runs this by hand. It is started once by systemd at a start, and a person who
/// wants to look now asks a surface, which is the other occasion and is not
/// this one.
pub const THE_PROGRAM: &str = "/usr/libexec/alo-looking-once";
