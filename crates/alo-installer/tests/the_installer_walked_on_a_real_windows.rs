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
use walking::{damaging, download, needs};

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

/// The second base: the settled Windows with hibernation and Fast Startup on.
///
/// The base every other walk uses has Fast Startup **off** — a shutdown with
/// it on hibernates the Windows volume, and every reading of a disk here is
/// taken from the host with the machine off. So the question the installer
/// asks about Fast Startup cannot be reached from that base at all, and this
/// is a second base beside it: an overlay of the first, which is kept, with
/// the setting turned on and the machine stopped once the firmware starts
/// again — a restart closes the volume whatever Fast Startup says.
///
/// Made once and kept; a run that has it uses it.
const THE_WINDOWS_WITH_FAST_STARTUP_ON: &str = "windows-fast-startup-on.qcow2";

/// That base, made from the settled one if it is not there yet.
fn the_windows_with_fast_startup_on(yard: &Path, chip: &SecurityChip) -> PathBuf {
    let variant = yard.join(THE_WINDOWS_WITH_FAST_STARTUP_ON);
    if variant.exists() {
        return variant;
    }
    Machine::fresh(
        yard,
        "variant",
        &yard.join(THE_INSTALLED_WINDOWS),
        &yard.join(THE_FIRMWARES_VARIABLES),
    );
    let disc = medium::the_fast_startup_disc(yard);
    let console = Console::fresh(&yard.join("console.log"));
    let machine = Machine::start(yard, "variant", Some(&disc), None, &console, chip);
    let turned = console.wait_for(&["ALOWALK-VARIANT-DONE"], A_SIGN_IN + A_WALK);
    assert!(
        turned.is_some(),
        "Fast Startup was never turned on. The screen is at {} and the serial line \
         said:\n{}",
        machine.screen("the-variant-stopped").display(),
        console.said()
    );
    // Stopped as the firmware starts again: by then Windows has closed the
    // volume, and nothing of the second base is a session left hibernated.
    let restarted = console.wait_for(&["BdsDxe: starting"], Duration::from_secs(300));
    assert!(
        restarted.is_some(),
        "the machine never restarted after turning Fast Startup on.\n{}",
        console.said()
    );
    drop(machine);
    walking::machine::stop();
    // Kept: the next boot's console writes over this one, and what this boot
    // said about hibernation is the reason the second base is what it is.
    let _ = std::fs::copy(
        yard.join("console.log"),
        yard.join("the-second-base-was-made.log"),
    );
    std::fs::rename(Machine::windows_of(yard, "variant"), &variant)
        .expect("keeping the second base");
    forget(yard, "variant");
    variant
}

