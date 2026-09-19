//! Whether the place has vouched for the build it offers — read out of the
//! names it already answered with, and never by asking it a third question.
//!
//! `alo_keeping_up::Vouching` is the decision; this is the one fact about
//! **places** it needs. A place that holds a build also holds, beside it, the
//! signature somebody made over that build — under a name derived from the
//! build itself. So the answer is already in the list [`crate::ThePlace`]
//! fetched for the first question, and finding it costs nothing that leaves
//! this machine.
//!
//! # Which name counts, and why only that one
//!
//! **`sha256-<the build>.sig`, and nothing else.** That is the name the
//! machine's own signature policy looks under: `containers/image` — the
//! library the base's `bootc` stages a build with — fetches a sigstore
//! signature as an *attachment* beside the build, under the build's own name
//! with `.sig` after it.
//!
//! A place may well hold other things named after a build. The one that
//! matters here is the one **this machine's base would read**, because the
//! whole point of the question is *would this machine accept it*, and a name
//! nothing on this machine reads is not an answer to that.
//!
//! **Measured, 2026-09-19**, which is why this file is written this way round
//! rather than the obvious one. The place this repository pins holds, beside
//! each signed release, a name `sha256-<the build>` with **no** `.sig` — the
//! fallback tag of an OCI 1.1 referrer, which is what `cosign` 3 writes by
//! default and which holds a real sigstore bundle. Read as *vouched for*, this
//! machine would have said so about the pinned release — and the base's own
//! tooling, out of the image `image/Containerfile` pins, run against that
//! exact build under a policy requiring the owner's own key, answered *Source
//! image rejected: A signature was required, but no signature exists*.
//!
//! That is the failure this whole task exists to end, arriving through the
//! back door: a machine telling a person an update is ready on evidence its
//! own base does not read. So the name counted is the name the base reads, and
//! the release process is where the two are brought back together — a finding
//! for the lane that owns it
//! ([ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)),
//! never something repaired here by lowering what counts.
//!
//! # What this is not
//!
//! It does not fetch the signature, read it, or check it against a key.
//! Nothing here verifies anything, and no answer from this file permits
//! anything: whether a build may be staged is the machine's signature policy's
//! answer, given by the base at the moment of staging, under
//! `--enforce-container-sigpolicy`, which nothing in this workspace may weaken.

use alo_keeping_up::{Digest, Vouching};

/// What separates a build's algorithm from its value in a name a place can
/// hold. A name cannot carry the `:` a build is written with.
const IN_A_NAME: char = '-';

/// What a signature is called, after the build it is over.
const A_SIGNATURE: &str = ".sig";

/// The name a signature for `build` is held under at a place.
///
/// `sha256:ab…` is `sha256-ab….sig`. Public so that a test — and anybody
/// reading a place's names by hand — writes the same name this does rather
/// than a second spelling of it.
#[must_use]
pub fn the_name_vouching_for(build: &Digest) -> String {
    format!(
        "{}{A_SIGNATURE}",
        build.as_str().replacen(':', &IN_A_NAME.to_string(), 1)
    )
}

/// Whether `names` — everything the place answered it holds — vouches for
/// `build`.
///
/// Missing is read as [`Vouching::NobodyHasVouchedForIt`], which is the only
/// safe direction: the sentence a person reads for it is cautious, and the
/// sentence they would read for the other one is a promise.
#[must_use]
pub fn vouched_for(build: &Digest, names: &[String]) -> Vouching {
    let vouching = the_name_vouching_for(build);
    if names.iter().any(|name| name == &vouching) {
        Vouching::ThePlaceVouchesForIt
    } else {
        Vouching::NobodyHasVouchedForIt
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::build;

    /// Every name in one list, as a place hands them over.
    fn holding(names: &[&str]) -> Vec<String> {
        names.iter().map(|&name| name.to_owned()).collect()
    }

    /// The name is the build's own, with what a place cannot hold replaced and
    /// what a signature is called after it.
    #[test]
    fn a_signature_is_named_after_the_build_it_is_over() {
        let name = the_name_vouching_for(&build("aa"));
        assert_eq!(name, format!("sha256-{}.sig", "aa".repeat(32)));
        assert!(!name.contains(':'), "{name}");
    }

    /// **A place holding the signature vouches for the build.**
    #[test]
    fn a_place_holding_the_signature_vouches_for_the_build() {
        let names = holding(&["0.0.2", &the_name_vouching_for(&build("bb")), "0.0.3"]);
        assert_eq!(
            vouched_for(&build("bb"), &names),
            Vouching::ThePlaceVouchesForIt
        );
    }

    /// **A place holding nothing for it does not**, and neither does one
    /// holding a signature for some other build.
    #[test]
    fn a_place_holding_nothing_for_it_vouches_for_nothing() {
        for names in [
            holding(&["0.0.3"]),
            holding(&[]),
            holding(&[&the_name_vouching_for(&build("cc"))]),
        ] {
            assert_eq!(
                vouched_for(&build("bb"), &names),
                Vouching::NobodyHasVouchedForIt,
                "{names:?}"
            );
        }
    }

    /// **The name without `.sig` after it does not vouch for anything** — the
    /// measurement in this file's header, as a test.
    ///
    /// It is the name this repository's own registry holds today, it names a
    /// real signature, and the base refuses the build anyway. A machine that
    /// counted it would tell a person an update is ready on evidence its own
    /// base does not read.
    #[test]
    fn the_name_this_machines_base_does_not_read_vouches_for_nothing() {
        let referrer = the_name_vouching_for(&build("bb")).replace(A_SIGNATURE, "");
        assert_eq!(referrer, format!("sha256-{}", "bb".repeat(32)));
        assert_eq!(
            vouched_for(&build("bb"), &holding(&[&referrer])),
            Vouching::NobodyHasVouchedForIt
        );
    }

    /// And the whole of the real place's answer on the day this was written,
    /// held to what the base said about the same two builds.
    ///
    /// The names are the ones `tests/against_the_real_registry.rs` read from
    /// the place this repository pins; the build is `0.0.3`'s, which
    /// `image/pinned.toml` pinned on the day this was written.
    #[test]
    fn neither_release_the_real_place_held_was_vouched_for_on_the_day_this_was_written() {
        let names = holding(&[
            "0.0.1",
            "sha256-d3f05b60975edcff51a44c1f21e764a32b286677e306ba24631bad6a00b6a13c",
            "0.0.2",
            "sha256-8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9",
            "0.0.3",
            "sha256-41d43c7ea491990eea602493e5c645bd7bf8e7d0d9d7a8e9a000bc895a9dd0d7",
            "0.0.4",
        ]);
        let pinned =
            Digest::read("sha256:41d43c7ea491990eea602493e5c645bd7bf8e7d0d9d7a8e9a000bc895a9dd0d7")
                .unwrap();
        assert_eq!(
            vouched_for(&pinned, &names),
            Vouching::NobodyHasVouchedForIt,
            "the pinned release was read as vouched for, and the base refuses it"
        );
    }
}
