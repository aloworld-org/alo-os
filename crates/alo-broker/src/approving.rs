//! The token a turn issues when a person approves a system verb, and how the
//! broker tells a genuine one from anything else.
//!
//! The plan: *only for a verb a person approved in a turn, which it verifies by
//! a token the turn issues rather than trusting the asker.* A [`Token`] is
//! that: the approval's number, the moment it was issued, and an HMAC-SHA-256
//! **over the exact verb and argument** under a key the broker holds. So a
//! token is worth exactly one request — this verb, this argument, this approval
//! — and nothing about the request can be changed without the proof failing.
//!
//! # What it proves, and what it does not
//!
//! It proves the request was issued by whoever holds the [`ApprovingKey`], for
//! these exact words, at that moment. Whoever holds the key is the turn in
//! `alo-agentd`, which issues a token only when `alo_capability::Approved`
//! is redeemed — and **the agent is not that**: an agent is a login of its own
//! (`image/usr/lib/sysusers.d/alo.conf`), it reaches `alo-agentd` through its
//! own door, and it can neither reach the broker's door nor read the turn's
//! memory. The model is assumed to be saying whatever an attacker wants (ADR
//! 0001); nothing it says becomes a token.
//!
//! What it cannot prove is which program of the person's own login asked,
//! because on this image `alo-agentd` **is** the person's login (ADR 0001 §2)
//! and the kernel cannot tell it from another program that person runs. Law 2
//! binds the agent and never the person, and a person changing their own
//! printer by hand is ADR 0009 rather than an escape — the report for this task
//! says so in full.
//!
//! # Once, and not for long
//!
//! A genuine token is spent by being used (`crate::spent`) and is refused
//! once it is older than [`LIFETIME`]. An approval is never a session (ADR 0001
//! §5): the token is issued at the moment the person's approval is spent and
//! crosses the door straight away, so a minute is generous rather than a window
//! anybody should need.

use std::time::{Duration, SystemTime};

use ring::hmac;

use crate::verbs::SystemVerb;

/// How long a token is good for after it is issued.
pub const LIFETIME: Duration = Duration::from_secs(60);

/// What a proof is made under, so no other HMAC this machine computes with a
/// key of the same bytes can ever be mistaken for one.
const WHAT_THIS_PROVES: &[u8] = b"alo-broker approval 1\0";

/// How many bytes a proof is.
const PROOF_BYTES: usize = 32;

/// The key a turn's approvals are proven with.
///
/// Deliberately not `Debug`, not `Clone` and without any way to read the bytes
/// back out: a key that can be printed is a key that ends up in a log.
pub struct ApprovingKey(hmac::Key);

/// There was no randomness to make a key from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoRandomness(pub String);

impl std::fmt::Display for NoRandomness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the kernel gave no randomness to make the broker's approving key from: {}",
            self.0
        )
    }
}

impl std::error::Error for NoRandomness {}

impl ApprovingKey {
    /// A new key, from the kernel's randomness.
    ///
    /// # Errors
    /// [`NoRandomness`] when the kernel would not give any. There is no
    /// fallback: a key from anywhere else is a key somebody could guess.
    pub fn fresh() -> Result<Self, NoRandomness> {
        let mut bytes = [0_u8; PROOF_BYTES];
        getrandom::fill(&mut bytes).map_err(|why| NoRandomness(why.to_string()))?;
        Ok(Self::of(&bytes))
    }

    /// A key made of these bytes — how the side that issues tokens and the
    /// side that verifies them come to hold the same one.
    #[must_use]
    pub fn of(bytes: &[u8; PROOF_BYTES]) -> Self {
        Self(hmac::Key::new(hmac::HMAC_SHA256, bytes))
    }

    /// Issue the token for one approval of exactly this verb.
    ///
    /// `approval` is the turn's own number for the approval that was spent —
    /// `alo_capability::ProposalId` — so the broker's entry and the turn's entry
    /// for one moment carry the same number.
    #[must_use]
    pub fn issue(&self, verb: &SystemVerb, approval: u64, at: SystemTime) -> Token {
        let issued = seconds(at);
        let mut proof = [0; PROOF_BYTES];
        proof.copy_from_slice(hmac::sign(&self.0, &proven(verb, approval, issued)).as_ref());
        Token {
            approval,
            issued,
            proof,
        }
    }

    /// Whether this token was issued under this key for exactly this verb.
    ///
    /// Compared in constant time, by `ring`.
    #[must_use]
    pub fn issued(&self, verb: &SystemVerb, token: &Token) -> bool {
        hmac::verify(
            &self.0,
            &proven(verb, token.approval, token.issued),
            &token.proof,
        )
        .is_ok()
    }
}

/// One approval of one verb, as it crosses the door.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    /// The turn's number for the approval.
    approval: u64,
    /// When it was issued, in whole seconds since the epoch.
    issued: u64,
    /// The proof.
    proof: [u8; PROOF_BYTES],
}

