//! `alo-ask`: the command a person types to put a question to the model on this
//! machine.
//!
//! ```text
//! $ alo-ask may the tenant sublet?
//! …the answer…
//! on this machine
//! ```
//!
//! Every decision here belongs to somebody else — `alo-asking` puts the
//! question, `alo-models` holds the runtime, `alo-answering` says what happened
//! when nothing came back, `alo-choosing` says whose settings these are. This
//! file is the order they happen in on a real machine, which is the only part a
//! test cannot reach.
//!
//! # It takes a question and no options
//!
//! Every argument is the question, joined with spaces, so a person may type one
//! without quoting it. There is no flag for which model answers, and that is
//! deliberate: which model answers is a **setting**, kept in the person's own
//! `settings.toml` by `alo-choosing`, and a flag would be a second place to
//! decide it — with the usual consequence that the two disagree and nobody can
//! say which one the machine obeyed.
//!
//! # The answer goes to stdout and everything else to stderr
//!
//! What the model said is the output, and it is the only thing on stdout, so
//! `alo-ask … > answer.txt` holds the answer and nothing around it. **The
//! sentence saying where it came from goes to stderr**, which is why it is never
//! the thing that gets piped away: on a machine sold on sovereignty, *on this
//! machine* is not a decoration on the answer, and a person who redirected the
//! output still sees it.
//!
//! # Whose machine, whose language, whose model
//!
//! The person's folder is worked out from what the session says, their settings
//! are read out of it, and the sentences are rendered in the languages those
//! settings name. **A session with no folder and a settings file that will not
//! parse are both said and then carried past**: neither is a reason to refuse to
//! ask a question, and honouring nothing in an unreadable file is that file's
//! own rule. What is lost in both cases is the choice of model, which the next
//! rule can still answer when exactly one set of weights is installed.
//!
//! # Why the runtime is not asked whether it is there first
//!
//! `alo_models::ollama::found_on_this_machine` knocks at the loopback address
//! and answers `None` when **either** nothing is listening **or** nothing is
//! installed. Those are two different things to tell a person — mend the service,
//! or fetch a model — and one `None` cannot say which. So the adapter is built
//! pointed at the same address and each failure arrives as the runtime's own
//! typed refusal, with the sentence that belongs to it.

use std::process::ExitCode;

use alo_answering::Answering;
use alo_ask::{
    TheDoor, Unanswered, beside_an_answer, the_only_one, the_persons_model, what_it_can_say,
};
use alo_asking::{Asking, NotAQuestion, NotAnswered, Question};
use alo_capability::Grantee;
use alo_choosing::{Picked, Settings, the_persons_folder};
use alo_models::{
    Catalogue, InferenceSource, ModelRuntime, SourcePolicy, ollama::Ollama,
    the_name_a_runtime_knows,
};
use alo_protocol::ToAPerson;
use alo_strings::Strings;

/// What this machine is set to permit, as this command holds it.
///
/// The strictest of the four rules, and it permits what this command does:
/// nothing here leaves the machine, so the rule that allows least still allows
/// it. Holding the strictest means **this command cannot be the place a rule
/// gets widened** — a question that would have to leave is refused by
/// `alo_answering::Answering::chosen` before anything is asked, and there is no
/// second door here for it to be sent through instead.
///
/// It is not a reading of the organisation's policy and does not pretend to be
/// one. Where that policy is kept for a whole machine is not settled in this
/// repository, and a command that guessed at it would be answering a question
/// nobody has asked it.
const THE_STRICTEST_RULE: SourcePolicy = SourcePolicy::ThisMachineOnly;

/// Who the question is asked by, as `alo_asking::Asking::by` wants it.
///
/// **Carried and never read on this door.** `Asking` holds a grantee because the
/// other door shows the egress on the indicator under somebody's authority, and
/// this one has no egress to show — `alo_asking::locally` names the field and
/// never touches it.
///
/// `alo_capability::Grantee` is an agent or an application, and a command a
/// person typed is neither. It is named for itself rather than for the person,
/// because the alternative is writing a login name into a type whose two kinds
/// are both software. **That is a finding about the shape of `Asking::by` rather
/// than something to fix by inventing a third kind here**, and it costs nothing
/// today precisely because this door never reads it.
const WHO_IS_ASKING: &str = "alo-ask";

