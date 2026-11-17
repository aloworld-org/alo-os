//! Whether a file was signed by a maker's pinned key (ADR 0158).
//!
//! The check is `gpgv`'s — GnuPG's verify-only program, a rented engine,
//! configured and never patched — run with a keyring holding only the maker's
//! pinned key. **Its verdict is read from its machine-readable status lines,
//! never from its exit status**: [`signer`] answers with the primary key's
//! fingerprint from a `VALIDSIG` line, and only when the same run said
//! `GOODSIG` and said none of the lines that mean the signature is not good.
//! `crate::installing_a_signed_release` then requires that fingerprint to be the
//! pinned one, so a keyring swapped for another key is refused too.

use std::path::Path;
use std::process::Command;

use crate::tool::Failed;

/// Where gpgv is on a machine.
pub const THE_VERIFIER: &str = "/usr/bin/gpgv";

/// Who signed a file, as far as a verifier can show.
pub trait Verifier {
    /// The primary fingerprint of the key in `keyring` that made `signature`
    /// over `file`, or [`None`] when no key there did.
    ///
    /// # Errors
    /// [`Failed::DidNotAnswer`] when the verifier could not be run.
    fn signer(
        &self,
        file: &Path,
        signature: &Path,
        keyring: &Path,
    ) -> Result<Option<String>, Failed>;
}

/// gpgv, at its place on this machine or at another for a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheRentedVerifier {
    /// The program run.
    program: String,
}

impl TheRentedVerifier {
    /// gpgv where this machine keeps it.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::at(THE_VERIFIER)
    }

    /// gpgv at another place.
    #[must_use]
    pub fn at(program: &str) -> Self {
        Self {
            program: program.to_owned(),
        }
    }
}

impl Verifier for TheRentedVerifier {
    fn signer(
        &self,
        file: &Path,
        signature: &Path,
        keyring: &Path,
    ) -> Result<Option<String>, Failed> {
        let ran = Command::new(&self.program)
            .arg("--status-fd")
            .arg("1")
            .arg("--keyring")
            .arg(keyring)
            .arg("--")
            .arg(signature)
            .arg(file)
            .env("LC_ALL", "C")
            .output()
            .map_err(|why| Failed::DidNotAnswer {
                said: format!("{} could not be started: {why}", self.program),
            })?;
        Ok(signer(&String::from_utf8_lossy(&ran.stdout)))
    }
}

