//! An update applied, and the same machine afterwards — measured in a virtual
//! machine rather than argued.
//!
//! *The person's data survived* is a claim that must be made about named
//! things, so this test names them. It builds two bootable images on the base
//! alo OS ships (`image/Containerfile`'s `THE_BASE`), the second differing from
//! the first by one file; installs the first onto a disk with the base's own
//! installer; pushes the second to a registry on the host; and boots the disk
//! under QEMU. Inside, this same test binary runs as a unit and:
//!
//! 1. **before the update** writes, by name, the person's settings, their
//!    grants, the machine's pairings, the record and the file indexes, and two
//!    of the person's own files; reads which build is running through
//!    [`alo_updating::running`]; hears the second build offered during a check
//!    on the indicator; applies it through [`alo_updating::apply`] for the next
//!    restart; finds it staged; is refused applying it a second time; and
//!    restarts the machine — the person's restart, which is when an update
//!    applies;
//! 2. **after the restart** finds the second build running and the first kept
//!    as the one before, every named thing byte for byte, and the record
//!    exactly as it was — and then [`alo_updating::after_a_restart`] adds one
//!    entry saying the machine updated from the first build to the second, with
//!    no agent, and a second start on the same build adds nothing.
//!
//! Each half prints one line to the machine's console, and the test on the host
//! reads the console for both.
//!
//! # What it needs, and why it is not in the suite
//!
//! Root, `podman`, `qemu-system-x86_64`, UEFI firmware at `/usr/share/OVMF`,
//! loop devices, and most of an hour: the machine that runs this lane has no
//! hardware virtualisation (`docs/quirks.md`), so both boots are emulated. It is
//! `#[ignore]`d and run by name. It **fails** rather than skips when something
//! it needs is missing, because a test that passes on a machine that could not
//! run it would be evidence of nothing.
//!
//! # What it is not
//!
//! Not the certified machine, and not the release image: the images here carry
//! this test instead of alo OS's services, and trust a registry on the host
//! without a signature (a scope in their `policy.json`, with the default
//! refusing). What it measures is the base's promise under the exact
//! instructions alo OS gives it, about the exact paths alo OS keeps things at.

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
    Digest, NotStaged, Offered, Running, Since, Source, Standing, Vouching, WhenItApplies,
    a_check_at,
};
use alo_record::{Entry, Happened};
use alo_updating::{
    AcrossRestarts, NotApplied, TheBase, after_a_restart, apply, deployments, running,
};

/// The base alo OS is built on, exactly as `image/Containerfile` pins it.
const THE_BASE: &str = "quay.io/fedora/fedora-bootc:42@sha256:077182b6ba853b3348d0bede602ac30b9e6568c6422bcf0654de5af96f19b9c3";

/// The registry the second build is served from, on the host.
const THE_REGISTRY: &str = "docker.io/library/registry@sha256:a3d8aaa63ed8681a604f1dea0aa03f100d5895b6a58ace528858a7b332415373";

/// Where the images under test are named on the host.
const THE_IMAGE: &str = "localhost/alo-update-test";

/// The repository the machine updates from: the host, as QEMU's user network
/// shows it to the guest.
const THE_SOURCE_IN_THE_MACHINE: &str = "10.0.2.2:5000/alo-update-test";

/// How the guest is told which build is offered: an SMBIOS OEM string.
const THE_OFFER: &str = "alo-update-test.offered=";

/// What each half prints to the console when it passed.
const BEFORE_PASSED: &str = "alo-update-test: before the update: passed";
/// What the second half prints.
const AFTER_PASSED: &str = "alo-update-test: after the restart: passed";

/// How long the machine may take, from being started to powering itself off
/// after both halves: both boots, the update and the checks, under emulation.
///
/// Sized from what was measured rather than guessed. The whole test, image
/// builds included, took 949 s and 1050 s on the machine that runs this lane,
/// with no hardware virtualisation (`docs/quirks.md`). Thirty minutes is about
/// twice that, which is generous without letting a machine that will never boot
/// hold the gate for most of two hours, as it did on 2026-09-15.
const THE_BOOT_DEADLINE: Duration = Duration::from_secs(30 * 60);

/// How every failure begins that is the virtual machine's rather than the
/// work's: it stopped before both halves had run, and so said nothing about
/// the update.
///
/// `tools/kernel-loop` reads a refusal for these words and runs the gates again
/// instead of sending a worker to repair work that nothing was wrong with. Keep
/// the two in step.
const DID_NOT_BOOT: &str = "the virtual machine did not finish booting";

