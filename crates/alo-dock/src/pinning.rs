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
    /// set **and** discovery has something to say.
    ///
    /// **Three answers, not two, and the third is the owner's ruling of
    /// 2026-10-10:**
    ///
    /// > Unresolved roles remain absent. Never substitute an arbitrary
    /// > application or create a placeholder. However, an unfinished scan or
    /// > discovery failure is not proof that an application is absent: do not
    /// > permanently initialise an empty pin list from either. Initialise once
    /// > discovery has successfully completed, then preserve the person's saved
    /// > choices.
    ///
    /// This took a `&[(ARole, AppId)]` until that ruling, and an empty slice was
    /// the same value whether the machine had nothing installed or had not
    /// looked yet. The first is a Dock with nothing pinned, written down and
    /// kept; the second is a Dock a person would find permanently empty because
    /// their disk was slow the morning they first signed in. [`Discovery`] is
    /// what tells them apart, and [`WhatToPin::NotYet`] is the answer that
    /// cannot be stored.
    #[must_use]
    pub fn what_to_pin(&self, discovery: &Discovery) -> WhatToPin {
        match (self, discovery) {
            // The person's own, whatever discovery is doing. A pin to an
            // application that is not installed is the host's to resolve when it
            // is pressed; it is not this crate's to quietly drop, because
            // dropping it would turn *not found this boot* into *unpinned*.
            (Self::Chosen(pins), _) => WhatToPin::These(pins.clone()),
            (Self::NeverSet, Discovery::Completed(installed)) => {
                WhatToPin::TheseAndWriteThemDown(what_a_fresh_machine_pins(installed))
            }
            (Self::NeverSet, Discovery::StillLooking) => WhatToPin::NotYet(WhyNotYet::StillLooking),
            (Self::NeverSet, Discovery::Failed) => WhatToPin::NotYet(WhyNotYet::ItFailed),
        }
    }
}

/// What the host's application discovery has to say.
///
/// **The Dock consumes installed application identities; it does not scan.**
/// The owner's ruling of 2026-10-10: *the host's application-discovery
/// integration supplies the role mapping. The Dock consumes installed
/// application identities; it does not scan desktop entries or guess from
/// display names. Keep Linux-specific discovery outside the platform-independent
/// `alo-applications` model.*
///
/// So this is the **shape of the answer** and nothing else. Whoever reads
/// desktop entries off a Linux machine fills it; this crate could not and should
/// not. [`ARole`] is a Dock concept — which jobs a fresh Dock pins — and which
/// application fills one on this machine is not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Discovery {
    /// It ran to completion, and this is what fills each role.
    ///
    /// **Possibly nothing**, which is then a real answer: a machine with no
    /// file manager, no browser, no store and no settings pins nothing and that
    /// is written down. Compare [`Self::StillLooking`], where the same empty
    /// list means the opposite.
    Completed(Vec<(ARole, AppId)>),
    /// It has not finished. Nothing may be concluded from it yet.
    StillLooking,
    /// It failed. Nothing may be concluded from it either.
    Failed,
}

/// What a caller should put on the Dock, and whether to write it down.
///
/// **The two questions are separate and a `Vec` answers only the first.** A
/// caller holding a list has to decide whether to persist it, and the case that
/// goes wrong is the empty one: persisting an empty list from a scan that never
/// finished turns a slow morning into a permanently empty Dock. So the type
/// carries the decision rather than leaving it to a caller's judgement at every
/// call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatToPin {
    /// The person's own pins. Nothing to write down; they are already stored.
    These(Vec<AppId>),
    /// The defaults, just worked out. **Write them down**, or they are worked
    /// out again on the next boot and a pin the person removes comes back.
    TheseAndWriteThemDown(Vec<AppId>),
    /// Not knowable yet. Pin nothing **for now**, store nothing, and ask again
    /// when discovery has something to say.
    NotYet(WhyNotYet),
}

impl WhatToPin {
    /// The pins to draw now — empty for [`Self::NotYet`], which is *pin nothing
    /// for now* and never *the person pinned nothing*.
    #[must_use]
    pub fn to_draw(&self) -> &[AppId] {
        match self {
            Self::These(pins) | Self::TheseAndWriteThemDown(pins) => pins,
            Self::NotYet(_) => &[],
        }
    }

