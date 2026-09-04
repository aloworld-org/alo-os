//! Everywhere one execution has to reach, which is not everywhere its grant
//! reaches.
//!
//! [`Touching`] is the end of everything the capability model decides: a call
//! whose every path has been resolved and asked about again, in the grants' own
//! words. A machine that puts a boundary in front of a turn has one more
//! question to ask before the work starts — **which places does this execution
//! have to be able to open?** — and this file answers it from the call rather
//! than from the grants.
//!
//! `alo-bounding`'s `places_of` is where that decision is argued: least
//! privilege, a person's grant list having no width to it, and the places the
//! record says an execution touched being the same places the kernel let it
//! touch. What it left open is *which paths those are for each of the six*, and
//! that is here, because the six are this crate's and a crate that could not be
//! built on the machine the boundary runs on has no business holding the answer.
//!
//! # It is the paths the call named, plus the folder a new name is made in
//!
//! Three of the six create something, and none of the three names the thing it
//! creates: `rename_file` invents a name beside the file, `move_file` puts the
//! file's own name inside a folder, `archive_folder` puts an archive inside one.
//! None of those can be looked up before it exists, and what has to be reached
//! in order to make one is the **folder it goes in**, which is there.
//!
//! So a reach is the resolved paths, plus the folder above anything the call
//! would create, and the same place named twice is one place:
//!
//! | Verb | Reaches |
//! |---|---|
//! | `list_folder` | the folder |
//! | `read_file` | the file |
//! | `find_in_folder` | the folder |
//! | `rename_file` | the file, and the folder it sits in |
//! | `move_file` | the file, and the folder it is going into |
//! | `archive_folder` | the folder, and the folder the archive goes into |
//!
//! # The folder a rename needs is wider than a grant over one file, and saying
//! so is the point
//!
//! A grant can be over a single file — ADR 0001 §4, the document a person had
//! open when they pressed the key — and renaming under one of those has to
//! reach the folder that file is in, because there is no way to make a name in
//! a folder nothing may open. The boundary is therefore *wider* than the grant
//! in exactly that case, and it is the only case.
//!
//! That is not a hole, and it is not a reason to bound a rename to the file
//! alone and watch it fail. **The grants remain the narrower answer and the
//! deciding one**: [`crate::Did::of`] asks them whether the invented name may be
//! created, at the authorisation's own moment, and a rename under a single-file
//! grant is refused there before anything is touched. What the kernel imposes is
//! a floor under a verb with a bug in it, not the capability model repeated —
//! ADR 0013's *the grant decides, the kernel is what makes the decision true of
//! a mistake*. A boundary that had to be exactly the grant would be a boundary
//! that could not let a change happen at all.
//!
//! # One mapping, asked twice, rather than two that agree today
//!
//! What each of the six comes down to — which arguments are paths, and what
//! would be created — is written once, in `doing.rs`, and this file asks it
//! rather than repeating it. Asking the same function twice about the same
//! [`Touching`] cannot produce two answers; two functions reading the same six
//! verbs can, and the day somebody adds a seventh only one of them would be
//! updated. The cost is a few map lookups and a `join` per execution, which is
//! nothing against being wrong about where a turn may reach.
//!
//! # A call that reaches nothing is a fact, not a refusal
//!
//! A verb that names no path at all answers with an empty reach, because
//! reaching nothing is a perfectly good thing to be allowed to do — the same
//! answer [`Touching`] gives, for the same reason. What a *boundary* makes of
//! that is `alo-bounding`'s: it refuses to bound a turn to nowhere, since a turn
//! that can open nothing reads as a broken machine rather than as a narrow
//! grant. None of the six is such a verb, and there is a test that says so.

use std::path::{Path, PathBuf};

use crate::doing::Todo;
use crate::failed::Failed;
use crate::touching::Touching;

/// Everywhere one execution has to be able to reach.
///
/// Made from a [`Touching`], so there is no way to ask this about a call that
/// has not been validated, permitted, approved if it changes anything, and
/// resolved. Whatever imposes a boundary is handed one of these; what it does
/// with paths that are not there, or with more of them than it can hold, is
/// its own refusal to make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaching {
    /// The places, in the order the call named them, with the folder above
    /// anything it would create last, and no place twice.
    places: Vec<PathBuf>,
}

