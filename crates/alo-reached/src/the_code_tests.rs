//! The split, asked about each of the five things that look like a caller and
//! are not.

use super::{split_off_test_blocks, without_comments, without_use_lines};

/// **A doc comment naming a function is not a caller.**
///
/// In this repository the prose and the thing are always adjacent, so counting
/// comments makes every documented function look reached.
#[test]
fn comments_are_not_code() {
    let code = "/// calls the_thing\n// and the_thing again\nfn other() {}\n";
    let left = without_comments(code);
    assert_eq!(left.matches("the_thing").count(), 0, "{left:?}");
    assert!(
        left.contains("fn other"),
        "the code itself survives: {left:?}"
    );
}

/// **A trailing comment goes and the code before it stays.**
#[test]
fn a_comment_after_code_takes_only_itself() {
    let left = without_comments("let a = b(); // calls the_thing\n");
    assert!(left.contains("b()"), "{left:?}");
    assert_eq!(left.matches("the_thing").count(), 0, "{left:?}");
}

/// **A block comment goes, including nested, which Rust allows.**
#[test]
fn block_comments_go_including_nested() {
    let left = without_comments("a(); /* the_thing /* inner the_thing */ */ b();\n");
    assert_eq!(left.matches("the_thing").count(), 0, "{left:?}");
    assert!(left.contains("a()") && left.contains("b()"), "{left:?}");
}

/// **A `pub use` is a declaration, not a call.**
///
/// Hole three: nearly every crate re-exports its own surface in `lib.rs`, so
/// counting `use` lines flattered every crate at once.
#[test]
fn use_lines_are_not_calls() {
    let code = "use crate::the_thing;\npub use other::the_thing;\nfn f() { something(); }\n";
    let left = without_use_lines(code);
    assert_eq!(left.matches("the_thing").count(), 0, "{left:?}");
    assert!(left.contains("something()"), "{left:?}");
}

/// **A `#[cfg(test)]` block is lifted out of the code and kept as test text.**
///
/// This is hole five, and this crate's own: the first rewrite of the hand sweep
/// stripped these blocks and **threw the text away**, so a function reached only
/// by its own module's tests was counted in neither place and reported as an
/// orphan. Three functions in `alo-access` were falsely accused that way.
#[test]
fn a_cfg_test_block_is_moved_to_the_tests_and_not_dropped() {
    let code = "fn f() {}\n#[cfg(test)]\nmod tests {\n    fn t() { the_thing(); }\n}\nfn g() {}\n";
    let (kept, tests) = split_off_test_blocks(code);

    assert_eq!(
        kept.matches("the_thing").count(),
        0,
        "the test's call is not a product caller: {kept:?}"
    );
    assert_eq!(
        tests.matches("the_thing").count(),
        1,
        "and it is not lost either, which is the whole of hole five: {tests:?}"
    );
    assert!(
        kept.contains("fn f") && kept.contains("fn g"),
        "the code either side of the block survives: {kept:?}"
    );
}

/// **A `#[cfg(test)]` on something with no braces loses nothing.**
#[test]
fn a_cfg_test_with_no_block_keeps_the_rest_of_the_file() {
    let code = "#[cfg(test)]\nuse std::fmt;\nfn after() { kept_call(); }\n";
    let (kept, _) = split_off_test_blocks(code);
    assert!(
        kept.contains("kept_call"),
        "everything after it is still there: {kept:?}"
    );
}
