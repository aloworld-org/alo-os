//! What the numbers must hold to, whatever `alo-access` decides next.

use alo_access::{Control, Surface};

use super::*;

/// Whether a set of state words has this bit set.
fn carries(words: [u32; 2], bit: u32) -> bool {
    let set = u64::from(words[0]) | (u64::from(words[1]) << 32);
    set & (1 << bit) != 0
}

/// Every state a control can be in.
const EVERY_STATE: [State; 4] = [
    State::ReadOnly,
    State::CanBeUsed,
    State::OnOrOff,
    State::AnnouncedWhenItChanges,
];

/// Every role a control can have, as this file must answer for each of them.
const EVERY_ROLE: [Role; 10] = [
    Role::Window,
    Role::Button,
    Role::Label,
    Role::Entry,
    Role::PasswordEntry,
    Role::StatusBar,
    Role::Switch,
    Role::List,
    Role::ListItem,
    Role::Dialogue,
];

/// **No two kinds of control are the same kind on the bus**, and none of them
/// is the shell itself or the box its controls hang in.
#[test]
fn every_kind_of_control_is_its_own_kind_on_the_bus() {
    let mut numbers: Vec<u32> = EVERY_ROLE.into_iter().map(number_of).collect();
    numbers.push(APPLICATION);
    numbers.push(FILLER);
    let how_many = numbers.len();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(numbers.len(), how_many, "two kinds share one number");
}

/// **A password field is its own kind**, and never the kind a field a person
/// types into is: no reader deciding what to read aloud can reach it by
/// matching on text fields.
#[test]
fn a_password_field_is_never_the_kind_an_ordinary_field_is() {
    assert_ne!(number_of(Role::PasswordEntry), number_of(Role::Entry));
}

/// **Nothing is chosen for the person, on the bus either** (ADR 0001). A
/// reader announces `IS_DEFAULT`, `FOCUSED` and `CHECKED`, and no state this
/// machine publishes carries any of them.
#[test]
fn nothing_published_reads_as_already_chosen() {
    for state in EVERY_STATE {
        let words = words_of(state);
        for (bit, what) in [
            (IS_DEFAULT, "the default"),
            (FOCUSED, "focused"),
            (CHECKED, "already on"),
        ] {
            assert!(
                !carries(words, bit),
                "{state:?} would be read as {what} before anybody chose anything"
            );
        }
    }
}

/// **What a reader is told it can act on is where the keyboard stops.**
///
/// `FOCUSABLE` follows `alo_access::Control::can_be_used`, which is the same
/// answer `alo_access::focus_order` filters on — so a control a reader offers
/// and the keyboard never reaches cannot be built.
#[test]
fn what_can_be_used_is_focusable_and_nothing_else_is() {
    for surface in Surface::ALL {
        for control in surface.read_aloud() {
            let focusable = carries(words_of(control.state), 11);
            assert_eq!(
                focusable,
                Control::can_be_used(&control),
                "{surface:?}: {:?} is offered and unreachable, or reachable and not offered",
                control.name.key()
            );
        }
    }
}

/// **Everything a reader is told about is on the screen**: a thing that is
/// visible and not showing is one a reader would read out of nowhere.
#[test]
fn everything_read_aloud_is_showing_as_well_as_visible() {
    for state in EVERY_STATE {
        let words = words_of(state);
        assert!(carries(words, 30), "{state:?} is not visible");
        assert!(carries(words, 25), "{state:?} is visible and not showing");
    }
}

/// **A surface that is not up is still described**: it stops showing and
/// stays visible, which is the difference between *not in front of you* and
/// *not there at all*.
#[test]
fn a_surface_that_is_not_up_stops_showing_and_is_still_described() {
    for state in EVERY_STATE {
        let up = words_of(state);
        let away = off_the_screen(up);
        assert!(carries(up, 25), "{state:?} was not showing to begin with");
        assert!(
            !carries(away, 25),
            "{state:?} still shows when it is not up"
        );
        assert!(carries(away, 30), "{state:?} stops being described as well");
    }
}

/// **What is announced is announced**, and nothing else is: the two
/// indicators the first law makes visible carry the `live` attribute, and no
/// other state does.
#[test]
fn only_what_changes_under_a_person_is_announced() {
    for state in EVERY_STATE {
        assert_eq!(
            is_announced(state),
            state == State::AnnouncedWhenItChanges,
            "{state:?}"
        );
    }
}

/// **A surface is a container or it is a filler**, and the roles that hold
/// others are the ones a surface is read as.
#[test]
fn the_roles_that_hold_others_are_the_ones_a_surface_can_be() {
    for role in EVERY_ROLE {
        assert_eq!(
            holds_others(role),
            matches!(
                role,
                Role::Window | Role::Dialogue | Role::StatusBar | Role::List
            ),
            "{role:?}"
        );
    }
}
