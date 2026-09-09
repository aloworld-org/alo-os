//! A supervisor-owned stdin lease keeps WSL active without changing its configuration.
use crate::Result;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const READY: &str = "alo-dev-loop-awake";
const HOLD: &str = "printf 'alo-dev-loop-awake\\n'\nexec cat > /dev/null";

/// Closing the owned pipe makes Linux cat exit on EOF, including parent death.
/// This does not mount bpffs, change services or prevent Windows sleep/reboots.
pub struct Awake {
    child: Child,
}

impl Awake {
    pub fn start() -> Result<Self> {
        let mut command = Command::new("wsl");
        command.args([
            "-d", "Ubuntu", "-u", "root", "--exec", "/bin/sh", "-c", HOLD,
        ]);
        Self::launch(&mut command, Duration::from_secs(30))
    }

    fn launch(command: &mut Command, timeout: Duration) -> Result<Self> {
        let mut lease = Self {
            child: command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?,
        };
        let output = lease
            .child
            .stdout
            .take()
            .ok_or("WSL lease has no output pipe")?;
        let (send, receive) = mpsc::channel();
        thread::spawn(move || {
            let mut line = String::new();
            let ready = BufReader::new(output).read_line(&mut line).is_ok() && line.trim() == READY;
            let _ = send.send(ready);
        });
        if receive.recv_timeout(timeout) != Ok(true) {
            return Err("WSL keep-awake lease did not become ready; work preserved".into());
        }
        lease.check()?;
        Ok(lease)
    }

    /// A dead helper is a failure, never permission to restart or repair WSL.
    pub fn check(&mut self) -> Result<()> {
        if let Some(exit) = self.child.try_wait()? {
            return Err(format!("WSL keep-awake lease ended ({exit}); work preserved").into());
        }
        self.child
            .stdin
            .as_mut()
            .ok_or("WSL lease input is closed")?
            .write_all(b"\n")?;
        Ok(())
    }

    fn release(&mut self) -> Result<()> {
        drop(self.child.stdin.take());
        let until = Instant::now() + Duration::from_secs(2);
        while self.child.try_wait()?.is_none() {
            if Instant::now() >= until {
                // Only our own launcher, never another WSL session or a name-wide kill.
                self.child.kill()?;
                self.child.wait()?;
                return Err("WSL launcher did not exit promptly after lease EOF".into());
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }
}

impl Drop for Awake {
    fn drop(&mut self) {
        if let Err(error) = self.release() {
            eprintln!("WSL lease cleanup: {error}");
        }
    }
}

#[cfg(test)]
#[path = "awake_tests.rs"]
mod tests;
