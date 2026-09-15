//! Whether the installer was started with an administrator's rights.
//!
//! Changing a disk and the list of systems a computer starts needs them, and
//! the installer asks before it asks anything else, so a person is never shown
//! a list of what their computer is only to be told at the end that nothing
//! could be done.
//!
//! Read from `whoami /groups`, by the one thing in its output that is the same
//! in every language: the security identifier Windows gives a process running
//! with its full rights, *Mandatory Label\High Mandatory Level*. A process an
//! administrator started without elevating carries the medium level instead.

/// The mandatory label of a process with an administrator's full rights.
pub const HIGH_MANDATORY_LEVEL: &str = "S-1-16-12288";

/// Whether what `whoami /groups /fo csv /nh` printed is an elevated process's.
#[must_use]
pub fn is_elevated(printed: &str) -> bool {
    printed.lines().any(|line| {
        line.split(',')
            .any(|field| field.trim().trim_matches('"') == HIGH_MANDATORY_LEVEL)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Elevated is the high level's identifier and nothing resembling it.**
    #[test]
    fn elevated_is_the_high_levels_identifier() {
        let elevated = "\"BUILTIN\\Administrators\",\"Alias\",\"S-1-5-32-544\",\"Group used for deny only\"\n\
             \"Mandatory Label\\High Mandatory Level\",\"Label\",\"S-1-16-12288\",\"\"\n";
        assert!(is_elevated(elevated));

        let not = "\"BUILTIN\\Administrators\",\"Alias\",\"S-1-5-32-544\",\"Group used for deny only\"\n\
             \"Mandatory Label\\Medium Mandatory Level\",\"Label\",\"S-1-16-8192\",\"\"\n";
        assert!(!is_elevated(not));
        assert!(!is_elevated("\"x\",\"Label\",\"S-1-16-122880\",\"\"\n"));
        assert!(!is_elevated(""));
    }
}
