//! **The same fixed set, put to a model at a provider.**
//!
//! A separate file from the loop beside it because it is a separate road: that
//! one is `alo-asking`'s local door under `SourcePolicy::ThisMachineOnly`, and
//! this one is `Asking::to_a_provider` under `Anywhere`. The exercises, the
//! scoring and the bar are shared — the *asking* is what differs, and it is
//! the only thing that differs.
//!
//! # This one leaves the machine, by design and visibly
//!
//! Law 1 is not bent to run a measurement. `alo_asking::Hosted::ask` is private
//! and its only caller has already obtained an `alo_egress::Departing`, so
//! every exercise here appears on the indicator and in the record, and the loop
//! asserts the indicator is quiet again at the end.

use alo_answering::Answering;
use alo_asking::{Asking, Question};
use alo_capability::Grantee;
use alo_driving::{Attempt, Exercises, Instructions, Measured};
use alo_models::{InferenceSource, SourcePolicy};

use super::{TO_WARM_IT_UP, as_far_as_it_helps, as_it_is_written, the_verbs};

/// Where a provider's endpoint resolves to, for the boundary to be told.
///
/// **What this machine connects to is what is registered**, which on a network
/// with a proxy is the proxy rather than the provider — `alo_asking::Hosted`
/// says so and answers with whichever it is.
fn resolved(where_to: Option<(String, u16)>) -> Vec<std::net::SocketAddr> {
    use std::net::ToSocketAddrs as _;
    where_to
        .and_then(|(host, port)| (host.as_str(), port).to_socket_addrs().ok())
        .map(Iterator::collect)
        .unwrap_or_default()
}

/// Put one exercise to a provider, through the road a turn really takes.
fn put_to_a_provider(
    text: &str,
    model: &str,
    hosted: &alo_asking::Hosted<'_>,
    agent: &Grantee,
    policy: &SourcePolicy,
    indicator: &mut alo_egress::Indicator,
    in_the_envelope: bool,
) -> Result<String, String> {
    let question = Question::asked(text, model).map_err(|why| format!("{why:?}"))?;
    let answering = Answering::chosen(hosted.named_source(), policy)
        .map_err(|why| format!("the policy forbids this provider: {why:?}"))?;
    let asking = Asking::by(agent, answering, &[], policy);
    let to = resolved(hosted.where_it_would_connect());
    let now = std::time::SystemTime::now();
    // **Both doors, so the same ten exercises measure the difference between
    // them and nothing else.** Every door takes `self`, so this branches on
    // which to call rather than on what to pass.
    let put = if in_the_envelope {
        asking.to_a_provider_in_the_envelope(&question, hosted, indicator, now, &to)
    } else {
        asking.to_a_provider(&question, hosted, indicator, now, &to)
    };
    let asked = put.map_err(|why| format!("{why:?}"))?;
    let answer = asked.ended(indicator);
    // **The answer says where it came from, and it is not this machine.** The
    // local measurement asserts the mirror of this; a provider measurement whose
    // answers came from the machine would be measuring the wrong thing just as
    // surely.
    assert_ne!(
        answer.source(),
        &InferenceSource::ThisMachine,
        "a provider measurement answered from this machine"
    );
    Ok(answer.text().to_owned())
}

