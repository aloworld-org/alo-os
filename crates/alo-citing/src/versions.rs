//! Every version this repository writes, held to the form its kind requires.
//!
//! `ROADMAP.md`'s *Two numbers that look alike* records that `v0.5` is a
//! milestone and `0.0.5` is an image, that they are one character apart, and
//! that on 2026-09-26 the owner and the loop spent an evening each meaning a
//! different one. It then gives rules — and until this module, **nothing read
//! them.** On 2026-10-05 twelve sentences across three plans broke one or the
//! other, and the fault was found by a person reading, which is the thing this
//! repository keeps proving does not scale.
//!
//! # Three kinds, three forms, one source each
//!
//! | kind | written | where the truth is |
//! |---|---|---|
//! | milestone | `v0.01`, `v0.5`, `v1` | `ROADMAP.md`'s own `## v…` headings |
//! | image | *image 0.0.6*, never bare | `image/pinned.toml`'s `version` |
//! | crate | `alo-canvas v0.0.1` | each `Cargo.toml`, and not this rule's business |
//!
//! The third is why this is not a regular expression over the whole tree:
//! cargo's own output says `Compiling alo-canvas v0.0.1`, which is a **crate**
//! version and correct. A check that flagged it would be refusing the compiler's
//! own words.
//!
//! # It reads paragraphs, because a line is not a unit of prose
//!
//! A sentence wraps. *…and image* can end one line and *0.0.6 reaches…* begin
//! the next, and a per-line check calls that a bare number. That exact failure
//! happened three times in one hour while this rule was being written, twice
//! inside a guard meant to catch it. So the text is joined and the markup
//! stripped before anything is matched, and a finding reports the line its
//! paragraph starts on so a person can find it.

use std::fmt::{self, Display};

/// How a milestone is allowed to be written. Three, ever.
pub const THE_MILESTONES: [&str; 3] = ["v0.01", "v0.5", "v1"];

/// The words that make a paragraph's numbers say they are images.
///
/// `ROADMAP.md` asks that *a reader can tell*, and these are the words by which
/// one can. They are not synonyms for *image*: each is a thing only done to one.
/// A paragraph about the pin, the digest, the registry or the recipe is a
/// paragraph about images, and a reader of it is in no doubt.
const SAYS_IT_IS_AN_IMAGE: [&str; 13] = [
    "image",
    "release",
    "signed",
    "signature",
    "pin",
    "digest",
    "registry",
    "recipe",
    "installer",
    "sha256",
    "carries",
    "published",
    "booted",
];

/// What is wrong with how a version was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amiss {
    /// A milestone written with a second zero: `v0.0.5`. Wrong whichever was
    /// meant, which is why it cannot be corrected without reading the sentence.
    AMilestoneWithASecondZero {
        /// The file it is in.
        file: String,
        /// The line its paragraph starts on.
        line: usize,
        /// Enough of the sentence to find it by.
        saying: String,
    },
    /// An image number with nothing saying it is an image.
    AnImageWrittenBare {
        /// The file it is in.
        file: String,
        /// The line its paragraph starts on.
        line: usize,
        /// Enough of the sentence to find it by.
        saying: String,
    },
    /// A build identifier's date written with dots, which reads as a product
    /// version with a point release.
    ADateWrittenAsAVersion {
        /// The file it is in.
        file: String,
        /// The line its paragraph starts on.
        line: usize,
        /// What was written.
        written: String,
    },
    /// A product version carrying a `v`, which is a milestone's mark.
    AProductVersionWithAV {
        /// The file it is in.
        file: String,
        /// The line its paragraph starts on.
        line: usize,
        /// What was written.
        written: String,
    },
    /// A milestone this repository does not have. There are three, ever.
    AMilestoneThatIsNotOne {
        /// The file it is in.
        file: String,
        /// The line its paragraph starts on.
        line: usize,
        /// What was written.
        written: String,
    },
}

