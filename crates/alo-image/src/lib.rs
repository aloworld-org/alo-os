//! What the image owes the two daemons, and whether the files it ships agree.
//!
//! alo OS is an OCI image ([ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)),
//! and `image/` is what that image adds to a rented base: two binaries, two
//! systemd units, two directories made at boot, the logins the machine has, and
//! the description `alo-agentd` reads. This crate reads those five declarations
//! and asks whether they can all be true at once.
//!
//! | | |
//! |---|---|
//! | [`Image`] | The five files, read off a directory |
//! | [`everything_wrong_with`] | Every promise in `docs/` they make to each other, checked |
//! | [`Wrong`] | One thing they disagree about, and which decision it is about |
//! | [`Unit`], [`Service`] | A systemd unit as text, and the seven settings alo OS asks about |
//! | [`Made`], [`Declared`], [`Description`] | What is made at boot, who the machine's logins are, and what it says about itself |
//!
//! # Nothing on a machine ever reads this
//!
//! It is `alo-driving`'s shape one crate on: that crate measures whether a model
//! can produce a verb call, it is run by whoever adds a catalogue entry, and
//! what ships is the grade they wrote down. This is run by whoever changes the
//! image, and what ships is the image. There is no `Image` in a booted alo OS,
//! nothing links against this, and `alo-keeping` is the only crate it reaches —
//! for [`alo_keeping::Keeping`], because the one thing an image may say about
//! retention is *everything*, and asking the crate that owns that rule is one
//! answer rather than a second spelling of it here.
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
//! is that the service talking to your agent holds nothing.
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
//! answered it.
//!
//! It is also **not a second reader of the machine description**'s rules;
//! `crate::description` says at length why it reads that file at all and what it
//! leaves to `alo-agentd`.

mod checking;
mod description;
mod image;
mod logins;
mod making;
mod refusing;
mod service;
#[cfg(test)]
mod testing;
mod unit;
mod wrong;

pub use checking::{THE_DOOR, everything_wrong_with};
pub use description::{Description, THE_DESCRIPTION, THE_FORMAT};
pub use image::{Image, THE_AGENT, THE_LOADER};
pub use logins::{Declared, every_login};
pub use making::{A_DIRECTORY, Made, everything_made};
pub use refusing::{NotAService, NotAUnit, NotAnImage, NotDeclared, NotDescribed, NotMade};
pub use service::{ROOT, Service};
pub use unit::Unit;
pub use wrong::Wrong;

/// Where alo OS's own image is, in this repository.
///
/// An absolute path built from the crate's own directory, because a test's
/// working directory is the package root and an integration test's is too — but
/// neither is something to rely on, and a check that silently found no image
/// would be a check that silently passed.
pub const THE_IMAGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../image");
