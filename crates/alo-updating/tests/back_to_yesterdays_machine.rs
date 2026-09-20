//! Back to yesterday's machine — measured in a virtual machine rather than
//! argued.
//!
//! It builds two bootable images on the base alo OS ships
//! (`image/Containerfile`'s `THE_BASE`), the second differing from the first by
//! one file; installs the first onto a disk with the base's own installer;
//! pushes the second to a registry on the host; and boots the disk under QEMU.
//! Inside, this same test binary runs as a unit at every start, and the machine
//! passes through three starts:
//!
//! 1. **On the first build**: writes the person's named things; finds that
//!    there is nothing to go back to, said before anything is offered; applies
//!    the second build for the next restart; and restarts.
//! 2. **On the second build**: records the update; finds the first build named
//!    as the one before, still on the disk, and replaced at the moment the
//!    record says; writes a file of the person's and a file of the machine's
//!    configuration under `/etc`, both *after* the update; goes back through
//!    [`alo_updating::go_back`] for the next restart; finds going back set, and
//!    a second request refused; and restarts.
//! 3. **Back on the first build**: finds the first build running and its own
//!    system files; every named thing, and the file written after the update,
//!    byte for byte; the record byte for byte as it was, and then one entry
//!    saying the machine went back from the second build to the first, with no
//!    agent, and none on a second start; the second build now named as the one
//!    before, still kept; and the configuration file written under `/etc` after
//!    the update **not there** — which is the base keeping `/etc` per build
//!    (`docs/quirks.md`), and why the sentence the person approves says so.
//!
//! Each start prints one line to the console when it passed, and the host reads
//! the console for all three.
//!
//! # What it needs, and why it is not in the suite
//!
//! Root, `podman`, `qemu-system-x86_64`, UEFI firmware at `/usr/share/OVMF`,
//! loop devices, and most of an hour: the machine that runs this lane has no
//! hardware virtualisation (`docs/quirks.md`), so all three boots are emulated.
//! It is `#[ignore]`d and run by name, and it **fails** rather than skips when
//! something it needs is missing.
//!
//! # What it is not
//!
//! Not the certified machine, and not the release image: the images carry this
//! test instead of alo OS's services, and trust a registry on the host without
//! a signature (a scope in their `policy.json`, with the default refusing).

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use alo_egress::{Destination, Indicator};
use alo_keeping::{Reading, Writing};
use alo_keeping_up::{
    CannotGoBack, Digest, Offered, Running, Since, Source, Standing, Vouching, WhenItApplies,
    a_check_at,
};
use alo_record::{Entry, Happened};
use alo_updating::{
    AcrossRestarts, NotGoneBack, TheBase, after_a_restart, apply, deployments, go_back, running,
    yesterday,
};

/// The base alo OS is built on, exactly as `image/Containerfile` pins it.
const THE_BASE: &str = "quay.io/fedora/fedora-bootc:42@sha256:077182b6ba853b3348d0bede602ac30b9e6568c6422bcf0654de5af96f19b9c3";

/// The registry the second build is served from, on the host.
const THE_REGISTRY: &str = "docker.io/library/registry@sha256:a3d8aaa63ed8681a604f1dea0aa03f100d5895b6a58ace528858a7b332415373";

/// The registry container's name on the host.
const THE_REGISTRY_CONTAINER: &str = "alo-rollback-test-registry";

/// Where the images under test are named on the host.
const THE_IMAGE: &str = "localhost/alo-rollback-test";

/// The repository the machine updates from: the host, as QEMU's user network
/// shows it to the guest.
const THE_SOURCE_IN_THE_MACHINE: &str = "10.0.2.2:5000/alo-rollback-test";

/// How the guest is told which build is offered: an SMBIOS OEM string.
const THE_OFFER: &str = "alo-rollback-test.offered=";

/// What each start prints to the console when it passed.
const FIRST_PASSED: &str = "alo-rollback-test: on the first build: passed";
/// The second start.
const SECOND_PASSED: &str = "alo-rollback-test: on the second build: passed";
/// The third start.
const BACK_PASSED: &str = "alo-rollback-test: back on the first build: passed";

