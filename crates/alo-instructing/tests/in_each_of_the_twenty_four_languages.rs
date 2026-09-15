//! **What the small model this machine holds answers, asked in each of the 24
//! official languages.**
//!
//! Task 5 of `docs/autonomy/v0-5-access-and-language-plan.md`, and ★
//! `docs/features.md`: *the agent answers in the language it was asked in* —
//! *the thing a cloud assistant does badly for smaller languages*. A promise
//! about 24 languages is worth what somebody has measured of it, so this asks
//! one real question in each, with the answering-language clause in front of
//! it, and records what came back as one of three things: **answers in it**,
//! **answers in another language**, or **does not answer**.
//!
//! **No larger model is run** — the owner's rule of 2026-09-15, *test on the
//! small model; once it works, larger models are not run*. A language this
//! model answers poorly is a fact about this model, recorded as that.
//!
//! # What reads the answer
//!
//! [`alo_instructing::the_language_of`], the same reader that decides which
//! language to ask for. It is small, it is ours, and it is wrong on short
//! sentences that share their words with a neighbour — so a run prints the
//! answer whole, and the report is written from the answers rather than from
//! this test's arithmetic alone.
//!
//! Ignored by default: it needs the runtime and the weights on this machine.
//! ```text
//! ALO_LANGUAGE_MODEL=qwen2.5:7b-instruct-q4_K_M \
//!   cargo test -p alo-instructing --test in_each_of_the_twenty_four_languages -- --ignored --nocapture
//! ```

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_answering::Answering;
use alo_asking::{Asking, Question};
use alo_capability::Grantee;
use alo_instructing::{answering_in, the_language_of};
use alo_models::{Catalogue, InferenceSource, Ollama, SourcePolicy};
use alo_strings::Language;

/// One question a person might really ask, in each of the 24, and the same
/// question in each: *where is the invoice from Northstar?*
const THE_SAME_QUESTION_IN_EACH: [(&str, &str); 24] = [
    ("bg", "Къде е фактурата от Northstar?"),
    ("hr", "Gdje je moja faktura od Northstara?"),
    ("cs", "Kde je soubor s fakturou od Northstar?"),
    ("da", "Hvor er fakturaen fra Northstar?"),
    ("nl", "Waar is het bestand met de factuur van Northstar?"),
    ("en", "Where is the invoice from Northstar?"),
    ("et", "Kus on Northstari arve fail?"),
    ("fi", "Missä on Northstarin lasku?"),
    ("fr", "Où est la facture de Northstar ?"),
    ("de", "Wo ist die Rechnung von Northstar?"),
    ("el", "Πού είναι το τιμολόγιο από τη Northstar;"),
    ("hu", "Hol van a Northstar számla fájl?"),
    ("ga", "Cá bhfuil mo shonrasc ó Northstar?"),
    ("it", "Dove è il file della fattura di Northstar?"),
    ("lv", "Kur ir mans Northstar rēķins?"),
    ("lt", "Kur yra mano Northstar sąskaita?"),
    ("mt", "Fejn hu l-fajl tal-fattura tiegħi?"),
    ("pl", "Gdzie jest faktura od Northstar?"),
    ("pt", "Onde está a fatura da Northstar?"),
    ("ro", "Unde este factura de la Northstar?"),
    ("sk", "Kde je súbor s faktúrou od Northstar?"),
    ("sl", "Kje je moja datoteka z računom?"),
    ("es", "¿Dónde está la factura de Northstar?"),
    ("sv", "Var är fakturan från Northstar?"),
];

/// Which model to ask.
const WHICH_MODEL: &str = "ALO_LANGUAGE_MODEL";

/// What became of one question.
#[derive(Debug, PartialEq, Eq)]
enum Answered {
    /// In the language it was asked in.
    InIt,
    /// In another language this machine carries, named.
    InAnother(String),
    /// Nothing usable came back, or in no language this reader knows.
    NotAtAll,
}

#[test]
#[ignore = "needs the runtime and the weights on this machine — see this file's header"]
fn the_small_model_asked_in_each_of_the_twenty_four_languages() {
    let model = std::env::var(WHICH_MODEL)
        .ok()
        .filter(|said| !said.trim().is_empty())
        .unwrap_or_else(|| panic!("set {WHICH_MODEL} to the small model this machine holds"));
    let runtime = Ollama::at(
        alo_models::ollama::DEFAULT_ENDPOINT,
        Catalogue::built_in().expect("the built-in catalogue loads"),
    );
    let policy = SourcePolicy::ThisMachineOnly;
    let asking_for = Grantee::named("@measuring");

    let mut answered: Vec<(&str, Answered)> = Vec::new();
    for (tag, question) in THE_SAME_QUESTION_IN_EACH {
        let language = Language::written(tag).unwrap();
        // The clause the access plan's task 5 adds, in front of the question a
        // person asked — which is what a turn shows a model.
        let text = format!("{question}{}", answering_in(&language));
        let question = Question::asked(&text, &model).expect("a question the harness wrote");
        let answering = Answering::chosen(InferenceSource::ThisMachine, &policy)
            .expect("no policy forbids this machine answering");
        let said = Asking::by(&asking_for, answering, &[], &policy)
            .to_this_machine(&question, &runtime)
            .unwrap_or_else(|why| panic!("{model} did not answer the {tag} question: {why:?}"));
        let text = said.text().trim().to_owned();
        let read = the_language_of(&text);
        let became = match read.as_ref().map(Language::tag) {
            Some(answered_in) if answered_in == tag => Answered::InIt,
            Some(answered_in) => Answered::InAnother(answered_in.to_owned()),
            None => Answered::NotAtAll,
        };
        println!("\n----- asked in {tag}: {became:?}\n{text}\n----- end");
        answered.push((tag, became));
    }

    println!("\nwhat {model} answered, by language:");
    let mut in_it = 0;
    for (tag, became) in &answered {
        let said = match became {
            Answered::InIt => {
                in_it += 1;
                "answers in it".to_owned()
            }
            Answered::InAnother(other) => format!("answers in another language ({other})"),
            Answered::NotAtAll => "does not answer in a language this machine reads".to_owned(),
        };
        println!("  {tag}: {said}");
    }
    println!(
        "\n{in_it} of {} answered in the language asked in",
        answered.len()
    );
    assert_eq!(answered.len(), 24, "every one of the 24 was asked");
}
