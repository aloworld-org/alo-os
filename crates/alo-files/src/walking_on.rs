//! Everything under a folder, gathered by as many walks as its size needs.
//!
//! [`MOST_WALKED`](crate::MOST_WALKED) is how many things one walk looks at,
//! so that a verb over a granted folder is bounded whatever the folder holds.
//! A person's photo library or a source tree with its dependencies is more
//! than that, and an index or a tree of sizes that stopped where one walk
//! stopped would be a search box that never finds the rest and a measurement
//! that stops where it was most wanted. So a caller that needs the whole
//! folder **walks on**: the first walk names, in
//! [`Walked::not_entered`](crate::Walked::not_entered), the folder it was in
//! and every folder it had found and not yet entered, and each of those is
//! walked in turn — the same walk, asked again from a folder it named, under
//! the same bound — until nothing is left unentered. Every step's `below` is
//! respelled from the folder the caller named, so a thing is said to be where
//! it is the way it is for a small folder.
//!
//! This is the one place in the repository that walks on, borrowed by
//! `alo-finding` for the index and by `alo-measuring` for the tree of sizes.
//! Two copies would be two opinions about which folder is walked again and
//! what is counted once, which is the reason those crates borrow one walker
//! instead of writing another. [`Walking::through`] is unchanged: the six
//! verbs of this crate make one walk, under one bound, exactly as before.
//!
//! # What still cannot be made whole
//!
//! The walk lists a folder, sorts its names and keeps the first `most` of
//! them; a folder holding more than `most` things **at one level** stops it
//! in the same place every time it is asked. Walking on from such a folder
//! would be the same walk again, so it is walked once more and, when that
//! walk stops inside it again, it is left in [`Gathered::not_entered`] and the
//! gathering is not [`Gathered::whole`]. What was found in it is kept — the
//! first `most` names, and everything under the folders among them — and the
//! caller says a folder holds more than one walk looks at. A folder the
//! machine would not let a later walk read is an [`Unread`] in the gathering
//! under the measuring policy, exactly as one the first walk stepped over,
//! and ends a searching walk-on exactly as it ends a searching walk.
//!
//! # A folder listed twice is counted once
//!
//! The folder a walk stopped inside was listed whole — the walk lists a
//! folder before it keeps any of it — and walking on from it lists it again.
//! The steps it had already kept are told apart by their paths and not kept
//! twice; the links and the things whose names could not be shown are only
//! counts, so the counts in that one folder are taken once more, with a walk
//! that keeps nothing, and subtracted. Every other folder walked on from was
//! found and never listed, and is counted once.
//!
//! # It finishes
//!
//! Every folder taken off the queue is either listed to its end, left not
//! entered, or replaced by folders strictly deeper than it; a link is never
//! followed, so there are no cycles; and nothing is walked that is not below
//! the folder the caller named.

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use alo_strings::{Strings, Vocabulary};

use crate::failed::Failed;
use crate::walking::{Step, Unread, Walking};
use crate::words::file_words;

/// Everything under a folder, however many walks it took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gathered {
    /// What is under the folder: each folder before the things inside it,
    /// each walk's steps after the walk before it, every `below` spelled from
    /// the folder the caller named.
    pub things: Vec<Step>,
    /// How many links were found, each counted once: stepped over by a
    /// searching walk, kept as steps by a measuring one.
    pub links: usize,
    /// How many things were left out because their names could not be shown,
    /// each counted once.
    pub could_not_be_named: usize,
    /// Whether every folder found was entered and listed to its end.
    pub whole: bool,
    /// Folders the machine would not let a walk read, each with what it
    /// said, spelled from the folder the caller named. Always empty for a
    /// searching walk-on, which fails instead.
    pub unread: Vec<Unread>,
    /// Folders on another filesystem, not entered.
    pub elsewhere: Vec<PathBuf>,
    /// Folders no walk can list to their end, because each holds more than
    /// one walk looks at, at one level. Empty when [`Self::whole`].
    pub not_entered: Vec<PathBuf>,
}