/// **On a Windows whose Fast Startup is on, the installer asks the owner's
/// question, and *turn off* turns it off** — read back from Windows' own value,
/// on a real Windows rather than a scripted one (ADR 0064 term 9).
#[test]
#[ignore = "starts virtual machines and installs; run by name"]
fn a_windows_with_fast_startup_on_is_asked_about_and_turned_off() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);
    let chip = SecurityChip::fresh(&yard);
    let base = the_windows_with_fast_startup_on(&yard, &chip);

    Machine::fresh(
        &yard,
        "fast-startup",
        &base,
        &yard.join(THE_FIRMWARES_VARIABLES),
    );
    let chip = SecurityChip::fresh(&yard);
    // Stopped as staging's first sentence about the disk appears: the question
    // is asked and answered before anything on a disk changes, and what the
    // answer did is read from Windows' own value.
    let answered = one_boot(
        &yard,
        "fast-startup",
        &Told::AnsweringFastStartup {
            answer: alo_installer::ANSWER_TURN_OFF.says().to_owned(),
        },
        &chip,
        &download,
    );
    let said = answered.console.said();
    forget(&yard, "fast-startup");

    let asked = alo_installer::ASK_FAST_STARTUP.says();
    assert!(
        said.contains(asked),
        "the installer did not ask about Fast Startup on a computer that has it on.\n{said}"
    );
    assert!(
        said.contains("fast-startup: HiberbootEnabled=[1]"),
        "this Windows' Fast Startup was not on before the installer ran.\n{said}"
    );
    assert!(
        said.contains("fast-startup: HiberbootEnabled=[0]"),
        "the installer did not turn Fast Startup off after the person said to.\n{said}"
    );
    // What the *installer* said, not what the walk did: the walk turns Fast
    // Startup on for this boot with Windows' own tool, and that tool's name on
    // the console is the walk's line and not the installer's.
    let the_installers_own: String = said
        .lines()
        .filter(|line| line.contains("installer: "))
        .collect();
    assert!(
        !the_installers_own.to_lowercase().contains("powercfg"),
        "the installer named powercfg, which removes hibernation altogether.\n{said}"
    );
    // **This boot is the whole claim.** Windows signed in — the session table
    // it printed says so — ran the walk, and answered the question, and the
    // value was read back from Windows' own registry afterwards. The shell's
    // own line is not waited for here, because this machine's first start is
    // its first ever and `explorer.exe` was not up when the table was printed;
    // that a Windows restarts to a *desktop session* after a kill is
    // `killed_at_every_step_the_computer_still_starts_windows`, on the base
    // that has Fast Startup off, and is not claimed twice.
    assert!(
        said.lines().any(|line| {
            line.contains("console") && line.contains("alo") && line.contains("Active")
        }),
        "no session of alo's was active on the console.\n{said}"
    );
}

/// **The way back in works from inside Windows**: the installer leaves a copy
/// of itself and a shortcut, the copy sets the firmware's next start to alo OS
/// and restarts, the firmware starts alo OS — and the start after that is
/// Windows again, because the default was never touched.
///
/// The installer is killed after its sixth step, so the entry is written and no
/// next start is set: what sets the next start here is the switch and nothing
/// else.
#[test]
#[ignore = "starts virtual machines and installs; run by name"]
fn the_way_back_into_alo_os_is_offered_from_inside_windows() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);
    a_fresh_machine(&yard, "switch");
    let chip = SecurityChip::fresh(&yard);

    let told = Told::SwitchingIntoAloOs {
        left_at: alo_installer::THE_PROGRAMS_HOME.to_owned(),
        left_as: alo_installer::THE_PROGRAMS_NAME.to_owned(),
        shortcut: alo_installer::THE_SHORTCUT.to_owned(),
        argument: alo_installer::THE_SWITCHS_WORD.to_owned(),
        agree: alo_installer::SWITCH_AGREED.says().to_owned(),
    };
    // The machine restarts itself at the end of this boot, so it is watched
    // rather than waited on: what the firmware starts next is the measurement.
    walking::machine::stop();
    let disc = medium::the_walk_disc(&yard, &told, &download);
    let console = Console::fresh(&yard.join("console.log"));
    // On the firmware that can start what the entry points at. Measured on
    // 2026-09-23: on the walk's own OVMF the environment's shim page-faults
    // (task 9's firmware fault), and a start that faults leaves the one-time
    // choice unconsumed — so every restart after it went to alo OS again and
    // the computer never came back to Windows. That is the firmware's fault
    // and not the switch's, and this test is about the switch.
    let firmware = walking::firmware::fedoras(&yard);
    let machine = Machine::start_on(
        &firmware,
        &yard,
        "switch",
        Some(&disc),
        None,
        &console,
        &chip,
    );
    let switched = console.wait_for(&["ALOWALK-DONE switch"], A_SIGN_IN + A_WALK);
    // What the firmware starts *after* the switch, and never a line from
    // before it: this console already holds the start that brought Windows up.
    let until = Instant::now() + Duration::from_secs(300);
    let mut started = false;
    while Instant::now() < until && !started {
        started = console
            .said()
            .split_once("ALOWALK-DONE switch")
            .is_some_and(|(_, after)| after.contains("BdsDxe: starting"));
        std::thread::sleep(Duration::from_secs(2));
    }
    let screen = machine.screen("the-way-back");
    let said = console.said();
    drop(machine);
    walking::machine::stop();

    assert!(
        switched.is_some(),
        "the way back never ran. The screen is at {}.\n{said}",
        screen.display()
    );
    assert!(
        said.contains("exists=True"),
        "the installer left no shortcut for it.\n{said}"
    );
    assert!(
        started,
        "the computer never restarted after the way back.\n{said}"
    );
    // Everything before the restart is one boot's account: the firmware's line
    // after it is what the switch actually did.
    let after_the_switch = said
        .split_once("ALOWALK-DONE switch")
        .map_or("", |(_, after)| after);
    let alo = walking::firmware::starts(after_the_switch)
        .into_iter()
        .find(|start| start.description == alo_installer::THE_ENTRYS_NAME);
    assert!(
        alo.is_some(),
        "the firmware did not start alo OS after the way back said it would.\n{said}"
    );

    // And the start after that is Windows: the switch set the next start only.
    // On the same firmware build as the start before it, since the machine's
    // variables are that firmware's.
    let looking = medium::the_walk_disc(&yard, &Told::JustLook, &download);
    let console = Console::fresh(&yard.join("console.log"));
    let back = Machine::start_on(
        &firmware,
        &yard,
        "switch",
        Some(&looking),
        None,
        &console,
        &chip,
    );
    let signed_in = console.wait_for(&[console::SESSION_SAID], A_SIGN_IN + A_WALK);
    let screen = back.screen("the-way-back-came-back");
    let came_back = console.said();
    drop(back);
    walking::machine::stop();
    assert!(
        signed_in.is_some(),
        "Windows did not come up after the way back. The screen is at {}.\n{came_back}",
        screen.display()
    );
    a_desktop_session(&console);
    let windows = walking::firmware::starts(&came_back)
        .into_iter()
        .any(|start| start.description == "Windows Boot Manager");
    assert!(
        windows,
        "the computer did not come back to Windows on its own, so the switch \
         changed more than the next start.\n{came_back}"
    );
    forget(&yard, "switch");
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

