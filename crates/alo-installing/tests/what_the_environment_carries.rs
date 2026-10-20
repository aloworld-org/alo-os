//! The recipe in `image/installing/` held to the program it carries.
//!
//! Three files there are configuration a build reads and no compiler checks,
//! and each of them can disagree with this crate in a way nothing notices until
//! a machine restarts into the environment:
//!
//! - **the initramfs's list** (`alo-installing.conf`) has to carry every program
//!   this crate runs, at the path it runs it from, and the pin and key where
//!   `alo_installing::WHERE_IT_IS` reads them, and the programs `bootc install`
//!   itself finishes with once the image is on the disk;
//! - **the units** have to start this program, and the target has to be the one
//!   the loader names, and the service has to run under the environment bound
//!   again (`run-alo-installing-root.mount`), because the sandbox `bootc
//!   install` starts the boot loader's installer in cannot pivot from the
//!   initramfs's own root;
//! - **the loader's entry** (`grub.cfg`) has to name that target and hand over
//!   the chosen disk in exactly the word `alo_installing::THE_CHOICE` reads — and
//!   never a disk of its own.
//!
//! And the refusals beside them: each check is shown to catch the file it reads
//! being wrong, against a copy of that file with one thing changed.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_installing::{EVERY_PROGRAM, THE_CHOICE, WHERE_IT_IS};

/// A file of `image/installing/`, as text.
fn the_recipes(file: &str) -> String {
    std::fs::read_to_string(
        Path::new(alo_image::THE_IMAGE)
            .join("installing")
            .join(file),
    )
    .expect("the recipe's file is there")
}

/// Everything an initramfs list installs, one path per entry.
fn installed_by(list: &str) -> Vec<String> {
    list.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.strip_prefix("install_items+="))
        .flat_map(|items| {
            items
                .trim_matches('"')
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// What the list is missing of what this crate needs aboard.
fn missing_from(list: &str) -> Vec<String> {
    let installed = installed_by(list);
    let pin = format!("{WHERE_IT_IS}/{}", alo_image::THE_PIN);
    let key = format!("{WHERE_IT_IS}/signing/alo-os.pub");
    EVERY_PROGRAM
        .iter()
        .map(|program| (*program).to_owned())
        .chain([
            "/usr/bin/alo-installing".to_owned(),
            "/usr/lib/systemd/system/alo-installing.target".to_owned(),
            "/usr/lib/systemd/system/alo-installing.service".to_owned(),
            format!("/usr/lib/systemd/system/{THE_ROOTS_UNIT}"),
            pin,
            key,
        ])
        .filter(|needed| !installed.contains(needed))
        .collect()
}

/// What is wrong with a loader entry, as sentences.
fn wrong_with_the_entry(entry: &str) -> Vec<&'static str> {
    let mut wrong = Vec::new();
    let lines: Vec<&str> = entry
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .collect();
    let linux: Vec<&&str> = lines
        .iter()
        .filter(|line| line.starts_with("linux "))
        .collect();
    let [linux] = linux.as_slice() else {
        wrong.push("the entry does not start exactly one kernel");
        return wrong;
    };
    let words: Vec<&str> = linux.split_whitespace().collect();
    if !words.contains(&"rd.systemd.unit=alo-installing.target") {
        wrong.push("the kernel is not told to start the installer's target");
    }
    let handed_over = format!("{THE_CHOICE}${{alo_installing_to}}");
    let choices: Vec<&str> = words
        .iter()
        .copied()
        .filter(|word| word.starts_with(THE_CHOICE))
        .collect();
    if choices != [handed_over.as_str()] {
        wrong
            .push("the chosen disk is not handed over as exactly one word, from the staged choice");
    }
    if !lines.contains(&"set alo_installing_to=") {
        wrong.push("the choice is not emptied before the staged file is read");
    }
    if !words.contains(&"rd.shell=0") {
        wrong.push("the environment offers a shell");
    }
    // Where the staged choice is read from. The base's signed loader leaves
    // `cmdpath` empty and sets only `config_directory`, so an entry that sources
    // `${cmdpath}/chosen.cfg` reads nothing and every install refuses *no disk
    // was chosen* (`docs/quirks.md`, *Fedora's signed loader leaves `cmdpath`
    // empty*).
    if !lines
        .iter()
        .any(|line| line.contains("${config_directory}/chosen.cfg"))
    {
        wrong.push("the staged choice is not read from the directory the entry was read from");
    }
    if lines.iter().any(|line| line.contains("${cmdpath}")) {
        wrong.push("the entry reads a directory the base's loader leaves empty");
    }
    wrong
}

/// Where the installer's root is bound, and the unit that binds it.
const THE_ROOT: &str = "/run/alo/installing/root";

/// The mount unit systemd names for [`THE_ROOT`].
const THE_ROOTS_UNIT: &str = "run-alo-installing-root.mount";

/// The value of `key=` in a unit's lines, when it is given exactly once.
fn given_once<'a>(unit: &'a str, key: &str) -> Option<&'a str> {
    let values: Vec<&str> = unit
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.strip_prefix(key)?.strip_prefix('='))
        .map(str::trim)
        .collect();
    match values.as_slice() {
        [value] => Some(value),
        _ => None,
    }
}

