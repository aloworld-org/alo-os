//! **A folder published through real `git`, on the road a machine runs it on**
//! — `alo_running::run_as_the_person_around`, `/bin/sh -c`, nobody there to answer —
//! to a repository on this disk, with a stand-in for GitHub's `gh` that answers
//! the four questions the line asks it, and the one asked before it: who is
//! signed in.
//!
//! What this proves, every build: one publication is one new version on the
//! site and nothing else; nothing is written into the person's folder; a folder
//! that is itself a `git` repository is published without its `.git`; the same
//! folder twice sends nothing new; a changed folder is published on top of the
//! last version, never over it; and nobody signed in, or the wrong person
//! signed in, publishes nothing and makes no repository.
//!
//! What it cannot prove is GitHub: that `gh` signs in, that github.com takes
//! the push, and that Pages serves the address. That is a walk on a booted
//! machine (`docs/autonomy/demo-two-alo-inside-applications-plan.md`).

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use alo_capability::{ACommand, AroundACommand, ItDidNotStart, NothingAround, WhatACommandDid};
use alo_publishing::{
    Account, NotPublished, Publication, Published, Runs, Through, publish, who_is_signed_in,
};
use alo_strings::{Strings, Vocabulary};

/// What a turn puts around a command, counting how often it started one — a
/// stand-in for the control group that counts it, which needs a kernel.
#[derive(Default)]
struct Watching {
    started: usize,
}

impl AroundACommand for Watching {
    fn starting(
        &mut self,
        start: &mut dyn FnMut() -> std::io::Result<std::process::Child>,
    ) -> std::io::Result<std::process::Child> {
        self.started += 1;
        NothingAround.starting(start)
    }

    fn looking(&mut self) {}
}

/// The road a machine runs a publication on.
struct AsThePerson;

impl Runs for AsThePerson {
    fn run(
        &self,
        line: &ACommand,
        around: &mut dyn AroundACommand,
    ) -> Result<WhatACommandDid, ItDidNotStart> {
        alo_running::run_as_the_person_around(line, Path::new("/"), around)
    }
}

/// A stand-in for `gh`: who is signed in is the file `who`, nobody is when
/// `signed-out` is there, and a repository it makes is a bare one under
/// `remotes`. Anything else it is asked fails, so a question the line starts
/// asking is noticed here.
const A_STAND_IN_FOR_GH: &str = r#"#!/bin/sh
here="$(dirname "$0")/.."
case "$1 $2" in
  "api user")
    if [ -f "$here/signed-out" ]; then
      echo 'To get started with GitHub CLI, please run:  gh auth login' >&2; exit 4
    fi
    cat "$here/who"; exit 0;;
  "config get")
    # `gh`'s own record of who signed in: nothing when nobody has, or when
    # `no-config` makes the line fall back to asking GitHub.
    if [ -f "$here/signed-out" ] || [ -f "$here/no-config" ]; then exit 1; fi
    cat "$here/who"; exit 0;;
  "repo view")
    [ -d "$here/remotes/$3.git" ] || exit 1
    case "$*" in *defaultBranchRef*) echo main;; esac
    exit 0;;
  "repo create")
    git init -q --bare "$here/remotes/$3.git" && echo "$3" >> "$here/made" && exit 0
    exit 1;;
  "api --method")
    echo "$*" >> "$here/pages"; exit 0;;
esac
echo "the stand-in for gh was asked something it does not know: $*" >&2
exit 9
"#;

/// Somewhere of the test's own: the stand-in, the remotes, and the folder.
struct Here {
    _kept: tempfile::TempDir,
    root: PathBuf,
    folder: PathBuf,
}

impl Here {
    fn signed_in_as(who: &str) -> Self {
        let kept = tempfile::tempdir().unwrap();
        let root = kept.path().to_owned();
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::create_dir_all(root.join("remotes/anna")).unwrap();
        let gh = root.join("bin/gh");
        fs::write(&gh, A_STAND_IN_FOR_GH).unwrap();
        fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(root.join("who"), format!("{who}\n")).unwrap();
        let folder = root.join("Anna's site");
        fs::create_dir_all(folder.join("pages")).unwrap();
        fs::write(folder.join("index.html"), "<h1>Hello</h1>\n").unwrap();
        fs::write(folder.join("pages/about.html"), "<p>About</p>\n").unwrap();
        Self {
            _kept: kept,
            root,
            folder,
        }
    }

    fn through(&self) -> Through {
        Through::at(
            &self.root.join("bin/gh").display().to_string(),
            &format!("file://{}/remotes", self.root.display()),
        )
    }

    fn the_site(&self) -> PathBuf {
        self.root.join("remotes/anna/anna.github.io.git")
    }

    fn publishing(&self) -> Result<Published, NotPublished> {
        let strings = in_english();
        let publication = Publication::of(&self.folder, Account::named("anna").unwrap());
        let mut watching = Watching::default();
        let done = publish(
            &publication,
            &self.through(),
            &AsThePerson,
            &mut watching,
            &strings,
        );
        // **Started through what counts it**, so on a machine the command is
        // born inside the control group the indicator's count is read from.
        assert_eq!(
            watching.started, 1,
            "the command was not started through what counts it"
        );
        done
    }

    /// Asked of the site's repository with real `git`.
    fn git(&self, arguments: &[&str]) -> String {
        let out = Command::new("git")
            .arg(format!("--git-dir={}", self.the_site().display()))
            .args(arguments)
            .output()
            .expect("git is on this machine");
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    }

    fn versions(&self) -> usize {
        self.git(&["rev-list", "--count", "refs/heads/main"])
            .parse()
            .unwrap_or(0)
    }

