//! The statement as bytes, rendered the same way by anybody, and its digest.
//!
//! This is the half a signature commits to, so everything here is chosen for
//! being reproducible rather than for reading nicely.
//!
//! # Nothing here is `Debug`
//!
//! A `Debug` rendering is a convenience the compiler may change between releases
//! and nobody promised to keep. Putting one inside a signed artifact would mean a
//! toolchain upgrade silently invalidating every attestation ever made — the same
//! class as rewording a sentence, with nobody to blame and no diff to find. So
//! every value is written out by this file, by hand, and the tests below hold it
//! to exactly the bytes.
//!
//! # Times are seconds since the epoch
//!
//! Not a date. A date needs a timezone and a calendar, and two readers who supply
//! different ones get different bytes over the same instant — the period problem
//! again, one layer down. Seconds are unambiguous, and a reader who wants a date
//! can convert one; a reader who wants to check a digest cannot undo a conversion
//! somebody already made.

use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use alo_egress::{Destination, Errand, Why};
use alo_models::Region;
use sha2::{Digest as _, Sha256};

use crate::{HeldBy, Statement};

/// The shape of a rendered statement.
///
/// **In the bytes from the first version, and it cannot be added later**, because
/// the artefacts that would need it are the ones already signed. A signature
/// commits to bytes; the day a sentence below is reworded, every attestation made
/// before it stops verifying against a fresh render of the same record — not
/// wrong and not tampered, but *unverifiable*, which an auditor cannot tell apart
/// from tampered. This number turns that into *render it the way version 1
/// rendered it*.
pub const THE_FORMAT: u32 = 1;

/// A moment, as seconds since the epoch.
///
/// A time before the epoch cannot be rendered and says so rather than wrapping to
/// something enormous: a statement with a nonsense instant in it is worse than one
/// that refuses, because the nonsense is signed.
fn moment(at: SystemTime) -> String {
    at.duration_since(UNIX_EPOCH).map_or_else(
        |_| "before-the-epoch".to_owned(),
        |since| since.as_secs().to_string(),
    )
}

/// Where something went, written out.
fn destination(destination: &Destination) -> String {
    match destination {
        Destination::PairedMachine { machine } => format!("paired-machine {machine}"),
        Destination::Provider { provider, region } => {
            format!("provider {provider} in {}", region_named(region))
        }
        Destination::Address { host } => format!("address {host}"),
    }
}

/// A region, written out rather than debugged.
///
/// **`Unknown` is rendered as a statement and not as a blank.** `alo-models` says
/// of it: *the provider has not said. Not a synonym for "probably fine".* An
/// auditor reading an attestation needs to see the difference between a provider
/// that declared where it runs and one that did not, and an empty field beside a
/// provider's name reads as the former.
fn region_named(region: &Region) -> String {
    match region {
        Region::Declared(said) => said.clone(),
        Region::Unknown => "a-region-the-provider-did-not-declare".to_owned(),
    }
}

/// Why something left, written out.
fn why(why: Why) -> &'static str {
    match why {
        Why::Asking => "asking",
        Why::Fetching => "fetching",
        Why::Sending => "sending",
    }
}

/// Which errand it was, written out.
///
/// **Exhaustive on purpose, with no wildcard.** A new errand is a new way this
/// machine reaches the network on its own, and adding one must not be able to
/// slip into a catch-all that renders every unfamiliar errand alike. Failing to
/// compile is the right cost: somebody adding an errand has to give it a stable
/// name here, deliberately, because that name goes into bytes somebody signs.
/// The compiler caught this file naming four of the seven.
fn errand(errand: Errand) -> &'static str {
    match errand {
        Errand::SigningIn => "signing-in",
        Errand::FetchingAModel => "fetching-a-model",
        Errand::CheckingForAnUpdate => "checking-for-an-update",
        Errand::FetchingAnUpdate => "fetching-an-update",
        Errand::InstallingAnApplication => "installing-an-application",
        Errand::CheckingForApplicationUpdates => "checking-for-application-updates",
        Errand::UpdatingAnApplication => "updating-an-application",
    }
}

