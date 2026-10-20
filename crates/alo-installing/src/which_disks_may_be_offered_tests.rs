//! That the installer's own medium is never offered, that what is on a disk is
//! said, and that an unmarked medium refuses what it cannot be sure about.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Strings;

use super::{TheMedium, what_may_be_offered};
use crate::disks::Disks;

/// A listing with two disks: one holding another system, one empty.
///
/// **Written as the lister prints it**, through `Disks::read`, so this test
/// exercises the same deserialisation a machine does rather than a shape built
/// by hand that could drift from it.
fn two_disks(installer_on: Option<&str>) -> Disks {
    let staged = installer_on.map_or(String::new(), |disk| {
        format!(
            r#",{{"name":"{disk}","type":"disk","ro":false,"mountpoints":[null],
                 "size":15728640000,"model":"Generic Flash Disk","children":[
                   {{"name":"{disk}1","type":"part","ro":false,"mountpoints":[null],
                     "parttype":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","label":"ALO-INSTALL"}}]}}"#
        )
    });
    Disks::read(&format!(
        r#"{{"blockdevices":[
            {{"name":"/dev/nvme0n1","type":"disk","ro":false,"mountpoints":[null],
              "size":1000204886016,"model":"Samsung SSD 870 EVO","children":[
                {{"name":"/dev/nvme0n1p1","type":"part","ro":false,"mountpoints":[null],
                  "parttype":"ebd0a0a2-b9e5-4433-87c0-68b6b72699c7","fsused":340000000000}}]}},
            {{"name":"/dev/sdb","type":"disk","ro":false,"mountpoints":[null],
              "size":2000398934016,"model":"WD Elements","children":[]}}
            {staged}]}}"#
    ))
    .expect("a listing this test wrote")
}

/// This crate's own sentences, in English.
fn english() -> Strings {
    Strings::of(crate::words::installing_words().expect("this crate's own vocabulary"))
}

/// **The disk the installer is running from is not offered**, and the same
/// listing with the installer elsewhere offers it.
///
/// The second half is the control. Without it this would pass on a function
/// that offered nothing at all.
#[test]
fn the_medium_this_installer_is_on_is_never_offered() {
    let strings = english();
    let disks = two_disks(Some("/dev/sdc"));
    let medium = TheMedium::of(&disks);
    assert_eq!(
        medium,
        TheMedium::ThisDisk("/dev/sdc".to_owned()),
        "the premise: the labelled area was found"
    );

    let shown = what_may_be_offered(&disks, &medium, &strings);
    assert!(
        !shown.offered.iter().any(|one| one.disk == "/dev/sdc"),
        "the disk this installer is running from was offered, so a person can erase the \
         installer while it is installing: {:?}",
        shown.offered
    );
    let refused = shown
        .refused
        .iter()
        .find(|one| one.disk == "/dev/sdc")
        .expect("the installer's own disk is refused in words, not hidden");
    assert!(
        refused.why.contains("installed from"),
        "the refusal does not say why: {}",
        refused.why
    );

    // **The control, and the first one written here was wrong.** It pointed
    // the medium at another disk and expected `/dev/sdc` to become offered.
    // It does not, and that is right: `crate::disks` refuses **any** disk
    // carrying the label, whatever this file thinks the medium is. So the two
    // rules overlap deliberately, and a test that moves the medium cannot show
    // this file doing anything.
    //
    // What isolates this file's rule is a disk the other rule would accept:
    // `/dev/sdb` is empty and unlabelled, so it is offered — unless this file
    // is told the installer is running from it, which is a stick somebody
    // wrote by hand onto a disk whose label is elsewhere.
    let on_the_empty_one = TheMedium::ThisDisk("/dev/sdb".to_owned());
    let shown = what_may_be_offered(&disks, &on_the_empty_one, &strings);
    assert!(
        !shown.offered.iter().any(|one| one.disk == "/dev/sdb"),
        "a disk that is otherwise offerable was still offered when it is the medium, so this \
         file's own rule does nothing: {:?}",
        shown.offered
    );
    let shown = what_may_be_offered(&disks, &TheMedium::Unlabelled, &strings);
    assert!(
        shown.offered.iter().any(|one| one.disk == "/dev/sdb"),
        "that same disk is refused even when it is not the medium, so the refusal above was \
         not this file's rule: {:?}",
        shown.refused
    );
}

