//! How the computer starts: UEFI or BIOS, and whether Secure Boot is on.
//!
//! Both are read from Windows rather than guessed. `%firmware_type%` is the
//! variable Windows sets from how it was itself started, and
//! `Confirm-SecureBootUEFI` asks the firmware's own `SecureBoot` variable. A
//! question Windows would not answer — a BIOS machine, where the second one
//! fails, or anything else — is kept as *not known*, and not known is never
//! read as *off*.

use serde::Deserialize;

/// How the computer starts, as Windows says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Starting {
    /// Whether it starts with UEFI; [`None`] when Windows did not say.
    pub uefi: Option<bool>,
    /// Whether Secure Boot is on; [`None`] when Windows did not say.
    pub secure_boot: Option<bool>,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// `UEFI`, `Legacy`, or empty.
    firmware_type: String,
    /// `true`, `false`, or `null` when the question failed.
    secure_boot: Option<bool>,
}

impl Starting {
    /// Nothing known.
    pub const NOT_READ: Self = Self {
        uefi: None,
        secure_boot: None,
    };

    /// What the script printed, or nothing known when it printed anything else.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        let Some(printed) =
            printed.and_then(|printed| serde_json::from_str::<Printed>(printed).ok())
        else {
            return Self::NOT_READ;
        };
        let uefi = match printed.firmware_type.trim() {
            "UEFI" => Some(true),
            "Legacy" => Some(false),
            _ => None,
        };
        Self {
            uefi,
            // A machine that does not start with UEFI has no Secure Boot to
            // ask about, and whatever came back is not an answer.
            secure_boot: if uefi == Some(true) {
                printed.secure_boot
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UEFI with Secure Boot on and off, as Windows prints them.
    #[test]
    fn uefi_and_secure_boot_are_read() {
        assert_eq!(
            Starting::read(Some(r#"{"FirmwareType":"UEFI","SecureBoot":true}"#)),
            Starting {
                uefi: Some(true),
                secure_boot: Some(true)
            }
        );
        assert_eq!(
            Starting::read(Some(r#"{"FirmwareType":"UEFI","SecureBoot":false}"#)),
            Starting {
                uefi: Some(true),
                secure_boot: Some(false)
            }
        );
    }

    /// **A question Windows did not answer is not known, never off.**
    #[test]
    fn an_unanswered_question_is_not_known_never_off() {
        assert_eq!(
            Starting::read(Some(r#"{"FirmwareType":"UEFI","SecureBoot":null}"#)).secure_boot,
            None
        );
        assert_eq!(
            Starting::read(Some(r#"{"FirmwareType":"Legacy","SecureBoot":false}"#)),
            Starting {
                uefi: Some(false),
                secure_boot: None
            }
        );
        for printed in [
            None,
            Some(""),
            Some("True"),
            Some(r#"{"FirmwareType":"Other"}"#),
        ] {
            assert_eq!(Starting::read(printed).uefi, None, "{printed:?}");
            assert_eq!(Starting::read(printed).secure_boot, None, "{printed:?}");
        }
    }
}
