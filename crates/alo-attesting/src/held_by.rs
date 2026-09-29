//! What holds the key this statement will be signed with, named in the signed
//! bytes.
//!
//! # Which kind, not whether
//!
//! *Is the key in hardware* is a yes-or-no, and the real answer is not one. A
//! discrete security chip, a secure element inside the processor, the operating
//! system's keystore and a file on disk are four different promises, and a
//! verifier that treats them alike is a verifier that accepts the weakest while
//! believing it accepted the strongest. So the artifact **names which**, and the
//! list can grow without the old names changing meaning.
//!
//! # Inside the bytes, so it cannot be removed quietly
//!
//! This is rendered into the statement that gets signed. A field kept beside the
//! artifact could be dropped, edited or lost in transit, and the attestation would
//! still verify — a document whose strength claim can be stripped without breaking
//! its signature is a document that overstates itself by default. Inside the bytes,
//! removing it breaks the signature, which is the behaviour a reader expects from
//! the rest of the artifact.
//!
//! # It says what was used, never that it was enough
//!
//! **Whoever checks an attestation decides what they accept**, and that is where
//! the decision belongs: a regulator, a customer's security team and a person
//! curious about their own laptop are not owed the same bar. This crate makes no
//! judgement, sets no minimum and refuses nothing — it reports.
//!
//! The corollary is that something has to *read* this. A field nothing checks is a
//! field that reassures without doing anything, which is the shape this repository
//! spent 2026-09-29 removing from its own gates. Whoever builds the verifier owes
//! it a rule about these names.
//!
//! # The machine chooses, and the person is told
//!
//! Which of these a machine uses is not a setting somebody has to go and find. The
//! machine picks the best it has, by itself — and **says so at setup**, so a person
//! learns what their proof is worth when they are deciding to trust it, rather than
//! months later from an auditor who tells them it is weaker than they assumed.
//! Neither the choosing nor the telling happens in this crate; this is the name
//! they both end up writing down.

/// What holds the key a statement is signed with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeldBy {
    /// A discrete security chip, separate from the processor.
    ASecurityChip,

    /// A secure element inside the processor.
    ASecureElement,

    /// The operating system's own keystore, protected by the system rather than
    /// by hardware that cannot export the key.
    TheSystemKeystore,

    /// A file on disk.
    ///
    /// The weakest of these, and the artifact says so in the same plain words as
    /// the others rather than omitting it. An attestation that quietly left this
    /// out when it was the answer would be the one case where silence is a claim.
    AFileOnDisk,

    /// Something this version of the format has no name for.
    ///
    /// Here so that a machine with protection nobody anticipated can still say
    /// what it used, in its own words, rather than having to claim one of the
    /// four above or say nothing. A verifier that does not recognise the name
    /// should treat it as unknown rather than as acceptable.
    SomethingElse(String),
}

impl HeldBy {
    /// The name this goes into the signed bytes under.
    ///
    /// Stable: these strings are part of the format, and changing one changes
    /// every signature made before it. A new kind gets a new name rather than a
    /// reused one.
    #[must_use]
    pub fn named(&self) -> String {
        match self {
            Self::ASecurityChip => "a-security-chip".to_owned(),
            Self::ASecureElement => "a-secure-element".to_owned(),
            Self::TheSystemKeystore => "the-system-keystore".to_owned(),
            Self::AFileOnDisk => "a-file-on-disk".to_owned(),
            Self::SomethingElse(what) => format!("something-else {what}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each kind has its own name, and no two share one — a name two kinds
    /// answered to would let a verifier accept the weaker believing it was the
    /// stronger, which is the whole reason this is not a boolean.
    #[test]
    fn every_kind_is_named_and_no_two_names_are_alike() {
        let every = [
            HeldBy::ASecurityChip,
            HeldBy::ASecureElement,
            HeldBy::TheSystemKeystore,
            HeldBy::AFileOnDisk,
            HeldBy::SomethingElse("a card somebody carries".to_owned()),
        ];
        let named: Vec<String> = every.iter().map(HeldBy::named).collect();
        let mut unique = named.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            unique.len(),
            named.len(),
            "two kinds share a name: {named:?}"
        );
        assert!(named.iter().all(|name| !name.is_empty()));
    }

    /// **The weakest kind is named rather than omitted.** Silence about protection
    /// would be the one case where saying nothing is a claim.
    #[test]
    fn a_file_on_disk_says_so() {
        assert_eq!(HeldBy::AFileOnDisk.named(), "a-file-on-disk");
    }

    /// A kind this version has no name for still says what it was.
    #[test]
    fn something_unanticipated_can_still_say_what_it_used() {
        assert_eq!(
            HeldBy::SomethingElse("a smartcard".to_owned()).named(),
            "something-else a smartcard"
        );
    }
}