impl Token {
    /// The turn's number for the approval this token claims.
    ///
    /// A claim until [`ApprovingKey::issued`] says otherwise.
    #[must_use]
    pub const fn approval(&self) -> u64 {
        self.approval
    }

    /// When it was issued, in whole seconds since the epoch.
    #[must_use]
    pub const fn issued(&self) -> u64 {
        self.issued
    }

    /// The proof's bytes, which is what a token is remembered by once spent.
    #[must_use]
    pub const fn proof(&self) -> &[u8; PROOF_BYTES] {
        &self.proof
    }

    /// A token from its three parts as the door wrote them, or nothing.
    ///
    /// Numbers are decimal with no sign and no leading zero; the proof is
    /// sixty-four lowercase hexadecimal characters. One spelling each, for the
    /// reason `crate::arguments` gives.
    #[must_use]
    pub fn read(approval: &str, issued: &str, proof: &str) -> Option<Self> {
        let mut bytes = [0; PROOF_BYTES];
        crate::hex::read_into(proof, &mut bytes).then_some(())?;
        Some(Self {
            approval: number(approval)?,
            issued: number(issued)?,
            proof: bytes,
        })
    }

    /// The three parts, as the door writes them.
    #[must_use]
    pub fn written(&self) -> String {
        format!(
            "{} {} {}",
            self.approval,
            self.issued,
            crate::hex::written(&self.proof)
        )
    }
}

/// A number in its one spelling.
fn number(written: &str) -> Option<u64> {
    let canonical = written.bytes().all(|byte| byte.is_ascii_digit())
        && !written.is_empty()
        && (written == "0" || !written.starts_with('0'));
    canonical.then(|| written.parse().ok()).flatten()
}

/// Whole seconds since the epoch; a moment before it is the epoch, which is a
/// token long lapsed rather than a failure.
pub fn seconds(at: SystemTime) -> u64 {
    at.duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// The bytes a proof is made over.
fn proven(verb: &SystemVerb, approval: u64, issued: u64) -> Vec<u8> {
    [
        WHAT_THIS_PROVES,
        verb.name().as_bytes(),
        b"\0",
        &verb.argument().proven_as(),
        &approval.to_be_bytes(),
        &issued.to_be_bytes(),
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arguments::{Identity, Switch};

    /// Noon on a day in 2025.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A key everybody in these tests shares.
    fn key() -> ApprovingKey {
        ApprovingKey::of(&[7; PROOF_BYTES])
    }

    /// Something reported.
    fn a_printer() -> Identity {
        Identity::of_what_was_reported(b"a printer")
    }

    /// **A token proves the verb it was issued for**, and reads back from what
    /// the door writes.
    #[test]
    fn a_token_proves_its_own_verb_and_reads_back_as_written() {
        let verb = SystemVerb::AddPrinter(a_printer());
        let token = key().issue(&verb, 3, noon());
        assert!(key().issued(&verb, &token));
        let written = token.written();
        let parts: Vec<&str> = written.split(' ').collect();
        let [approval, issued, proof] = parts.as_slice() else {
            return assert_eq!(parts.len(), 3);
        };
        assert_eq!(Token::read(approval, issued, proof), Some(token));
    }

    /// **And nothing else**: another verb with the same argument, the same verb
    /// with another argument, another approval, another moment, another key.
    #[test]
    fn a_token_proves_nothing_but_its_own_verb() {
        let verb = SystemVerb::AddPrinter(a_printer());
        let token = key().issue(&verb, 3, noon());

        let other = Identity::of_what_was_reported(b"another printer");
        assert!(!key().issued(&SystemVerb::RemovePrinter(a_printer()), &token));
        assert!(!key().issued(&SystemVerb::AddPrinter(other), &token));
        assert!(!ApprovingKey::of(&[8; PROOF_BYTES]).issued(&verb, &token));

        let renumbered = Token {
            approval: 4,
            ..token
        };
        let redated = Token {
            issued: token.issued + 1,
            ..token
        };
        assert!(!key().issued(&verb, &renumbered));
        assert!(!key().issued(&verb, &redated));

        let radio_on = key().issue(&SystemVerb::SetRadio(Switch::On), 3, noon());
        assert!(!key().issued(&SystemVerb::SetRadio(Switch::Off), &radio_on));
    }

    /// Two fresh keys are two keys.
    #[test]
    fn a_fresh_key_is_nobody_elses() {
        let (Ok(one), Ok(two)) = (ApprovingKey::fresh(), ApprovingKey::fresh()) else {
            return assert!(ApprovingKey::fresh().is_ok());
        };
        let verb = SystemVerb::SetRadio(Switch::On);
        assert!(!two.issued(&verb, &one.issue(&verb, 0, noon())));
    }

    /// Numbers have one spelling.
    #[test]
    fn a_number_has_one_spelling() {
        assert_eq!(number("0"), Some(0));
        assert_eq!(number("42"), Some(42));
        for written in ["", "042", "+42", "-1", "4 2", "18446744073709551616", "٤"] {
            assert_eq!(number(written), None, "{written}");
        }
    }
}