/// What is wrong with the root the installer runs under, as sentences.
///
/// `bwrap`, which `bootc install` starts the boot loader's installer in, always
/// calls `pivot_root(2)`, and the kernel refuses that call (*EINVAL*) for a
/// process whose root is the initial root file system — the initramfs, which
/// the environment never switches out of. Measured in a virtual machine on
/// 2026-09-16 (`docs/quirks.md`, *the bootloader's probe dies in `bwrap`'s
/// `pivot_root`*): from the initramfs's root, *bwrap: pivot_root: Invalid
/// argument*; from a service whose root is the environment bound again, the
/// same `bwrap` starts `bootupctl`. So the service must run under that bound
/// root, and the bind must carry the whole environment with its mounts.
fn wrong_with_the_root(service: &str, mount: &str) -> Vec<&'static str> {
    let mut wrong = Vec::new();
    if given_once(service, "RootDirectory") != Some(THE_ROOT) {
        wrong.push("the installer does not run under the bound root, so its sandbox cannot pivot");
    }
    let wants = |key: &str| {
        service
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.strip_prefix(key)?.strip_prefix('='))
            .any(|units| units.split_whitespace().any(|unit| unit == THE_ROOTS_UNIT))
    };
    if !wants("Requires") {
        wrong.push("the installer can start without its root bound");
    }
    if !wants("After") {
        wrong.push("the installer can start before its root is bound");
    }
    if given_once(mount, "Where") != Some(THE_ROOT) {
        wrong.push("the root is bound somewhere other than where the installer runs");
    }
    // systemd names a mount unit by its path; a unit whose name does not match
    // its `Where=` is refused when it loads, and the installer would never start.
    let named = format!(
        "{}.mount",
        THE_ROOT.trim_start_matches('/').replace('/', "-")
    );
    if named != THE_ROOTS_UNIT
        || THE_ROOT
            .trim_start_matches('/')
            .split('/')
            .any(|part| part.contains('-'))
    {
        wrong.push("the root's unit is not named for the path it binds");
    }
    if given_once(mount, "What") != Some("/") {
        wrong.push("the bound root is not the whole environment");
    }
    let options: Vec<&str> = given_once(mount, "Options")
        .map(|options| options.split(',').map(str::trim).collect())
        .unwrap_or_default();
    // A plain bind carries `/` without `/dev`, `/proc`, `/sys` or `/run` — no
    // disk to write, and no log or network manager to speak to.
    if !options.contains(&"rbind") {
        wrong.push("the bound root does not carry the mounts beneath the environment's root");
    }
    // Shared, what the installer mounts on the new disk would appear in the
    // environment's own tree as well.
    if !options.contains(&"rslave") {
        wrong.push("what the installer mounts under its root spreads back into the environment");
    }
    wrong
}

