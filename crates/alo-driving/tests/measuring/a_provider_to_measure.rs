//! **What the environment said, and what is missing from it.**
//!
//! Its own file because reading configuration is not asking a model anything,
//! and because of what it buys: the refusal path of a measurement that needs a
//! network, a paid account and a key becomes testable on a machine that has
//! none of them. Without this split the only thing a developer could run is
//! nothing, and *the refusal path tested as carefully as the happy path* would
//! have been a sentence rather than a test.
//!
//! Nothing here reaches the network, and nothing here holds a key: the key is
//! read by the test that uses it, straight into an `alo_models::Secret`, so it
//! never passes through a `String` this file could print.

use alo_models::{Provider, Region};

/// What the five settings have to say before anything can be measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AProviderToMeasure {
    /// The provider, checked the way a person's own settings are checked.
    pub provider: Provider,
    /// The model, as the provider names it.
    pub model: String,
    /// How many times to put the whole set.
    pub rounds: usize,
}

/// Why nothing can be measured, in the words that say what to do about it.
///
/// One variant per thing that can be absent or wrong, because *something was
/// missing* is not a message anybody can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NothingToMeasure {
    /// A setting was not given at all. Carries the variable to set.
    Unset(&'static str),
    /// The provider was refused for the same reason a person's would be.
    TheProviderWasRefused(String),
    /// The address is on this machine, so this is the local measurement.
    ThatIsThisMachine(String),
    /// Rounds was given as something that is not a number above zero.
    RoundsIsNotANumber(String),
}

impl NothingToMeasure {
    /// What to do about it, as a line to print before the test fails.
    #[must_use]
    pub fn what_to_do(&self) -> String {
        match self {
            Self::Unset(variable) => format!("set {variable}"),
            Self::TheProviderWasRefused(why) => {
                format!("the provider was refused, as a person's would be: {why}")
            }
            Self::ThatIsThisMachine(endpoint) => format!(
                "{endpoint} is on this machine, so this would be the local measurement wearing a \
                 provider's name — run against_a_model_on_this_machine.rs instead"
            ),
            Self::RoundsIsNotANumber(said) => {
                format!("rounds is a number above zero, not `{said}`")
            }
        }
    }
}

/// Read the five settings, and say what is missing rather than guessing.
///
/// Every value arrives as an `Option<&str>` rather than being read in here, so
/// the tests below can put every combination to it on a machine with no
/// provider configured at all.
///
/// **There is no default for any of them.** A default endpoint would be a
/// network address nobody chose, and a default region would be a claim about
/// where a model runs that nobody made — [`Region::Unknown`] is what *we were
/// not told* looks like, and it is only reachable by a person saying so.
///
/// # Errors
/// [`NothingToMeasure`] naming the one thing to fix first.
pub fn from_what_was_said(
    name: Option<&str>,
    endpoint: Option<&str>,
    region: Option<&str>,
    model: Option<&str>,
    rounds: Option<&str>,
) -> Result<AProviderToMeasure, NothingToMeasure> {
    let name = name.ok_or(NothingToMeasure::Unset("ALO_PROVIDER_NAME"))?;
    let endpoint = endpoint.ok_or(NothingToMeasure::Unset("ALO_PROVIDER_ENDPOINT"))?;
    let region = region.ok_or(NothingToMeasure::Unset("ALO_PROVIDER_REGION"))?;
    let model = model.ok_or(NothingToMeasure::Unset("ALO_PROVIDER_MODEL"))?;
    let rounds = match rounds {
        None => 1,
        Some(said) => said
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|rounds| *rounds > 0)
            .ok_or_else(|| NothingToMeasure::RoundsIsNotANumber(said.to_owned()))?,
    };
    let provider = Provider::checked(name, endpoint, Region::Declared(region.to_owned()), None)
        .map_err(|why| NothingToMeasure::TheProviderWasRefused(format!("{why:?}")))?;
    // The same rule `alo-asking`'s local door asks, asked here so the answer is
    // the same one: an endpoint on this machine is not a provider, whatever it
    // was named, and measuring it through the provider road would produce a
    // number that looks hosted and is not.
    if provider.source() == alo_models::InferenceSource::ThisMachine {
        return Err(NothingToMeasure::ThatIsThisMachine(
            provider.endpoint.clone(),
        ));
    }
    Ok(AProviderToMeasure {
        provider,
        model: model.to_owned(),
        rounds,
    })
}

#[cfg(test)]
mod tests {
    use super::{NothingToMeasure, from_what_was_said};