impl Display for Amiss {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AMilestoneWithASecondZero { file, line, saying } => write!(
                out,
                "{file}:{line} writes a milestone with a second zero, which ROADMAP.md says is \
                 wrong whichever was meant. If an image was meant write `image 0.0.5`; if the \
                 milestone was, write `v0.5`. Read the sentence before choosing — on 2026-10-05 \
                 every one of these turned out to mean the image. In: {saying}"
            ),
            Self::AnImageWrittenBare { file, line, saying } => write!(
                out,
                "{file}:{line} writes an image number with nothing saying it is an image. Write \
                 `image 0.0.6`, and give its date where a reader might care. In: {saying}"
            ),
            Self::ADateWrittenAsAVersion {
                file,
                line,
                written,
            } => write!(
                out,
                "{file}:{line} writes `{written}`, a date with dots, which reads as a product \
                 version with a point release. ADR 0097 rule 3: dots are a version, hyphens are \
                 a date. A build identifier is `2026-10-08+4799555`; a product version is \
                 `2026.10` or `2026.10.1`."
            ),
            Self::AProductVersionWithAV {
                file,
                line,
                written,
            } => write!(
                out,
                "{file}:{line} writes `{written}`, a product version carrying a `v`. A `v` is a \
                 milestone's mark (v0.01, v0.5, v1) and ADR 0097 keeps those as planning \
                 numbers. A product version is written bare: `2026.10`."
            ),
            Self::AMilestoneThatIsNotOne {
                file,
                line,
                written,
            } => write!(
                out,
                "{file}:{line} names a milestone `{written}`, and this product has three ever: \
                 v0.01, v0.5 and v1. An image is not a milestone."
            ),
        }
    }
}

/// A paragraph of this repository's own prose, joined to one line, and the line
/// it begins on.
///
/// Markdown is left in: stripping it is the matcher's business, and a finding
/// quotes what a person will actually see in the file.
///
/// **A fenced block is not prose and is not returned.** It holds quoted output —
/// a registry listing its tags as `["0.0.1", "sha256-d3f05b60…", "0.0.2", …]`,
/// cargo saying `Checking alo-models v0.0.1`. That is a transcript of what
/// another program said, and this rule governs the sentences we write. Asking a
/// registry to call its own tag an image is asking it to lie about what it
/// printed. Fifteen of the first sixty findings were exactly this.
#[must_use]
pub fn paragraphs(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    let mut began = 1;
    let mut fenced = false;
    for (number, line) in text.lines().enumerate() {
        let number = number + 1;
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            if fenced && !held.is_empty() {
                out.push((began, held.join(" ")));
                held.clear();
            }
            continue;
        }
        if fenced {
            continue;
        }
        if line.trim().is_empty() {
            if !held.is_empty() {
                out.push((began, held.join(" ")));
                held.clear();
            }
            continue;
        }
        if held.is_empty() {
            began = number;
        }
        held.push(line.trim());
    }
    if !held.is_empty() {
        out.push((began, held.join(" ")));
    }
    out
}

/// The same text with the markdown taken out, so a word beside a number is
/// found whether or not somebody bolded it.
fn plainly(text: &str) -> String {
    text.replace(['*', '`', '_', '#'], " ")
}

