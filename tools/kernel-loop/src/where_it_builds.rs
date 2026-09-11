//! Where the gates put what they build, and whether there is room where they
//! put it.
//!
//! # The defect this replaces
//!
//! The gates ran in the distribution and measured the host. `df` was asked
//! about `.` — the directory the checkout is in, which on this machine is a
//! Windows drive that is 98% full — while the build itself went to a directory
//! on the distribution's own filesystem, which had 868 GB free and held almost
//! nothing. So the reserve refused a run over a shortage that was nowhere near
//! the build, and on 2026-09-11 that refusal parked a task that was finished.
//! The answer people reached for was to delete a lane's build directory by
//! hand, which spends an hour of recompilation to free space that was never
//! scarce where the compiling happened.
//!
//! # What it does instead
//!
//! One build directory per checkout, on the filesystem the gates actually run
//! on, named from the checkout's own path so that two lanes on one machine can
//! never land on the same one. A shared `target` is a lock, and a lock is a
//! lane waiting.
//!
//! The reserve is then asked of **that** directory, and the refusal names the
//! filesystem it asked about — because a sentence about free space that does
//! not say *free where* is the sentence that cost the afternoon above.
//!
//! # Nothing here removes anything
//!
//! Not the directory it chooses, and above all not the ones from before it:
//! a build directory holding an afternoon of compilation may belong to a lane
//! that is merely idle, and a supervisor that tidied up would be a supervisor
//! that can throw work away. The old ones are **named** when a run starts, with
//! the command that says how much they hold, and whether they go is a person's
//! decision. `nothing_here_can_remove_a_build_directory` reads this file to
//! keep it that way.
//!
//! # And a machine where none of this is possible
//!
//! Falls back to what the gates did before — Cargo's own `target/` beside the
//! checkout, with the reserve measured on the checkout's filesystem — and says
//! so in a line. A supervisor that refused to gate because it could not make a
//! directory would have turned a convenience into a new way to stop.

use std::path::Path;
use std::sync::OnceLock;

use crate::gates;

/// The directory every checkout's own build directory sits under.
///
/// Not hidden, deliberately: a build directory nobody can find is one nobody
/// cleans, and the run says the whole path out loud anyway.
const ALL_OF_THEM: &str = "alo-builds";

/// The disk a build needs before it starts, in bytes.
///
/// Twelve gibibytes, unchanged. A full workspace build with the graphics crates
/// in it takes several, and a build that runs out partway does not fail as a
/// build: it fails as a linker that could not open a file, which reads like a
/// broken change and is not one. That happened on 2026-09-09 and cost an
/// afternoon of looking in the wrong place.
const THE_RESERVE: u64 = 12 * 1024 * 1024 * 1024;

/// Build directories this loop used before it chose one per checkout, relative
/// to the home the gates run in.
///
/// Named so that a person can find them, never touched. The first entry is
/// Cargo's own default beside the checkout, which is where a worker running
/// `cargo` by hand still builds.
const FROM_BEFORE: &[&str] = &["target-claude", "alo-os-target"];

/// Where this checkout's gates build, and the line that says so.
#[derive(Debug, PartialEq, Eq)]
pub struct BuildsIn {
    /// The directory, as the side that runs the gates names it — or [`None`]
    /// when none could be made and Cargo's own `target/` is what the gates use.
    pub directory: Option<String>,

    /// What a run says about it, once, when it starts.
    pub because: String,
}

impl BuildsIn {
    /// What the reserve is asked about: the build directory, or the checkout
    /// itself when there is none.
    pub fn measured(&self) -> &str {
        self.directory.as_deref().unwrap_or(".")
    }

    /// The same, as a sentence names it.
    pub fn named(&self) -> String {
        self.directory
            .clone()
            .unwrap_or_else(|| "this checkout's own `target/`".to_owned())
    }
}