/// What the installed alo OS says on the serial line once it is up, then turns
/// itself off: handed over as a systemd credential in the firmware's tables,
/// which Windows and the environment never act on (nothing they start wants
/// it), so nothing on any disk is changed to make it say this.
const THE_INSTALLED_SYSTEM_SAYS: &str = "[Unit]\n\
     Description=What the installed system started from, said on the serial line for the walk\n\
     After=multi-user.target\n\
     [Service]\n\
     Type=oneshot\n\
     ExecStart=/usr/bin/echo ALO-INSTALLED-BEGIN\n\
     ExecStart=-/usr/bin/findmnt --noheadings --output SOURCE,FSTYPE /sysroot\n\
     ExecStart=-/usr/bin/sh -c 'lsblk --noheadings --inverse --output NAME,SERIAL \"$$(findmnt --noheadings --output SOURCE /sysroot | cut -d[ -f1)\"'\n\
     ExecStart=-/usr/bin/sh -c '. /usr/lib/os-release; echo \"os-release: $$NAME $$VERSION_ID\"'\n\
     ExecStart=-/usr/bin/systemctl show --property=Id,ActiveState,SubState alo-boundaryd.service alo-agentd.service\n\
     ExecStart=-/usr/sbin/efibootmgr\n\
     ExecStart=-/usr/bin/lsblk --noheadings --output NAME,LABEL,SIZE\n\
     ExecStart=/usr/bin/echo ALO-INSTALLED-END\n\
     ExecStartPost=/usr/bin/systemctl --no-block poweroff\n\
     StandardOutput=file:/dev/ttyS0\n\
     StandardError=file:/dev/ttyS0\n";

/// The English of one of the environment's sentences, with the walk's disk in
/// its gap.
fn the_environment_says(named: &str) -> String {
    alo_installing::EVERY_WORD
        .iter()
        .find(|word| word.named() == named)
        .unwrap_or_else(|| panic!("{named} is one of the environment's sentences"))
        .says()
        .replace("{disk}", THE_SECOND_DISKS_NAME)
}