/// Which road a sentence takes, asked of the model on this machine.
///
/// **The verdict is read in `alo_ask::answering_or_acting` and asked here**,
/// which is this file's own division: the part with a rule in it is testable
/// without a runtime, and the part that reaches one is the order a real machine
/// does things in.
///
/// A model that will not answer is [`WhatItWants::CannotTell`] rather than a
/// failure of its own, because the person asked for something and the thing to
/// tell them is that alo could not tell which — not that a classification step
/// they never asked for did not work.
fn the_road_this_takes(
    put: &Question,
    runtime: &Ollama,
    strings: &Strings,
) -> alo_ask::answering_or_acting::WhatItWants {
    let Ok(answering) = Answering::chosen(InferenceSource::ThisMachine, &THE_STRICTEST_RULE) else {
        return alo_ask::answering_or_acting::WhatItWants::CannotTell;
    };
    let who = Grantee::named(WHO_IS_ASKING);
    let asking = Asking::by(&who, answering, &[], &THE_STRICTEST_RULE);
    let _ = strings;
    match asking.to_this_machine(put, runtime) {
        Ok(answer) => alo_ask::answering_or_acting::what_the_reply_says(answer.text()),
        Err(_) => alo_ask::answering_or_acting::WhatItWants::CannotTell,
    }
}

/// **What alo was asked to do, handed to the agent.**
///
/// [ADR 0124](../../../docs/decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md)
/// term 2: the one command *proposes, says what grant the work needs, and waits
/// for an approval before anything happens*. None of those three is this
/// function's. It writes the sentence to ADR 0109's door and prints what came
/// back; the agent decides which verb it means, the daemon puts the proposal,
/// and the person approves it. **Nothing here can act**, which is why a
/// classifier being wrong costs a sentence rather than a change.
fn the_agent_was_asked(typed: &str, strings: &Strings) -> ExitCode {
    let at = std::path::Path::new(alo_protocol::where_the_door_is::WHERE_A_PERSONS_WORDS_GO);
    match alo_ask::at_the_agents_door::what_the_agent_says(at, typed) {
        Ok(said) => {
            println!("{said}");
            ExitCode::SUCCESS
        }
        // **Its own English, and no fallback to answering.** A command that met
        // a missing agent and quietly answered the request instead would be the
        // silent substitution ADR 0008 forbids, arriving as robustness — and the
        // person would be told about their request rather than have it done,
        // with nothing saying which happened.
        Err(why) => {
            eprintln!("alo-ask: {why}");
            let _ = strings;
            ExitCode::FAILURE
        }
    }
}