/// The generator the test's images mask, because under emulation it can hold
/// systemd's generators past their deadline and freeze the machine.
///
/// It probes for a hypervisor socket to offer SSH over it, which loads the
/// vsock modules. On this lane's emulated machine that took over a minute, while
/// PID 1 gives all its generators 45 seconds together and freezes when they run
/// over (`docs/quirks.md`, *systemd freezes the machine when its generators run
/// past 45 seconds*). Nothing this test measures reaches the machine over SSH.
/// A symlink to `/dev/null` under `/etc/systemd/system-generators/` is how
/// `systemd.generator(7)` says a generator is masked. This is the test's own
/// image only; the image alo OS ships is not changed.
const THE_SLOW_GENERATOR: &str = "systemd-ssh-generator";

/// What systemd prints when PID 1 has stopped for good. Nothing more will start
/// on that machine however long it is waited on, so the wait ends there.
const PID_ONE_FROZE: &str = "Freezing execution";

/// Where the guest half keeps what it needs across the restart.
const THE_TESTS_OWN: &str = "/var/lib/alo-update-test";

/// The person whose folder is written to.
const THE_PERSONS_FOLDER: &str = "/var/home/ada";

// ---------------------------------------------------------------------------
// The named things, and the bytes each one holds.
// ---------------------------------------------------------------------------

/// Every named thing the person keeps, where alo OS keeps it, and its bytes.
///
/// The record is not here: it is written through `alo-keeping` and checked on
/// its own, because after the restart it gains exactly one entry.
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

/// **The machine under test is built on the base alo OS ships**, so what this
/// measures is that base's behaviour and not some other one's.
#[test]
fn the_machine_under_test_is_built_on_the_base_alo_os_ships() {
    let recipe = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../image/Containerfile");
    let recipe = std::fs::read_to_string(recipe).unwrap();
    assert!(
        recipe.contains(&format!("ARG THE_BASE={THE_BASE}\n")),
        "image/Containerfile no longer pins {THE_BASE}; this test must follow it"
    );
}

/// **An update applied in a virtual machine keeps every named thing byte for
/// byte, and the record says the machine updated.**
#[test]
#[ignore = "builds two images and boots a virtual machine twice under emulation, for most of an hour; run by name"]
fn an_update_applied_in_a_virtual_machine_keeps_every_named_thing_byte_for_byte() {
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

    let work = std::env::temp_dir().join("alo-updating-in-a-machine");
    let _ = std::fs::remove_dir_all(&work);
    let context = work.join("context");
    std::fs::create_dir_all(&context).unwrap();

    // The two builds: this test inside the base, then one file more.
    std::fs::copy(
        std::env::current_exe().unwrap(),
        context.join("alo-update-test"),
    )
    .unwrap();
    std::fs::write(context.join("alo-update-test.service"), THE_UNIT).unwrap();
    std::fs::write(context.join("50-alo-update-test.conf"), THE_REGISTRY_TRUST).unwrap();
    std::fs::write(context.join("policy.json"), THE_POLICY).unwrap();
    std::fs::write(
        context.join("Containerfile.one"),
        format!(
            "FROM {THE_BASE}\n\
             COPY alo-update-test /usr/libexec/alo-update-test\n\
             COPY alo-update-test.service /usr/lib/systemd/system/alo-update-test.service\n\
             COPY 50-alo-update-test.conf /etc/containers/registries.conf.d/50-alo-update-test.conf\n\
             COPY policy.json /etc/containers/policy.json\n\
             RUN chmod 0755 /usr/libexec/alo-update-test \
              && systemctl enable alo-update-test.service \
              && mkdir -p /usr/share/alo-update-test \
              && echo one > /usr/share/alo-update-test/build \
              && mkdir -p /etc/systemd/system-generators \
              && ln -s /dev/null /etc/systemd/system-generators/{THE_SLOW_GENERATOR}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        context.join("Containerfile.two"),
        format!("FROM {THE_IMAGE}:one\nRUN echo two > /usr/share/alo-update-test/build\n"),
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

    // The place updates come from, on the host, with the second build in it.
    let registry = Registry::started();
    let digest_file = work.join("two.digest");
    run(Command::new("podman").args([
        "push",
        "--tls-verify=false",
        "--digestfile",
        &digest_file.display().to_string(),
        &format!("{THE_IMAGE}:two"),
        "localhost:5000/alo-update-test:two",
    ]));
    let offered = Digest::read(std::fs::read_to_string(&digest_file).unwrap().trim())
        .expect("the registry named the second build by a whole digest");

    // The first build, installed onto a disk by the base's own installer.
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
        // The filesystem a person's machine is really installed onto, taken
        // from the one place that decides it rather than spelt here: what this
        // test proves about an update keeping a person's things has to be
        // proved about the machine alo OS ships, and until 2026-09-21 it was
        // proved about `ext4`, which no alo OS machine has.
        alo_image::THE_ONLY_FILESYSTEM,
        "--karg",
        "console=ttyS0,115200n8",
        "/output/disk.raw",
    ]));

    // Boot it, and wait for the machine to power itself off.
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
    // The machine's failures first, so that a machine which never reached the
    // test is never reported as a test that failed.
    assert!(
        !said.contains(PID_ONE_FROZE),
        "{DID_NOT_BOOT}: systemd froze while the machine was starting, and nothing after \
         it ran:\n{the_end_of_it}"
    );
    let both_halves_ran = said.contains(BEFORE_PASSED) && said.contains(AFTER_PASSED);
    assert!(
        ended || both_halves_ran,
        "{DID_NOT_BOOT} within {THE_BOOT_DEADLINE:?}:\n{the_end_of_it}"
    );
    assert!(
        ended,
        "the machine passed both halves and did not power itself off within \
         {THE_BOOT_DEADLINE:?}:\n{the_end_of_it}"
    );
    assert!(
        said.contains(BEFORE_PASSED),
        "the machine did not pass before the update:\n{the_end_of_it}"
    );
    assert!(
        said.contains(AFTER_PASSED),
        "the machine did not pass after the restart:\n{the_end_of_it}"
    );
    let _ = std::fs::remove_dir_all(&work);
}