/// The serial line without the kernel's own messages, which arrive in the
/// middle of the environment's sentences: measured on 2026-09-22,
/// `alo OS is being installed on thi[   10.517276] e1000e …` and the rest of the
/// sentence on the line after. Each message is cut from its stamp through the
/// end of its line.
fn without_the_kernels_messages(said: &str) -> String {
    let mut kept = String::with_capacity(said.len());
    let mut rest = said;
    while let Some(at) = rest.find('[') {
        let (before, from) = rest.split_at(at);
        kept.push_str(before);
        let stamp = from
            .get(1..)
            .and_then(|inside| inside.split_once(']'))
            .map(|(inside, _)| inside.trim_start());
        let is_the_kernels = stamp.is_some_and(|stamp| {
            stamp.split_once('.').is_some_and(|(seconds, micros)| {
                !seconds.is_empty()
                    && seconds.chars().all(|c| c.is_ascii_digit())
                    && micros.len() == 6
                    && micros.chars().all(|c| c.is_ascii_digit())
            })
        });
        if is_the_kernels {
            rest = from.split_once('\n').map_or("", |(_, after)| after);
        } else {
            kept.push('[');
            rest = from.get(1..).unwrap_or_default();
        }
    }
    kept.push_str(rest);
    kept
}

/// **The kernel's messages are cut from the middle of a sentence, and nothing
/// else is**: the line measured on 2026-09-22, and a bracket that is not a
/// kernel's stamp.
#[test]
fn a_sentence_the_kernel_broke_is_read_whole() {
    let said = "alo OS is being installed on thi[   10.517276] e1000e 0000:00:03.0: \
                Interrupt Throttling Rate (ints/sec) set to dynamic conservative mode\r\n\
                s computer. Each step is written here as it happens\r\n\
                [  OK  ] Reached target alo-installing.target\r\n";
    assert_eq!(
        without_the_kernels_messages(said),
        "alo OS is being installed on this computer. Each step is written here as it \
         happens\r\n[  OK  ] Reached target alo-installing.target\r\n"
    );
}

