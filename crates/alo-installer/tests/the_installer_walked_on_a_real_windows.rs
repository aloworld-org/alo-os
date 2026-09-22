//! The installer, walked on a real Windows in a virtual machine and killed at
//! every step.
//!
//! The installer plan's task 10. `crates/alo-installer` is tested against a
//! scripted Windows everywhere else in this crate, and a scripted Windows
//! cannot show three things this file exists for:
//!
//! - that the storage cmdlets and `bcdedit` do what `crate::program` asks of
//!   them on a Windows that is really running;
//! - that the names `crate::naming` makes are the names the environment finds
//!   under `/dev/disk/by-id/`;
//! - that a copy of `{bootmgr}` with a `device` and a `path` is an entry the
//!   firmware starts.
//!
//! # The machine
//!
//! QEMU's `q35` with OVMF, not Hyper-V, because the account the tests run under
//! on the development PC cannot manage Hyper-V; **Secure Boot off**, because
//! the shipped installer refuses to run with it on (ADR 0033 §4) — both said in
//! full in [`walking`]. Nothing here touches this computer's own disks: every
//! destructive step is inside a machine the test made, whose disks are files
//! under [`walking::needs::THE_YARD`].
//!
//! # How each thing is known
//!
//! - **Windows reached its desktop session**: the guest's session table shows
//!   `alo` *Active* on the console and `explorer.exe` running in that session,
//!   printed on its serial line by a task that runs at sign-in — never the
//!   machine merely still running.
//! - **The installer was killed after a step**: for steps 1–3, frozen while the
//!   step's own program finishes; for steps 4–7, caught as it asks Windows to
//!   start the next step's first program, which Windows' Image File Execution
//!   Options replace with a stand-in for that one run — nothing in the
//!   installer knows. Then where it landed is read from Windows' own tools and
//!   the firmware's variables, never assumed.
//! - **The Windows partition's files are what they were**: every file of the
//!   partition, and of the partition the firmware starts Windows from, hashed
//!   from the host with the machine off ([`walking::reading`]), and held to
//!   **three controls**, each started the same number of times: the same
//!   Windows with the installer never run; with the installer run the same way
//!   and refused at the consent; and with it killed at the consent. A running
//!   Windows rewrites some of its own files on every start, and running any
//!   program through Windows' own tools changes more of them (measured: the WMI
//!   repository, Defender's scan history, the TPM's key cache) — so *identical
//!   to the installed image* is not a claim any start could meet. What is
//!   claimed is that **staging changed nothing on Windows' partition that
//!   running the installer and stopping it at the consent did not also
//!   change**, outside the directories in which the controls disagree with each
//!   other; and it is measured, never written down by hand. **The test asserts
//!   that for the start partition only.** For the Windows partition the run of
//!   2026-09-22 found changes no control explains yet, under the user's
//!   profile, `ProgramData` and `Windows\`; the test prints them and leaves the
//!   claim to the installer plan's task 19.
//!
//! # Run by name, never in the suite
//!
//! Each of these starts virtual machines and writes tens of gigabytes. They are
//! `#[ignore]`d, and a host missing anything they need **fails** with the list
//! rather than passing quietly. Run
//! [`a_windows_installs_itself_and_reaches_a_desktop_session`] first; the others
//! are overlays of the Windows it leaves.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod walking;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use walking::console::{self, Console};
use walking::machine::{Machine, SecurityChip};
use walking::medium::{self, Told};
use walking::reading::{Changed, Noise, Reading};
use walking::{download, needs};

/// One virtual machine at a time: every test here shares a directory, a
/// security chip and a monitor socket.
static ONE_MACHINE: Mutex<()> = Mutex::new(());

/// The pristine Windows every walk is an overlay of.
const THE_INSTALLED_WINDOWS: &str = "windows.qcow2";

/// The firmware's variables as they were when Windows had just been installed.
const THE_FIRMWARES_VARIABLES: &str = "VARS.fd.base";

