//! The release a promise is made for, as `docs/features.md` marks it.
//!
//! Every line of the definition carries a tier — `[v0.01]`, `[v0.5]`, `[v1]` —
//! and this crate never holds a list of what they are. It reads them out of the
//! document, so *the release that owns the answer* means a release alo OS
//! actually ships rather than a word somebody typed between brackets. A tier
//! invented in this file is a tier that would go on passing after the roadmap
//! renamed it.

/// One release of alo OS, as a promise is marked with it.
///
/// There is no constructor from anything but [`Release::named`], and it refuses
/// anything that is not shaped like a tier — which is what keeps an ordinary
/// markdown list item, `- [ ] something`, from being read as a promise for a
/// release called nothing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Release {
    /// The tier as the definition writes it, without its brackets.
    named: String,
}

impl Release {
    /// The release this text names, or [`None`] if it is not shaped like a tier.
    ///
    /// A `v` and then digits and dots: `v0.01`, `v0.5`, `v1`. Deliberately a
    /// shape and not a list — the list is whatever `docs/features.md` uses, and
    /// [`crate::promised::releases_among`] is what answers that.
    #[must_use]
    pub fn named(text: &str) -> Option<Self> {
        let named = text.trim();
        let numbered = named.strip_prefix('v')?;
        if numbered.is_empty() || !numbered.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return None;
        }
        Some(Self {
            named: named.to_owned(),
        })
    }

    /// The tier, as the definition writes it.
    #[must_use]
    pub fn as_written(&self) -> &str {
        &self.named
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three tiers this repository uses, read as themselves.
    #[test]
    fn a_tier_is_a_v_and_a_number() {
        for tier in ["v0.01", "v0.5", "v1"] {
            assert_eq!(
                Release::named(tier).map(|release| release.as_written().to_owned()),
                Some(tier.to_owned())
            );
        }
        assert_eq!(Release::named("  v0.5 "), Release::named("v0.5"));
    }

    /// And everything that is not one. The empty checkbox matters most: it is
    /// how markdown writes a task list, and reading it as a tier would turn
    /// every unticked box in the definition into a promise.
    #[test]
    fn what_is_not_a_tier_is_not_a_release() {
        for not_one in ["", " ", "v", "x", "0.5", "later", "v0.5-rc1", " ", "]"] {
            assert_eq!(Release::named(not_one), None, "{not_one:?}");
        }
    }
}
