//! **A command alo writes as `sudo …` is run as administrator — after the
//! person's password, every time — or refused in words**
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! This service runs as the person with no capability and no new privileges,
//! so `sudo` and `pkexec` can do nothing inside it, on purpose. A command whose
//! line *starts* with either is therefore not run here at all: what follows
//! the word is handed to `alo-running-as-administrator`, which asks polkit,
//! which asks the person on their own screen through the desktop's password
//! sheet. The line the person approved and the record names is the whole line,
//! `sudo` included, because that is what was asked for.
//!
//! **Only the first word decides.** `cd /tmp && sudo make install` is one
//! command run as the person, in which `sudo` fails as it would at a prompt
//! with no terminal — and the model is told so by what the command says, and
//! told by its instructions to put `sudo` first (`alo_instructing`).
//!
//! **A refusal is never a fallback.** Every way the runner says no becomes a
//! sentence the person reads and a stamp in the record, and nothing is ever run
//! as the person instead.

use std::path::Path;

use alo_administrator::answering_as_administrator::{NotAsAdministrator, asked_of_the_runner};
use alo_capability::{ACommand, WhatACommandDid, words};
use alo_record::AsAnAdministrator;
use alo_strings::{Filling, Said, Strings};

/// The words a command starts with that ask for administrator rights.
const THE_WORDS_THAT_ASK: [&str; 2] = ["sudo", "pkexec"];

/// **What should run as administrator**, when `command` asks for it: the line
/// after its leading `sudo` or `pkexec`. [`None`] for every other command,
/// and for one that is that word and nothing after it.
#[must_use]
pub fn the_administrators_part(command: &ACommand) -> Option<&str> {
    let line = command.text().trim_start();
    THE_WORDS_THAT_ASK.iter().find_map(|word| {
        line.strip_prefix(word)
            .filter(|rest| rest.starts_with([' ', '\t']))
            .map(str::trim)
            .filter(|rest| !rest.is_empty())
    })
}

/// **Ask the runner at `socket` to run `command` as administrator**, and say
/// what came of it in the person's language.
///
/// # Errors
/// The sentence the person is told, and what they did, for the record.
pub fn through_the_runner(
    socket: &Path,
    command: &ACommand,
    strings: &Strings,
) -> Result<WhatACommandDid, (Said, AsAnAdministrator)> {
    let Some(part) = the_administrators_part(command) else {
        return Err(said(NotAsAdministrator::NotACommand, strings));
    };
    asked_of_the_runner(socket, part).map_err(|why| said(why, strings))
}

/// What a refusal says, and what the record keeps of it.
#[must_use]
pub fn said(why: NotAsAdministrator, strings: &Strings) -> (Said, AsAnAdministrator) {
    let (word, how) = match why {
        NotAsAdministrator::TheyCancelled => {
            (words::NOTHING_RAN_THEY_CANCELLED, AsAnAdministrator::TheyCancelled)
        }
        NotAsAdministrator::NotAccepted => {
            (words::NOTHING_RAN_NOT_ACCEPTED, AsAnAdministrator::NotAccepted)
        }
        NotAsAdministrator::NothingCanAsk => {
            (words::NOTHING_RAN_NOTHING_CAN_ASK, AsAnAdministrator::NothingCouldAsk)
        }
        NotAsAdministrator::NotAnAdministrator => (
            words::NOTHING_RAN_NOT_AN_ADMINISTRATOR,
            AsAnAdministrator::NotAnAdministrator,
        ),
        NotAsAdministrator::NoAnswerInTime => (
            words::NOTHING_RAN_NO_ANSWER_IN_TIME,
            AsAnAdministrator::NoAnswerInTime,
        ),
        NotAsAdministrator::DidNotStart => {
            (words::NOTHING_RAN_WOULD_NOT_START, AsAnAdministrator::CouldNotAsk)
        }
        NotAsAdministrator::NotTheAgentService
        | NotAsAdministrator::NotACommand
        | NotAsAdministrator::PolkitDidNotAnswer
        | NotAsAdministrator::TheRunnerDidNotAnswer => (
            words::NOTHING_RAN_AS_ADMINISTRATOR_COULD_NOT_ASK,
            AsAnAdministrator::CouldNotAsk,
        ),
    };
    eprintln!("alo-agentd: a command was not run as administrator: {why}");
    (strings.say(&word.key(), &Filling::nothing()), how)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The command in `line`.
    fn command(line: &str) -> ACommand {
        ACommand::of(line).unwrap()
    }

    /// **Only a line that starts with `sudo` or `pkexec` asks**, and what runs
    /// as administrator is what follows the word.
    #[test]
    fn only_a_line_that_starts_with_the_word_asks() {
        assert_eq!(
            the_administrators_part(&command("sudo dnf install -y htop")),
            Some("dnf install -y htop")
        );
        assert_eq!(
            the_administrators_part(&command("  pkexec  systemctl restart sshd")),
            Some("systemctl restart sshd")
        );
        for line in [
            "dnf install -y htop",
            "cd /tmp && sudo make install",
            "sudo",
            "sudo   ",
            "sudoku --solve",
            "echo sudo",
        ] {
            assert_eq!(the_administrators_part(&command(line)), None, "{line:?}");
        }
    }

    /// **Every refusal says something and keeps what the person did**, and
    /// only a refusal that was theirs says it was theirs.
    #[test]
    fn every_refusal_says_something() {
        let strings = crate::testing::in_english();
        for why in [
            NotAsAdministrator::TheyCancelled,
            NotAsAdministrator::NotAccepted,
            NotAsAdministrator::NothingCanAsk,
            NotAsAdministrator::NotAnAdministrator,
            NotAsAdministrator::NoAnswerInTime,
            NotAsAdministrator::NotTheAgentService,
            NotAsAdministrator::NotACommand,
            NotAsAdministrator::PolkitDidNotAnswer,
            NotAsAdministrator::DidNotStart,
            NotAsAdministrator::TheRunnerDidNotAnswer,
        ] {
            let (sentence, how) = said(why, &strings);
            assert!(sentence.text().starts_with("nothing ran"), "{why:?}: {}", sentence.text());
            assert!(!how.they_authenticated(), "{why:?}");
        }
        assert_eq!(
            said(NotAsAdministrator::TheyCancelled, &strings).1,
            AsAnAdministrator::TheyCancelled
        );
    }

    /// **No runner is a refusal in words**, never a run as the person.
    #[test]
    fn no_runner_is_a_refusal_in_words() {
        let strings = crate::testing::in_english();
        let place = tempfile::tempdir().unwrap();
        let came = through_the_runner(
            &place.path().join("nobody.socket"),
            &command("sudo true"),
            &strings,
        );
        let (sentence, how) = came.unwrap_err();
        assert_eq!(how, AsAnAdministrator::CouldNotAsk);
        assert!(sentence.text().contains("could not ask"), "{}", sentence.text());
    }
}