/// The same fixed set, put to a provider somewhere else.
///
/// **The same exercises, the same scoring, the same grade scale** as
/// [`the_fixed_set_put_to`] — so a provider's number and a local model's are
/// comparable, which is the whole reason this exists rather than a second
/// measurement of its own design.
///
/// # This one leaves the machine, and cannot not
///
/// The local measurement runs under `SourcePolicy::ThisMachineOnly`, which is
/// what makes *the measurement caused no egress* a fact about the run. This one
/// runs under `Anywhere` and every exercise departs. That is not a weakening of
/// the rule: `alo_asking::Hosted::ask` is private and reachable only through a
/// door that has already obtained an `alo_egress::Departing`, so law 1's
/// indicator shows each one and the record keeps it.
/// **One prompt put to a provider**, scored by whoever asked.
///
/// `the_fixed_set_put_to_a_provider` below carries the whole ten and grades
/// them; this carries one and hands back what the model said. It exists for
/// `does_a_model_look_before_it_proposes`, which asks a question the fixed ten
/// deliberately cannot ask — every one of them names the file it is about, so
/// none of them can measure whether a model finds out first.
///
/// **It leaves the machine**, exactly as the set does and for the same reason:
/// the only road is a door that already holds an `alo_egress::Departing`, so
/// the indicator shows it and the record keeps it.
///
/// # Errors
/// What the provider said, when it would not answer.
pub fn one_prompt_put_to_a_provider(
    text: &str,
    model: &str,
    provider: &alo_models::Provider,
    key: &alo_models::Secret,
    in_the_envelope: bool,
) -> Result<String, String> {
    let hosted = alo_asking::Hosted::provider(provider, Some(key));
    let mut indicator = alo_egress::Indicator::default();
    put_to_a_provider(
        text,
        model,
        &hosted,
        &Grantee::named("@measuring"),
        &SourcePolicy::Anywhere,
        &mut indicator,
        in_the_envelope,
    )
}

pub fn the_fixed_set_put_to_a_provider(
    model: &str,
    provider: &alo_models::Provider,
    key: &alo_models::Secret,
    rounds: usize,
    instructions: Instructions,
    in_the_envelope: bool,
) -> Measured {
    let verbs = the_verbs();
    let exercises = Exercises::over(&verbs).expect("the fixed set is built over alo OS's verbs");
    let hosted = alo_asking::Hosted::provider(provider, Some(key));
    let policy = SourcePolicy::Anywhere;
    let measuring = Grantee::named("@measuring");
    let mut indicator = alo_egress::Indicator::default();

    println!("asking {} at {}", model, provider.endpoint);
    println!(
        "under the instructions {}, sha256 {}",
        instructions.named(),
        instructions.digest()
    );
    println!("**this leaves the machine**: every exercise departs for the provider above");

    let warmed = put_to_a_provider(
        TO_WARM_IT_UP,
        model,
        &hosted,
        &measuring,
        &policy,
        &mut indicator,
        in_the_envelope,
    );
    assert!(
        warmed.is_ok(),
        "{model} did not answer at all, so nothing was measured: {warmed:?}"
    );

    let mut attempts: Vec<Attempt> = Vec::new();
    for round in 1..=rounds {
        for exercise in exercises.all() {
            let answered = put_to_a_provider(
                &exercises.prompt_under(instructions, exercise),
                model,
                &hosted,
                &measuring,
                &policy,
                &mut indicator,
                in_the_envelope,
            );
            let answer = match answered {
                Ok(answer) => answer,
                Err(why) => panic!(
                    "round {round}, {}: the provider did not answer — {why}. A provider that \
                     cannot answer is not a model that failed",
                    exercise.named()
                ),
            };
            let attempt = exercises.attempt(exercise, &answer);
            println!(
                "round {round}  {:<8}  {:?}",
                exercise.named(),
                attempt.outcome()
            );
            if !attempt.drove() {
                println!("                    {}", as_far_as_it_helps(&answer));
            }
            attempts.push(attempt);
        }
    }
    // **The indicator is quiet at the end.** Every departure this run made was
    // ended, which is the same guarantee a person watching the corner relies on.
    assert!(
        indicator.is_quiet(),
        "a departure was left open, so the indicator would still be lit"
    );
    let measured = Measured::of(&exercises, attempts)
        .expect("every exercise was asked, which is what makes a run a measurement");
    println!(
        "\n{model} at {} under {}: {} of {} drove the verbs — \"{}\"",
        provider.name,
        instructions.named(),
        measured.drove(),
        measured.how_many(),
        as_it_is_written(measured.grade())
    );
    assert!(
        measured.grade().has_been_measured(),
        "a run that happened never grades as one that did not"
    );
    measured
}
