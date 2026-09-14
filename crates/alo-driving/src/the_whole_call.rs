//! **A grammar for the whole call**, in the protocol's own key order.
//!
//! Task 17 of `docs/autonomy/v0-5-the-models-measured-plan.md` and
//! [ADR 0035](../../../docs/decisions/0035-the-wrapper-or-the-engine.md). The
//! pinned runtime can hold a model to a schema, but it orders a schema's keys
//! alphabetically, and the protocol's argument is `named` then `is` — which is
//! why [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)
//! could hold a model to the envelope and no further. `llama.cpp`'s server
//! takes a grammar in GBNF, which orders what it likes, so the whole call can be
//! held: the two keys, the door, the verb, and for that verb exactly the
//! arguments it takes, named in the order it declares them.
//!
//! # What the grammar decides, and what it leaves to the model
//!
//! Held by construction: the envelope, the door a verb's effect requires, the
//! spelling of every verb and argument name, the key order, and the shape of a
//! value. Left to the model: **which** verb the request calls, and what the
//! values are — the path, the name, the number. So a grade earned under this
//! grammar is not comparable with one earned under the envelope alone, and the
//! failures it can still have are the ones worth reading: the wrong verb for the
//! request, or a path the request never mentioned.
//!
//! This is derived from the registry it is given, never written down: a grammar
//! typed by hand would be a second statement of the verbs, and the first thing
//! to drift from `alo-capability`.

use alo_capability::{Effect, Takes, Verb, Verbs};

/// The whole call as GBNF, for the verbs this machine really has.
///
/// One rule per door, one alternative per verb, and one sequence per verb's
/// arguments. A registry with no verbs of a kind writes no door for it, because
/// a door with no verb behind it is a shape the model could produce and nothing
/// could act on.
#[must_use]
pub fn grammar_for(verbs: &Verbs) -> String {
    let mut doors: Vec<String> = Vec::new();
    let mut rules: Vec<String> = Vec::new();
    for (door, effect) in [("read", Effect::Read), ("propose", Effect::Change)] {
        let named: Vec<&Verb> = verbs.all().filter(|verb| verb.effect() == effect).collect();
        if named.is_empty() {
            continue;
        }
        doors.push(format!("{door}-door"));
        let calls: Vec<String> = named
            .iter()
            .map(|verb| format!("call-{}", verb.name().replace('_', "-")))
            .collect();
        rules.push(format!(
            "{door}-door ::= \"\\\"{door}\\\":{{\" ({}) \"}}\"",
            calls.join(" | ")
        ));
        for verb in named {
            rules.push(format!(
                "call-{} ::= \"\\\"verb\\\":\\\"{}\\\",\\\"given\\\":[\" {} \"]\"",
                verb.name().replace('_', "-"),
                verb.name(),
                given(verb)
            ));
        }
    }
    let mut grammar = format!(
        "root ::= \"{{\\\"format\\\":1,\\\"asks\\\":{{\" ({}) \"}}}}\"\n",
        doors.join(" | ")
    );
    for rule in rules {
        grammar.push_str(&rule);
        grammar.push('\n');
    }
    grammar.push_str(VALUES);
    grammar
}

/// The `given` list for one verb: every argument it takes, in the order it
/// declares them, each with the value its type allows.
fn given(verb: &Verb) -> String {
    verb.args()
        .iter()
        .map(|arg| {
            format!(
                "\"{{\\\"named\\\":\\\"{}\\\",\\\"is\\\":\" {} \"}}\"",
                arg.name(),
                value(arg.takes())
            )
        })
        .collect::<Vec<String>>()
        .join(" \",\" ")
}

/// The rule a value of this type is written by.
fn value(takes: &Takes) -> String {
    match takes {
        Takes::Path => "a-path".to_owned(),
        Takes::Application | Takes::Name { .. } => "a-string".to_owned(),
        Takes::Count { .. } => "a-number".to_owned(),
        Takes::Choice(options) => {
            let named: Vec<String> = options
                .iter()
                .map(|offered| format!("\"\\\"{}\\\"\"", offered.name()))
                .collect();
            format!("({})", named.join(" | "))
        }
    }
}

/// What a value may be made of.
///
/// A path starts at the root, which is `alo-capability`'s rule for one and the
/// only one of its checks a grammar can make: the range a count accepts and the
/// length a name may be are its business, and a grammar that repeated them would
/// be the second statement this file exists to avoid.
const VALUES: &str = "a-string ::= \"\\\"\" char* \"\\\"\"\n\
                      a-path ::= \"\\\"/\" char* \"\\\"\"\n\
                      a-number ::= \"-\"? [0-9]+\n\
                      char ::= [^\"\\\\\\x7F\\x00-\\x1F]\n";

/// **The SHA-256 of a grammar's text**, in sixty-four lowercase hexadecimal
/// characters — what a grade earned under it records beside itself, as a grade
/// records its instructions.
#[must_use]
pub fn digest_of(grammar: &str) -> String {
    ring::digest::digest(&ring::digest::SHA256, grammar.as_bytes())
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
