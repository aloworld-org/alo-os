//! How a line of the image's recipe is read.
//!
//! Two modules ask the Containerfile questions — `crate::runtime` about the
//! model runtime, `crate::weights` about the model — and both were going to
//! spell *a copy out of a build stage* and *a whole sha256* for themselves. A
//! second spelling of either is the drift this crate exists to catch everywhere
//! else, so the spelling lives once, here, and the two readers are about what
//! they are reading rather than about how a `COPY` line is written.
//!
//! Nothing here knows what the runtime or the weights **are**. It knows what a
//! Containerfile looks like, which is one responsibility and a small one.

/// How many hexadecimal characters a sha256 digest has.
const A_WHOLE_DIGEST: usize = 64;

/// Whether this line copies something out of another build stage to exactly
/// this place on the machine.
///
/// `--from=` is part of the question on purpose: an artefact copied out of the
/// build context would be something committed into this repository, which is the
/// *source tree* shape ADR 0006 refuses — pinned upstream artefacts arrive
/// through a stage that fetched and checked them.
pub(crate) fn copies_from_a_stage_to(line: &str, landing: &str) -> bool {
    line.starts_with("COPY")
        && line.contains("--from=")
        && line.split_whitespace().next_back() == Some(landing)
}

/// Whether this is a whole sha256 and nothing else.
///
/// Half a digest is not half a pin: it is a pin that stopped saying what
/// arrived, and `sha256sum --check` would refuse it on the build machine long
/// after somebody believed the recipe said something.
pub(crate) fn is_a_whole_digest(digest: &str) -> bool {
    digest.len() == A_WHOLE_DIGEST && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

/// What this line says after `ARG NAME=`, trimmed, where it is that argument.
pub(crate) fn argument<'a>(line: &'a str, named: &str) -> Option<&'a str> {
    line.strip_prefix(named).map(str::trim)
}

/// The lines of one named build stage, in the order the recipe has them, with
/// comments and blank lines dropped.
///
/// Scoped by stage because order is a question here and order is only a question
/// inside one: *the digest is checked before anything reads the file* is a
/// sentence about a stage, and a reader that swept the whole recipe would
/// compare a line of the weights stage against a line of the runtime's and mean
/// nothing by it.
pub(crate) fn in_stage<'a>(containerfile: &'a str, stage: &str) -> Vec<&'a str> {
    let mut lines = Vec::new();
    let mut here = false;
    for line in containerfile.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with("FROM ") {
            here = named_stage(line) == Some(stage);
            continue;
        }
        if here {
            lines.push(line);
        }
    }
    lines
}

/// The name a `FROM` line gives its stage, where it names one.
fn named_stage(line: &str) -> Option<&str> {
    let mut words = line.split_whitespace();
    while let Some(word) = words.next() {
        if word.eq_ignore_ascii_case("AS") {
            return words.next();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A copy out of the build context is not an artefact landing.** A binary
    /// committed into this repository is the source-tree shape ADR 0006
    /// refuses, and a commented-out line lands nothing at all.
    #[test]
    fn only_a_copy_out_of_a_stage_to_exactly_that_place_counts() {
        assert!(copies_from_a_stage_to(
            "COPY --from=runtime /runtime/bin/ollama /usr/bin/ollama",
            "/usr/bin/ollama"
        ));
        assert!(!copies_from_a_stage_to(
            "COPY vendored/ollama /usr/bin/ollama",
            "/usr/bin/ollama"
        ));
        assert!(!copies_from_a_stage_to(
            "COPY --from=runtime /runtime/bin/ollama /usr/bin/somewhere-else",
            "/usr/bin/ollama"
        ));
    }

    /// A digest is whole and hexadecimal, or it is prose.
    #[test]
    fn half_a_digest_is_not_a_pin() {
        assert!(is_a_whole_digest(&"ab".repeat(32)));
        assert!(!is_a_whole_digest(&"ab".repeat(16)));
        assert!(!is_a_whole_digest(&"zz".repeat(32)));
        assert!(!is_a_whole_digest(""));
    }

    /// **A stage is read as itself.** The lines of one stage, in order, and
    /// nothing from the stage before or after it — which is the whole of what
    /// makes *checked before anything reads it* an answerable question.
    #[test]
    fn a_stage_is_read_in_order_and_alone() {
        let recipe = "\
FROM ubuntu AS first
RUN fetch /one
# a comment nobody runs

FROM ubuntu AS second
RUN fetch /two
RUN check /two
FROM fedora
COPY --from=second /two /two";

        assert_eq!(in_stage(recipe, "first"), vec!["RUN fetch /one"]);
        assert_eq!(
            in_stage(recipe, "second"),
            vec!["RUN fetch /two", "RUN check /two"]
        );
        assert!(in_stage(recipe, "nobody").is_empty());
    }

    /// An argument is what it says after the `=`, and nothing is not an
    /// argument.
    #[test]
    fn an_argument_is_what_it_says() {
        assert_eq!(
            argument("ARG THE_MODEL=phi ", "ARG THE_MODEL="),
            Some("phi")
        );
        assert_eq!(argument("ARG THE_OTHER=phi", "ARG THE_MODEL="), None);
    }
}
