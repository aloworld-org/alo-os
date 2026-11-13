//! **Signing in to GitHub, and out, with GitHub's own command, `gh`** — the
//! owner's approval of 2026-11-13 of Settings → Accounts and the sign-in
//! sheet (`docs/design/github-sign-in.md`).
//!
//! Four asks arrive from the shell (`alo_shell::AskingGitHub`):
//!
//! - **who is signed in** — `gh config get -h github.com user`, which reads
//!   gh's own settings and does not reach GitHub;
//! - **sign in** — `gh auth login --web --hostname github.com --git-protocol
//!   https`. gh prints a one-time code and the page it is typed on; the code
//!   is handed to the shell, which opens the page in the person's default
//!   browser. **alo fills nothing in**: the person types the code at GitHub,
//!   and the password, if GitHub asks for one, on GitHub's own page;
//! - **stop signing in** — the waiting `gh` is ended;
//! - **sign out** — `gh auth logout --hostname github.com`.
//!
//! # Nothing waits for a question
//!
//! gh is started with no input, no colour and no prompts, so it never sits
//! waiting for an answer nobody can give. A sign-in that has printed no code
//! after `NO_CODE_WITHIN` is ended and said as *no code arrived*.
//!
//! # Every sign-in, sign-out and failure is recorded
//!
//! Each is handed to `keep`, which the desktop points at the daemon's door
//! (`alo_changing::TheDaemonsDoor::keep_a_github_sign_in`). Reading who is
//! signed in changes nothing and is not recorded.
//!
//! # Off the drawing thread, and in order
//!
//! A sign-in waits minutes for the person. Every ask goes to **one** worker
//! thread, in the order it was made, and answers come back on a channel the
//! shell asks once a frame — `crate::starting_the_screen_reader`'s shape.

