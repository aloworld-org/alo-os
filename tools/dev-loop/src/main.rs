//! Single-writer development supervisor. This is a developer tool, never an OS verb.
mod process;
mod report;

use process::{checked, git, worker};
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
    git(&["fetch", "origin", "main"])?;
    let head = git(&["rev-parse", "HEAD"])?;
    if head != git(&["rev-parse", "refs/remotes/origin/main"])? {
        return Err("Local and remote main differ; reconcile before restarting".into());
    }
    Ok(head)
}

fn run(state: &Path, codex: &str) -> Result<()> {
    loop {
        if state.join("STOP").exists() {
            status(state, "STOPPED: requested by owner")?;
            return Ok(());
        }
        let head = synchronized()?;
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let directory = state.join(stamp.to_string());
        fs::create_dir(&directory)?;
        let response = directory.join("result.txt");
        status(
            state,
            &format!("WORKING: {} logs={}", head.trim(), directory.display()),
        )?;
        worker(codex, &response, &directory)?;
        let result = fs::read_to_string(&response)?;
        let title = report::done_title(&result)?;
        if head != git(&["rev-parse", "HEAD"])? {
            return Err("Worker committed unexpectedly; review before publishing".into());
        }
        if git(&["status", "--porcelain"])?.trim().is_empty() {
            return Err("Worker reported done without changes".into());
        }
        // The running supervisor cannot gate a changed copy of itself.
        if !git(&["status", "--porcelain", "--", "tools/dev-loop"])?
            .trim()
            .is_empty()
        {
            return Err("Supervisor changes require interactive review and restart".into());
        }
        for file in [
            "CHANGELOG.md",
            "ROADMAP.md",
            "docs/autonomy/QUEUE.md",
            "docs/autonomy/STATE.md",
        ] {
            if git(&["diff", "HEAD", "--", file])?.is_empty() {
                return Err(format!("Required progress document was not updated: {file}").into());
            }
        }
        status(state, "VERIFYING: Windows, Linux, rustdoc and kernel gates")?;
        let mut log = File::create(directory.join("gates.log"))?;
        checked("cargo", &["fmt", "--all", "--check"], &mut log)?;
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
            &mut log,
        )?;
        checked(
            "cargo",
            &["test", "--workspace", "--locked", "--quiet"],
            &mut log,
        )?;
        checked(
            "wsl",
            &[
                "-d", "Ubuntu", "-u", "root", "--", "bash", "-lc", LINUX_GATE,
            ],
            &mut log,
        )?;
        checked("git", &["diff", "--check"], &mut log)?;
        // A normal push also rejects races after this fetch. Never force-push.
        git(&["fetch", "origin", "main"])?;
        if head != git(&["rev-parse", "refs/remotes/origin/main"])? {
            return Err("Remote moved during this step; preserve work for reconciliation".into());
        }
        git(&["add", "--all"])?;
        git(&[
            "commit",
            "-m",
            title,
            "-m",
            "Implemented by the development worker; the supervisor independently passed Windows and Linux gates before publication. See docs/autonomy/STATE.md for decisions and verification.",
        ])?;
        status(state, "PUSHING: origin main")?;
        git(&["push", "origin", "HEAD:refs/heads/main"])?;
        synchronized()?;
        status(
            state,
            &format!("PUSHED: {} {title}", git(&["rev-parse", "HEAD"])?.trim()),
        )?;
    }
}

// Fixed local checkout and separate Linux artifacts are intentional. No user data
// is interpolated into this shell programme. It never installs system services.
const LINUX_GATE: &str = r#"set -eu
export PATH=/root/.cargo/bin:/usr/lib/llvm-22/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export LLVM_PREFIX=/usr/lib/llvm-22
export CARGO_TARGET_DIR=/root/alo-os-target
cd /mnt/c/dev/alo-os
mountpoint -q /sys/fs/bpf || mount -t bpf bpffs /sys/fs/bpf
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked
cd crates/alo-bounding-kernel
cargo fmt --all --check
cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
"#;