/// **The initramfs carries every program this crate runs**, this program, its
/// units, and the pin and key where it reads them.
#[test]
fn the_initramfs_carries_everything_this_program_runs() {
    assert_eq!(
        missing_from(&the_recipes("alo-installing.conf")),
        Vec::<String>::new()
    );
}

/// **A list that dropped one is caught**, for each of them.
#[test]
fn a_list_missing_a_program_is_caught() {
    let list = the_recipes("alo-installing.conf");
    for program in EVERY_PROGRAM {
        let dropped = list.replace(&format!(" {program} "), " ");
        assert_ne!(dropped, list, "the list no longer names {program}");
        assert_eq!(missing_from(&dropped), vec![program.to_owned()]);
    }
    let commented = list.replace(
        "install_items+=\" /usr/lib/alo/installing/",
        "# install_items+=\" /usr/lib/alo/installing/",
    );
    assert_eq!(missing_from(&commented).len(), 2, "{commented}");
}

/// **The unit starts this program and the target requires the unit.**
#[test]
fn the_units_start_this_program() {
    let service = the_recipes("alo-installing.service");
    assert!(
        service
            .lines()
            .any(|line| line.trim() == "ExecStart=/usr/bin/alo-installing"),
        "{service}"
    );
    assert!(
        service.lines().any(|line| line.trim() == "Type=oneshot"),
        "{service}"
    );
    let target = the_recipes("alo-installing.target");
    assert!(
        target
            .lines()
            .any(|line| line.trim() == "Requires=alo-installing.service"),
        "{target}"
    );
}

/// **The loader's entry names the installer's target and hands over exactly the
/// staged choice** — and never a disk of its own.
#[test]
fn the_loader_hands_over_the_staged_choice_and_no_disk_of_its_own() {
    assert_eq!(
        wrong_with_the_entry(&the_recipes("grub.cfg")),
        Vec::<&str>::new()
    );
}

/// **An entry that chose a disk itself, chose twice, forgot the target, or
/// offered a shell is caught.**
#[test]
fn an_entry_that_chooses_a_disk_itself_is_caught() {
    let entry = the_recipes("grub.cfg");
    for (from, to) in [
        (
            "alo.installing.to=${alo_installing_to}",
            "alo.installing.to=nvme-Samsung_SSD_980",
        ),
        (
            "alo.installing.to=${alo_installing_to}",
            "alo.installing.to=${alo_installing_to} alo.installing.to=virtio-a",
        ),
        ("rd.systemd.unit=alo-installing.target ", ""),
        ("rd.shell=0 ", ""),
        ("set alo_installing_to=\n", "\n"),
        // The loader variable that reads nothing: measured empty in a virtual
        // machine on 2026-09-16.
        ("${config_directory}/chosen.cfg", "${cmdpath}/chosen.cfg"),
    ] {
        assert!(entry.contains(from), "the entry no longer says {from}");
        assert!(
            !wrong_with_the_entry(&entry.replace(from, to)).is_empty(),
            "{to}"
        );
    }
}

/// **The installer runs under a root its sandbox can pivot from**: the
/// environment bound again, whole, with its mounts, and the unit waits for it.
#[test]
fn the_installer_runs_under_a_root_its_sandbox_can_pivot_from() {
    assert_eq!(
        wrong_with_the_root(
            &the_recipes("alo-installing.service"),
            &the_recipes(THE_ROOTS_UNIT)
        ),
        Vec::<&str>::new()
    );
}

