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
use alo_ask::{the_only_one, the_persons_model, what_it_can_say};
use alo_asking::{Asking, NotAQuestion, NotAnswered, Question};
use alo_capability::Grantee;
use alo_choosing::{Settings, the_persons_folder};
use alo_models::{Catalogue, InferenceSource, ModelRuntime, SourcePolicy, ollama::Ollama};
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
    let runtime = Ollama::new(catalogue);

    // Which model answers: the one they chose, or the only one installed. The
    // runtime is asked only in the second case, so a person who has chosen has
    // one round trip rather than two.
    let of_model = match the_persons_model(settings.chosen()) {
        Some(theirs) => theirs.to_owned(),
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
            eprintln!("{}", answer.came_from(&strings));
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