    fn what_is_on_the_site(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .git(&["ls-tree", "-r", "--name-only", "refs/heads/main"])
            .lines()
            .map(str::to_owned)
            .collect();
        files.sort();
        files
    }
}

fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_publishing::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// Every file and folder under `folder`, so a test can show nothing was added.
fn everything_in(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut waiting = vec![folder.to_owned()];
    while let Some(next) = waiting.pop() {
        for entry in fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                waiting.push(path.clone());
            }
            found.push(path);
        }
    }
    found.sort();
    found
}

/// **One publication is one new version, made in the account's name, and
/// nothing is written into the person's folder** — then the same folder again
/// sends nothing new, and a changed one goes on top of the last version.
#[test]
fn one_publication_is_one_version_and_the_folder_is_left_as_it_was() {
    let here = Here::signed_in_as("Anna");
    let before = everything_in(&here.folder);

    let published = here.publishing().unwrap();
    assert!(published.made_the_repository(), "{published:?}");
    assert!(!published.nothing_new(), "{published:?}");
    assert_eq!(published.address(), "https://anna.github.io/");
    assert_eq!(here.versions(), 1, "one publication was not one version");
    assert_eq!(
        here.what_is_on_the_site(),
        ["index.html", "pages/about.html"]
    );
    assert_eq!(
        here.git(&["log", "-1", "--format=%an <%ae> %s", "refs/heads/main"]),
        "Anna <Anna@users.noreply.github.com> Published from alo OS"
    );
    assert_eq!(
        everything_in(&here.folder),
        before,
        "publishing wrote into the person's folder"
    );
    assert!(
        fs::read_to_string(here.root.join("pages"))
            .unwrap()
            .contains("source[branch]=main"),
        "GitHub Pages was not asked to serve it"
    );

    let again = here.publishing().unwrap();
    assert!(
        again.nothing_new(),
        "the same folder was sent again: {again:?}"
    );
    assert!(!again.made_the_repository());
    assert_eq!(here.versions(), 1, "nothing new still made a version");

    let first = here.git(&["rev-parse", "refs/heads/main"]);
    fs::write(here.folder.join("index.html"), "<h1>Hello again</h1>\n").unwrap();
    fs::remove_file(here.folder.join("pages/about.html")).unwrap();
    let changed = here.publishing().unwrap();
    assert!(!changed.nothing_new());
    assert_eq!(here.versions(), 2);
    assert_eq!(
        here.git(&["rev-parse", "refs/heads/main~1"]),
        first,
        "the new version is not on top of the last one"
    );
    assert_eq!(
        here.what_is_on_the_site(),
        ["index.html"],
        "a file taken out of the folder is still on the site"
    );
}

/// **A folder that is itself a `git` repository is published without its
/// `.git`**, and its own history is not touched.
#[test]
fn a_folder_that_is_a_repository_is_published_without_its_own_git() {
    let here = Here::signed_in_as("anna");
    let own = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&here.folder)
        .status()
        .unwrap();
    assert!(own.success());
    let before = everything_in(&here.folder);

    here.publishing().unwrap();

    assert_eq!(
        here.what_is_on_the_site(),
        ["index.html", "pages/about.html"],
        "the folder's own .git was published"
    );
    assert_eq!(everything_in(&here.folder), before);
}

/// **The refusal path: nobody signed in publishes nothing, makes no
/// repository, and says how to sign in.**
#[test]
fn nobody_signed_in_publishes_nothing_and_makes_no_repository() {
    let here = Here::signed_in_as("anna");
    fs::write(here.root.join("signed-out"), "").unwrap();

    assert_eq!(here.publishing(), Err(NotPublished::NotSignedIn));
    assert!(!here.the_site().exists(), "a repository was made anyway");
    assert!(
        NotPublished::NotSignedIn
            .said(&{
                let mut vocabulary = Vocabulary::empty();
                alo_publishing::words::declare_into(&mut vocabulary).unwrap();
                alo_capability::words::declare_into(&mut vocabulary).unwrap();
                Strings::of(vocabulary)
            })
            .text()
            .contains("gh auth login --web")
    );
}

/// **The refusal path: signed in as somebody else publishes nothing**, because
/// it would have gone to a site the sentence did not name.
#[test]
fn somebody_else_signed_in_publishes_nothing() {
    let here = Here::signed_in_as("bob");

    assert_eq!(
        here.publishing(),
        Err(NotPublished::SignedInAsAnother {
            signed_in: "bob".to_owned(),
            account: "anna".to_owned()
        })
    );
    assert!(!here.the_site().exists(), "a repository was made anyway");
}

/// Who `gh` says is signed in, asked through the real shell.
fn who(here: &Here) -> Result<Account, NotPublished> {
    let mut watching = Watching::default();
    let who = who_is_signed_in(&here.through(), &AsThePerson, &mut watching);
    assert_eq!(
        watching.started, 1,
        "asking who is signed in was not started through what counts it"
    );
    who
}

/// **The account comes from `gh`, not from a model**: `gh`'s own record
/// first, GitHub where that is empty, and nobody signed in is said as that —
/// the refusal that raises the sign-in sheet.
#[test]
fn the_account_is_the_one_gh_is_signed_in_as() {
    let here = Here::signed_in_as("Anna");
    assert_eq!(who(&here), Ok(Account::named("anna").unwrap()));

    fs::write(here.root.join("no-config"), "").unwrap();
    assert_eq!(
        who(&here),
        Ok(Account::named("anna").unwrap()),
        "an empty configuration did not fall back to asking GitHub"
    );

    fs::write(here.root.join("signed-out"), "").unwrap();
    assert_eq!(who(&here), Err(NotPublished::NotSignedIn));
}