/// **A service left on the initramfs's own root is caught**, and so is every
/// other way the bound root stops being one the sandbox can pivot from.
#[test]
fn a_root_the_sandbox_cannot_pivot_from_is_caught() {
    let service = the_recipes("alo-installing.service");
    let mount = the_recipes(THE_ROOTS_UNIT);
    for (from, to) in [
        // The fault itself: the installer on the initramfs's root.
        ("RootDirectory=/run/alo/installing/root\n", "\n"),
        (
            "RootDirectory=/run/alo/installing/root\n",
            "RootDirectory=/\n",
        ),
        (
            "RootDirectory=/run/alo/installing/root\n",
            "RootDirectory=/run/alo/installing/elsewhere\n",
        ),
        (
            "Requires=run-alo-installing-root.mount\n",
            "Wants=run-alo-installing-root.mount\n",
        ),
        ("After=run-alo-installing-root.mount\n", "\n"),
    ] {
        assert!(service.contains(from), "the service no longer says {from}");
        assert!(
            !wrong_with_the_root(&service.replace(from, to), &mount).is_empty(),
            "{to}"
        );
    }
    for (from, to) in [
        (
            "Where=/run/alo/installing/root\n",
            "Where=/run/alo-installing-root\n",
        ),
        ("What=/\n", "What=/usr\n"),
        ("Options=rbind,rslave\n", "Options=bind,rslave\n"),
        ("Options=rbind,rslave\n", "Options=rbind\n"),
        ("Options=rbind,rslave\n", "\n"),
    ] {
        assert!(mount.contains(from), "the mount no longer says {from}");
        assert!(
            !wrong_with_the_root(&service, &mount.replace(from, to)).is_empty(),
            "{to}"
        );
    }
}

/// **The recipe copies the root's unit to where the initramfs's list takes it
/// from**, and the list takes it: a unit the service requires and the
/// initramfs lacks is an installer that never starts.
#[test]
fn the_roots_unit_is_built_into_the_environment() {
    let landing = format!("/usr/lib/systemd/system/{THE_ROOTS_UNIT}");
    let recipe = the_recipes("Containerfile");
    let image = std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join("Containerfile"))
        .expect("the image's recipe is there");
    let read = alo_image::TheEnvironment::read(&recipe, &image);
    assert!(
        read.copies(&format!("image/installing/{THE_ROOTS_UNIT}"), &landing),
        "{recipe}"
    );
    assert!(installed_by(&the_recipes("alo-installing.conf")).contains(&landing));
    let without = recipe.replace(&format!("COPY image/installing/{THE_ROOTS_UNIT} "), "# ");
    assert!(
        !alo_image::TheEnvironment::read(&without, &image)
            .copies(&format!("image/installing/{THE_ROOTS_UNIT}"), &landing)
    );
}

/// **What `bootc install` finishes with, from the environment rather than from
/// the image it deployed**: after the boot loader, bootc 1.15.1 trims each file
/// system it made (`fstrim`), remounts it read-only (`mount`), freezes and thaws
/// it (`fsfreeze`), and unmounts them all (`umount`) — each started by name, with
/// nothing said around a failure to start it.
///
/// Measured in a virtual machine on 2026-09-16: with `fstrim` missing from the
/// initramfs, the image deployed and the install then ended *No such file or
/// directory (os error 2)*, naming no program (`docs/quirks.md`, *`bootc install`
/// ends "No such file or directory" when the environment lacks `fstrim`*).
const WHAT_THE_INSTALLER_FINISHES_WITH: [&str; 4] = [
    "/usr/sbin/fstrim",
    "/usr/bin/mount",
    "/usr/sbin/fsfreeze",
    "/usr/bin/umount",
];

/// What the list is missing of what the installer finishes with.
fn missing_for_finishing(list: &str) -> Vec<&'static str> {
    let installed = installed_by(list);
    WHAT_THE_INSTALLER_FINISHES_WITH
        .into_iter()
        .filter(|program| !installed.iter().any(|item| item == program))
        .collect()
}