    /// Everything a real run gives, minus the key, which this never sees.
    fn all_of_it() -> [Option<&'static str>; 5] {
        [
            Some("a provider"),
            Some("https://api.example.invalid"),
            Some("the EU"),
            Some("a-model"),
            None,
        ]
    }

    fn put(said: [Option<&str>; 5]) -> Result<super::AProviderToMeasure, NothingToMeasure> {
        from_what_was_said(said[0], said[1], said[2], said[3], said[4])
    }

    #[test]
    fn with_all_five_there_is_something_to_measure() {
        let ready = put(all_of_it()).expect("every setting was given");
        assert_eq!(ready.model, "a-model");
        assert_eq!(ready.rounds, 1, "one round unless more were asked for");
        assert!(ready.provider.region.is("the EU"));
    }

    /// **The refusal path, one variable at a time.** Each absence names its own
    /// variable, because a measurement that says only *something is missing*
    /// costs whoever reads it the search this test exists to avoid.
    #[test]
    fn each_missing_setting_names_itself() {
        type Leaving = (&'static str, fn([Option<&str>; 5]) -> [Option<&str>; 5]);
        let leaving: [Leaving; 4] = [
            ("ALO_PROVIDER_NAME", |[_, e, r, m, n]| [None, e, r, m, n]),
            ("ALO_PROVIDER_ENDPOINT", |[p, _, r, m, n]| {
                [p, None, r, m, n]
            }),
            ("ALO_PROVIDER_REGION", |[p, e, _, m, n]| [p, e, None, m, n]),
            ("ALO_PROVIDER_MODEL", |[p, e, r, _, n]| [p, e, r, None, n]),
        ];
        for (variable, without) in leaving {
            assert_eq!(
                put(without(all_of_it())),
                Err(NothingToMeasure::Unset(variable)),
                "leaving {variable} out has to say so"
            );
        }
    }

    #[test]
    fn nothing_is_measured_with_no_settings_at_all() {
        // What a developer gets on a machine with no provider: a refusal, and
        // the first thing to set. Not a run, and not a pass.
        let nothing = put([None; 5]).expect_err("with nothing set there is nothing to measure");
        assert_eq!(nothing, NothingToMeasure::Unset("ALO_PROVIDER_NAME"));
        assert_eq!(nothing.what_to_do(), "set ALO_PROVIDER_NAME");
    }

    #[test]
    fn an_endpoint_on_this_machine_is_not_a_provider() {
        let [name, _, region, model, rounds] = all_of_it();
        let said = [name, Some("http://127.0.0.1:11434"), region, model, rounds];
        let refused = put(said).expect_err("a local address is the other measurement");
        assert!(matches!(refused, NothingToMeasure::ThatIsThisMachine(_)));
        assert!(
            refused
                .what_to_do()
                .contains("against_a_model_on_this_machine.rs"),
            "the refusal says which measurement to run instead: {}",
            refused.what_to_do()
        );
    }

    #[test]
    fn an_address_that_is_not_an_address_is_refused_here() {
        let [name, _, region, model, rounds] = all_of_it();
        let said = [name, Some("api.example.invalid"), region, model, rounds];
        assert!(matches!(
            put(said),
            Err(NothingToMeasure::TheProviderWasRefused(_))
        ));
    }

    /// Plain http to anywhere but this machine would put the key and the
    /// question on the wire in clear. `Provider::checked` refuses it, and this
    /// asserts the measurement road gets that refusal rather than its own rule.
    #[test]
    fn plain_http_to_somewhere_else_is_refused_here_too() {
        let [name, _, region, model, rounds] = all_of_it();
        let said = [
            name,
            Some("http://api.example.invalid"),
            region,
            model,
            rounds,
        ];
        assert!(matches!(
            put(said),
            Err(NothingToMeasure::TheProviderWasRefused(_))
        ));
    }

    #[test]
    fn rounds_that_is_not_a_number_above_zero_is_refused() {
        for bad in ["0", "", "lots", "-2", "1.5"] {
            let [name, endpoint, region, model, _] = all_of_it();
            assert_eq!(
                put([name, endpoint, region, model, Some(bad)]),
                Err(NothingToMeasure::RoundsIsNotANumber(bad.to_owned())),
                "`{bad}` is not a number of rounds"
            );
        }
        let [name, endpoint, region, model, _] = all_of_it();
        assert_eq!(
            put([name, endpoint, region, model, Some("3")])
                .expect("three rounds")
                .rounds,
            3
        );
    }
}