/// How long an unattended install of Windows 11 is given. Measured on
/// 2026-09-21 under KVM: 32 minutes from the first key to the desktop.
const AN_INSTALL: Duration = Duration::from_secs(3600);

/// How long a boot is given to reach its sign-in before it is reset.
///
/// Measured on 2026-09-21: this guest sometimes stops at Windows' start-up
/// spinner with no disk writes and nothing on the serial line — **in the
/// control, where the installer never ran** — and one reset brings it up. Every
/// reset is counted and printed.
const A_SIGN_IN: Duration = Duration::from_secs(480);

/// How long a boot is given to finish what it was told, after its sign-in.
const A_WALK: Duration = Duration::from_secs(1200);

/// The seven steps of `crate::staging`, each named as the plan names it.
const THE_SEVEN_STEPS: [(u8, &str); 7] = [
    (1, "after the shrink"),
    (2, "after the area is made"),
    (3, "after it is prepared"),
    (4, "after the copy"),
    (5, "after the entry"),
    (6, "after its letter is taken"),
    (7, "after the next start is set"),
];

/// The name udev gives the walk's empty second disk: `ata-`, the model QEMU
/// reports, and the serial the machine gives it.
const THE_SECOND_DISKS_NAME: &str = "ata-QEMU_HARDDISK_ALOTARGET1";

// ---------------------------------------------------------------------------
// The Windows the walk is measured in
// ---------------------------------------------------------------------------

/// **A Windows 11 installs itself into the machine and reaches a desktop
/// session**, with no person at the keyboard and nothing typed by hand.
#[test]
#[ignore = "installs a Windows into a virtual machine; run by name"]
fn a_windows_installs_itself_and_reaches_a_desktop_session() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    walking::machine::stop();

    let windows = fetched_windows(&yard);
    let answer = medium::the_answer_disc(&yard);
    let chip = SecurityChip::fresh(&yard);
    for (disk, size) in [
        (THE_INSTALLED_WINDOWS, walking::machine::THE_WINDOWS_DISK),
        ("target.qcow2", walking::machine::THE_SECOND_DISK),
    ] {
        let _ = std::fs::remove_file(yard.join(disk));
        walking::machine::run(
            "qemu-img",
            &[
                "create",
                "-f",
                "qcow2",
                &yard.join(disk).display().to_string(),
                size,
            ],
        );
    }
    std::fs::copy(needs::THE_BLANK_VARIABLES, yard.join("VARS.fd"))
        .expect("the firmware's variables");

    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start(&yard, "", Some(&answer), Some(&windows), &console, &chip);
    // The media's *press any key to boot from CD* times out in about five
    // seconds, and the firmware then waits on the network for ever.
    machine.hold_a_key_down(25);

    // The sign-in's whole account, not its first line: measured on
    // 2026-09-22, judging the session on `ALOWALK-BEGIN` read the table before
    // the guest had printed the shell's own line.
    let said = console.wait_for(&[console::SESSION_SAID], AN_INSTALL);
    assert!(
        said.is_some(),
        "the install never reached a sign-in. The screen was kept at {} and the \
         serial line said:\n{}",
        machine.screen("the-install-stopped").display(),
        console.said()
    );
    a_desktop_session(&console);
    assert!(
        machine.shut_down(Duration::from_secs(300)),
        "the installed Windows would not shut down"
    );
    drop(machine);

    // Settled once, before it becomes the base. Measured on 2026-09-21: an
    // overlay of an unsettled Windows spent about 1 000 s on its first start
    // (the first PowerShell alone took four minutes), and a restarted one
    // 100–160 s; and fast startup is switched off so that a shutdown leaves
    // NTFS as a reader on the host can trust.
    let settle = medium::the_settle_disc(&yard);
    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start(&yard, "", Some(&settle), None, &console, &chip);
    let settled = machine.has_stopped_within(Duration::from_secs(2400));
    assert!(
        settled && console.contains("ALOWALK-SETTLE-SHUTDOWN"),
        "the installed Windows did not settle and shut itself down. The screen is \
         at {} and the serial line said:\n{}",
        machine.screen("the-settle-stopped").display(),
        console.said()
    );
    drop(machine);
    std::fs::copy(yard.join("VARS.fd"), yard.join(THE_FIRMWARES_VARIABLES))
        .expect("keeping the firmware's variables");
}