/// The unit that runs this test binary inside the machine, at every start.
const THE_UNIT: &str = "[Unit]
Description=alo OS update test, inside the machine
Wants=network-online.target
After=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/libexec/alo-update-test --exact --ignored --nocapture --test-threads=1 in_the_machine
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

/// A signature policy that refuses by default — which the base insists on
/// before it stages anything under `--enforce-container-sigpolicy` — and
/// accepts the host's registry, which serves nothing but the build under test.
///
/// **And the local image store**, because `bootc install` runs inside the image
/// it installs and reads *that image's* policy to open it from the store: with
/// the default refusing and no scope for the store, the install is refused
/// before anything is written (`docs/quirks.md`, 2026-09-15).
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
            .args(["rm", "-f", "alo-update-test-registry"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        run(Command::new("podman").args([
            "run",
            "-d",
            "--name",
            "alo-update-test-registry",
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
            .args(["rm", "-f", "alo-update-test-registry"])
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
/// not.
///
/// A machine whose console says PID 1 froze is stopped at once rather than
/// waited on to the deadline, and the caller reports it by that line.
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

/// The half that runs inside the virtual machine, at each start.
#[test]
#[ignore = "runs inside the virtual machine the test above boots, by name"]
fn in_the_machine() {
    let own = Path::new(THE_TESTS_OWN);
    let was = own.join("running-before");
    // Whatever happens below, the machine is left to end: restarted when the
    // first half passed, and powered off otherwise, so the host is never left
    // waiting on a machine that stopped part of the way through.
    let passed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if was.exists() {
            after_the_restart(
                &Digest::read(std::fs::read_to_string(&was).unwrap().trim()).unwrap(),
            );
            say(AFTER_PASSED);
            "poweroff"
        } else {
            std::fs::create_dir_all(own).unwrap();
            let before = before_the_update();
            std::fs::write(&was, before.as_str()).unwrap();
            say(BEFORE_PASSED);
            "reboot"
        }
    }));
    let then = passed.as_ref().map_or("poweroff", |then| *then);
    run(Command::new("systemctl").args(["--no-block", then]));
    assert!(passed.is_ok(), "the machine did not pass; see its console");
}

/// Write every named thing, apply the update, and hand back the build running.
fn before_the_update() -> Digest {
    let base = TheBase::on_this_machine();
    let before = running(&base)
        .expect("what is running is readable at any moment")
        .digest()
        .clone();
    say(&format!("alo-update-test: running {}", before.as_str()));

    for (_, path, bytes) in the_named_things() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
    }
    let record_at = Path::new("/var/lib/alo/record.jsonl");
    let mut record = Writing::opening(record_at).unwrap();
    record
        .keep(&Entry::paired(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000),
        ))
        .unwrap();
    assert_eq!(
        after_a_restart(
            &base,
            &AcrossRestarts::on_this_machine(),
            &mut record,
            SystemTime::now()
        )
        .unwrap(),
        Since::FirstKnown(before.clone())
    );
    drop(record);
    std::fs::copy(record_at, Path::new(THE_TESTS_OWN).join("record-before")).unwrap();

    let (source, offered) = the_offer();
    let mut indicator = Indicator::default();
    let underway = indicator.beginning_on_its_own(
        a_check_at(Destination::at("10.0.2.2").unwrap()),
        SystemTime::now(),
    );
    let offer = Offered::heard(&underway, offered.clone(), Vouching::ThePlaceVouchesForIt).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    let Standing::Ready(ready) = Standing::between(&Running::reported(before.clone()), &offer)
    else {
        panic!("the second build was not offered as an update");
    };

    let staging = apply(&base, &ready, &source, WhenItApplies::AtTheNextRestart)
        .unwrap_or_else(|why| panic!("the update was not applied: {why:?}"));
    assert_eq!(staging.to(), &offered);
    let now = deployments(&base).unwrap();
    assert_eq!(now.staged(), Some(&offered), "the update is not waiting");
    assert_eq!(
        now.running().unwrap().digest(),
        &before,
        "staging changed what is running before any restart"
    );
    assert_eq!(
        apply(&base, &ready, &source, WhenItApplies::AtTheNextRestart),
        Err(NotApplied::Refused(NotStaged::AlreadyWaiting)),
        "the same update was staged twice"
    );
    before
}