/// **What is on a disk is said, and whether choosing it destroys anything.**
#[test]
fn each_disk_says_what_is_on_it() {
    let strings = english();
    let disks = two_disks(Some("/dev/sdc"));
    let shown = what_may_be_offered(&disks, &TheMedium::of(&disks), &strings);

    let windows = shown
        .offered
        .iter()
        .find(|one| one.disk == "/dev/nvme0n1")
        .expect("a disk holding another system is offered, because the person decides");
    assert_eq!(
        windows.called, "Samsung SSD 870 EVO  ·  1 TB",
        "a disk is not named the way a person bought it"
    );
    assert!(
        windows.erases,
        "choosing a disk holding another system is not marked as destroying anything"
    );
    assert!(
        windows.holding.contains("340 GB"),
        "what is on the disk is not said in words a person recognises: {}",
        windows.holding
    );

    let empty = shown
        .offered
        .iter()
        .find(|one| one.disk == "/dev/sdb")
        .expect("an empty disk is offered");
    assert_eq!(empty.called, "WD Elements  ·  2 TB");
    assert!(
        !empty.erases,
        "an empty disk is marked as destroying something"
    );
    assert!(
        empty.holding.contains("Nothing would be lost"),
        "an empty disk does not say that nothing is lost: {}",
        empty.holding
    );
}

/// **A medium with no mark refuses what it cannot be sure about**, and still
/// offers what costs nothing.
#[test]
fn an_unmarked_medium_will_not_erase_anything() {
    let strings = english();
    let disks = two_disks(None);
    let medium = TheMedium::of(&disks);
    assert_eq!(
        medium,
        TheMedium::Unlabelled,
        "the premise: nothing in this listing carries the installer's label"
    );

    let shown = what_may_be_offered(&disks, &medium, &strings);
    assert!(
        !shown.offered.iter().any(|one| one.erases),
        "an installer that cannot tell which disk it is on offered to erase one: {:?}",
        shown.offered
    );
    let refused = shown
        .refused
        .iter()
        .find(|one| one.disk == "/dev/nvme0n1")
        .expect("the disk holding another system is refused and said");
    assert!(
        refused.why.contains("cannot tell which disk"),
        "the refusal does not say what is uncertain: {}",
        refused.why
    );
    // **And it is a boundary rather than a dead end**: the empty disk is still
    // there, so a person has somewhere to go.
    assert!(
        shown.has_anything(),
        "nothing at all was offered, so the person meets a dead end"
    );
}

/// **A disk that reports no maker is read as its own path**, never as a name
/// this product invented.
#[test]
fn a_disk_with_no_maker_is_called_what_it_is() {
    let strings = english();
    let disks = Disks::read(
        r#"{"blockdevices":[{"name":"/dev/vda","type":"disk","ro":false,"mountpoints":[null],
             "size":68719476736,"children":[]}]}"#,
    )
    .expect("a listing this test wrote");
    let shown = what_may_be_offered(&disks, &TheMedium::Unlabelled, &strings);
    let only = shown.offered.first().expect("the disk is offered");
    assert_eq!(
        only.called, "/dev/vda  ·  68 GB",
        "a disk with no maker's name was given one"
    );
}

/// **Sizes are said in the units a disk was sold in.**
///
/// A 1 TB disk holds 1,000,204,886,016 bytes. Read as powers of 1024 that is
/// 931 GB, which is correct arithmetic about a question nobody asked.
#[test]
fn a_size_is_said_the_way_the_disk_was_sold() {
    assert_eq!(super::roughly(1_000_204_886_016), "1 TB");
    assert_eq!(super::roughly(2_000_398_934_016), "2 TB");
    assert_eq!(super::roughly(500_107_862_016), "500 GB");
    assert_eq!(super::roughly(340_000_000_000), "340 GB");
    // A disk sold as 1.5 TB keeps its half.
    assert_eq!(super::roughly(1_500_000_000_000), "1.5 TB");
}
