//! **The order of the canvas walk**, in one place because two things read it.
//!
//! Task 10's acceptance asks that *the table is read from the report rather than a
//! copy, so a step that changes without the table fails.* That needs the walk's
//! own order to be a value rather than ten string literals buried in the fixture:
//! `crate::the_canvas_walk_check` labels each raster from this list, and
//! `crates/alo-shell/tests/the_canvas_walked.rs` holds this list against the table
//! in `docs/autonomy/updates/the-canvas-walked.md`.
//!
//! **Only the order lives here.** Where the report is and what its heading says
//! are the test's business, kept in the test: a constant included in two places
//! and used in one is dead code in the other, which the clippy gate denies — and
//! it was, the first time this file existed.
//!
//! **Its own file, included by both**, rather than the test reaching into the
//! fixture. The fixture needs a Wayland parent and a GLES context to exist at all;
//! a test that pulled it in to read one constant would drag those requirements
//! into the ordinary suite, and the sequence is the half that is meant to run
//! anywhere.
//!
//! The six steps before the divider are the plan's sentence in its order — *open
//! three applications, drag one, resize another, pan, zoom out to Show all, zoom
//! back into one and work in it*. The four after it are *the same by keyboard
//! alone*, as far as v0.5 promises a keyboard form: **dragging and resizing have
//! none**, which is why four follow six rather than six following six, and the
//! report says so in the row where a person would look for them.

/// Every moment of the walk, in the order a person meets them.
pub const EVERY_MOMENT: [&str; 10] = [
    "three applications open on the canvas",
    "one of them dragged by its name",
    "another resized from its corner",
    "the canvas panned",
    "zoomed out to show all of it",
    "zoomed back into one and worked in it",
    "panned by the arrow keys, with no pointer",
    "show all again, from the keyboard",
    "zoomed back in from the keyboard",
    "a frame reached and focused with no pointer",
];