// ---------------------------------------------------------------------------
// The seven kills, read against the control
// ---------------------------------------------------------------------------

/// **Killed at each of `staging.rs`'s seven steps, the kill lands exactly on
/// the step, and the installer leaves a Windows that starts to its desktop
/// session, with nothing of its start partition changed beyond what the
/// controls change.** The Windows partition's own byte-for-byte claim is the
/// plan's task 19 (see [`beyond_the_controls`]).
///
/// For each step: a fresh overlay of the installed Windows; a boot that runs
/// the installer, types the name the installer itself showed, and kills it the
/// moment the step's effect is there; a reading of the whole disk with the
/// machine off; a restart; and a second reading. The first reading is held to
/// the control's first, the second to the control's second.
///
/// **Step 7 is the one step that changes what starts**, and `staging.rs` says
/// so: the next start is the last thing done before the restart, after the
/// person agreed to exactly that. So after the seventh kill the first restart
/// is *watched* — the firmware starts the entry once — and the restart after
/// that is the one held to *Windows starts to its desktop*.
#[test]
#[ignore = "starts over twenty virtual machines; run by name"]
fn killed_at_every_step_the_computer_still_starts_windows() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);
    let base = Reading::of(&yard.join(THE_INSTALLED_WINDOWS), &yard);

    // The first control: the same Windows, started twice, the installer never
    // run.
    a_fresh_machine(&yard, "control");
    let chip = SecurityChip::fresh(&yard);
    let first = one_boot(&yard, "control", &Told::JustLook, &chip, &download);
    a_desktop_session(&first.console);
    let second = one_boot(&yard, "control", &Told::JustLook, &chip, &download);
    a_desktop_session(&second.console);
    forget(&yard, "control");

    // The second control: the installer run the same way, refused at the
    // consent — every read it makes and every side effect running it has, and
    // no change. Measured on 2026-09-21 that this is needed: against the first
    // control alone, a kill at step 1 left 32 paths changed outside the noise,
    // every one of them Windows' own bookkeeping of a program having run.
    a_fresh_machine(&yard, "refused");
    let chip = SecurityChip::fresh(&yard);
    let refused = one_boot(
        &yard,
        "refused",
        &Told::RefuseAtTheConsent,
        &chip,
        &download,
    );
    assert!(
        refused.console.contains("THE WINDOWS FILES ARE UNCHANGED")
            && refused.reading.table == base.table,
        "the installer refused at the consent changed something.\n{}",
        refused.console.said()
    );
    let refused_restarted = one_boot(&yard, "refused", &Told::JustLook, &chip, &download);
    a_desktop_session(&refused_restarted.console);
    forget(&yard, "refused");

    // The third control: the installer run the same way and killed at the
    // consent, after as long there as a kill run spends after it — everything
    // a kill run does to Windows except staging. Measured on 2026-09-21 that
    // the second is not enough: kills at steps 1, 5 and 7 left per-user shell
    // caches and error reporting's temporary files that the refusal did not.
    a_fresh_machine(&yard, "killed");
    let chip = SecurityChip::fresh(&yard);
    let killed_there = one_boot(&yard, "killed", &Told::KillAtTheConsent, &chip, &download);
    assert!(
        killed_there
            .console
            .contains("THE WINDOWS FILES ARE UNCHANGED")
            && killed_there.reading.table == base.table,
        "the installer killed at the consent changed something.\n{}",
        killed_there.console.said()
    );
    let killed_restarted = one_boot(&yard, "killed", &Told::JustLook, &chip, &download);
    a_desktop_session(&killed_restarted.console);
    forget(&yard, "killed");

    let control_once = base
        .changed_to(&refused.reading)
        .and(&base.changed_to(&killed_there.reading));
    let control_twice = base
        .changed_to(&refused_restarted.reading)
        .and(&base.changed_to(&killed_restarted.reading));
    let noise = Noise::between(&first.reading, &second.reading)
        .and(&Noise::between(&first.reading, &refused.reading))
        .and(&Noise::between(&second.reading, &refused_restarted.reading))
        .and(&Noise::between(&refused.reading, &killed_there.reading))
        .and(&Noise::between(
            &refused_restarted.reading,
            &killed_restarted.reading,
        ));
    eprintln!(
        "the installer refused or killed at the consent changed {} paths once \
         and {} twice; the controls disagree in {} directories; unread in the \
         installed Windows, and so not compared: {:?}",
        control_once.windows.len() + control_once.start_partition.len(),
        control_twice.windows.len() + control_twice.start_partition.len(),
        noise.windows.len() + noise.start_partition.len(),
        base.unread()
    );

    // Every step is walked whatever an earlier one found, and the test fails
    // once, at the end, with every finding: a run that takes a night must not
    // stop at its first.
    let mut findings: Vec<String> = Vec::new();
    for (step, called) in THE_SEVEN_STEPS {
        let name = format!("step{step}");
        a_fresh_machine(&yard, &name);
        let chip = SecurityChip::fresh(&yard);

        let killed = one_boot(&yard, &name, &Told::KillAfterStep(step), &chip, &download);
        if !killed.console.contains(&format!("killed at step {step}")) {
            findings.push(format!(
                "step {step} ({called}) was never reached, so nothing was killed there"
            ));
        }
        // Where the kill landed is read from Windows' own tools afterwards —
        // this step's effect there, the next one's not. Measured on 2026-09-21:
        // a kill aimed at step 4 by polling alone landed after step 6.
        if !killed
            .console
            .contains(&format!("LANDED EXACTLY after step {step}"))
        {
            findings.push(format!("the kill aimed {called} did not land there"));
        }
        // From step 5 the installer writes the start-up entry, and the entry
        // lives in the start partition's `BCD`; so there, and only there, a
        // change around the kill is the step itself.
        let changed_around_the_kill = bracket_changes(&killed.console);
        let allowed = |path: &String| {
            step >= 5
                && (path.ends_with("\\efi\\microsoft\\boot\\bcd")
                    || path.ends_with("\\efi\\microsoft\\boot\\bcd.log"))
        };
        if !changed_around_the_kill.iter().all(allowed) {
            findings.push(format!(
                "around the kill at step {step} ({called}), the files Windows starts \
                 from changed while it ran: {changed_around_the_kill:?}"
            ));
        }
        findings.extend(beyond_the_controls(
            &base
                .changed_to(&killed.reading)
                .beyond(&control_once)
                .outside(&noise),
            &format!("after the kill at step {step} ({called})"),
        ));

        if step == 7 {
            // The firmware's own line says which entry it started, and from
            // which partition. Measured on 2026-09-21: it started the alo OS
            // entry — and from Windows' own EFI system partition, not the area
            // (`docs/quirks.md`), so Windows came up. This holds it to the area.
            // The area is read off the killed boot's console before the
            // restart, which starts its console afresh in the same file.
            let area = the_areas_first_sector(&killed.console);
            let next = the_restart_after_the_next_start_was_set(&yard, &name, &chip);
            let started = walking::firmware::starts(&next);
            match started.iter().find(|start| start.description == "alo OS") {
                None => findings.push(
                    "after the kill at step 7 the firmware did not start the alo OS entry"
                        .to_owned(),
                ),
                Some(alo) if alo.first_sector != area => findings.push(format!(
                    "after the kill at step 7 the firmware started the alo OS entry from \
                     partition {:?} at sector {:?}, and the area begins at sector {area:?}",
                    alo.partition, alo.first_sector
                )),
                Some(_) => {}
            }
        }
        let restarted = one_boot(&yard, &name, &Told::JustLook, &chip, &download);
        a_desktop_session(&restarted.console);
        findings.extend(beyond_the_controls(
            &base
                .changed_to(&restarted.reading)
                .beyond(&control_twice)
                .outside(&noise),
            &format!("after the restart that followed the kill at step {step} ({called})"),
        ));
        eprintln!(
            "step {step} ({called}) walked; findings so far: {}",
            findings.len()
        );
        forget(&yard, &name);
    }
    forget(&yard, "control");
    assert!(
        findings.is_empty(),
        "the walk found {} things:\n  {}",
        findings.len(),
        findings.join("\n  ")
    );
}

