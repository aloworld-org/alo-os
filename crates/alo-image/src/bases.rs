//! The bases alo OS is built on, as `image/bases.toml` records alo's copy of
//! each.
//!
//! [ADR 0154](../../../docs/decisions/0154-the-base-is-a-copy-alo-keeps.md),
//! the owner's decision of 2026-11-13: *"alo's GitHub registry"*. Fedora
//! rebuilds `fedora-bootc:44` daily and Quay deletes the manifest its tag stops
//! naming, so a digest pinned on Quay lasted a day. A base is therefore copied,
//! byte for byte and keeping its digest, into `ghcr.io/aloworld-org`, and the
//! recipe builds from the copy. The digest still proves the bytes are the
//! maker's; what changes is who decides how long they stay.
//!
//! # Read strictly, for `crate::pinned`'s reason
//!
//! A record with a copy outside alo's registry, an origin that is alo's own
//! copy, or a digest that is not a digest is not a base with something wrong
//! with it. It is not a record of where a base came from, and the workflow that
//! copies bases would act on it. So [`TheBases::read`] refuses those.
//!
//! # One line per base
//!
//! `.github/workflows/copying-a-base.yml` finds a base by the line naming its
//! digest and reads the rest of that line, because a workflow step is not a TOML
//! reader. A file whose entry spans lines would read one way here and another
//! there, so this refuses it: the two readers are made to agree by the file's
//! shape, which is checked, rather than by hoping nobody reformats it.

use serde::Deserialize;

use crate::pinned::is_a_digest;
use crate::refusing::NotABase;

/// Where the record is, beneath the image's directory.
pub const THE_BASES: &str = "bases.toml";

/// Where alo keeps its copies: GitHub's registry, beside the code.
pub const ALOS_REGISTRY: &str = "ghcr.io/aloworld-org/";

/// The longest tag a registry accepts, from the OCI distribution specification.
const THE_LONGEST_TAG: usize = 128;

/// Every base the file records, in its order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBases {
    /// The bases.
    bases: Vec<Base>,
}

/// One base, as alo keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Base {
    /// alo's repository for it, under [`ALOS_REGISTRY`], with no tag.
    copy: String,
    /// What the copy is tagged: the maker's own version of that build.
    tag: String,
    /// The maker's digest, which the copy keeps.
    digest: String,
    /// Where it was copied from, with the tag that named the digest then.
    upstream: String,
    /// The day it was recorded, `YYYY-MM-DD`.
    recorded: String,
}

/// The file's shape, before anything about it is judged.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TheFile {
    /// The bases.
    bases: Vec<Base>,
}

impl TheBases {
    /// The bases this file records.
    ///
    /// # Errors
    ///
    /// [`NotABase`] when the file is not TOML of exactly this shape, when a
    /// base's fields are not all on the line naming its digest, when a copy is
    /// not under [`ALOS_REGISTRY`], when a digest is not a whole SHA-256, when a
    /// tag is not one a registry accepts, when an origin is alo's own registry
    /// or names no tag, when a day is not `YYYY-MM-DD`, or when one digest is
    /// recorded twice.
    pub fn read(text: &str) -> Result<Self, NotABase> {
        let file: TheFile = toml::from_str(text).map_err(|why| NotABase::NotToml {
            why: why.message().to_owned(),
        })?;

        let mut seen: Vec<&str> = Vec::new();
        for base in &file.bases {
            base.judged()?;
            if seen.contains(&base.digest.as_str()) {
                return Err(NotABase::Twice {
                    digest: base.digest.clone(),
                });
            }
            seen.push(&base.digest);
        }
        // Only once every digest is known to be recorded once: a digest on two
        // lines is also a digest on more than one line, and *twice* is the
        // sentence that says what to do about it.
        for base in &file.bases {
            if !base.is_on_one_line_of(text) {
                return Err(NotABase::NotOnOneLine {
                    digest: base.digest.clone(),
                });
            }
        }
        Ok(Self { bases: file.bases })
    }

    /// Every base recorded, in the file's order.
    #[must_use]
    pub fn bases(&self) -> &[Base] {
        &self.bases
    }

    /// The base this reference is alo's copy of, where it is one.
    ///
    /// Only `copy@digest` answers: a tag is a name somebody can move, so a
    /// recipe naming the copy by its tag is not naming these bytes.
    #[must_use]
    pub fn copied_as(&self, reference: &str) -> Option<&Base> {
        self.bases.iter().find(|base| base.reference() == reference)
    }

    /// The base this reference is the maker's original of, where it is one.
    ///
    /// **The transition, and only that.** ADR 0154 is landed in two changes:
    /// the record and the copying workflow first, then the recipes moved to the
    /// copy once it exists. Between the two, the recipes still name the
    /// original, and this is how the check knows it is the recorded one. The
    /// second change removes it.
    #[must_use]
    pub fn original_of(&self, reference: &str) -> Option<&Base> {
        self.bases.iter().find(|base| base.origin() == reference)
    }
}