/// **Run all the way through, the road installs alo OS onto the disk the
/// person chose, and the computer then starts the installed alo OS from that
/// disk.**
///
/// Nothing here says what to start at any point: the machine is given no boot
/// order, so each of its three starts — Windows, the environment, the
/// installed system — is the firmware's own choice from its variables, as on a
/// computer. The installed system is known by what it says about itself: its
/// root is btrfs on the disk with the walk's second serial.
#[test]
#[ignore = "starts a virtual machine, installs, and pulls the release; run by name"]
fn the_whole_road_installs_alo_os_and_the_installed_system_starts() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);

    a_fresh_machine(&yard, "installed");
    let chip = SecurityChip::fresh(&yard);
    let firmware = walking::firmware::fedoras(&yard);
    let disc = medium::the_walk_disc(&yard, &Told::TheWholeRoad, &download);
    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start_as_its_variables_decide(
        &firmware,
        &yard,
        "installed",
        Some(&disc),
        &console,
        &chip,
        &[
            (
                "systemd.extra-unit.alo-walk-installed.service",
                THE_INSTALLED_SYSTEM_SAYS,
            ),
            (
                "systemd.unit-dropin.multi-user.target~alo-walk-installed",
                "[Unit]\nWants=alo-walk-installed.service\n",
            ),
        ],
    );

    let done = the_environment_says("installing.installed");
    let not_done = the_environment_says("installing.not-installed");
    let refused = the_environment_says("installing.restart-when-ready");
    let ended = console.wait_for(
        &[done.as_str(), not_done.as_str(), refused.as_str()],
        A_WALK + A_SIGN_IN + AN_INSTALL,
    );
    let installed = ended.as_deref() == Some(done.as_str());
    let reached = installed && console.wait_for(&["ALO-INSTALLED-END"], A_WALK).is_some();
    let screen = machine.screen("the-installed-system");
    let stopped = machine.has_stopped_within(Duration::from_secs(120));
    let said = console.said();
    drop(machine);
    forget(&yard, "installed");

    assert!(
        installed,
        "the environment did not say alo OS is installed; it said {ended:?}. The screen \
         is at {}.\n{said}",
        screen.display()
    );
    let said = without_the_kernels_messages(&said);
    let mut sentences = said.find(&the_environment_says("installing.starting"));
    for named in [
        "installing.reading-the-choice",
        "installing.looking-for-the-disk",
        "installing.checking-the-disk",
        "installing.connecting",
        "installing.checking-it-is-genuine",
        "installing.genuine",
        "installing.installing",
        "installing.installed",
    ] {
        let at = sentences.and_then(|from| {
            said[from..]
                .find(&the_environment_says(named))
                .map(|found| from + found)
        });
        assert!(
            at.is_some(),
            "the environment never said {named} in order.\n{said}"
        );
        sentences = at;
    }
    assert!(
        reached,
        "the installed alo OS never said what it started from. The screen is at {}.\n{said}",
        screen.display()
    );
    let after_the_install = &said[sentences.unwrap_or_default()..];
    let account = after_the_install
        .split("ALO-INSTALLED-BEGIN")
        .nth(1)
        .and_then(|rest| rest.split("ALO-INSTALLED-END").next())
        .unwrap_or_default();
    assert!(
        account
            .lines()
            .any(|line| line.trim_end().ends_with(" btrfs")),
        "the installed system's root is not btrfs:\n{account}"
    );
    assert!(
        account
            .lines()
            .any(|line| line.contains(walking::machine::THE_SECOND_DISKS_SERIAL)),
        "the installed system did not start from the disk the person chose:\n{account}"
    );
    // What the install left behind, read by the installed system itself: the
    // entry named as a person reads it, Windows directly behind it, and no
    // staging area left on any disk (ADR 0062 term 1).
    let entries = alo_installing::Entries::read(account);
    let alo_os = entries
        .every
        .iter()
        .find(|entry| entry.named == alo_installing::THE_ENTRYS_NAME)
        .cloned();
    assert!(
        alo_os.is_some(),
        "the firmware lists no entry named {}, so the install left its own name behind:\n{account}",
        alo_installing::THE_ENTRYS_NAME
    );
    let windows = entries
        .every
        .iter()
        .find(|entry| entry.named.contains("Windows Boot Manager"))
        .cloned();
    if let (Some(alo_os), Some(windows)) = (alo_os, windows) {
        assert_eq!(
            entries.order.first().map(String::as_str),
            Some(alo_os.number.as_str()),
            "alo OS is not the system this computer starts:\n{account}"
        );
        assert_eq!(
            entries.order.get(1).map(String::as_str),
            Some(windows.number.as_str()),
            "Windows Boot Manager is not directly behind alo OS:\n{account}"
        );
    }
    assert!(
        !account.contains(alo_installing::THIS_INSTALLER),
        "the installer's own area is still on a disk:\n{account}"
    );
    assert!(stopped, "the installed system did not turn itself off");
    eprintln!(
        "after the install the firmware started: {:?}\nthe installed system said:\n{account}",
        walking::firmware::starts(after_the_install)
    );
}