/// Where this checkout builds, chosen once for the whole of a run.
///
/// Once, because the directory is made the first time it is asked for and every
/// gate after that has to be handed the same answer: a run that chose twice
/// could gate half a task in one directory and half in another, which is a full
/// rebuild in the middle of a task and two answers about the same tree.
pub fn chosen(at: &Path) -> &'static BuildsIn {
    static CHOSEN: OnceLock<BuildsIn> = OnceLock::new();
    CHOSEN.get_or_init(|| {
        let wanted = match where_it_goes(at) {
            Ok(wanted) => wanted,
            Err(why) => return the_choice(String::new(), Err(why)),
        };
        let made = made_it(at, &wanted);
        the_choice(wanted, made)
    })
}

/// That there is room to build where the build is going.
///
/// # Errors
/// A sentence naming the filesystem, how much is free on it, and the directory
/// the gates were going to build in. A reading that cannot be understood is a
/// refusal too: a reserve that passed whenever it could not measure anything
/// would not be a reserve.
pub fn there_is_room(at: &Path) -> Result<(), String> {
    let builds = chosen(at);
    let asked_about = builds.measured().to_owned();
    let said = gates::without_a_target_directory(
        at,
        ".",
        "df",
        &[
            "-B1".to_owned(),
            "--output=avail,target".to_owned(),
            asked_about.clone(),
        ],
    )?
    .output()
    .map_err(|why| format!("this machine could not be asked about {asked_about}: {why}"))?;
    if !said.status.success() {
        return Err(format!(
            "this machine could not say how much room there is in {asked_about}, so nothing was \
             published: {}",
            String::from_utf8_lossy(&said.stderr).trim()
        ));
    }
    room_enough(&String::from_utf8_lossy(&said.stdout), &builds.named())
}

/// The build directories from before this one, whichever of them are still
/// here.
///
/// It looks; it never removes. Each of these may hold an afternoon of
/// compilation belonging to a lane that is merely idle.
pub fn the_old_ones(at: &Path) -> Vec<String> {
    let in_use = chosen(at).directory.clone();
    let mut still_here = Vec::new();
    for candidate in every_old_one() {
        if Some(&candidate) == in_use.as_ref() {
            continue;
        }
        let Ok(mut asking) = gates::without_a_target_directory(
            at,
            ".",
            "test",
            &["-d".to_owned(), candidate.clone()],
        ) else {
            continue;
        };
        if asking.output().is_ok_and(|said| said.status.success()) {
            still_here.push(candidate);
        }
    }
    still_here
}

/// Every directory a build of this repository has gone to before now.
///
/// `target` first, because Cargo's default beside the checkout is the one that
/// grows without anybody choosing it.
fn every_old_one() -> Vec<String> {
    let mut named = vec!["target".to_owned()];
    if let Ok(home) = the_home() {
        named.extend(FROM_BEFORE.iter().map(|old| format!("{home}/{old}")));
    }
    named
}

/// The choice, made: what was wanted and whether it could be had.
///
/// Separate from having it so that *what a machine that cannot make the
/// directory does* is a thing this crate's own tests can state without a
/// filesystem.
fn the_choice(wanted: String, made: Result<(), String>) -> BuildsIn {
    let Err(why) = made else {
        return BuildsIn {
            because: format!(
                "the gates build in {wanted}, which is on the filesystem they run on rather than \
                 the one this checkout is on, and no other checkout builds there."
            ),
            directory: Some(wanted),
        };
    };
    BuildsIn {
        directory: None,
        because: format!(
            "the gates build in this checkout's own `target/`, as they did before, because a \
             build directory of their own could not be made: {why}. Nothing is refused for it, \
             and the room a build needs is measured on the filesystem the checkout is on instead."
        ),
    }
}

/// Make the directory, or say why it could not be.
///
/// # Errors
/// A sentence carrying what `mkdir` said, which is what the fallback line
/// repeats to whoever reads it.
fn made_it(at: &Path, directory: &str) -> Result<(), String> {
    let said = gates::without_a_target_directory(
        at,
        ".",
        "mkdir",
        &["-p".to_owned(), directory.to_owned()],
    )?
    .output()
    .map_err(|why| format!("`mkdir -p {directory}` could not be run: {why}"))?;
    if said.status.success() {
        return Ok(());
    }
    Err(format!(
        "`mkdir -p {directory}` refused: {}",
        String::from_utf8_lossy(&said.stderr).trim()
    ))
}

