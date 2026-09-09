//! Real process evidence for the Windows supervisor's non-mutating WSL lease.
use super::*;

#[test]
fn a_missing_launcher_refuses_startup() {
    let mut command = Command::new("alo-dev-loop-no-such-launcher");
    assert!(Awake::launch(&mut command, Duration::from_secs(1)).is_err());
}

#[test]
fn a_launcher_without_the_handshake_is_refused() {
    let mut command = Command::new("git");
    command.arg("--version");
    assert!(Awake::launch(&mut command, Duration::from_secs(1)).is_err());
}

#[cfg(windows)]
#[test]
fn a_real_wsl_lease_lives_until_its_owned_pipe_closes() -> Result<()> {
    let mut lease = Awake::start()?;
    lease.check()?;
    assert!(lease.child.try_wait()?.is_none());
    lease.release()?;
    assert!(lease.child.try_wait()?.is_some_and(|exit| exit.success()));
    assert!(lease.check().is_err());
    Ok(())
}

#[cfg(windows)]
#[test]
fn loss_of_the_owned_launcher_is_not_silently_repaired() -> Result<()> {
    let mut lease = Awake::start()?;
    lease.child.kill()?;
    lease.child.wait()?;
    assert!(lease.check().is_err());
    // Drop closes the pipe even if the launcher already died.
    Ok(())
}

#[cfg(windows)]
#[test]
fn a_missing_handshake_times_out_and_closes_the_pipe() {
    let mut command = Command::new("wsl");
    command.args(["-d", "Ubuntu", "-u", "root", "--exec", "/bin/cat"]);
    assert!(Awake::launch(&mut command, Duration::from_millis(100)).is_err());
}
