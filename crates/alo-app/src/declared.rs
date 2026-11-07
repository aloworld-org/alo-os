//! **`<application> <ability> [values]`**: an ability an adapter declares,
//! checked by the validation alo's own calls go through and carried out by the
//! same code (`docs/contracts/terminal-commands.md`, section 1, *Arguments*).
//!
//! Values come in the order the ability declares its arguments, or by name as
//! `--<argument> <value>`; each is given exactly once, and every argument is
//! required. **A value alo would refuse is refused here too, in the same
//! words**, because what is validated is an `alo_capability::Call` and nothing
//! else becomes one.
//!
//! Two things are done for the person before validation, and both are the
//! contract's: a relative path is taken from the folder their shell is in, and
//! a number is read as a number. Nothing else is rewritten — a path with `..`
//! in it is refused rather than tidied, because tidying is a guess about which
//! folder was meant.

use std::path::Path;

use alo_adapters::{
    Adapter, AdapterVerb, Adapters, Kind, NotDelivered, NotDrivenByHand, drive_by_hand,
};
use alo_capability::{Call, CallError, Effect, Given, Verbs};
use alo_record::{Answered, Road};
use alo_strings::{Filling, Strings};

use crate::ended::Ended;
use crate::machine::{Knowing, Machine};
use crate::outcome::Outcome;
use crate::subject::Subject;
use crate::typed::GivenAs;
use crate::words;

/// One declared ability of one application.
#[derive(Debug, Clone, Copy)]
pub struct Declared {
    /// Its adapter.
    pub adapter: &'static Adapter,
    /// The ability.
    pub verb: &'static AdapterVerb,
}

impl Declared {
    /// Whether it changes something, and so is written down and has a place to
    /// return to kept before it.
    #[must_use]
    pub fn changes(self) -> bool {
        self.verb.effect == Effect::Change
    }
}

/// The ability `named` of `application`, if an adapter declares one.
///
/// # Errors
/// [`Outcome::NotAsked`]: it does not offer that (exit 69).
pub fn the_ability(
    adapters: &Adapters,
    application: &str,
    named: &str,
    strings: &Strings,
) -> Result<Declared, Box<Outcome>> {
    adapters
        .for_application(application)
        .and_then(|adapter| {
            adapter
                .verbs
                .iter()
                .find(|verb| verb.name == named)
                .map(|verb| Declared { adapter, verb })
        })
        .ok_or_else(|| {
            Outcome::refusing(
                Ended::NotOffered,
                strings.say(
                    &words::NO_SUCH_ABILITY.key(),
                    &Filling::of(words::APPLICATION, application.to_owned())
                        .and(words::ABILITY, named.to_owned()),
                ),
            )
        })
}

/// The values given, by argument name, in the order they were given; the
/// value of one of them may be the pipe.
///
/// # Errors
/// [`Outcome::NotAsked`]: more values in order than the ability takes.
pub fn by_name(
    declared: Declared,
    given: &[GivenAs],
    strings: &Strings,
) -> Result<Vec<(String, String)>, Box<Outcome>> {
    let mut in_order = declared.verb.args.iter();
    let mut named = Vec::with_capacity(given.len());
    for value in given {
        match value {
            GivenAs::ByName { argument, value } => named.push((argument.clone(), value.clone())),
            GivenAs::InOrder(value) => {
                let Some(arg) = in_order.next() else {
                    let said = strings.say(
                        &words::TOO_MANY_VALUES.key(),
                        &Filling::of(words::ABILITY, declared.verb.name.to_owned()),
                    );
                    return Err(Outcome::refusing(Ended::NotThisShape, said));
                };
                named.push((arg.name.to_owned(), value.clone()));
            }
        }
    }
    Ok(named)
}

