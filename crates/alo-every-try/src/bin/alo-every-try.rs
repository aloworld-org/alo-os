//! **Try the fixed set against the real model, several times each**, and say
//! whether the tries took the expected road (law 9).
//!
//! ```text
//! ALO_PROVIDER_KEY=…                        # never printed, never written
//! ALO_EVERY_TRY_ENDPOINT=https://api.openai.com   # the default
//! ALO_EVERY_TRY_MODEL=gpt-4o-mini                 # the default
//! ALO_EVERY_TRY_TRIES=5                           # the default
//! ALO_EVERY_TRY_ONLY="write in VSCodium"          # one request, optional
//! ALO_EVERY_TRY_SUMMARY=summary.md                # where the summary goes, optional
//! ALO_EVERY_TRY_SHOWN=shown.txt                   # every question's whole text, optional
//! ALO_EVERY_TRY_LEVEL=ask-each-time               # full-access (the default) or ask-each-time
//! ALO_EVERY_TRY_BEFORE=before.txt                 # the base's declaration: try a change's own
//! ALO_EVERY_TRY_OTHER_TRIES=3                     # with BEFORE: tries of every other request
//!   alo-every-try
//!
//! alo-every-try declare [to]                      # write what-the-model-is-offered.txt again, or to `to`
//! ```
//!
//! **The whole set**, without `ALO_EVERY_TRY_BEFORE`: exits 0 when every try
//! of every request took the road, 1 when any left it, and 2 when nothing
//! could be measured.
//!
//! **A change's own**, with `ALO_EVERY_TRY_BEFORE` naming the base's
//! declaration (`alo_every_try::the_declaration`): the requests the change
//! owns (`alo_every_try::its_own`) are tried `ALO_EVERY_TRY_TRIES` times and
//! held to at least nine in every ten; every other request is tried
//! `ALO_EVERY_TRY_OTHER_TRIES` times and reported, not counted
//! (`alo_every_try::counted`). A change that owns nothing exits 0 **before the
//! key is read and without asking anybody**. Exits 0 when its own held, 1 when
//! one slipped and 2 when nothing could be said.
//!
//! **Nothing is carried out**: the model's choices are recorded and answered
//! from a script.

use std::io::Write as _;
use std::process::ExitCode;

use alo_capability::Verbs;
use alo_every_try::counted::tallying_its_own;
use alo_every_try::counted::{counted, summary_counting};
use alo_every_try::its_own::{ItsOwn, its_own_against};
use alo_every_try::the_declaration::{Declared, THE_FILE};
use alo_every_try::{
    ARequest, AtAProvider, TheLevel, TheSet, Tried, Verdict, Went, summary, tallying_at, the_set,
    verdict,
};
use alo_models::{Provider, Region, Secret};

/// The key, read here and nowhere else, and never printed.
const THE_KEY: &str = "ALO_PROVIDER_KEY";

/// What the variable `name` says, trimmed, or `otherwise` when it is unset or
/// empty. Never used for the key, which may not be printed.
fn said(name: &str, otherwise: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| otherwise.to_owned())
}

/// A count the variable `name` says, or `otherwise`.
fn a_count(name: &str, otherwise: &str) -> Result<usize, String> {
    said(name, otherwise)
        .parse()
        .map_err(|_| format!("{name} is a whole number"))
}

fn main() -> ExitCode {
    let mut arguments = std::env::args().skip(1);
    if arguments.next().as_deref() == Some("declare") {
        return match declaring(arguments.next()) {
            Ok(to) => {
                println!("What the model is offered was written to {to}.");
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("nothing was written: {why}");
                ExitCode::FAILURE
            }
        };
    }
    match running() {
        Ok(code) => ExitCode::from(code),
        Err(why) => {
            eprintln!("nothing was measured: {why}");
            ExitCode::from(Verdict::NotMeasured.exit_code())
        }
    }
}

/// Write the declaration of what the code offers the model to `to`, or to
/// this crate's committed file, answering with where.
fn declaring(to: Option<String>) -> Result<String, String> {
    let set = the_set().map_err(|why| why.to_string())?;
    let verbs = alo_turn::the_verbs_a_whole_machine_offers().map_err(|why| why.to_string())?;
    let declared = Declared::of(&set, &verbs).map_err(|why| why.to_string())?;
    let to = to.unwrap_or_else(|| format!("{}/{THE_FILE}", env!("CARGO_MANIFEST_DIR")));
    std::fs::write(&to, declared.written()).map_err(|why| format!("{to}: {why}"))?;
    Ok(to)
}

/// Append `written` to the summary file a run named, if it named one.
fn writing_the_summary(written: &str) -> Result<(), String> {
    let to = said("ALO_EVERY_TRY_SUMMARY", "");
    if to.is_empty() {
        return Ok(());
    }
    let failed = |why: std::io::Error| format!("the summary could not be written to {to}: {why}");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&to)
        .map_err(failed)?
        .write_all(written.as_bytes())
        .map_err(failed)
}