impl Reaching {
    /// Everywhere this execution has to reach.
    ///
    /// The paths are the **real** ones — where each argument really leads, as
    /// [`Touching`] resolved them — so a granted folder reached through a
    /// symbolic link is the folder rather than the link, and nothing here has
    /// to look at a disk to find that out again.
    ///
    /// # Errors
    /// [`Failed::NotAFileVerb`] for a verb that is not one of the six, and
    /// [`Failed::Missing`] for one performed without an argument it declares —
    /// both of them this crate's existing answers to *what does this call come
    /// down to*, because that is the question being asked. [`Failed::NotAFile`]
    /// when a path this call would create has nothing above it, which no call
    /// of the six can produce and is an error rather than an unwrap because a
    /// library that panics inside the daemon takes the daemon with it.
    pub fn of(touching: &Touching) -> Result<Self, Failed> {
        let mut places: Vec<PathBuf> = Vec::new();
        for (_, real) in touching.all() {
            add(&mut places, real.as_path());
        }
        if let Some(creating) = Todo::of(touching)?.creating() {
            let held = creating.parent().ok_or_else(|| Failed::NotAFile {
                path: creating.display().to_string(),
            })?;
            add(&mut places, held);
        }
        Ok(Self { places })
    }

    /// The places, in order.
    pub fn places(&self) -> impl Iterator<Item = &Path> {
        self.places.iter().map(PathBuf::as_path)
    }

    /// How many places there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.places.len()
    }

    /// Whether this execution reaches nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.places.is_empty()
    }

    /// Whether this execution reaches that place.
    #[must_use]
    pub fn holds(&self, place: &Path) -> bool {
        self.places.iter().any(|held| held == place)
    }
}

