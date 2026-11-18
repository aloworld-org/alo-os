//! **A renamed argument still answers to the name it had**, deprecated
//! (`Arg::formerly_called`, 2026-11-17): a call giving the old name is read
//! as giving the new one, under the new one; the old name is never what a
//! verb offers; and a declaration whose old name means something else is
//! refused.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_capability::{
    Arg, CallError, Effect, Given, Requires, Takes, Value, Verb, VerbError, Verbs,
};
use alo_strings::Word;

fn a_word(key: &'static str, says: &'static str) -> Word {
    Word::saying(key, says)
}

fn scrolling_to(to: Arg) -> Result<Verb, VerbError> {
    Verb::checked(
        "testing.scroll_to",
        a_word("testing.renamed.purpose", "scroll to a thing"),
        Effect::Change,
        vec![
            Arg::taking(
                "application",
                a_word("testing.renamed.application", "the application"),
                Takes::Application,
            ),
            to,
        ],
        Requires::grants_over(["application"]),
        a_word("testing.renamed.sentence", "scroll {application} to “{to}”"),
    )
}

fn the_argument() -> Arg {
    Arg::taking(
        "to",
        a_word("testing.renamed.to", "the thing"),
        Takes::name(200),
    )
    .formerly_called("name")
}

fn verbs() -> Verbs {
    let mut verbs = Verbs::default();
    verbs
        .declare(scrolling_to(the_argument()).unwrap())
        .unwrap();
    verbs
}

#[test]
fn a_call_under_the_old_name_is_read_as_the_new_one() {
    let verbs = verbs();
    for named in ["to", "name"] {
        let call = verbs
            .call(
                "testing.scroll_to",
                &[
                    ("application", Given::text("org.gnome.Papers")),
                    (named, Given::text("Chapter 3")),
                ],
            )
            .unwrap();
        assert_eq!(
            call.value("to"),
            Some(&Value::Name("Chapter 3".to_owned())),
            "{named}"
        );
        assert_eq!(call.value("name"), None, "{named}");
    }
}

#[test]
fn both_names_with_two_values_are_one_argument_given_twice() {
    let verbs = verbs();
    let twice = verbs.call(
        "testing.scroll_to",
        &[
            ("application", Given::text("org.gnome.Papers")),
            ("to", Given::text("Chapter 3")),
            ("name", Given::text("Chapter 4")),
        ],
    );
    assert!(
        matches!(twice, Err(CallError::SameArgumentTwice { ref argument }) if argument == "to"),
        "{twice:?}"
    );
    let same = verbs.call(
        "testing.scroll_to",
        &[
            ("application", Given::text("org.gnome.Papers")),
            ("to", Given::text("Chapter 3")),
            ("name", Given::text("Chapter 3")),
        ],
    );
    assert!(same.is_ok(), "{same:?}");
}

#[test]
fn the_old_name_is_never_what_the_verb_offers() {
    let verbs = verbs();
    let verb = verbs.of("testing.scroll_to").unwrap();
    let names: Vec<&str> = verb.args().iter().map(Arg::name).collect();
    assert_eq!(names, ["application", "to"]);
    assert_eq!(verb.arg("name"), None);
    assert_eq!(verb.arg("to").and_then(Arg::formerly), Some("name"));
}

#[test]
fn an_old_name_that_is_another_arguments_name_is_refused() {
    let clashing = Arg::taking(
        "to",
        a_word("testing.renamed.to", "the thing"),
        Takes::name(200),
    )
    .formerly_called("application");
    assert!(matches!(
        scrolling_to(clashing),
        Err(VerbError::SameArgumentTwice { ref argument }) if argument == "application"
    ));
    let unspellable = Arg::taking(
        "to",
        a_word("testing.renamed.to", "the thing"),
        Takes::name(200),
    )
    .formerly_called("the name");
    assert!(matches!(
        scrolling_to(unspellable),
        Err(VerbError::ArgumentNotAnIdentifier { .. })
    ));
}
