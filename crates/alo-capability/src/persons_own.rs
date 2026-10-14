//! The applications that are a terminal, and so a person's unless they say
//! otherwise.
//!
//! **This file's name and its header both said *never an agent's*, and
//! [ADR 0111](../../../docs/decisions/0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md)
//! makes that false.** It supersedes
//! [ADR 0043](../../../docs/decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md),
//! on the owner's ruling: *we are not dictators so the users should be able to
//! choose if the agent can use the terminal or not.* The name is kept because
//! it is the identifier three files and a contract already use; what changed is
//! what it means, and this paragraph is where a reader finds that out.
//!
//! ADR 0043's reasoning was that *whatever is typed into it runs*, so a
//! terminal is the whole machine by another road. ADR 0111's answer is that the
//! road is **driving** a shell and not opening a window: nothing in the verb
//! list types. `docs/contracts/app-adapters.md` carries two verbs that reach an
//! application — `accessible.read_window`, a read, and
//! `accessible.activate_control`, which presses a named control — and neither
//! sends text. [`crate::a_terminal_for_the_agent`] holds that argument in full.
//!
//! So this list is still the list, and the three places that ask it now ask the
//! person's choice beside it. **A grant to `/` stays refused at every setting**,
//! which is the half of ADR 0043's reasoning that survives.
//!
//! Three places ask [`is_a_persons_own`], and all three ask about an **agent**:
//!
//! - [`crate::Grant::checked_for_when`] refuses to make the grant
//!   ([`crate::GrantError::APersonsOwn`]);
//! - [`crate::Grants::permitting_when`] refuses the ask whatever the list holds,
//!   so a grant written into the grants file by hand permits nothing;
//! - [`crate::Call::permitting_when`] refuses a call that *names* one in any
//!   argument, including a verb that requires no grant at all.
//!
//! Each has a shorter name without `_when` that means **the person has not
//! chosen**, so a caller that has not been taught about the choice keeps the
//! refusal it had. `CLAUDE.md`: *a flag that fails open is a flag that will be
//! forgotten in the direction that matters.*
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

/// Every application identifier that is a terminal.
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

/// Whether this application identifier is a terminal, and so reachable by an
/// agent only where the person has allowed it.
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
