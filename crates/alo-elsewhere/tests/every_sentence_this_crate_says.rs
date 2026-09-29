//! Every sentence this crate can say is declared, and every sentence declared
//! can be said.
//!
//! # Why this test and not another count
//!
//! Each module already checks that its own states have distinct keys and that
//! the count matches. **None of that catches the fault this file exists for.**
//!
//! A `word()` method returns a `&'static str` from [`alo_elsewhere::words`], and
//! those constants are ordinary strings. Change a key in one place and not the
//! other — a rename, a typo, a moved area prefix — and everything still
//! compiles, every existing test still passes, and the only symptom is a lookup
//! that misses at run time in front of a person, in a language nobody on this
//! team reads.
//!
//! So this walks the two directions separately, because they fail separately:
//!
//! - **every key a state can name resolves in the vocabulary.** A state whose
//!   sentence is not declared is a state that cannot be said.
//! - **every declared sentence is named by some state.** A sentence nothing can
//!   reach is a sentence that will be translated, reviewed and paid for, and
//!   never shown — and it is the same shape as a check nothing can fail: it
//!   looks like part of the product and is not.
//!
//! The second direction is the one a count cannot do at all. Twenty-two
//! declared and twenty-two published agree perfectly while one of them is
//! unreachable.

// An integration test is not `#[cfg(test)]`, so the workspace's lints apply to
// it as they do to shipped code. Both are declared here rather than silenced at
// each line: in a test, a panic on an unexpected `Err` **is** the failure being
// reported, and a named panic carrying the offending key is more use to whoever
// reads the run than an assertion that only says `false`.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::collections::BTreeSet;

use alo_elsewhere::changing::WhatTheAgentHas;
use alo_elsewhere::machine::TheName;
use alo_elsewhere::saying_where::Marked;
use alo_elsewhere::sending::HowItIsGoing;
use alo_elsewhere::words::{EVERY_WORD, elsewhere_words};
use alo_elsewhere::{NotElsewhere, Reaching};
use alo_strings::{Key, Word};

/// Every key a state of this crate can name.
///
/// Built by asking the states rather than by listing keys, so a state added
/// without a sentence reaches this test through its own `word()` rather than
/// through somebody remembering to add a line here.
fn every_refusal_key() -> BTreeSet<&'static str> {
    // Listed rather than iterated because an error enum has no `ALL`. The
    // number is asserted below, so a variant added to `NotElsewhere` and not
    // added here is noticed rather than being checked in neither direction.
    [
        NotElsewhere::Unnamed,
        NotElsewhere::NameTooLong {
            how_long: 65,
            at_most: 64,
        },
        NotElsewhere::AlreadyAdded,
        NotElsewhere::EndsBeforeItBegins,
        NotElsewhere::AlreadyDriving,
        NotElsewhere::NoGoal,
        NotElsewhere::GoalTooLong {
            how_long: 501,
            at_most: 500,
        },
        NotElsewhere::ItWasStopped,
    ]
    .iter()
    .map(NotElsewhere::word)
    .collect()
}

fn every_key_a_state_can_name() -> BTreeSet<&'static str> {
    let mut named = BTreeSet::new();

    for state in Reaching::ALL {
        named.insert(state.word());
    }
    for state in HowItIsGoing::ALL {
        named.insert(state.word());
    }

    named.extend(every_refusal_key());

    // **The one sentence no state names**, and it is deliberate. ADR 0079 makes
    // showing it a term of the decision rather than a presentation choice, so it
    // is named on the type that needs it — `alo_elsewhere::WHAT_IT_CANNOT_ENUMERATE`
    // — and read by whoever is granting, at the moment of granting. There is no
    // state of anything that means *the person is deciding right now*.
    //
    // It is listed here rather than exempted, so it is still checked in both
    // directions: declared, and reachable by something.
    named.insert(alo_elsewhere::WHAT_IT_CANNOT_ENUMERATE);

    // What a window says about where it is. `Here` says nothing, on purpose.
    let called = TheName::given("in the studio").expect("a name");
    for marked in [
        Marked::Here,
        Marked::Elsewhere { called },
        Marked::ElsewhereUnnamed,
    ] {
        if let Some(word) = marked.word() {
            named.insert(word);
        }
    }

    // What the agent has on a machine, as the person's settings show it.
    for has in [
        WhatTheAgentHas::Nothing,
        WhatTheAgentHas::TheWholeMachine {
            until: std::time::SystemTime::UNIX_EPOCH,
        },
        WhatTheAgentHas::Ended,
    ] {
        named.insert(has.word());
    }

    named
}