// ---------------------------------------------------------------------------
// The road, run once
// ---------------------------------------------------------------------------

/// **Run all the way through, the firmware starts the environment on the next
/// restart, and the environment finds the disk by the name the installer
/// wrote.**
#[test]
#[ignore = "starts virtual machines and installs; run by name"]
fn the_whole_road_starts_the_environment_on_the_next_restart() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);

    // The name the installer writes is read where it writes it, at step 4,
    // while the area still has a letter.
    a_fresh_machine(&yard, "road");
    let chip = SecurityChip::fresh(&yard);
    let copied = one_boot(&yard, "road", &Told::KillAfterStep(4), &chip, &download);
    let written = copied
        .console
        .after("the-name-written: ")
        .expect("the name the installer wrote into the area");
    assert_eq!(written.trim(), THE_SECOND_DISKS_NAME);
    forget(&yard, "road");

    // The road itself, on the firmware that can start the base's shim.
    a_fresh_machine(&yard, "road");
    let chip = SecurityChip::fresh(&yard);
    let firmware = walking::firmware::fedoras(&yard);
    let disc = medium::the_walk_disc(&yard, &Told::TheWholeRoad, &download);
    let console = Console::fresh(&yard.join("console.log"));
    let machine = Machine::start_on(&firmware, &yard, "road", Some(&disc), None, &console, &chip);
    // The installer restarts the computer itself, and nothing here says what
    // starts next: the firmware's own variables decide.
    let reached = console.wait_for(
        &[
            "Checking that ata-QEMU_HARDDISK_ALOTARGET1 is safe to install onto",
            "is not connected to this computer",
        ],
        A_WALK + A_SIGN_IN,
    );
    let screen = machine.screen("the-road-next-restart");
    let said = console.said();
    drop(machine);
    forget(&yard, "road");

    let area = the_areas_first_sector(&console);
    let alo = walking::firmware::starts(&said)
        .into_iter()
        .find(|start| start.description == "alo OS")
        .unwrap_or_else(|| {
            panic!(
                "the firmware did not start the alo OS entry on the installer's own \
                 restart. The screen is at {}.\n{said}",
                screen.display()
            )
        });
    assert_eq!(
        (alo.first_sector, alo.file.as_str()),
        (area, "\\EFI\\BOOT\\BOOTX64.EFI"),
        "the firmware started the alo OS entry from somewhere other than the \
         area's loader.\n{said}"
    );
    assert!(
        reached.is_some(),
        "the environment never started. The screen is at {}.\n{said}",
        screen.display()
    );
    assert!(
        said.contains(&format!("alo.installing.to={THE_SECOND_DISKS_NAME}")),
        "the environment was not handed the name the installer wrote.\n{said}"
    );
    // The environment's own sentences: it looked for the name, found it under
    // /dev/disk/by-id/, and went on to check it — which it cannot do for a
    // disk that is not there. Measured on 2026-09-21; what it did after that
    // (the install itself) is task 12's.
    assert!(
        said.contains(&format!(
            "Looking for the disk you chose: {THE_SECOND_DISKS_NAME}"
        )) && said.contains(&format!(
            "Checking that {THE_SECOND_DISKS_NAME} is safe to install onto"
        )) && !said.contains("is not connected to this computer"),
        "the environment did not find {THE_SECOND_DISKS_NAME} under /dev/disk/by-id/.\n{said}"
    );
}

