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
//! ALO_EVERY_TRY_REACHED_BEFORE=reached.txt         # the base's record of what each has reached
//! ALO_EVERY_TRY_MEASURED=measured.txt              # with BEFORE: where what was measured is added
//!
//! alo-every-try owns                            # with BEFORE: none, some or the-whole-set
//! alo-every-try compare <change> <base>          # judge the whole set against its base
//! alo-every-try the-bar                          # the bar's tries, and how many more a flag gets
//! alo-every-try reached <measured>...             # raise what-each-request-has-reached.txt
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
use alo_every_try::compared_with_its_base::{compared, summary_of, the_flagged, verdict_of};
use alo_every_try::counted::tallying_its_own;
use alo_every_try::counted::{
    counted, counted_with, summary_counting, the_flagged_of_its_own, with_more,
};
use alo_every_try::its_own::{ItsOwn, its_own_against, owns_the_whole_set};
use alo_every_try::the_bar::THE_BAR;
use alo_every_try::the_bar::THE_CONFIRMING_TRIES;
use alo_every_try::the_declaration::{Declared, THE_FILE};
use alo_every_try::what_each_has_reached::{self, Measured, Reached};
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
    let first = arguments.next();
    if first.as_deref() == Some("the-bar") {
        println!("{} {THE_CONFIRMING_TRIES}", THE_BAR.tries());
        return ExitCode::SUCCESS;
    }
    if first.as_deref() == Some("owns") {
        return match owning() {
            Ok(which) => {
                println!("{which}");
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("which requests this change owns cannot be said: {why}");
                ExitCode::from(Verdict::NotMeasured.exit_code())
            }
        };
    }
    if first.as_deref() == Some("compare") {
        let (Some(change), Some(base)) = (arguments.next(), arguments.next()) else {
            eprintln!("compare takes what the change measured and what its base measured");
            return ExitCode::from(Verdict::NotMeasured.exit_code());
        };
        return match comparing(&change, &base) {
            Ok(code) => ExitCode::from(code),
            Err(why) => {
                eprintln!("nothing was compared: {why}");
                ExitCode::from(Verdict::NotMeasured.exit_code())
            }
        };
    }
    if first.as_deref() == Some("reached") {
        return match raising(arguments.collect()) {
            Ok(to) => {
                println!("What each request has reached was written to {to}.");
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("nothing was written: {why}");
                ExitCode::FAILURE
            }
        };
    }
    if first.as_deref() == Some("declare") {
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

/// **Raise the record of what each request has reached** from the measured
/// files `from` (each run's `every-try-measured` artifact), writing it to
/// `ALO_EVERY_TRY_REACHED_TO` or this crate's committed file, answering with
/// where. Nothing is ever taken out of it.
fn raising(from: Vec<String>) -> Result<String, String> {
    if from.is_empty() {
        return Err("name at least one file of what a run measured".to_owned());
    }
    let to = said(
        "ALO_EVERY_TRY_REACHED_TO",
        &format!(
            "{}/{}",
            env!("CARGO_MANIFEST_DIR"),
            what_each_has_reached::THE_FILE
        ),
    );
    let now = std::fs::read_to_string(&to).map_err(|why| format!("{to}: {why}"))?;
    let mut record = Reached::read(&now).map_err(|why| why.to_string())?;
    for file in &from {
        let text = std::fs::read_to_string(file).map_err(|why| format!("{file}: {why}"))?;
        record = record.raised_by(&Measured::read(&text).map_err(|why| why.to_string())?);
    }
    std::fs::write(&to, record.written()).map_err(|why| format!("{to}: {why}"))?;
    Ok(to)
}

/// **The record may not be lowered**: the base's record, when the run was
/// given it, against this change's, for the requests still in `set`.
fn the_record_against_its_base(set: &TheSet) -> Result<Reached, String> {
    let record =
        Reached::read(what_each_has_reached::AS_COMMITTED).map_err(|why| why.to_string())?;
    let before = said("ALO_EVERY_TRY_REACHED_BEFORE", "");
    if before.is_empty() {
        return Ok(record);
    }
    let text = std::fs::read_to_string(&before)
        .map_err(|why| format!("the base's record {before} could not be read: {why}"))?;
    let base = Reached::read(&text).map_err(|why| why.to_string())?;
    let in_the_set = |named: &str| set.requests().iter().any(|one| one.named() == named);
    let lowered = record.lowered_from(&base, &in_the_set);
    if lowered.is_empty() {
        Ok(record)
    } else {
        let each: Vec<String> = lowered
            .iter()
            .map(|(level, named)| format!("{named} at {level}"))
            .collect();
        Err(format!(
            "what each request has reached was lowered, which no change may do: {}",
            each.join("; ")
        ))
    }
}

/// Append `text` to the file the run named for what it measured, if any.
fn writing_what_was_measured(text: &str) -> Result<(), String> {
    let to = said("ALO_EVERY_TRY_MEASURED", "");
    if to.is_empty() {
        return Ok(());
    }
    let failed =
        |why: std::io::Error| format!("what was measured could not be written to {to}: {why}");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&to)
        .map_err(failed)?
        .write_all(text.as_bytes())
        .map_err(failed)
}