/// **Every key a state can name resolves in the vocabulary.**
///
/// The fault: a key changed in one place and not the other compiles, passes
/// every existing test, and misses at run time in front of a person.
#[test]
fn every_sentence_a_state_can_name_is_declared() {
    let vocabulary = elsewhere_words().expect("no key is declared twice");

    for key in every_key_a_state_can_name() {
        let named = Key::named(key).unwrap_or_else(|why| panic!("{key} is not a key: {why}"));
        assert!(
            vocabulary.phrase(&named).is_some(),
            "a state of this crate names {key}, and nothing declares it. A person would meet an \
             empty string where a sentence belongs."
        );
    }
}

/// **Every declared sentence is named by some state.**
///
/// A sentence nothing can reach is translated, reviewed and paid for, and never
/// shown. It is the same shape as a check nothing can fail: it looks like part of
/// the product and is not — and no count can tell, because the counts agree.
#[test]
fn every_declared_sentence_can_actually_be_said() {
    let reachable = every_key_a_state_can_name();

    let orphans: Vec<&str> = EVERY_WORD
        .iter()
        .map(Word::named)
        .filter(|key| !reachable.contains(key))
        .collect();

    assert!(
        orphans.is_empty(),
        "these sentences are declared and nothing in this crate can name them: {orphans:?}. \
         Either a state is missing its word(), or a sentence outlived the thing that said it."
    );
}

/// The refusals are listed by hand above, so their number is held here: a new
/// variant changes this count and is noticed, rather than being silently absent
/// from both directions of the check.
#[test]
fn the_refusals_listed_by_hand_are_all_of_them() {
    assert_eq!(
        every_refusal_key().len(),
        8,
        "the number of refusal sentences changed. If a variant was added to NotElsewhere, add it \
         to the list in this file as well — a refusal missing from that list is checked in \
         neither direction."
    );
}

/// **The first version of this file counted the refusals by their key prefix**
/// — anything under `elsewhere.not-` — and answered seven for eight variants,
/// because `ItWasStopped` is a refusal whose sentence lives under
/// `elsewhere.work.`, where a person reads it.
///
/// That is the subject of this whole file arriving inside it: the count was a
/// property of how keys happen to be spelled, standing in for the number of
/// things that can be refused. It answered confidently and it answered wrong,
/// and it would have kept answering wrong as variants were added under the
/// prefix.
///
/// Kept as a test rather than a comment so the distinction is exercised: a
/// refusal's key is not required to begin with `not-`, and nothing should start
/// assuming it is.
#[test]
fn a_refusals_sentence_need_not_live_under_the_refusal_prefix() {
    let under_not = every_refusal_key()
        .iter()
        .filter(|key| key.starts_with("elsewhere.not-"))
        .count();
    assert_eq!(
        under_not, 7,
        "if this is 8, a refusal moved and the note above needs rewriting rather than deleting"
    );
    assert!(
        every_refusal_key().contains(&"elsewhere.work.stopped-before-it-came-back"),
        "the refusal that reads as part of the work's own story is the reason a prefix count lies"
    );
}

/// Both directions together, stated as one number, so the ordinary case is one
/// line to read: everything declared is sayable and everything sayable is
/// declared.
#[test]
fn the_two_directions_agree() {
    let reachable = every_key_a_state_can_name();
    let declared: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
    assert_eq!(
        reachable, declared,
        "what this crate can say and what it declares are not the same set"
    );
}