/// Check every named thing, the build running, and the record.
fn after_the_restart(before: &Digest) {
    let base = TheBase::on_this_machine();
    let (_, offered) = the_offer();
    let now = deployments(&base).expect("what is running is readable at any moment");
    assert_eq!(
        now.running().unwrap().digest(),
        &offered,
        "the machine did not start on the update"
    );
    assert_eq!(
        now.rollback(),
        Some(before),
        "the build before was not kept"
    );
    assert_eq!(
        std::fs::read_to_string("/usr/share/alo-update-test/build").unwrap(),
        "two\n",
        "the system files are not the second build's"
    );

    for (named, path, bytes) in the_named_things() {
        let found = std::fs::read(&path)
            .unwrap_or_else(|why| panic!("{named} at {} is gone: {why}", path.display()));
        assert!(
            found == bytes,
            "{named} at {} is not byte for byte what was written",
            path.display()
        );
        say(&format!(
            "alo-update-test: {named} is byte for byte as it was"
        ));
    }

    let record_at = Path::new("/var/lib/alo/record.jsonl");
    let kept_before = std::fs::read(Path::new(THE_TESTS_OWN).join("record-before")).unwrap();
    assert!(
        std::fs::read(record_at).unwrap() == kept_before,
        "the record is not byte for byte what it was"
    );
    say("alo-update-test: the record is byte for byte as it was");

    let mut record = Writing::opening(record_at).unwrap();
    assert_eq!(
        after_a_restart(
            &base,
            &AcrossRestarts::on_this_machine(),
            &mut record,
            SystemTime::now()
        )
        .unwrap(),
        Since::Updated {
            from: before.clone(),
            to: offered.clone(),
        }
    );
    let entries: Vec<Entry> = Reading::at(record_at)
        .unwrap()
        .record()
        .everything()
        .cloned()
        .collect();
    assert_eq!(entries.len(), 2, "{entries:?}");
    let updated = entries.last().unwrap();
    assert_eq!(updated.agent(), None);
    assert!(
        matches!(
            updated.happened(),
            Happened::Updated { from, to } if from.is(before.as_str()) && to.is(offered.as_str())
        ),
        "{updated:?}"
    );
    assert!(
        std::fs::read(record_at).unwrap().starts_with(&kept_before),
        "the record was rewritten rather than added to"
    );
    say(&format!(
        "alo-update-test: the record says the machine updated from {} to {}",
        before.as_str(),
        offered.as_str()
    ));

    assert_eq!(
        after_a_restart(
            &base,
            &AcrossRestarts::on_this_machine(),
            &mut record,
            SystemTime::now()
        )
        .unwrap(),
        Since::Unchanged(offered)
    );
    assert_eq!(
        Reading::at(record_at)
            .unwrap()
            .record()
            .everything()
            .count(),
        2,
        "one update became two entries"
    );
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
