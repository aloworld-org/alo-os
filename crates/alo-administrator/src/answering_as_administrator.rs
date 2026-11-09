//! **The line `alo-agentd` and `alo-running-as-administrator` speak**: one
//! command asked for, and either what it did or why it did not run.
//!
//! The agent service sends the command as one line — the same line
//! `alo_capability::ACommand` holds, with its leading `sudo` already taken off
//! — and the runner answers with one line of JSON, [`Answered`]. Nothing here
//! carries a password: the runner asks polkit, polkit asks the person through
//! their desktop's sheet, and what comes back to the agent service is whether
//! they said yes.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use alo_capability::WhatACommandDid;
use serde::{Deserialize, Serialize};

/// Where the runner listens.
pub const THE_RUNNERS_SOCKET: &str = "/run/alo-running-as-administrator.socket";

/// The most the runner waits for the person to answer the password sheet
/// before it takes the question back: five minutes, long enough to read what
/// is being asked and short enough that a question nobody saw does not hold
/// the agent service for the rest of the day.
pub const THE_LONGEST_THE_PERSON_IS_WAITED_FOR: Duration = Duration::from_secs(300);

/// The most the agent service waits for the runner: the person, the command's
/// own bound, and a minute over both for everything between.
pub const THE_LONGEST_THE_SERVICE_WAITS: Duration = Duration::from_secs(300 + 120 + 60);

/// The most bytes of answer read back: what the command said, bounded as
/// `alo_running` bounds it, as JSON.
const THE_LONGEST_ANSWER: u64 = 8 * 64 * 1024;

/// **Why a command was not run as administrator.** Every one is something the
/// person can be told plainly, and none is a fallback: nothing here ever runs
/// the command as the person instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "kebab-case")]
pub enum NotAsAdministrator {
    /// The person closed the password sheet.
    #[error("the person closed the password window")]
    TheyCancelled,
    /// The person was asked and polkit did not accept what was given.
    #[error("the password was not accepted")]
    NotAccepted,
    /// There is no desktop signed in that could ask: no session, or no
    /// password sheet in it.
    #[error("nothing on this machine can ask for the password")]
    NothingCanAsk,
    /// The person's account is not an administrator here.
    #[error("the account is not an administrator")]
    NotAnAdministrator,
    /// Nobody answered the sheet in time, so the question was taken back.
    #[error("nobody answered in time")]
    NoAnswerInTime,
    /// Somebody other than the agent service asked.
    #[error("only the agent service may ask")]
    NotTheAgentService,
    /// The line was not one command.
    #[error("that was not one command")]
    NotACommand,
    /// polkit could not be asked at all.
    #[error("polkit could not be asked")]
    PolkitDidNotAnswer,
    /// The person said yes and the shell would not start.
    #[error("the command would not start")]
    DidNotStart,
    /// The runner could not be reached, or its answer would not read.
    #[error("the runner did not answer")]
    TheRunnerDidNotAnswer,
}

/// What the runner answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Answered {
    /// The person gave their password, and the command ran as administrator.
    Ran {
        /// Its exit code, when it ended with one.
        returned: Option<i32>,
        /// What it wrote, bounded.
        said: String,
        /// Whether what it wrote was cut short.
        cut_short: bool,
        /// Whether it was stopped for running too long.
        took_too_long: bool,
    },
    /// It did not run.
    Refused(NotAsAdministrator),
}

impl Answered {
    /// What a command that ran did, in the shape the record is written in.
    #[must_use]
    pub fn of(did: &WhatACommandDid) -> Self {
        Self::Ran {
            returned: did.returned(),
            said: did.said().to_owned(),
            cut_short: did.cut_short(),
            took_too_long: did.took_too_long(),
        }
    }

