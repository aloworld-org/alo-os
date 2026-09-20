//! **Every road a keyboard takes through this crate's surfaces, walked
//! against the crate that decides them.**
//!
//! Task 12 of `docs/autonomy/v0-5-the-shell-plan.md`: *focus is always visible,
//! never trapped, and every action has the keyboard road `alo-access` lists,
//! with a test that walks them*. `alo-access` holds its own half already — its
//! `tests/everything_the_shell_offers_has_a_keyboard_road.rs` walks every
//! action in its model. This is the shell's half: the keys a person actually
//! presses on the surfaces this crate draws, held to what that crate decided
//! they mean.
//!
//! # Why this found two surfaces that disagreed
//!
//! `alo_access::leaving` says what Escape does on each surface, and two of this
//! crate's surfaces did something else: the approval surface ignored Escape,
//! and so did the sign-in screen. Both now do what that crate says — Escape
//! **answers no** on an approval, because a proposal left waiting behind
//! somebody's belief that they dealt with it is worse than either answer, and
//! it **clears what was typed** at sign-in, because there is nowhere further
//! back to go from the screen a machine starts at. Neither was noticed by a
//! test of either crate alone, which is what a walk across the two is for.
//!
//! # What this crate has no keyboard of its own for
//!
//! Four of the nine surfaces `alo-access` names have no key of this crate's:
//! the desktop, the dock, the status area and the window controls. The desktop
//! and the window controls are reached by the chords `alo-shortcuts` ships and
//! by the pointer; the dock's launcher and the desktop's *ask the agent* stop
//! are surfaces **this crate does not draw yet** — there is no launcher and no
//! agent overlay in it — so the road that ends at them cannot be walked here.
//! [`the_surfaces_this_crate_has_no_key_of_its_own_for`] names them, so the
//! lane that draws one is told to widen this walk in the same change.

use alo_access::tree::Surface;
use alo_access::{Leaving, focus_order, leaving};
use alo_shell::{ApprovalKey, RecordKey, RecoveryKey, SettingsKey, SignInKey};
use smithay::input::keyboard::{Keysym, ModifiersState};

/// No modifier held, which is how every one of these keys is pressed.
fn alone() -> ModifiersState {
    ModifiersState::default()
}

/// Every chord a person might be holding when a key of theirs arrives here.
fn chords() -> [ModifiersState; 3] {
    [
        ModifiersState {
            ctrl: true,
            ..alone()
        },
        ModifiersState {
            alt: true,
            ..alone()
        },
        ModifiersState {
            logo: true,
            ..alone()
        },
    ]
}

/// **The surfaces this crate draws a keyboard for**, and the surface each one
/// is in `alo-access`' list.
const THE_SURFACES_WITH_KEYS: [Surface; 5] = [
    Surface::Recovery,
    Surface::SignIn,
    Surface::Approval,
    Surface::Record,
    Surface::Settings,
];

/// **Escape leaves every surface, doing what that crate says it does.**
///
/// The one that matters most is the approval: Escape answers **no**. It must
/// never approve — what a person approves is the sentence (ADR 0001) — and it
/// must not leave the proposal unanswered either.
#[test]
fn escape_does_what_that_crate_says_on_every_surface_this_one_draws() {
    assert_eq!(
        leaving(Surface::Approval),
        Leaving::Declines,
        "that crate no longer says Escape declines; this crate followed it and must be told"
    );
    assert_eq!(
        ApprovalKey::of(Keysym::Escape, &alone()),
        ApprovalKey::Decline
    );

    assert_eq!(leaving(Surface::SignIn), Leaving::ClearsWhatWasTyped);
    assert_eq!(SignInKey::of(Keysym::Escape, &alone()), SignInKey::Clear);

    for surface in [Surface::Settings, Surface::Record] {
        assert_eq!(
            leaving(surface),
            Leaving::ReturnsToTheDesktop,
            "{surface:?}"
        );
    }
    assert_eq!(
        SettingsKey::of(Keysym::Escape, &alone()),
        SettingsKey::Close
    );
    assert_eq!(RecordKey::of(Keysym::Escape, &alone()), RecordKey::Close);

    // The screen a person reaches when the desktop will not start is the
    // floor: there is nowhere further back, and leaving it for real is
    // restarting the machine, which is not a key.
    assert_eq!(leaving(Surface::Recovery), Leaving::AlreadyTheFloor);
    assert_eq!(
        RecoveryKey::of(Keysym::Escape, &alone()),
        RecoveryKey::Nothing
    );
}