impl Base {
    /// alo's repository for it, with no tag.
    #[must_use]
    pub fn copy(&self) -> &str {
        &self.copy
    }

    /// What the copy is tagged.
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The maker's digest, which the copy keeps.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// Where it was copied from, with the tag that named the digest then.
    #[must_use]
    pub fn upstream(&self) -> &str {
        &self.upstream
    }

    /// The day it was recorded.
    #[must_use]
    pub fn recorded(&self) -> &str {
        &self.recorded
    }

    /// What a recipe names to build on alo's copy: the copy, by digest.
    #[must_use]
    pub fn reference(&self) -> String {
        format!("{}@{}", self.copy, self.digest)
    }

    /// The maker's original, as a recipe named it before the copy: the origin
    /// with its tag, by digest.
    #[must_use]
    pub fn origin(&self) -> String {
        format!("{}@{}", self.upstream, self.digest)
    }

    /// Why this base is not one alo can keep, where it is not.
    fn judged(&self) -> Result<(), NotABase> {
        let repository = self.copy.strip_prefix(ALOS_REGISTRY).unwrap_or_default();
        if repository.is_empty() || repository.contains([':', '@']) {
            return Err(NotABase::NotAlosRegistry {
                stated: self.copy.clone(),
            });
        }
        if !is_a_digest(&self.digest) {
            return Err(NotABase::NotADigest {
                stated: self.digest.clone(),
            });
        }
        if !is_a_tag(&self.tag) {
            return Err(NotABase::NotATag {
                stated: self.tag.clone(),
            });
        }
        if !is_an_origin(&self.upstream) {
            return Err(NotABase::NotAnOrigin {
                stated: self.upstream.clone(),
            });
        }
        if !is_a_day(&self.recorded) {
            return Err(NotABase::NotADay {
                stated: self.recorded.clone(),
            });
        }
        Ok(())
    }

    /// Whether every field of this base is on the one line naming its digest,
    /// written as the copying workflow reads it.
    fn is_on_one_line_of(&self, text: &str) -> bool {
        let named = format!("digest = \"{}\"", self.digest);
        let mut lines = text.lines().filter(|line| line.contains(&named));
        let (Some(line), None) = (lines.next(), lines.next()) else {
            return false;
        };
        [
            ("copy", &self.copy),
            ("tag", &self.tag),
            ("upstream", &self.upstream),
            ("recorded", &self.recorded),
        ]
        .into_iter()
        .all(|(key, value)| line.contains(&format!("{key} = \"{value}\"")))
    }
}