/// Ask the daemon to put the question to the provider the person chose.
///
/// One connection, one line, one answer. Everything a person can read here is
/// somebody else's sentence: the daemon's refusal in the words of whoever
/// refused it, or `alo-protocol`'s two about the service itself. **This file
/// words nothing**, which is the same rule `beside_an_answer`'s header states
/// for the other road.
fn asked_of_their_provider(question: &str, strings: &Strings) -> ExitCode {
    match TheDoor::for_this_person().asked(question) {
        Ok(ToAPerson::Answered {
            text, came_from, ..
        }) => {
            println!("{text}");
            // **Where it came from, beside the answer**, which is what
            // `docs/features.md` promises and is the last place it can be lost.
            // Rendered by the daemon, so the screen and the record cannot
            // become two accounts of one moment.
            eprintln!("{}", came_from.text());
            ExitCode::SUCCESS
        }
        Ok(ToAPerson::Refused(wording)) => {
            eprintln!("{}", wording.text());
            ExitCode::FAILURE
        }
        // Any other answer is the service answering something this request
        // cannot earn, which is this repository disagreeing with itself. The
        // person is told the one thing that is true and useful rather than
        // something inventive, and it is not their fault or theirs to fix.
        Ok(_) => {
            eprintln!("{}", Unanswered::NotAnswered.said(strings));
            ExitCode::FAILURE
        }
        Err(why) => {
            eprintln!("{}", why.said(strings));
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let vocabulary = match what_it_can_say() {
        Ok(vocabulary) => vocabulary,
        // English, and the only English a person could see here. Whoever reads
        // this is fixing a word list; there is no honest way to say it in the
        // person's language, because the sentence would have had to come out of
        // the vocabulary that could not be built.
        Err(why) => {
            eprintln!("alo-ask: {why}");
            return ExitCode::FAILURE;
        }
    };
    let mut strings = Strings::of(vocabulary);

    // Whose settings these are, and then what they say. Both refusals are said
    // and passed: a question is still worth asking on a machine whose settings
    // could not be read.
    let settings = match the_persons_folder(
        std::env::var_os(alo_choosing::CONFIG_HOME).as_deref(),
        std::env::var_os(alo_choosing::HOME).as_deref(),
    ) {
        Err(no_folder) => {
            eprintln!("{}", no_folder.said(&strings));
            Settings::untouched()
        }
        Ok(folder) => match Settings::at(&folder.path_of(alo_choosing::THE_SETTINGS)) {
            Ok(settings) => settings,
            Err(not_set) => {
                eprintln!("{}", not_set.said(&strings));
                Settings::untouched()
            }
        },
    };
    strings.prefers(settings.languages());

    // **What the person typed, before the machine is touched.** Every argument is
    // the question, joined with spaces. It is checked here rather than after the
    // model is worked out because the order is visible to whoever typed it: a
    // person who pressed return by mistake is owed *write the question first*,
    // and on a machine whose runtime is not running they would otherwise be told
    // it is not reachable — true, unasked for, and about the wrong thing.
    //
    // `NotAQuestion::Nothing` is `alo-asking`'s own refusal, made here rather
    // than obtained from `Question::asked`, because the model is not known yet
    // and a question is not built until it is.
    let asked = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    if asked.trim().is_empty() {
        eprintln!("{}", NotAQuestion::Nothing.said(&strings));
        return ExitCode::FAILURE;
    }

    // **What the person forbade, before the machine is touched.**
    //
    // `docs/features.md`: *Unless the person says otherwise, in Settings.* A
    // person who chose a provider and turned off letting alo use anything else
    // is not answered from this machine — and is told which of their own
    // settings decided it, so they can change it if they want to.
    //
    // Asked here rather than after the model is worked out, which is this
    // file's own order: the answer does not depend on what is installed, and a
    // person owed *you told alo not to use anything else* must not first be
    // told the runtime is unreachable.
    if let Some(chosen) = settings.chosen()
        && let Some(forbidden) =
            chosen.nothing_else_may_answer(settings.may_use_another(), &strings)
    {
        eprintln!("{forbidden}");
        return ExitCode::FAILURE;
    }

    // **A question the person's settings send to a provider goes to the
    // daemon, and this is where the two roads part.**
    //
    // Not because of law 1 — a person's own question goes on no indicator,
    // because they chose their provider in Settings (ADR 0126). Because of two
    // things only the service can reach: the organisation's rule, read with the
    // checks that guard the file naming which login is the agent, and signing
    // in to a proxy that asks who this machine is, whose password belongs to
    // the daemon's unit (ADR 0059). A command that went straight out instead
    // would repeat a fault already found and fixed here — `alo-agentd`'s
    // `the_road_out`: *a machine read its organisation's proxy off the disk and
    // then asked a provider directly.*
    //
    // Asked **before** the catalogue and the runtime, which is this file's own
    // order: nothing below is about a provider, and a person whose question is
    // going to OpenAI must not first be told a local runtime is unreachable.
    //
    // **Nothing falls back.** If the daemon is not there the person is told so
    // and told to start it; this does not then try a local model, because that
    // is the silent substitution ADR 0008 forbids arriving as robustness.
    if matches!(settings.chosen(), Some(Picked::FromAProvider { .. })) {
        return asked_of_their_provider(&asked, &strings);
    }

    let catalogue = match Catalogue::built_in() {
        Ok(catalogue) => catalogue,
        // Also English, and for the same audience: a catalogue that will not
        // parse is a fault in what the release shipped, and nothing a person
        // typed caused it or can do anything about.
        Err(why) => {
            eprintln!("alo-ask: the catalogue this release ships could not be read: {why}");
            return ExitCode::FAILURE;
        }
    };
    // **The name the person chose is resolved before the catalogue is moved
    // into the runtime**, which is the one place in this process that holds
    // both. A person chooses a catalogue *entry*; the runtime answers to an
    // *artefact*; the two are different strings and nothing converted between
    // them until 2026-10-11.
    //
    // Measured that day: a question put with the catalogue id came back
    // `NoModelThere` from a runtime that was listening, with the weights on
    // the disk — because ollama had never heard of `smollm2-1.7b-instruct`,
    // only of `smollm2:1.7b-instruct-q4_K_M`. The refusal was true of the
    // name sent and false of the machine, so a person would have gone looking
    // for a download they already had.
    //
    // **Only the chosen path needs it.** `the_only_one` reads the runtime's
    // own `installed()` list, whose ids are artefacts already
    // (`Ollama::catalogue_id` strips `:latest` and nothing else), so
    // resolving that would refuse a perfectly good name as an unknown id. Two
    // kinds of name in one variable is what made this possible; they are
    // converted at their own source now.
    let theirs = match the_persons_model(settings.chosen()) {
        Some(chosen) => match the_name_a_runtime_knows(&catalogue, chosen) {
            Ok(artefact) => Some(artefact.to_owned()),
            // **In the person's own language**, which this printed in English
            // until 2026-10-12. The note that stood here said a person could
            // only have chosen an entry the catalogue offered, so an id it
            // does not hold was a fault in whatever wrote the setting — and
            // that if it ever became reachable by an ordinary choice it would
            // need words in the vocabulary.
            //
            // It is reachable by an ordinary choice. Measured by writing
            // `catalogue = "no-such-model-anywhere"` into a settings file, the
            // way a person with a typo would, and reading what came back: a
            // `thiserror` string that appeared nowhere in the published
            // vocabulary. Settings are a person's to edit, so a name they
            // mistyped is a condition they are in and not a fault in a tool.
            Err(not_named) => {
                eprintln!("{}", not_named.said(&strings));
                return ExitCode::FAILURE;
            }
        },
        None => None,
    };
    let runtime = Ollama::new(catalogue);

    // Which model answers: the one they chose, or the only one installed. The
    // runtime is asked only in the second case, so a person who has chosen has
    // one round trip rather than two.
    let of_model = match theirs {
        Some(artefact) => artefact,
        None => match runtime.installed() {
            Ok(installed) => the_only_one(&installed).unwrap_or_default().to_owned(),
            Err(why) => {
                eprintln!("{}", why.said(&strings));
                return ExitCode::FAILURE;
            }
        },
    };

    // The model is known, so what was typed is a question now. An empty model
    // name is not a failure to report here either: it is a question without a
    // model in it, and `Question::asked` refuses it with the sentence that tells
    // the person to choose one.
    // **Which of the two this sentence is, decided before it is treated as
    // either.** ADR 0124 term 4, the owner's words: *it should be smart enough
    // to know if the question needs answering or acting.* So the same model that
    // would answer is asked which it is, and the person says nothing.
    //
    // **A question costs two asks for this**, which is the price of there being
    // one command. The alternative is a flag, and a flag is the person being
    // made to say what term 4 says they never say.
    let which = match Question::asked(
        &alo_ask::answering_or_acting::the_question_about(&asked),
        &of_model,
    ) {
        Ok(put) => the_road_this_takes(&put, &runtime, &strings),
        Err(not_a_question) => {
            eprintln!("{}", not_a_question.said(&strings));
            return ExitCode::FAILURE;
        }
    };
    match which {
        alo_ask::answering_or_acting::WhatItWants::Acting => {
            return the_agent_was_asked(&asked, &strings);
        }
        // **Nothing is guessed.** ADR 0124 leaves open what happens when the
        // reading is genuinely ambiguous, and this lane's reading — recorded in
        // the ADR and marked as the owner's to overturn — is that it asks,
        // because the standing rule is *never a silent fallback* and choosing
        // between two readings in silence is a silent choice between them.
        alo_ask::answering_or_acting::WhatItWants::CannotTell => {
            eprintln!("{}", Unanswered::NotAnswered.said(&strings));
            return ExitCode::FAILURE;
        }
        alo_ask::answering_or_acting::WhatItWants::Answering => {}
    }

    let question = match Question::asked(&asked, &of_model) {
        Ok(question) => question,
        Err(not_a_question) => {
            eprintln!("{}", not_a_question.said(&strings));
            return ExitCode::FAILURE;
        }
    };

    let answering = match Answering::chosen(InferenceSource::ThisMachine, &THE_STRICTEST_RULE) {
        Ok(answering) => answering,
        // Unreachable while the rule above is the strictest one and the place is
        // this machine, and handled rather than asserted: the refusal has a
        // sentence of its own, and a command that panicked here would be telling
        // a person their machine is broken when what happened is that a rule
        // refused them.
        Err(not_allowed) => {
            eprintln!("{}", not_allowed.said(&strings));
            return ExitCode::FAILURE;
        }
    };

    // No other places are offered. `alo-answering` makes offers out of that
    // list, and an offer to ask somewhere else — from a command named for the
    // place it asks — is the substitution ADR 0008 forbids, arriving as a
    // helpful suggestion.
    let who = Grantee::named(WHO_IS_ASKING);
    let asking = Asking::by(&who, answering, &[], &THE_STRICTEST_RULE);
    match asking.to_this_machine(&question, &runtime) {
        Ok(answer) => {
            println!("{}", answer.text());
            // **Where it came from, and when the person chose somewhere else,
            // that this is not it.** Composed in `alo_ask::beside_an_answer`
            // rather than printed as two lines here, because the answered path
            // cannot be reached by a test on a machine with no model runtime —
            // so a second line written here would be a caller nothing checks.
            for line in beside_an_answer(answer.source(), settings.chosen(), &strings) {
                eprintln!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(NotAnswered::DidNotAnswer(failed)) => {
            eprintln!("{}", failed.said(&strings));
            eprintln!("{}", failed.nothing_was_sent(&strings));
            ExitCode::FAILURE
        }
        // Keeps its English, which is `alo_asking::Miswired`'s own rule: its
        // reader is whoever wired a question to somewhere, and it cannot happen
        // while the only place this file builds is `InferenceSource::ThisMachine`.
        Err(NotAnswered::Miswired(why)) => {
            eprintln!("alo-ask: {why}");
            ExitCode::FAILURE
        }
    }
}
