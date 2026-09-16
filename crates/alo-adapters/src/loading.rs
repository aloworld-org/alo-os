//! Loading an adapter: every rule the contract makes, checked against the
//! declaration, before a single verb reaches the machine's list.
//!
//! [`load`] is the only way to make a [`Loaded`], and a `Loaded` is the only
//! thing [`crate::Adapters`] holds — so a verb an agent can ask for through an
//! adapter is one whose declaration passed everything here **and**
//! `alo_capability::Verb::checked`, which every other verb on the machine
//! passes too. Nothing here decides reach or approval; it decides whether a
//! declaration is one this machine will carry out.
//!
//! In order, with the first refusal the answer:
//!
//! 1. **the adapter**: a name an agent can be called by and not the accessibility
//!    fallback's (`accessible`), an application identifier, not a person's own
//!    application (ADR 0043), at least one release, a mechanism this machine
//!    carries out and never screenshots and synthetic input, and at least one
//!    verb;
//! 2. **each verb**: how a person does it by hand (ADR 0009); every word it is
//!    declared with among the adapter's words; no argument that is a script, a
//!    command or free text; a well-formed method that is not named for running
//!    something, whose action — where it has one — is not an argument's to
//!    choose; no parameter the application interprets; every parameter filled
//!    from an argument of the right kind; every argument sent; every path under
//!    a grant; and then the verb contract itself, through `Verb::checked`.

use alo_applications::Application;
use alo_capability::{Arg, Offered, Requires, Takes, Verb, Verbs, is_a_persons_own};
use alo_strings::Word;

use crate::adapter::Adapter;
use crate::adapter_arg::{AdapterArg, Kind};
use crate::adapter_verb::{AdapterVerb, ONLY_ITS_APPLICATION, Reaches};
use crate::becoming_code::{chooses_an_action, runs_something};
use crate::bus_names::{is_a_method, is_an_interface, is_an_object};
use crate::invocation::{DBusMethod, Invocation, Part};
use crate::mechanism::Mechanism;
use crate::not_loaded::NotLoaded;

/// An adapter that passed every rule, with its verbs declared.
#[derive(Debug, Clone)]
pub struct Loaded {
    /// The declaration.
    adapter: &'static Adapter,
    /// Its verbs, as the machine's list will hold them, in declared order.
    verbs: Vec<Verb>,
}

impl Loaded {
    /// The declaration it was loaded from.
    #[must_use]
    pub fn adapter(&self) -> &'static Adapter {
        self.adapter
    }

    /// Its verbs, checked, under the names the machine knows them by.
    #[must_use]
    pub fn verbs(&self) -> &[Verb] {
        &self.verbs
    }

    /// The declared verb the machine knows by this name, if it is this
    /// adapter's.
    #[must_use]
    pub fn declared(&self, named: &str) -> Option<&'static AdapterVerb> {
        let within = named.strip_prefix(self.adapter.name)?.strip_prefix('.')?;
        self.adapter.verbs.iter().find(|verb| verb.name == within)
    }
}

/// Load an adapter, checking everything the contract asks of one.
///
/// # Errors
/// [`NotLoaded`], saying what to change about the declaration.
pub fn load(adapter: &'static Adapter) -> Result<Loaded, NotLoaded> {
    the_adapter(adapter)?;
    let verbs = adapter
        .verbs
        .iter()
        .map(|verb| a_verb(adapter, verb))
        .collect::<Result<Vec<_>, _>>()?;
    // A name declared twice within one adapter is refused here, where its
    // author is, rather than when a machine puts it on its list.
    let mut once = Verbs::default();
    for verb in &verbs {
        once.declare(verb.clone())?;
    }
    Ok(Loaded { adapter, verbs })
}

/// Everything about the adapter as a whole.
fn the_adapter(adapter: &Adapter) -> Result<(), NotLoaded> {
    let named = || adapter.name.to_owned();
    if !is_a_name(adapter.name) {
        return Err(NotLoaded::NotAName { name: named() });
    }
    if adapter.name == crate::fallback_verbs::PREFIX {
        return Err(NotLoaded::TheFallbacksName);
    }
    if Application::identified(adapter.application).is_err()
        || adapter.application.trim() != adapter.application
    {
        return Err(NotLoaded::NotAnApplication {
            application: adapter.application.to_owned(),
        });
    }
    if is_a_persons_own(adapter.application) {
        return Err(NotLoaded::APersonsOwn {
            application: adapter.application.to_owned(),
        });
    }
    if adapter
        .releases
        .iter()
        .all(|series| series.trim().is_empty())
    {
        return Err(NotLoaded::NoReleases { adapter: named() });
    }
    match adapter.mechanism {
        Mechanism::DBus => {}
        Mechanism::Synthetic => return Err(NotLoaded::Synthetic { adapter: named() }),
        Mechanism::Api | Mechanism::Accessibility => {
            return Err(NotLoaded::NotCarriedOutHere {
                adapter: named(),
                mechanism: adapter.mechanism.named(),
            });
        }
    }
    if adapter.verbs.is_empty() {
        return Err(NotLoaded::NoVerbs { adapter: named() });
    }
    Ok(())
}

