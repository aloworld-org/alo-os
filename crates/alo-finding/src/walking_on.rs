//! Everything under a folder, gathered by as many walks as its size needs.
//!
//! `alo_files::MOST_WALKED` is how many things one walk looks at, so that a
//! verb over a granted folder is bounded whatever the folder holds. A
//! person's photo library or a source tree with its dependencies is more than
//! that, and an index that stopped where one walk stopped would be a search
//! box that never finds the rest. So the index **walks on**: the first walk
//! names, in `not_entered`, the folder it was in and every folder it had
//! found and not yet entered, and each of those is walked in turn — the same
//! walker, asked again from a folder it named, under the same bound — until
//! nothing is left unentered. Every step's `below` is respelled from the
//! folder the person named, so an entry says where a thing is the way it
//! does for a small folder.
//!
//! # What still cannot be made whole
//!
//! The walker lists a folder, sorts its names and keeps the first `most` of
//! them; a folder holding more than `most` things **at one level** stops it in
//! the same place every time it is asked. Walking on from such a folder would
//! be the same walk again, so it is walked once more and, when that walk stops
//! inside it again, it is left in [`Gathered::not_entered`] and the gathering
//! is not [`Gathered::whole`]. What was found in it is kept — the first `most`
//! names, and everything under the folders among them — and the sentence
//! above the index says a folder holds more than one walk looks at. A folder
//! the machine would not let a later walk read is an [`Unread`] in the
//! gathering, exactly as one the first walk stepped over, so that *nothing
//! matched* is never said about a folder nobody looked at.
//!
//! # One file walks
//!
//! This is the one file in the crate that names the walker, and
//! `tests/nothing_here_opens_a_socket_or_asks_anybody.rs` holds it to one:
//! walking on is that walker asked again from a folder it named, never a
//! second walker and never a wider bound, and nothing is walked that is not
//! below the folder the person named. A link is still never followed, and a
//! folder on another filesystem is still noted and not entered.
//!
//! # A folder listed twice is counted once
//!
//! The folder a walk stopped inside was listed whole — the walker lists a
//! folder before it keeps any of it — and walking on from it lists it again.
//! The steps it had already kept are told apart by their paths and not kept
//! twice; the things whose names could not be shown are only a count, so the
//! count of them in that one folder is taken once more, with a walk that keeps
//! nothing, and subtracted. Every other folder walked on from was found and
//! never listed, and is counted once.

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use alo_files::{Failed, Step, Unread, Walking, file_words};
use alo_strings::{Strings, Vocabulary};

/// Everything under a folder, however many walks it took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Gathered {
    /// What is under the folder: each folder before the things inside it,
    /// each walk's steps after the walk before it, every `below` spelled from
    /// the folder the person named.
    pub things: Vec<Step>,
    /// How many things were left out because their names could not be shown,
    /// each counted once.
    pub could_not_be_named: usize,
    /// Whether every folder found was entered and listed to its end.
    pub whole: bool,
    /// Folders the machine would not let a walk read, each with what it
    /// said, spelled from the folder the person named.
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
    /// Where it is below the folder the person named.
    below: PathBuf,
    /// Whether an earlier walk listed it and kept some of what it holds —
    /// true for the folder a walk stopped inside, false for one it had found
    /// and not yet entered.
    listed_before: bool,
}

/// Everything under `folder`, gathered by walks of at most `most` things
/// each, walked on from every folder a walk left unentered.
///
/// # Errors
///
/// [`Failed`] when the folder itself is not there, is a file, or could not
/// be read — the first walk's own refusal. A folder found *inside* it that a
/// later walk could not read is an [`Unread`] in the gathering, not an
/// error, because the rest of the gathering is still true.
pub(crate) fn everything_under(folder: &Path, most: usize) -> Result<Gathered, Failed> {
    let first = Walking::measuring(most).through(folder)?;
    let mut left: VecDeque<Left> = VecDeque::new();
    let mut gathered = Gathered {
        things: first.things,
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
        let walked = match Walking::measuring(most).through(&here) {
            Ok(walked) => walked,
            Err(why) => {
                gathered.unread.push(Unread {
                    below: sub,
                    why: what_the_machine_said(why),
                });
                continue;
            }
        };
        // This walk found again everything the earlier one had found under
        // `sub` and not entered, and either entered it or names it below.
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
        let counted_before = if listed_before { unnamed_in(&here) } else { 0 };
        gathered.could_not_be_named += walked.could_not_be_named.saturating_sub(counted_before);
        left_after(&mut left, &mut gathered, sub, walked.not_entered);
    }

    gathered.whole = gathered.not_entered.is_empty();
    Ok(gathered)
}

/// What a walk from `sub` left unentered, put where it goes: the folder the
/// walk stopped inside is walked on from next, unless it is `sub` itself —
/// which the walker would stop inside again, so it stays not entered — and
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
/// carries: the machine's own words when it said no, exactly as the walker
/// keeps them for a folder it steps over — and, for a folder gone between the
/// walk that found it and this one, `alo-files`' own sentence for a thing that
/// is not there any more, in its source words, since an index has no reader's
/// language to hand.
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

