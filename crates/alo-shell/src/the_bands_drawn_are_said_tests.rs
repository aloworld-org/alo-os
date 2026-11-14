//! What the journal is told about the bands over alo's own windows: a line
//! when a count changes, none when it does not, and again when a window
//! comes back.

use super::WhatTheBandsSaid;

/// The terminal's band drawing `count` controls, and nothing else open.
fn the_terminal_drawing(count: usize) -> WhatTheBandsSaid {
    WhatTheBandsSaid {
        terminal: Some(count),
        ..WhatTheBandsSaid::default()
    }
}

/// **The terminal opening at rest says so**, in the words the KVM walk reads.
#[test]
fn a_terminal_opening_at_rest_says_it_draws_none() {
    assert_eq!(
        WhatTheBandsSaid::default().lines_to(the_terminal_drawing(0)),
        vec!["window band over the terminal: 0 controls drawn".to_owned()]
    );
}

/// **A frame with nothing new says nothing**, so the journal is not a frame log.
#[test]
fn nothing_new_says_nothing() {
    assert!(
        the_terminal_drawing(0)
            .lines_to(the_terminal_drawing(0))
            .is_empty()
    );
    assert!(
        WhatTheBandsSaid::default()
            .lines_to(WhatTheBandsSaid::default())
            .is_empty()
    );
}

/// **Pointing at the band says so, and leaving it says so again.**
#[test]
fn revealing_and_resting_are_each_said() {
    assert_eq!(
        the_terminal_drawing(0).lines_to(the_terminal_drawing(2)),
        vec!["window band over the terminal: 2 controls drawn".to_owned()]
    );
    assert_eq!(
        the_terminal_drawing(2).lines_to(the_terminal_drawing(0)),
        vec!["window band over the terminal: 0 controls drawn".to_owned()]
    );
}

/// **A window that closes says nothing, and says again when it is back.**
#[test]
fn a_window_back_again_is_said_again() {
    assert!(
        the_terminal_drawing(0)
            .lines_to(WhatTheBandsSaid::default())
            .is_empty()
    );
    let tasks = WhatTheBandsSaid {
        tasks: Some(0),
        ..WhatTheBandsSaid::default()
    };
    assert_eq!(
        WhatTheBandsSaid::default().lines_to(tasks),
        vec!["window band over the task windows: 0 controls drawn".to_owned()]
    );
}

/// **The line the KVM walk waits for is the line this says** — read from the
/// walk's own source, `crates/alo-walking/src/the_first_walk.rs`, because the
/// walk does not build this crate and a copy kept in either would drift.
///
/// What would have to be true for this to pass while the walk is broken: the
/// walk would have to stop using `THE_TERMINAL_AT_REST` in its check, which
/// `alo-walking`'s own `the_terminal_check_reads_from_the_chord` holds.
#[test]
fn the_walk_waits_for_the_words_this_says() {
    let walk = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../alo-walking/src/the_first_walk.rs"),
    )
    .expect("the first walk's source is beside this crate");
    let declared = walk
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("pub const THE_TERMINAL_AT_REST: &str = \"")
                .and_then(|rest| rest.strip_suffix("\";"))
        })
        .expect("the walk declares THE_TERMINAL_AT_REST on one line");
    assert_eq!(
        WhatTheBandsSaid::default().lines_to(the_terminal_drawing(0)),
        vec![declared.to_owned()]
    );
}
