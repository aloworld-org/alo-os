//! Whether the Windows volume is encrypted, as `Get-BitLockerVolume` says.
//!
//! Read as the names of Windows' own enumerations, which the script turns into
//! strings, so the answer is the same whatever language Windows is set to.
//!
//! # What it decides
//!
//! One thing. **A volume BitLocker is still encrypting or decrypting is not
//! shrunk**: the conversion is Windows rewriting every block of that volume,
//! and this installer does not change the size of a volume while that is
//! happening. An encrypted volume is otherwise shrunk as any other is, through
//! Windows' own tool, which is what knows how; the installer never reads what
//! is on it and never touches its protectors.
//!
//! A volume whose state Windows did not report — a Windows edition without the
//! BitLocker cmdlets, say — is said as not known and decides nothing, because
//! the shrink goes through the same Windows tool that would refuse a
//! conversion in progress itself.

use serde::Deserialize;

/// The Windows volume's encryption, as Windows says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitLocker {
    /// Encrypted, or protection on.
    On,
    /// Not encrypted.
    Off,
    /// Being encrypted or decrypted, or paused part way.
    Changing,
    /// Windows did not say.
    NotRead,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// `FullyDecrypted`, `FullyEncrypted`, `EncryptionInProgress`, …
    volume_status: String,
    /// `Off`, `On`, `Unknown`.
    protection_status: String,
}

impl BitLocker {
    /// What the script printed.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        let Some(printed) =
            printed.and_then(|printed| serde_json::from_str::<Printed>(printed).ok())
        else {
            return Self::NotRead;
        };
        match (
            printed.volume_status.trim(),
            printed.protection_status.trim(),
        ) {
            (
                "EncryptionInProgress"
                | "DecryptionInProgress"
                | "EncryptionPaused"
                | "DecryptionPaused",
                _,
            ) => Self::Changing,
            ("FullyEncrypted", _) | ("FullyDecrypted", "On") => Self::On,
            ("FullyDecrypted", _) => Self::Off,
            _ => Self::NotRead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each state Windows names.
    #[test]
    fn each_state_is_read() {
        let read = |volume: &str, protection: &str| {
            BitLocker::read(Some(&format!(
                r#"{{"VolumeStatus":"{volume}","ProtectionStatus":"{protection}"}}"#
            )))
        };
        assert_eq!(read("FullyEncrypted", "On"), BitLocker::On);
        assert_eq!(read("FullyEncrypted", "Off"), BitLocker::On);
        assert_eq!(read("FullyDecrypted", "Off"), BitLocker::Off);
        assert_eq!(read("EncryptionInProgress", "Off"), BitLocker::Changing);
        assert_eq!(read("DecryptionPaused", "On"), BitLocker::Changing);
    }

    /// **A state Windows did not name is not known, never off.**
    #[test]
    fn an_unnamed_state_is_not_known() {
        assert_eq!(BitLocker::read(None), BitLocker::NotRead);
        assert_eq!(BitLocker::read(Some("1")), BitLocker::NotRead);
        assert_eq!(
            BitLocker::read(Some(r#"{"VolumeStatus":"2","ProtectionStatus":"1"}"#)),
            BitLocker::NotRead
        );
    }
}
