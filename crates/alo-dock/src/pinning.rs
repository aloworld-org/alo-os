//! What a fresh machine pins, and why *nothing pinned* is not the same fact as
//! *nobody has chosen yet*.
//!
//! The owner's ruling of 2026-10-10:
//!
//! > For each new person, initialise the pins once, in this order: Files,
//! > Browser, Apps, Settings. Use the actual installed applications providing
//! > those roles. Omit unavailable applications; never show placeholders or
//! > non-working buttons. …
//! >
//! > Distinguish "never initialised" from "initialised with no pins." An
//! > intentionally empty list must remain empty. Updates, missing applications
//! > and settings-read failures must not trigger default initialisation again.
//! > Applications omitted initially must not automatically become pinned if
//! > installed later.
//!
//! # The distinction is a type, because every other way of holding it decays
//!
//! An empty `Vec` cannot tell those two apart, and the difference is the whole
//! ruling: a machine that reads *no pins* as *not initialised* re-pins four
//! applications on the next boot, and a person who deliberately emptied their
//! Dock gets it filled again by an update. So [`ThePins`] has two states and
//! the empty one is reachable only through [`ThePins::Chosen`].
//!
//! **This is the opposite shape to every other setting in this crate.**
//! `crate::Changes` is release-coupled: absent means *whatever this release
//! ships*, so a release that moves a default reaches every machine that never
//! touched it. That is right for the Dock's edge and wrong for pins, because
//! the shipped default is four applications — absent would mean *put them
//! back*. The difference is deliberate and is the thing to read before changing
//! either.
//!
//! # What this crate decides, and what it does not
//!
//! It decides **which applications a fresh machine pins and in what order**.
//! It does not decide what is installed — [`ARole`] is filled by whoever knows
//! that, which is not a layout crate — and it does not decide how the answer is
//! stored. The owner put the persisted representation with the owner of
//! `alo-kept` and `where-a-persons-settings-are-kept-plan.md`;
//! `docs/contracts/person-settings.md` is where the two meet.

use crate::holding::Holding;
use crate::window::AppId;

/// A job a default pin is for, rather than an application's name.
///
/// **Roles and not names**, because the owner's ruling says *use the actual
/// installed applications providing those roles*: a machine with a different
/// file manager pins that one, and a machine with none pins nothing in its
/// place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ARole {
    /// A person's files.
    Files,
    /// The web.
    Browser,
    /// The application store.
    Apps,
    /// This machine's own settings.
    Settings,
}

impl ARole {
    /// Every role a fresh machine pins, **in the order they are pinned**.
    ///
    /// The order is the owner's and is part of the ruling rather than a
    /// consequence of how the enum is written: *Files, Browser, Apps,
    /// Settings*. A person reads their Dock left to right and this is that
    /// reading.
    pub const EVERY: [Self; 4] = [Self::Files, Self::Browser, Self::Apps, Self::Settings];
}

/// A person's pins, and whether they have ever been set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThePins {
    /// Nobody has set them. The defaults apply, **once**.
    ///
    /// A machine out of the box, and nothing else. A settings file that failed
    /// to read is **not** this: the owner's ruling is explicit that a read
    /// failure must not trigger initialisation, because a person whose disk
    /// hiccuped would find their Dock refilled with applications they had
    /// removed.
    NeverSet,
    /// Set, and this is what to. **Possibly to nothing**, which is a choice and
    /// stays one.
    Chosen(Vec<AppId>),
}

impl ThePins {
    /// What to put on the Dock, applying the defaults only if nothing has been
    /// set.
    ///
    /// `installed` says which application fills each role on this machine, in
    /// any order; a role nothing fills is **omitted** rather than drawn as a
    /// placeholder.
    ///
    /// **Answers the pins and whether they have just been initialised**, so a
    /// caller knows whether it has something new to persist. A caller that
    /// ignored that would re-derive the defaults every boot and never write
    /// them, which reads as working until somebody removes a pin.
    #[must_use]
    pub fn or_the_defaults(&self, installed: &[(ARole, AppId)]) -> (Vec<AppId>, JustInitialised) {
        match self {
            Self::Chosen(pins) => (pins.clone(), JustInitialised::No),
            Self::NeverSet => (what_a_fresh_machine_pins(installed), JustInitialised::Yes),
        }
    }