/// The statement, as the bytes a signature would cover.
///
/// Every line is `key value`, one per line, in the order written here. The
/// counts come before the lists so that a reader who checks only the arithmetic
/// does not have to read to the end to find it.
#[must_use]
pub fn rendered(statement: &Statement, held_by: &HeldBy) -> String {
    let mut out = String::new();

    // The format first, so a reader knows which rules the rest was written under
    // before reading any of it.
    let _ = writeln!(out, "alo-egress-attestation {THE_FORMAT}");
    let _ = writeln!(out, "from {}", moment(statement.period().from()));
    let _ = writeln!(out, "until {}", moment(statement.period().until()));
    let _ = writeln!(out, "from-is-included yes");
    let _ = writeln!(out, "until-is-included no");

    // **What holds the signing key, inside the bytes it will sign.** Kept beside
    // the artifact it could be dropped, edited or lost and the attestation would
    // still verify — a document whose strength claim can be stripped without
    // breaking its signature overstates itself by default. In here, removing it
    // breaks the signature. It says which kind was used and never that it was
    // enough: whoever checks an attestation decides what they accept.
    let _ = writeln!(out, "held-by {}", held_by.named());

    let _ = writeln!(out, "departures {}", statement.departures().len());
    let _ = writeln!(out, "held-back {}", statement.held_back().len());
    let _ = writeln!(out, "on-its-own {}", statement.on_its_own().len());
    let _ = writeln!(out, "answered-here {}", statement.answered_here());
    let _ = writeln!(out, "inference-egress {}", statement.inference_egress());
    let _ = writeln!(
        out,
        "entries-outside-this-period {}",
        statement.entries_outside()
    );

    // **No grantee is named in these lines, and that is the decision rather than
    // an omission.** `Statement` carries the grantee, because a person asking for
    // their own record is entitled to it — but a machine's attestation is a
    // document built to be handed to a third party, and naming who was working is
    // a different question from saying what left. The owner decided the person
    // chooses, each for themselves, and **off until they do**; a format that
    // named everybody by default would be that decision inverted.
    //
    // Opting in is not built here on purpose. Done properly it needs a per-person
    // setting nobody else can set for you, an aggregate for the people who did not
    // choose — *four more, from people who have not chosen to be named*, with no
    // per-person gaps, because a gap where Bob was names Bob by elimination on a
    // two-person machine — and a sentence at the point of choosing saying it works
    // **forwards only**, since a signed artifact cannot be reached back into. That
    // is a change with its own vocabulary and its own consent, not a field to add
    // while nobody is looking.
    for one in statement.departures() {
        let _ = writeln!(
            out,
            "left {} {} {}",
            moment(one.at),
            why(one.why),
            destination(&one.destination)
        );
    }
    for one in statement.held_back() {
        let _ = writeln!(
            out,
            "held-back {} {} {} {}",
            moment(one.at),
            why(one.why),
            destination(&one.destination),
            one.refused
        );
    }
    for one in statement.on_its_own() {
        let _ = writeln!(
            out,
            "on-its-own {} {} {}",
            moment(one.at),
            errand(one.errand),
            destination(&one.destination)
        );
    }

    out
}

