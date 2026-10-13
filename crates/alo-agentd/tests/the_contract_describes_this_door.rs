//! `docs/contracts/application-door.md` describes the door this daemon knocks
//! at, and a test is what keeps that true.
//!
//! # Why this exists rather than a shared constant
//!
//! Two crates spell the same two names: `alo_shell`'s
//! `the_application_doors_place` serves the door and
//! `alo_agentd::at_the_compositors_door` knocks at it. **Neither may depend on
//! the other** — `alo-shell` is a Wayland compositor and a daemon that linked
//! one to learn a filename would be unbuildable anywhere a compositor is not.
//!
//! So the name is written twice, which is the shape this repository distrusts:
//! *two constants agree with each other whatever either of them says*. What
//! makes it safe is that **neither is held to the other**. Both are held to the
//! contract document, which is also the thing a third party building a
//! compositor of their own reads. This is that check on the daemon's side;
//! `alo-shell` has its own.
//!
//! # The phrases are looked for with whitespace normalised
//!
//! The document is hard-wrapped, so a phrase that spans a line break is not
//! findable by a plain search — a `0` would then mean *the document was
//! wrapped*, not *the document does not say this*. Every line is joined and
//! runs of whitespace collapsed before anything is looked for, and there is a
//! control below that must be absent so a hit proves the search ran.

// Counted: one `panic!` and no `unwrap` or `expect`, so only `panic` is
// listed — an unfulfilled expectation is an error in this workspace.
#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_agentd::at_the_compositors_door::{AtTheCompositorsDoor, THE_COMPOSITORS_SOCKET};

/// The contract, as one line with runs of whitespace collapsed.
fn the_contract() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("contracts")
        .join("application-door.md");
    let text = std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("the contract is where it says, at {}: {why}", at.display()));
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// **The contract is searchable at all**, which every assertion below rests on.
///
/// A control that must be found and a control that must not: without the first,
/// a document that failed to read would make every check below pass by finding
/// nothing to contradict; without the second, a search that matched anything
/// would too.
#[test]
fn the_contract_can_be_read_and_searched() {
    let contract = the_contract();
    assert!(
        contract.contains("what `alo-agentd` asks the compositor about an application"),
        "the contract was read but is not the document this test is about"
    );
    assert!(
        !contract.contains("knuckle.sock"),
        "the search matches text the document does not contain"
    );
}

/// **The socket's name is the contract's.**
#[test]
fn the_socket_is_named_what_the_contract_publishes() {
    let contract = the_contract();
    assert_eq!(THE_COMPOSITORS_SOCKET, "shell.sock");
    assert!(
        contract.contains("/run/alo/<uid>/shell.sock"),
        "the contract does not publish the path this daemon knocks at"
    );
    assert!(
        contract.contains("`shell.sock` for the socket"),
        "the contract does not name the socket this daemon knocks at"
    );
}

/// **The path is the contract's**, assembled the way this daemon assembles it.
///
/// Asserted on the real type rather than on the constant, because what matters
/// is where it would knock and not what it was told to call the file.
#[test]
fn the_path_is_the_one_the_contract_publishes() {
    let door = AtTheCompositorsDoor::beside(Path::new("/run/alo/1000"));
    assert_eq!(door.socket(), PathBuf::from("/run/alo/1000/shell.sock"));
}

/// **The contract says the directory is the daemon's and the socket is the
/// compositor's**, which is why this daemon appends a filename and makes
/// nothing.
#[test]
fn the_contract_says_who_owns_each_part_of_the_path() {
    let contract = the_contract();
    for said in [
        "is the **daemon's**, made when that person's session starts",
        "is the **compositor's**, made when it starts",
        "The compositor never creates it and never sets its mode",
    ] {
        assert!(
            contract.contains(said),
            "the contract no longer says: {said}"
        );
    }
}

/// **The contract says the socket is `0600` and why**, which is the whole of
/// what keeps the agent out: the daemon's own socket is `0660` because the
/// agent must reach it, and this one is handed to nobody.
#[test]
fn the_contract_says_the_socket_is_shut_to_everybody_but_the_person() {
    let contract = the_contract();
    assert!(
        contract.contains("The socket is `0600`"),
        "the contract no longer publishes the mode the door depends on"
    );
    assert!(
        contract.contains("only the person's own uid may connect"),
        "the contract no longer says who may reach the door"
    );
}

/// **The contract says one line is one act, with no session** — which is why
/// this daemon makes a connection per request and keeps none.
#[test]
fn the_contract_says_one_line_is_one_act() {
    let contract = the_contract();
    assert!(
        contract.contains("One line is one act"),
        "the contract no longer says a connection carries one act"
    );
}

/// **The contract publishes the three acts and the five answers**, which are
/// the words `alo-acting` reads and writes. If the document grew a sixth, this
/// daemon would be reading a wire it does not know — and
/// `alo_acting::Answer::read` deliberately answers `None` rather than success
/// for a word it has not met.
#[test]
fn the_contract_publishes_the_words_this_daemon_knows() {
    let contract = the_contract();
    for word in [
        "`open`",
        "`focus`",
        "`close`",
        "`acted`",
        "`no-window`",
        "`not-installed`",
        "`it-did-not`",
        "`could-not-ask`",
    ] {
        assert!(
            contract.contains(word),
            "the contract no longer publishes {word}, which this daemon reads"
        );
    }
    // And the fourth verb is published as *absent from this wire*, not as one
    // of the acts — so a reader cannot take it for a fourth word.
    assert!(
        contract.contains("`arrange`"),
        "the contract no longer says what happened to the fourth verb"
    );
}