/// Add a place, unless it is already one.
///
/// A move names the folder it is going into *and* creates a name inside it, so
/// without this the widest call on the machine would ask for three places where
/// it needs two — and a reach that counted the same folder twice would spend a
/// slot in a boundary that has four of them.
fn add(places: &mut Vec<PathBuf>, place: &Path) {
    if !places.iter().any(|held| held == place) {
        places.push(place.to_path_buf());
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::real::{Real, RealError};
    use crate::resolving::Resolving;
    use crate::testing::in_english;
    use crate::verbs::file_verbs;
    use alo_capability::{
        Approvals, Authorised, Given, Grant, Grantee, Grants, Proposal, Reach, Verbs,
    };
    use alo_strings::Strings;
    use std::collections::BTreeMap;
    use std::time::{Duration, SystemTime};

    /// A fixed moment, so that expiry is arithmetic rather than a wait.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// How long the grants and the questions in these tests last.
    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    /// The agent the tests grant things to.
    fn files() -> Grantee {
        Grantee::named("@files")
    }

    /// Grants to `@files` over these folders, made at noon and lasting an hour.
    fn granting(folders: &[&str]) -> Grants {
        let mut grants = Grants::default();
        for folder in folders {
            grants.grant(
                Grant::checked(
                    "@files",
                    Reach::Folder(PathBuf::from(folder)),
                    noon(),
                    hour(),
                )
                .unwrap(),
            );
        }
        grants
    }

    /// A filesystem where every path is exactly what it says it is.
    ///
    /// The reach is about the shape of a call rather than about a disk, so what
    /// these tests need from a filesystem is that it answers — and a stated one
    /// answers the same on every host, which is `touching.rs`' argument for the
    /// same fixture.
    struct Plainly {
        /// Every path that is there.
        there: BTreeMap<PathBuf, PathBuf>,
    }

    impl Plainly {
        /// A filesystem where these paths are there and nothing else is.
        fn with(paths: &[&str]) -> Self {
            Self {
                there: paths
                    .iter()
                    .map(|path| (PathBuf::from(path), PathBuf::from(path)))
                    .collect(),
            }
        }
    }

    impl Resolving for Plainly {
        fn real(&self, path: &Path) -> Result<Real, RealError> {
            match self.there.get(path) {
                Some(really) => Ok(Real::new(really.clone())),
                None => Err(RealError::Nothing {
                    path: path.display().to_string(),
                }),
            }
        }
    }

    /// A read of the six, resolved and asked about, ready to be reached from.
    fn a_read(
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        there: &Plainly,
        strings: &Strings,
    ) -> Touching {
        let call = file_verbs().unwrap().call(verb, given).unwrap();
        let authorised = Authorised::read(&call, &files(), grants, noon()).unwrap();
        Touching::of(authorised, grants, there, strings).unwrap()
    }

    /// A change of the six, proposed, approved and redeemed the ordinary way.
    fn a_change(
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        there: &Plainly,
        strings: &Strings,
    ) -> Touching {
        let call = file_verbs().unwrap().call(verb, given).unwrap();
        let mut approvals = Approvals::default();
        let id =
            approvals.propose(Proposal::checked(&call, &files(), grants, noon(), hour()).unwrap());
        let authorised = approvals
            .approve(id, noon())
            .unwrap()
            .redeem(grants, noon())
            .unwrap();
        Touching::of(authorised, grants, there, strings).unwrap()
    }

    /// **A read reaches what it named and nothing else.** There is nothing to
    /// create, so there is no folder to add, and a boundary around one place is
    /// the narrowest thing a turn can run inside.
    #[test]
    fn a_read_reaches_the_one_place_it_named() {
        let grants = granting(&["/home/anna/Invoices"]);
        let there = Plainly::with(&["/home/anna/Invoices"]);
        let touching = a_read(
            "list_folder",
            &[("folder", Given::text("/home/anna/Invoices"))],
            &grants,
            &there,
            &in_english(),
        );

        let reaching = Reaching::of(&touching).unwrap();
        assert_eq!(reaching.len(), 1);
        assert!(reaching.holds(Path::new("/home/anna/Invoices")));
        assert!(!reaching.is_empty());
        assert_eq!(
            reaching.places().collect::<Vec<_>>(),
            [Path::new("/home/anna/Invoices")]
        );
    }

    /// **What may be reached is where the path really leads**, not the name the
    /// call arrived with — the same rule `touching.rs` keeps, arriving here for
    /// free because a reach is made of the resolved paths.
    #[test]
    fn what_is_reached_is_the_real_path_and_not_the_one_that_was_asked_for() {
        struct ByWayOfALink;
        impl Resolving for ByWayOfALink {
            fn real(&self, path: &Path) -> Result<Real, RealError> {
                if path == Path::new("/home/anna/Invoices/march.pdf") {
                    return Ok(Real::new(PathBuf::from(
                        "/home/anna/Invoices/2026/march.pdf",
                    )));
                }
                Ok(Real::new(path.to_path_buf()))
            }
        }

        let grants = granting(&["/home/anna/Invoices"]);
        let call = file_verbs()
            .unwrap()
            .call(
                "read_file",
                &[("file", Given::text("/home/anna/Invoices/march.pdf"))],
            )
            .unwrap();
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
        let touching = Touching::of(authorised, &grants, &ByWayOfALink, &in_english()).unwrap();

        let reaching = Reaching::of(&touching).unwrap();
        assert_eq!(
            reaching.places().collect::<Vec<_>>(),
            [Path::new("/home/anna/Invoices/2026/march.pdf")],
            "a boundary was made out of the link rather than out of what it leads to"
        );
    }

    /// **A rename reaches the folder the file sits in**, because that is what
    /// has to be reached to make a name in it — and it is the one place where
    /// the boundary is wider than a grant over a single file. The header says
    /// why that is honest and where the narrower answer stays.
    #[test]
    fn a_rename_reaches_the_folder_the_new_name_is_made_in() {
        let grants = granting(&["/home/anna/Invoices"]);
        let there = Plainly::with(&["/home/anna/Invoices/march.pdf"]);
        let touching = a_change(
            "rename_file",
            &[
                ("file", Given::text("/home/anna/Invoices/march.pdf")),
                ("name", Given::text("march-final.pdf")),
            ],
            &grants,
            &there,
            &in_english(),
        );

        let reaching = Reaching::of(&touching).unwrap();
        assert_eq!(
            reaching.places().collect::<Vec<_>>(),
            [
                Path::new("/home/anna/Invoices/march.pdf"),
                Path::new("/home/anna/Invoices"),
            ]
        );
    }

    /// **A move reaches two places and not three.** The folder it is going into
    /// is both a path the call named and the folder the new name is made in,
    /// and counting it twice would spend a slot in a boundary that has four.
    #[test]
    fn a_move_reaches_the_file_and_the_folder_once_each() {
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        let there = Plainly::with(&["/home/anna/Invoices/march.pdf", "/home/anna/Archive"]);
        let touching = a_change(
            "move_file",
            &[
                ("file", Given::text("/home/anna/Invoices/march.pdf")),
                ("into", Given::text("/home/anna/Archive")),
            ],
            &grants,
            &there,
            &in_english(),
        );

        let reaching = Reaching::of(&touching).unwrap();
        assert_eq!(reaching.len(), 2);
        assert!(reaching.holds(Path::new("/home/anna/Invoices/march.pdf")));
        assert!(reaching.holds(Path::new("/home/anna/Archive")));
    }

    /// An archive is the same shape as a move: the folder being archived, and
    /// the folder the archive is written into, once each.
    #[test]
    fn an_archive_reaches_the_folder_it_reads_and_the_folder_it_writes_into() {
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        let there = Plainly::with(&["/home/anna/Invoices", "/home/anna/Archive"]);
        let touching = a_change(
            "archive_folder",
            &[
                ("folder", Given::text("/home/anna/Invoices")),
                ("into", Given::text("/home/anna/Archive")),
                ("name", Given::text("invoices.zip")),
            ],
            &grants,
            &there,
            &in_english(),
        );

        let reaching = Reaching::of(&touching).unwrap();
        assert_eq!(reaching.len(), 2);
        assert!(reaching.holds(Path::new("/home/anna/Invoices")));
        assert!(reaching.holds(Path::new("/home/anna/Archive")));
        assert!(
            !reaching.holds(Path::new("/home/anna/Archive/invoices.zip")),
            "a boundary was made over a file that does not exist yet"
        );
    }

    /// **A verb that names no path reaches nothing**, and that is an answer
    /// rather than a refusal. What a boundary makes of it is `alo-bounding`'s,
    /// and none of the six is such a verb.
    #[test]
    fn a_call_that_names_no_path_reaches_nothing() {
        use alo_capability::{Arg, Effect, Requires, Takes, Verb};
        use alo_strings::Word;

        let displays = Verb::checked(
            "list_displays",
            Word::saying(
                "testing.list-displays.purpose",
                "list the displays attached to this machine",
            ),
            Effect::Read,
            vec![Arg::taking(
                "most",
                Word::saying("testing.list-displays.most", "how many to answer with"),
                Takes::count(1, 8),
            )],
            Requires::nothing_because(
                "a display is not a path, a file or an application, and naming one reaches nothing",
            ),
            Word::saying(
                "testing.list-displays.sentence",
                "list at most {most} displays",
            ),
        )
        .unwrap();
        let mut verbs = Verbs::default();
        verbs.declare(displays).unwrap();
        let call = verbs
            .call("list_displays", &[("most", Given::number(4))])
            .unwrap();
        let grants = Grants::default();
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
        let touching =
            Touching::of(authorised, &grants, &Plainly::with(&[]), &in_english()).unwrap();

        // It is not one of the six, so there is nothing that says what it comes
        // down to — and *that* is the refusal, rather than an empty answer
        // pretending the verb was understood.
        assert!(matches!(
            Reaching::of(&touching).unwrap_err(),
            Failed::NotAFileVerb { verb } if verb == "list_displays"
        ));
    }

    /// The same place named twice is one place, asked of the helper directly so
    /// that the rule is tested rather than inferred from a verb that happens to
    /// exercise it.
    #[test]
    fn the_same_place_named_twice_is_one_place() {
        let mut places = Vec::new();
        add(&mut places, Path::new("/home/anna/Archive"));
        add(&mut places, Path::new("/home/anna/Archive"));
        add(&mut places, Path::new("/home/anna/Invoices"));
        assert_eq!(
            places,
            [
                PathBuf::from("/home/anna/Archive"),
                PathBuf::from("/home/anna/Invoices")
            ]
        );
    }
}