/// The digest of a rendered statement, as lower-case hexadecimal.
///
/// SHA-256 because that is what this repository already signs by —
/// [ADR 0036](../../../docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md),
/// *the owner signs by digest*. It is derived from the existing arrangement
/// rather than chosen here: **who signs an attestation is undecided**, and if
/// that decision names a different algorithm, [`THE_FORMAT`] is what lets the
/// change be made without stranding what was signed before it.
#[must_use]
pub fn digest_of(rendered: &str) -> String {
    format!("{:x}", Sha256::digest(rendered.as_bytes()))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use alo_record::{Entry, Record};

    use super::*;
    use crate::Period;

    /// What the tests say holds the key. Any kind renders; these tests are
    /// about the bytes and not about which protection is acceptable.
    fn a_key() -> HeldBy {
        HeldBy::ASecurityChip
    }

    /// A moment, for readability.
    fn at(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    /// A statement over a period with nothing in it.
    fn quiet() -> Statement {
        let period = Period::of(at(1_000), at(2_000)).expect("a period");
        Statement::of(period, [].iter())
    }

    /// **The bytes are held to exactly this**, because a signature covers them and
    /// a test that only checked they parse would let every one of them change.
    #[test]
    fn a_quiet_period_renders_these_exact_bytes() {
        assert_eq!(
            rendered(&quiet(), &a_key()),
            "alo-egress-attestation 1\n\
             from 1000\n\
             until 2000\n\
             from-is-included yes\n\
             until-is-included no\n\
             held-by a-security-chip\n\
             departures 0\n\
             held-back 0\n\
             on-its-own 0\n\
             answered-here 0\n\
             inference-egress 0\n\
             entries-outside-this-period 0\n"
        );
    }

    /// **What holds the key is inside the bytes**, so it cannot be dropped from
    /// the artifact without breaking the signature over it.
    #[test]
    fn what_holds_the_key_is_in_the_signed_bytes() {
        for (kind, named) in [
            (HeldBy::ASecurityChip, "held-by a-security-chip"),
            (HeldBy::AFileOnDisk, "held-by a-file-on-disk"),
            (
                HeldBy::SomethingElse("a smartcard".to_owned()),
                "held-by something-else a smartcard",
            ),
        ] {
            let said = rendered(&quiet(), &kind);
            assert!(said.contains(named), "{named} is not in:\n{said}");
        }
    }

    /// **And a different protection is a different digest**, which is what stops a
    /// statement signed from a file on disk being presented as one from a chip.
    #[test]
    fn the_same_period_held_by_different_things_does_not_share_a_digest() {
        assert_ne!(
            digest_of(&rendered(&quiet(), &HeldBy::ASecurityChip)),
            digest_of(&rendered(&quiet(), &HeldBy::AFileOnDisk))
        );
    }

    /// **The format number is the first thing in it**, so a reader knows which
    /// rules the rest was written under before reading any of it.
    #[test]
    fn the_format_number_is_the_first_thing_a_reader_meets() {
        let said = rendered(&quiet(), &a_key());
        let first = said.lines().next().expect("a rendering has a first line");
        assert_eq!(first, format!("alo-egress-attestation {THE_FORMAT}"));
    }

    /// The same statement rendered twice is the same bytes, and the same bytes are
    /// the same digest. Without this the artifact settles nothing.
    #[test]
    fn rendering_is_the_same_every_time_and_so_is_the_digest() {
        let once = rendered(&quiet(), &a_key());
        let again = rendered(&quiet(), &a_key());
        assert_eq!(once, again);
        assert_eq!(digest_of(&once), digest_of(&again));
        assert_eq!(digest_of(&once).len(), 64, "SHA-256 in hexadecimal");
    }

    /// **A different period is a different digest**, which is what stops one
    /// signature standing for a statement about another stretch of time.
    #[test]
    fn a_statement_about_another_period_does_not_share_its_digest() {
        let other = Period::of(at(2_000), at(3_000)).expect("a period");
        let other = Statement::of(other, [].iter());
        assert_ne!(
            digest_of(&rendered(&quiet(), &a_key())),
            digest_of(&rendered(&other, &a_key()))
        );
    }

    /// An entry outside the period is counted as outside and changes the bytes,
    /// so *nothing happened* and *nothing was looked at* cannot render alike.
    #[test]
    fn an_entry_the_period_does_not_cover_is_said_rather_than_dropped() {
        let mut record = Record::default();
        // Bound rather than inlined: a `&Grantee::named(..)` temporary is dropped
        // before the call that borrows it, which cost an E0716 earlier in this
        // repository.
        let who = alo_capability::Grantee::named("somebody");
        record.keep(Entry::answered_here(&who, at(5_000)));
        let period = Period::of(at(1_000), at(2_000)).expect("a period");
        let statement = Statement::of(period, record.everything());
        assert_eq!(statement.entries_outside(), 1);
        assert!(
            rendered(&statement, &a_key()).contains("entries-outside-this-period 1"),
            "the statement must say how much it looked past"
        );
        assert_ne!(
            digest_of(&rendered(&statement, &a_key())),
            digest_of(&rendered(&quiet(), &a_key()))
        );
    }
}