/// A folder a walk named and did not finish, waiting to be walked on from.
#[derive(Debug)]
struct Left {
    /// Where it is below the folder the caller named.
    below: PathBuf,
    /// Whether an earlier walk listed it and kept some of what it holds —
    /// true for the folder a walk stopped inside, false for one it had found
    /// and not yet entered.
    listed_before: bool,
}

impl Walking {
    /// Everything under this folder, walked on from every folder one walk
    /// left unentered until nothing is left, each walk under this policy and
    /// its bound.
    ///
    /// The folder itself is not in the answer, as with [`Self::through`].
    ///
    /// # Errors
    /// [`Failed`] when the folder itself is not there, is not a folder, or
    /// the machine would not read it — the first walk's own refusal. A folder
    /// found *inside* it that a later walk could not read is, for a measuring
    /// walk, an [`Unread`] in the gathering rather than an error, because the
    /// rest of the gathering is still true; for a searching walk it is the
    /// error, because a search that quietly leaves out a folder answers "it
    /// is not here" about something that may be.
    pub fn throughout(&self, folder: &Path) -> Result<Gathered, Failed> {
        let first = self.through(folder)?;
        let mut left: VecDeque<Left> = VecDeque::new();
        let mut gathered = Gathered {
            things: first.things,
            links: first.links,
            could_not_be_named: first.could_not_be_named,
            whole: true,
            unread: first.unread,
            elsewhere: first.elsewhere,
            not_entered: Vec::new(),
        };
        left_after(&mut left, &mut gathered, PathBuf::new(), first.not_entered);

        while let Some(Left {
            below: sub,
            listed_before,
        }) = left.pop_front()
        {
            let here = folder.join(&sub);
            let walked = match self.through(&here) {
                Ok(walked) => walked,
                Err(why) if self.notes_an_unreadable_folder() => {
                    gathered.unread.push(Unread {
                        below: sub,
                        why: what_the_machine_said(why),
                    });
                    continue;
                }
                Err(why) => return Err(why),
            };
            // This walk found again everything the earlier one had found
            // under `sub` and not entered, and either entered it or names it
            // below.
            left.retain(|waiting| !waiting.below.starts_with(&sub));
            let already = if listed_before {
                children_kept_of(&gathered.things, &sub)
            } else {
                HashSet::new()
            };
            for step in walked.things {
                let below = sub.join(&step.below);
                if already.contains(&below) {
                    continue;
                }
                gathered.things.push(Step { below, ..step });
            }
            gathered
                .unread
                .extend(walked.unread.into_iter().map(|unread| Unread {
                    below: sub.join(&unread.below),
                    why: unread.why,
                }));
            gathered
                .elsewhere
                .extend(walked.elsewhere.into_iter().map(|below| sub.join(below)));
            let (links_before, unnamed_before) = if listed_before {
                self.counted_in(&here)
            } else {
                (0, 0)
            };
            gathered.links += walked.links.saturating_sub(links_before);
            gathered.could_not_be_named += walked.could_not_be_named.saturating_sub(unnamed_before);
            left_after(&mut left, &mut gathered, sub, walked.not_entered);
        }

        gathered.whole = gathered.not_entered.is_empty();
        Ok(gathered)
    }

    /// How many links, and how many things whose names cannot be shown, are
    /// directly inside this folder: a walk that keeps nothing lists the
    /// folder and counts both, and stops before it enters anything. Zero
    /// when the folder cannot be listed, in which case the walk on from it
    /// said so already.
    fn counted_in(&self, folder: &Path) -> (usize, usize) {
        self.keeping_nothing()
            .through(folder)
            .map_or((0, 0), |walked| (walked.links, walked.could_not_be_named))
    }
}

/// What a walk from `sub` left unentered, put where it goes: the folder the
/// walk stopped inside is walked on from next, unless it is `sub` itself —
/// which the walk would stop inside again, so it stays not entered — and
/// every folder found and not entered is walked on from after it.
fn left_after(
    left: &mut VecDeque<Left>,
    gathered: &mut Gathered,
    sub: PathBuf,
    not_entered: Vec<PathBuf>,
) {
    let mut named = not_entered.into_iter();
    if let Some(stopped_inside) = named.next() {
        if stopped_inside.as_os_str().is_empty() {
            gathered.not_entered.push(sub.clone());
        } else {
            left.push_back(Left {
                below: sub.join(stopped_inside),
                listed_before: true,
            });
        }
    }
    left.extend(named.map(|found| Left {
        below: sub.join(found),
        listed_before: false,
    }));
}