/// The first sector of the installer's area, from the state the guest printed:
/// the partition labelled as the installer labels it.
fn the_areas_first_sector(console: &Console) -> Option<u64> {
    console.said().lines().find_map(|line| {
        if !line.contains(&format!("label=[{}]", alo_installing::THIS_INSTALLER)) {
            return None;
        }
        let offset = line.split("offset=").nth(1)?.split_whitespace().next()?;
        offset.parse::<u64>().ok().map(|bytes| bytes / 512)
    })
}

// ---------------------------------------------------------------------------
// The pieces every test above is made of
// ---------------------------------------------------------------------------

/// What one boot left: what it said, and every file of its disk after it.
struct Booted {
    /// The serial line.
    console: Console,
    /// The disk, read from the host with the machine off.
    reading: Reading,
}

/// One boot of a machine, told one thing, read back off its serial line and
/// then off its disk.
fn one_boot(yard: &Path, name: &str, told: &Told, chip: &SecurityChip, download: &Path) -> Booted {
    walking::machine::stop();
    let disc = medium::the_walk_disc(yard, told, download);
    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start(yard, name, Some(&disc), None, &console, chip);
    let mut resets = 0;
    while console.wait_for(&[console::BEGINS], A_SIGN_IN).is_none() {
        resets += 1;
        assert!(
            resets <= 2,
            "{name} never reached a sign-in, after {resets} resets. The screen is \
             at {} and the serial line said:\n{}",
            machine.screen(&format!("{name}-never-signed-in")).display(),
            console.said()
        );
        eprintln!(
            "{name}: no sign-in after {} s; reset {resets}",
            A_SIGN_IN.as_secs()
        );
        machine.reset();
    }
    let said = console.wait_for(&[console::DONE, console::NOTHING_TO_DO], A_WALK);
    assert_eq!(
        said.as_deref(),
        Some(console::DONE),
        "the boot of {name} never finished what it was told. The screen is at {} \
         and the serial line said:\n{}",
        machine.screen(&format!("{name}-unfinished")).display(),
        console.said()
    );
    let shut = machine.shut_down(Duration::from_secs(180));
    eprintln!(
        "{name}: {} after its walk",
        if shut { "shut down" } else { "cut off" }
    );
    drop(machine);
    Booted {
        console,
        reading: Reading::of(&Machine::windows_of(yard, name), yard),
    }
}

