//! **Whether a command failed because nobody is signed in to GitHub**, read
//! from what the command said.
//!
//! A `git push` with no sign-in, or GitHub's own `gh` before anybody ran
//! `gh auth login`, ends with a status like any other failure — and *the
//! command ended with exit status 128* tells a person nothing they can act on.
//! What they need is *you are not signed in to GitHub, and this is how to sign
//! in*. So [`crate::ItDidNotWork`] asks this, and says that instead.
//!
//! # Read from the tools' own words, and only GitHub's
//!
//! Each marker below is a line `git`, `ssh` or `gh` prints, exactly as it
//! prints it (compared without case). A marker that only says *authentication
//! failed* is held to an address on `github.com`, so a sign-in refused by
//! another host is not misreported as GitHub's — that would send the person to
//! sign in somewhere that was never the problem. Anything not on this list is
//! still a failure, said with its status as before; nothing is guessed.
//!
//! It never decides whether a run worked — only *why* one that did not, did
//! not.

/// Lines that say, on their own, that `gh` has nobody signed in.
const GH_HAS_NOBODY: [&str; 2] = [
    // `gh` before anybody signed in: *To get started with GitHub CLI, please
    // run: gh auth login*, and *You are not logged into any GitHub hosts. To
    // log in, run: gh auth login*.
    "gh auth login",
    // `gh` with a sign-in GitHub no longer accepts: *HTTP 401: Bad credentials
    // (https://api.github.com/...)*.
    "http 401: bad credentials",
];

/// Lines that say a sign-in was missing or refused — GitHub's only when they
/// name `github.com` as well.
const A_SIGN_IN_WAS_REFUSED: [&str; 4] = [
    // `git` over `https` with nobody there to type a name: *fatal: could not
    // read Username for 'https://github.com': terminal prompts disabled*.
    "could not read username",
    "terminal prompts disabled",
    // `git` over `https` with a sign-in GitHub refused: *fatal: Authentication
    // failed for 'https://github.com/...'*.
    "authentication failed",
    // `git` over `ssh` with no key GitHub knows: *git@github.com: Permission
    // denied (publickey).*
    "permission denied (publickey)",
];

/// Whether `said`, everything a command that failed wrote, says it failed
/// because nobody is signed in to GitHub.
#[must_use]
pub fn not_signed_in_to_github(said: &str) -> bool {
    let said = said.to_lowercase();
    GH_HAS_NOBODY.iter().any(|line| said.contains(line))
        || (said.contains("github.com")
            && A_SIGN_IN_WAS_REFUSED.iter().any(|line| said.contains(line)))
}

#[cfg(test)]
mod tests {
    use super::not_signed_in_to_github;

    /// **Each tool's own words for *nobody is signed in* are read as that**,
    /// copied as the tools print them.
    #[test]
    fn what_git_ssh_and_gh_say_with_nobody_signed_in_is_read_as_that() {
        for said in [
            "fatal: could not read Username for 'https://github.com': terminal prompts disabled\n",
            "remote: Invalid username or password.\nfatal: Authentication failed for \
             'https://github.com/anna/anna.github.io.git/'\n",
            "git@github.com: Permission denied (publickey).\nfatal: Could not read from remote \
             repository.\n",
            "To get started with GitHub CLI, please run:  gh auth login\nAlternatively, populate \
             the GH_TOKEN environment variable with a GitHub API authentication token.\n",
            "You are not logged into any GitHub hosts. To log in, run: gh auth login\n",
            "HTTP 401: Bad credentials (https://api.github.com/user)\n",
        ] {
            assert!(not_signed_in_to_github(said), "{said:?}");
        }
    }

    /// **The refusal path: another host's refusal, and any other failure, is
    /// not GitHub's sign-in** — the person is not sent to sign in somewhere
    /// that was never the problem.
    #[test]
    fn another_hosts_refusal_and_any_other_failure_is_not_a_github_sign_in() {
        for said in [
            "fatal: Authentication failed for 'https://gitlab.com/anna/site.git/'\n",
            "fatal: could not read Username for 'https://codeberg.org': terminal prompts \
             disabled\n",
            "git@gitlab.com: Permission denied (publickey).\n",
            "fatal: unable to access 'https://github.com/anna/site.git/': Could not resolve \
             host: github.com\n",
            "mkdir: cannot create directory '/nope/demo': No such file or directory\n",
            "! [rejected]        main -> main (fetch first)\nerror: failed to push some refs to \
             'https://github.com/anna/anna.github.io.git'\n",
            "",
        ] {
            assert!(!not_signed_in_to_github(said), "{said:?}");
        }
    }
}
