//! **No agent can ask for an undo to be forgotten**, now or later — walked by
//! the compiler and by this suite rather than remembered.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s
//! seventh term asks for exactly this, and gives the reason that is stronger
//! than surface area: **an agent that can forget an undo can erase the evidence
//! of what it did.** Undo is the record's counterpart — ADR 0001 §5 holds that
//! a change waits for one approval and is written down — and a destructive verb
//! over the written-down past is the one verb whose approval a person is least
//! able to judge, because what it destroys is the thing they would judge it by.
//!
//! So this asks `alo_broker`'s own closed list rather than keeping a second
//! copy of it here that could quietly come to disagree with it, and it asks
//! this crate's own manifest for the shape of the thing it cannot be.

#![expect(
    clippy::panic,
    reason = "in a test, a panic naming what could not be read is the failure being reported"
)]

use std::collections::BTreeSet;

/// How many verbs the broker's list has, and the number every change to it has
/// to move on purpose.
///
/// It was eleven when this was written, which is what ADR 0053's amendment
/// asked of the change that wrote it: *`crates/alo-broker/src/verbs.rs` gains
/// nothing. `SystemVerb` is a closed enum and stays closed here.* That sentence
/// was about **undo**, and it still is: nothing on the list may be a verb over
/// the written-down past, which is what the two tests below actually hold.
///
/// **Twelve since 2026-09-22**, for `starting.windows-next` — setting the
/// firmware's next start to the Windows already on the disk, for one start.
/// [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
/// names it as the alo OS half of the two one-restart switches, and task 16 of
/// `docs/autonomy/v0-5-the-installer-plan.md` says in as many words that the
/// verb is added and gets the tests every other verb has. It takes nothing away
/// from anybody's undo: it writes one firmware variable, reads no record, and
/// removes nothing.
///
/// Moving this number is what a verb costs. It is a line in this file, in the
/// change that adds the verb, with the decision it rests on named beside it —
/// which is the point: the list cannot grow quietly.
const AS_MANY_AS_IT_HAD: usize = 12;

/// **No name on the broker's list begins `undo.`** — the sentence ADR 0045's
/// seventh term asks a test to hold, over the one list there is.
#[test]
fn no_verb_on_the_brokers_list_begins_undo() {
    for name in alo_broker::EVERY_NAME {
        assert!(
            !name.starts_with("undo."),
            "{name} is a verb over the written-down past"
        );
    }
}

/// **And the list gained nothing at all.** A verb named something else that did
/// the same thing would keep the letter of the rule and lose the whole of it,
/// so the count is held too: this change adds no verb, and neither may the next
/// one without moving this number and saying why in an ADR.
#[test]
fn the_brokers_list_is_exactly_as_long_as_it_was() {
    let names: BTreeSet<&str> = alo_broker::EVERY_NAME.into_iter().collect();
    assert_eq!(names.len(), AS_MANY_AS_IT_HAD);
    assert_eq!(alo_broker::EVERY_NAME.len(), AS_MANY_AS_IT_HAD);
    for name in alo_broker::EVERY_NAME {
        assert!(
            !name.contains("undo") && !name.contains("snapshot") && !name.contains("expire"),
            "{name} reads as a verb over what an undo would put back"
        );
    }
}

/// **This crate cannot be asked for anything.** It depends on no door, no
/// capability and no agent service — read off its manifest as text, which is
/// the stricter reading and the right one: *nothing here can be asked for* is a
/// promise about a file somebody audits, not about a link graph they would have
/// to compute.
#[test]
fn nothing_here_can_be_asked_for() {
    // The comments are where the manifest says what it deliberately does not
    // depend on, so what is read here is the lines that declare something.
    let manifest: String = include_str!("../Cargo.toml")
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n");
    for never in [
        "alo-agentd",
        "alo-capability",
        "alo-corridor",
        "alo-declared",
        "alo-granted",
        "alo-turn",
    ] {
        assert!(
            !manifest.contains(never),
            "this crate names {never}, and expiry is housekeeping rather than a petition"
        );
    }
    // `alo-broker` is here, and only as a dev-dependency: this suite asks its
    // list and nothing shipped ever reaches it.
    let (shipped, tested) = manifest
        .split_once("[dev-dependencies]")
        .unwrap_or((&manifest, ""));
    assert!(!shipped.contains("alo-broker"), "{shipped}");
    assert!(tested.contains("alo-broker"), "{tested}");
}

/// **`alo-turn` gains no capability**, which is the other half of the seventh
/// term: it runs as the person, and a capability granted to the thing that
/// takes the brackets would be one held all day for an act performed once a
/// day. Its unit is not this crate's to write, so what is held here is that
/// nothing in `alo-turn` asks for one and nothing in this crate hands it one.
#[test]
fn alo_turn_gains_no_capability() {
    let theirs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../alo-turn/Cargo.toml");
    let manifest = std::fs::read_to_string(&theirs).unwrap_or_else(|why| {
        panic!("{} could not be read: {why}", theirs.display());
    });
    for never in ["caps", "capctl", "capsh", "alo-letting-go"] {
        assert!(
            !manifest.contains(never),
            "alo-turn names {never}, and it holds no capability"
        );
    }
}