/// The requests the change owns, when the run was given the base's
/// declaration.
fn what_it_owns(set: &TheSet, verbs: &Verbs) -> Result<Option<Vec<ItsOwn>>, String> {
    let before = said("ALO_EVERY_TRY_BEFORE", "");
    if before.is_empty() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&before)
        .map_err(|why| format!("the base's declaration {before} could not be read: {why}"))?;
    its_own_against(&text, set, verbs).map(Some)
}

/// Say one try as it ends.
fn telling(request: &ARequest, try_number: usize, tried: &Tried) {
    let went = match tried.went() {
        Went::TheRoad => "took the road".to_owned(),
        Went::WentWrong(why) => format!("left the road: {why}"),
        Went::NotAnswered(why) => format!("could not ask the model: {why}"),
    };
    println!(
        "{} — try {try_number}: {} — {went}",
        request.named(),
        tried.chose().join(" → ")
    );
}

/// Try the set, or a change's own, and write its summary, answering with the
/// exit code, or why nothing could be measured.
fn running() -> Result<u8, String> {
    let endpoint = said("ALO_EVERY_TRY_ENDPOINT", "https://api.openai.com");
    let model = said("ALO_EVERY_TRY_MODEL", "gpt-4o-mini");
    let tries = a_count("ALO_EVERY_TRY_TRIES", "5")?;
    if tries == 0 {
        return Err("ALO_EVERY_TRY_TRIES is a whole number above zero".to_owned());
    }
    let asked_of = format!("{model} at {endpoint}");
    let mut set = the_set().map_err(|why| why.to_string())?;
    let verbs = alo_turn::the_verbs_a_whole_machine_offers().map_err(|why| why.to_string())?;

    // **A change that owns nothing is answered here**, before the key is read
    // and before anybody is asked.
    let own = what_it_owns(&set, &verbs)?;
    if let Some(own) = &own
        && own.is_empty()
    {
        let written = summary_counting(&[], own, &asked_of);
        println!("{written}");
        writing_the_summary(&written)?;
        return Ok(counted(&[], own).exit_code());
    }

    let only = said("ALO_EVERY_TRY_ONLY", "");
    if !only.is_empty() {
        if own.is_some() {
            return Err(
                "ALO_EVERY_TRY_ONLY and ALO_EVERY_TRY_BEFORE choose the requests two ways; give one"
                    .to_owned(),
            );
        }
        set = set
            .only(&only)
            .ok_or(format!("the set has no request called `{only}`"))?;
    }
    let others = a_count("ALO_EVERY_TRY_OTHER_TRIES", "3")?;

    let key = std::env::var(THE_KEY)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .ok_or(format!(
            "{THE_KEY} is not set, so the model cannot be asked"
        ))?;
    let key = Secret::typed(key.trim())
        .map_err(|_| format!("{THE_KEY} is not a key that can be sent"))?;
    let strings =
        alo_every_try::words_for_the_answers::the_words().map_err(|why| why.to_string())?;
    let provider = Provider::checked(
        "the provider",
        &endpoint,
        Region::Declared("as the provider states".to_owned()),
        None,
    )
    .map_err(|why| format!("the provider was refused: {why:?}"))?;
    let mut asking = AtAProvider::asking(provider, key, &model, &verbs, set.places());
    let shown = said("ALO_EVERY_TRY_SHOWN", "");
    if !shown.is_empty() {
        asking = asking.writing_what_it_is_shown_to(shown.into());
    }

    // **At which level a change is answered**: `full-access`, as before, or
    // `ask-each-time`, where each change is approved once and the turn goes on
    // from what the daemon told the person it did.
    let level = match said("ALO_EVERY_TRY_LEVEL", "full-access").as_str() {
        "full-access" => TheLevel::FullAccess,
        "ask-each-time" => TheLevel::AskEachTime,
        other => {
            return Err(format!(
                "ALO_EVERY_TRY_LEVEL is full-access or ask-each-time, not `{other}`"
            ));
        }
    };
    println!(
        "Trying set {} against {asked_of}. This leaves the machine: every try asks the provider.",
        alo_every_try::THE_VERSION
    );
    let (mut written, code) = match &own {
        None => {
            let tallied = tallying_at(
                level,
                &set,
                tries,
                &verbs,
                &strings,
                &mut asking,
                &mut telling,
            );
            (summary(&tallied, &asked_of), verdict(&tallied).exit_code())
        }
        Some(own) => {
            let tallied = tallying_its_own(
                level,
                &set,
                own,
                (tries, others),
                &verbs,
                &strings,
                &mut asking,
                &mut telling,
            );
            (
                summary_counting(&tallied, own, &asked_of),
                counted(&tallied, own).exit_code(),
            )
        }
    };
    written.push_str(&format!(
        "\nThe provider was asked {} times, second attempts after no answer included.\n",
        asking.calls()
    ));
    println!("\n{written}");
    writing_the_summary(&written)?;
    // After the summary is written, so a fault here never hides what was
    // measured.
    if !asking.is_quiet() {
        return Err("a question was left open, so the indicator would still be lit".to_owned());
    }
    Ok(code)
}