    /// Whether the caller owes the persistence layer a write.
    ///
    /// **False for [`Self::NotYet`]**, which is the whole of the owner's fourth
    /// clause: a caller that wrote here would store an empty list as a choice.
    #[must_use]
    pub const fn should_be_written_down(&self) -> bool {
        matches!(self, Self::TheseAndWriteThemDown(_))
    }

    /// The pins as a [`Holding`], ready to be asked what the Dock shows.
    #[must_use]
    pub fn holding(&self) -> Holding {
        let mut holding = Holding::nothing();
        for app in self.to_draw() {
            holding.pin(app.clone());
        }
        holding
    }
}

/// Why there is nothing to conclude yet.
///
/// Carried rather than collapsed into one variant, because the two have
/// different remedies: one is waited for and the other is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyNotYet {
    /// Discovery has not finished.
    StillLooking,
    /// Discovery failed.
    ItFailed,
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
///
/// Not public: a caller reaching this directly has skipped the question
/// [`ThePins::what_to_pin`] exists to ask, which is whether the defaults apply
/// at all. It was public until 2026-10-10 and had no caller outside this file.
fn what_a_fresh_machine_pins(installed: &[(ARole, AppId)]) -> Vec<AppId> {
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

    use super::{ARole, Discovery, ThePins, WhatToPin, WhyNotYet, what_a_fresh_machine_pins};
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
        let found = Discovery::Completed(a_full_machine());
        let emptied = ThePins::Chosen(Vec::new()).what_to_pin(&found);
        assert!(
            emptied.to_draw().is_empty(),
            "a deliberately empty Dock was refilled with the defaults"
        );
        assert!(!emptied.should_be_written_down());

        // And the other state, on the same machine, does fill it — so this test
        // cannot pass by the defaults never applying at all.
        let fresh = ThePins::NeverSet.what_to_pin(&found);
        assert_eq!(fresh.to_draw().len(), 4);
        assert!(fresh.should_be_written_down());
    }

    /// **The defaults apply once.** Having applied them, a caller persists them and
    /// the next read is `Chosen` — so an update, a missing application or a failed
    /// read never re-runs them.
    #[test]
    fn the_defaults_are_applied_once_and_then_never_again() {
        let found = Discovery::Completed(a_full_machine());
        let first = ThePins::NeverSet.what_to_pin(&found);
        assert!(first.should_be_written_down(), "the premise");
        let first: Vec<AppId> = first.to_draw().to_vec();

        // What a caller stores, and what it reads back next time.
        let second = ThePins::Chosen(first.clone()).what_to_pin(&found);
        assert_eq!(second.to_draw(), first);
        assert!(
            !second.should_be_written_down(),
            "the defaults ran a second time, so a person's removals would be undone on every boot"
        );

        // **And a person's removal survives.** This is the update case: they take
        // the browser off, and the next read must not put it back.
        let without_the_browser: Vec<AppId> = first
            .into_iter()
            .filter(|app| app.name() != "org.mozilla.firefox")
            .collect();
        let kept = ThePins::Chosen(without_the_browser).what_to_pin(&found);
        assert_eq!(kept.to_draw().len(), 3);
        assert!(
            !names(kept.to_draw()).contains(&"org.mozilla.firefox".to_owned()),
            "an application the person removed was restored by the defaults"
        );
        assert!(!kept.should_be_written_down());
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
        let first = ThePins::NeverSet.what_to_pin(&Discovery::Completed(without_a_store));
        let first: Vec<AppId> = first.to_draw().to_vec();
        assert_eq!(first.len(), 3, "the premise: the store was not there");

        // The store arrives. The person's pins are what they were.
        let now = ThePins::Chosen(first).what_to_pin(&Discovery::Completed(a_full_machine()));
        assert_eq!(
            now.to_draw().len(),
            3,
            "installing an application pinned it"
        );
        assert!(!names(now.to_draw()).contains(&"alo.Apps".to_owned()));
        assert!(!now.should_be_written_down());
    }

    /// **A scan that has not finished, and one that failed, initialise nothing.**
    ///
    /// The owner's ruling of 2026-10-10 and the reason [`Discovery`] exists:
    ///
    /// > An unfinished scan or discovery failure is not proof that an
    /// > application is absent: do not permanently initialise an empty pin list
    /// > from either.
    ///
    /// **Both halves matter and they are different.** Pinning nothing *for now*
    /// is right — there is nothing to pin yet. Writing that down is the fault:
    /// it turns a slow morning into a Dock that is empty for ever, and because
    /// the stored value would be `Chosen(vec![])`, every later boot would read
    /// it as a choice the person made.
    #[test]
    fn a_scan_that_did_not_finish_initialises_nothing_and_stores_nothing() {
        for (discovery, why) in [
            (Discovery::StillLooking, WhyNotYet::StillLooking),
            (Discovery::Failed, WhyNotYet::ItFailed),
        ] {
            let answer = ThePins::NeverSet.what_to_pin(&discovery);
            assert_eq!(
                answer,
                WhatToPin::NotYet(why),
                "{discovery:?} was read as an answer about what is installed"
            );
            assert!(
                answer.to_draw().is_empty(),
                "{discovery:?}: something was pinned from a scan with no result"
            );
            assert!(
                !answer.should_be_written_down(),
                "{discovery:?}: an empty pin list would be stored as the person's choice, and \
                 every later boot would read it back as one"
            );
        }

        // **The premise**: the same `ThePins` on the same machine *does* get the
        // four once discovery completes — so this test cannot pass by the
        // defaults never applying at all.
        let completed = ThePins::NeverSet.what_to_pin(&Discovery::Completed(a_full_machine()));
        assert_eq!(completed.to_draw().len(), 4);
        assert!(completed.should_be_written_down());
    }

    /// **A completed scan that found nothing is a real answer**, and the
    /// opposite of the one above.
    ///
    /// The same empty list, and this time it is written down: a machine with no
    /// file manager, no browser, no store and no settings pins nothing, and that
    /// is a fact rather than an absence of one. If this and the test above ever
    /// agree, [`Discovery`] has stopped carrying what it exists to carry.
    #[test]
    fn a_completed_scan_that_found_nothing_is_written_down() {
        let nothing_installed = ThePins::NeverSet.what_to_pin(&Discovery::Completed(Vec::new()));
        assert!(nothing_installed.to_draw().is_empty());
        assert!(
            nothing_installed.should_be_written_down(),
            "a machine that really has none of the four would re-run discovery's defaults on \
             every boot"
        );

        // And it is *not* the same answer as a scan that never finished, which
        // draws the same nothing.
        let never_finished = ThePins::NeverSet.what_to_pin(&Discovery::StillLooking);
        assert_eq!(never_finished.to_draw(), nothing_installed.to_draw());
        assert_ne!(
            never_finished.should_be_written_down(),
            nothing_installed.should_be_written_down(),
            "two empty Docks, one chosen and one not yet known, are being stored the same way"
        );
    }

    /// **The person's own pins are their own whatever discovery says.**
    ///
    /// A scan that has not finished must not empty a Dock somebody already has.
    /// This is the case that would bite hardest: every boot, briefly, until the
    /// scan lands — and if a caller persisted it, permanently.
    #[test]
    fn a_stored_dock_survives_a_scan_that_has_not_finished() {
        let theirs = ThePins::Chosen(vec![app("org.mozilla.firefox"), app("alo.Settings")]);
        for discovery in [
            Discovery::StillLooking,
            Discovery::Failed,
            Discovery::Completed(Vec::new()),
        ] {
            let answer = theirs.what_to_pin(&discovery);
            assert_eq!(
                names(answer.to_draw()),
                ["org.mozilla.firefox", "alo.Settings"],
                "{discovery:?} changed a Dock the person had already chosen"
            );
            assert!(!answer.should_be_written_down(), "{discovery:?}");
        }
    }

    /// **The pins reach the Dock in their order**, which is the one thing a person
    /// sees of all of this.
    #[test]
    fn the_pins_become_a_holding_in_the_same_order() {
        let holding = ThePins::NeverSet
            .what_to_pin(&Discovery::Completed(a_full_machine()))
            .holding();
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