/// Watch what the firmware starts on the restart after the next start was
/// set, without telling it anything, and give back what it said.
fn the_restart_after_the_next_start_was_set(
    yard: &Path,
    name: &str,
    chip: &SecurityChip,
) -> String {
    walking::machine::stop();
    let console = Console::fresh(&yard.join("console.log"));
    let machine = Machine::start(yard, name, None, None, &console, chip);
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(420) {
        if console.contains("alo-installing") || console.contains(console::BEGINS) {
            break;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    let _ = machine.screen(&format!("{name}-first-restart"));
    drop(machine);
    walking::machine::stop();
    console.said()
}

/// Every path the guest's own reading of the files Windows starts from found
/// changed across the installer's run — nothing, when it said they were not.
///
/// # Panics
/// When the guest said neither.
fn bracket_changes(console: &Console) -> Vec<String> {
    let said = console.said();
    if said.contains("THE WINDOWS FILES ARE UNCHANGED") {
        return Vec::new();
    }
    assert!(
        said.contains("THE WINDOWS FILES CHANGED"),
        "the guest never compared the files Windows starts from.\n{said}"
    );
    said.lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.split_once("s ").map_or(line, |(_, rest)| rest);
            (rest.starts_with("<= ") || rest.starts_with("=> "))
                .then(|| rest.rsplit(' ').next().unwrap_or_default().to_owned())
        })
        .collect()
}

/// Every path of the start partition that changed beyond what the controls
/// change, as findings; the Windows partition's, printed and not judged.
///
/// **The Windows partition is not claimed byte for byte here.** The run of
/// 2026-09-22 found paths there after every kill that no control explains —
/// per-user caches, and files under `ProgramData` and `Windows\` (Defender's
/// scan history, a WMI performance file) — each of a kind the base and the
/// controls also hold. Whether the kills or Windows' own schedule wrote them is
/// the installer plan's task 19, which owns that claim and the run that decides
/// it. They are printed for it in full, and nothing is waved away by a rule.
fn beyond_the_controls(beyond: &Changed, when: &str) -> Vec<String> {
    if !beyond.windows.is_empty() {
        eprintln!(
            "{when}, the Windows partition changed beyond the controls in {} paths \
             (the plan's task 19): {:?}",
            beyond.windows.len(),
            beyond.windows
        );
    }
    if beyond.start_partition.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "{when}, the start partition changed beyond what the controls change: {:?}",
        beyond.start_partition
    )]
}

