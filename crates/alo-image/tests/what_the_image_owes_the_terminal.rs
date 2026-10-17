//! **What alo's own terminal needs from the machine underneath it.**
//!
//! `alo_terminal`'s `WHAT_THIS_TERMINAL_IS_CALLED` tells every shell it starts
//! that it is on an `xterm-256color`. That is a claim about the machine as much
//! as about the terminal: a program reads the name, looks it up in a terminfo
//! database, and refuses to run when the lookup fails. `clear` answers *TERM
//! environment variable not set.* when there is no name, and something equally
//! unhelpful when there is a name nothing can resolve.
//!
//! So the name is only as good as the image carrying the database, and nothing
//! in `alo-terminal` can check that — a terminfo database is a property of an
//! installed system. This is where it can be checked.
//!
//! # What was measured, and what this test actually holds
//!
//! **Measured once, by hand, on 2026-10-17**: the pinned base was pulled and
//! asked, and it carries `/usr/share/terminfo/x/xterm-256color` and
//! `/usr/sbin/clear`.
//!
//! ```text
//! podman run --rm quay.io/fedora/fedora-bootc:42@sha256:077182b6… \
//!     ls /usr/share/terminfo/x/xterm-256color
//! ```
//!
//! **That measurement does not run again, and this test is not pretending it
//! does.** Pulling a base image needs a network and a container runtime, which
//! a gate has neither of reliably. What this holds instead is the one thing
//! that would make the measurement stale: **the base is pinned by digest**, so
//! it cannot change under the measurement without this line changing. When
//! somebody bumps the digest, this fails and says to measure again.
//!
//! *A measurement performed once and a check that runs for ever are different
//! things, and `CLAUDE.md` asks which one a report is. This is the first, with
//! the second built around it to say when the first expired.*

#![expect(
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic naming the file and the line is the failure being reported"
)]

use std::path::Path;

/// The base the terminfo database was measured on.
///
/// Changing the base means the measurement in this file's header is about
/// another image. Re-run the command there against the new one and put its
/// digest here.
const THE_BASE_THAT_WAS_MEASURED: &str = "quay.io/fedora/fedora-bootc:42@sha256:077182b6ba853b3348d0bede602ac30b9e6568c6422bcf0654de5af96f19b9c3";

/// What the recipe calls the base.
const THE_ARGUMENT: &str = "ARG THE_BASE=";

/// The recipe this repository builds its image from.
fn the_recipe() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("image")
        .join("Containerfile");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} did not read: {why}", at.display()))
}

/// **The base is still the one the terminfo database was measured on.**
///
/// Not a check that terminfo is there — see this file's header for why it
/// cannot be, and for the measurement that found it. This is the expiry date on
/// that measurement.
#[test]
fn the_base_is_the_one_the_terminal_was_measured_against() {
    let recipe = the_recipe();
    let named = recipe
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(THE_ARGUMENT))
        .expect("the recipe names a base for the image");

    assert_eq!(
        named, THE_BASE_THAT_WAS_MEASURED,
        "the image's base has changed since anybody asked it whether it carries a terminfo \
         database. alo's own terminal tells every shell it starts that it is on an \
         `xterm-256color`, and a name nothing can resolve leaves `clear`, `less`, `vim` and \
         `top` refusing to run. Pull the new base, run `ls \
         /usr/share/terminfo/x/xterm-256color` in it, and when it is there put the new digest \
         in this test beside the measurement in its header."
    );
}

/// **The base is pinned by digest, which is what makes the measurement hold.**
///
/// A floating tag would let the image change under the measurement without a
/// line of this repository changing, and the test above would go on passing
/// while saying nothing. The pin is the mechanism; this holds the mechanism.
#[test]
fn the_base_is_pinned_by_digest_and_not_by_a_tag() {
    let recipe = the_recipe();
    let named = recipe
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(THE_ARGUMENT))
        .expect("the recipe names a base for the image");

    assert!(
        named.contains("@sha256:"),
        "the image's base is {named:?}, which is a tag rather than a digest. A tag moves, so \
         anything measured about the base — the terminfo database alo's terminal needs, among \
         the rest — stops being about the image this repository builds."
    );
}
