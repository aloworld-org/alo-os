//! Whose grants these are: a file for each person, and the one move that gets a
//! machine there from the single file it used to keep.
//!
//! `docs/features.md` promises *Multi-user on one machine, with per-person
//! grants and no shared agent memory*. [`crate::THE_GRANTS`] is one literal
//! path with no person in it, and `keeping.rs`'s own header already says what
//! the file is for: *it is the person's own state … what an agent did on
//! somebody's machine is theirs and so is what they let it reach*. **One file
//! whose owner is checked is not one file per person**, and on a machine with
//! two accounts the second person to sign in met the first one's grants.
//!
//! # Why the person is a uid and not a name
//!
//! `believing.rs` believes a file that belongs to *root or to the login reading
//! it*, which it asks of the open file as a **uid**. Keyed by uid, the path and
//! that check compare the same number. The other per-person path on this
//! machine, `/var/lib/alo/undo/<name>/…`, keys by the name a person was given —
//! and `alo-shell`'s `settings_places.rs` records what that would cost here:
//! *a login with no folder … has no places for the person's own sections at
//! all. The grants and pairings are the machine's rather than the person's
//! folder's, so they are still read.* A person with no home directory still has
//! grants, and a path under their folder would take that away.
//!
//! # Why the name is flat and not a folder
//!
//! `grants-<uid>.toml` beside the machine's old file, rather than
//! `grants/<uid>.toml` under a new one. The image already makes
//! `/var/lib/alo` — `0700 alo alo` in `image/usr/lib/tmpfiles.d/alo.conf` — and
//! this crate **refuses a folder that is not there rather than making one**
//! (`believing.rs`). A new directory would need a line in the image, which is
//! another lane's file, for a name that buys nothing.
//!
//! # Why the move happens once and nothing falls back to the old file
//!
//! The obvious implementation is the bug this module exists to fix. *A person
//! with no file of their own reads the machine's* cannot be bounded by the
//! owner check, because `believing.rs` believes a file owned by **root** for
//! any reader, and the daemon that re-reads the grants file is root — so every
//! account would inherit the same grants.
//!
//! ADR 0001 §3 settles it beyond bounding: *a grant comes from a **deliberate
//! act** … there is no grant to `/`, and there is no grant that outlives the
//! reason it was made.* **An inherited grant is nobody's deliberate act**, and
//! it outlives the reason it was made the moment it answers for a second
//! person. So [`moved_to_whoever_had_them`] runs once, for the one person a
//! machine had, and afterwards a person with no file of their own has **no
//! grants** — which is the only answer that model permits.
//!
//! # And a machine that cannot tell says so where somebody will find it
//!
//! A machine with more than one account at the moment of the move has no safe
//! answer: the single file was written by one of them and nothing on disk says
//! which. Guessing hands one person's grants to another. So nothing is moved,
//! and [`could_not_tell_whose`] is written.
//!
//! **Its absence is the safe reading.** A machine that never had anything to
//! move has no marker, which is also a machine whose grants were moved
//! correctly — so nothing can be made to look refused by failing to write a
//! file. The refusal this module returns goes to a service log, which is gone
//! by the time anybody signs in; the marker is the same fact kept where
//! whatever shows a person their grants can still find it.

use std::path::{Path, PathBuf};

use crate::believing::{read_believed, replaced_whole};
use crate::refusing::NotRemembered;

/// The name the machine's single grants file had, inside the folder.
///
/// Named here rather than taken from [`crate::THE_GRANTS`] because that
/// constant is the whole path and this module is given the folder, so a test
/// can hand it a folder of its own.
const THE_MACHINES: &str = "grants.toml";

/// Where a person keeps their grants, inside the folder a machine keeps them in.
///
/// `grants-1000.toml` for the login whose uid is 1000. See this module's header
/// for why the uid rather than the name, and why a flat name rather than a
/// folder of its own.
#[must_use]
pub fn the_persons_grants(folder: &Path, uid: u32) -> PathBuf {
    folder.join(format!("grants-{uid}.toml"))
}