/// How many things directly inside this folder have names that cannot be
/// shown: a walk that keeps nothing lists the folder and counts them, and
/// stops before it enters anything. Zero when the folder cannot be listed,
/// in which case the walk on from it said so already.
fn unnamed_in(folder: &Path) -> usize {
    Walking::measuring(0)
        .through(folder)
        .map_or(0, |walked| walked.could_not_be_named)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::fs;

    use super::*;

    /// A folder of this test's own, under this machine's temporary directory.
    fn a_folder_of_our_own(what: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-finding-walking-on-{}-{what}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        folder.canonicalize().unwrap()
    }

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
    /// the bound is the one thing no walk can finish.
    #[test]
    fn a_folder_larger_than_one_walk_is_gathered_whole_and_nothing_twice() {
        let root = a_folder_of_our_own("whole");
        let things = a_tree(&root, 6, 4, 3);
        assert_eq!(things, 54);

        for bound in [6, 7, 9, 13, 53] {
            let gathered = everything_under(&root, bound).unwrap();
            assert!(gathered.whole, "bound {bound}: {gathered:?}");
            assert!(gathered.not_entered.is_empty(), "bound {bound}");
            assert!(gathered.unread.is_empty(), "bound {bound}");
            assert_eq!(gathered.could_not_be_named, 0, "bound {bound}");
            assert_eq!(gathered.things.len(), things, "bound {bound}");
            let in_one = everything_under(&root, things).unwrap();
            assert!(in_one.whole);
            assert_eq!(belows(&gathered), belows(&in_one), "bound {bound}");
            for step in &gathered.things {
                assert_eq!(root.join(&step.below), step.at, "bound {bound}");
            }
        }

        // Each folder is still before the things inside it.
        let gathered = everything_under(&root, 6).unwrap();
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
        let root = a_folder_of_our_own("wide");
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

        let gathered = everything_under(&root, 4).unwrap();
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

        // The same folder walked with the root itself too wide for the bound:
        // the root stays not entered, and `wide`, which the walk never
        // reached, is not found at all — the one name the walk kept is
        // gathered whole.
        let gathered = everything_under(&root, 1).unwrap();
        assert!(!gathered.whole);
        assert_eq!(gathered.not_entered, [PathBuf::new()]);
        assert_eq!(
            belows(&gathered),
            ["narrow", "narrow/inner", "narrow/inner/n.txt"]
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// **A folder a later walk could not read is an unread folder in the
    /// gathering**, spelled from the folder named, and the gathering goes on
    /// — and a folder that is gone by the time it is walked on from is the
    /// same. On Unix only, and only where this process is not root, which
    /// no permission stops.
    #[cfg(unix)]
    #[test]
    fn a_folder_a_later_walk_could_not_read_is_unread_and_the_rest_is_gathered() {
        use std::os::unix::fs::PermissionsExt;

        let root = a_folder_of_our_own("unread");
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
        let gathered = everything_under(&root, 3).unwrap();
        fs::set_permissions(root.join("c-private"), fs::Permissions::from_mode(0o755)).unwrap();
        if stopped {
            assert_eq!(gathered.unread.len(), 1, "{gathered:?}");
            let unread = gathered.unread.first().unwrap();
            assert_eq!(unread.below, PathBuf::from("c-private"));
            assert!(!unread.why.is_empty());
            assert!(gathered.whole, "unread is not unentered");
            assert_eq!(gathered.things.len(), 7, "{:?}", belows(&gathered));
        } else {
            assert!(gathered.unread.is_empty(), "root reads anything");
            assert!(gathered.whole);
        }
        let _ = fs::remove_dir_all(&root);
    }

    /// **A name that cannot be shown is counted once**, though the folder
    /// holding it is listed by the walk that stopped inside it and again by
    /// the walk on from it. On Unix only, where such a name can be made.
    #[cfg(unix)]
    #[test]
    fn a_name_that_cannot_be_shown_is_counted_once_however_often_its_folder_is_listed() {
        let root = a_folder_of_our_own("unnamed");
        let things = a_tree(&root, 3, 4, 2);
        assert_eq!(things, 24);
        fs::write(root.join("f-01").join("bell\u{7}.txt"), b"rings").unwrap();
        fs::write(root.join("f-01").join("deeper").join("tab\t.txt"), b"").unwrap();
        fs::write(root.join("f-02").join("nl\n.txt"), b"").unwrap();

        let in_one = everything_under(&root, things).unwrap();
        assert!(in_one.whole);
        assert_eq!(in_one.could_not_be_named, 3);
        // The widest folder holds five things whose names can be shown.
        for bound in [5, 6, 8, 11] {
            let gathered = everything_under(&root, bound).unwrap();
            assert!(gathered.whole, "bound {bound}");
            assert_eq!(gathered.things.len(), things, "bound {bound}");
            assert_eq!(
                gathered.could_not_be_named, 3,
                "bound {bound}: {gathered:?}"
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
