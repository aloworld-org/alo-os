//! Single-writer development supervisor. This is a developer tool, never an OS verb.
mod awake;
mod linux_gate;
mod process;
mod publication;
mod recovery;
mod report;
mod step;
mod storage;

use process::{checked, git};
use std::{
    env,
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args
        .next()
        .ok_or("usage: alo-dev-loop <run|status|stop> [codex executable]")?;
    let root = git(&["rev-parse", "--show-toplevel"])?;
    if !root
        .trim()
        .replace('\\', "/")
        .eq_ignore_ascii_case("C:/dev/alo-os")
    {
        return Err("This runner's Linux gate requires the checkout at C:\\dev\\alo-os".into());
    }
    env::set_current_dir(root.trim())?;
    let state = Path::new(".git/alo-loop");
    fs::create_dir_all(state)?;
    match command.as_str() {
        "status" => println!(
            "{}",
            fs::read_to_string(state.join("status.txt")).unwrap_or_else(|_| "Not started".into())
        ),
        "stop" => {
            fs::write(state.join("STOP"), "Stop after the current step\n")?;
            println!("Stop requested after the current step.");
        }
        "run" => {
            let codex = args
                .next()
                .ok_or("run requires the absolute path to codex.exe")?;
            let lock = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(state.join("lock"))?;
            lock.try_lock()
                .map_err(|_| "Another supervisor is already running")?;
            if state.join("STOP").exists() {
                return Err(
                    "STOP is present; remove .git/alo-loop/STOP deliberately before restarting"
                        .into(),
                );
            }
            let result = run(state, &codex);
            if let Err(error) = &result {
                status(state, &format!("HALTED: {error}"))?;
            }
            result?;
        }
        _ => return Err("Unknown command; use run, status or stop".into()),
    }
    Ok(())
}

fn status(state: &Path, message: &str) -> Result<()> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let line = format!("{timestamp} pid={} {message}\n", std::process::id());
    fs::write(state.join("status.txt"), &line)?;
    let mut history = OpenOptions::new()
        .create(true)
        .append(true)
        .open(state.join("history.log"))?;
    history.write_all(line.as_bytes())?;
    println!("{line}");
    Ok(())
}

fn synchronized() -> Result<String> {
    if git(&["branch", "--show-current"])?.trim() != "main" {
        return Err("Expected branch main".into());
    }
    let remote = git(&["remote", "get-url", "--push", "origin"])?;
    if !report::expected_remote(remote.trim()) {
        return Err("Unexpected origin push URL".into());
    }
    if !git(&["status", "--porcelain"])?.trim().is_empty() {
        return Err("Working tree contains unfinished changes; review before restarting".into());
    }
    git(&["pull", "--ff-only", "origin", "main"])?;
    let head = git(&["rev-parse", "HEAD"])?;
    if head != git(&["rev-parse", "refs/remotes/origin/main"])? {
        return Err("Unpublished local commits remain; review before starting another task".into());
    }
    Ok(head)
}

fn run(state: &Path, codex: &str) -> Result<()> {
    let mut linux = awake::Awake::start()?;
    loop {
        if state.join("STOP").exists() {
            status(state, "STOPPED: requested by owner")?;
            return Ok(());
        }
        storage::require_space()?;
        linux.check()?;
        let head = synchronized()?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let directory = state.join(stamp.to_string());
        fs::create_dir(&directory)?;
        status(
            state,
            &format!("WORKING: {} logs={}", head.trim(), directory.display()),
        )?;
        let title = step::finish(state, codex, &head, &directory, None, &mut linux)?;
        linux.check()?;
        git(&["add", "--all"])?;
        git(&[
            "commit",
            "-m",
            &title,
            "-m",
            "Implemented by the development worker; the supervisor independently passed Windows and Linux gates before publication. See docs/autonomy/STATE.md for decisions and verification.",
        ])?;
        status(state, "PUBLISHING: integrate main and push")?;
        let mut integration_attempt = 0;
        let published = publication::publish(&head, git, || {
            linux.check()?;
            integration_attempt += 1;
            let integrated_directory = directory.join(format!("integration-{integration_attempt}"));
            fs::create_dir_all(&integrated_directory)?;
            let mut log = File::create(integrated_directory.join("gates.log"))?;
            status(state, "VERIFYING: integrated changes from main")?;
            if let Err(error) = gates(&mut log) {
                let integrated_head = git(&["rev-parse", "HEAD"])?;
                let repair_title = step::finish(
                    state,
                    codex,
                    &integrated_head,
                    &integrated_directory,
                    Some(format!("Combined-tree gate failed: {error}; see gates.log")),
                    &mut linux,
                )?;
                git(&["add", "--all"])?;
                git(&[
                    "commit",
                    "-m",
                    &repair_title,
                    "-m",
                    "Repair of the integrated task; all independent gates rerun before publication.",
                ])?;
            }
            linux.check()
        })?;
        status(state, &format!("PUSHED: {} {title}", published.trim()))?;
    }
}

/// Gate the current tree, including any changes integrated from another worker.
fn gates(log: &mut File) -> Result<()> {
    checked("cargo", &["fmt", "--all", "--check"], log)?;
    checked(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        log,
    )?;
    checked(
        "cargo",
        &["test", "--workspace", "--locked", "--quiet"],
        log,
    )?;
    linux_gate::run(log)?;
    checked("git", &["diff", "--check"], log)?;
    Ok(())
}
