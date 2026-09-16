//! Whether an authorised call may reach an application through the
//! accessibility fallback — asked before its tree is asked anything.
//!
//! Both fallback verbs ask the same four questions, in this order, and **the
//! tree is not touched by any of them**:
//!
//! 1. **Is this one of the fallback's verbs, and the one expected?** An
//!    authority for any other verb reads and presses nothing.
//! 2. **Is the application granted to this agent, at the authorisation's own
//!    moment?** Asked by `alo_applications::Reaching`, which asks the grants
//!    before it looks at what is installed — so an ungranted application's
//!    refusal is the same whether it is on the machine or not, and a person's
//!    own application is refused whatever the grants hold (ADR 0043).
//! 3. **Is it installed?** The same type's second question.
//! 4. **Does it have an adapter of its own?** If so, the fallback is refused:
//!    an adapter is a narrower, reviewed list of what an agent may do in that
//!    application, and pressing any control in its windows instead would make
//!    that list decorative. *For applications without an adapter* is the
//!    fallback's whole scope.

use alo_applications::{Installed, Reaching};
use alo_capability::{Authorised, Grants, Refused};
use alo_strings::{Filling, Strings};

use crate::adapters::Adapters;
use crate::fallback_verbs::APPLICATION;
use crate::fallback_words as words;

/// An authorised fallback call, and the application it may reach.
#[derive(Debug)]
pub(crate) struct Reached {
    /// What may run.
    pub(crate) authorised: Authorised,
    /// The application's identifier, as this machine has it installed.
    pub(crate) application: String,
}

/// Ask the four questions of an authorised call of `verb`.
pub(crate) fn reached(
    authorised: Authorised,
    verb: &str,
    adapters: &Adapters,
    grants: &Grants,
    installed: &Installed,
    strings: &Strings,
) -> Result<Reached, Refused> {
    if authorised.verb() != verb {
        let said = strings.say(
            &words::NOT_THIS_VERB.key(),
            &Filling::of(words::VERB, authorised.verb().to_owned()),
        );
        return Err(Refused::worded_elsewhere(authorised.call().clone(), said));
    }
    let reaching = Reaching::of(authorised, grants, installed, strings)?;
    let Some(application) = reaching
        .application(APPLICATION)
        .map(|application| application.identifier().to_owned())
    else {
        let authorised = reaching.into_authorised();
        let said = strings.say(
            &words::NOT_THIS_VERB.key(),
            &Filling::of(words::VERB, authorised.verb().to_owned()),
        );
        return Err(Refused::worded_elsewhere(authorised.call().clone(), said));
    };
    let authorised = reaching.into_authorised();
    if adapters
        .all()
        .any(|adapter| adapter.application == application)
    {
        let said = strings.say(
            &words::HAS_ITS_OWN.key(),
            &Filling::of(words::APPLICATION, application),
        );
        return Err(Refused::worded_elsewhere(authorised.call().clone(), said));
    }
    Ok(Reached {
        authorised,
        application,
    })
}