/// **A computer that cannot start alo OS starts Windows, with nobody at the
/// keyboard** (ADR 0062 term 1: Windows stands directly behind alo OS, and a
/// person who does nothing ends up in a system that runs).
///
/// This is measured on a computer that really has alo OS installed and really
/// starts it first: the whole road is walked, and only then — with the machine
/// off and from outside the guest — is alo OS's loader taken away from the
/// partition its firmware entry names. The restart after that is given no
/// disc, no boot order and no keypress at all; what the firmware does with a
/// first entry it cannot start is read off its own serial line, and Windows is
/// known to have come up because its own start-up said so.
#[test]
#[ignore = "starts a virtual machine, installs, and pulls the release; run by name"]
fn with_alo_os_unstartable_the_computer_starts_windows_by_itself() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);

    a_fresh_machine(&yard, "fallthrough");
    let chip = SecurityChip::fresh(&yard);
    let firmware = walking::firmware::fedoras(&yard);
    let (installed, installing_said) =
        the_whole_road_installs(&yard, "fallthrough", &firmware, &chip, &download);
    assert!(
        installed,
        "the environment did not say alo OS is installed, so there is nothing here to \
         make unstartable.\n{installing_said}"
    );

    // The damage, from outside the guest with nothing running: one file — the
    // one the firmware's own entry starts — renamed on the disk alo OS was
    // installed on.
    let taken = damaging::take_the_loader_away(&Machine::second_of(&yard, "fallthrough"), &yard);
    assert!(
        taken.gone,
        "alo OS's loader is still where the firmware looks for it: {taken:?}"
    );
    eprintln!(
        "alo OS's loader was taken away from {} ({} bytes); nothing else was changed",
        taken.partition, taken.was_bytes
    );

    // The restart. No disc is put in, no boot order is given and no key is
    // held down: whatever comes up, the firmware chose it from its own
    // variables while a person did nothing.
    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start_as_its_variables_decide(
        &firmware,
        &yard,
        "fallthrough",
        None,
        &console,
        &chip,
        &[],
    );
    let came_up = console.wait_for(&[console::NOTHING_TO_DO, console::DONE], A_SIGN_IN + A_WALK);
    let screen = machine.screen("after-the-loader-was-taken-away");
    let stopped = machine.shut_down(Duration::from_secs(240));
    drop(machine);
    walking::machine::stop();
    let said = without_the_kernels_messages(&console.said());
    forget(&yard, "fallthrough");

    let starts = walking::firmware::starts(&said);
    eprintln!(
        "with alo OS's loader gone, the firmware started: {starts:?}\n\
         the serial line of that restart, whole:\n{said}"
    );
    assert!(
        came_up.is_some(),
        "nothing came up on the restart after alo OS's loader was taken away, so this \
         computer was left with no system a person could use. The screen is at {} and \
         the serial line said:\n{said}",
        screen.display()
    );
    assert!(
        starts
            .iter()
            .any(|started| started.description.contains("Windows Boot Manager")),
        "the firmware never started Windows Boot Manager, so what came up was not \
         Windows falling in behind alo OS. It started {starts:?}.\n{said}"
    );
    assert!(
        !said.contains(&the_environment_says("installing.starting")),
        "the environment started again, so what was taken away was not what this \
         computer starts alo OS with.\n{said}"
    );
    eprintln!(
        "Windows came up by itself and {}",
        if stopped {
            "shut down when it was asked to"
        } else {
            "was cut off"
        }
    );
}

/// Install alo OS the whole way on a machine that has its Windows: the
/// installer run to its end, the restart it asks for, the environment's
/// install, and the machine left off. Gives back whether the environment said
/// alo OS is installed, and everything its serial line said.
///
/// Nothing here says what to start at any point — the machine is given no boot
/// order, so each start is the firmware's own choice from its variables, as on
/// a computer.
fn the_whole_road_installs(
    yard: &Path,
    name: &str,
    firmware: &Path,
    chip: &SecurityChip,
    download: &Path,
) -> (bool, String) {
    let disc = medium::the_walk_disc(yard, &Told::TheWholeRoad, download);
    let console = Console::fresh(&yard.join("console.log"));
    let machine = Machine::start_as_its_variables_decide(
        firmware,
        yard,
        name,
        Some(&disc),
        &console,
        chip,
        &[],
    );
    let done = the_environment_says("installing.installed");
    let not_done = the_environment_says("installing.not-installed");
    let refused = the_environment_says("installing.restart-when-ready");
    let ended = console.wait_for(
        &[done.as_str(), not_done.as_str(), refused.as_str()],
        A_WALK + A_SIGN_IN + AN_INSTALL,
    );
    let installed = ended.as_deref() == Some(done.as_str());
    // The environment says *installed* and then tidies up — the entry named,
    // Windows put behind it, the staging area taken back — and only then
    // restarts. A machine stopped on the first of those sentences is a machine
    // stopped in the middle of the second: measured on 2026-09-26, a walk that
    // stopped there left a firmware with no entry for alo OS at all, and the
    // next boot had nothing to remove.
    let tidied = the_environment_says("installing.tidied");
    let not_whole = the_environment_says("installing.tidy-not-whole");
    let after = installed
        .then(|| console.wait_for(&[tidied.as_str(), not_whole.as_str()], A_WALK))
        .flatten();
    let said = without_the_kernels_messages(&console.said());
    drop(machine);
    walking::machine::stop();
    assert!(
        !installed || after.is_some(),
        "alo OS was installed and the environment never finished tidying up, so what \
         this computer starts is whatever the tidy was in the middle of.\n{said}"
    );
    (installed, said)
}

