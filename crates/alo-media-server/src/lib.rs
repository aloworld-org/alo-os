//! **Reaching this machine's rented media server, and reading what it says.**
//!
//! One crate for the twenty lines that four crates each had their own copy of.
//! `alo-in-use` reads the graph to say what is watching or listening,
//! `alo-sound` reads and sets the devices a person hears through, `alo-cameras`
//! lists what can see, and `alo-capturing` opens a stream — and every one of
//! them started the server's own tools, cleared an environment and parsed a
//! record on its own.
//!
//! **The copies drifted, which is the whole argument.** One of them did not pass
//! the variable that says where the server is listening, so it spent a day
//! telling a machine with a working server that its server would not answer. One
//! of them turned a record that arrived as two lists into *this machine answered
//! something unreadable*. Each was fixed where it was found; each fix had to be
//! made three times. It is the same fault as a list of crates maintained by
//! hand: **one truth written down in several places.**
//!
//! | | |
//! |---|---|
//! | [`ATool`] | starting one of the server's tools, with the environment it needs and nothing else |
//! | [`TheMediaServer`], [`AsksIt`] | asking for the record |
//! | [`TheRecord`] | what it said, read as a stream of lists |
//! | [`NotAsked`] | **four facts**, of which two used to read the same |
//!
//! # The four facts, which is why this is a crate rather than a function
//!
//! - **Nothing on this machine handles sound and video** — there is no such tool.
//! - **This machine has no media server running** — the tool is there and nothing
//!   is listening. An ordinary build host, or a machine whose session has not
//!   started. **Not a fault**, and reading it as one sent somebody looking for a
//!   broken service that was never started.
//! - **The server would not answer** — it is there and the asking failed.
//! - **The server answered something unreadable** — which must never quietly
//!   become an empty list, because an indicator showing nothing looks exactly
//!   like a machine where no camera is on.
//!
//! # What this crate does not do
//!
//! **It says nothing to a person.** There is no vocabulary here and there will
//! not be one: *what is watching*, *which speaker is this* and *which camera did
//! you mean* are different sentences belonging to the crates that know which is
//! being asked. This one hands over facts and a record.
//!
//! It also decides nothing about what a node means. Two crates reading the same
//! object mean different things by it, and that is theirs.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod asking;
pub mod reaching;
pub mod record;
pub mod refusing;

pub use asking::{AsksIt, THE_TOOL, TheMediaServer};
pub use reaching::{ATool, WHERE_ITS_PROGRAMS_ARE, WHERE_THE_SERVER_IS};
pub use record::{TheRecord, read};
pub use refusing::NotAsked;
