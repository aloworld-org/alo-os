//! A disk installed with `--filesystem btrfs`, updated and put back — and the
//! person's home subvolume and `/var/lib/alo` found untouched on the far side.
//!
//! The installer names `btrfs` so that
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s undo
//! has something to rewind from, and **a filesystem is chosen at install and
//! cannot be converted afterwards**. That argument is only worth anything if
//! what it makes survives the two things that move a bootc machine underneath a
//! person: an update, and a return to the build before. This test is that
//! sentence measured rather than argued.
//!
//! It builds two bootable images on the base alo OS ships
//! (`image/Containerfile`'s `THE_BASE`), the second differing from the first by
//! one file; installs the first onto a disk with the base's own installer,
//! **with `--filesystem btrfs`**; pushes the second to a registry on the host;
//! and boots the disk. Inside, this same test binary runs as a unit at every
//! start, and the machine passes through three:
//!
//! 1. **On the first build**: finds the disk is btrfs; makes the person's home
//!    a subvolume, which the base makes none of its own accord (`docs/quirks.md`);
//!    writes known bytes into that home and into `/var/lib/alo`; takes the
//!    read-only snapshot an undo would rewind from; and updates to the second
//!    build for the next start.
//! 2. **On the second build**: finds the second build running, every known byte
//!    where it was, the home still a subvolume and the snapshot still a
//!    read-only one holding what it held; and goes back to the build before.
//! 3. **Back on the first build**: finds the first build running and all of the
//!    above still true.
//!
//! Each start prints one line to the console when it passed, and the host reads
//! the console for all three.
//!
//! # What it needs, and why it is not in the suite
//!
//! Root, `podman`, `qemu-system-x86_64`, UEFI firmware at `/usr/share/OVMF`,
//! loop devices, a registry reachable to pull the base, and room for a disk. It
//! is `#[ignore]`d and run by name, and it **fails** rather than skips when
//! something it needs is missing, because a test that passed on a machine that
//! could not run it would be evidence of nothing.
//!
//! # What it is not
//!
//! Not the certified machine, and not the release image: the images here carry
//! this test instead of alo OS's services, and trust a registry on the host
//! without a signature. What it measures is the base's promise under the exact
//! argument alo OS gives it, about the exact places alo OS keeps things.
//!
//! It also decides nothing about what a snapshot is **for** — when one is taken,
//! how long it is kept, who may remove it. That is ADR 0045's and
//! `alo-keeping-up`'s. This says only that the disk does not lose one when the
//! machine moves.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use alo_image::THE_ONLY_FILESYSTEM;

/// The base alo OS is built on, exactly as `image/Containerfile` pins it.
const THE_BASE: &str = "quay.io/fedora/fedora-bootc:42@sha256:077182b6ba853b3348d0bede602ac30b9e6568c6422bcf0654de5af96f19b9c3";

/// The registry the second build is served from, on the host.
const THE_REGISTRY: &str = "docker.io/library/registry@sha256:a3d8aaa63ed8681a604f1dea0aa03f100d5895b6a58ace528858a7b332415373";

/// The registry container's name on the host.
const THE_REGISTRY_CONTAINER: &str = "alo-btrfs-test-registry";

/// Where the images under test are named on the host.
const THE_IMAGE: &str = "localhost/alo-btrfs-test";

/// The repository the machine updates from: the host, as QEMU's user network
/// shows it to the guest.
const THE_SOURCE_IN_THE_MACHINE: &str = "10.0.2.2:5000/alo-btrfs-test:two";

/// What each start prints to the console when it passed.
const FIRST_PASSED: &str = "alo-btrfs-test: on the first build: passed";
/// The second start.
const SECOND_PASSED: &str = "alo-btrfs-test: on the second build: passed";
/// The third start.
const BACK_PASSED: &str = "alo-btrfs-test: back on the first build: passed";

