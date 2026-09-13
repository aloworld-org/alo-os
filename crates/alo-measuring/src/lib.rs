//! What is running on this machine and what it is using, and what is filling
//! its disk — read, not estimated.
//!
//! `docs/features.md` promises, at v0.5: *what is running, and what it is
//! using — processes, memory, disk and network in a window. The plain answer
//! to "why is it slow?", for the person who cannot or will not ask.* And,
//! beside it: *what is filling the disk — shown as sizes you can open up and
//! click through, not a number in Settings.* The windows are the shell's.
//! What they show is this crate's, and the whole of both promises is in the
//! word **plain**: a number the person can check against the machine, not
//! one the machine derived and hopes is close.
//!
//! So every number here is **read** — from the kernel's `/proc` for what is
//! running, from the filesystem for what is filling a folder — and every
//! number says where. A [`Number`] carries its [`Source`]: the file and the
//! line or field in it. A [`Node`] carries its path, and `stat` on it shows
//! the same bytes. A test opens the file and compares, and so can a person.
//!
//! | | |
//! |---|---|
//! | [`Reading`] | Everything the kernel says at one moment: a total per process, per number |
//! | [`Reading::now`] | That, read |
//! | [`Reading::since`] | Two readings and the time between them, as rates: [`Running`] |
//! | [`Running`], [`Process`], [`Gone`] | What is running, what each is using, and what ended between the readings |
//! | [`Number`], [`Source`] | One number and where it came from — or why there is no number |
//! | [`Holding`], [`Holding::of`] | What is filling a folder: a tree of sizes, counted now |
//! | [`Node`], [`Counted`] | One thing in that tree, its size, and whether the size is the whole truth |
//! | [`NotMeasured`] | The five ways nothing can be said at all |
//! | [`words`] | Every sentence a window shows beside these, in the reader's language |
//!
//! ```no_run
//! use std::path::Path;
//! use std::time::Duration;
//! use alo_measuring::{Holding, Reading};
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
//!
//! // What is filling a folder: each node's size is its own bytes plus its
//! // children's, and a node whose size is not the whole truth says why.
//! let holding = Holding::of(Path::new("Documents"))?;
//! for child in &holding.tree.children {
//!     let _ = (child.size, &child.counted);
//! }
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
//! # A size that is silently too small is worse than no size
//!
//! [`Holding::of`] answers *what is filling this folder* as a tree, each
//! node's size the sum of its children plus its own bytes. The walk is
//! `alo-files`' — borrowed, not written again, so that there is one opinion
//! in this repository about what a link is — and every way a size can fall
//! short of the truth is written on the node it happens to, as a [`Counted`]:
//! a file with two names is counted once and the second name says where; a
//! link is the bytes of the link and is never followed; a folder the machine
//! would not read says so rather than being a zero; a folder on another
//! filesystem is not entered, which is how naming the root of the machine is
//! answered — a tree that stops at each mount point; and a count that reached
//! the walk's bound says so on every folder it had not finished, and once
//! above the tree. Nothing here deletes, moves or empties anything, and the
//! whole disk is never walked unasked: a caller names a folder.
//!
//! # This crate reads, and does nothing else
//!
//! Nothing here signals, stops, renices or otherwise touches a process, and
//! nothing here changes a file; *what is using the machine* and *what is
//! filling it* are measurements, and acting on either would be a verb needing
//! a grant under ADR 0001. Nothing here opens a socket, reads a setting, or
//! asks who is asking: the list and the tree are the same whether an agent or
//! a person wanted them, because they are lists of facts.
//! `tests/nothing_here_acts_or_asks_who_is_asking.rs` reads the shipped
//! source and holds all of that.
//!
//! # This crate measures Linux
//!
//! `/proc` is Linux, and alo OS boots Linux. The crate compiles everywhere —
//! its types, its parsers and its arithmetic are portable and tested on every
//! host against a kernel a test writes out and a walk a test writes out — but
//! [`Reading::now`] and [`Holding::of`] on any other host answer
//! [`NotMeasured::NotOnThisHost`] rather than a reading of nothing, the way
//! `alo-agentd` is absent rather than pretending. What runs the tests that
//! open the real `/proc` and the real disk is the Linux host;
//! `docs/autonomy/LOOP.md` says how.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod counting;
mod holding;
mod io;
mod kernel;
mod kilobytes;
mod listing;
mod looking;
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

pub use alo_files::Kind;
pub use holding::{Counted, Holding, Node};
pub use kernel::{Disk, Kernel};
pub use reading::Reading;
pub use refusing::NotMeasured;
pub use running::{Gone, Network, Process, Running};
pub use sampled::{Machine, Sampled};
pub use source::{Known, Number, Source};
pub use words::{WordsError, declare_into, measuring_words};
