//! Fixed identity and shared deadline prerequisites for the private agent walk.

use std::time::{Duration, Instant};

use crate::studio_agent_probe::Stage;

pub use crate::studio_agent_connect::before_deadline;
#[cfg(unix)]
pub use crate::studio_agent_description::read_description;

#[cfg(unix)]
pub use crate::studio_agent_deadline::DeadlineGuard;
#[cfg(unix)]
pub use crate::studio_agent_stage::exchange_stage;

/// Check the public daemon description against this disposable image's logins.
///
/// # Errors
/// Refuse malformed, oversized, unsupported or differently assigned descriptions.
/// The caller must independently establish the opened file's trusted metadata.
pub fn require_description(description: &str) -> Result<(), String> {
    if description.len() > 65_536 {
        return Err("the machine description exceeds its size limit".to_owned());
    }
    let value: toml::Value = toml::from_str(description)
        .map_err(|_| "the machine description is not valid TOML".to_owned())?;
    if !matches!(
        value.get("format").and_then(toml::Value::as_integer),
        Some(1..=4)
    ) {
        return Err("the machine description format is unsupported".to_owned());
    }
    let logins = value.get("logins");
    for (name, expected) in [("person", 1000), ("agent", 60_989), ("group", 60_989)] {
        if logins
            .and_then(|table| table.get(name))
            .and_then(toml::Value::as_integer)
            != Some(expected)
        {
            return Err(
                "the machine description does not name the private walk's logins".to_owned(),
            );
        }
    }
    Ok(())
}

/// Check metadata obtained from the already opened description file.
///
/// # Errors
/// Refuse non-root ownership, nonregular files, symlinks or untrusted write access.
pub fn require_description_metadata(
    owner_uid: u32,
    mode: u32,
    is_regular: bool,
    is_symlink: bool,
) -> Result<(), String> {
    if owner_uid != 0 || mode & 0o022 != 0 || !is_regular || is_symlink {
        return Err("the machine description must be a trusted root-owned regular file".to_owned());
    }
    Ok(())
}

/// Return the unspent part of the one five-minute private probe budget.
///
/// # Errors
/// Refuse backward clock observations and an exhausted budget.
pub fn remaining_budget(start: Instant, now: Instant) -> Result<Duration, String> {
    now.checked_duration_since(start)
        .and_then(|spent| Duration::from_secs(300).checked_sub(spent))
        .filter(|left| !left.is_zero())
        .ok_or_else(|| "the private agent probe exhausted its shared time budget".to_owned())
}

/// The exact person-controller handshake token for a native Settings boundary.
#[must_use]
pub const fn stage_token(stage: Stage) -> &'static str {
    match stage {
        Stage::AskBeforeGrant => "ask-before-grant",
        Stage::Granted => "granted",
        Stage::Revoked => "revoked",
        Stage::Full => "full",
        Stage::AskAgain => "ask-again",
    }
}
