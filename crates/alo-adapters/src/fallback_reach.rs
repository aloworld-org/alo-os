//! Whether an authorised call may reach an application through the
//! accessibility fallback — asked before its tree is asked anything.
//!
//! Every fallback verb asks the same three questions, in this order, and **the
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
//!
//! **An application with an adapter of its own is reached too.** Until
//! 2026-11-21 a fourth question refused it, on the reading that the fallback
//! would make an adapter's reviewed list decorative.
//! `docs/contracts/app-adapters.md`, *Amended 2026-11-06*, point 1, replaced
//! that reading: *alo is offered an adapter's verbs **and** the fallback's
//! verbs over the same application, under the same grant*. The list is kept
//! honest by the sentence instead: a fallback press says it is a press of a
//! control by its name, never in a reviewed verb's words, and the record names
//! the verb, which is the road. Kept, the fourth question would have taken from Text Editor every
//! control its two verbs do not cover the day alo was offered them.

use alo_applications::{Installed, Reaching};
use alo_capability::{Authorised, Grants, Refused};
use alo_strings::{Filling, Strings};

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

/// Ask the three questions of an authorised call of `verb`.
pub(crate) fn reached(
    authorised: Authorised,
    verb: &str,
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
    Ok(Reached {
        authorised,
        application,
    })
}
