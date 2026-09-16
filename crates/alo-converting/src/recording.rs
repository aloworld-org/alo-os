//! What the record keeps about a conversion.
//!
//! ADR 0039 §6: **every conversion and every refusal is recorded**, with the
//! grant it ran under and what the copy could not carry. A conversion that ran
//! — whether or not a copy came of it — is `alo-record`'s *ran*, carrying the
//! authorisation's grants and approval, stamped with exactly what the person
//! was told: the copy and what it lost, or why there is no copy. A conversion
//! the grants refused is `alo_record::Entry::refused`, as every other refusal
//! is, and needs nothing from here.

use alo_record::Entry;
use alo_strings::Strings;

use crate::converting::Done;

/// The entry a conversion that ran is recorded as.
#[must_use]
pub fn entry(done: &Done, strings: &Strings) -> Entry {
    Entry::ran(done.authorised(), strings).telling(&done.said(strings))
}