/// Where **this** process's person keeps their grants, on a real machine.
///
/// [`the_persons_grants`] with the folder the image makes and the uid **asked
/// of the kernel**, which is the whole reason it exists beside that one rather
/// than being assembled by each caller.
///
/// `crates/alo-desktop`'s own header states the rule this obeys: the three
/// facts a machine differs by arrive in its unit file, and **not the uid** —
/// *whose session this is, is whose session this is, and a desktop that took a
/// person's number from a variable could be started for the wrong one*. A
/// caller that built this path out of an `Environment=` line would be exactly
/// that mistake; a caller that reached for `geteuid` itself would be a second
/// place answering *whose machine is this*.
///
/// So the question is asked once, here, by the crate that owns the file.
#[must_use]
pub fn this_persons_grants() -> PathBuf {
    the_persons_grants(Path::new(crate::THE_FOLDER), this_person())
}

/// Whose process this is, asked of the kernel.
///
/// **The one place on a machine that answers it.** The paragraph above says
/// why — *a caller that reached for `geteuid` itself would be a second place
/// answering whose machine is this* — and that argument does not stop at this
/// crate's own files. Every per-person path on a machine is keyed by this
/// number, and two readings of it are two different people as far as a `0700`
/// directory is concerned.
///
/// So it is named and exported here, beside the first path that needed it,
/// rather than copied into each crate that wants a path with a person in it.
/// `alo_changing::this_persons_door` is the second caller and asks this.
#[must_use]
pub fn this_person() -> u32 {
    crate::believing::us()
}

/// Where a machine says it could not tell whose the old grants were.
///
/// Written only by [`moved_to_whoever_had_them`], and **absent on every machine
/// that had nothing to move or moved it correctly** — so its absence is the
/// safe reading and no failure to write it can invent a refusal.
#[must_use]
pub fn could_not_tell_whose(folder: &Path) -> PathBuf {
    folder.join("grants-not-moved")
}

/// What the one move did, which is four different machines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moved {
    /// There was no machine-wide file: a machine on its first morning, or one
    /// already moved and tidied.
    NothingToMove,

    /// The machine's grants are now this person's, and the old file is gone.
    ToThePerson {
        /// The login they belong to.
        uid: u32,
    },

    /// This person already has a file of their own, so nothing was touched.
    ///
    /// The move is run at every start and has to be safe to run twice. **A
    /// person's current grants are never replaced by an older machine-wide
    /// file**, which is why this answer exists rather than an overwrite.
    AlreadyTheirs {
        /// The login whose file was already there.
        uid: u32,
    },

    /// More than one account, so nothing was moved and the marker says so.
    CouldNotTellWhose {
        /// How many accounts the machine had when it was asked.
        how_many: usize,
    },
}

impl Moved {
    /// Whether a person reading an empty list is owed an explanation.
    ///
    /// Named rather than left to a caller matching four variants, because
    /// *should somebody be told* is the question whatever shows a person their
    /// grants has to ask, and getting it from the wrong variants is how a
    /// machine comes to show an empty list that is quietly wrong.
    #[must_use]
    pub const fn owes_somebody_an_explanation(self) -> bool {
        matches!(self, Self::CouldNotTellWhose { .. })
    }
}

