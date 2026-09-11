//! What the image owes the three processes, and whether the files it ships
//! agree.
//!
//! alo OS is an OCI image ([ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)),
//! and `image/` is what that image adds to a rented base: three binaries of our
//! own, four systemd units, two directories made at boot, the logins the
//! machine has, the description `alo-agentd` reads, and — since
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! — the pinned model runtime, the weights and the one unit that serves them.
//! This crate reads those declarations — and the
//! accounts file the image is answerable for **not** shipping — and asks
//! whether they can all be true at once.
//!
//! | | |
//! |---|---|
//! | [`Image`] | The image's files, read off a directory |
//! | [`everything_wrong_with`] | Every promise in `docs/` they make to each other, checked |
//! | [`Wrong`] | One thing they disagree about, and which decision it is about |
//! | [`Unit`], [`Service`] | A systemd unit as text, and the settings alo OS asks about |
//! | [`THE_LOADER`], [`THE_AGENT`], [`THE_OPENER`], [`THE_SERVER`] | The four units a machine starts, as systemd names them |
//! | [`Made`], [`Declared`], [`Description`] | What is made at boot, who the machine's logins are, and what it says about itself |
//! | [`TheStore`], [`where_a_sign_in_looks`] | The accounts a sign-in reads, which an image must not ship |
//! | [`TheRuntime`] | The model runtime the recipe carries, and whether it is pinned |
//! | [`TheWeights`] | The weights a machine arrives with, and whether the catalogue measured them |
//! | [`TheDisk`], [`TheDocument`] | The disk a machine boots from, as the recipe declares it and as `docs/booting.md` tells a person to make it |
//!
//! # Nothing on a machine ever reads this
//!
//! It is `alo-driving`'s shape one crate on: that crate measures whether a model
//! can produce a verb call, it is run by whoever adds a catalogue entry, and
//! what ships is the grade they wrote down. This is run by whoever changes the
//! image, and what ships is the image. There is no `Image` in a booted alo OS,
//! nothing links against this. The crates it reaches are each reached for one
//! answer it would otherwise spell out a second time: [`alo_keeping::Keeping`],
//! because the one thing an image may say about retention is *everything*;
//! `alo-entering`, because what a session hands a daemon is derived from a
//! sign-in rather than written into a checker; `alo-accounts`, for where a
//! sign-in looks for the accounts this machine has; `alo-sessiond`, for
//! where the one thing that turns a correct password into a session opens its
//! door; and `alo-models`, for the catalogue's own measurement of the weights
//! and for the address a machine looks for a runtime at — which
//! [ADR 0019](../../../docs/decisions/0019-a-runtime-is-found-not-configured.md)
//! keeps in one file, so a checker holding a unit to it asks that file rather
//! than spelling a second copy.
//!
//! # What it is for, said plainly: a build cannot catch any of this
//!
//! `docker build` produces an image whose machine description names a login the
//! image never creates. It produces one whose loader runs in root's group, so
//! the map of turns is pinned where no daemon can write it. It produces one
//! where somebody added a capability to `alo-agentd.service` to make something
//! work — which is
//! [ADR 0018](../../../docs/decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
//! undone in one line, in the file nobody reviews, on a system whose whole claim
//! is that the service talking to your agent holds nothing. It produces one
//! whose sign-in opener has been given a capability, or whose door is handed to
//! the agent's group — which is
//! [ADR 0024](../../../docs/decisions/0024-what-a-person-signs-in-at.md)'s
//! price for a second privileged component quietly stopping being paid.
//!
//! Every one of those is a green build and a machine that is wrong. They are
//! tests here, which is `CLAUDE.md`'s rule about promises in `docs/` applied to
//! the one part of alo OS that is not Rust.
//!
//! # And what it is deliberately not
//!
//! **It does not say the image boots.** That is the machine half of
//! `ROADMAP.md`'s image line, it needs a machine this repository does not have,
//! and *an image that builds is not an image that boots* is the sentence the
//! whole item was written around. Nothing here may ever be read as having
//! answered it — and that is true of [`TheDisk`] too: a recipe that declares
//! how it becomes a disk, and a document that says how to write one, are not a
//! disk anybody has watched come up.
//!
//! It is also **not a second reader of the machine description**'s rules;
//! `crate::description` says at length why it reads that file at all and what it
//! leaves to `alo-agentd`.

mod accounts;
mod booting;
mod checking;
mod description;
mod disk;
mod image;
mod logins;
mod making;
mod recipe;
mod refusing;
mod runtime;
mod service;
#[cfg(test)]
mod testing;
mod unit;
mod weights;
mod wrong;

pub use accounts::{TheStore, where_a_sign_in_looks};
pub use booting::TheDocument;
pub use checking::{THE_DOOR, everything_wrong_with};
pub use description::{Description, THE_DESCRIPTION, THE_FORMAT};
pub use disk::{NO_PARTITIONER, THE_ONLY_TOOL, TheDisk};
pub use image::{Image, THE_AGENT, THE_LOADER, THE_OPENER, THE_SERVER};
pub use logins::{Declared, every_login};
pub use making::{A_DIRECTORY, Made, everything_made};
pub use refusing::{NotAService, NotAUnit, NotAnImage, NotDeclared, NotDescribed, NotMade};
pub use runtime::{THE_RUNTIMES_BINARY, THE_RUNTIMES_LIBRARIES, TheRuntime};
pub use service::{ROOT, Service};
pub use unit::Unit;
pub use weights::{THE_WEIGHTS, TheWeights};
pub use wrong::Wrong;

/// Where alo OS's own image is, in this repository.
///
/// An absolute path built from the crate's own directory, because a test's
/// working directory is the package root and an integration test's is too — but
/// neither is something to rely on, and a check that silently found no image
/// would be a check that silently passed.
pub const THE_IMAGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../image");
