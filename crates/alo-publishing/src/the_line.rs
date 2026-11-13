//! **The one command a publication runs**, written by the machine — never by a
//! model — out of a checked folder and a checked account.
//!
//! It is `git` and GitHub's own `gh`, in this order, each step ending the run
//! with a number of its own when it fails, so [`crate::what_came_of_it()`] can
//! tell the person which step it was:
//!
//! | step | ends with |
//! |---|---|
//! | `git` and `gh` are on this computer | 70, 71 |
//! | `gh api user`: who is signed in | 72 |
//! | that is the account the sentence named | 73, after printing who it is |
//! | the site's repository is there, or `gh repo create` makes it, public | 75 |
//! | somewhere of its own to prepare it (`mktemp -d`), and `git init` there | 76 |
//! | the site as it is now, fetched, so the new version follows it | 77 |
//! | every file in the folder, added — and there are some | 77, 79 |
//! | already exactly this? then nothing is sent | 0, printing `alo-nothing-new` |
//! | one commit, and one `git push` — never forced | 77, 78 |
//! | GitHub Pages switched on for it, where it was not | (ignored: on already is the usual answer) |
//!
//! # Nothing is written into the person's folder
//!
//! `git` is pointed at the folder as its working tree (`GIT_WORK_TREE`) and at
//! a temporary place of alo's own for everything else (`GIT_DIR`), so no
//! `.git` and no index ever appears in the folder, and the temporary place is
//! removed however the run ends. **Which is why the sentence says Put back
//! cannot recall it rather than offering to**: nothing on this computer
//! changes, and what changed on GitHub is not this computer's to put back.
//!
//! # A push is never forced
//!
//! The new version is made on top of the site as it was fetched a moment
//! before, so the push only ever adds to the site's history. If the site was
//! changed somewhere else in between, GitHub refuses it (78) and nothing is
//! lost on either side. No `--force`, no `+` before a branch: a test holds the
//! line to it.
//!
//! # The sign-in it uses is `gh`'s, and only for github.com
//!
//! `git` is told, for `https://github.com` only, to ask `gh auth
//! git-credential` and no other helper — so the account `gh` showed in step 3
//! is the account that pushes. A helper the person set for any other host is
//! untouched. Commits are made in the account's own name, at the private
//! address GitHub gives every account (`<name>@users.noreply.github.com`), so
//! no e-mail address of the person's is published with their site.
//!
//! # Every argument is quoted
//!
//! The folder is the one thing here a person named, and it reaches the shell
//! inside single quotes with any single quote in it closed and reopened — the
//! shell's own rule for carrying any text whole. The account cannot need it
//! ([`crate::Account`] admits letters, numbers and hyphens only) and is quoted
//! anyway.

use alo_capability::ACommand;

use crate::not_published::NotPublished;
use crate::publication::Publication;
use crate::runs::Through;

/// What the run prints when the site already held exactly this folder.
pub const NOTHING_NEW: &str = "alo-nothing-new";

/// What the run prints when it made the site's repository.
pub const MADE_THE_REPOSITORY: &str = "alo-made-the-repository";

/// What the run prints, before who, when the wrong account is signed in.
pub const SIGNED_IN_AS: &str = "alo-signed-in-as:";

/// What the run prints when it published.
pub const PUBLISHED: &str = "alo-published";