/// **The initramfs carries what the installer finishes with**, so an install
/// that has written the image does not end at the step that makes the disk
/// clean to start.
#[test]
fn the_initramfs_carries_what_the_installer_finishes_with() {
    assert_eq!(
        missing_for_finishing(&the_recipes("alo-installing.conf")),
        Vec::<&str>::new()
    );
}

/// The program that makes the one file system the installer asks for, where
/// the initramfs list installs it: `mkfs.` and `alo_image::THE_ONLY_FILESYSTEM`,
/// so a change of file system is a change of this list too.
fn the_maker_of_the_one_filesystem() -> String {
    format!("/usr/sbin/mkfs.{}", alo_image::THE_ONLY_FILESYSTEM)
}

/// **The initramfs carries the maker of the file system the installer asks
/// for.** `bootc install to-disk --filesystem btrfs` starts `mkfs.btrfs` by name
/// as it makes the disk's root, and says nothing around a failure to start it.
/// Measured on the installer's own road on 2026-09-22: without it the install
/// stopped *Creating rootfs: No such file or directory (os error 2)* 3–4 s after
/// the disk was partitioned (`docs/quirks.md`).
#[test]
fn the_initramfs_carries_the_maker_of_the_one_filesystem() {
    let maker = the_maker_of_the_one_filesystem();
    assert!(
        installed_by(&the_recipes("alo-installing.conf")).contains(&maker),
        "alo-installing.conf does not install {maker}"
    );
}

/// **A list without it is caught**, whether dropped or commented out.
#[test]
fn a_list_missing_the_maker_of_the_one_filesystem_is_caught() {
    let maker = the_maker_of_the_one_filesystem();
    let list = the_recipes("alo-installing.conf");
    let line = format!("install_items+=\" {maker} \"");
    assert!(
        list.contains(&line),
        "the list names {maker} on its own line"
    );
    for changed in [
        list.replace(&line, ""),
        list.replace(&line, &format!("# {line}")),
    ] {
        assert!(!installed_by(&changed).contains(&maker));
    }
}

/// **A list that dropped one of them is caught**, each on its own — `fstrim`
/// first, because it is the one the install of 2026-09-16 stopped at.
#[test]
fn a_list_missing_what_the_installer_finishes_with_is_caught() {
    let list = the_recipes("alo-installing.conf");
    for program in WHAT_THE_INSTALLER_FINISHES_WITH {
        let dropped = list.replace(&format!(" {program} "), " ");
        assert_ne!(dropped, list, "the list no longer names {program}");
        assert_eq!(missing_for_finishing(&dropped), vec![program]);
    }
    let commented = list.replace(
        "install_items+=\" /usr/sbin/fstrim",
        "# install_items+=\" /usr/sbin/fstrim",
    );
    assert_ne!(commented, list);
    assert_eq!(
        missing_for_finishing(&commented),
        vec!["/usr/sbin/fstrim", "/usr/bin/umount"]
    );
}

// ---------------------------------------------------------------------------
// A wireless join, which an initramfs carries only when every part is named
// ---------------------------------------------------------------------------
//
// `docs/features.md` promises *download, click, reboot, sign in* with no cable
// in it, and the audience is developers, most of whom have none. The installed
// system gained wireless in #844; the environment is the other half.
//
// **Every path here was read out of image 0.0.8 by digest on 2026-10-20**
// (`sha256:a543874e…`), unpacked and listed, rather than taken from a
// distribution's convention — and two of them are not where a convention would
// put them.

