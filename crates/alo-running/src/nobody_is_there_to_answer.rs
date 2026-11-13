//! **A command alo runs is told, before it starts, that nobody is there to
//! answer a question** — so a program that would have asked for a password or
//! a yes stops at once and says so, rather than waiting until it is killed.
//!
//! Every command alo runs has no keyboard: its input is the empty device, it
//! has no terminal, and it is stopped after
//! [`crate::THE_LONGEST_A_COMMAND_RUNS`]. Most programs notice that and fail at
//! once. Some do not. `git` asks for a user name and password through a helper
//! program when it has no terminal; GitHub's own `gh` asks before it creates
//! anything; `ssh` hands its question to a graphical helper when the desktop is
//! there. A question nobody can see is two minutes of a frozen request and then
//! *the command ran longer than a command may* — which tells the person nothing
//! about what was actually wrong.
//!
//! So each of these programs is given the switch it already has for exactly
//! this, and the person is told the real reason (`alo_capability::ItDidNotWork`
//! reads *not signed in* out of what the command said).
//!
//! | variable | what it stops |
//! |---|---|
//! | `GIT_TERMINAL_PROMPT=0` | `git` asking for a user name or password |
//! | `GCM_INTERACTIVE=never` | Git Credential Manager opening a window to sign in, where a person installed it |
//! | `GH_PROMPT_DISABLED=1` | `gh` asking anything at all |
//! | `GH_NO_UPDATE_NOTIFIER=1` | `gh` going to github.com on its own to look for a newer `gh` — law 1: nothing leaves that the person did not ask for |
//! | `SSH_ASKPASS_REQUIRE=never` | `ssh` handing a passphrase or host-key question to a graphical helper |
//!
//! # Why not `ssh -o BatchMode=yes`
//!
//! The usual way to say this to `ssh` is `GIT_SSH_COMMAND="ssh -o
//! BatchMode=yes"`, and it is deliberately not set. That variable **replaces**
//! whatever `ssh` command the person configured for `git` (`core.sshCommand`),
//! so a person who pointed `git` at a key of their own would find alo's pushes
//! failing where their own succeed — a choice of theirs quietly overridden
//! (law 5). It is also not needed for what this file is for: with no terminal
//! and no graphical helper, `ssh` has nowhere to ask and fails at once
//! (*Host key verification failed*, or the key it could not unlock skipped),
//! which is what `BatchMode` would have made it do.
//!
//! # A variable the service was already started with is kept
//!
//! Each is set only where the environment does not already name it, so a
//! machine whose owner deliberately set one of these for the service keeps
//! their value.

use std::process::Command;

/// The variables, and the value each is set to.
pub const NOBODY_ANSWERS: [(&str, &str); 5] = [
    ("GIT_TERMINAL_PROMPT", "0"),
    ("GCM_INTERACTIVE", "never"),
    ("GH_PROMPT_DISABLED", "1"),
    ("GH_NO_UPDATE_NOTIFIER", "1"),
    ("SSH_ASKPASS_REQUIRE", "never"),
];

/// Tell the program `command` will start that nobody is there to answer.
pub(crate) fn nobody_is_there_to_answer(command: &mut Command) -> &mut Command {
    for (variable, value) in NOBODY_ANSWERS {
        if std::env::var_os(variable).is_none() {
            command.env(variable, value);
        }
    }
    command
}

#[cfg(test)]
mod tests {
    use super::NOBODY_ANSWERS;

    #[test]
    fn each_variable_is_named_once_and_none_replaces_a_persons_ssh_command() {
        let mut named: Vec<&str> = NOBODY_ANSWERS.iter().map(|(name, _)| *name).collect();
        named.sort_unstable();
        named.dedup();
        assert_eq!(named.len(), NOBODY_ANSWERS.len());
        for replaces_a_choice in ["GIT_SSH_COMMAND", "GIT_SSH", "GIT_ASKPASS", "SSH_ASKPASS"] {
            assert!(
                !named.contains(&replaces_a_choice),
                "{replaces_a_choice} would replace a program the person chose"
            );
        }
    }
}
