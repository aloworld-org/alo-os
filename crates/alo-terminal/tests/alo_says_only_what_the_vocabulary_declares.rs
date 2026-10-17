//! **An `alo` line is the one place in the terminal a program must never reach,
//! and that is an absence with a guard.**
//!
//! `what_alo_said.rs` promises two things. Every word alo says in its own voice
//! is a key this repository declares, so it is read in the person's own
//! language and never in the language the code was written in. And **there is no
//! road from a program's bytes to a line labelled `alo`** — a program can print
//! whatever it likes and it is still labelled `program`.
//!
//! The second is kept by a function *not existing*. `WhatAloSaid` has no
//! `says_this(&str)`, and the failure mode is somebody adding one in good faith:
//! it looks helpful, it is one line, and it hands every program on the machine
//! the ability to put a sentence under alo's name.
//!
//! **This test was cited by `what_alo_said.rs` before it existed.** The module
//! header named it as holding the guarantee while the file was not in the
//! repository — one mention in the whole tree, and that mention was the claim.
//! It is written here as part of the change that gave the type a second kind of
//! remark, because that change is the one that made the guard load-bearing
//! rather than theoretical: until then nothing but a key could get in.
//!
//! # The explanation is the exception, and it is why this is not a one-line test
//!
//! A model's explanation *is* prose reaching an `alo` line, and it is meant to.
//! What keeps that from being the hole is where the door is: the only way in is
//! a [`alo_terminal::WhatACommandWillDo`], built from an answer to a question
//! somebody asked. A program's output is not one, and cannot be turned into one
//! on the way past.
//!
//! So this holds two different lines:
//!
//! - **`what_alo_said.rs` takes no text at all.** Its doors are a key and an
//!   explanation, and nothing else.
//! - **every key it can carry is declared**, including the ones that arrive
//!   through an explanation's marks, which are ours even though its words are
//!   not.

#![expect(
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic on an unreadable module is the failure being reported"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use alo_terminal::what_a_command_will_do::{WhatACommandWillDo, WorthSeeing};
use alo_terminal::what_alo_said::{AloSLine, WhatAloSaid};
use alo_terminal::why_a_command_is_waiting::WhyItIsWaiting;

/// One of this crate's modules, by file name.
fn the_module(named: &str) -> String {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src", named].iter().collect();
    fs::read_to_string(&path)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", path.display()))
}

/// Every function signature in a module, as one line each.
///
/// Signatures only. A doc comment in `what_alo_said.rs` says `&str` while
/// explaining this very rule, and a test that refused that would make the
/// module undocumentable — which is the trap this repository has now fallen
/// into three times in three files.
fn the_signatures(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
        .filter(|line| {
            line.starts_with("pub fn ")
                || line.starts_with("fn ")
                || line.starts_with("pub const fn ")
                || line.starts_with("const fn ")
        })
        .map(str::to_owned)
        .collect()
}

/// Every shape `WhatAloSaid` can be put into, built through its own doors.
fn every_shape() -> Vec<WhatAloSaid> {
    let mut shapes = vec![WhatAloSaid::nothing()];
    for why in WhyItIsWaiting::ALL {
        let mut said = WhatAloSaid::nothing();
        said.says(why.the_word().key());
        shapes.push(said);
    }
    // Every combination of marks, so no mark's word escapes the check by never
    // being put into an explanation.
    for marks in [
        Vec::new(),
        vec![WorthSeeing::Destructive],
        vec![WorthSeeing::NeedsElevatedRights],
        vec![WorthSeeing::LeavesTheMachine],
        WorthSeeing::ALL.to_vec(),
    ] {
        let mut said = WhatAloSaid::nothing();
        said.explains(WhatACommandWillDo::of("whatever came back", &marks));
        shapes.push(said);
    }
    shapes
}

/// **Every key alo can say is declared in this crate's vocabulary.**
///
/// Walked from the types rather than from a list written here, so a sixth
/// waiting answer or a fourth mark arrives in this test without anybody
/// remembering to extend it.
#[test]
fn every_key_alo_can_say_is_declared() {
    let vocabulary = alo_terminal::words::terminal_words().expect("this crate's own word list");

    let mut checked = 0_usize;
    for said in every_shape() {
        for line in said.the_lines() {
            if let AloSLine::InOurWords(key) = line {
                assert!(
                    vocabulary.phrase(&key).is_some(),
                    "alo can say {key:?}, and this crate's vocabulary does not declare it"
                );
                checked += 1;
            }
        }
    }

    // The control. An empty walk would pass the assertion above while checking
    // nothing, which is this repository's *a null result and a negative result
    // look the same*: five waiting answers, plus three marks and a disclaimer
    // across the explanations built above.
    assert!(
        checked >= 5,
        "only {checked} keys were checked, so this test did not reach the keys it is about"
    );
}

