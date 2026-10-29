//! Setting the zone a person's region decided, on a disk a test owns.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// **A zone taken is set once**, and one that cannot be set is said and
/// tried again rather than believed.
#[test]
fn a_zone_taken_is_set_once_and_a_refusal_is_said() {
    let folder = tempfile::tempdir().unwrap();
    let zoneinfo = folder.path().join("zoneinfo");
    std::fs::create_dir_all(zoneinfo.join("America")).unwrap();
    std::fs::write(zoneinfo.join("America/Chicago"), "TZif").unwrap();
    let localtime = folder.path().join("localtime");
    let mut chose = TheZoneAPersonChose::nothing_yet();
    assert_eq!(
        chose.chose(None, &localtime, &zoneinfo),
        AboutTheZone::Unchanged
    );
    assert_eq!(
        chose.chose(Some("America/Chicago"), &localtime, &zoneinfo),
        AboutTheZone::Set("America/Chicago".to_owned())
    );
    assert_eq!(
        chose.chose(Some("America/Chicago"), &localtime, &zoneinfo),
        AboutTheZone::Unchanged
    );
    assert!(matches!(
        chose.chose(Some("America/Denver"), &localtime, &zoneinfo),
        AboutTheZone::NotSet(_)
    ));
    assert_eq!(
        std::fs::read_link(&localtime).unwrap(),
        zoneinfo.join("America/Chicago"),
        "a zone that could not be set moved the machine"
    );
}