use std::io::{BufRead as _, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use alo_changing::{AtGitHub, WhyNotSignedIn};
use alo_shell::{AskingGitHub, GitHubAnswered, WhyGitHubDidNotWork};

use crate::what_gh_says;

/// How long a sign-in may go without printing its code before it is ended.
pub const NO_CODE_WITHIN: Duration = Duration::from_secs(30);

/// How often a waiting sign-in looks for a cancel and for gh having ended.
const BETWEEN_LOOKS: Duration = Duration::from_millis(100);

/// Where a sign-in, sign-out or failure is handed to be recorded.
pub type Keep = Box<dyn FnMut(&AtGitHub) + Send>;

/// GitHub's sign-in as this desktop runs it: one worker, and its answers.
pub struct TheGitHubSignIn {
    /// Where an ask goes, where there is a worker.
    to_the_worker: Option<Sender<AskingGitHub>>,
    /// Where its answers come back.
    answers: Receiver<GitHubAnswered>,
    /// Answers to asks nothing could carry out, given here rather than lost.
    unanswered: Vec<GitHubAnswered>,
}

impl TheGitHubSignIn {
    /// A worker running `gh` — the program at `gh`, found on `PATH` when it is
    /// a bare name — handing every sign-in, sign-out and failure to `keep`.
    ///
    /// **A machine that cannot start a thread answers every ask** with gh's
    /// failure, rather than taking it and doing nothing.
    #[must_use]
    pub fn running(gh: PathBuf, keep: Keep) -> Self {
        Self::waiting_for_a_code(gh, keep, NO_CODE_WITHIN)
    }

    /// [`Self::running`], ending a sign-in that has printed no code after
    /// `no_code_within` — shorter in a test than on a machine.
    #[must_use]
    pub fn waiting_for_a_code(gh: PathBuf, keep: Keep, no_code_within: Duration) -> Self {
        let (to_the_worker, asks) = mpsc::channel::<AskingGitHub>();
        let (answer, answers) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("signing-in-to-github".to_owned())
            .spawn(move || {
                let mut worker = Worker {
                    gh,
                    keep,
                    answer,
                    no_code_within,
                };
                while let Ok(asked) = asks.recv() {
                    if !worker.asked(asked, &asks) {
                        return;
                    }
                }
            });
        let to_the_worker = match worker {
            Ok(_) => Some(to_the_worker),
            Err(why) => {
                eprintln!(
                    "alo-desktop: nothing can sign in to GitHub, because no thread could be made \
                     for it: {why}"
                );
                None
            }
        };
        Self {
            to_the_worker,
            answers,
            unanswered: Vec::new(),
        }
    }

    /// Ask the worker; it answers later, through [`Self::answered`].
    pub fn asked(&mut self, asked: AskingGitHub) {
        let sent = self
            .to_the_worker
            .as_ref()
            .is_some_and(|worker| worker.send(asked).is_ok());
        if !sent && asked != AskingGitHub::StopSigningIn {
            self.unanswered.push(GitHubAnswered::DidNotWork {
                asked,
                why: WhyGitHubDidNotWork::GhFailed(None),
            });
        }
    }

    /// The next answer, once there is one.
    pub fn answered(&mut self) -> Option<GitHubAnswered> {
        if !self.unanswered.is_empty() {
            return Some(self.unanswered.remove(0));
        }
        self.answers.try_recv().ok()
    }
}

/// The worker's side.
struct Worker {
    /// The program.
    gh: PathBuf,
    /// Where each sign-in, sign-out and failure is recorded.
    keep: Keep,
    /// Where answers go.
    answer: Sender<GitHubAnswered>,
    /// How long a sign-in may go without printing its code.
    no_code_within: Duration,
}

/// What one short run of gh came to.
enum Ran {
    /// It ended well, having printed these lines.
    Well(Vec<String>),
    /// It ended badly, having printed these lines.
    Badly(Vec<String>),
    /// It is not on this computer.
    NotHere,
}

impl Worker {
    /// Carry `asked` out; `false` once nobody is listening any more.
    fn asked(&mut self, asked: AskingGitHub, asks: &Receiver<AskingGitHub>) -> bool {
        let answer = match asked {
            AskingGitHub::WhoIsSignedIn => self.who_is_signed_in(),
            AskingGitHub::SignIn => return self.signing_in(asks),
            // **Nothing is waiting**, so there is nothing to stop.
            AskingGitHub::StopSigningIn => return true,
            AskingGitHub::SignOut => self.signing_out(),
        };
        self.answer.send(answer).is_ok()
    }

    /// `gh config get -h github.com user`.
    fn who_is_signed_in(&self) -> GitHubAnswered {
        match self.short(&["config", "get", "-h", "github.com", "user"]) {
            Ran::Well(lines) => GitHubAnswered::SignedInAs(
                lines.iter().find_map(|line| what_gh_says::a_login(line)),
            ),
            // **gh says nothing is set by failing**, which is nobody.
            Ran::Badly(_) => GitHubAnswered::SignedInAs(None),
            Ran::NotHere => GitHubAnswered::DidNotWork {
                asked: AskingGitHub::WhoIsSignedIn,
                why: WhyGitHubDidNotWork::GhIsNotHere,
            },
        }
    }

    /// `gh auth logout --hostname github.com`, recorded.
    fn signing_out(&mut self) -> GitHubAnswered {
        let login = match self.who_is_signed_in() {
            GitHubAnswered::SignedInAs(login) => login,
            _ => None,
        };
        match self.short(&["auth", "logout", "--hostname", "github.com"]) {
            Ran::Well(_) => {
                if let Some(what) = AtGitHub::signed_out(login.as_deref()) {
                    (self.keep)(&what);
                }
                GitHubAnswered::SignedOut
            }
            Ran::Badly(lines) => {
                let said = what_gh_says::the_last_line(&lines);
                (self.keep)(&AtGitHub::did_not_sign_out(said.as_deref()));
                GitHubAnswered::DidNotWork {
                    asked: AskingGitHub::SignOut,
                    why: WhyGitHubDidNotWork::GhFailed(said),
                }
            }
            Ran::NotHere => {
                (self.keep)(&AtGitHub::did_not_sign_out(None));
                GitHubAnswered::DidNotWork {
                    asked: AskingGitHub::SignOut,
                    why: WhyGitHubDidNotWork::GhIsNotHere,
                }
            }
        }
    }

    /// `gh auth login --web`, watched until it ends, is cancelled, or prints
    /// no code in time. `false` once nobody is listening any more.
    fn signing_in(&mut self, asks: &Receiver<AskingGitHub>) -> bool {
        let mut child = match self
            .gh(&[
                "auth",
                "login",
                "--web",
                "--hostname",
                "github.com",
                "--git-protocol",
                "https",
            ])
            .spawn()
        {
            Ok(child) => child,
            Err(why) => {
                let why = if why.kind() == std::io::ErrorKind::NotFound {
                    WhyGitHubDidNotWork::GhIsNotHere
                } else {
                    WhyGitHubDidNotWork::GhFailed(None)
                };
                return self.did_not_sign_in(why);
            }
        };
        let lines = every_line_of(&mut child);
        let began = Instant::now();
        let mut said: Vec<String> = Vec::new();
        let mut code: Option<String> = None;
        let mut page: Option<String> = None;
        let mut handed = false;
        loop {
            match lines.recv_timeout(BETWEEN_LOOKS) {
                Ok(line) => {
                    if code.is_none() {
                        code = what_gh_says::the_code_on(&line);
                    }
                    if page.is_none() {
                        page = what_gh_says::the_page_on(&line);
                    }
                    said.push(line);
                    continue;
                }
                Err(RecvTimeoutError::Timeout) => {}
                // **Both streams closed**: wait between looks rather than
                // spinning while gh finishes.
                Err(RecvTimeoutError::Disconnected) => std::thread::sleep(BETWEEN_LOOKS),
            }
            // **The code is handed over once gh has printed it**, with the
            // page it named — or GitHub's own device page where it named none
            // by the time it went quiet.
            if !handed && let Some(the_code) = code.clone() {
                let answer = GitHubAnswered::TheCodeIs {
                    code: the_code,
                    page: page
                        .clone()
                        .unwrap_or_else(|| what_gh_says::THE_DEVICE_PAGE.to_owned()),
                };
                if self.answer.send(answer).is_err() {
                    ended(&mut child);
                    return false;
                }
                handed = true;
            }
            if let Ok(AskingGitHub::StopSigningIn) = asks.try_recv() {
                ended(&mut child);
                return self.did_not_sign_in(WhyGitHubDidNotWork::Cancelled);
            }
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Whatever it printed last is still on its way.
                    while let Ok(line) = lines.recv_timeout(BETWEEN_LOOKS) {
                        said.push(line);
                    }
                    return if status.success() {
                        self.signed_in(&said)
                    } else {
                        self.did_not_sign_in(WhyGitHubDidNotWork::GhFailed(
                            what_gh_says::the_last_line(&said),
                        ))
                    };
                }
                Ok(None) => {}
                Err(_) => {
                    ended(&mut child);
                    return self.did_not_sign_in(WhyGitHubDidNotWork::GhFailed(None));
                }
            }
            if !handed && began.elapsed() >= self.no_code_within {
                ended(&mut child);
                return self.did_not_sign_in(WhyGitHubDidNotWork::NoCodeArrived);
            }
        }
    }

    /// gh ended well: signed in as the login it named, or as gh's settings
    /// now say.
    fn signed_in(&mut self, said: &[String]) -> bool {
        let login = said
            .iter()
            .find_map(|line| what_gh_says::the_login_on(line))
            .or_else(|| match self.who_is_signed_in() {
                GitHubAnswered::SignedInAs(login) => login,
                _ => None,
            });
        let Some(login) = login else {
            // **Ended well and nobody is signed in** is a sign-in that did
            // not finish, said as gh's failure rather than as a success.
            return self.did_not_sign_in(WhyGitHubDidNotWork::GhFailed(
                what_gh_says::the_last_line(said),
            ));
        };
        if let Some(what) = AtGitHub::signed_in(&login) {
            (self.keep)(&what);
        }
        self.answer
            .send(GitHubAnswered::SignInWorked(login))
            .is_ok()
    }

    /// A sign-in that did not finish: recorded, and answered.
    fn did_not_sign_in(&mut self, why: WhyGitHubDidNotWork) -> bool {
        let kept = match &why {
            WhyGitHubDidNotWork::GhIsNotHere => WhyNotSignedIn::GhIsNotHere,
            WhyGitHubDidNotWork::GhFailed(said) => WhyNotSignedIn::gh_failed(said.as_deref()),
            WhyGitHubDidNotWork::NoCodeArrived => WhyNotSignedIn::NoCodeArrived,
            WhyGitHubDidNotWork::Cancelled => WhyNotSignedIn::TheyCancelled,
        };
        (self.keep)(&AtGitHub::did_not_sign_in(kept));
        self.answer
            .send(GitHubAnswered::DidNotWork {
                asked: AskingGitHub::SignIn,
                why,
            })
            .is_ok()
    }

    /// Run gh to its end, with `arguments`.
    fn short(&self, arguments: &[&str]) -> Ran {
        match self.gh(arguments).output() {
            Ok(output) => {
                let mut lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(str::to_owned)
                    .collect();
                lines.extend(
                    String::from_utf8_lossy(&output.stderr)
                        .lines()
                        .map(str::to_owned),
                );
                if output.status.success() {
                    Ran::Well(lines)
                } else {
                    Ran::Badly(lines)
                }
            }
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ran::NotHere,
            Err(why) => Ran::Badly(vec![why.to_string()]),
        }
    }

    /// gh with `arguments`: no input, no colour, no prompts, no update notice,
    /// and no browser of its own — the desktop opens the page.
    fn gh(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(&self.gh);
        command
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("NO_COLOR", "1")
            .env("CLICOLOR", "0")
            .env("GH_PROMPT_DISABLED", "1")
            .env("GH_NO_UPDATE_NOTIFIER", "1")
            .env("GH_BROWSER", "true");
        command
    }
}

/// Every line `child` prints, on either stream, as it prints it.
fn every_line_of(child: &mut Child) -> Receiver<String> {
    let (line, lines) = mpsc::channel();
    if let Some(out) = child.stdout.take() {
        reading(out, line.clone());
    }
    if let Some(err) = child.stderr.take() {
        reading(err, line);
    }
    lines
}

/// Read `stream` line by line onto `line`, on a thread of its own.
fn reading(stream: impl Read + Send + 'static, line: Sender<String>) {
    let _ = std::thread::Builder::new()
        .name("reading-gh".to_owned())
        .spawn(move || {
            for read in BufReader::new(stream).lines() {
                let Ok(read) = read else { return };
                if line.send(read).is_err() {
                    return;
                }
            }
        });
}

/// End `child` and wait for it, so nothing is left running.
fn ended(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "signing_in_to_github_tests.rs"]
mod tests;
