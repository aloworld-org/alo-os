//! What is running on this machine and what it is using — read, not estimated.
//!
//! `docs/features.md` promises, at v0.5: *what is running, and what it is
//! using — processes, memory, disk and network in a window. The plain answer
//! to "why is it slow?", for the person who cannot or will not ask.* The window
//! is the shell's. What it shows is this crate's, and the whole of the promise
//! is in the word **plain**: a number the person can check against the
//! machine, not one the machine derived and hopes is close.
//!
//! So every number here is **read from the kernel** — `/proc`, and nowhere
//! else — and every number is **named for the file it came from**. A
//! [`Number`] carries its [`Source`]: the file and the line or field in it.
//! A test opens that file and compares, and so can a person with `cat`.
//!
//! | | |
//! |---|---|
//! | [`Reading`] | Everything the kernel says at one moment: a total per process, per number |
//! | [`Reading::now`] | That, read |
//! | [`Reading::since`] | Two readings and the time between them, as rates: [`Running`] |
//! | [`Running`], [`Process`], [`Gone`] | What is running, what each is using, and what ended between the readings |
//! | [`Number`], [`Source`] | One number and where it came from — or why there is no number |
//! | [`NotMeasured`] | The four ways nothing can be said at all |
//! | [`words`] | Every sentence a window shows beside these, in the reader's language |
//!
//! ```no_run
//! use std::time::Duration;
//! use alo_measuring::Reading;
//!
//! // A total is a fact about a moment; a rate needs two of them.
//! let earlier = Reading::now()?;
//! std::thread::sleep(Duration::from_secs(1));
//! let later = Reading::now()?;
//!
//! // The interval is passed in, not read from a clock this crate does not own.
//! let running = later.since(&earlier, Duration::from_secs(1))?;
//! for process in running.processes() {
//!     // Bytes written per second, and the file it was read from.
//!     let _ = (&process.written, process.written.from());
//! }
//! // A process that ended between the readings is here, not reported as zero.
//! let _ = running.gone();
//! # Ok::<(), alo_measuring::NotMeasured>(())
//! ```
//!
//! # Asking twice gives a rate, and the interval is passed in
//!
//! A [`Reading`] holds totals: bytes read since the process began, ticks of
//! processor time since it began. A total is not what a person asking *why is
//! it slow?* wants — they want what it is doing **now** — and *now* is two
//! readings with time between them. [`Reading::since`] turns the pair into
//! per-second rates and a share of the processor, and the interval is an
//! argument rather than something read from a clock: the caller that took the
//! two readings knows how far apart they were, and a crate that consulted a
//! clock of its own would be introducing a second opinion about time.
//!
//! **A process that exited between the readings is reported as [`Gone`]**, not
//! as a process using nothing. A pid the kernel has reused for a new process
//! between two readings is told apart by the moment it started, which is in
//! `/proc/<pid>/stat` — the old one is gone, and the new one has no rate yet.
//!
//! # Where a number is not a number
//!
//! The kernel does not always give one, and a zero where it did not would be
//! the exact lie the word *plain* forbids. [`Number`] says which it is:
//! *withheld* when the kernel refused to show the file — another person's
//! process, usually; *not said* when the file has no such line — a kernel
//! thread has no resident memory; *not yet* when a process began after the
//! earlier reading and has nothing to be a rate over. Each carries a sentence
//! in the reader's language for the window to show in the number's place.
//!
//! # What a process's network traffic is, and is not
//!
//! Linux keeps no per-process count of bytes on the network. What
//! `/proc/<pid>/net/dev` holds is the count for the **network namespace** the
//! process is in — which is exactly the process's own traffic for one
//! sandboxed into a namespace of its own, and the whole machine's for one that
//! shares the default namespace with everything else. Reporting the second as
//! though it were the first would be a number the machine hopes is close, so
//! every [`Process`] carries a [`Network`] saying which namespace the count is
//! for and how many other processes in the list share it. Loopback traffic is
//! left out of the sum, because bytes that never left the machine are not what
//! *network* means to the person asking.
//!
//! # This crate reads, and does nothing else
//!
//! Nothing here signals, stops, renices or otherwise touches a process; *what
//! is using the machine* is a measurement, and acting on it would be a verb
//! needing a grant under ADR 0001. Nothing here opens a socket, reads a
//! setting, or asks who is asking: the list is the same whether an agent or a
//! person wanted it, because it is a list of facts.
//! `tests/nothing_here_acts_or_asks_who_is_asking.rs` reads the shipped
//! source and holds all of that.
//!
//! # This crate measures Linux
//!
//! `/proc` is Linux, and alo OS boots Linux. The crate compiles everywhere —
//! its types, its parsers and its arithmetic are portable and tested on every
//! host against a kernel a test writes out — but [`Reading::now`] on any other
//! host answers [`NotMeasured::NotOnThisHost`] rather than a reading of
//! nothing, the way `alo-agentd` is absent rather than pretending. What runs
//! the tests that open the real `/proc` is the Linux host;
//! `docs/autonomy/LOOP.md` says how.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod io;
mod kernel;
mod kilobytes;
mod listing;
mod netdev;
mod rating;
mod reading;
mod refusing;
mod running;
mod sampled;
mod sampling;
mod source;
mod stat;
pub mod words;

pub use kernel::{Disk, Kernel};
pub use reading::Reading;
pub use refusing::NotMeasured;
pub use running::{Gone, Network, Process, Running};
pub use sampled::{Machine, Sampled};
pub use source::{Known, Number, Source};
pub use words::{WordsError, declare_into, measuring_words};