/// The primary fingerprint a good signature names, from gpgv's status lines.
///
/// [`None`] unless there is exactly one `GOODSIG`, a `VALIDSIG` whose last
/// field is a forty-digit fingerprint, and none of `BADSIG`, `ERRSIG`,
/// `EXPSIG`, `EXPKEYSIG`, `REVKEYSIG` or `NO_PUBKEY`.
#[must_use]
pub fn signer(status: &str) -> Option<String> {
    let lines: Vec<Vec<&str>> = status
        .lines()
        .filter_map(|line| line.strip_prefix("[GNUPG:] "))
        .map(|line| line.split(' ').collect())
        .collect();
    let said = |word: &str| {
        lines
            .iter()
            .filter(|one| one.first() == Some(&word))
            .count()
    };
    let refusing = [
        "BADSIG",
        "ERRSIG",
        "EXPSIG",
        "EXPKEYSIG",
        "REVKEYSIG",
        "NO_PUBKEY",
    ];
    if said("GOODSIG") != 1 || said("VALIDSIG") != 1 || refusing.iter().any(|no| said(no) > 0) {
        return None;
    }
    let valid = lines.iter().find(|one| one.first() == Some(&"VALIDSIG"))?;
    let primary = *valid.last()?;
    let is_a_fingerprint = primary.len() == 40 && primary.chars().all(|c| c.is_ascii_hexdigit());
    is_a_fingerprint.then(|| primary.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What gpgv 2.4.4 printed on 2026-11-17 for a Craft release's checksum
    /// file and its signature, with the pinned key as the keyring.
    const GOOD: &str = "[GNUPG:] NEWSIG\n\
        [GNUPG:] KEY_CONSIDERED 2B76741B414AAD6CD33E631213A005C0B9D7C3ED 0\n\
        [GNUPG:] SIG_ID HwSt7jZa7o9MaIHMuhBmg6YxbAU 2026-11-12 1794454668\n\
        [GNUPG:] GOODSIG 006DAF6BB0EF334E ArtCraft Linux Release Signing Key\n\
        [GNUPG:] VALIDSIG 568DC1F68F8FAECC797F3C7B006DAF6BB0EF334E 2026-11-12 1794454668 0 4 0 \
        22 10 00 2B76741B414AAD6CD33E631213A005C0B9D7C3ED\n";

    /// The same, with one byte of the file changed.
    const BAD: &str = "[GNUPG:] NEWSIG\n\
        [GNUPG:] KEY_CONSIDERED 2B76741B414AAD6CD33E631213A005C0B9D7C3ED 0\n\
        [GNUPG:] BADSIG 006DAF6BB0EF334E ArtCraft Linux Release Signing Key\n";

    /// The same, with a keyring gpgv could not read.
    const NO_KEY: &str = "[GNUPG:] NEWSIG\n\
        [GNUPG:] ERRSIG 006DAF6BB0EF334E 22 10 00 1794454668 9 \
        568DC1F68F8FAECC797F3C7B006DAF6BB0EF334E\n\
        [GNUPG:] NO_PUBKEY 006DAF6BB0EF334E\n";

    #[test]
    fn a_good_signature_names_its_primary_key() {
        assert_eq!(
            signer(GOOD).as_deref(),
            Some("2B76741B414AAD6CD33E631213A005C0B9D7C3ED")
        );
    }

    /// **The refusals**: a bad signature, no key, nothing at all, a good line
    /// beside a bad one, two signatures, an expired key, and a fingerprint
    /// that is not one.
    #[test]
    fn anything_but_one_good_signature_names_nobody() {
        let two = format!("{GOOD}{GOOD}");
        let good_and_bad = format!("{GOOD}{BAD}");
        let expired = GOOD.replace("GOODSIG", "EXPKEYSIG");
        let expired_beside = format!("{GOOD}[GNUPG:] EXPKEYSIG 006DAF6BB0EF334E x\n");
        let short = GOOD.replace(
            "2B76741B414AAD6CD33E631213A005C0B9D7C3ED\n",
            "2B76741B414AAD6CD33E\n",
        );
        let not_hex = GOOD.replace(
            "2B76741B414AAD6CD33E631213A005C0B9D7C3ED\n",
            "2B76741B414AAD6CD33E631213A005C0B9D7C3EZ\n",
        );
        let unprefixed = GOOD.replace("[GNUPG:] ", "");
        // Two good signatures and one valid one: a second signer nobody named.
        let a_second_goodsig = format!("{GOOD}[GNUPG:] GOODSIG 1111111111111111 Somebody Else\n");
        // One good signature and no valid one at all.
        let no_validsig = GOOD
            .lines()
            .filter(|line| !line.contains("VALIDSIG"))
            .collect::<Vec<_>>()
            .join("\n");
        for no in [
            BAD,
            NO_KEY,
            "",
            good_and_bad.as_str(),
            two.as_str(),
            expired.as_str(),
            expired_beside.as_str(),
            short.as_str(),
            not_hex.as_str(),
            unprefixed.as_str(),
            a_second_goodsig.as_str(),
            no_validsig.as_str(),
        ] {
            assert_eq!(signer(no), None, "{no:?} named a signer");
        }
    }

    /// **A verifier that is not there did not answer**, and is never read as a
    /// verdict either way.
    #[test]
    fn a_verifier_that_is_not_there_did_not_answer() {
        let missing = TheRentedVerifier::at("/nonexistent/alo-software-test/no-gpgv");
        let answer = missing.signer(Path::new("a"), Path::new("b"), Path::new("c"));
        assert!(
            matches!(answer, Err(Failed::DidNotAnswer { ref said }) if said.contains("could not be started")),
            "{answer:?}"
        );
    }
}
