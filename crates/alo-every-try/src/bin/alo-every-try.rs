//! **Try the fixed set against the real model, several times each**, and say
//! whether every try took the expected road (law 9).
//!
//! ```text
//! ALO_PROVIDER_KEY=…                        # never printed, never written
//! ALO_EVERY_TRY_ENDPOINT=https://api.openai.com   # the default
//! ALO_EVERY_TRY_MODEL=gpt-4o-mini                 # the default
//! ALO_EVERY_TRY_TRIES=5                           # the default
//! ALO_EVERY_TRY_ONLY="write in VSCodium"          # one request, optional
//! ALO_EVERY_TRY_SUMMARY=summary.md                # where the summary goes, optional
//! ALO_EVERY_TRY_SHOWN=shown.txt                   # every question's whole text, optional
//!   alo-every-try
//! ```
//!
//! Exits 0 when every try of every request took the road, 1 when any left it,
//! and 2 when nothing could be measured: no key, a provider that did not
//! answer, or a set that would not build. **Nothing is carried out**: the
//! model's choices are recorded and answered from a script.

use std::io::Write as _;
use std::process::ExitCode;

use alo_every_try::{AtAProvider, Tried, Verdict, Went, summary, tallying, the_set, verdict};
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

fn main() -> ExitCode {
    match running() {
        Ok(verdict) => ExitCode::from(verdict.exit_code()),
        Err(why) => {
            eprintln!("nothing was measured: {why}");
            ExitCode::from(Verdict::NotMeasured.exit_code())
        }
    }
}

/// Try the set and write its summary, answering with the verdict, or why
/// nothing could be measured.
fn running() -> Result<Verdict, String> {
    let endpoint = said("ALO_EVERY_TRY_ENDPOINT", "https://api.openai.com");
    let model = said("ALO_EVERY_TRY_MODEL", "gpt-4o-mini");
    let tries: usize = said("ALO_EVERY_TRY_TRIES", "5")
        .parse()
        .ok()
        .filter(|tries| *tries > 0)
        .ok_or("ALO_EVERY_TRY_TRIES is a whole number above zero")?;
    let key = std::env::var(THE_KEY)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .ok_or(format!(
            "{THE_KEY} is not set, so the model cannot be asked"
        ))?;
    let key = Secret::typed(key.trim())
        .map_err(|_| format!("{THE_KEY} is not a key that can be sent"))?;

    let mut set = the_set().map_err(|why| why.to_string())?;
    let only = said("ALO_EVERY_TRY_ONLY", "");
    if !only.is_empty() {
        set = set
            .only(&only)
            .ok_or(format!("the set has no request called `{only}`"))?;
    }
    let verbs = alo_turn::the_verbs_a_whole_machine_offers().map_err(|why| why.to_string())?;
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

    let asked_of = format!("{model} at {endpoint}");
    println!(
        "Trying set {} against {asked_of}, {tries} times per request. This leaves the machine: \
         every try asks the provider.",
        alo_every_try::THE_VERSION
    );
    let tallied = tallying(
        &set,
        tries,
        &verbs,
        &strings,
        &mut asking,
        &mut |request, try_number, tried: &Tried| {
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
        },
    );
    let mut written = summary(&tallied, &asked_of);
    written.push_str(&format!(
        "\nThe provider was asked {} times, second attempts after no answer included.\n",
        asking.calls()
    ));
    println!("\n{written}");
    let to = said("ALO_EVERY_TRY_SUMMARY", "");
    if !to.is_empty() {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&to)
            .map_err(|why| format!("the summary could not be written to {to}: {why}"))?;
        file.write_all(written.as_bytes())
            .map_err(|why| format!("the summary could not be written to {to}: {why}"))?;
    }
    // After the summary is written, so a fault here never hides what was
    // measured.
    if !asking.is_quiet() {
        return Err("a question was left open, so the indicator would still be lit".to_owned());
    }
    Ok(verdict(&tallied))
}
