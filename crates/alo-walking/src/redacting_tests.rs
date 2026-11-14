//! A secret is taken out of the log the walk keeps, wherever it appears.

use super::{REMOVED, without};

/// Every occurrence goes, and the rest of the log stays as it was.
#[test]
fn every_occurrence_of_a_secret_is_removed() {
    let log = b"key=sk-123 and again sk-123.\n";
    let kept = without(log, &["sk-123".to_owned()]);
    let mut expected = b"key=".to_vec();
    expected.extend_from_slice(REMOVED);
    expected.extend_from_slice(b" and again ");
    expected.extend_from_slice(REMOVED);
    expected.extend_from_slice(b".\n");
    assert_eq!(kept, expected);
    assert!(!String::from_utf8_lossy(&kept).contains("sk-123"));
}

/// An empty secret, or none at all, leaves the log untouched.
#[test]
fn no_secret_removes_nothing() {
    let log = b"nothing secret here";
    assert_eq!(without(log, &[]), log.to_vec());
    assert_eq!(without(log, &[String::new()]), log.to_vec());
}
