//! A person's own change to the printers, in Settings — through the same three
//! verbs an agent proposes.
//!
//! ADR 0009: *anything an agent verb can do, a person must also be able to do
//! by hand*, and ★ *printers, solved* names printers in Settings as the plain
//! way. So Settings lists printers ([`crate::Listed`]), a person picks one and
//! the change they picked, and that becomes **the broker's own verb** —
//! identical to what an agent's approved proposal becomes, crossing the same
//! door, written into the same record. There is no second road into the
//! machine's printers for a surface to take instead.
//!
//! The person's click is the approval. There is no turn behind it and no
//! proposal to number, so the token is issued under `alo_broker::BY_HAND`, and
//! the broker's record says a person made this change themselves.

use alo_broker::SystemVerb;

use crate::choosing::Listed;
use crate::wanted::Change;

/// The broker's verb for the change a person picked, to the printer they picked
/// it for.
///
/// A printer found is set up; a printer set up is removed or made the one this
/// machine prints on. Picking the wrong kind of printer for a change is a
/// surface drawing the wrong list, and the broker refuses it as no match: the
/// identity of a printer found is never the identity of one set up.
#[must_use]
pub fn picked(change: Change, printer: &Listed) -> SystemVerb {
    change.to(printer.identity())
}
