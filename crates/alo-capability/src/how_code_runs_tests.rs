//! **Law 5 as checks rather than as prose.**
//!
//! *The person chooses. alo never takes the choice away.* A type that could
//! only tighten, or a level that could not be left, or a protection that turned
//! out to depend on the level, would each break that law quietly — so each is
//! asserted here rather than described in the file above.

use super::*;

/// **A fresh machine runs the agent's code in a sealed box.**
///
/// ADR 0064 names the default, and the default is the strict reading.
/// `CLAUDE.md`: *where a flag chooses between the strict and the loose reading,
/// the strict one is the default: a flag that fails open is a flag that will be
/// forgotten in the direction that matters.*
#[test]
fn a_fresh_machine_runs_code_in_a_sealed_box() {
    assert_eq!(HowCodeRuns::default(), HowCodeRuns::InASealedBox);
    assert!(HowCodeRuns::default().the_kernel_confines_it());
    assert!(!HowCodeRuns::default().runs_without_asking());
}

/// **Nothing here can refuse a level, and that is law 5's real exposure.**
///
/// The first version of this test assigned one level to a variable and then
/// another, and asserted the variable held the second. It passed, it could not
/// fail, and the compiler said so — *value assigned to `chosen` is never read*.
/// A plain enum has no transitions to test; **any value can be constructed, so
/// there is nothing a test of this file can observe being refused.**
///
/// What law 5 is actually exposed to is a *later* change: a `change_to` that
/// returns a `Result`, a level that may only be entered under a condition, a
/// downgrade applied quietly. Each would read as prudence and each would be a
/// wall. So this reads the module's own source and refuses the shapes a refusal
/// would have to take, which is the same guard
/// `alo-terminal`'s `nothing_reads_the_bytes_to_decide_who_spoke` applies to
/// ADR 0113's absence.
#[test]
fn nothing_in_this_module_can_refuse_a_level() {
    let source = the_module();

    // A positive control first: the search can see this file's real
    // signatures, so an empty answer below means absence and not a filter that
    // never matched.
    let signatures: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("pub const fn ") || line.starts_with("pub fn "))
        .collect();
    assert!(
        signatures.len() >= 5,
        "found {} signatures, so the filter is not reading this file",
        signatures.len()
    );
    assert!(
        signatures.iter().any(|line| line.contains("what_it_costs")),
        "the search cannot see a function this file certainly has"
    );

    for refusing in signatures {
        assert!(
            !refusing.contains("Result<"),
            "`{refusing}` can fail, so some road here may refuse a person their choice"
        );
    }
    for forbidden in [
        "fn change_to",
        "fn may_choose",
        "fn is_allowed",
        "fn downgrade",
    ] {
        assert!(
            !source.contains(forbidden),
            "`{forbidden}` would let something stand between a person and their own choice"
        );
    }
}

/// The module this file is about, read off the disk it is compiled from.
fn the_module() -> String {
    let path: std::path::PathBuf = [env!("CARGO_MANIFEST_DIR"), "src", "how_code_runs.rs"]
        .iter()
        .collect();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", path.display()))
}

/// **Only full trust runs without asking**, and only the sealed box is confined
/// by the kernel.
///
/// Both halves, because a type that answered `true` to everything would pass
/// either one alone.
#[test]
fn each_level_is_what_it_says_and_not_the_others() {
    let runs_without_asking: Vec<HowCodeRuns> = HowCodeRuns::ALL
        .into_iter()
        .filter(|level| level.runs_without_asking())
        .collect();
    assert_eq!(runs_without_asking, vec![HowCodeRuns::AtFullTrust]);

    let confined: Vec<HowCodeRuns> = HowCodeRuns::ALL
        .into_iter()
        .filter(|level| level.the_kernel_confines_it())
        .collect();
    assert_eq!(confined, vec![HowCodeRuns::InASealedBox]);
}

