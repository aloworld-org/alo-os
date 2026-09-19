//! Telling *the machine could not confirm this is alo OS* apart from *the
//! update could not be prepared* — in the base's own words, because the base
//! gives no other sign.
//!
//! # Why this file exists
//!
//! Until it did, both read as one sentence: *the update could not be prepared,
//! so nothing was changed*. That is true of a download that stopped halfway
//! and true of a build the signature policy refused, and it tells a person
//! neither — at the one moment a machine sold on sovereignty cannot afford to
//! be vague, and about the one refusal they can actually do something with.
//!
//! The two are also not equally interesting. *The download failed* is weather:
//! ask again later. *This machine could not confirm the new version came from
//! alo OS* is the strongest promise the system keeps about itself — it would
//! rather stay as it is — and a person who is never told it never learns that
//! the promise is there.
//!
//! # Read from what it said, because there is nothing else to read
//!
//! The base exits unsuccessfully either way, with the same code
//! ([`crate::NotAnswered::SaidNo`]), so what it said is the only evidence
//! there is. That is a weaker thing to depend on than an exit code and it is
//! written down here as such — see `docs/quirks.md`.
//!
//! Two rules keep it honest:
//!
//! - **Every marker is a whole clause**, never a word. The base prints
//!   *Getting image source signatures* on the way to a **successful** stage,
//!   and a tail of its output carrying that line must not be read as a refusal;
//!   a test below holds this to that exact line.
//! - **What is not recognised is not guessed at.** Anything this cannot place
//!   stays [`crate::NotApplied::TheBaseDidNotStageIt`] and reads as *the update
//!   could not be prepared*, which is true of everything. A refusal invented
//!   from a sentence nobody recognised would be a machine accusing its own
//!   registry of something, on no evidence.
//!
//! # Measured
//!
//! 2026-09-19, on the base `image/Containerfile` pins, with its own
//! `skopeo` 1.22.2 and `containers-common` 0.67.0 — the library its `bootc`
//! stages a build with — against `ghcr.io/aloworld-org/alo-os` under a policy
//! requiring the owner's key:
//!
//! ```text
//! level=fatal msg="Source image rejected: A signature was required, but no signature exists"
//! ```
//!
//! for both `0.0.4`, which nothing at the place vouches for, and `0.0.3`,
//! which `image/pinned.toml` pins. The second of those is a finding for the
//! release process rather than for this file — see
//! `docs/autonomy/updates/an-offer-a-person-can-act-on.md`.

use crate::refusing::NotAnswered;

/// What the base says when a build is refused by the signature policy.
///
/// Whole clauses, each of which `containers/image` — the library the base
/// stages with — produces only when a policy requirement was not met. The
/// first is the umbrella it wraps every such refusal in; the rest are the
/// reasons underneath it, kept separately so that a base which stops printing
/// the umbrella still says something this recognises.
const WHAT_IT_SAYS: [&str; 5] = [
    "source image rejected",
    "a signature was required",
    "none of the signatures were accepted",
    "signature verification failed",
    "invalid signature",
];

/// Whether what the base said is its signature policy refusing the build.
///
/// `false` for everything it does not recognise, which is deliberate: see this
/// file's header.
#[must_use]
pub fn refused_for_its_signature(answer: &NotAnswered) -> bool {
    let said = match answer {
        // It could not be started at all, so no policy was ever consulted.
        NotAnswered::NotStarted { .. } => return false,
        NotAnswered::SaidNo { said, .. } => said.to_lowercase(),
    };
    WHAT_IT_SAYS.iter().any(|clause| said.contains(clause))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the base said, as this crate carries it.
    fn said_no(said: &str) -> NotAnswered {
        NotAnswered::SaidNo {
            program: "bootc".to_owned(),
            code: Some(1),
            said: said.to_owned(),
        }
    }

    /// **The refusal measured on the real base against the real place**, word
    /// for word as it was printed on 2026-09-19.
    #[test]
    fn the_refusal_the_real_base_printed_is_read_as_a_signature_refusal() {
        assert!(refused_for_its_signature(&said_no(
            "time=\"2026-09-19T22:53:00Z\" level=fatal msg=\"Source image rejected: A signature \
             was required, but no signature exists\""
        )));
    }

    /// And the other policy refusals the same library produces, for a build
    /// that has a signature which does not satisfy the policy.
    #[test]
    fn every_way_the_policy_refuses_is_read_as_one() {
        for said in [
            "Source image rejected: None of the signatures were accepted",
            "Source image rejected: Invalid signature when validating ASN.1 encoding",
            "error: signature verification failed",
        ] {
            assert!(refused_for_its_signature(&said_no(said)), "{said}");
        }
    }

    /// **A download that stopped is not a signature refusal**, and neither is
    /// a base that could not be started at all.
    #[test]
    fn a_failure_that_is_not_about_a_signature_is_not_read_as_one() {
        for said in [
            "error: reading blob sha256:abc: unexpected EOF",
            "error: no space left on device",
            "Error: failed to resolve: name unknown",
            "",
        ] {
            assert!(!refused_for_its_signature(&said_no(said)), "{said}");
        }
        assert!(!refused_for_its_signature(&NotAnswered::NotStarted {
            program: "bootc".to_owned(),
            why: "no such file or directory".to_owned(),
        }));
    }

    /// **The line the base prints on the way to a *successful* stage is not a
    /// refusal**, which is the whole reason every marker here is a clause
    /// rather than the word *signature*.
    #[test]
    fn the_line_printed_on_the_way_to_a_successful_stage_is_not_a_refusal() {
        assert!(!refused_for_its_signature(&said_no(
            "Getting image source signatures\nCopying blob sha256:26cdc51f\nerror: no space left \
             on device"
        )));
    }
}