/// How long the machine may take, from being started to powering itself off
/// after all three starts, under emulation.
///
/// The update test's two boots took 949 s and 1050 s with its image builds
/// (`tests/an_update_keeps_the_persons_things.rs`), which gives it thirty
/// minutes. This one boots three times, so forty-five.
const THE_BOOT_DEADLINE: Duration = Duration::from_secs(45 * 60);

/// How every failure begins that is the virtual machine's rather than the
/// work's. `tools/kernel-loop` reads a refusal for these words; keep the two in
/// step.
const DID_NOT_BOOT: &str = "the virtual machine did not finish booting";

/// The generator the test's images mask, for the reason
/// `tests/an_update_keeps_the_persons_things.rs` gives (`docs/quirks.md`).
const THE_SLOW_GENERATOR: &str = "systemd-ssh-generator";

/// What systemd prints when PID 1 has stopped for good.
const PID_ONE_FROZE: &str = "Freezing execution";

/// Where the guest half keeps what it needs across restarts.
const THE_TESTS_OWN: &str = "/var/lib/alo-rollback-test";

/// The person whose folder is written to.
const THE_PERSONS_FOLDER: &str = "/var/home/ada";

/// Where alo OS keeps its record.
const THE_RECORD: &str = "/var/lib/alo/record.jsonl";

/// A file of the machine's own configuration, written after the update.
const WRITTEN_UNDER_ETC_AFTER_THE_UPDATE: &str = "/etc/alo/written-after-the-update.toml";

// ---------------------------------------------------------------------------
// The named things, and the bytes each one holds.
// ---------------------------------------------------------------------------

/// Every named thing the person keeps, where alo OS keeps it, and its bytes —
/// the same list the update test checks.
fn the_named_things() -> Vec<(&'static str, PathBuf, Vec<u8>)> {
    let folder = Path::new(THE_PERSONS_FOLDER);
    vec![
        (
            "the settings",
            folder.join(".config/alo/settings.toml"),
            b"format = 1\n\n[model]\nchosen = \"phi-3-mini-instruct\"\n\n[language]\nread = \"ga\"\n"
                .to_vec(),
        ),
        (
            "the appearance settings",
            folder.join(".config/alo/appearance.toml"),
            b"format = 1\nbackground = \"/var/home/ada/Pictures/harbour.jpg\"\naccent = \"teal\"\n"
                .to_vec(),
        ),
        (
            "the grants",
            PathBuf::from("/var/lib/alo/grants.toml"),
            b"format = 1\n\n[[grant]]\npath = \"/var/home/ada/Invoices\"\nreach = \"read-write\"\nuntil = 1790000000\n"
                .to_vec(),
        ),
        (
            "the pairings",
            PathBuf::from("/var/lib/alo/pairings.toml"),
            b"format = 1\n\n[[paired]]\nwith = \"0f1e2d3c4b5a69788796a5b4c3d2e1f0\"\nname = \"the reception machine\"\n"
                .to_vec(),
        ),
        (
            "the file index",
            folder.join(".local/share/alo/finding/7c3a9e21d4b8f605.index"),
            bytes_counting(64 * 1024, 7),
        ),
        (
            "the list of indexed folders",
            folder.join(".local/share/alo/finding/folders.list"),
            b"/var/home/ada/Invoices\n/var/home/ada/Documents\n".to_vec(),
        ),
        (
            "a letter the person wrote",
            folder.join("Documents/letter to the council.txt"),
            "A chara,\n\nI am writing about the harbour — ná déan dearmad.\n"
                .as_bytes()
                .to_vec(),
        ),
        (
            "a photograph the person kept",
            folder.join("Pictures/harbour.jpg"),
            bytes_counting(3 * 1024 * 1024, 131),
        ),
    ]
}

