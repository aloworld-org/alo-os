//! Suspend, resume, the lid, and what may keep a machine awake.
//!
//! The mechanism is `logind`'s, rented and configured (ADR 0011). What is this
//! crate's is **the order of things and who may hold the machine awake**, and
//! each is a type rather than a habit:
//!
//! | | |
//! |---|---|
//! | [`asked`], [`Why`], [`Decided`] | Whether the machine sleeps, decided before anything reaches `logind` |
//! | [`Going`], [`Slept`], [`Asleep`] | A sleep carried out: the seat locked, then the machine asked |
//! | [`Woke`], [`AfterSleep`] | Waking locked, and what became of a turn that was running |
//! | [`Woke::the_desk`] | The screens in front of the person at a resume, asked all at once |
//! | [`Lid`], [`Displays`] | What closing the lid does, and the person's choice |
//! | [`Keeper`], [`Holding`] | The closed list of what may keep the machine awake, each named |
//! | [`TheLidIsOurs`], [`UntilLocked`] | The two holds alo OS keeps for a signed-in session |
//! | [`Logind`], [`Inhibit`], [`LockedFirst`] | The two things asked of the machine, and the proof sleeping takes |
//! | [`Changes`], [`Settings`], [`keeping`] | The person's two settings, kept in `sleeping.toml` (ADR 0038) |
//! | `machine` | `systemd-logind` over the system bus — Linux only |
//! | [`words`] | Every sentence this crate can say |
//!
//! # The rules, one clause each
//!
//! 1. **A session is locked before the machine sleeps.** [`Logind::sleep`] takes
//!    a [`LockedFirst`], made only out of a locked seat by
//!    [`Going::carried_out`]; a sleep started elsewhere waits on
//!    [`UntilLocked`], which locks first. A resume lands on the lock screen, and
//!    a sleep that fails leaves the seat locked.
//! 2. **Closing the lid sleeps the machine unless another display is attached
//!    and the person chose otherwise** ([`Lid`]), a choice kept by this crate at
//!    a path it is handed ([`keeping`]).
//! 3. **What may keep the machine awake is a closed list** — the person's own
//!    setting, an application holding the inhibit portal under a grant, a turn
//!    that is running — **and each is named** when it holds the machine awake
//!    ([`StaysAwake::said`]).
//! 4. **An agent cannot keep the machine awake by asking.** [`Holding`] has no
//!    door that takes an agent or a verb; a turn holds for its own length,
//!    read off the turn, and no agent's verb reaches this crate
//!    (`tests/an_agent_cannot_keep_this_machine_awake.rs`).
//! 5. **A turn running when the machine slept is resumed or refused with a
//!    sentence, and written down** ([`Woke::a_turn`]), never silently lost.
//! 6. **A resume asks what the screens are now**, as one whole set rather than
//!    a cable at a time, because a machine that was asleep saw no cable move
//!    ([`Woke::the_desk`], and `alo-displays` decides everything about them).
//!
//! # What is not here
//!
//! **A daemon.** Everything here is called by the session that holds the seat.
//! **Battery thresholds, power profiles and how long idle is** — the devices
//! plan's. **Drawing** — the shell's, from what is decided here.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod changes;
pub mod deciding;
pub mod going;
pub mod holding;
pub mod keeper;
pub mod keeping;
pub mod lid;
pub mod logind;
#[cfg(target_os = "linux")]
mod machine;
pub mod refusing;
pub mod session_holds;
pub mod the_desk;
pub mod unkept;
pub mod waking;
pub mod words;

#[cfg(test)]
mod testing;

pub use changes::{Changes, Setting, Settings};
pub use deciding::{Decided, Going, StaysAwake, Why, asked};
pub use going::{Asleep, NotAsleep, Slept};
pub use holding::{HoldId, Holding};
pub use keeper::Keeper;
pub use lid::{Displays, Lid, LidClosed};
pub use logind::{Inhibit, LockedFirst, Logind, NotHeld, NotSlept};
#[cfg(target_os = "linux")]
pub use machine::TheMachinesLogind;
pub use refusing::NotKeptAwake;
pub use session_holds::{TheLidIsOurs, UntilLocked};
pub use unkept::{FileNotRead, FileNotWritten};
pub use waking::{AfterSleep, Woke};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, sleeping_words};