/// The one machine, whichever test holds it — and still the next test's when
/// an earlier one panicked holding it: each test here stands on its own, and a
/// poisoned lock would otherwise fail every test after the first failure
/// without running it (measured on 2026-09-22).
fn one_machine_at_a_time() -> std::sync::MutexGuard<'static, ()> {
    ONE_MACHINE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Fail with the list of what is missing, rather than pass without running.
fn the_host_has_what_this_needs() {
    let mut missing = needs::what_is_missing();
    match needs::room_in_gib() {
        Some(room) if room >= needs::ROOM_A_WALK_NEEDS => {}
        Some(room) => missing.push(format!(
            "room on the host's own disk: {room} GB, and a walk needs {}",
            needs::ROOM_A_WALK_NEEDS
        )),
        None => missing.push("a reading of the host's own free space".to_owned()),
    }
    assert!(
        missing.is_empty(),
        "this machine cannot walk the installer. It is missing:\n  {}",
        missing.join("\n  ")
    );
}

/// The Windows media, fetched once and kept.
fn fetched_windows(yard: &Path) -> PathBuf {
    let iso = yard.join("win11.iso");
    let there = std::fs::metadata(&iso).map(|it| it.len()).unwrap_or(0);
    if there != needs::THE_WINDOWS_IS {
        walking::machine::run(
            "curl",
            &[
                "-sL",
                "--retry",
                "3",
                "-o",
                &iso.display().to_string(),
                needs::THE_WINDOWS,
            ],
        );
    }
    let fetched = std::fs::metadata(&iso).map(|it| it.len()).unwrap_or(0);
    assert_eq!(
        fetched,
        needs::THE_WINDOWS_IS,
        "the Windows media is {fetched} bytes and should be {}",
        needs::THE_WINDOWS_IS
    );
    iso
}

/// The download a person would have, built from this checkout — once per run
/// of these tests, which the kill test and the road test then share.
fn the_download(yard: &Path) -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .expect("the repository")
                .to_path_buf();
            let target_directory = std::env::var_os("CARGO_TARGET_DIR")
                .map_or_else(|| repository.join("target"), PathBuf::from);
            download::assembled(yard, &repository, &target_directory)
        })
        .clone()
}

/// An overlay of the installed Windows, an empty second disk, and the
/// firmware's variables as they were.
fn a_fresh_machine(yard: &Path, name: &str) {
    Machine::fresh(
        yard,
        name,
        &yard.join(THE_INSTALLED_WINDOWS),
        &yard.join(THE_FIRMWARES_VARIABLES),
    );
}

/// Throw a machine's disks away: the host's disk is the scarce thing here.
fn forget(yard: &Path, name: &str) {
    for disk in [
        Machine::windows_of(yard, name),
        Machine::second_of(yard, name),
    ] {
        let _ = std::fs::remove_file(disk);
    }
}

/// **That the guest reached an interactive desktop session**, from the session
/// table and the shell's own process — never from the machine still running.
fn a_desktop_session(console: &Console) {
    let said = console.said();
    let active = said
        .lines()
        .any(|line| line.contains("console") && line.contains("alo") && line.contains("Active"));
    assert!(
        active,
        "no session of alo's was active on the console, so this Windows did not \
         reach a desktop session.\n{said}"
    );
    let shell = said
        .lines()
        .any(|line| line.starts_with("explorer.exe") && line.contains("Console"));
    assert!(
        shell,
        "no shell was running in the console session, so this Windows did not \
         reach a desktop session.\n{said}"
    );
}