/// A file the person wrote on the second build, after the update.
fn written_after_the_update() -> (PathBuf, Vec<u8>) {
    (
        Path::new(THE_PERSONS_FOLDER).join("Documents/reply from the council.txt"),
        "Dear Ada,\n\nThe harbour works begin in the spring.\n"
            .as_bytes()
            .to_vec(),
    )
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

/// **The machine under test is built on the base alo OS ships.**
#[test]
fn the_machine_going_back_is_built_on_the_base_alo_os_ships() {
    let recipe = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/Containerfile");
    let recipe = std::fs::read_to_string(recipe).unwrap();
    assert!(
        recipe.contains(&format!("ARG THE_BASE={THE_BASE}\n")),
        "image/Containerfile no longer pins {THE_BASE}; this test must follow it"
    );
}

/// **Going back in a virtual machine returns the earlier build, leaves every
/// named thing byte for byte, and the record says the machine went back.**
#[test]
#[ignore = "builds two images and boots a virtual machine three times under emulation, for most of an hour; run by name"]
fn going_back_in_a_virtual_machine_runs_the_earlier_build_and_leaves_the_persons_things() {
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

    let work = std::env::temp_dir().join("alo-updating-going-back-in-a-machine");
    let _ = std::fs::remove_dir_all(&work);
    let context = work.join("context");
    std::fs::create_dir_all(&context).unwrap();

    std::fs::copy(
        std::env::current_exe().unwrap(),
        context.join("alo-rollback-test"),
    )
    .unwrap();
    std::fs::write(context.join("alo-rollback-test.service"), THE_UNIT).unwrap();
    std::fs::write(
        context.join("50-alo-rollback-test.conf"),
        THE_REGISTRY_TRUST,
    )
    .unwrap();
    std::fs::write(context.join("policy.json"), THE_POLICY).unwrap();
    std::fs::write(
        context.join("Containerfile.one"),
        format!(
            "FROM {THE_BASE}\n\
             COPY alo-rollback-test /usr/libexec/alo-rollback-test\n\
             COPY alo-rollback-test.service /usr/lib/systemd/system/alo-rollback-test.service\n\
             COPY 50-alo-rollback-test.conf /etc/containers/registries.conf.d/50-alo-rollback-test.conf\n\
             COPY policy.json /etc/containers/policy.json\n\
             RUN chmod 0755 /usr/libexec/alo-rollback-test \
              && systemctl enable alo-rollback-test.service \
              && mkdir -p /usr/share/alo-rollback-test /etc/alo \
              && echo one > /usr/share/alo-rollback-test/build \
              && mkdir -p /etc/systemd/system-generators \
              && ln -s /dev/null /etc/systemd/system-generators/{THE_SLOW_GENERATOR}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        context.join("Containerfile.two"),
        format!("FROM {THE_IMAGE}:one\nRUN echo two > /usr/share/alo-rollback-test/build\n"),
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

    let registry = Registry::started();
    let digest_file = work.join("two.digest");
    run(Command::new("podman").args([
        "push",
        "--tls-verify=false",
        "--digestfile",
        &digest_file.display().to_string(),
        &format!("{THE_IMAGE}:two"),
        "localhost:5000/alo-rollback-test:two",
    ]));
    let offered = Digest::read(std::fs::read_to_string(&digest_file).unwrap().trim())
        .expect("the registry named the second build by a whole digest");

    let disk = work.join("disk.raw");
    run(Command::new("truncate").args(["-s", "20G", &disk.display().to_string()]));
    run(Command::new("podman").args([
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
        "install",
        "to-disk",
        "--via-loopback",
        "--wipe",
        "--filesystem",
        "ext4",
        "--karg",
        "console=ttyS0,115200n8",
        "/output/disk.raw",
    ]));

    let own_variables = work.join("variables.fd");
    std::fs::copy(variables, &own_variables).unwrap();
    let console = work.join("console.log");
    let mut machine = Command::new("qemu-system-x86_64")
        .args([
            "-machine",
            "q35",
            "-accel",
            "tcg",
            "-cpu",
            "max",
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
            "-smbios",
            &format!(
                "type=11,value={THE_OFFER}{}",
                Source::named(THE_SOURCE_IN_THE_MACHINE)
                    .unwrap()
                    .at(&offered)
            ),
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
        .take(80)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !said.contains(PID_ONE_FROZE),
        "{DID_NOT_BOOT}: systemd froze while the machine was starting, and nothing after \
         it ran:\n{the_end_of_it}"
    );
    let all_three_ran =
        said.contains(FIRST_PASSED) && said.contains(SECOND_PASSED) && said.contains(BACK_PASSED);
    let one_failed = said.contains("alo-rollback-test: did not pass");
    assert!(
        ended || all_three_ran || one_failed,
        "{DID_NOT_BOOT} within {THE_BOOT_DEADLINE:?}:\n{the_end_of_it}"
    );
    assert!(
        said.contains(FIRST_PASSED),
        "the machine did not pass on the first build:\n{the_end_of_it}"
    );
    assert!(
        said.contains(SECOND_PASSED),
        "the machine did not pass on the second build:\n{the_end_of_it}"
    );
    assert!(
        said.contains(BACK_PASSED),
        "the machine did not pass back on the first build:\n{the_end_of_it}"
    );
    assert!(
        ended,
        "the machine passed all three starts and did not power itself off within \
         {THE_BOOT_DEADLINE:?}:\n{the_end_of_it}"
    );
    let _ = std::fs::remove_dir_all(&work);
}

/// The unit that runs this test binary inside the machine, at every start.
const THE_UNIT: &str = "[Unit]
Description=alo OS going-back test, inside the machine
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/libexec/alo-rollback-test --exact --ignored --nocapture --test-threads=1 in_the_machine
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
/// the local store, for the reasons the update test gives (`docs/quirks.md`).
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
    if passed.is_err() {
        say("alo-rollback-test: did not pass");
    }
    let then = passed.as_ref().map_or("poweroff", |then| *then);
    run(Command::new("systemctl").args(["--no-block", then]));
    assert!(passed.is_ok(), "the machine did not pass; see its console");
}

/// Write every named thing, find nothing to go back to, and apply the update.
fn on_the_first_build() {
    let base = TheBase::on_this_machine();
    let kept = AcrossRestarts::on_this_machine();
    let first = running(&base)
        .expect("what is running is readable at any moment")
        .digest()
        .clone();
    say(&format!("alo-rollback-test: running {}", first.as_str()));
    std::fs::write(Path::new(THE_TESTS_OWN).join("first"), first.as_str()).unwrap();

    for (_, path, bytes) in the_named_things() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
    }
    let mut record = Writing::opening(Path::new(THE_RECORD)).unwrap();
    record
        .keep(&Entry::paired(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000),
        ))
        .unwrap();
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, SystemTime::now()).unwrap(),
        Since::FirstKnown(first.clone())
    );

    let nothing = yesterday(&base, Path::new(THE_RECORD)).unwrap();
    assert_eq!(nothing.before(), None);
    assert_eq!(
        nothing.going_back(),
        Err(&CannotGoBack::NothingBefore),
        "a machine that never updated was offered going back"
    );
    say("alo-rollback-test: nothing to go back to, said before anything was offered");

    let (source, offered) = the_offer();
    let mut indicator = Indicator::default();
    let underway = indicator.beginning_on_its_own(
        a_check_at(Destination::at("10.0.2.2").unwrap()),
        SystemTime::now(),
    );
    let offer = Offered::heard(&underway, offered.clone(), Vouching::ThePlaceVouchesForIt).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    let Standing::Ready(ready) = Standing::between(&Running::reported(first), &offer) else {
        panic!("the second build was not offered as an update");
    };
    apply(&base, &ready, &source, WhenItApplies::AtTheNextRestart)
        .unwrap_or_else(|why| panic!("the update was not applied: {why:?}"));
    assert_eq!(deployments(&base).unwrap().staged(), Some(&offered));
}