/// **Which way the workflow goes**: `none`, `some` or `the-whole-set` — and
/// first, the ratchet, so a record lowered against its base fails whichever.
fn owning() -> Result<&'static str, String> {
    let set = the_set().map_err(|why| why.to_string())?;
    let verbs = alo_turn::the_verbs_a_whole_machine_offers().map_err(|why| why.to_string())?;
    the_record_against_its_base(&set)?;
    let own = what_it_owns(&set, &verbs)?
        .ok_or("ALO_EVERY_TRY_BEFORE names the base's declaration to compare with")?;
    Ok(if own.is_empty() {
        "none"
    } else if owns_the_whole_set(&own) {
        "the-whole-set"
    } else {
        "some"
    })
}

/// **Compare what the change measured with what its base did**
/// (`alo_every_try::compared_with_its_base`): write the summary, write what
/// was flagged to `ALO_EVERY_TRY_FLAGGED`, and answer with the exit code.
fn comparing(change: &str, base: &str) -> Result<u8, String> {
    let read = |file: &str| {
        std::fs::read_to_string(file)
            .map_err(|why| format!("{file}: {why}"))
            .and_then(|text| Measured::read(&text).map_err(|why| why.to_string()))
    };
    let record =
        Reached::read(what_each_has_reached::AS_COMMITTED).map_err(|why| why.to_string())?;
    let base = read(base)?;
    // **A base that measured nothing is not compared with**: every request
    // would read as new and be held to the bar alone.
    if base.each().next().is_none() {
        return Err(
            "the base measured nothing, so the change cannot be compared with it".to_owned(),
        );
    }
    let rows = compared(&read(change)?, &base, &record);
    let asked_of = format!(
        "{} at {}",
        said("ALO_EVERY_TRY_MODEL", "gpt-4o-mini"),
        said("ALO_EVERY_TRY_ENDPOINT", "https://api.openai.com")
    );
    let written = summary_of(&rows, &asked_of);
    println!("{written}");
    writing_the_summary(&written)?;
    let flagged = said("ALO_EVERY_TRY_FLAGGED", "");
    if !flagged.is_empty() {
        let lines: String = the_flagged(&rows)
            .iter()
            .map(|(level, named)| format!("{level}\t{named}\n"))
            .collect();
        std::fs::write(&flagged, lines).map_err(|why| format!("{flagged}: {why}"))?;
    }
    Ok(verdict_of(&rows).exit_code())
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
    // **Before anything else, the ratchet**: a record lowered against its base
    // fails whatever the change owns.
    let record = the_record_against_its_base(&set)?;

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
    let named = said("ALO_EVERY_TRY_LEVEL", "full-access");
    let level: TheLevel = what_each_has_reached::the_level_named(&named).ok_or(format!(
        "ALO_EVERY_TRY_LEVEL is full-access or ask-each-time, not `{named}`"
    ))?;
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
            writing_what_was_measured(&Measured::of(level, &tallied).written())?;
            (summary(&tallied, &asked_of), verdict(&tallied).exit_code())
        }
        Some(own) => {
            let mut tallied = tallying_its_own(
                level,
                &set,
                own,
                // **The bar's tries**, not the variable's, so the bar is one
                // declared setting (`alo_every_try::the_bar`).
                (THE_BAR.tries(), others),
                &verbs,
                &strings,
                &mut asking,
                &mut telling,
            );
            let mut written = summary_counting(&tallied, own, &asked_of);
            // **Flag, then confirm** (approved 2026-11-20): what slipped on
            // the first tries is tried again, and the verdict is on both.
            let flagged = the_flagged_of_its_own(&tallied, own, &|named| record.has(level, named));
            if !flagged.is_empty() && THE_CONFIRMING_TRIES > 0 {
                let again = tallying_at(
                    level,
                    &set.clone()
                        .retaining(|request| flagged.iter().any(|named| named == request.named())),
                    THE_CONFIRMING_TRIES,
                    &verbs,
                    &strings,
                    &mut asking,
                    &mut telling,
                );
                tallied = with_more(tallied, again);
                written.push_str(&format!(
                    "\n### After {THE_CONFIRMING_TRIES} more tries of what was flagged\n\n"
                ));
                written.push_str(&summary_counting(&tallied, own, &asked_of));
            }
            writing_what_was_measured(&Measured::of(level, &tallied).written())?;
            let held_to_every_try: Vec<&str> = own
                .iter()
                .map(|one| one.named())
                .filter(|named| record.has(level, named))
                .collect();
            if !held_to_every_try.is_empty() {
                written.push_str(&format!(
                    "\nHeld to every try at this level, because each once took every try \
                     here (`what-each-request-has-reached.txt`): {}.\n",
                    held_to_every_try.join(", ")
                ));
            }
            (
                written,
                counted_with(&tallied, own, &|named| record.has(level, named)).exit_code(),
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
