//! Whether the computer has a TPM, and whether it is ready — as `Get-Tpm` says.
//!
//! Said to the person and decides nothing: installing beside Windows neither
//! needs a TPM nor changes one. It is read because the plan asks that a person
//! be told what their computer is, and a TPM is part of that.

use serde::Deserialize;

/// The TPM, as Windows says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityChip {
    /// There is one, and it is ready.
    Ready,
    /// There is one, and it is not ready for use.
    NotReady,
    /// There is none.
    Absent,
    /// Windows did not say.
    NotRead,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// Whether there is one.
    tpm_present: bool,
    /// Whether it is ready.
    tpm_ready: bool,
}

impl SecurityChip {
    /// What the script printed.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        match printed.and_then(|printed| serde_json::from_str::<Printed>(printed).ok()) {
            Some(Printed {
                tpm_present: true,
                tpm_ready: true,
            }) => Self::Ready,
            Some(Printed {
                tpm_present: true,
                tpm_ready: false,
            }) => Self::NotReady,
            Some(Printed {
                tpm_present: false, ..
            }) => Self::Absent,
            None => Self::NotRead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each answer, and no answer.
    #[test]
    fn each_answer_is_read_and_no_answer_is_not_read() {
        assert_eq!(
            SecurityChip::read(Some(r#"{"TpmPresent":true,"TpmReady":true}"#)),
            SecurityChip::Ready
        );
        assert_eq!(
            SecurityChip::read(Some(r#"{"TpmPresent":true,"TpmReady":false}"#)),
            SecurityChip::NotReady
        );
        assert_eq!(
            SecurityChip::read(Some(r#"{"TpmPresent":false,"TpmReady":false}"#)),
            SecurityChip::Absent
        );
        assert_eq!(SecurityChip::read(None), SecurityChip::NotRead);
        assert_eq!(SecurityChip::read(Some("{}")), SecurityChip::NotRead);
    }
}