/// Record the update, name yesterday, write after the update, and go back.
fn on_the_second_build() {
    let base = TheBase::on_this_machine();
    let kept = AcrossRestarts::on_this_machine();
    let first = the_first();
    let (_, second) = the_offer();
    assert_eq!(
        running(&base).unwrap().digest(),
        &second,
        "the machine did not start on the update"
    );

    let mut record = Writing::opening(Path::new(THE_RECORD)).unwrap();
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, SystemTime::now()).unwrap(),
        Since::Updated {
            from: first.clone(),
            to: second.clone(),
        }
    );
    drop(record);
    let updated_at = Reading::at(Path::new(THE_RECORD))
        .unwrap()
        .record()
        .everything()
        .last()
        .unwrap()
        .at();

    let looked = yesterday(&base, Path::new(THE_RECORD)).unwrap();
    let before = looked.before().expect("the build before is named");
    assert_eq!(before.build(), &first, "the build before is not the first");
    assert!(before.is_kept(), "the build before is not on the disk");
    assert_eq!(
        looked.replaced_at(),
        Some(updated_at),
        "when it was replaced is not the record's moment"
    );
    say(&format!(
        "alo-rollback-test: the build before is {}, kept, replaced when the record says",
        first.as_str()
    ));
    let offer = looked
        .going_back()
        .unwrap_or_else(|why| panic!("going back was not offered: {why:?}"))
        .clone();

    let (path, bytes) = written_after_the_update();
    std::fs::write(&path, bytes).unwrap();
    std::fs::create_dir_all(
        Path::new(WRITTEN_UNDER_ETC_AFTER_THE_UPDATE)
            .parent()
            .unwrap(),
    )
    .unwrap();
    std::fs::write(WRITTEN_UNDER_ETC_AFTER_THE_UPDATE, "format = 1\n").unwrap();
    std::fs::copy(
        THE_RECORD,
        Path::new(THE_TESTS_OWN).join("record-before-going-back"),
    )
    .unwrap();

    let returning = go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart)
        .unwrap_or_else(|why| panic!("going back was not set: {why:?}"));
    assert_eq!(returning.to(), &first);
    let now = deployments(&base).unwrap();
    assert!(
        now.is_going_back(),
        "the base does not say it is going back"
    );
    assert_eq!(now.running().unwrap().digest(), &second);
    assert_eq!(now.rollback(), Some(&first));
    say("alo-rollback-test: going back is set for the next restart");

    assert_eq!(
        go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart),
        Err(NotGoneBack::Refused(CannotGoBack::AlreadyGoingBack)),
        "going back was set twice"
    );
    assert_eq!(
        yesterday(&base, Path::new(THE_RECORD))
            .unwrap()
            .going_back()
            .unwrap_err(),
        &CannotGoBack::AlreadyGoingBack
    );
    assert!(
        deployments(&base).unwrap().is_going_back(),
        "a refused second request turned the machine round"
    );
}

