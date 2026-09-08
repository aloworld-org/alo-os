//! Keep a working reserve on the host disk shared by Windows and WSL builds.
use crate::Result;
use std::process::Command;

/// Operational headroom, not an OS installation or hardware minimum.
const RESERVE: u64 = 12 * 1024 * 1024 * 1024;

/// Refuse new work on low space or an unavailable measurement; never clean files.
pub fn require_space() -> Result<()> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[System.IO.DriveInfo]::new('C:\\').AvailableFreeSpace",
        ])
        .output()?;
    if !output.status.success() {
        return Err("Cannot measure C: free space; work preserved, no cleanup attempted".into());
    }
    require_reading(&String::from_utf8(output.stdout)?)
}

/// Parse integer bytes only, so missing or malformed output cannot allow work.
fn require_reading(reading: &str) -> Result<()> {
    let available = reading
        .trim()
        .parse::<u64>()
        .map_err(|_| "Invalid C: free-space reading; work preserved")?;
    if available < RESERVE {
        return Err(format!(
            "C: has {available} bytes free, below the {RESERVE}-byte (12 GiB) build reserve; work preserved. Ask for storage review; never delete company-managed files automatically"
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_at_or_above_the_reserve_allows_work() {
        assert!(require_reading(&RESERVE.to_string()).is_ok());
        assert!(require_reading(&(RESERVE + 1).to_string()).is_ok());
    }

    #[test]
    fn space_below_the_reserve_refuses_work() {
        assert!(require_reading("0").is_err());
        assert!(require_reading(&(RESERVE - 1).to_string()).is_err());
    }

    #[test]
    fn invalid_or_missing_measurements_refuse_work() {
        for reading in ["", "-1", "17.5", "unknown", "1\n2", "18446744073709551616"] {
            assert!(require_reading(reading).is_err());
        }
    }

    #[test]
    fn powershell_line_endings_do_not_change_the_measurement() {
        assert!(require_reading(&format!("{RESERVE}\r\n")).is_ok());
    }
}
