//! How much memory the computer has, as `Win32_ComputerSystem` says.
//!
//! Said to the person, with a second sentence when it is less than alo OS is
//! made for (`docs/hardware.md`: the ordinary business laptop it is designed
//! around has 16 GB). It decides nothing: less memory is a slower alo OS, not
//! a broken computer, and whether that is worth it is the person's call.

use serde::Deserialize;

/// The memory alo OS is made for.
pub const MADE_FOR: u64 = 16 * crate::sizes::GIB;

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// Bytes.
    total_physical_memory: u64,
}

/// The memory, in bytes, or [`None`] when Windows did not say.
#[must_use]
pub fn read(printed: Option<&str>) -> Option<u64> {
    printed
        .and_then(|printed| serde_json::from_str::<Printed>(printed).ok())
        .map(|printed| printed.total_physical_memory)
        .filter(|bytes| *bytes > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bytes are read, and nothing else is.
    #[test]
    fn bytes_are_read_and_nothing_else_is() {
        assert_eq!(
            read(Some(r#"{"TotalPhysicalMemory":17062334464}"#)),
            Some(17_062_334_464)
        );
        assert_eq!(read(Some(r#"{"TotalPhysicalMemory":0}"#)), None);
        assert_eq!(read(Some(r#"{"TotalPhysicalMemory":-1}"#)), None);
        assert_eq!(read(None), None);
    }
}