/// Find the first build, the person's things, the record, and what `/etc` did.
fn back_on_the_first_build() {
    let base = TheBase::on_this_machine();
    let kept = AcrossRestarts::on_this_machine();
    let first = the_first();
    let (_, second) = the_offer();

    let now = deployments(&base).expect("what is running is readable at any moment");
    assert_eq!(
        now.running().unwrap().digest(),
        &first,
        "the machine did not go back to the first build"
    );
    assert_eq!(
        now.rollback(),
        Some(&second),
        "the second build was not kept"
    );
    assert!(!now.is_going_back());
    assert_eq!(
        std::fs::read_to_string("/usr/share/alo-rollback-test/build").unwrap(),
        "one\n",
        "the system files are not the first build's"
    );
    say(&format!(
        "alo-rollback-test: running {} again",
        first.as_str()
    ));

    for (named, path, bytes) in the_named_things() {
        let found = std::fs::read(&path)
            .unwrap_or_else(|why| panic!("{named} at {} is gone: {why}", path.display()));
        assert!(
            found == bytes,
            "{named} at {} is not byte for byte what was written",
            path.display()
        );
        say(&format!(
            "alo-rollback-test: {named} is byte for byte as it was"
        ));
    }
    let (path, bytes) = written_after_the_update();
    assert!(
        std::fs::read(&path).unwrap() == bytes,
        "the file written after the update is not byte for byte what was written"
    );
    say("alo-rollback-test: the file written after the update is byte for byte as it was");

    let kept_before =
        std::fs::read(Path::new(THE_TESTS_OWN).join("record-before-going-back")).unwrap();
    assert!(
        std::fs::read(THE_RECORD).unwrap() == kept_before,
        "the record is not byte for byte what it was"
    );
    say("alo-rollback-test: the record is byte for byte as it was");

    assert!(
        !Path::new(WRITTEN_UNDER_ETC_AFTER_THE_UPDATE).exists(),
        "configuration written under /etc after the update came back with the earlier build; \
         the sentence the person approves says it does not"
    );
    say(
        "alo-rollback-test: configuration written under /etc after the update stayed with the newer build",
    );

    let mut record = Writing::opening(Path::new(THE_RECORD)).unwrap();
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, SystemTime::now()).unwrap(),
        Since::RolledBack {
            from: second.clone(),
            to: first.clone(),
        }
    );
    let entries: Vec<Entry> = Reading::at(Path::new(THE_RECORD))
        .unwrap()
        .record()
        .everything()
        .cloned()
        .collect();
    assert_eq!(entries.len(), 3, "{entries:?}");
    let went_back = entries.last().unwrap();
    assert_eq!(went_back.agent(), None);
    assert!(
        matches!(
            went_back.happened(),
            Happened::RolledBack { from, to } if from.is(second.as_str()) && to.is(first.as_str())
        ),
        "{went_back:?}"
    );
    assert!(
        std::fs::read(THE_RECORD).unwrap().starts_with(&kept_before),
        "the record was rewritten rather than added to"
    );
    assert!(
        !kept.going_back_to().exists(),
        "the note outlived the return"
    );
    say(&format!(
        "alo-rollback-test: the record says the machine went back from {} to {}",
        second.as_str(),
        first.as_str()
    ));

    assert_eq!(
        after_a_restart(&base, &kept, &mut record, SystemTime::now()).unwrap(),
        Since::Unchanged(first)
    );
    assert_eq!(
        Reading::at(Path::new(THE_RECORD))
            .unwrap()
            .record()
            .everything()
            .count(),
        3,
        "one return became two entries"
    );

    let looked = yesterday(&base, Path::new(THE_RECORD)).unwrap();
    let before = looked.before().unwrap();
    assert_eq!(before.build(), &second);
    assert!(before.is_kept());
    assert_eq!(looked.replaced_at(), Some(went_back.at()));
    assert!(looked.going_back().is_ok());
}

/// The first build, as the first start kept it.
fn the_first() -> Digest {
    Digest::read(
        std::fs::read_to_string(Path::new(THE_TESTS_OWN).join("first"))
            .unwrap()
            .trim(),
    )
    .unwrap()
}

/// The build offered, as the host put it in the machine's SMBIOS strings.
fn the_offer() -> (Source, Digest) {
    let mut found = None;
    for entry in std::fs::read_dir("/sys/firmware/dmi/entries").unwrap() {
        let path = entry.unwrap().path();
        if !path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("11-"))
        {
            continue;
        }
        let raw = std::fs::read(path.join("raw")).unwrap();
        for piece in raw.split(|&byte| byte == 0) {
            let text = String::from_utf8_lossy(piece);
            if let Some(at) = text.find(THE_OFFER) {
                found = text.get(at + THE_OFFER.len()..).map(str::to_owned);
            }
        }
    }
    let reference = found.expect("the host named the build offered");
    let (source, digest) = reference.split_once('@').unwrap();
    (
        Source::named(source).unwrap(),
        Digest::read(digest).unwrap(),
    )
}

/// One line to the machine's console, which is what the host reads.
fn say(line: &str) {
    println!("{line}");
}