/// **Tab moves on every surface with more than one place to be, and Shift+Tab
/// moves back**, so a person who knows one key can reach everything on the
/// surface in front of them.
#[test]
fn tab_moves_forwards_and_back_on_every_surface_with_more_than_one_stop() {
    let shifted = ModifiersState {
        shift: true,
        ..alone()
    };

    assert_eq!(
        ApprovalKey::of(Keysym::Tab, &alone()),
        ApprovalKey::NextAnswer
    );
    assert_eq!(
        ApprovalKey::of(Keysym::Tab, &shifted),
        ApprovalKey::PreviousAnswer
    );
    assert_eq!(
        ApprovalKey::of(Keysym::ISO_Left_Tab, &alone()),
        ApprovalKey::PreviousAnswer
    );

    // Two fields, so forwards and backwards are one move: Tab from either
    // arrives at the other, which is a cycle and not a trap.
    for held in [alone(), shifted] {
        assert_eq!(
            SignInKey::of(Keysym::Tab, &held),
            SignInKey::OtherField,
            "{held:?}"
        );
    }
    assert_eq!(
        SignInKey::of(Keysym::ISO_Left_Tab, &alone()),
        SignInKey::OtherField
    );

    assert_eq!(
        SettingsKey::of(Keysym::Tab, &alone()),
        SettingsKey::NextSection
    );
    assert_eq!(
        SettingsKey::of(Keysym::Tab, &shifted),
        SettingsKey::PreviousSection
    );

    assert_eq!(RecoveryKey::of(Keysym::Tab, &alone()), RecoveryKey::Next);
    assert_eq!(
        RecoveryKey::of(Keysym::Tab, &shifted),
        RecoveryKey::Previous
    );
}

/// **Enter uses what the keyboard is on**, on every surface that has something
/// to use — and Space does the same wherever a person might reach for it.
#[test]
fn enter_uses_what_the_keyboard_is_on() {
    for enter in [Keysym::Return, Keysym::KP_Enter] {
        assert_eq!(ApprovalKey::of(enter, &alone()), ApprovalKey::Choose);
        assert_eq!(SettingsKey::of(enter, &alone()), SettingsKey::Choose);
        assert_eq!(SignInKey::of(enter, &alone()), SignInKey::Enter);
        assert_eq!(RecoveryKey::of(enter, &alone()), RecoveryKey::Choose);
    }
    assert_eq!(
        ApprovalKey::of(Keysym::space, &alone()),
        ApprovalKey::Choose
    );
    assert_eq!(
        SettingsKey::of(Keysym::space, &alone()),
        SettingsKey::Choose
    );
}

/// **A chord is never one of these keys.** A shortcut meant for an application
/// — Control, Alt or the logo key held — never answers a question, revokes a
/// grant, types into a password field or goes back to the build before.
#[test]
fn a_chord_meant_for_something_else_never_acts_on_one_of_these_surfaces() {
    for held in chords() {
        for symbol in [
            Keysym::Tab,
            Keysym::Return,
            Keysym::space,
            Keysym::Escape,
            Keysym::a,
        ] {
            assert_eq!(
                ApprovalKey::of(symbol, &held),
                ApprovalKey::Nothing,
                "{symbol:?} with {held:?}"
            );
            assert_eq!(
                SettingsKey::of(symbol, &held),
                SettingsKey::Nothing,
                "{symbol:?} with {held:?}"
            );
            assert_eq!(
                RecoveryKey::of(symbol, &held),
                RecoveryKey::Nothing,
                "{symbol:?} with {held:?}"
            );
            assert_eq!(
                RecordKey::of(symbol, &held),
                RecordKey::Nothing,
                "{symbol:?} with {held:?}"
            );
        }
        assert_eq!(SignInKey::of(Keysym::a, &held), SignInKey::Nothing);
    }
}

/// **Every surface `alo-access` names has somewhere for the keyboard to
/// be**, and the ones this crate draws a key of its own for are the five
/// walked above.
///
/// The other four are named here rather than left out: the desktop and the
/// window controls answer to the chords `alo-shortcuts` ships, and the dock's
/// launcher and the desktop's *ask the agent* stop are **not drawn by this
/// crate yet** — there is no launcher and no agent overlay in it. When one is
/// drawn, this list is wrong, and it fails here rather than leaving a surface
/// nobody walks.
#[test]
fn the_surfaces_this_crate_has_no_key_of_its_own_for() {
    let without: Vec<Surface> = Surface::ALL
        .into_iter()
        .filter(|surface| !THE_SURFACES_WITH_KEYS.contains(surface))
        .collect();
    assert_eq!(
        without,
        vec![
            Surface::Desktop,
            Surface::Dock,
            Surface::StatusArea,
            Surface::WindowControls,
        ],
        "a surface was added or its keyboard was written; this walk has to be told"
    );
    for surface in Surface::ALL {
        // The status area is the one surface with no stop, and that is its
        // decision rather than an omission: what is leaving and what the agent
        // is doing are announced when they change, never waited for by
        // somebody tabbing to them. Every other surface has somewhere for a
        // keyboard to be.
        assert_eq!(
            focus_order(surface).is_empty(),
            surface == Surface::StatusArea,
            "{surface:?} has nowhere for a keyboard to be at all"
        );
    }
}
