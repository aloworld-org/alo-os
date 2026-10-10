//! **Placement has a production caller, and it runs in the right order.**
//!
//! `crate::where_a_window_opens` implements the owner's
//! `docs/design/where-a-new-window-opens.md` and, until 2026-10-09, **nothing
//! called it**. `new_toplevel` gave a window a Place and no position, so every
//! window opened at the plane's origin and the second opened exactly on the
//! first — `docs/features.md`'s `[v0.01] ★ Every goal is a canvas` promises
//! *nothing is stacked*, and the machine broke it on the second window.
//!
//! # Why this is a source-level guard and not a behaviour test
//!
//! Two reasons, and the second is the owner's own bar.
//!
//! The placer's rules are tested where they are implemented, against
//! rectangles, and those tests passed every day while no window was ever
//! placed. **A function with no caller passes all of its own tests.** That is
//! the whole failure this file is about, and no assertion about rectangles can
//! see it — only asking whether production reaches them can.
//!
//! The owner's bar for this work was *an exported function with no caller is
//! not evidence of working behaviour*. The evidence that a person's two
//! windows land side by side is the production demonstration on real hardware,
//! which is the third PC's to run and is **not** this file. What this file
//! holds is the part a demonstration cannot: that the call stays, and stays
//! **where** the argument for it put it.
//!
//! # The order is the claim, not the presence
//!
//! Three things must already have happened when a window is placed, and one
//! must not have. Each is a line below, because each is a way for this to
//! break while still compiling and still being called:
//!
//! - **after the remembered places are put back**, so a window being restored
//!   keeps where the person left it. The contract's rules 1 and 2 are kept by
//!   this order and by nothing else — move the call above it and a restored
//!   window is placed as though it were new.
//! - **after this frame's fixed controls are recorded**, because at the top of
//!   `present` those bounds are the previous frame's, and on the first frame
//!   there are none. A window would be placed against a Dock nobody had laid
//!   out.
//! - **before the frame is composited**, so the window is drawn where it
//!   belongs on its first frame and nobody sees it at the origin.

#![expect(
    clippy::panic,
    reason = "an unreadable source file here is the failure this test reports, and a formatted \
              panic names what it was looking for"
)]

use std::path::Path;

/// This crate's source directory.
fn src() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// One source file's text.
fn read(name: &str) -> String {
    let path = src().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|why| panic!("{name} must be readable: {why}"))
}

/// The part of a file that ships, cut at its first `#[cfg(test)]`.
///
/// **A call from a test is not a caller.** This is the distinction the whole
/// file rests on: `where_a_window_opens` had ten passing tests and no
/// production caller, and a search that counted both would have found eleven
/// and reported it wired.
fn production_part_of(text: &str) -> &str {
    text.find("#[cfg(test)]").map_or(text, |at| &text[..at])
}

/// The text with its comment lines gone.
///
/// **A mention is not a call, and this test proved it on itself.** The first
/// version searched for `render_each_display` and failed — because the comment
/// explaining *the frame has not been composited yet, `render_each_display` is
/// below* sits **above** the call it describes. The name's first appearance was
/// in prose about the ordering, so the test compared the placement against a
/// sentence rather than against the composite, and reported the order wrong
/// when the order was right.
///
/// Two defences rather than one, because this file is about ordering and a
/// wrong index fails in the direction that looks like a real regression:
/// comment lines go, **and** every marker below is written as a call, with its
/// receiver and its opening parenthesis.
fn without_comments(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where a call first appears, refusing to guess if it does not.
fn at(text: &str, marker: &str, file: &str) -> usize {
    text.find(marker).unwrap_or_else(|| {
        panic!(
            "{file} no longer contains the call {marker:?}, so the order this test checks \
             cannot be read — the test is wrong, or the thing it guards has been renamed"
        )
    })
}

/// Where a call **last** appears.
///
/// Used for the fixed controls, which are handed over once per display: the
/// claim is that *all* of this frame's controls are recorded before a window
/// is placed, so the last of them is the one that has to come first.
fn at_the_last(text: &str, marker: &str, file: &str) -> usize {
    text.rfind(marker)
        .unwrap_or_else(|| panic!("{file} no longer contains the call {marker:?}"))
}

#[test]
fn the_placer_is_called_from_production_and_not_only_from_tests() {
    let desk = read("direct_desktop.rs");
    let shipped = &without_comments(production_part_of(&desk));
    // The positive control, in the same assertion's scope: a marker this file
    // is certain of. Without it, a walk that read an empty string would report
    // every absence below as a clean pass.
    assert!(
        shipped.contains("fn present<T: crate::direct_loop::LoopTarget"),
        "direct_desktop.rs's production part no longer holds the frame loop, so nothing below \
         is measuring what it claims"
    );
    assert!(
        shipped.contains("server.place_every_window_that_just_opened("),
        "nothing in the desktop's production path places a window that just opened. This is \
         exactly the state the repository was in until 2026-10-09: where_a_window_opens \
         implemented, tested, and reached by nobody, while every window opened at the origin."
    );
    let caller = read("a_new_window_is_placed.rs");
    assert!(
        production_part_of(&caller).contains("crate::where_a_window_opens("),
        "a_new_window_is_placed.rs no longer calls the placer, so the caller is a caller in \
         name only"
    );
}

#[test]
fn a_window_is_placed_after_a_remembered_place_has_been_offered() {
    let desk = read("direct_desktop.rs");
    let shipped = &without_comments(production_part_of(&desk));
    let put_back = at(
        shipped,
        "server.put_back_where_it_was(",
        "direct_desktop.rs",
    );
    let placed = at(
        shipped,
        "server.place_every_window_that_just_opened(",
        "direct_desktop.rs",
    );
    assert!(
        put_back < placed,
        "a window is placed before the place it was left in is offered, so restoring a window \
         now moves it somewhere new. The contract's rule 1 is *explicit placement wins* and \
         rule 2 is *restoring is different from opening*; both are kept by this order."
    );
}

#[test]
fn a_window_is_placed_against_this_frames_controls_and_before_the_frame_is_drawn() {
    let desk = read("direct_desktop.rs");
    let shipped = &without_comments(production_part_of(&desk));
    let controls = at_the_last(
        shipped,
        "server.the_fixed_controls_were_drawn(",
        "direct_desktop.rs",
    );
    let placed = at(
        shipped,
        "server.place_every_window_that_just_opened(",
        "direct_desktop.rs",
    );
    let drawn = at(shipped, "server.render_each_display(", "direct_desktop.rs");
    assert!(
        controls < placed,
        "a window is placed before this frame's fixed controls are recorded, so it is placed \
         against the previous frame's bounds — and against none at all on the first frame, \
         which is the frame the first window opens on."
    );
    assert!(
        placed < drawn,
        "a window is placed after the frame is composited, so its first frame shows it at the \
         plane's origin and it moves under the person's eyes on the second."
    );
}