/// The command that publishes `publication` through `through`, its commit
/// saying `message`.
///
/// # Errors
/// [`NotPublished::TheNameCannotBeCarried`] when the folder's name has a line
/// break in it, or makes the command longer than a command may be.
pub fn the_line(
    publication: &Publication,
    through: &Through,
    message: &str,
) -> Result<ACommand, NotPublished> {
    let gh = quoted(through.gh());
    let account = quoted(publication.account().name());
    let repository = quoted(&publication.account().the_sites_repository());
    let remote = quoted(&format!(
        "{}/{}.git",
        through.github(),
        publication.account().the_sites_repository()
    ));
    let folder = quoted(&publication.folder().display().to_string());
    let helper = quoted(&format!("!{} auth git-credential", through.gh()));
    let message = quoted(message);
    let steps = [
        "command -v git >/dev/null 2>&1 || exit 70;".to_owned(),
        format!("command -v {gh} >/dev/null 2>&1 || exit 71;"),
        format!("l=$({gh} api user --jq .login) || exit 72;"),
        format!(
            "[ \"$(printf %s \"$l\" | tr A-Z a-z)\" = {account} ] || \
             {{ printf '{SIGNED_IN_AS}%s\\n' \"$l\"; exit 73; }};"
        ),
        format!("r={repository}; u={remote};"),
        format!(
            "if ! {gh} repo view \"$r\" --json name >/dev/null 2>&1; then \
             {gh} repo create \"$r\" --public >/dev/null || exit 75; \
             echo {MADE_THE_REPOSITORY}; fi;"
        ),
        format!(
            "b=$({gh} repo view \"$r\" --json defaultBranchRef --jq .defaultBranchRef.name \
             2>/dev/null); [ -n \"$b\" ] || b=main;"
        ),
        "t=$(mktemp -d) || exit 76; trap 'rm -rf \"$t\"' EXIT;".to_owned(),
        format!(
            "export GIT_DIR=\"$t/g\" GIT_WORK_TREE={folder} GIT_CONFIG_COUNT=2 \
             GIT_CONFIG_KEY_0=credential.https://github.com.helper GIT_CONFIG_VALUE_0= \
             GIT_CONFIG_KEY_1=credential.https://github.com.helper GIT_CONFIG_VALUE_1={helper} \
             GIT_AUTHOR_NAME=\"$l\" GIT_AUTHOR_EMAIL=\"$l@users.noreply.github.com\" \
             GIT_COMMITTER_NAME=\"$l\" GIT_COMMITTER_EMAIL=\"$l@users.noreply.github.com\";"
        ),
        "git init -q || exit 76;".to_owned(),
        "if git fetch -q --depth=1 \"$u\" \"refs/heads/$b\" 2>/dev/null; then \
         git update-ref HEAD FETCH_HEAD || exit 77; fi;"
            .to_owned(),
        "git add -A || exit 77;".to_owned(),
        "[ -n \"$(git ls-files | head -n 1)\" ] || exit 79;".to_owned(),
        format!(
            "if git rev-parse -q --verify HEAD >/dev/null && git diff --cached --quiet HEAD; \
             then echo {NOTHING_NEW}; exit 0; fi;"
        ),
        format!("git commit -q -m {message} || exit 77;"),
        "git push -q \"$u\" \"HEAD:refs/heads/$b\" || exit 78;".to_owned(),
        format!(
            "{gh} api --method POST \"repos/$r/pages\" -f \"source[branch]=$b\" \
             -f 'source[path]=/' >/dev/null 2>&1 || true;"
        ),
        format!("echo {PUBLISHED}"),
    ];
    ACommand::of(&steps.join(" ")).map_err(|_| NotPublished::TheNameCannotBeCarried)
}

/// `text`, carried whole through the shell: inside single quotes, with each
/// single quote in it closed, escaped and reopened.
#[must_use]
pub fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::account::Account;
    use std::path::Path;

    fn anna(folder: &str) -> Publication {
        Publication::of(Path::new(folder), Account::named("anna").unwrap())
    }

    #[test]
    fn a_quote_in_a_folders_name_is_carried_whole() {
        assert_eq!(quoted("/home/anna/site"), "'/home/anna/site'");
        assert_eq!(
            quoted("/home/anna/Anna's site"),
            r"'/home/anna/Anna'\''s site'"
        );
        assert_eq!(quoted("$(reboot)"), "'$(reboot)'");
    }

    /// **The line is one command that names the folder, the account's site and
    /// github.com, and nothing in it forces a push.**
    #[test]
    fn the_line_names_what_it_publishes_and_never_forces() {
        let line = the_line(
            &anna("/home/anna/Anna's site"),
            &Through::github(),
            "Published from alo OS",
        )
        .unwrap();
        let text = line.text();
        assert!(
            text.contains(r"GIT_WORK_TREE='/home/anna/Anna'\''s site'"),
            "{text}"
        );
        assert!(text.contains("r='anna/anna.github.io'"), "{text}");
        assert!(
            text.contains("u='https://github.com/anna/anna.github.io.git'"),
            "{text}"
        );
        assert!(text.contains("-m 'Published from alo OS'"), "{text}");
        for forcing in [
            "--force", "-f HEAD", "+HEAD", " push -f", "--mirror", "--delete",
        ] {
            assert!(!text.contains(forcing), "{forcing:?} is in {text}");
        }
        assert_eq!(text.matches("git push").count(), 1, "{text}");
        assert!(
            text.len() < alo_capability::THE_LONGEST_COMMAND,
            "{}",
            text.len()
        );
    }

    /// **The refusal path**: a folder whose name cannot be carried in one line
    /// is refused before anything runs.
    #[test]
    fn a_folder_with_a_line_break_in_its_name_is_refused_before_anything_runs() {
        assert_eq!(
            the_line(&anna("/home/anna/site\nreboot"), &Through::github(), "m"),
            Err(NotPublished::TheNameCannotBeCarried)
        );
        let long = format!("/home/anna/{}", "a".repeat(5000));
        assert_eq!(
            the_line(&anna(&long), &Through::github(), "m"),
            Err(NotPublished::TheNameCannotBeCarried)
        );
    }
}