/// What a start prints when it did not pass, so the host can tell a failed
/// check from a machine that never came up.
const DID_NOT_PASS: &str = "alo-btrfs-test: did not pass";

/// How every failure begins that is the virtual machine's rather than the
/// work's. `tools/kernel-loop` reads a refusal for these words; keep them in
/// step.
const DID_NOT_BOOT: &str = "the virtual machine did not finish booting";

/// The generator the test's images mask, because under emulation it can hold
/// systemd's generators past their deadline and freeze the machine
/// (`docs/quirks.md`).
const THE_SLOW_GENERATOR: &str = "systemd-ssh-generator";

/// What systemd prints when PID 1 has stopped for good.
const PID_ONE_FROZE: &str = "Freezing execution";

/// Where the build marker each image carries is.
const THE_BUILD: &str = "/usr/share/alo-btrfs-test/build";

/// Where the guest half keeps which start this is, across restarts.
const THE_TESTS_OWN: &str = "/var/lib/alo-btrfs-test";

/// The person whose home is made a subvolume.
const THE_PERSONS_HOME: &str = "/var/home/ada";

/// Where alo OS keeps what is the machine's rather than a person's, and where
/// ADR 0045 keeps a bracket — outside every grant.
const THE_MACHINES_OWN: &str = "/var/lib/alo";

/// The read-only snapshot an undo would rewind from, taken before the update.
const THE_SNAPSHOT: &str = "/var/lib/alo/undo/before-the-update";

/// How long the machine may take, over all three starts.
///
/// Three starts, an image pulled over the host's registry and a rollback. With
/// hardware virtualisation a start is seconds; without it, the update test next
/// door measured 949 s and 1050 s for two emulated starts, so three gets the
/// same forty-five minutes its own rollback test gets.
const THE_BOOT_DEADLINE: Duration = Duration::from_secs(45 * 60);

/// How much of the drive the disk is written on must be free before a machine
/// is started, in gigabytes.
///
/// The installer plan's rule, after this work filled a development PC's drive
/// from 33 GB to nothing in an hour on 2026-09-15.
const ROOM_FOR_A_DISK: u64 = 15;

// ---------------------------------------------------------------------------
// The known bytes.
// ---------------------------------------------------------------------------

/// Everything written before the update, where it is kept, and its bytes.
///
/// Two in the person's home, which is the subvolume, and two under
/// `/var/lib/alo`, which is the machine's own. Both are named in ADR 0045's
/// acceptance, and an update that kept one and not the other would be an undo
/// with nowhere to keep its bracket.
fn the_known_bytes() -> Vec<(&'static str, PathBuf, Vec<u8>)> {
    let home = Path::new(THE_PERSONS_HOME);
    let machines = Path::new(THE_MACHINES_OWN);
    vec![
        (
            "a letter the person wrote",
            home.join("Documents/letter to the council.txt"),
            "A chara,\n\nI am writing about the harbour.\n"
                .as_bytes()
                .to_vec(),
        ),
        (
            "a photograph the person kept",
            home.join("Pictures/harbour.jpg"),
            bytes_counting(2 * 1024 * 1024, 131),
        ),
        (
            "the grants",
            machines.join("grants.toml"),
            b"format = 1\n\n[[grant]]\npath = \"/var/home/ada/Invoices\"\nreach = \"read-write\"\n"
                .to_vec(),
        ),
        (
            "the record",
            machines.join("record.jsonl"),
            b"{\"format\":1,\"at\":1790000000,\"happened\":\"ran\"}\n".to_vec(),
        ),
    ]
}

