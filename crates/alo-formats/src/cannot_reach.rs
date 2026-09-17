//! **What CLDR has and the rented crate does not hand over.**
//!
//! This plan's constraint is that the formats come from rented CLDR data and
//! that no CLDR data is edited. Two of the six things task 6 asks for are in
//! CLDR and are not in `icu` 2's stable surface, so they are written down here
//! as what they are — a gap with a name and an address — rather than typed into
//! a table of our own, which is the thing the constraint forbids and which would
//! be wrong first in the languages nobody here reads.

/// One thing CLDR knows that this machine cannot ask it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Missing {
    /// What a person would have wanted.
    pub wanted: &'static str,
    /// Where CLDR keeps it.
    pub cldr_keeps_it_in: &'static str,
    /// Why the rented crate does not hand it over.
    pub why_it_cannot_be_reached: &'static str,
    /// What it would take, for whoever decides to pay it.
    pub what_it_would_take: &'static str,
}

/// **The two**, as of `icu` 2 and 2026-09-17.
pub const fn what_cldr_has_that_this_cannot_reach() -> [Missing; 2] {
    [
        Missing {
            wanted: "how an amount of money is written",
            cldr_keeps_it_in: "common/main/<language>.xml, the `currencyFormats` and \
                               `currencies` blocks",
            why_it_cannot_be_reached: "currency formatting lives in `icu_experimental`, which \
                                       the `icu` crate does not re-export; its API is not \
                                       stable, and pinning an unstable one here pins it for \
                                       every crate in this workspace",
            what_it_would_take: "either depending on `icu_experimental` and accepting that it \
                                 breaks between releases, or reading CLDR's own JSON at build \
                                 time — both are decisions with an ADR's weight, and neither \
                                 belongs in a task that was asked to rent",
        },
        Missing {
            wanted: "which paper a printer is given by default",
            cldr_keeps_it_in: "common/supplemental/supplementalData.xml, `measurementData`'s \
                               `paperSize`",
            why_it_cannot_be_reached: "`icu` exposes no measurement data at all, in any module",
            what_it_would_take: "reading that one file from CLDR, or asking the print system, \
                                 which knows the answer for the printer that is actually there \
                                 — `alo-printing` is where that question lives",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A gap says where the data is and what reaching it would cost**, or it
    /// is a complaint rather than a finding.
    #[test]
    fn every_gap_names_the_data_and_the_price_of_reaching_it() {
        for missing in what_cldr_has_that_this_cannot_reach() {
            assert!(missing.cldr_keeps_it_in.contains("common/"));
            assert!(missing.why_it_cannot_be_reached.len() > 40);
            assert!(missing.what_it_would_take.len() > 40);
        }
    }
}
