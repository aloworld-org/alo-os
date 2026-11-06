//! Granted application controls use the same approval and record as other verbs.

use crate::{Machine, NotDone};
use alo_capability::{Authorised, Grants};
use alo_record::Entry;

/// What an application's own interface answered. A missing reply is never success
/// inferred from a timeout; `unanswered` records that uncertainty explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Controlled {
    /// The application the person granted.
    pub application: String,
    /// The validated, fixed action that ran.
    pub action: String,
    /// A bounded read result, if one was actually returned.
    pub reply: Option<String>,
    /// The call was sent but its result could not be established.
    pub unanswered: bool,
}

pub(crate) fn controlling(
    machine: &Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<Controlled, NotDone>) {
    let strings = machine.strings();
    let agent = authorised.under().clone();
    let at = authorised.at();
    let (adapters, transport) = machine.application_controls();
    let result = alo_adapters::Driving::of(
        authorised,
        adapters,
        grants,
        machine.the_applications_installed(),
        strings,
    )
    .and_then(|driving| driving.deliver(transport.unwrap_or(&Unavailable), strings));
    match result {
        Ok(driven) => {
            let entry = Entry::ran(driven.authorised(), strings);
            let controlled = Controlled {
                application: driven.application().to_owned(),
                action: driven.authorised().verb().to_owned(),
                reply: driven.reply().map(str::to_owned),
                unanswered: driven.unanswered(),
            };
            (entry, Ok(controlled))
        }
        Err(refused) => (
            Entry::refused(&refused, &agent, strings, at),
            Err(NotDone::Refused(refused)),
        ),
    }
}

struct Unavailable;
impl alo_adapters::Delivers for Unavailable {
    fn deliver(&self, _: &alo_adapters::Message) -> Result<(), alo_adapters::NotDelivered> {
        Err(alo_adapters::NotDelivered::NotThere)
    }
}