/// Whether a registry would accept this as a tag.
fn is_a_tag(stated: &str) -> bool {
    let mut bytes = stated.bytes();
    stated.len() <= THE_LONGEST_TAG
        && bytes
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric() || first == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// Whether this is the maker's repository with the tag that named the digest.
///
/// A tag, and not a digest: the digest is its own field, and an origin with one
/// would be two digests that could disagree. Never alo's registry: a copy of a
/// copy says nothing about where the bytes were made.
fn is_an_origin(stated: &str) -> bool {
    let Some((repository, tag)) = stated.rsplit_once(':') else {
        return false;
    };
    !stated.starts_with(ALOS_REGISTRY)
        && !stated.contains('@')
        && repository.contains('/')
        && !tag.contains('/')
        && is_a_tag(tag)
}

/// Whether this is a day written `YYYY-MM-DD`.
fn is_a_day(stated: &str) -> bool {
    let parts: Vec<&str> = stated.split('-').collect();
    matches!(parts.as_slice(), [year, month, day]
        if [(year, 4), (month, 2), (day, 2)]
            .into_iter()
            .all(|(part, length)| part.len() == length && part.bytes().all(|b| b.is_ascii_digit())))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The record this repository ships, as text.
    fn the_record() -> String {
        std::fs::read_to_string(std::path::Path::new(crate::THE_IMAGE).join(THE_BASES)).unwrap()
    }

    /// The shipped record with one value replaced.
    fn with(from: &str, to: &str) -> Result<TheBases, NotABase> {
        let record = the_record();
        assert!(record.contains(from), "the record no longer says {from}");
        TheBases::read(&record.replacen(from, to, 1))
    }

    /// A base written as the shipped one is.
    const A_BASE: &str = r#"{ copy = "ghcr.io/aloworld-org/fedora-bootc", tag = "44.1", digest = "sha256:0000000000000000000000000000000000000000000000000000000000000000", upstream = "quay.io/fedora/fedora-bootc:44", recorded = "2026-11-13" }"#;

    /// **The shipped record reads**, and its first base is Fedora's.
    #[test]
    fn the_shipped_record_reads() {
        let bases = TheBases::read(&the_record()).unwrap();
        let first = bases.bases().first().unwrap();
        assert!(first.copy().starts_with(ALOS_REGISTRY));
        assert!(first.upstream().starts_with("quay.io/fedora/"));
    }

    /// **A copy outside alo's registry is refused**, including one that only
    /// begins with alo's organisation's name.
    #[test]
    fn a_copy_outside_alos_registry_is_refused() {
        for elsewhere in [
            "quay.io/aloworld-org/fedora-bootc",
            "ghcr.io/aloworld-organisation/fedora-bootc",
            "ghcr.io/aloworld-org/",
            "ghcr.io/aloworld-org/fedora-bootc:44",
        ] {
            assert!(
                matches!(
                    with(
                        "ghcr.io/aloworld-org/fedora-bootc\"",
                        &format!("{elsewhere}\"")
                    ),
                    Err(NotABase::NotAlosRegistry { .. })
                ),
                "{elsewhere}"
            );
        }
    }

    /// **A digest that is not a whole SHA-256 is refused.**
    #[test]
    fn a_digest_that_is_not_one_is_refused() {
        assert!(matches!(
            with("sha256:5a3472162ba7", "sha256:5A3472162ba7"),
            Err(NotABase::NotADigest { .. })
        ));
        assert!(matches!(
            with("e868\"", "\""),
            Err(NotABase::NotADigest { .. })
        ));
    }

    /// **An origin that is alo's own copy, or names a digest or no tag, is
    /// refused.**
    #[test]
    fn an_origin_that_is_not_the_makers_is_refused() {
        for origin in [
            "ghcr.io/aloworld-org/fedora-bootc:44",
            "quay.io/fedora/fedora-bootc",
            "quay.io/fedora/fedora-bootc@sha256:00",
            "fedora-bootc:44",
        ] {
            assert!(
                matches!(
                    with("quay.io/fedora/fedora-bootc:44\"", &format!("{origin}\"")),
                    Err(NotABase::NotAnOrigin { .. })
                ),
                "{origin}"
            );
        }
    }

    /// **A tag a registry would refuse, and a day that is not a day, are
    /// refused.**
    #[test]
    fn a_tag_or_a_day_that_is_not_one_is_refused() {
        assert!(matches!(
            with("tag = \"44.20261113.0\"", "tag = \".44\""),
            Err(NotABase::NotATag { .. })
        ));
        assert!(matches!(
            with("tag = \"44.20261113.0\"", "tag = \"\""),
            Err(NotABase::NotATag { .. })
        ));
        assert!(matches!(
            with(
                "recorded = \"2026-11-13\"",
                "recorded = \"13 November 2026\""
            ),
            Err(NotABase::NotADay { .. })
        ));
    }

    /// **A base spread over several lines is refused**, because the workflow
    /// that copies it would read only the line naming its digest.
    #[test]
    fn a_base_on_several_lines_is_refused() {
        // The same base as a table of its own, which TOML reads identically and
        // a line-reading workflow does not.
        let spread = A_BASE
            .trim_start_matches("{ ")
            .trim_end_matches(" }")
            .replace(", ", "\n");
        let spread = format!("[[bases]]\n{spread}\n");
        assert!(matches!(
            TheBases::read(&spread),
            Err(NotABase::NotOnOneLine { .. })
        ));
        let whole = format!("bases = [\n  {A_BASE},\n]\n");
        assert!(TheBases::read(&whole).is_ok());
    }

    /// **One digest recorded twice is refused**, whatever the two lines say.
    #[test]
    fn a_digest_recorded_twice_is_refused() {
        let twice = format!("bases = [\n  {A_BASE},\n  {A_BASE},\n]\n");
        assert!(matches!(
            TheBases::read(&twice),
            Err(NotABase::Twice { .. })
        ));
    }

    /// **A key nobody declared is refused** rather than ignored.
    #[test]
    fn an_unknown_key_is_refused() {
        assert!(matches!(
            with("recorded = ", "signed = \"yes\", recorded = "),
            Err(NotABase::NotToml { .. })
        ));
    }

    /// **The copy answers only by digest, and the original only as the maker
    /// named it.**
    #[test]
    fn a_reference_is_recognised_only_by_digest() {
        let bases = TheBases::read(&the_record()).unwrap();
        let base = bases.bases().first().unwrap();
        assert_eq!(bases.copied_as(&base.reference()), Some(base));
        assert_eq!(bases.original_of(&base.origin()), Some(base));
        let by_tag = format!("{}:{}", base.copy(), base.tag());
        assert_eq!(bases.copied_as(&by_tag), None);
        assert_eq!(bases.copied_as(&base.origin()), None);
        assert_eq!(bases.original_of(&base.reference()), None);
    }
}