/// **alo OS is removed again, and what is left is the Windows that was there.**
///
/// The installer plan's task 4: *remove alo OS exists as a documented, tested
/// road back — the partition freed, the boot entry gone, Windows as it was.*
/// The whole road installs alo OS first, so there is a real installation to
/// remove; then Windows is started and the copy the install left behind is run
/// with the removal's word, as a person would run it from the Start menu, and
/// the disk's name is typed back from the sentence the removal itself printed.
///
/// Windows is reached by starting its disk first, which is the harness's way of
/// making the choice a person makes at the menu; nothing else about the start
/// is arranged. The restart after the removal is given no boot order at all.
#[test]
#[ignore = "starts a virtual machine, installs, and pulls the release; run by name"]
fn alo_os_is_removed_again_and_windows_is_what_is_left() {
    let _one = one_machine_at_a_time();
    the_host_has_what_this_needs();
    let yard = needs::the_yard();
    let download = the_download(&yard);

    a_fresh_machine(&yard, "removal");
    let chip = SecurityChip::fresh(&yard);
    let firmware = walking::firmware::fedoras(&yard);
    let (installed, installing_said) =
        the_whole_road_installs(&yard, "removal", &firmware, &chip, &download);
    assert!(
        installed,
        "the environment did not say alo OS is installed, so there is nothing here to \
         remove.\n{installing_said}"
    );

    // Windows, and the removal run from inside it.
    let told = Told::RemovingAloOs {
        left_at: alo_installer::THE_PROGRAMS_HOME.to_owned(),
        left_as: alo_installer::THE_PROGRAMS_NAME.to_owned(),
        shortcut: alo_installer::THE_REMOVALS_SHORTCUT.to_owned(),
        argument: alo_installer::THE_REMOVALS_WORD.to_owned(),
    };
    let disc = medium::the_walk_disc(&yard, &told, &download);
    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start(&yard, "removal", Some(&disc), None, &console, &chip);
    let mut resets = 0;
    while console.wait_for(&[console::BEGINS], A_SIGN_IN).is_none() {
        resets += 1;
        assert!(
            resets <= 2,
            "Windows never reached a sign-in to remove alo OS from, after {resets} resets. \
             The screen is at {} and the serial line said:\n{}",
            machine.screen("removal-never-signed-in").display(),
            console.said()
        );
        machine.reset();
    }
    let finished = console.wait_for(&[console::DONE, console::NOTHING_TO_DO], A_WALK);
    let screen = machine.screen("after-the-removal");
    let shut = machine.shut_down(Duration::from_secs(240));
    drop(machine);
    walking::machine::stop();
    let removing_said = console.said();
    eprintln!("what the removal did, on the guest's own serial line:\n{removing_said}");

    assert_eq!(
        finished.as_deref(),
        Some(console::DONE),
        "the boot that was to remove alo OS never finished what it was told. The screen \
         is at {} and the serial line said:\n{removing_said}",
        screen.display()
    );
    assert!(
        !removing_said.contains("FAIL:"),
        "the guest reported a failure:\n{removing_said}"
    );
    // The name typed is the name the removal itself printed, and nothing else.
    let named = removing_said
        .lines()
        .find_map(|line| line.split("the disk it named: [").nth(1))
        .and_then(|rest| rest.split(']').next())
        .map(str::to_owned);
    assert!(
        named.is_some(),
        "the removal never named the disk it would erase:\n{removing_said}"
    );
    assert!(
        removing_said.contains(&format!("typing: [{}]", named.clone().unwrap_or_default())),
        "the guest typed something other than the disk the removal named:\n{removing_said}"
    );
    assert!(
        removing_said.contains("is empty, its space is free"),
        "the removal never said alo OS is removed:\n{removing_said}"
    );

    // What the computer is afterwards, read by Windows itself: no entry for
    // alo OS anywhere in the firmware, and its disk with no partition table at
    // all — so the space is free, not merely unnamed.
    let after = removing_said
        .split("--- state (after the removal) ---")
        .nth(1)
        .and_then(|rest| rest.split("--- end state ---").next())
        .unwrap_or_default()
        .to_owned();
    assert!(
        !after.is_empty(),
        "Windows never read the computer back after the removal:\n{removing_said}"
    );
    assert!(
        !after.contains(&format!("[{}]", alo_installing::THE_ENTRYS_NAME)),
        "the firmware still lists alo OS:\n{after}"
    );
    let its_disk = after
        .lines()
        .find(|line| {
            line.starts_with("disk ") && line.contains(walking::machine::THE_SECOND_DISKS_SERIAL)
        })
        .unwrap_or_default()
        .to_owned();
    assert!(
        its_disk.contains("style=RAW"),
        "the disk alo OS was on still has a partition table: {its_disk}\n{after}"
    );
    // And no partition of it is left at all. Not by label: Windows reports
    // none for alo OS's own partition, because it cannot read btrfs.
    let its_number = its_disk
        .split_whitespace()
        .nth(1)
        .and_then(|number| number.strip_suffix(':'))
        .unwrap_or("?")
        .to_owned();
    let left = after
        .lines()
        .filter(|line| line.contains(&format!("partition {its_number}/")))
        .collect::<Vec<_>>();
    assert!(
        left.is_empty(),
        "the disk alo OS was on still has partitions: {left:?}\n{after}"
    );
    eprintln!(
        "after the removal Windows read: {its_disk}\nand it {} when it was asked to",
        if shut { "shut down" } else { "was cut off" }
    );

    // And the restart, with no boot order and no keypress: Windows is what
    // this computer is now.
    let console = Console::fresh(&yard.join("console.log"));
    let mut machine = Machine::start_as_its_variables_decide(
        &firmware,
        &yard,
        "removal",
        None,
        &console,
        &chip,
        &[],
    );
    let came_up = console.wait_for(&[console::NOTHING_TO_DO, console::DONE], A_SIGN_IN + A_WALK);
    let screen = machine.screen("after-alo-os-was-removed");
    let stopped = machine.shut_down(Duration::from_secs(240));
    drop(machine);
    walking::machine::stop();
    let said = without_the_kernels_messages(&console.said());
    forget(&yard, "removal");
    let starts = walking::firmware::starts(&said);
    eprintln!(
        "with alo OS removed, the firmware started: {starts:?}\n\
         the serial line of that restart, whole:\n{said}"
    );
    assert!(
        came_up.is_some(),
        "nothing came up after alo OS was removed. The screen is at {} and the serial \
         line said:\n{said}",
        screen.display()
    );
    assert!(
        starts
            .iter()
            .any(|started| started.description.contains("Windows Boot Manager")),
        "the firmware did not start Windows Boot Manager: {starts:?}\n{said}"
    );
    assert!(
        !starts
            .iter()
            .any(|started| started.description == alo_installing::THE_ENTRYS_NAME),
        "the firmware still tried alo OS: {starts:?}\n{said}"
    );
    assert!(stopped, "Windows did not shut down when it was asked to");
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
