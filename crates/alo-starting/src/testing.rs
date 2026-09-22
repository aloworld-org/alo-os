//! A start-up entry written the way a firmware reports one, for the tests on
//! both sides of the broker's door.
//!
//! It is here rather than in each test because there are two of them and they
//! must agree byte for byte: the side that proposes the change digests what the
//! firmware reported, and the side that carries it out digests it again. Two
//! copies of the shape would be two tests that pass while the machine they
//! describe does not exist.
//!
//! **Nothing on a machine calls this.** It builds bytes; it reads no firmware,
//! opens nothing, and is reached only from tests.

/// A start-up entry as a firmware reports one: the attributes, the length of
/// the path, the name, and the path.
#[must_use]
pub fn as_a_firmware_reports_it(named: &str, path: &str) -> Vec<u8> {
    let mut path_bytes = Vec::new();
    for unit in path.encode_utf16() {
        path_bytes.extend_from_slice(&unit.to_le_bytes());
    }
    path_bytes.extend_from_slice(&[0, 0]);

    let mut bytes = 1_u32.to_le_bytes().to_vec();
    let length = u16::try_from(path_bytes.len()).unwrap_or(u16::MAX);
    bytes.extend_from_slice(&length.to_le_bytes());
    for unit in named.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    bytes.extend_from_slice(&path_bytes);
    bytes
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::firmware::Entry;
    use crate::systems::THE_WINDOWS_LOADER_IN_A_PATH;

    /// **What this writes is what the reader reads**, which is the only reason
    /// a fixture is allowed to exist.
    #[test]
    fn what_this_writes_reads_back_as_the_entry_it_describes() {
        let bytes = as_a_firmware_reports_it("Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH);
        let entry = Entry::reported(1, &bytes).unwrap();
        assert_eq!(entry.described(), "Windows Boot Manager");
        assert!(entry.starts_windows());
    }
}