/// This checkout's own build directory, as the side that runs the gates names
/// it.
///
/// # Errors
/// A sentence when the home the gates run in cannot be named.
fn where_it_goes(at: &Path) -> Result<String, String> {
    Ok(named_for(&the_home()?, at))
}

/// The home directory the gates run in.
///
/// On Windows it is left as `$HOME` for the shell the bridge already runs
/// through: the home that matters is the distribution's, and this process
/// cannot see it. Every path built from it is free of spaces by construction —
/// the distribution's home has none and [`a_name_from`] allows none — which is
/// what makes it safe in a command line the bridge assembles unquoted.
///
/// # Errors
/// None here; the signature matches the Linux half so the caller has one shape.
#[cfg(windows)]
fn the_home() -> Result<String, String> {
    Ok("$HOME".to_owned())
}

/// The same, where the gates run without a bridge.
///
/// # Errors
/// A sentence when the environment names no home, which is what sends the
/// choice to its fallback rather than to a directory called `$HOME`.
#[cfg(not(windows))]
fn the_home() -> Result<String, String> {
    std::env::var("HOME")
        .map_err(|_| "this machine's environment names no HOME to build under".to_owned())
}

/// One checkout's build directory under a home.
///
/// The last part of the checkout's path makes it readable and a fingerprint of
/// the whole path makes it that checkout's alone, so two clones called
/// `alo-os-claude` in different places never meet. A checkout reached by two
/// different names — a drive letter on one host and a mount point on another —
/// is two directories, which costs a rebuild and never shares one.
fn named_for(home: &str, checkout: &Path) -> String {
    // A trailing separator is the same checkout, so it must not be a different
    // fingerprint: `C:\dev\alo-os-claude\` and `C:/dev/alo-os-claude` are one
    // directory and one build of it.
    let written = checkout.to_string_lossy().replace('\\', "/");
    let written = written.trim_end_matches('/');
    let readable = a_name_from(
        written
            .rsplit('/')
            .find(|part| !part.is_empty())
            .unwrap_or_default(),
    );
    format!(
        "{home}/{ALL_OF_THEM}/{readable}-{}",
        fingerprint(&written.to_lowercase())
    )
}

/// A path component as a directory name: lowercase, and nothing in it that a
/// command line would have to quote.
///
/// Runs of anything else become one `-` rather than one each, so that a name a
/// person chose reads as a name rather than as punctuation.
fn a_name_from(component: &str) -> String {
    let mut tidied = String::new();
    for letter in component.chars() {
        if letter.is_ascii_alphanumeric() {
            tidied.push(letter.to_ascii_lowercase());
        } else if !tidied.ends_with('-') {
            tidied.push('-');
        }
    }
    let tidied = tidied.trim_matches('-').to_owned();
    if tidied.is_empty() {
        "checkout".to_owned()
    } else {
        tidied
    }
}