/// One set of values, validated into a call.
///
/// # Errors
/// [`Outcome::NotAsked`], in the validation's own words: 65 for a value
/// refused, 64 for an argument missing, unknown or given twice.
pub fn the_call(
    declared: Declared,
    values: &[(String, String)],
    standing_in: &Path,
    verbs: &Verbs,
    strings: &Strings,
) -> Result<Call, Box<Outcome>> {
    let given: Vec<(&str, Given)> = values
        .iter()
        .map(|(argument, value)| {
            let kind = declared
                .verb
                .args
                .iter()
                .find(|arg| arg.name == argument)
                .map(|arg| arg.kind);
            (argument.as_str(), as_given(kind, value, standing_in))
        })
        .collect();
    verbs
        .call(&declared.adapter.verb_named(declared.verb), &given)
        .map_err(|why| {
            let ended = match why {
                CallError::Argument(_) => Ended::ValueRefused,
                CallError::NoSuchVerb { .. }
                | CallError::NoSuchArgument { .. }
                | CallError::Missing { .. }
                | CallError::SameArgumentTwice { .. } => Ended::NotThisShape,
            };
            Outcome::refusing(ended, why.said(strings))
        })
}

/// A value as the validation is handed it.
fn as_given(kind: Option<Kind>, value: &str, standing_in: &Path) -> Given {
    match kind {
        // **A relative path is taken from the folder the shell is in**, which is
        // what it means to the person who typed it. `..` is left where it is,
        // for the validation to refuse.
        Some(Kind::Path) if !value.is_empty() && !Path::new(value).has_root() => {
            Given::text(standing_in.join(value).to_string_lossy().into_owned())
        }
        // A number that does not read as one goes as text, so the refusal is
        // the validation's own *a number was wanted*.
        Some(Kind::Count { .. }) => value
            .trim()
            .parse::<i64>()
            .map_or_else(|_| Given::text(value), Given::number),
        _ => Given::text(value),
    }
}

/// Carry the call out, once.
pub fn carried(
    call: &Call,
    subject: &Subject,
    knowing: Knowing<'_>,
    machine: &mut dyn Machine,
) -> Outcome {
    let strings = knowing.strings;
    let arguments: Vec<(String, String)> = call
        .values()
        .iter()
        .map(|(name, value)| (name.clone(), value.describe()))
        .collect();
    let answered = match machine.bus() {
        Ok(bus) => drive_by_hand(knowing.adapters, knowing.installed, call, bus, strings),
        Err(why) => Err(NotDrivenByHand::NotDelivered(
            why,
            why.said(subject.identifier(), strings),
        )),
    };
    let sentence = call.sentence(strings);
    let filling =
        Filling::of(words::APPLICATION, subject.called.clone()).and_said(words::SAID, &sentence);
    match answered {
        Ok(driven) if driven.unanswered() => Outcome::Carried {
            road: Road::Declared,
            arguments,
            answered: Answered::NoAnswer,
            ended: Ended::NotKnown,
            said: strings.say(&words::DECLARED_UNANSWERED.key(), &filling),
        },
        Ok(_) => Outcome::Carried {
            road: Road::Declared,
            arguments,
            answered: Answered::Done,
            ended: Ended::Done,
            said: strings.say(&words::DECLARED_DONE.key(), &filling),
        },
        Err(why) => {
            let said = why.said().clone();
            match why {
                NotDrivenByHand::NotDelivered(NotDelivered::Refused, _) => {
                    Outcome::refused_by_it(Road::Declared, arguments, Ended::Refused, said)
                }
                NotDrivenByHand::NotDelivered(NotDelivered::DoesNotOffer, _) => {
                    Outcome::refused_by_it(Road::Declared, arguments, Ended::NotOffered, said)
                }
                NotDrivenByHand::NotDeclared(_)
                | NotDrivenByHand::NotInstalled(_)
                | NotDrivenByHand::NotBuilt(_)
                | NotDrivenByHand::NotDelivered(_, _) => {
                    Outcome::not_asked(Ended::NotOffered, said)
                }
            }
        }
    }
}