    /// Back into what the command did, or why it did not run.
    ///
    /// # Errors
    /// The refusal it carries.
    pub fn into_what_it_did(self) -> Result<WhatACommandDid, NotAsAdministrator> {
        match self {
            Self::Ran {
                returned,
                said,
                cut_short,
                took_too_long,
            } => Ok(WhatACommandDid::of(returned, said, cut_short, took_too_long)),
            Self::Refused(why) => Err(why),
        }
    }

    /// The one line it is sent as.
    #[must_use]
    pub fn as_line(&self) -> String {
        let mut line = serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"refused":"the-runner-did-not-answer"}"#.to_owned()
        });
        line.push('\n');
        line
    }
}

/// **The agent service's side**: ask the runner at `socket` to run `line` as
/// administrator, and wait for its answer.
///
/// # Errors
/// [`NotAsAdministrator::TheRunnerDidNotAnswer`] when it could not be reached,
/// said nothing in [`THE_LONGEST_THE_SERVICE_WAITS`], or said something that
/// is not an answer; otherwise whatever the runner refused for.
pub fn asked_of_the_runner(socket: &Path, line: &str) -> Result<WhatACommandDid, NotAsAdministrator> {
    let unanswered = |_| NotAsAdministrator::TheRunnerDidNotAnswer;
    let mut stream = UnixStream::connect(socket).map_err(unanswered)?;
    stream
        .set_read_timeout(Some(THE_LONGEST_THE_SERVICE_WAITS))
        .map_err(unanswered)?;
    stream
        .write_all(line.as_bytes())
        .and_then(|()| stream.write_all(b"\n"))
        .and_then(|()| stream.flush())
        .map_err(unanswered)?;
    let mut answer = String::new();
    BufReader::new(stream.take(THE_LONGEST_ANSWER))
        .read_line(&mut answer)
        .map_err(unanswered)?;
    serde_json::from_str::<Answered>(answer.trim_end())
        .map_err(|_| NotAsAdministrator::TheRunnerDidNotAnswer)?
        .into_what_it_did()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::os::unix::net::UnixListener;

    use super::*;

    /// **Every answer is one line and reads back as itself.**
    #[test]
    fn every_answer_is_one_line_and_reads_back() {
        let ran = Answered::of(&WhatACommandDid::of(Some(0), "done\nand more\n".to_owned(), false, false));
        let refused = Answered::Refused(NotAsAdministrator::TheyCancelled);
        for answer in [ran, refused] {
            let line = answer.as_line();
            assert_eq!(line.matches('\n').count(), 1, "{line}");
            assert_eq!(serde_json::from_str::<Answered>(line.trim_end()).unwrap(), answer);
        }
        assert_eq!(
            Answered::Refused(NotAsAdministrator::NothingCanAsk).as_line(),
            "{\"refused\":\"nothing-can-ask\"}\n"
        );
    }

    /// **The service sends the line and believes the runner's answer**, a
    /// refusal included.
    #[test]
    fn the_service_sends_the_line_and_reads_the_answer() {
        let place = tempfile::tempdir().unwrap();
        let socket = place.path().join("runner.socket");
        let listener = UnixListener::bind(&socket).unwrap();
        let runner = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut asked = String::new();
            BufReader::new(stream.try_clone().unwrap()).read_line(&mut asked).unwrap();
            (&stream)
                .write_all(Answered::Refused(NotAsAdministrator::NotAccepted).as_line().as_bytes())
                .unwrap();
            asked
        });
        assert_eq!(
            asked_of_the_runner(&socket, "dnf install -y htop"),
            Err(NotAsAdministrator::NotAccepted)
        );
        assert_eq!(runner.join().unwrap(), "dnf install -y htop\n");
    }

    /// **No runner is said, never taken for a run.**
    #[test]
    fn no_runner_is_said() {
        let place = tempfile::tempdir().unwrap();
        assert_eq!(
            asked_of_the_runner(&place.path().join("none.socket"), "true"),
            Err(NotAsAdministrator::TheRunnerDidNotAnswer)
        );
    }
}