/// A path, as sixteen hexadecimal characters.
///
/// FNV-1a, written out here because this is a supervisor that publishes and a
/// dependency added to shorten a name is a dependency in the thing that
/// publishes. It distinguishes; it is not asked to do anything else.
fn fingerprint(of: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in of.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// What `df` answered, read.
///
/// # Errors
/// A sentence whenever there is less than the reserve, or whenever the answer
/// could not be understood at all.
fn room_enough(said: &str, directory: &str) -> Result<(), String> {
    let Some((free, filesystem)) = what_df_said(said) else {
        return Err(format!(
            "this machine's answer about the room in {directory} could not be read, so nothing \
             was published. A reserve that passed whenever it could not measure anything would \
             not be a reserve. It said: {}",
            said.trim()
        ));
    };
    if free >= THE_RESERVE {
        return Ok(());
    }
    Err(format!(
        "there is less than 12 GiB free on `{filesystem}`, which is the filesystem the gates \
         build on — they build in {directory}, and `{filesystem}` has {} GiB free. A build needs \
         more than that. Nothing was staged, committed or pushed. A build started here would not \
         fail as a build — it fails as a linker that cannot open a file, which reads like a \
         broken change and is not one. **Do not delete anything shared to get past this.** \
         Caches, another worker's build directory and anything under a system folder belong to \
         whoever owns them; ask for space to be made.",
        free / (1024 * 1024 * 1024)
    ))
}

/// The last line of a `df --output=avail,target` that can be read as one, as
/// bytes free and the filesystem they are free on.
///
/// Read from the bottom so the header is never parsed, and a mount point with
/// spaces in it stays whole.
fn what_df_said(said: &str) -> Option<(u64, String)> {
    said.lines().rev().find_map(|line| {
        let mut words = line.split_whitespace();
        let free: u64 = words.next()?.parse().ok()?;
        let filesystem: Vec<&str> = words.collect();
        if filesystem.is_empty() {
            return None;
        }
        Some((free, filesystem.join(" ")))
    })
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected Ok is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Two checkouts never share a build directory**, however alike their
    /// names, because a shared `target` is a lock and a lock is a lane waiting.
    #[test]
    fn two_checkouts_never_build_in_the_same_directory() {
        let one = named_for("/root", Path::new("C:/dev/alo-os-claude"));
        let other = named_for("/root", Path::new("C:/dev2/alo-os-claude"));
        assert_ne!(one, other);
        assert!(one.starts_with("/root/alo-builds/alo-os-claude-"), "{one}");
        assert!(
            other.starts_with("/root/alo-builds/alo-os-claude-"),
            "{other}"
        );
    }

    /// And the same checkout is the same directory every time it is asked,
    /// whichever separator its path is written with — a run that chose a new
    /// name each time would rebuild the workspace from nothing on every gate.
    #[test]
    fn one_checkout_is_one_directory_however_it_is_written() {
        assert_eq!(
            named_for("$HOME", Path::new(r"C:\dev\alo-os-claude")),
            named_for("$HOME", Path::new("C:/dev/alo-os-claude")),
        );
        assert_eq!(
            named_for("$HOME", Path::new("C:/dev/alo-os-claude/")),
            named_for("$HOME", Path::new("C:/dev/alo-os-claude")),
        );
    }

    /// **Nothing in the name needs quoting**, because the bridge assembles its
    /// command line by joining words with spaces: a directory whose name
    /// carried one would become two arguments and build somewhere else.
    #[test]
    fn a_checkout_with_an_awkward_name_still_names_a_plain_directory() {
        let awkward = named_for("/home/one", Path::new("/srv/My Checkout (2)"));
        assert_eq!(
            &awkward[.."/home/one/alo-builds/".len()],
            "/home/one/alo-builds/"
        );
        let name = awkward.rsplit('/').next().unwrap_or_default();
        assert!(
            name.chars().all(|letter| letter.is_ascii_lowercase()
                || letter.is_ascii_digit()
                || letter == '-'),
            "{name} would have to be quoted"
        );
        assert!(name.starts_with("my-checkout-2-"), "{name}");
    }

    /// A path with no last part of its own still names a directory rather than
    /// producing one that ends in nothing.
    #[test]
    fn a_checkout_with_no_name_of_its_own_is_still_given_one() {
        let named = named_for("/root", Path::new("/"));
        assert!(named.starts_with("/root/alo-builds/checkout-"), "{named}");
    }

    /// **Room enough is a pass, and the sentence for too little names the
    /// filesystem it measured** — which is the whole of the defect: a refusal
    /// that says *there is no room* without saying *where* sent people to
    /// delete a build directory on a filesystem that was almost empty.
    #[test]
    fn too_little_room_says_which_filesystem_it_measured() {
        let plenty = "Avail Mounted on\n935951220736 /\n";
        assert_eq!(room_enough(plenty, "/root/alo-builds/x"), Ok(()));

        let barely = "Avail Mounted on\n4294967296 /mnt/c\n";
        let Err(why) = room_enough(barely, "/root/alo-builds/x") else {
            panic!("a filesystem with 4 GiB free was gated on")
        };
        assert!(why.contains("/mnt/c"), "{why}");
        assert!(why.contains("/root/alo-builds/x"), "{why}");
        assert!(why.contains("12 GiB"), "{why}");
        assert!(
            why.contains("nothing was published") || why.contains("Nothing was staged"),
            "{why}"
        );
    }

    /// Exactly the reserve is enough, and one byte less is not.
    #[test]
    fn the_reserve_is_a_floor_rather_than_a_target() {
        let at = format!("Avail Mounted on\n{THE_RESERVE} /\n");
        assert_eq!(room_enough(&at, "/root/alo-builds/x"), Ok(()));
        let under = format!("Avail Mounted on\n{} /\n", THE_RESERVE - 1);
        assert!(room_enough(&under, "/root/alo-builds/x").is_err());
    }

    /// **An answer that cannot be read is a refusal, not a pass.** A reserve
    /// that waved a run through whenever it failed to measure anything would be
    /// a reserve in name only, and the failure it would wave through is the one
    /// that fails as a linker rather than as a build.
    #[test]
    fn an_unreadable_answer_refuses() {
        for nonsense in [
            "",
            "Avail Mounted on\n",
            "df: /root/alo-builds/x: No such file or directory\n",
            "935951220736\n",
        ] {
            assert!(
                room_enough(nonsense, "/root/alo-builds/x").is_err(),
                "`{nonsense}` was read as room to build in"
            );
        }
    }

    /// **A machine that cannot make the directory falls back rather than
    /// failing**, and the line says which directory it is using and why.
    #[test]
    fn a_directory_that_could_not_be_made_falls_back_and_says_so() {
        let fallen = the_choice(
            "/root/alo-builds/x".to_owned(),
            Err("mkdir: cannot create directory: Read-only file system".to_owned()),
        );
        assert_eq!(fallen.directory, None);
        assert_eq!(
            fallen.measured(),
            ".",
            "the fallback must measure the filesystem it is actually building on"
        );
        assert!(
            fallen.because.contains("Read-only file system"),
            "{}",
            fallen.because
        );
        assert!(fallen.because.contains("target/"), "{}", fallen.because);
    }

    /// And a directory that was made is the one the gates build in, the one the
    /// reserve is measured on, and the one the line names.
    #[test]
    fn a_directory_that_was_made_is_the_one_everything_uses() {
        let made = the_choice("/root/alo-builds/x".to_owned(), Ok(()));
        assert_eq!(made.directory.as_deref(), Some("/root/alo-builds/x"));
        assert_eq!(made.measured(), "/root/alo-builds/x");
        assert_eq!(made.named(), "/root/alo-builds/x");
        assert!(
            made.because.contains("/root/alo-builds/x"),
            "{}",
            made.because
        );
    }

    /// **The build directories from before this one are named**, including the
    /// one Cargo makes beside the checkout without anybody choosing it.
    #[test]
    fn every_directory_from_before_is_one_a_person_can_be_pointed_at() {
        let named = every_old_one();
        assert!(named.contains(&"target".to_owned()), "{named:?}");
        assert!(
            named.len() > 1,
            "the directories this loop used before are not named: {named:?}"
        );
        for old in &named {
            assert!(!old.is_empty());
        }
    }

    /// **Nothing here can remove a build directory.**
    ///
    /// The constraint is not a preference: a directory holding an afternoon of
    /// compilation may belong to a lane that is merely idle, and a supervisor
    /// that tidied up would be one that can throw work away. Held on the source
    /// because a convenience added later would pass every behavioural test
    /// above on the day it was written.
    #[test]
    fn nothing_here_can_remove_a_build_directory() {
        let whole = include_str!("where_it_builds.rs");
        let source = whole.split("#[cfg(test)]").next().unwrap_or_default();
        assert!(
            source.contains("pub fn there_is_room"),
            "this test is no longer reading the choice of build directory"
        );
        for line in source
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
        {
            for forbidden in [
                "\"rm\"",
                "\"rmdir\"",
                "\"clean\"",
                "remove_dir",
                "remove_file",
                "Command::new",
            ] {
                assert!(
                    !line.contains(forbidden),
                    "something here can throw away somebody's compilation: {line}"
                );
            }
        }
    }
}