/// Move a machine's single grants file to the one person who had it.
///
/// Run once per machine, and safe to run at every start: a person who already
/// has a file of their own is left alone, and a machine with nothing to move
/// says so rather than failing.
///
/// `accounts` is the logins this machine has, which this crate is **not** the
/// one to decide — it keeps a list and decides nothing about it, the same
/// reason there is no `Deserialize` for a grant here. The caller reads them and
/// hands them over.
///
/// **The bytes are not parsed.** The text is read believed and written whole,
/// so a file that said `format = 1` still says `format = 1` afterwards, byte
/// for byte. `written.rs` writes *the lowest format that holds what is
/// written*, and a rolled-back update has to still read what it wrote; a move
/// that parsed and re-serialised could quietly raise the format and break that.
///
/// # Errors
///
/// Everything `believing.rs`'s believed read refuses about the old file —
/// including [`NotRemembered::SomebodyElses`], so a file this machine would not
/// believe never becomes a person's grants — and everything its whole
/// replacement refuses about writing the new one, including a folder that is
/// not there.
/// [`NotRemembered::NotWritten`] if the old file cannot be removed once the new
/// one is down, and if the marker cannot be written, because a refusal nobody
/// can find is the failure this module exists to avoid.
pub fn moved_to_whoever_had_them(folder: &Path, accounts: &[u32]) -> Result<Moved, NotRemembered> {
    let the_machines = folder.join(THE_MACHINES);

    let [uid] = *accounts else {
        if !the_machines.exists() {
            return Ok(Moved::NothingToMove);
        }
        let how_many = accounts.len();
        let marker = could_not_tell_whose(folder);
        replaced_whole(&marker, &format!("{how_many}\n"))?;
        return Ok(Moved::CouldNotTellWhose { how_many });
    };

    let theirs = the_persons_grants(folder, uid);
    if theirs.exists() {
        return Ok(Moved::AlreadyTheirs { uid });
    }
    if !the_machines.exists() {
        return Ok(Moved::NothingToMove);
    }

    let text = read_believed(&the_machines)?;
    replaced_whole(&theirs, &text)?;
    std::fs::remove_file(&the_machines).map_err(|why| NotRemembered::NotWritten {
        at: the_machines.clone(),
        why: why.to_string(),
    })?;

    Ok(Moved::ToThePerson { uid })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::keeping::kept;
    use crate::testing::{a_folder_of_our_own, granted_in, noon};

    /// A machine that keeps grants the old way, in one file with no person in it.
    fn a_machine_with_one_file(folder: &Path) -> PathBuf {
        let at = folder.join(THE_MACHINES);
        kept(&at, &granted_in("/home/ada/Invoices"), noon()).unwrap();
        at
    }

    /// **The one person a machine had keeps their grants.** The constraint in
    /// the task is exact about this: whatever replaces the old file must leave a
    /// machine that had one person with *that person's* grants rather than with
    /// none.
    #[test]
    fn the_one_person_a_machine_had_keeps_what_they_granted() {
        let folder = a_folder_of_our_own("one-person");
        let old = a_machine_with_one_file(&folder);
        let before = std::fs::read_to_string(&old).unwrap();

        let moved = moved_to_whoever_had_them(&folder, &[1000]).unwrap();

        assert_eq!(moved, Moved::ToThePerson { uid: 1000 });
        let theirs = the_persons_grants(&folder, 1000);
        assert_eq!(
            std::fs::read_to_string(&theirs).unwrap(),
            before,
            "the move did not keep the bytes it was given"
        );
        assert!(!old.exists(), "the old path is still there to be read");
    }

    /// **Every byte survives, including the ones no writer here would emit.**
    ///
    /// This test exists in this shape because the obvious version of it could
    /// not fail. A fixture written by `kept` round-trips to identical bytes, so
    /// asserting the moved file equals it passed even when the move was
    /// replaced by a parse-and-reserialise — the implementation this module's
    /// header argues against, measured and still green.
    ///
    /// A comment and a blank line are the discriminator: `written.rs` emits
    /// neither, so a move that passes bytes through keeps them and a move that
    /// parses drops them. **The claim is only tested by a file whose writer
    /// would not produce it.**
    #[test]
    fn a_file_a_person_edited_by_hand_moves_unchanged() {
        let folder = a_folder_of_our_own("by-hand");
        let old = folder.join(THE_MACHINES);
        let by_hand = "# ada granted this before the machine had two people\n\
                       format = 1\n\
                       \n\
                       [[granted]]\n\
                       grantee = \"@files\"\n\
                       folder = \"/home/ada/Invoices\"\n\
                       ask = \"read\"\n";
        replaced_whole(&old, by_hand).unwrap();

        moved_to_whoever_had_them(&folder, &[1000]).unwrap();

        let theirs = std::fs::read_to_string(the_persons_grants(&folder, 1000)).unwrap();
        assert_eq!(
            theirs, by_hand,
            "the move did not pass the bytes through unchanged"
        );
        assert!(
            theirs.starts_with('#'),
            "the comment a person wrote was dropped"
        );
    }

    /// **A second account meets no grants it did not make**, which is the whole
    /// of why this module exists. Nothing falls back to the machine's old file.
    #[test]
    fn a_second_account_has_no_grants_of_its_own_to_find() {
        let folder = a_folder_of_our_own("second-account");
        a_machine_with_one_file(&folder);
        moved_to_whoever_had_them(&folder, &[1000]).unwrap();

        let theirs = the_persons_grants(&folder, 1001);
        assert!(
            !theirs.exists(),
            "a second login was given a grants file nobody made for it"
        );
    }

    /// **Running it twice does not replace a person's grants with an older
    /// file.** The move runs at every start, so this is the ordinary case
    /// rather than an edge one.
    #[test]
    fn a_person_who_already_has_a_file_is_left_alone() {
        let folder = a_folder_of_our_own("twice");
        a_machine_with_one_file(&folder);
        assert_eq!(
            moved_to_whoever_had_them(&folder, &[1000]).unwrap(),
            Moved::ToThePerson { uid: 1000 }
        );
        let theirs = the_persons_grants(&folder, 1000);
        let after_the_move = std::fs::read_to_string(&theirs).unwrap();

        // A machine that somehow has both files again: the person's is the one
        // that counts, and the old one is not read back over it.
        a_machine_with_one_file(&folder);
        assert_eq!(
            moved_to_whoever_had_them(&folder, &[1000]).unwrap(),
            Moved::AlreadyTheirs { uid: 1000 }
        );
        assert_eq!(
            std::fs::read_to_string(&theirs).unwrap(),
            after_the_move,
            "a second run wrote over what the person had"
        );
    }

    /// **A machine that cannot tell whose they are moves nothing and says so.**
    /// Guessing would hand one person's grants to another.
    #[test]
    fn two_accounts_means_nothing_is_moved_and_the_marker_is_written() {
        let folder = a_folder_of_our_own("two-accounts");
        let old = a_machine_with_one_file(&folder);

        let moved = moved_to_whoever_had_them(&folder, &[1000, 1001]).unwrap();

        assert_eq!(moved, Moved::CouldNotTellWhose { how_many: 2 });
        assert!(moved.owes_somebody_an_explanation());
        assert!(old.exists(), "the old file was moved after all");
        assert!(!the_persons_grants(&folder, 1000).exists());
        assert!(!the_persons_grants(&folder, 1001).exists());
        assert_eq!(
            std::fs::read_to_string(could_not_tell_whose(&folder)).unwrap(),
            "2\n"
        );
    }

    /// **A machine with nothing to move is not a refusal**, and leaves no
    /// marker — so the marker's absence is the safe reading.
    #[test]
    fn nothing_to_move_is_not_a_refusal_and_leaves_no_marker() {
        let folder = a_folder_of_our_own("nothing");

        for accounts in [vec![1000], vec![1000, 1001], vec![]] {
            let moved = moved_to_whoever_had_them(&folder, &accounts).unwrap();
            assert_eq!(moved, Moved::NothingToMove, "accounts: {accounts:?}");
            assert!(!moved.owes_somebody_an_explanation());
            assert!(
                !could_not_tell_whose(&folder).exists(),
                "a machine with nothing to move wrote a refusal"
            );
        }
    }

    /// **A file this machine would not believe never becomes a person's
    /// grants.** The old file is read through `believing.rs` rather than copied,
    /// so the three questions it asks are asked here too.
    #[test]
    fn a_file_somebody_else_could_write_is_refused_rather_than_moved() {
        use std::os::unix::fs::PermissionsExt;

        let folder = a_folder_of_our_own("writable");
        let old = a_machine_with_one_file(&folder);
        let mut mode = std::fs::metadata(&old).unwrap().permissions();
        mode.set_mode(0o666);
        std::fs::set_permissions(&old, mode).unwrap();

        let why = moved_to_whoever_had_them(&folder, &[1000]).unwrap_err();

        assert!(
            matches!(why, NotRemembered::WritableByOthers { .. }),
            "a world-writable file was moved: {why:?}"
        );
        assert!(
            !the_persons_grants(&folder, 1000).exists(),
            "it became a person's grants anyway"
        );
    }

    /// The name is the uid, and nothing else is in it.
    #[test]
    fn a_persons_file_is_named_by_their_uid() {
        let folder = Path::new("/var/lib/alo");
        assert_eq!(
            the_persons_grants(folder, 1000),
            Path::new("/var/lib/alo/grants-1000.toml")
        );
        assert_eq!(
            the_persons_grants(folder, 1001),
            Path::new("/var/lib/alo/grants-1001.toml")
        );
        assert_ne!(
            the_persons_grants(folder, 1000),
            the_persons_grants(folder, 1001)
        );
    }
}