/// Why a folder could not be walked on from, as the sentence an [`Unread`]
/// carries: the machine's own words when it said no, exactly as the walk
/// keeps them for a folder it steps over — and, for a folder gone between the
/// walk that found it and this one, this crate's own sentence for a thing
/// that is not there any more, in its source words, since a gathering has no
/// reader's language to hand.
fn what_the_machine_said(why: Failed) -> String {
    match why {
        Failed::TheMachineSaidNo { why, .. } => why,
        other => {
            let vocabulary = file_words().unwrap_or_else(|_| Vocabulary::empty());
            other.said(&Strings::of(vocabulary)).text().to_owned()
        }
    }
}

/// The steps already kept that are directly inside `sub`: the ones a walk
/// that stopped inside `sub` had kept before it stopped, which walking on
/// from `sub` finds again.
fn children_kept_of(things: &[Step], sub: &Path) -> HashSet<PathBuf> {
    things
        .iter()
        .filter(|step| step.below.parent() == Some(sub))
        .map(|step| step.below.clone())
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::fs;

    use super::*;
    use crate::named::Kind;
    use crate::testing::a_folder_of_our_own;

    /// `folders` folders named `f-NN`, each holding `files` files named
    /// `NN-MM.txt`, and inside each a folder `deeper` holding `deeper_files`
    /// more.
    fn a_tree(root: &Path, folders: usize, files: usize, deeper_files: usize) -> usize {
        for f in 0..folders {
            let folder = root.join(format!("f-{f:02}"));
            fs::create_dir_all(folder.join("deeper")).unwrap();
            for m in 0..files {
                fs::write(folder.join(format!("{f:02}-{m:02}.txt")), b"a word").unwrap();
            }
            for m in 0..deeper_files {
                fs::write(
                    folder.join("deeper").join(format!("d-{f:02}-{m:02}.txt")),
                    b"deeper",
                )
                .unwrap();
            }
        }
        folders * (2 + files + deeper_files)
    }

    /// Every `below`, spelled with `/`, sorted.
    fn belows(gathered: &Gathered) -> Vec<String> {
        let mut all: Vec<String> = gathered
            .things
            .iter()
            .map(|step| {
                step.below
                    .iter()
                    .map(|part| part.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect();
        all.sort();
        all
    }

    /// **A folder of more than one walk's bound is gathered whole**, every
    /// thing in it once, each `below` spelled from the folder named, and
    /// the gathering is the same as one walk with a bound the folder fits
    /// under — apart from the order the walks arrived in. The bounds tried
    /// start at the widest folder's width, six, because a folder wider than
    /// the bound is the one thing no walk can finish. Under both policies.
    #[test]
    fn a_folder_larger_than_one_walk_is_gathered_whole_and_nothing_twice() {
        let root = a_folder_of_our_own("walked-on-whole");
        let things = a_tree(&root, 6, 4, 3);
        assert_eq!(things, 54);

        for policy in [Walking::measuring, Walking::searching] {
            for bound in [6, 7, 9, 13, 53] {
                let gathered = policy(bound).throughout(&root).unwrap();
                assert!(gathered.whole, "bound {bound}: {gathered:?}");
                assert!(gathered.not_entered.is_empty(), "bound {bound}");
                assert!(gathered.unread.is_empty(), "bound {bound}");
                assert_eq!(gathered.could_not_be_named, 0, "bound {bound}");
                assert_eq!(gathered.links, 0, "bound {bound}");
                assert_eq!(gathered.things.len(), things, "bound {bound}");
                let in_one = policy(things).throughout(&root).unwrap();
                assert!(in_one.whole);
                assert_eq!(belows(&gathered), belows(&in_one), "bound {bound}");
                for step in &gathered.things {
                    assert_eq!(root.join(&step.below), step.at, "bound {bound}");
                }
            }
        }

        // Each folder is still before the things inside it.
        let gathered = Walking::measuring(6).throughout(&root).unwrap();
        let position = |below: &str| {
            gathered
                .things
                .iter()
                .position(|step| step.below == Path::new(below).iter().collect::<PathBuf>())
                .unwrap()
        };
        assert!(position("f-03") < position("f-03/deeper"));
        assert!(position("f-03/deeper") < position("f-03/deeper/d-03-02.txt"));
        assert!(position("f-03") < position("f-03/03-01.txt"));
        let _ = fs::remove_dir_all(&root);
    }

    /// **A folder holding more than one walk looks at, at one level, stays
    /// not entered** — walked on from once, and left when the walk stops
    /// inside it again rather than walked forever — and the gathering says it
    /// is not whole; what was found in it is kept, and everything beside it
    /// is gathered whole.
    #[test]
    fn a_folder_wider_than_the_bound_at_one_level_is_left_not_entered_and_said() {
        let root = a_folder_of_our_own("walked-on-wide");
        fs::create_dir_all(root.join("narrow").join("inner")).unwrap();
        fs::write(root.join("narrow").join("inner").join("n.txt"), b"n").unwrap();
        fs::create_dir_all(root.join("wide")).unwrap();
        for n in 0..9 {
            fs::write(root.join("wide").join(format!("w-{n}.txt")), b"w").unwrap();
        }
        fs::create_dir_all(root.join("wide").join("a-folder-first")).unwrap();
        fs::write(
            root.join("wide").join("a-folder-first").join("in.txt"),
            b"in",
        )
        .unwrap();

        let gathered = Walking::measuring(4).throughout(&root).unwrap();
        assert!(!gathered.whole);
        assert_eq!(gathered.not_entered, [PathBuf::from("wide")]);
        assert!(gathered.unread.is_empty());
        let found = belows(&gathered);
        assert!(
            found.contains(&"narrow/inner/n.txt".to_owned()),
            "{found:?}"
        );
        // The first four names of `wide`, sorted: the folder and three files,
        // and the folder was entered.
        assert!(
            found.contains(&"wide/a-folder-first".to_owned()),
            "{found:?}"
        );
        assert!(
            found.contains(&"wide/a-folder-first/in.txt".to_owned()),
            "{found:?}"
        );
        assert!(found.contains(&"wide/w-2.txt".to_owned()), "{found:?}");
        assert!(!found.contains(&"wide/w-3.txt".to_owned()), "{found:?}");
        let mut once = found.clone();
        once.dedup();
        assert_eq!(once, found, "nothing twice");
        // A folder left not entered holds exactly the bound's worth of
        // children, which is how a caller tells it from a folder that was
        // merely large.
        let kept_in_wide = gathered
            .things
            .iter()
            .filter(|step| step.below.parent() == Some(Path::new("wide")))
            .count();
        assert_eq!(kept_in_wide, 4);

        // The same folder walked with the root itself too wide for the bound:
        // the root stays not entered, and `wide`, which the walk never
        // reached, is not found at all — the one name the walk kept is
        // gathered whole.
        let gathered = Walking::measuring(1).throughout(&root).unwrap();
        assert!(!gathered.whole);
        assert_eq!(gathered.not_entered, [PathBuf::new()]);
        assert_eq!(
            belows(&gathered),
            ["narrow", "narrow/inner", "narrow/inner/n.txt"]
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// **A link found by a later walk is a link**: kept as a step and never
    /// followed by a measuring walk-on, stepped over by a searching one, and
    /// counted once by both however often its folder is listed.
    #[cfg(unix)]
    #[test]
    fn a_link_found_by_a_later_walk_is_counted_once_and_never_followed() {
        let root = a_folder_of_our_own("walked-on-links");
        let things = a_tree(&root, 3, 4, 2);
        assert_eq!(things, 24);
        let elsewhere = root.join("elsewhere");
        fs::create_dir_all(&elsewhere).unwrap();
        fs::write(elsewhere.join("secret.txt"), b"not counted").unwrap();
        std::os::unix::fs::symlink(&elsewhere, root.join("f-01").join("out")).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("secret.txt"), root.join("f-02").join("s"))
            .unwrap();
        // The folder the links point into, the file in it, and the two links.
        let things = things + 4;

        // The widest folder now holds six things, links included: the
        // bounds start there, because a folder wider than the bound is the
        // one thing no walk can finish.
        for bound in [6, 7, 9, 12, 40] {
            let measured = Walking::measuring(bound).throughout(&root).unwrap();
            assert!(measured.whole, "bound {bound}");
            assert_eq!(measured.links, 2, "bound {bound}: {measured:?}");
            let links: Vec<_> = measured
                .things
                .iter()
                .filter(|step| step.kind == Kind::Link)
                .collect();
            assert_eq!(links.len(), 2, "bound {bound}");
            assert!(
                !measured
                    .things
                    .iter()
                    .any(|step| step.below.starts_with(Path::new("f-01").join("out"))
                        && step.kind != Kind::Link),
                "nothing behind a link was walked: {measured:?}"
            );
            assert_eq!(measured.things.len(), things, "bound {bound}");

            let searched = Walking::searching(bound).throughout(&root).unwrap();
            assert!(searched.whole, "bound {bound}");
            assert_eq!(searched.links, 2, "bound {bound}: {searched:?}");
            assert!(
                searched.things.iter().all(|step| step.kind != Kind::Link),
                "a searching walk-on steps over links: {searched:?}"
            );
            assert_eq!(searched.things.len(), things - 2, "bound {bound}");
        }
        let _ = fs::remove_dir_all(&root);
    }

    /// **A folder a later walk could not read is an unread folder in the
    /// gathering** under the measuring policy, spelled from the folder named,
    /// and the gathering goes on — and it is the error under the searching
    /// policy, exactly as it would be for one walk. On Unix only, and only
    /// where this process is not root, which no permission stops.
    #[cfg(unix)]
    #[test]
    fn a_folder_a_later_walk_could_not_read_is_unread_by_a_measurement_and_fatal_to_a_search() {
        use std::os::unix::fs::PermissionsExt;

        let root = a_folder_of_our_own("walked-on-unread");
        for name in ["a", "b", "c-private"] {
            fs::create_dir_all(root.join(name)).unwrap();
            fs::write(root.join(name).join("one.txt"), b"one").unwrap();
            fs::write(root.join(name).join("two.txt"), b"two").unwrap();
        }
        fs::create_dir_all(root.join("c-private").join("inside")).unwrap();
        fs::set_permissions(root.join("c-private"), fs::Permissions::from_mode(0o000)).unwrap();
        let stopped = fs::read_dir(root.join("c-private")).is_err();

        // A bound of three: the first walk keeps a, b and c-private, and stops
        // inside the root; a, b and c-private are walked on from.
        let measured = Walking::measuring(3).throughout(&root).unwrap();
        let searched = Walking::searching(3).throughout(&root);
        fs::set_permissions(root.join("c-private"), fs::Permissions::from_mode(0o755)).unwrap();
        if stopped {
            assert_eq!(measured.unread.len(), 1, "{measured:?}");
            let unread = measured.unread.first().unwrap();
            assert_eq!(unread.below, PathBuf::from("c-private"));
            assert!(!unread.why.is_empty());
            assert!(measured.whole, "unread is not unentered");
            assert_eq!(measured.things.len(), 7, "{:?}", belows(&measured));
            let refused = searched.unwrap_err();
            assert!(
                matches!(refused, Failed::TheMachineSaidNo { .. }),
                "{refused:?}"
            );
        } else {
            assert!(measured.unread.is_empty(), "root reads anything");
            assert!(measured.whole);
            assert!(searched.unwrap().whole);
        }
        let _ = fs::remove_dir_all(&root);
    }

    /// **A folder gone between the walk that found it and the walk on from
    /// it** is an unread folder with this crate's own sentence for a thing
    /// that is not there, rather than a panic or a silent gap.
    #[test]
    fn a_folder_gone_before_it_is_walked_on_from_is_unread_in_words() {
        let gone = what_the_machine_said(Failed::Gone {
            path: "Documents/Later".to_owned(),
        });
        assert!(gone.contains("Documents/Later"), "{gone}");
        let said_no = what_the_machine_said(Failed::TheMachineSaidNo {
            path: "Documents/Theirs".to_owned(),
            doing: "listed".to_owned(),
            why: "permission denied".to_owned(),
        });
        assert_eq!(said_no, "permission denied");
    }

    /// **A name that cannot be shown is counted once**, though the folder
    /// holding it is listed by the walk that stopped inside it and again by
    /// the walk on from it. On Unix only, where such a name can be made.
    #[cfg(unix)]
    #[test]
    fn a_name_that_cannot_be_shown_is_counted_once_however_often_its_folder_is_listed() {
        let root = a_folder_of_our_own("walked-on-unnamed");
        let things = a_tree(&root, 3, 4, 2);
        assert_eq!(things, 24);
        fs::write(root.join("f-01").join("bell\u{7}.txt"), b"rings").unwrap();
        fs::write(root.join("f-01").join("deeper").join("tab\t.txt"), b"").unwrap();
        fs::write(root.join("f-02").join("nl\n.txt"), b"").unwrap();

        let in_one = Walking::measuring(things).throughout(&root).unwrap();
        assert!(in_one.whole);
        assert_eq!(in_one.could_not_be_named, 3);
        // The widest folder holds five things whose names can be shown.
        for bound in [5, 6, 8, 11] {
            let gathered = Walking::measuring(bound).throughout(&root).unwrap();
            assert!(gathered.whole, "bound {bound}");
            assert_eq!(gathered.things.len(), things, "bound {bound}");
            assert_eq!(
                gathered.could_not_be_named, 3,
                "bound {bound}: {gathered:?}"
            );
        }
        let _ = fs::remove_dir_all(&root);
    }

    /// **Nothing outside the folder named is walked, and the folder itself
    /// is refused the way one walk refuses it**: a folder that is not there,
    /// and a file, are the first walk's own error under both policies.
    #[test]
    fn the_folder_named_is_refused_the_way_one_walk_refuses_it() {
        let root = a_folder_of_our_own("walked-on-refused");
        fs::write(root.join("march.pdf"), b"an invoice").unwrap();
        for policy in [Walking::measuring, Walking::searching] {
            let gone = policy(3).throughout(&root.join("Taxes")).unwrap_err();
            assert!(matches!(gone, Failed::Gone { .. }), "{gone:?}");
            let not_a_folder = policy(3).throughout(&root.join("march.pdf")).unwrap_err();
            assert!(
                !matches!(not_a_folder, Failed::Gone { .. }),
                "{not_a_folder:?}"
            );
        }
        let _ = fs::remove_dir_all(&root);
    }

    /// The folder a walk stopped inside is walked on from first and marked as
    /// listed before; the folders it had found and not entered come after it
    /// and are not; a walk that stopped inside the folder it started from
    /// leaves that folder not entered rather than walking on from it.
    #[test]
    fn what_a_walk_left_is_queued_in_its_order_or_left_not_entered() {
        let mut left = VecDeque::new();
        let mut gathered = Gathered {
            things: Vec::new(),
            links: 0,
            could_not_be_named: 0,
            whole: true,
            unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: Vec::new(),
        };
        let queued = |left: &VecDeque<Left>| {
            left.iter()
                .map(|waiting| (waiting.below.clone(), waiting.listed_before))
                .collect::<Vec<_>>()
        };
        left_after(
            &mut left,
            &mut gathered,
            PathBuf::from("sub"),
            vec![PathBuf::from("inside"), PathBuf::from("found")],
        );
        assert_eq!(
            queued(&left),
            [
                (PathBuf::from("sub").join("inside"), true),
                (PathBuf::from("sub").join("found"), false),
            ]
        );
        assert!(gathered.not_entered.is_empty());

        left_after(
            &mut left,
            &mut gathered,
            PathBuf::from("wide"),
            vec![PathBuf::new(), PathBuf::from("child")],
        );
        assert_eq!(gathered.not_entered, [PathBuf::from("wide")]);
        assert_eq!(
            queued(&left),
            [
                (PathBuf::from("sub").join("inside"), true),
                (PathBuf::from("sub").join("found"), false),
                (PathBuf::from("wide").join("child"), false),
            ]
        );

        left_after(&mut left, &mut gathered, PathBuf::from("done"), Vec::new());
        assert_eq!(left.len(), 3);
        assert_eq!(gathered.not_entered.len(), 1);
    }
}
