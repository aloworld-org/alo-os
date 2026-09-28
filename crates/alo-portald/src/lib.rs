//! The portal backend, as a machine really runs it: one person's session bus,
//! one name on it, and a machine read at every request.
//!
//! `#193` landed as *The portal backend is written, and nothing runs it*, and
//! that was exactly true. `alo_portals::Backend::serve_on` owns
//! `org.freedesktop.portal.Desktop` and answers four interfaces, and
//! `the_portal_backend_answers_on_a_real_bus.rs` proves it on a real bus — but
//! **no binary called it and no machine existed to call it with**.
//!
//! This crate is the two things that were missing, and nothing else:
//!
//! | | |
//! |---|---|
//! | [`this_machine::ThisMachine`] | `alo_portals::TheMachine` for a real person's session, which had **eight implementations and every one a test double** |
//! | `src/main.rs` | the process: assemble the four arguments, take the name, stay |
//!
//! # No policy here, and that is the task's own constraint
//!
//! Every decision about a portal request stays in `alo-portals`: what a grant
//! covers, who is asking, what opens what, and what a refusal says. This crate
//! reads files and services and hands the readings over. If it ever looks as
//! though something here has to *decide*, that is a finding about the split
//! rather than a licence to decide it — the task says so and the header of
//! `alo_portals::serving` says why.
//!
//! So there are no words here. Every sentence a person reads about a portal
//! comes out of `alo-portals`' own string list, in their language; what this
//! crate writes is English for whoever is standing the machine up, in a service
//! log, which is `alo-sessiond`'s arrangement for the same reason.
//!
//! # What it cannot do yet, said here rather than discovered
//!
//! **OpenURI is refused on every machine this runs on**, because nothing in this
//! repository can say what a person has installed.
//! `ThisMachine`'s [`TheMachine::applications`](alo_portals::TheMachine::applications) answers `None` and
//! [`this_machine`]'s header carries the argument at length: an empty list of
//! installed applications would make *what opens this* answer **nothing opens
//! it**, confidently, about a machine with seven applications on it. A refusal
//! the person can read is the better answer, and it is the one
//! `alo_portals::Unanswered::ApplicationsUnread` exists for.
//!
//! **FileChooser is not served at all**, and that is `alo-portals`' deliberate
//! choice rather than this crate's omission: `serving.rs` registers only the
//! portals decided without a dialog, so the bus itself tells an application that
//! nothing answers the rest. A backend running from this crate can therefore
//! hold the name, answer Secret, Settings and NetworkMonitor for real, and still
//! not let an application open a file — which is the second half of task 1's own
//! text and the content of task 6.

pub mod this_machine;

pub use this_machine::ThisMachine;