/// `length` bytes that are not all the same and are the same every time.
fn bytes_counting(length: usize, seed: u32) -> Vec<u8> {
    let mut state = seed;
    (0..length)
        .map(|_| {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            state.to_be_bytes().first().copied().unwrap_or_default()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// On the host.
// ---------------------------------------------------------------------------

/// **The machine under test is built on the base alo OS ships**, so what it
/// measures is the tool alo OS installs with rather than some other `bootc`.
#[test]
fn the_machine_under_test_is_built_on_the_base_alo_os_ships() {
    let recipe = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/Containerfile");
    let recipe = std::fs::read_to_string(recipe).unwrap();
    assert!(
        recipe.contains(&format!("ARG THE_BASE={THE_BASE}\n")),
        "image/Containerfile no longer pins {THE_BASE}; this test must follow it"
    );
}

/// **The disk this test installs is installed the way the installer installs
/// one**: with the one filesystem, and named from the one place it is named.
///
/// Without this the test could keep passing on `btrfs` while the installer
/// quietly moved, which is the whole failure it exists to catch.
#[test]
fn the_disk_under_test_is_installed_with_the_filesystem_the_installer_names() {
    assert_eq!(
        the_install_arguments("/output/disk.raw")
            .iter()
            .zip(the_install_arguments("/output/disk.raw").iter().skip(1))
            .find(|(argument, _)| argument.as_str() == "--filesystem")
            .map(|(_, value)| value.clone()),
        Some(THE_ONLY_FILESYSTEM.to_owned()),
        "this test no longer installs the filesystem the installer names"
    );
}

/// **A disk installed on btrfs keeps the person's home subvolume and
/// `/var/lib/alo` across an update and a return to the build before.**
#[test]
#[ignore = "builds two images and boots a virtual machine three times; run by name"]
fn an_update_and_a_return_leave_the_home_subvolume_and_the_machines_own_alone() {
    for program in ["podman", "qemu-system-x86_64", "truncate"] {
        assert!(
            Command::new("sh")
                .args(["-c", &format!("command -v {program}")])
                .stdout(Stdio::null())
                .status()
                .unwrap()
                .success(),
            "{program} is needed to run this test and is not installed"
        );
    }
    let firmware = Path::new("/usr/share/OVMF/OVMF_CODE_4M.fd");
    let variables = Path::new("/usr/share/OVMF/OVMF_VARS_4M.fd");
    assert!(
        firmware.exists() && variables.exists(),
        "UEFI firmware is needed at /usr/share/OVMF (the `ovmf` package)"
    );

    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join("a-btrfs-disk");
    let _ = std::fs::remove_dir_all(&work);
    let context = work.join("context");
    std::fs::create_dir_all(&context).unwrap();
    let _swept = Swept(work.clone());
    there_is_room_for_a_disk(&work);

    // Two builds, differing by one file, carrying this test binary.
    std::fs::copy(
        std::env::current_exe().unwrap(),
        context.join("alo-btrfs-test"),
    )
    .unwrap();
    std::fs::write(context.join("alo-btrfs-test.service"), THE_UNIT).unwrap();
    std::fs::write(context.join("50-alo-btrfs-test.conf"), THE_REGISTRY_TRUST).unwrap();
    std::fs::write(context.join("policy.json"), THE_POLICY).unwrap();
    std::fs::write(
        context.join("Containerfile.one"),
        format!(
            "FROM {THE_BASE}\n\
             COPY alo-btrfs-test /usr/libexec/alo-btrfs-test\n\
             COPY alo-btrfs-test.service /usr/lib/systemd/system/alo-btrfs-test.service\n\
             COPY 50-alo-btrfs-test.conf /etc/containers/registries.conf.d/50-alo-btrfs-test.conf\n\
             COPY policy.json /etc/containers/policy.json\n\
             RUN chmod 0755 /usr/libexec/alo-btrfs-test \
              && systemctl enable alo-btrfs-test.service \
              && mkdir -p /usr/share/alo-btrfs-test \
              && echo one > {THE_BUILD} \
              && mkdir -p /etc/systemd/system-generators \
              && ln -s /dev/null /etc/systemd/system-generators/{THE_SLOW_GENERATOR}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        context.join("Containerfile.two"),
        format!("FROM {THE_IMAGE}:one\nRUN echo two > {THE_BUILD}\n"),
    )
    .unwrap();
    for build in ["one", "two"] {
        run(Command::new("podman").args([
            "build",
            "--pull=never",
            "-f",
            &context
                .join(format!("Containerfile.{build}"))
                .display()
                .to_string(),
            "-t",
            &format!("{THE_IMAGE}:{build}"),
            &context.display().to_string(),
        ]));
    }
    let _images = Images;

    // The place the update comes from, on the host, with the second build in it.
    let registry = Registry::started();
    run(Command::new("podman").args([
        "push",
        "--tls-verify=false",
        &format!("{THE_IMAGE}:two"),
        "localhost:5000/alo-btrfs-test:two",
    ]));

    // The first build, installed the way the installer installs: on btrfs.
    let disk = work.join("disk.raw");
    run(Command::new("truncate").args(["-s", "20G", &disk.display().to_string()]));
    let mut installing = Command::new("podman");
    installing.args([
        "run",
        "--rm",
        "--privileged",
        "--pid=host",
        "--security-opt",
        "label=type:unconfined_t",
        "-v",
        "/var/lib/containers:/var/lib/containers",
        "-v",
        &format!("{}:/output", work.display()),
        &format!("{THE_IMAGE}:one"),
        "bootc",
    ]);
    installing.args(the_install_arguments("/output/disk.raw"));
    run(&mut installing);

    // Three starts, on one machine that restarts itself twice.
    let own_variables = work.join("variables.fd");
    std::fs::copy(variables, &own_variables).unwrap();
    let console = work.join("console.log");
    let mut machine = Command::new("qemu-system-x86_64")
        .args([
            "-machine",
            "q35",
            "-accel",
            the_accelerator(),
            "-cpu",
            the_processor(),
            "-smp",
            "3",
            "-m",
            "3072",
            "-no-user-config",
            "-nodefaults",
            "-display",
            "none",
            "-drive",
            &format!(
                "if=pflash,format=raw,readonly=on,file={}",
                firmware.display()
            ),
            "-drive",
            &format!("if=pflash,format=raw,file={}", own_variables.display()),
            "-drive",
            &format!("file={},format=raw,if=virtio", disk.display()),
            "-netdev",
            "user,id=n0",
            "-device",
            "virtio-net-pci,netdev=n0",
            "-serial",
            &format!("file:{}", console.display()),
        ])
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    let ended = waited(&mut machine, THE_BOOT_DEADLINE, &console);
    drop(registry);

    let said = std::fs::read_to_string(&console).unwrap_or_default();
    let the_end_of_it = said
        .lines()
        .rev()
        .take(120)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    // The machine's own failures first, so that a machine which never reached
    // the test is never reported as a test that failed.
    assert!(
        !said.contains(PID_ONE_FROZE),
        "{DID_NOT_BOOT}: systemd froze while the machine was starting:\n{the_end_of_it}"
    );
    let all_three_ran =
        said.contains(FIRST_PASSED) && said.contains(SECOND_PASSED) && said.contains(BACK_PASSED);
    assert!(
        ended || all_three_ran || said.contains(DID_NOT_PASS),
        "{DID_NOT_BOOT} within {THE_BOOT_DEADLINE:?}:\n{the_end_of_it}"
    );
    for (what, passed) in [
        ("on the first build", FIRST_PASSED),
        ("on the second build", SECOND_PASSED),
        ("back on the first build", BACK_PASSED),
    ] {
        assert!(
            said.contains(passed),
            "the machine did not pass {what}:\n{the_end_of_it}"
        );
    }
    assert!(
        ended,
        "the machine passed all three starts and did not power itself off within \
         {THE_BOOT_DEADLINE:?}:\n{the_end_of_it}"
    );
}

/// What `bootc` is given to write the disk, in one place, so the test and its
/// own guard read the same list.
fn the_install_arguments(onto: &str) -> Vec<String> {
    [
        "install",
        "to-disk",
        "--via-loopback",
        "--wipe",
        "--filesystem",
        THE_ONLY_FILESYSTEM,
        "--karg",
        "console=ttyS0,115200n8",
        onto,
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

/// Hardware virtualisation where this machine has it, and emulation where it
/// does not — the same measurement either way, at very different speeds.
fn the_accelerator() -> &'static str {
    if Path::new("/dev/kvm").exists() {
        "kvm"
    } else {
        "tcg"
    }
}

/// The processor to give the machine, which follows the accelerator: `host` is
/// only meaningful with KVM.
fn the_processor() -> &'static str {
    if Path::new("/dev/kvm").exists() {
        "host"
    } else {
        "max"
    }
}

/// Stop before a machine is started if the drive the disk goes on is nearly
/// full, rather than filling it and taking the development machine with it.
fn there_is_room_for_a_disk(work: &Path) {
    let said = Command::new("df")
        .args(["--output=avail", "--block-size=1G"])
        .arg(work)
        .output()
        .unwrap();
    let free: u64 = String::from_utf8_lossy(&said.stdout)
        .lines()
        .nth(1)
        .and_then(|line| line.trim().parse().ok())
        .expect("df said how much room there is");
    assert!(
        free >= ROOM_FOR_A_DISK,
        "{free} GB is free where this test writes its disk and it needs {ROOM_FOR_A_DISK} GB; \
         a full drive takes the whole machine with it"
    );
}

/// The unit that runs this test binary inside the machine, at every start.
const THE_UNIT: &str = "[Unit]
Description=alo OS btrfs disk test, inside the machine
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/libexec/alo-btrfs-test --exact --ignored --nocapture --test-threads=1 in_the_machine
StandardOutput=journal+console
StandardError=journal+console
TimeoutStartSec=infinity

[Install]
WantedBy=multi-user.target
";

/// The host's registry, over plain HTTP, as the guest reaches it.
const THE_REGISTRY_TRUST: &str = "[[registry]]
location = \"10.0.2.2:5000\"
insecure = true
";

/// A signature policy refusing by default and accepting the host's registry and
/// the local store, which is the shape the update tests next door give it.
const THE_POLICY: &str = r#"{
  "default": [{"type": "reject"}],
  "transports": {
    "docker": {"10.0.2.2:5000": [{"type": "insecureAcceptAnything"}]},
    "docker-daemon": {"": [{"type": "insecureAcceptAnything"}]},
    "containers-storage": {"": [{"type": "insecureAcceptAnything"}]}
  }
}
"#;

/// The registry container, removed when this goes out of scope.
struct Registry;

impl Registry {
    fn started() -> Self {
        let _ = Command::new("podman")
            .args(["rm", "-f", THE_REGISTRY_CONTAINER])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        run(Command::new("podman").args([
            "run",
            "-d",
            "--name",
            THE_REGISTRY_CONTAINER,
            "-p",
            "5000:5000",
            THE_REGISTRY,
        ]));
        Self
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let _ = Command::new("podman")
            .args(["rm", "-f", THE_REGISTRY_CONTAINER])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

/// The two images this test built, removed when it ends — a build that leaves
/// gigabytes in the container store is how this plan filled a drive once.
struct Images;

impl Drop for Images {
    fn drop(&mut self) {
        for build in ["one", "two"] {
            let _ = Command::new("podman")
                .args(["rmi", "-f", &format!("{THE_IMAGE}:{build}")])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

/// The disks this test wrote, removed when it ends, pass or fail.
struct Swept(PathBuf);

impl Drop for Swept {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Run a command to completion, failing the test with what it said if it fails.
fn run(command: &mut Command) {
    let output = command.stdin(Stdio::null()).output().unwrap();
    assert!(
        output.status.success(),
        "{command:?} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Wait for the machine to power itself off, up to `longest`; stop it if it has
/// not, or at once if PID 1 froze.
fn waited(child: &mut Child, longest: Duration, console: &Path) -> bool {
    let started = Instant::now();
    while started.elapsed() < longest {
        if child.try_wait().unwrap().is_some() {
            return true;
        }
        if std::fs::read_to_string(console).is_ok_and(|said| said.contains(PID_ONE_FROZE)) {
            break;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    let _ = child.kill();
    let _ = child.wait();
    false
}

// ---------------------------------------------------------------------------
// Inside the machine.
// ---------------------------------------------------------------------------

/// Which start this is, kept across restarts in the test's own folder.
const THE_STAGE: &str = "stage";

/// The half that runs inside the virtual machine, at each start.
#[test]
#[ignore = "runs inside the virtual machine the test above boots, by name"]
fn in_the_machine() {
    let own = Path::new(THE_TESTS_OWN);
    let stage_at = own.join(THE_STAGE);
    // Whatever happens below, the machine is left to end: restarted after a
    // start that passed and has another after it, powered off otherwise.
    let passed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::fs::create_dir_all(own).unwrap();
        match std::fs::read_to_string(&stage_at).ok().as_deref() {
            None => {
                on_the_first_build();
                std::fs::write(&stage_at, "updated").unwrap();
                say(FIRST_PASSED);
                "reboot"
            }
            Some("updated") => {
                on_the_second_build();
                std::fs::write(&stage_at, "going-back").unwrap();
                say(SECOND_PASSED);
                "reboot"
            }
            Some("going-back") => {
                back_on_the_first_build();
                std::fs::write(&stage_at, "done").unwrap();
                say(BACK_PASSED);
                "poweroff"
            }
            Some(other) => panic!("a start after the test ended: {other}"),
        }
    }));
    let next = match passed {
        Ok(next) => next,
        Err(why) => {
            say(&format!("{DID_NOT_PASS}: {}", what_went_wrong(&why)));
            "poweroff"
        }
    };
    let _ = Command::new("systemctl")
        .args(["--no-block", next])
        .status();
}

/// The first start: the disk is btrfs, the home becomes a subvolume, the known
/// bytes are written, the bracket is taken, and the machine updates.
fn on_the_first_build() {
    assert_eq!(
        the_build(),
        "one",
        "the machine did not start on the first build"
    );
    the_disk_is_the_one_the_installer_makes();

    // The base makes no home subvolume of its own (`docs/quirks.md`), so the
    // thing an undo would rewind is made here — as whoever creates an account
    // will make it.
    in_the_machine_run(Command::new("btrfs").args(["subvolume", "create", THE_PERSONS_HOME]));
    assert!(
        is_a_subvolume(Path::new(THE_PERSONS_HOME)),
        "the person's home was not made a subvolume"
    );

    for (what, at, bytes) in the_known_bytes() {
        std::fs::create_dir_all(at.parent().unwrap())
            .unwrap_or_else(|why| panic!("{what} has nowhere to go: {why}"));
        std::fs::write(&at, &bytes).unwrap_or_else(|why| panic!("{what} was not written: {why}"));
    }

    // The bracket ADR 0045 rewinds from, taken the way it will be taken: a
    // read-only snapshot of the home, kept outside every grant under
    // /var/lib/alo. The destination must not exist first: btrfs would make the
    // snapshot inside it and answer `Read-only file system` (`docs/quirks.md`).
    std::fs::create_dir_all(Path::new(THE_SNAPSHOT).parent().unwrap()).unwrap();
    assert!(
        !Path::new(THE_SNAPSHOT).exists(),
        "the snapshot's place is already taken"
    );
    in_the_machine_run(Command::new("btrfs").args([
        "subvolume",
        "snapshot",
        "-r",
        THE_PERSONS_HOME,
        THE_SNAPSHOT,
    ]));
    the_bracket_is_still_there();

    in_the_machine_run(Command::new("bootc").args([
        "switch",
        "--retain",
        THE_SOURCE_IN_THE_MACHINE,
    ]));
}

/// The second start: the second build is running, and nothing of the person's
/// or the machine's moved.
fn on_the_second_build() {
    assert_eq!(
        the_build(),
        "two",
        "the update did not bring the second build"
    );
    the_disk_is_the_one_the_installer_makes();
    everything_known_is_as_it_was("after the update");
    the_bracket_is_still_there();

    in_the_machine_run(Command::new("bootc").arg("rollback"));
}

/// The third start: the build before is running, and still nothing moved.
fn back_on_the_first_build() {
    assert_eq!(
        the_build(),
        "one",
        "going back did not bring the first build"
    );
    the_disk_is_the_one_the_installer_makes();
    everything_known_is_as_it_was("after going back");
    the_bracket_is_still_there();
}

/// The build running, as the image itself says.
fn the_build() -> String {
    std::fs::read_to_string(THE_BUILD)
        .expect("the image says which build it is")
        .trim()
        .to_owned()
}

/// The person's things and the machine's own, byte for byte where they were.
fn everything_known_is_as_it_was(when: &str) {
    assert!(
        is_a_subvolume(Path::new(THE_PERSONS_HOME)),
        "the person's home is no longer a subvolume {when}"
    );
    for (what, at, bytes) in the_known_bytes() {
        let found = std::fs::read(&at)
            .unwrap_or_else(|why| panic!("{what} is not at {} {when}: {why}", at.display()));
        assert!(
            found == bytes,
            "{what} is not what it was {when}: {} bytes where there were {}",
            found.len(),
            bytes.len()
        );
    }
}

/// The bracket: still a subvolume, still read-only, and still holding what the
/// home held when it was taken.
fn the_bracket_is_still_there() {
    let at = Path::new(THE_SNAPSHOT);
    assert!(is_a_subvolume(at), "the snapshot is no longer a subvolume");
    let said = Command::new("btrfs")
        .args(["property", "get"])
        .arg(at)
        .arg("ro")
        .output()
        .expect("btrfs said whether the snapshot is read-only");
    assert!(
        String::from_utf8_lossy(&said.stdout).contains("ro=true"),
        "the snapshot is no longer read-only: {}",
        String::from_utf8_lossy(&said.stdout)
    );
    for (what, kept_at, bytes) in the_known_bytes() {
        let Ok(inside) = kept_at.strip_prefix(THE_PERSONS_HOME) else {
            continue;
        };
        let found = std::fs::read(at.join(inside))
            .unwrap_or_else(|why| panic!("the snapshot no longer holds {what}: {why}"));
        assert!(found == bytes, "the snapshot's {what} is not what it was");
    }
}

/// The root filesystem is the one the installer names, on the mount the
/// person's things and the machine's own are on.
fn the_disk_is_the_one_the_installer_makes() {
    let said = Command::new("findmnt")
        .args(["-no", "FSTYPE", "/var"])
        .output()
        .expect("findmnt said what /var is on");
    assert_eq!(
        String::from_utf8_lossy(&said.stdout).trim(),
        THE_ONLY_FILESYSTEM,
        "the disk this machine was installed onto is not {THE_ONLY_FILESYSTEM}"
    );
}

/// Whether this path is a btrfs subvolume, as the base's own tool answers.
fn is_a_subvolume(at: &Path) -> bool {
    Command::new("btrfs")
        .args(["subvolume", "show"])
        .arg(at)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|ended| ended.success())
}

/// Run a command inside the machine, failing with what it said.
fn in_the_machine_run(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?} failed inside the machine:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// What a panic said, as a line.
fn what_went_wrong(why: &(dyn std::any::Any + Send)) -> String {
    why.downcast_ref::<String>()
        .cloned()
        .or_else(|| why.downcast_ref::<&str>().map(ToString::to_string))
        .unwrap_or_else(|| "something that said nothing".to_owned())
        .replace('\n', " / ")
}

/// Say one line on the machine's console, where the host is reading.
fn say(line: &str) {
    println!("{line}");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}