/// The programs, units, bus files, library and firmware a wireless join needs
/// aboard, at the paths they are at.
///
/// Three are worth saying out loud, because each is a way for every other part
/// to be right and the join to fail:
///
/// - **the bus policy is in `/etc`**, not `/usr/share` where the activation
///   service is. Without it the bus denies NetworkManager's call to the
///   supplicant, and the error names NetworkManager rather than this file;
/// - **the supplicant is at `/usr/bin`**, which is the rpm's own path.
///   `/usr/sbin/wpa_supplicant` resolves too, because Fedora 42 merged the two,
///   and is not the one named;
/// - **the firmware is the file and not the link.** See
///   [`THE_LINK_THAT_IS_NOT_THE_FIRMWARE`].
///
/// And one of them is a **pattern** rather than a path: NetworkManager's plugin
/// lives in a directory named for NetworkManager's version, which `dnf` chooses
/// from the live repository when the environment is built and which the base's
/// own digest therefore does not fix. A pattern that matches nothing expands to
/// itself and is silently absent, so `image/installing/Containerfile` refuses
/// one that matches nothing — see
/// [`the_recipe_holds_the_initramfs_to_its_list`].
const EVERY_WIRELESS_PART: [&str; 10] = [
    "/usr/bin/wpa_supplicant",
    "/usr/bin/wpa_cli",
    "/usr/bin/nmcli",
    "/usr/lib/systemd/system/wpa_supplicant.service",
    "/usr/share/dbus-1/system-services/fi.w1.wpa_supplicant1.service",
    "/etc/dbus-1/system.d/wpa_supplicant.conf",
    "/usr/lib64/NetworkManager/*/libnm-device-plugin-wifi.so",
    "/usr/lib/firmware/intel/iwlwifi/iwlwifi-3168-29.ucode.xz",
    "/usr/lib/firmware/regulatory.db",
    "/usr/lib/firmware/regulatory.db.p7s",
];

/// **The path that looks like the firmware and is 38 bytes.**
///
/// `/usr/lib/firmware/iwlwifi-3168-29.ucode.xz` is a symlink into
/// `intel/iwlwifi/`; the file it points at is 262,656 bytes. A list naming the
/// link puts the link in the initramfs, where it points at nothing: the path
/// exists, `test -e` passes, and the kernel cannot read the firmware. So this
/// one is checked by its **absence** — an entry that names it is the fault,
/// whether or not the real path is named beside it.
///
/// `image/Containerfile`'s own wireless proof names this path, correctly:
/// there, inside the installed image, the link resolves and `test -f` is asking
/// whether the firmware is installed at all. Here it is a different question.
const THE_LINK_THAT_IS_NOT_THE_FIRMWARE: &str = "/usr/lib/firmware/iwlwifi-3168-29.ucode.xz";

/// The kernel modules a wireless card needs, which an initramfs carries only
/// when it is told to by name — `iwlmvm` is what the certified machine's Intel
/// AC 3168 uses, `iwldvm` is Intel's older cards, and `cfg80211` and
/// `mac80211` are the stack beneath both.
const EVERY_WIRELESS_DRIVER: [&str; 5] = ["iwlwifi", "iwlmvm", "iwldvm", "cfg80211", "mac80211"];