    /// The pins as a [`Holding`], ready to be asked what the Dock shows.
    #[must_use]
    pub fn holding(&self, installed: &[(ARole, AppId)]) -> (Holding, JustInitialised) {
        let (pins, initialised) = self.or_the_defaults(installed);
        let mut holding = Holding::nothing();
        for app in pins {
            holding.pin(app);
        }
        (holding, initialised)
    }
}

/// Whether the defaults were just applied, and so whether there is something
/// new to write down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustInitialised {
    /// They were. Persist them, or they will be applied again next time.
    Yes,
    /// They were not: these are the person's own pins, already stored.
    No,
}

/// Which applications a fresh machine pins, from those installed.
///
/// **In the roles' order and not the caller's.** `installed` may arrive in any
/// order — it is a lookup, not a list of pins — and the Dock a person first
/// sees reads Files, Browser, Apps, Settings whatever order the machine
/// discovered them in.
///
/// **A role nothing fills is omitted**, which is the owner's words: *omit
/// unavailable applications; never show placeholders or non-working buttons.* A
/// Dock of four icons where one opens nothing is worse than a Dock of three.
#[must_use]
pub fn what_a_fresh_machine_pins(installed: &[(ARole, AppId)]) -> Vec<AppId> {
    ARole::EVERY
        .into_iter()
        .filter_map(|role| {
            installed
                .iter()
                .find(|(filled, _)| *filled == role)
                .map(|(_, app)| app.clone())
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    //! The owner's five pin clauses, one test each where a test can hold one.
    //!
    //! **Inline rather than in a `pinning_tests.rs` of its own**, which is this
    //! crate's habit and is load-bearing here: `alo-choosing`'s i18n guard cuts
    //! a file at its first `#[cfg(test)]`, so an assertion message in an
    //! included file reads as English a person could meet. It is not — it is a
    //! sentence for whoever broke the test — and the cut is how this crate says
    //! so.

    use super::{ARole, JustInitialised, ThePins, what_a_fresh_machine_pins};
    use crate::window::AppId;

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    fn names(apps: &[AppId]) -> Vec<String> {
        apps.iter().map(|app| app.name().to_owned()).collect()
    }

    /// Everything a machine might have, in an order nothing asked for.
    fn a_full_machine() -> Vec<(ARole, AppId)> {
        vec![
            (ARole::Settings, app("alo.Settings")),
            (ARole::Files, app("org.gnome.Nautilus")),
            (ARole::Apps, app("alo.Apps")),
            (ARole::Browser, app("org.mozilla.firefox")),
        ]
    }

    /// **Files, Browser, Apps, Settings — in that order, whatever order they were
    /// discovered in.**
    #[test]
    fn a_fresh_machine_pins_the_four_roles_in_the_owners_order() {
        let pinned = what_a_fresh_machine_pins(&a_full_machine());
        assert_eq!(
            names(&pinned),
            [
                "org.gnome.Nautilus",
                "org.mozilla.firefox",
                "alo.Apps",
                "alo.Settings"
            ],
            "the Dock a person first sees does not read Files, Browser, Apps, Settings"
        );
    }

    /// **An unavailable application is omitted, not drawn as a placeholder.**
    ///
    /// The owner: *omit unavailable applications; never show placeholders or
    /// non-working buttons.* A Dock of four icons where one opens nothing is worse
    /// than a Dock of three.
    #[test]
    fn a_role_nothing_fills_leaves_no_gap_and_no_placeholder() {
        let without_a_store: Vec<_> = a_full_machine()
            .into_iter()
            .filter(|(role, _)| *role != ARole::Apps)
            .collect();
        let pinned = what_a_fresh_machine_pins(&without_a_store);
        assert_eq!(
            names(&pinned),
            ["org.gnome.Nautilus", "org.mozilla.firefox", "alo.Settings"],
            "a machine with no application store pinned something in its place, or left a hole"
        );
        assert_eq!(pinned.len(), 3);

        // And a machine with nothing at all pins nothing, rather than four
        // non-working buttons.
        assert!(what_a_fresh_machine_pins(&[]).is_empty());
    }

    /// **An intentionally empty list stays empty.**
    ///
    /// The clause this whole type exists for. A `Vec` alone cannot hold it: an
    /// empty one is the same value whether a person emptied their Dock or never
    /// touched it, and a machine that confused them would refill the Dock of the
    /// person who most clearly did not want one.
    #[test]
    fn a_person_who_emptied_their_dock_keeps_it_empty() {
        let emptied = ThePins::Chosen(Vec::new());
        let (pins, initialised) = emptied.or_the_defaults(&a_full_machine());
        assert!(
            pins.is_empty(),
            "a deliberately empty Dock was refilled with the defaults"
        );
        assert_eq!(initialised, JustInitialised::No);

        // And the other state, on the same machine, does fill it — so this test
        // cannot pass by the defaults never applying at all.
        let (fresh, initialised) = ThePins::NeverSet.or_the_defaults(&a_full_machine());
        assert_eq!(fresh.len(), 4);
        assert_eq!(initialised, JustInitialised::Yes);
    }

    /// **The defaults apply once.** Having applied them, a caller persists them and
    /// the next read is `Chosen` — so an update, a missing application or a failed
    /// read never re-runs them.
    #[test]
    fn the_defaults_are_applied_once_and_then_never_again() {
        let (first, initialised) = ThePins::NeverSet.or_the_defaults(&a_full_machine());
        assert_eq!(initialised, JustInitialised::Yes, "the premise");

        // What a caller stores, and what it reads back next time.
        let stored = ThePins::Chosen(first.clone());
        let (second, initialised) = stored.or_the_defaults(&a_full_machine());
        assert_eq!(second, first);
        assert_eq!(
            initialised,
            JustInitialised::No,
            "the defaults ran a second time, so a person's removals would be undone on every boot"
        );

        // **And a person's removal survives.** This is the update case: they take
        // the browser off, and the next read must not put it back.
        let without_the_browser: Vec<AppId> = first
            .into_iter()
            .filter(|app| app.name() != "org.mozilla.firefox")
            .collect();
        let (kept, initialised) =
            ThePins::Chosen(without_the_browser).or_the_defaults(&a_full_machine());
        assert_eq!(kept.len(), 3);
        assert!(
            !names(&kept).contains(&"org.mozilla.firefox".to_owned()),
            "an application the person removed was restored by the defaults"
        );
        assert_eq!(initialised, JustInitialised::No);
    }

    /// **An application omitted at first run does not pin itself when installed
    /// later.**
    ///
    /// The owner's words. It follows from the defaults applying once — but it
    /// follows *only* because they do, and this is the clause somebody would be
    /// most tempted to "fix" by re-deriving the defaults when the installed set
    /// changes.
    #[test]
    fn installing_an_application_later_does_not_pin_it() {
        let without_a_store: Vec<_> = a_full_machine()
            .into_iter()
            .filter(|(role, _)| *role != ARole::Apps)
            .collect();
        let (first, _) = ThePins::NeverSet.or_the_defaults(&without_a_store);
        assert_eq!(first.len(), 3, "the premise: the store was not there");

        // The store arrives. The person's pins are what they were.
        let (now, initialised) = ThePins::Chosen(first).or_the_defaults(&a_full_machine());
        assert_eq!(now.len(), 3, "installing an application pinned it");
        assert!(!names(&now).contains(&"alo.Apps".to_owned()));
        assert_eq!(initialised, JustInitialised::No);
    }

    /// **The pins reach the Dock in their order**, which is the one thing a person
    /// sees of all of this.
    #[test]
    fn the_pins_become_a_holding_in_the_same_order() {
        let (holding, initialised) = ThePins::NeverSet.holding(&a_full_machine());
        assert_eq!(initialised, JustInitialised::Yes);
        let pinned: Vec<String> = holding
            .each_pinned()
            .map(|app| app.name().to_owned())
            .collect();
        assert_eq!(
            pinned,
            [
                "org.gnome.Nautilus",
                "org.mozilla.firefox",
                "alo.Apps",
                "alo.Settings"
            ]
        );
    }
}
