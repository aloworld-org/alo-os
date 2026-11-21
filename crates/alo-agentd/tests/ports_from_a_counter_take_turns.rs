//! **The tests that take a port from a counter run one at a time.**
//!
//! `listeners.rs` and `responding.rs` each hand their tests ports from a
//! counter that starts at a fixed number, because asking the kernel for port
//! zero raced on 2026-09-19 and 2026-09-20 (their own comments say how). A
//! counter cannot hand one number out twice **within one process**, which is
//! all it was ever checked against under `cargo test`, where a binary's tests
//! are threads.
//!
//! The gate runs nextest, and nextest runs **every test in a process of its
//! own**. So every one of those tests starts the counter at the same number, and
//! two of them running at once are handed the same port. Measured on
//! 2026-11-21, once the gate moved to a 16-core runner and ran more of them at
//! once: `a_let_go_that_leaves_the_port_held_is_still_refused_and_not_said_again`
//! and `a_network_that_goes_is_let_go_of_and_one_that_comes_is_answered_on`
//! failed with `Address already in use` in four of the five failed gate runs
//! that morning, on code nobody had touched.
//!
//! The answer is `.config/nextest.toml`'s `ports-from-a-counter` group, which
//! runs them one at a time. This test fails when a file whose tests take ports
//! from such a counter is not in that group's filter, in each profile.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// What a file whose tests hand out ports from a counter says.
const A_COUNTER_OF_PORTS: &str = "static NEXT: AtomicU16";

/// The group that runs them one at a time.
const THE_GROUP: &str = "test-group = 'ports-from-a-counter'";

fn every_file(at: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_file(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs") {
            into.push(path);
        }
    }
}

/// The module paths whose tests take a port from a counter.
fn what_takes_ports_from_a_counter() -> Vec<String> {
    let mut files = Vec::new();
    every_file(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(files.len() > 50, "only {} files were read", files.len());
    let mut modules: Vec<String> = files
        .iter()
        .filter(|file| {
            std::fs::read_to_string(file).is_ok_and(|text| text.contains(A_COUNTER_OF_PORTS))
        })
        .filter_map(|file| Some(file.file_stem()?.to_str()?.to_owned()))
        .collect();
    modules.sort();
    modules
}

/// The filter of each override that puts tests in the group, one per profile.
fn the_groups_filters() -> Vec<String> {
    let config = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.config/nextest.toml"),
    )
    .expect(".config/nextest.toml can be read");
    let lines: Vec<&str> = config.lines().map(str::trim).collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == THE_GROUP)
        .filter_map(|(at, _)| {
            lines
                .get(at.checked_sub(1)?)
                .and_then(|line| line.strip_prefix("filter = "))
                .map(str::to_owned)
        })
        .collect()
}

/// **Every file whose tests take ports from a counter is in the group, in each
/// profile a person or the gate runs.**
#[test]
fn every_test_that_takes_a_port_from_a_counter_takes_its_turn() {
    let modules = what_takes_ports_from_a_counter();
    assert!(
        modules.iter().any(|it| it == "listeners") && modules.iter().any(|it| it == "responding"),
        "the two files measured on 2026-11-21 were not found: {modules:?}"
    );
    let filters = the_groups_filters();
    assert_eq!(
        filters.len(),
        2,
        "the default and gates profiles each put these in the group: {filters:?}"
    );
    for filter in &filters {
        for module in &modules {
            assert!(
                filter.contains(&format!("test(/^{module}::tests::/)")),
                "{module}'s tests take ports from a counter and are not in the group: {filter}"
            );
        }
    }
}