/// Every module an initramfs list adds, one per entry.
fn drivers_added_by(list: &str) -> Vec<String> {
    list.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.strip_prefix("add_drivers+="))
        .flat_map(|drivers| {
            drivers
                .trim_matches('"')
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// What is wrong with a list's wireless, as sentences naming the part.
fn wrong_with_the_wireless(list: &str) -> Vec<String> {
    let installed = installed_by(list);
    let added = drivers_added_by(list);
    let mut wrong: Vec<String> = EVERY_WIRELESS_PART
        .iter()
        .filter(|part| !installed.iter().any(|aboard| aboard == *part))
        .map(|part| format!("the environment cannot join a network without {part}"))
        .chain(
            EVERY_WIRELESS_DRIVER
                .iter()
                .filter(|driver| !added.iter().any(|aboard| aboard == *driver))
                .map(|driver| format!("no card is found without the {driver} module")),
        )
        .collect();
    if installed
        .iter()
        .any(|aboard| aboard == THE_LINK_THAT_IS_NOT_THE_FIRMWARE)
    {
        wrong.push(format!(
            "{THE_LINK_THAT_IS_NOT_THE_FIRMWARE} is a link into intel/iwlwifi/, and an initramfs would carry the link"
        ));
    }
    wrong
}

/// **The initramfs carries everything a wireless join needs**, at the paths the
/// image has them at, and the modules by name.
#[test]
fn the_initramfs_carries_what_a_wireless_join_needs() {
    assert_eq!(
        wrong_with_the_wireless(&the_recipes("alo-installing.conf")),
        Vec::<String>::new()
    );
}

/// **A list missing one part is caught**, each on its own — and so is a list
/// that names the firmware's link instead of the firmware.
#[test]
fn a_list_that_cannot_join_a_network_is_caught() {
    let list = the_recipes("alo-installing.conf");
    for part in EVERY_WIRELESS_PART {
        let dropped = list.replace(&format!(" {part} "), " ");
        assert_ne!(dropped, list, "the list no longer names {part}");
        assert_eq!(
            wrong_with_the_wireless(&dropped),
            vec![format!(
                "the environment cannot join a network without {part}"
            )]
        );
    }
    for driver in EVERY_WIRELESS_DRIVER {
        let dropped = list.replace(&format!(" {driver} "), " ");
        assert_ne!(dropped, list, "the list no longer adds {driver}");
        assert_eq!(
            wrong_with_the_wireless(&dropped),
            vec![format!("no card is found without the {driver} module")]
        );
    }
    // The substitution a person would make reading a distribution's own
    // documentation, which names the link everywhere.
    let the_link = list.replace(
        "/usr/lib/firmware/intel/iwlwifi/iwlwifi-3168-29.ucode.xz",
        THE_LINK_THAT_IS_NOT_THE_FIRMWARE,
    );
    assert_ne!(the_link, list);
    assert_eq!(wrong_with_the_wireless(&the_link).len(), 2);
    // And naming it beside the real one is still wrong, because the initramfs
    // carries both and the link in it still points at nothing.
    let both = list.replace(
        "install_items+=\" /usr/lib/firmware/regulatory.db",
        &format!(
            "install_items+=\" {THE_LINK_THAT_IS_NOT_THE_FIRMWARE} /usr/lib/firmware/regulatory.db"
        ),
    );
    assert_ne!(both, list);
    assert_eq!(wrong_with_the_wireless(&both).len(), 1);
}

/// The environment's recipe, with its prose dropped and its continued lines
/// joined — **what the build runs, rather than what the file says.**
///
/// Both tests below were written against the whole file first, and both passed
/// a mutation that broke what they are for: one was satisfied by a comment
/// naming a package the `dnf` line no longer installed, and the other by an
/// `exit 1` belonging to a different branch three lines up. A recipe is prose
/// and program in one file, and a test that greps the file cannot tell them
/// apart.
fn what_the_recipe_runs() -> String {
    let recipe = std::fs::read_to_string(
        Path::new(alo_image::THE_IMAGE)
            .join("installing")
            .join("Containerfile"),
    )
    .expect("the environment's recipe is there");
    let program: Vec<&str> = recipe
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect();
    // A `\` at the end of a line continues it, so the shell sees one command.
    // Joining them is what lets a test say *this refusal, then this exit*.
    program
        .join("\n")
        .replace("\\\n", " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

/// **The recipe holds the initramfs it built to the list it was built from.**
///
/// This is the check that makes every other claim in `alo-installing.conf`
/// worth anything, and it is here so that deleting it fails a gate.
///
/// **`dracut` exits 0 for an `install_items` path it cannot find** — measured
/// in the pinned base on 2026-10-20, with a deliberately absent path: it
/// printed nothing a build reads, wrote the initramfs without the file, and
/// returned success. So a list every one of whose entries was wrong would build
/// green, which is how `/etc/containers/containers.conf` and `storage.conf` sat
/// in that list from the day it was written without ever being aboard: Fedora
/// ships those two in `/usr/share/containers/`, and the base has no `/etc`
/// copy at all.
///
/// Five things are asked of the recipe, each naming the way the check can be
/// true and useless:
///
/// - it reads the built **initramfs** (`lsinitrd`) rather than the machine it
///   was built on, which is the whole point;
/// - it reads the **list** rather than a copy of it, because a second copy is
///   the thing that goes stale;
/// - it looks at `install_items` **and** `add_drivers`, because an initramfs
///   with every program and no `iwlwifi` finds no card;
/// - it **refuses** what is absent, rather than printing it. This is checked as
///   the refusal and the exit **adjacent**, because the recipe exits 1
///   elsewhere and `contains("exit 1")` passed with this branch turned into a
///   `true`;
/// - it runs **after** `dracut`, which is the difference between measuring the
///   artefact and measuring what was handed to it.
#[test]
fn the_recipe_holds_the_initramfs_to_its_list() {
    let runs = what_the_recipe_runs();
    for (what, said) in [
        (
            "it does not read the initramfs it built",
            "lsinitrd /environment",
        ),
        (
            "it does not read the list the initramfs was built from",
            "list=/etc/dracut.conf.d/alo-installing.conf",
        ),
        ("it does not read the list's paths", "install_items+="),
        ("it does not read the list's modules", "add_drivers+="),
        (
            "it does not refuse a pattern that matches nothing",
            "nothing in this image matches ${pattern}\" >&2; exit 1",
        ),
        (
            "it does not refuse what the initramfs is missing",
            "does not carry what its list names:${absent}\" >&2; exit 1;",
        ),
    ] {
        assert!(
            runs.contains(said),
            "{what}: the recipe does not say {said}"
        );
    }
    let dracut = runs
        .find("dracut --no-hostonly")
        .expect("the recipe runs dracut");
    let reading = runs
        .find("lsinitrd")
        .expect("the recipe reads the initramfs");
    assert!(
        dracut < reading,
        "the recipe reads the initramfs before it builds one"
    );
}

/// **The recipe installs what the base does not have.**
///
/// The environment is built from the pinned base and installed nothing until
/// 2026-10-20, when that base was unpacked and listed — 145,474 paths, with
/// `/usr/bin/nm-online` as the control, which this environment already runs and
/// which is there. Against that control the base has **no** supplicant, no
/// NetworkManager Wi-Fi plugin, no `iwlwifi` firmware and no regulatory
/// database; it has `nmcli`, `dbus-broker` and all five wireless modules.
///
/// So naming those parts in the list is not enough on its own, and this test is
/// the one that fails if the install is dropped while the list keeps them —
/// which would otherwise build green and ship an initramfs that cannot join a
/// network, because `dracut` does not mind a path that is not there.
///
/// Each package is looked for in **what the recipe runs**, not in the recipe:
/// `wireless-regdb` was removed from the `dnf` line and this test passed,
/// because the comment above that line explains why it is asked for by name.
#[test]
fn the_recipe_installs_the_wireless_parts_the_base_lacks() {
    let runs = what_the_recipe_runs();
    for package in [
        "NetworkManager-wifi",
        "wpa_supplicant",
        // Asked for by name rather than taken as something's dependency: a
        // card with no regulatory database falls back to the most restrictive
        // domain, and the person is told *no networks found*.
        "wireless-regdb",
        "iwlwifi-mvm-firmware",
        "iwlwifi-dvm-firmware",
    ] {
        assert!(
            runs.contains(package),
            "the environment does not install {package}, which the base does not have"
        );
    }
    // Run rather than found, which is this repository's own recorded
    // substitution: a binary that cannot resolve its libraries passes `test -x`.
    assert!(
        runs.contains("wpa_supplicant -v"),
        "the recipe finds the supplicant without starting it"
    );
}