/// A byte index moved back to the nearest character boundary.
///
/// This repository's prose is full of em dashes, stars and curly quotes, and a
/// window taken by subtracting thirty from a byte index lands inside one about
/// as often as not. The first run of this rule panicked at byte 392, inside an
/// em dash — the same fault as everything else it was written to catch, one
/// level lower: a byte is not a unit of text.
fn back_to_a_boundary(text: &str, mut at: usize) -> usize {
    at = at.min(text.len());
    while at > 0 && !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// A byte index moved forward to the nearest character boundary.
fn on_to_a_boundary(text: &str, mut at: usize) -> usize {
    at = at.min(text.len());
    while at < text.len() && !text.is_char_boundary(at) {
        at += 1;
    }
    at
}

/// Whether a paragraph says, in words, that its numbers are images.
///
/// **The paragraph is the unit, not a window around the number.** The first
/// version of this asked only the thirty characters either side, and reported
/// sixty sentences whose own paragraph opened *the published image pinned by
/// digest*. A reader of those is in no doubt which kind is meant, which is the
/// whole of what `ROADMAP.md` asks; a rule that refused them would be enforcing
/// a word count rather than clarity, and the first person to meet it would
/// learn to sprinkle the word *image* rather than to write plainly.
fn says_it_is_an_image(plain: &str) -> bool {
    let lower = plain.to_lowercase();
    SAYS_IT_IS_AN_IMAGE.iter().any(|word| lower.contains(word))
}

/// Every alo image number in a string, as byte ranges.
///
/// **`0.0.N` and nothing else.** An alo OS image is that series — `image
/// /pinned.toml` records 0.0.1 to 0.0.5 — and the first version of this
/// function matched any digit-dot-digit-dot-digit, which made it report
/// `PipeWire 1.0.5`, `Ubuntu 24.04.5`, `Mesa 25.2.8` and `cosign 3.1.3`. Those
/// are other people's software and this rule has nothing to say about them.
///
/// The test caught it by refusing rather than passing, which is the only reason
/// the scope was ever questioned.
///
/// Every byte is reached through `get`, never by `[]`, because this crate denies
/// `clippy::indexing_slicing` — and rightly: a walk that panics on the one
/// document with an awkward ending is a check that stops rather than refuses.
fn numbers_in(plain: &str) -> Vec<(usize, usize)> {
    /// `0.0.N` is five bytes: three digits and the two dots between them.
    const WIDTH: usize = 5;

    let bytes = plain.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while at + WIDTH <= bytes.len() {
        let looks = matches!(
            bytes.get(at..at + WIDTH),
            Some([b'0', b'.', b'0', b'.', last]) if last.is_ascii_digit()
        );
        if looks {
            let part_of_a_longer_word = |byte: &u8| byte.is_ascii_alphanumeric() || *byte == b'.';
            let before_is_part = at
                .checked_sub(1)
                .and_then(|just_before| bytes.get(just_before))
                .is_some_and(part_of_a_longer_word);
            let after = at + WIDTH;
            let after_is_part = bytes.get(after).is_some_and(part_of_a_longer_word);
            if !before_is_part && !after_is_part {
                found.push((at, after));
                at = after;
                continue;
            }
        }
        at += 1;
    }
    found
}

/// Every dotted `YYYY.MM.DD` in a paragraph, by byte range.
///
/// **A build identifier's date written with dots** (ADR 0097 rule 3), which
/// reads as a product version with a point release. The rule exists because
/// that ADR's own first draft proposed exactly this form.
///
/// Four-two-two is the whole of the narrowing and it is enough, measured: the
/// durations, the other projects' versions and the protocol example that a bare
/// four-two shape matched in this repository are all four-two or four-two-one,
/// and none of them matches this.
fn dates_with_dots_in(plain: &str) -> Vec<(usize, usize)> {
    /// `YYYY.MM.DD` is ten bytes.
    const WIDTH: usize = 10;
    shaped(plain, WIDTH, |it| {
        matches!(it, [a, b, c, d, b'.', e, f, b'.', g, h]
            if [a, b, c, d, e, f, g, h].iter().all(|byte| byte.is_ascii_digit()))
    })
}

/// Every `vYYYY.MM` in a paragraph, by byte range.
///
/// A `v` is a milestone's mark and ADR 0097 keeps the milestones as planning
/// numbers, so a product version never carries one.
fn product_versions_with_a_v_in(plain: &str) -> Vec<(usize, usize)> {
    /// `vYYYY.MM` is eight bytes.
    const WIDTH: usize = 8;
    shaped(plain, WIDTH, |it| {
        matches!(it, [b'v' | b'V', a, b, c, d, b'.', e, f]
            if [a, b, c, d, e, f].iter().all(|byte| byte.is_ascii_digit()))
    })
}

/// Every run of this width whose bytes look right, and which is not part of a
/// longer word or number.
///
/// The boundary check is `numbers_in`'s, lifted rather than copied: a shape
/// inside a longer run of digits and dots is part of something else, and
/// reporting it would be reporting a fragment.
fn shaped(plain: &str, width: usize, looks: impl Fn(&[u8]) -> bool) -> Vec<(usize, usize)> {
    let bytes = plain.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while at + width <= bytes.len() {
        if bytes.get(at..at + width).is_some_and(&looks) {
            let part_of_a_longer_word =
                |byte: &u8| byte.is_ascii_alphanumeric() || *byte == b'.' || *byte == b'-';
            let before_is_part = at
                .checked_sub(1)
                .and_then(|just_before| bytes.get(just_before))
                .is_some_and(part_of_a_longer_word);
            let after = at + width;
            // **A `+` after it is not a boundary that excuses it.** A build
            // identifier written with dots ends `+4799555`, and that is the
            // clearest case of this fault rather than a reason to skip it.
            let after_is_part = bytes
                .get(after)
                .is_some_and(|byte| part_of_a_longer_word(byte) && *byte != b'+');
            if !before_is_part && !after_is_part {
                found.push((at, after));
                at = after;
                continue;
            }
        }
        at += 1;
    }
    found
}

/// Hold one document to the rule.
///
/// `file` is only used to name a finding. A paragraph that carries the rule's
/// own words — recognised by naming `ROADMAP.md`'s section or quoting the form
/// it forbids inside backticks — is skipped, because a rule has to be able to
/// state itself.
#[must_use]
pub fn held(file: &str, text: &str) -> Vec<Amiss> {
    let mut found = Vec::new();
    for (line, paragraph) in paragraphs(text) {
        if states_the_rule(&paragraph) {
            continue;
        }
        let plain = plainly(&paragraph);
        let lower = plain.to_lowercase();
        if let Some(at) = lower.find("v0.0.")
            && !is_a_crates_version(&plain, at)
            && !is_the_git_tag(&lower, at)
        {
            found.push(Amiss::AMilestoneWithASecondZero {
                file: file.to_owned(),
                line,
                saying: nearby(&paragraph, at),
            });
        }
        let says = says_it_is_an_image(&plain);
        for (at, _) in numbers_in(&plain) {
            if says || is_a_crates_version(&plain, at) {
                continue;
            }
            found.push(Amiss::AnImageWrittenBare {
                file: file.to_owned(),
                line,
                saying: nearby(&paragraph, at),
            });
        }
        // **ADR 0097's two forbidden forms.** Reported by shape rather than by
        // paragraph context, unlike the image rule above: these are wrong
        // however the sentence around them reads, and the shapes are narrow
        // enough that nothing else in this repository's prose matches them.
        for (at, to) in dates_with_dots_in(&plain) {
            found.push(Amiss::ADateWrittenAsAVersion {
                file: file.to_owned(),
                line,
                written: plain[at..to].to_owned(),
            });
        }
        for (at, to) in product_versions_with_a_v_in(&plain) {
            found.push(Amiss::AProductVersionWithAV {
                file: file.to_owned(),
                line,
                written: plain[at..to].to_owned(),
            });
        }
    }
    found
}

/// Whether a paragraph is the rule talking about itself.
fn states_the_rule(paragraph: &str) -> bool {
    let lower = paragraph.to_lowercase();
    lower.contains("two numbers that look alike")
        || lower.contains("never `v0.0")
        || lower.contains("wrong whichever was meant")
        || lower.contains("never a second zero")
        || lower.contains("never written as a bare number")
}

/// Whether a number is cargo's, as in `Compiling alo-canvas v0.0.1`.
///
/// A crate's version is a third kind this rule does not govern, and refusing it
/// would be refusing the compiler's own words.
fn is_a_crates_version(plain: &str, at: usize) -> bool {
    let from = back_to_a_boundary(plain, at.saturating_sub(40));
    let before = plain[from..back_to_a_boundary(plain, at)].to_lowercase();
    before.contains("compiling ") || before.contains("checking ") || before.contains("alo-")
}

/// Whether a `v0.0.N` is the git tag, which is published and carries a `v`.
///
/// `release.yml` fires on `v*` and refuses any tag that is not `v` followed by
/// `image/pinned.toml`'s version, so the tag for image 0.0.5 **is** `v0.0.5` and
/// there is one in this repository today. `ROADMAP.md` says nothing published is
/// renamed, so a document that names the tag is naming it correctly. The rule
/// forbids the *form in a sentence*, not the tag it collides with.
fn is_the_git_tag(lower: &str, at: usize) -> bool {
    let from = back_to_a_boundary(lower, at.saturating_sub(40));
    let before = &lower[from..back_to_a_boundary(lower, at)];
    before.contains("tag") || before.contains("release.yml") || before.contains("github_ref")
}

/// Enough of a paragraph around a point to find it in the file by.
fn nearby(paragraph: &str, at: usize) -> String {
    let from = back_to_a_boundary(paragraph, at.saturating_sub(40));
    let to = on_to_a_boundary(paragraph, at.saturating_add(40));
    paragraph[from..to].trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What kind each finding is, so an assertion names the refusal rather than
    /// counting anonymous ones.
    fn kinds(found: &[Amiss]) -> Vec<&'static str> {
        found
            .iter()
            .map(|it| match it {
                Amiss::AMilestoneWithASecondZero { .. } => "second zero",
                Amiss::AnImageWrittenBare { .. } => "bare",
                Amiss::AMilestoneThatIsNotOne { .. } => "not a milestone",
            })
            .collect()
    }

    /// **A bare image number in a paragraph that never says it is one.** The
    /// fault `ROADMAP.md` names first, and the thing this whole module exists
    /// for.
    #[test]
    fn an_image_number_alone_in_a_sentence_is_refused() {
        let found = held(
            "a-plan.md",
            "What is left after 0.0.6 is what it always was.",
        );
        assert_eq!(kinds(&found), vec!["bare"], "{found:?}");
    }

    /// **The same number, in a paragraph that says what it is, is fine.** The
    /// rule asks that a reader can tell, and a reader of this one can.
    #[test]
    fn an_image_number_a_paragraph_explains_is_left_alone() {
        let found = held(
            "a-plan.md",
            "What is left after image 0.0.6 is pinned is what it always was.",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    /// **`v0.0.5` is wrong whichever kind was meant** — `ROADMAP.md`'s own
    /// words. A milestone carries no second zero and an image carries no `v`.
    #[test]
    fn a_milestone_with_a_second_zero_is_refused() {
        let found = held("a-plan.md", "This closes v0.0.5 and nothing after it.");
        assert_eq!(kinds(&found), vec!["second zero"], "{found:?}");
    }

    /// **A crate's version is a third kind this rule does not govern.** Cargo
    /// prints `Checking alo-models v0.0.1`, 113 crates in this repository carry
    /// a version of that shape, and refusing it would be refusing the
    /// compiler's own words. The first run against the real documents reported
    /// exactly this line as a milestone.
    #[test]
    fn a_crates_version_is_not_this_rules_business() {
        let found = held(
            "an-update.md",
            "The one `Checking alo-models v0.0.1` line is Cargo's.",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    /// **The git tag for an image is `v0.0.5` and there is one here today.**
    /// `release.yml` fires on `v*` and refuses any tag that is not `v` followed
    /// by the pinned version, so a document naming the tag names it correctly,
    /// and `ROADMAP.md` says nothing published is renamed.
    #[test]
    fn the_git_tag_keeps_the_name_it_was_pushed_under() {
        let found = held("an-update.md", "The only tag is v0.0.5, from 2026-09-20.");
        assert!(found.is_empty(), "{found:?}");
    }

    /// **A fenced block is a transcript, not a sentence.** A registry listing
    /// its own tags cannot be asked to call them images, and fifteen of the
    /// first sixty findings were that. The prose on either side is still read,
    /// which this fixture also shows by leaving the paragraph before it
    /// untouched and refusing nothing.
    #[test]
    fn quoted_output_is_not_this_repositorys_prose() {
        let text = "The place answered with its own names.\n\n```text\n\
                    it holds 4 names: [\"0.0.1\", \"sha256-d3f05b60\", \"0.0.2\", \"sha256-8f9c36e0\"]\n\
                    ```\n";
        assert!(
            held("an-update.md", text).is_empty(),
            "{:?}",
            held("an-update.md", text)
        );
        assert_eq!(
            paragraphs(text).len(),
            1,
            "only the sentence is prose: {:?}",
            paragraphs(text)
        );
    }

    /// **A sentence wraps, and a per-line check would call this a bare number.**
    /// That exact fault happened three times in one hour while this rule was
    /// being written, twice inside a guard meant to catch it. The word that
    /// saves this paragraph is on the line before the number.
    #[test]
    fn a_paragraph_is_joined_before_anything_is_matched() {
        let found = held(
            "a-plan.md",
            "Everything here waits on the image\n0.0.6 nobody has built yet.\n",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    /// **Another program's version number is not an alo image.** The first
    /// matcher took any digit-dot-digit-dot-digit and reported PipeWire,
    /// Ubuntu, Mesa and cosign. An alo image is the `0.0.N` series.
    #[test]
    fn another_programs_version_is_not_an_alo_image() {
        let found = held(
            "docs/quirks.md",
            "A KVM guest on Ubuntu 24.04.5 with PipeWire 1.0.5, Mesa 25.2.8 and cosign 3.1.3.",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    /// **A rule has to be able to state the form it forbids**, or it refuses
    /// `ROADMAP.md` and every document that quotes it.
    #[test]
    fn the_rule_may_quote_itself() {
        let found = held(
            "ROADMAP.md",
            "A milestone never carries a second zero — `v0.5`, never `v0.0.5`.",
        );
        assert!(found.is_empty(), "{found:?}");
    }
}
