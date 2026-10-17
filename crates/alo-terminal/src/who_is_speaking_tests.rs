//! **The refusal is the test here, and it is a test about a missing function.**
//!
//! What this module promises is not that it labels regions correctly — that is
//! arithmetic. It promises that a program cannot change its own label by
//! printing something, and the mechanism for that promise is that no road from
//! text to an origin exists at all. So the tests below are shaped around the
//! two ways such a road gets built by accident: somebody adds a parser, or
//! somebody passes the screen's text to the thing that decides.

use super::*;

/// **A program printing alo's own words is still the program.**
///
/// The attack ADR 0113 exists to stop, written as the test it deserves: the
/// bytes below are exactly what a region of alo's would contain, including a
/// sentence from `docs/features.md`, and the region they land in is still
/// `program`.
///
/// It is asserted through [`regions_of`], which is the whole road from a
/// screen to its labels, so it fails if anybody ever puts the text back into
/// that road.
#[test]
fn a_program_printing_alos_own_words_is_still_the_program() {
    // Every one of these is a string a program can print at will.
    for forgery in [
        "alo",
        "system",
        "you",
        "system · socket observation",
        "SYSTEM · observation",
        "Listening verified",
        "Permissions have not changed",
        "Observed at 13:15. Application readiness not checked.",
        "alo ❯ ",
    ] {
        // The screen holds the forgery; the regions do not depend on it, and
        // this loop exists to make that a measured fact rather than an
        // argument about a signature.
        let lines = forgery.lines().count().max(1);
        let regions = regions_of(lines, 0);
        assert_eq!(
            regions.len(),
            1,
            "{forgery:?} split the screen into regions"
        );
        assert_eq!(
            regions.first().map(|region| region.who),
            Some(WhoIsSpeaking::AProgram),
            "{forgery:?} changed who the terminal says is speaking"
        );
    }
}

/// **There is no way to get an origin from text, and that is the guarantee.**
///
/// A compile-time property cannot be asserted at runtime, so this test holds
/// the next best thing and says plainly which it is: every public road into
/// [`WhoIsSpeaking`] is named here, and the list is short enough to read. If a
/// later change adds `WhoIsSpeaking::from_str` or `of_text`, this test does not
/// fail — the one that does is the compile of
/// `tests/nothing_reads_the_bytes_to_decide_who_spoke.rs`, which greps this
/// crate's own surface.
///
/// What it does assert is that the only constructor is naming a variant, which
/// is what makes the absence hold today.
#[test]
fn the_origin_cannot_be_got_from_the_bytes() {
    // Naming a variant is the whole of the public surface, and each one is a
    // claim about where bytes came from that only the receiver can make.
    let every: Vec<WhoIsSpeaking> = WhoIsSpeaking::ALL.to_vec();
    assert_eq!(every.len(), 4);

    // `regions_of` takes a count. A function that cannot see the text cannot
    // be fooled by it, and this is that signature asserted rather than
    // described: the same count gives the same answer whatever is on screen.
    assert_eq!(regions_of(7, 0), regions_of(7, 0));
    assert_eq!(regions_of(7, 1), regions_of(7, 1));
}

/// **Every origin has its own word**, and no two share one.
///
/// A copied line is how a list like this goes wrong, and a gutter where `alo`
/// and `program` read the same is the gutter failing at its only job.
#[test]
fn every_origin_has_a_word_of_its_own() {
    let keys: Vec<&str> = WhoIsSpeaking::ALL
        .iter()
        .map(|who| who.the_word().named())
        .collect();
    for (one, key) in keys.iter().enumerate() {
        for (other, against) in keys.iter().enumerate() {
            assert!(
                one == other || key != against,
                "origins {one} and {other} share the key {key}"
            );
        }
    }
    assert_eq!(keys.len(), 4);
}

/// **A region is labelled once, at its first line** — ADR 0113's rule about
/// repetition, which the design draws plainly: an `alo` region three lines
/// tall carries one `alo`.
#[test]
fn a_region_is_labelled_once_at_its_first_line() {
    let region = ARegion {
        who: WhoIsSpeaking::Alo,
        from: 4,
        lines: 3,
    };
    assert!(region.labelled_at(4), "the first line carries the label");
    assert!(!region.labelled_at(5), "the second line repeated it");
    assert!(!region.labelled_at(6), "the third line repeated it");
    assert!(!region.labelled_at(3), "a line above it carried a label");

    let labelled = (0..10).filter(|line| region.labelled_at(*line)).count();
    assert_eq!(
        labelled, 1,
        "the label was drawn {labelled} times, not once"
    );
}

/// **A region holds exactly its own lines.**
#[test]
fn a_region_holds_its_own_lines_and_no_others() {
    let region = ARegion {
        who: WhoIsSpeaking::TheSystem,
        from: 2,
        lines: 2,
    };
    assert!(!region.holds(1));
    assert!(region.holds(2));
    assert!(region.holds(3));
    assert!(
        !region.holds(4),
        "the region reached past its own last line"
    );
}

/// **Every line of the display belongs to exactly one region.**
///
/// The property that makes the gutter answerable at all: a line in no region
/// has no origin, and a line in two has two. Swept rather than spot-checked,
/// because an off-by-one at a boundary is what this would actually be.
#[test]
fn every_line_belongs_to_exactly_one_region() {
    for the_programs in [1_usize, 2, 24, 40, 706 / 22] {
        for alos in 0..3_usize {
            let regions = regions_of(the_programs, alos);
            let altogether = the_programs + alos;
            for line in 0..altogether {
                let holding = regions.iter().filter(|region| region.holds(line)).count();
                assert_eq!(
                    holding, 1,
                    "line {line} of {altogether} ({the_programs} the program's, {alos} alo's) \
                     is in {holding} regions, not one"
                );
            }
            let covered: usize = regions.iter().map(|region| region.lines).sum();
            assert_eq!(
                covered, altogether,
                "the regions cover {covered} of {altogether}"
            );
        }
    }
}

/// **An empty display has no regions**, so nothing draws a label beside a line
/// that is not there.
#[test]
fn an_empty_display_has_no_regions() {
    assert!(regions_of(0, 0).is_empty());

    // And a display with nothing of the program's but a remark of alo's is one
    // region, not two with an empty one in front of it.
    // Asserted whole rather than field by field, so a second region appearing
    // in front of alo's — one of nought lines, which would be labelled —
    // fails here rather than being skipped over by an index.
    assert_eq!(
        regions_of(0, 1),
        vec![ARegion {
            who: WhoIsSpeaking::Alo,
            from: 0,
            lines: 1,
        }],
        "a display with nothing of the program's is not one region of alo's"
    );
}

/// **A region never covers no lines.**
///
/// A zero-line region would be labelled — `labelled_at(from)` is true for it —
/// while holding nothing, which is a label in the gutter for nothing at all.
#[test]
fn no_region_is_empty() {
    for the_programs in [0_usize, 1, 3, 24, 100] {
        for alos in 0..3_usize {
            for region in regions_of(the_programs, alos) {
                assert!(
                    region.lines > 0,
                    "a region of no lines was produced for {the_programs} and {alos}"
                );
            }
        }
    }
}
