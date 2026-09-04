//! Which places a turn is bound to, and why they are the execution's rather
//! than the turn's.
//!
//! `place.rs` is the conversion — one path, as the two numbers the kernel knows
//! it by. This is the decision on top of it: **a turn is bound to the places
//! this execution named, not to everywhere its grants reach.**
//!
//! # Item 26b's question, and the answer
//!
//! Item 26a left one thing open. A turn's grants can cover several folders and
//! one call can name two paths, so either the bound is the turn's whole grant or
//! it is narrower than that. It is narrower, and there are three reasons, only
//! the first of which is a principle:
//!
//! - **Least privilege.** A turn holding a grant over four folders is doing one
//!   thing at a time, and the boundary around that one thing has no reason to
//!   include the other three. A verb with a bug reaches what the call it was
//!   part of named, which is the smallest thing ADR 0013 can be made to mean.
//! - **It fits.** The widest call on this machine names two paths, and the
//!   grants a person has made have no width at all — one entry would have to
//!   hold however many folders somebody had granted, which is not a number.
//! - **It is the same list the record names.** What the record says an execution
//!   touched and what the kernel let it touch are then one set of paths, so the
//!   two cannot disagree about what happened.
//!
//! # A path that does not exist yet is the folder it will be made in
//!
//! A rename invents a name, a move makes one inside a folder, and an archive
//! creates a file. None of those can be looked up, because none of them is
//! there — and the thing that has to be opened in order to create them is the
//! **folder**, which is. So a caller hands over the folder, and
//! [`places_of`] never has to answer *where is a file that is not there yet*:
//! it takes paths that exist, and one that does not is
//! [`NotBounded::NotAPlace`] rather than a place made up.
//!
//! Which paths those are for each of the six verbs is the wiring's, and the
//! wiring is queue item 26c. What is settled here is that the bound is made of
//! them and of nothing else.
//!
//! # Too many is refused, and refusing is the only safe direction
//!
//! One entry holds [`alo_bounding_map::PLACES`] places. A call needing more is
//! [`NotBounded::TooManyPlaces`] and the turn does not run, because the
//! alternative — bounding it to the first four and letting it fail on the fifth
//! — is a turn that half works, and ADR 0015's rule is that a turn whose
//! boundary cannot be applied does not run at all.

use std::path::Path;

use alo_bounding_map::{Bounds, PLACES};

use crate::{failing::NotBounded, place::place_of};

/// The bound a turn is given, made of the paths this execution named.
///
/// Every path is looked up, so a granted folder reached through a symbolic link
/// is the folder rather than the link, and a path that is not there is refused
/// rather than turned into a place made of zeroes.
///
/// # Errors
/// [`NotBounded::NothingToBound`] when no path was named — a boundary around
/// nothing is a turn that cannot open anything rather than a narrower grant.
/// [`NotBounded::TooManyPlaces`] when there are more than one entry holds.
/// [`NotBounded::NotAPlace`] when the machine would not say where a path leads.
pub fn places_of(paths: &[&Path]) -> Result<Bounds, NotBounded> {
    if paths.is_empty() {
        return Err(NotBounded::NothingToBound);
    }
    if paths.len() > PLACES {
        return Err(NotBounded::TooManyPlaces {
            asked: paths.len(),
            most: PLACES,
        });
    }

    // Both refusals above happen before anything is looked up, which is
    // `alo-files`' order for the same reason: a call that cannot be bounded is
    // refused without the machine being asked about paths it named.
    let mut places = Vec::with_capacity(paths.len());
    for path in paths {
        places.push(place_of(path)?);
    }

    // Cannot be reached — the width was settled above and one place per path
    // was added. It is an error rather than an unwrap because a library inside
    // the daemon that panics takes the daemon with it, and it is the *same*
    // error because if it ever happened it would be for the same reason.
    Bounds::of(&places).ok_or(NotBounded::TooManyPlaces {
        asked: places.len(),
        most: PLACES,
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    /// A folder of our own with a folder inside it, so the tests below name real
    /// paths on the machine they run on.
    fn a_folder_of_our_own(what: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("alo-places-{}-{what}", std::process::id()));
        fs::create_dir_all(&root).expect("a temporary directory can be made");
        root
    }

    /// **A call that names two paths is bounded to both.** `move_file` is the
    /// case: the file and the folder it is going into, in two different parts of
    /// the tree, and a bound over only the first would refuse every move on the
    /// machine.
    #[test]
    fn the_paths_an_execution_named_are_the_places_it_is_bounded_to() {
        let root = a_folder_of_our_own("two");
        let invoices = root.join("Invoices");
        let archive = root.join("Archive");
        fs::create_dir_all(&invoices).expect("a temporary directory can be made");
        fs::create_dir_all(&archive).expect("a temporary directory can be made");

        let bound = places_of(&[&invoices, &archive]).expect("both folders are there");
        assert_eq!(bound.len(), 2);
        assert!(bound.holds(place_of(&invoices).expect("it is there")));
        assert!(bound.holds(place_of(&archive).expect("it is there")));
        assert!(!bound.holds(place_of(&root).expect("it is there")));
    }

    /// **A boundary around nothing is refused**, because it is not a narrower
    /// grant — it is a turn that cannot open anything, which would reach whoever
    /// was watching as the machine failing rather than as a call being stopped.
    #[test]
    fn a_turn_bounded_to_nothing_is_refused_before_it_runs() {
        assert!(matches!(places_of(&[]), Err(NotBounded::NothingToBound)));
    }

    /// **More places than one entry holds is refused, not truncated.** A turn
    /// bounded to the first four of five would work for part of what it was
    /// asked to do, and ADR 0015's rule is that a boundary that cannot be
    /// applied means the turn does not run.
    #[test]
    fn more_places_than_an_entry_holds_is_refused_rather_than_cut_down() {
        let root = a_folder_of_our_own("too-many");
        let mut made = Vec::new();
        for which in 0..=PLACES {
            let folder = root.join(format!("folder-{which}"));
            fs::create_dir_all(&folder).expect("a temporary directory can be made");
            made.push(folder);
        }
        let named: Vec<&Path> = made.iter().map(PathBuf::as_path).collect();

        let too_many = places_of(&named).expect_err("one more than PLACES is too many");
        assert!(matches!(
            too_many,
            NotBounded::TooManyPlaces { asked, most } if asked == PLACES + 1 && most == PLACES
        ));
        assert!(
            places_of(named.get(..PLACES).expect("PLACES of them were made")).is_ok(),
            "the widest bound there is was refused"
        );
    }

    /// A path that is not there is a refusal naming it, rather than a place made
    /// of zeroes — which would be a bound to whichever filesystem happens to be
    /// numbered nothing. `place.rs`'s rule, asked at the door a turn arrives by.
    #[test]
    fn a_path_that_is_not_there_is_not_a_place_to_bound_a_turn_to() {
        let root = a_folder_of_our_own("missing");
        let gone = root.join("nothing-here");
        assert!(matches!(
            places_of(&[&root, &gone]),
            Err(NotBounded::NotAPlace { path, .. }) if path == gone.display().to_string()
        ));
    }

    /// One path is the bound `alo_bounding_map::Bounds::of_one` makes, so a
    /// caller holding a place and a caller holding a path arrive at the same
    /// value rather than at two that currently agree.
    #[test]
    fn one_path_is_the_bound_the_door_that_cannot_fail_makes() {
        let root = a_folder_of_our_own("one");
        let place = place_of(&root).expect("it is there");
        assert_eq!(
            Bounds::of_one(place),
            places_of(&[&root]).expect("it is there")
        );
    }
}
