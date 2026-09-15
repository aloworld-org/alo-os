//! The applications that are a person's own, and never an agent's.
//!
//! [ADR 0043](../../../docs/decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md).
//! ADR 0001 §1 says no verb runs an arbitrary command, and ADR 0001 §3 says
//! there is no grant to `/`. A terminal is both rules met at once: whatever is
//! typed into it runs, with everything the person may do. So a terminal is the
//! whole machine by another road, and it is refused to an agent the way `/` is
//! — at the grant, where every verb written later inherits the refusal.
//!
//! Three places ask [`is_a_persons_own`], and all three ask about an **agent**:
//!
//! - [`crate::Grant::checked_for`] refuses to make the grant
//!   ([`crate::GrantError::APersonsOwn`]);
//! - [`crate::Grants::permitting`] refuses the ask whatever the list holds, so a
//!   grant written into the grants file by hand permits nothing;
//! - [`crate::Call::permitting`] refuses a call that *names* one in any argument,
//!   including a verb that requires no grant at all.
//!
//! An application is not refused: a person may choose the terminal to open
//! something another application hands it, and that is a portal request a person
//! answers.
//!
//! # A closed list, and what it is not
//!
//! Identifiers are matched exactly, as everywhere in this crate. The list is every
//! terminal emulator found by identifier on the place a fresh machine installs
//! from, on 2026-09-15, checked against that place's own catalogue — and **not a
//! proof of completeness**. ADR 0043 says what is owed: deciding by what an
//! application declares itself to be, which needs that declaration where a grant
//! is made.

/// Every application identifier that is a person's own, and never an agent's.
///
/// Sorted, so a test can hold that nothing appears twice and a reader can find
/// one. `alo-software`'s list of what a fresh machine ships is refused if its
/// terminal is not here.
pub const A_PERSONS_OWN: [&str; 7] = [
    "app.devsuite.Ptyxis",
    "com.raggesilver.BlackBox",
    "dev.boxi.Boxi",
    "org.contourterminal.Contour",
    "org.kde.konsole",
    "org.kde.qmlkonsole",
    "org.wezfurlong.wezterm",
];

/// Whether this application identifier is a person's own, and so never
/// reachable by an agent.
///
/// Exact, with no case folding and no trimming: a spelling that differs is a
/// different application to the rented tool as well, and a grant over it would
/// not reach this one.
#[must_use]
pub fn is_a_persons_own(identifier: &str) -> bool {
    A_PERSONS_OWN.contains(&identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_sorted_and_names_nothing_twice() {
        assert!(
            A_PERSONS_OWN
                .iter()
                .zip(A_PERSONS_OWN.iter().skip(1))
                .all(|(earlier, later)| earlier < later)
        );
    }

    #[test]
    fn a_terminal_is_a_persons_own_and_an_editor_is_not() {
        assert!(is_a_persons_own("app.devsuite.Ptyxis"));
        assert!(is_a_persons_own("org.kde.konsole"));
        assert!(!is_a_persons_own("org.gnome.TextEditor"));
        assert!(!is_a_persons_own(""));
    }

    /// **Exactly the identifier.** A spelling that differs names another
    /// application, which a grant over would not reach this one through either.
    #[test]
    fn only_the_exact_identifier_matches() {
        assert!(!is_a_persons_own("app.devsuite.ptyxis"));
        assert!(!is_a_persons_own(" app.devsuite.Ptyxis"));
        assert!(!is_a_persons_own("app.devsuite"));
    }
}