/// **`what_alo_said.rs` has no door that takes text.**
///
/// The absence this file exists for. A `&str`, a `String`, a `&[u8]` or an
/// `impl Into<String>` in a signature here is a road from a program's output to
/// a line labelled `alo`.
#[test]
fn nothing_in_the_module_takes_text() {
    let source = the_module("what_alo_said.rs");
    let signatures = the_signatures(&source);

    // The control first: a scanner that found nothing would pass every
    // assertion below while reading an empty list.
    assert!(
        signatures.len() >= 6,
        "only {} signatures were found in what_alo_said.rs, so this test read the wrong thing: \
         {signatures:?}",
        signatures.len()
    );

    for signature in &signatures {
        for text in ["&str", "String", "&[u8]", "Vec<u8>", "AsRef<str>"] {
            assert!(
                !signature.contains(text),
                "{signature:?} takes {text}, which is a road from a program's bytes to an alo line"
            );
        }
    }
}

/// **And the scanner would catch one**, which is what makes the test above
/// evidence rather than a pattern that happens not to match.
///
/// The planted signature is checked against the same function the real one goes
/// through. Without this, a scanner with a typo in its prefixes would pass for
/// ever and nobody would know.
#[test]
fn a_planted_door_that_takes_text_is_caught() {
    let planted = "    pub fn says_this(&mut self, said: &str) {";
    let found = the_signatures(planted);

    assert_eq!(
        found.len(),
        1,
        "the scanner did not see a planted signature"
    );
    assert!(
        found
            .first()
            .is_some_and(|signature| signature.contains("&str")),
        "the scanner saw the signature and not its argument: {found:?}"
    );
}

/// **A comment naming a forbidden type is not a door**, which is the trap this
/// repository fell into three times before this file was written.
///
/// `what_alo_said.rs` discusses `String` in its own header, explaining why it
/// does not take one. A guard that read comments would fail on the sentence
/// describing the rule it enforces.
#[test]
fn a_comment_naming_text_is_not_a_door() {
    let commented =
        "//! A type that held both as `String` would lose that.\n    //  fn of(s: &str)";
    assert!(
        the_signatures(commented).is_empty(),
        "a comment was read as a signature"
    );

    // And the real module does contain such a comment, so the exclusion above is
    // load-bearing rather than defensive.
    let source = the_module("what_alo_said.rs");
    assert!(
        source.contains("String"),
        "what_alo_said.rs no longer discusses String, so this test is about nothing"
    );
}

/// **The one door onto a model's words is the explanation**, and it is in
/// another module.
///
/// This is the shape of the whole guarantee: the text door exists, once, behind
/// a type whose only source is an answer to a question. Asserting it here keeps
/// the two tests above from being read as *no prose ever reaches an alo line*,
/// which is not true and is not the promise.
#[test]
fn the_explanation_is_the_only_way_prose_reaches_an_alo_line() {
    let mut said = WhatAloSaid::nothing();
    said.explains(WhatACommandWillDo::of("it deletes the folder", &[]));

    let prose: Vec<String> = said
        .the_lines()
        .into_iter()
        .filter_map(|line| match line {
            AloSLine::AsTheModelSaidIt(text) => Some(text),
            AloSLine::InOurWords(_) => None,
        })
        .collect();
    assert_eq!(
        prose,
        vec!["it deletes the folder".to_owned()],
        "an explanation did not carry its words as a model's"
    );

    // And a waiting answer carries no prose at all, which is the control: if
    // every remark produced a prose line, the test above would pass about
    // nothing in particular.
    let mut waiting = WhatAloSaid::nothing();
    waiting.says(WhyItIsWaiting::ForTheKeyboard.the_word().key());
    let none: BTreeSet<bool> = waiting
        .the_lines()
        .into_iter()
        .map(|line| matches!(line, AloSLine::AsTheModelSaidIt(_)))
        .collect();
    assert_eq!(
        none,
        BTreeSet::from([false]),
        "a waiting answer produced a line labelled as a model's words"
    );
}