/// **The three protections stay on at every level**, which is ADR 0064's *at
/// every level, always on* made checkable.
///
/// Including at full trust, which is the level where somebody would be tempted
/// to drop them — and where the snapshot is the thing that makes *undo what the
/// agent did* still true.
#[test]
fn the_same_three_protections_stay_on_at_every_level() {
    for level in HowCodeRuns::ALL {
        assert_eq!(
            level.what_stays_on(),
            AlwaysOn::ALL,
            "{level:?} does not keep all three"
        );
    }
    assert!(
        HowCodeRuns::AtFullTrust
            .what_stays_on()
            .contains(&AlwaysOn::TheSnapshotBeforeEachRun),
        "full trust dropped the snapshot, so undo stops working at the level that needs it most"
    );
}

/// **Only full trust says what it costs**, and the other two say nothing.
///
/// A warning on a level that carries no risk teaches a person to dismiss
/// warnings, which is how a real warning stops being read.
#[test]
fn the_cost_is_said_for_full_trust_and_for_nothing_else() {
    assert!(HowCodeRuns::InASealedBox.what_it_costs().is_none());
    assert!(HowCodeRuns::AskingEachTime.what_it_costs().is_none());

    let cost = HowCodeRuns::AtFullTrust
        .what_it_costs()
        .expect("full trust says what it costs");
    assert!(
        cost.says().contains("steer"),
        "the sentence does not say what the risk actually is: {:?}",
        cost.says()
    );
}

/// **Saying what it costs is all that happens.**
///
/// ADR 0064: *it informs and does not forbid.* So the level that carries a cost
/// is not harder to choose, not refused, and not different in any answer this
/// type gives except the sentence itself.
#[test]
fn the_cost_informs_and_does_not_forbid() {
    // Having a cost does not stop a machine being at that level.
    let chosen = HowCodeRuns::AtFullTrust;
    assert!(chosen.what_it_costs().is_some());
    assert_eq!(chosen, HowCodeRuns::AtFullTrust);

    // And it keeps every protection the others keep, so the warning is not
    // standing in for a protection that was quietly dropped.
    assert_eq!(chosen.what_stays_on(), AlwaysOn::ALL);
}

/// **Every level and every protection has a sentence of its own**, and no two
/// share a key.
#[test]
fn every_level_and_protection_has_its_own_sentence() {
    let mut keys: Vec<&str> = HowCodeRuns::ALL
        .iter()
        .map(|level| level.the_word().named())
        .collect();
    keys.extend(AlwaysOn::ALL.iter().map(|on| on.the_word().named()));
    keys.push(words::WHAT_FULL_TRUST_COSTS.named());

    assert_eq!(keys.len(), 7);
    for (one, key) in keys.iter().enumerate() {
        for (other, against) in keys.iter().enumerate() {
            assert!(
                one == other || key != against,
                "two of these share the key {key}"
            );
        }
    }
}

/// **Nothing a person reads here names the machinery.**
///
/// `CLAUDE.md`: a person never learns the name of anything we rented. These
/// sentences are about what happens to their machine, so they must not say
/// *cgroup*, *namespace*, *seccomp* or *sandbox* — the words an implementation
/// would reach for.
#[test]
fn no_sentence_here_names_the_machinery() {
    let said: Vec<&str> = HowCodeRuns::ALL
        .iter()
        .map(|level| level.the_word().says())
        .chain(AlwaysOn::ALL.iter().map(|on| on.the_word().says()))
        .chain([words::WHAT_FULL_TRUST_COSTS.says()])
        .collect();
    for sentence in said {
        for machinery in [
            "cgroup",
            "namespace",
            "seccomp",
            "sandbox",
            "container",
            "chroot",
            "eBPF",
        ] {
            assert!(
                !sentence.to_lowercase().contains(&machinery.to_lowercase()),
                "{sentence:?} names {machinery}"
            );
        }
    }
}

/// **The choice survives being written down and read back**, which is what a
/// setting has to do.
#[test]
fn a_choice_survives_being_written_down() {
    for level in HowCodeRuns::ALL {
        let written = serde_json::to_string(&level).expect("a level can be written down");
        let read: HowCodeRuns = serde_json::from_str(&written).expect("and read back");
        assert_eq!(read, level, "{level:?} did not survive {written}");
    }
    // The spelling is the one a person could reasonably edit by hand, and is
    // part of the file format rather than an accident of the type's names.
    assert_eq!(
        serde_json::to_string(&HowCodeRuns::AtFullTrust).expect("written"),
        "\"at-full-trust\""
    );
}
