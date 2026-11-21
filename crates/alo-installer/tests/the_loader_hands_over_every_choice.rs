//! **Every choice the installer stages reaches the environment through the
//! loader**: what `alo-installer` writes into `chosen.cfg`, read by the
//! loader's own entry (`image/installing/grub.cfg`), is the kernel command line
//! `alo-installing` reads its road from.
//!
//! A seam test, with both real sides: the installer's own writers and the
//! environment's own reader, and between them the entry exactly as the
//! repository ships it. Only the loader is stood in for, by the two things it
//! does with this file: `set` a variable, and put each `${variable}` on the
//! kernel line, empty when nothing set it.
//!
//! Written because each side was tested against its own idea of the other. The
//! installer's test checked that the entry reads `chosen.cfg` and names the
//! disk's variable. The environment's tests started from a command line that
//! already carried `alo.installing.into=` and `alo.installing.efi=`. Neither
//! asked whether the entry passes those two on, and it did not: on the road that
//! keeps Windows, the environment was told only the disk.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;

use alo_installing::{DiskName, NotTold, PartitionName, Replacing, Told};

/// The loader's entry, as the boot environment's recipe ships it.
fn the_entry() -> String {
    let at = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../image/installing/grub.cfg"
    );
    std::fs::read_to_string(at).unwrap_or_else(|why| panic!("{at} could not be read: {why}"))
}

/// **What the loader hands its kernel**, given this staged choice: the entry's
/// lines in order, with the line that sources `chosen.cfg` standing for the
/// choice's own lines, and the `linux` line with every `${variable}` replaced by
/// its value — empty when nothing set it, as the loader leaves one.
fn handed_to_the_kernel(entry: &str, choice: &str) -> String {
    let mut set: BTreeMap<String, String> = BTreeMap::new();
    let mut line_for_the_kernel = None;
    for line in entry
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
    {
        let lines: Vec<&str> = if line.starts_with("source ") && line.contains("chosen.cfg") {
            choice.lines().map(str::trim).collect()
        } else {
            vec![line]
        };
        for line in lines {
            if let Some((name, value)) = line
                .strip_prefix("set ")
                .and_then(|setting| setting.split_once('='))
            {
                set.insert(name.to_owned(), value.to_owned());
            }
        }
        if let Some(kernel) = line.strip_prefix("linux ") {
            line_for_the_kernel = Some(kernel.to_owned());
        }
    }
    let mut line = line_for_the_kernel.expect("the entry starts a kernel");
    while let Some(start) = line.find("${") {
        let end = line[start..]
            .find('}')
            .map(|at| start + at)
            .expect("a variable is closed");
        let name = &line[start + 2..end];
        let value = set.get(name).cloned().unwrap_or_default();
        line.replace_range(start..=end, &value);
    }
    line
}

/// The disk the installer on Windows names, as the machine names it.
fn the_disk() -> DiskName {
    DiskName::named("nvme-Samsung_SSD_980_1TB_S64ANS0T123456A").expect("a disk's name")
}

/// One of that disk's partitions.
fn its_partition(number: u32) -> PartitionName {
    PartitionName::named(&format!("{}-part{number}", the_disk().as_str()))
        .expect("a partition's name")
}

/// What the environment reads off the kernel line the loader hands it.
fn told_through_the_loader(choice: &str) -> Result<Told, NotTold> {
    Told::from_the_command_line(&handed_to_the_kernel(&the_entry(), choice))
}

/// **The road that keeps Windows arrives as that road**: the space the
/// installer made, and Windows' start-up area, both reach the environment, and
/// nothing says replace.
#[test]
fn the_road_beside_windows_reaches_the_environment_with_both_partitions() {
    let choice = alo_installer::the_choice_beside_what_is_there(
        &the_disk(),
        &its_partition(6),
        &its_partition(1),
    );
    let told = told_through_the_loader(&choice).expect("the environment understands the line");
    assert_eq!(told.disk(), &the_disk());
    assert!(!told.replaces_what_is_there());
    let beside = told.beside().unwrap_or_else(|| {
        panic!(
            "the environment was told only the disk, so it takes the whole-disk road: {}",
            handed_to_the_kernel(&the_entry(), &choice)
        )
    });
    assert_eq!(beside.root(), &its_partition(6));
    assert_eq!(beside.efi(), &its_partition(1));
}

/// **The ordinary road and the road that replaces Windows arrive as they did**:
/// a disk taken whole, replacing only where the choice said so in so many
/// words, and beside nothing.
#[test]
fn the_roads_that_take_a_disk_whole_reach_the_environment_unchanged() {
    for (replacing, replaces) in [
        (Replacing::Nothing, false),
        (Replacing::TheSystemOnTheDisk, true),
    ] {
        let choice = alo_installer::the_choice(&the_disk(), replacing);
        let told = told_through_the_loader(&choice).expect("the environment understands the line");
        assert_eq!(told.disk(), &the_disk());
        assert_eq!(told.replaces_what_is_there(), replaces, "{choice}");
        assert!(told.beside().is_none(), "{choice}");
    }
}

/// **No staged choice is no choice**: an area without `chosen.cfg` gives the
/// environment nothing to install onto, and it refuses.
#[test]
fn no_staged_choice_is_no_choice() {
    assert_eq!(
        told_through_the_loader("").map(|told| told.disk().clone()),
        Err(NotTold::NothingChosen)
    );
}

/// **The stand-in for the loader can tell a line that hands a variable over
/// from one that does not**, so the tests above can fail: a variable the entry
/// sets and passes on arrives with the choice's value, one it never names
/// arrives as nothing, and one nothing set arrives empty.
#[test]
fn the_stand_in_for_the_loader_hands_over_only_what_the_entry_names() {
    let entry = "set a=\nset b=\nsource \"${config_directory}/chosen.cfg\"\n\
                 linux /vmlinuz first=${a} third=${c}\n";
    let line = handed_to_the_kernel(entry, "set a=one\nset b=two\n");
    assert_eq!(line, "/vmlinuz first=one third=");
}
