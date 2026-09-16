//! From an approved adapter verb to the message its application is sent.
//!
//! An [`alo_capability::Authorised`] is the end of ADR 0001 §5's journey:
//! validated, permitted, approved once and redeemed against the grants at the
//! moment it would run. A [`Driving`] is an `Authorised` for an adapter's verb
//! whose application has been asked about too. It has one constructor, takes
//! the authorisation **by value**, and is not `Clone` — and [`Driving::deliver`]
//! consumes it, so **one approval sends exactly one message**.
//!
//! # The questions, in this order
//!
//! 1. **Is this an adapter's verb?** An authority for any other verb sends
//!    nothing.
//! 2. **Is the adapter's application granted to this agent?** Asked at the
//!    authorisation's own moment. Refused here, a person is told the grants'
//!    own words — and **what is installed is never looked at**, so a refusal
//!    about an ungranted application says nothing about whether it is on the
//!    machine (the order `alo_applications::Reaching` keeps, for the same
//!    reason). A person's own application is refused here whatever the grants
//!    hold (ADR 0043).
//! 3. **Is it installed?** `alo_applications::NotInstalled`'s words if not.
//! 4. **What message does the call become?** Built from validated values only.

use alo_applications::{Installed, NotInstalled};
use alo_capability::{Ask, Authorised, Grants, Refused};
use alo_strings::{Filling, Said, Strings};

use crate::adapters::Adapters;
use crate::delivering::{Delivers, NotDelivered};
use crate::invocation::Invocation;
use crate::message::Message;
use crate::words;

/// An adapter verb that may be carried out, and the message it becomes.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct Driving {
    /// What may run, and under whose authority.
    authorised: Authorised,
    /// The application it goes to.
    application: &'static str,
    /// What it becomes.
    message: Message,
}

impl Driving {
    /// Ask everything an adapter's verb is asked before anything is sent.
    ///
    /// # Errors
    /// [`Refused`], carrying the call: the grants' own words when the
    /// application is not granted, and this crate's or `alo-applications`'
    /// otherwise.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let refused = |authorised: &Authorised, said: Said| {
            Refused::worded_elsewhere(authorised.call().clone(), said)
        };
        // 1. An adapter's verb at all?
        let Some((adapter, verb)) = adapters.carrying_out(authorised.verb()) else {
            let said = strings.say(
                &words::NOT_AN_ADAPTERS_VERB.key(),
                &Filling::of(words::VERB, authorised.verb().to_owned()),
            );
            return Err(refused(&authorised, said));
        };
        // 2. Granted? Refused here, and nothing has been looked for.
        let application = Ask::Application(adapter.application.to_owned());
        if let Err(why) = grants.permitting(authorised.under(), &application, authorised.at()) {
            return Err(Refused::not_granted(authorised.call().clone(), why));
        }
        // 3. Here?
        if !installed.has(adapter.application) {
            let said = NotInstalled::wanting(adapter.application).said(strings);
            return Err(refused(&authorised, said));
        }
        // 4. The message.
        let Invocation::DBus(method) = verb.carried_out;
        let Some(message) = Message::of(adapter.application, &method, authorised.call()) else {
            let said = strings.say(
                &words::NOT_BUILT.key(),
                &Filling::of(words::APPLICATION, adapter.application.to_owned()),
            );
            return Err(refused(&authorised, said));
        };
        Ok(Self {
            authorised,
            application: adapter.application,
            message,
        })
    }

    /// What may run.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// The message it becomes.
    #[must_use]
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Send the message, once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was done in the application — it is not there,
    /// does not offer this, or refused — with the words a person is told, which
    /// are also what the record keeps. An application that did not answer is
    /// **not** an error: something was sent under the approval, and
    /// [`Driven::unanswered`] says so.
    pub fn deliver(self, to: &dyn Delivers, strings: &Strings) -> Result<Driven, Refused> {
        match to.deliver(&self.message) {
            Ok(()) => Ok(Driven {
                authorised: self.authorised,
                application: self.application,
                unanswered: false,
            }),
            Err(why) if why.may_have_happened() => Ok(Driven {
                authorised: self.authorised,
                application: self.application,
                unanswered: true,
            }),
            Err(why) => Err(Refused::worded_elsewhere(
                self.authorised.call().clone(),
                why.said(self.application, strings),
            )),
        }
    }
}

/// An adapter verb that was sent to its application.
#[derive(Debug)]
pub struct Driven {
    /// What ran.
    authorised: Authorised,
    /// Where it went.
    application: &'static str,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl Driven {
    /// What ran.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// Whether the application did not answer, so what happened is not known.
    #[must_use]
    pub fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// What a person is told beyond the sentence they approved — only when the
    /// application did not answer.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        self.unanswered
            .then(|| NotDelivered::DidNotAnswer.said(self.application, strings))
    }

    /// Give back what ran, so it can be recorded.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}