/// Everything about one verb, ending in the verb contract's own check.
fn a_verb(adapter: &Adapter, verb: &AdapterVerb) -> Result<Verb, NotLoaded> {
    let named = adapter.verb_named(verb);
    let Some(by_hand) = verb.by_hand else {
        return Err(NotLoaded::NoByHand { verb: named });
    };
    for word in words_of(verb, by_hand) {
        if !adapter
            .words
            .iter()
            .any(|declared| declared.named() == word.named())
        {
            return Err(NotLoaded::WordNotDeclared {
                adapter: adapter.name.to_owned(),
                verb: named,
                key: word.named().to_owned(),
            });
        }
    }
    for arg in verb.args {
        if arg.kind.is_code() {
            return Err(NotLoaded::TakesCode {
                verb: named,
                argument: arg.name.to_owned(),
                kind: arg.kind.named(),
            });
        }
    }
    let Invocation::DBus(method) = verb.carried_out;
    the_method(&named, verb, &method)?;
    within_its_grant(&named, verb)?;

    let requires = match verb.reaches {
        Reaches::Over(arguments) => Requires::grants_over(arguments.iter().copied()),
        Reaches::OnlyItsApplication => Requires::nothing_because(ONLY_ITS_APPLICATION),
    };
    Verb::checked(
        &named,
        verb.purpose,
        verb.effect,
        verb.args.iter().map(as_an_arg).collect(),
        requires,
        verb.sentence,
    )
    .map_err(|why| NotLoaded::Verb { verb: named, why })
}

/// Every word a verb is declared with.
fn words_of(verb: &AdapterVerb, by_hand: Word) -> Vec<Word> {
    let mut words = vec![verb.purpose, verb.sentence, by_hand];
    for arg in verb.args {
        words.push(arg.purpose);
        if let Kind::Choice(offers) = arg.kind {
            words.extend(offers.iter().map(|offer| offer.words));
        }
    }
    words
}

/// The method a verb calls, and what each of its parameters is filled from.
fn the_method(named: &str, verb: &AdapterVerb, method: &DBusMethod) -> Result<(), NotLoaded> {
    for (well_formed, what) in [
        (is_an_object(method.object), method.object),
        (is_an_interface(method.interface), method.interface),
        (is_a_method(method.method), method.method),
    ] {
        if !well_formed {
            return Err(NotLoaded::NotWellFormed {
                verb: named.to_owned(),
                what: what.to_owned(),
            });
        }
    }
    let called = || format!("{}.{}", method.interface, method.method);
    if runs_something(method.interface, method.method) {
        return Err(NotLoaded::RunsSomething {
            verb: named.to_owned(),
            method: called(),
        });
    }
    if chooses_an_action(method.interface, method.method)
        && !matches!(method.parameters.first(), Some(Part::Literal(_)))
    {
        return Err(NotLoaded::ChoosesWhatRuns {
            verb: named.to_owned(),
            method: called(),
        });
    }
    for part in method.parameters {
        if let Part::Evaluated { argument, by } = part {
            return Err(NotLoaded::BecomesCode {
                verb: named.to_owned(),
                argument: (*argument).to_owned(),
                by: (*by).to_owned(),
            });
        }
        let Some(argument) = part.argument() else {
            continue;
        };
        let Some(arg) = verb.args.iter().find(|arg| arg.name == argument) else {
            return Err(NotLoaded::NoSuchArgument {
                verb: named.to_owned(),
                argument: argument.to_owned(),
            });
        };
        let fits = matches!(
            (part, arg.kind),
            (Part::FileAddressOf(_), Kind::Path)
                | (Part::TextOf(_), Kind::Name { .. } | Kind::Choice(_))
                | (Part::CountOf(_), Kind::Count { .. })
        );
        if !fits {
            return Err(NotLoaded::WrongKind {
                verb: named.to_owned(),
                argument: argument.to_owned(),
            });
        }
    }
    for arg in verb.args {
        if !method
            .parameters
            .iter()
            .any(|part| part.argument() == Some(arg.name))
        {
            return Err(NotLoaded::GoesNowhere {
                verb: named.to_owned(),
                argument: arg.name.to_owned(),
            });
        }
    }
    Ok(())
}

/// Every path the verb takes is one a grant is required over.
fn within_its_grant(named: &str, verb: &AdapterVerb) -> Result<(), NotLoaded> {
    let over: &[&str] = match verb.reaches {
        Reaches::Over(arguments) => arguments,
        Reaches::OnlyItsApplication => &[],
    };
    for arg in verb.args {
        if matches!(arg.kind, Kind::Path) && !over.contains(&arg.name) {
            return Err(NotLoaded::OutsideItsGrant {
                verb: named.to_owned(),
                argument: arg.name.to_owned(),
            });
        }
    }
    Ok(())
}

/// A declared argument, as the verb contract's.
fn as_an_arg(arg: &AdapterArg) -> Arg {
    let takes = match arg.kind {
        Kind::Path => Takes::Path,
        Kind::Name { longest } => Takes::name(longest),
        Kind::Count { least, most } => Takes::count(least, most),
        Kind::Choice(offers) => Takes::choice(
            offers
                .iter()
                .map(|offer| Offered::called(offer.name, offer.words)),
        ),
        // Refused before this is reached; declared as the narrowest kind there
        // is, so a mistake in the order above could never widen anything.
        Kind::Text | Kind::Script { .. } | Kind::Command => Takes::count(0, 0),
    };
    Arg::taking(arg.name, arg.purpose, takes)
}

/// Whether an adapter's name is one an agent can be called by.
fn is_a_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
