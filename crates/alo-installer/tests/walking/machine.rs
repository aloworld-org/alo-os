//! The machine: its firmware, its security chip, its two disks, and starting
//! and stopping it.
//!
//! Every destructive thing this test does happens inside a machine made here.
//! Nothing touches the host's own disks — the disks are files under
//! [`crate::walking::needs::THE_YARD`], and each walk runs on an overlay of the
//! installed Windows rather than on the installed Windows itself, so eight
//! kills cost eight boots and not eight installs.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use super::console::Console;
use super::needs::THE_FIRMWARE;

/// What the walk's SATA disks report as their serial number.
///
/// The serials are the point of the choice. Windows reports an AHCI disk with
/// bus `SATA`, model `QEMU HARDDISK` and this serial, and `naming.rs`'s SATA
/// rule then makes `ata-QEMU_HARDDISK_<serial>` — which is the name udev gives
/// the same disk under `/dev/disk/by-id/`. So the SATA row of `naming.rs`'s
/// table is checkable from both sides in this machine, without a physical disk.
pub const THE_WINDOWS_DISKS_SERIAL: &str = "ALOWINDOWS1";

/// The serial of the empty disk alo OS would be installed onto.
pub const THE_SECOND_DISKS_SERIAL: &str = "ALOTARGET1";

/// How big the Windows disk is.
pub const THE_WINDOWS_DISK: &str = "64G";

/// How big the empty second disk is — over `sizes::THE_LEAST_DISK`, so the
/// installer offers it rather than calling it too small.
pub const THE_SECOND_DISK: &str = "32G";

/// How much memory the guest is given.
///
/// Measured on this development PC, 2026-09-21: a guest of 4096 MiB beside a
/// release build took the host's free memory to 0.24 GB and wedged WSL's own
/// relay for over an hour while the machine inside it kept running
/// (`docs/quirks.md`). Three gibibytes is what Windows 11 wants and what this
/// host can spare.
pub const THE_MEMORY: &str = "3072";

/// What decides which device a machine starts.
enum Starting {
    /// The Windows disk first, through QEMU's own boot order, as every walk of
    /// Windows needs: the firmware otherwise falls through to the network.
    WindowsDiskFirst,
    /// Nothing put first, so the firmware's own variables decide every start;
    /// with these systemd credentials, by name and content.
    AsItsVariablesDecide(Vec<(String, String)>),
}

/// Standard base64, for a credential in the firmware's tables.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let byte = |at: usize| u32::from(chunk.get(at).copied().unwrap_or(0));
        let n = (byte(0) << 16) | (byte(1) << 8) | byte(2);
        for (i, shift) in [18_u32, 12, 6, 0].into_iter().enumerate() {
            let sextet = usize::try_from((n >> shift) & 63).unwrap_or(0);
            match ALPHABET.get(sextet) {
                Some(letter) if i <= chunk.len() => out.push(char::from(*letter)),
                _ => out.push('='),
            }
        }
    }
    out
}

/// A running machine.
#[derive(Debug)]
pub struct Machine {
    /// Where its files are.
    yard: PathBuf,
    /// Its monitor socket.
    monitor: PathBuf,
    /// QEMU itself, stopped and reaped when the machine is dropped.
    process: Child,
}

impl Machine {
    /// Everything under this name, made fresh: an overlay of the installed
    /// Windows, an empty second disk, and the firmware's variables as they were
    /// when Windows was installed.
    ///
    /// # Panics
    /// When a disk cannot be made.
    pub fn fresh(yard: &Path, name: &str, from_windows: &Path, from_variables: &Path) {
        for file in [
            Self::windows_of(yard, name),
            Self::second_of(yard, name),
            Self::variables_of(yard, name),
        ] {
            let _ = std::fs::remove_file(file);
        }
        run(
            "qemu-img",
            &[
                "create",
                "-f",
                "qcow2",
                "-b",
                &from_windows.display().to_string(),
                "-F",
                "qcow2",
                &Self::windows_of(yard, name).display().to_string(),
            ],
        );
        run(
            "qemu-img",
            &[
                "create",
                "-f",
                "qcow2",
                &Self::second_of(yard, name).display().to_string(),
                THE_SECOND_DISK,
            ],
        );
        std::fs::copy(from_variables, Self::variables_of(yard, name))
            .expect("the firmware's variables");
    }

    /// This machine's Windows disk. The machine with no name is the one
    /// Windows is installed into, whose disk every other is an overlay of.
    #[must_use]
    pub fn windows_of(yard: &Path, name: &str) -> PathBuf {
        yard.join(named(name, "windows.qcow2"))
    }

    /// This machine's empty second disk.
    #[must_use]
    pub fn second_of(yard: &Path, name: &str) -> PathBuf {
        yard.join(named(name, "target.qcow2"))
    }

    /// This machine's firmware variables, which is where a next-start choice
    /// lives across a restart.
    #[must_use]
    pub fn variables_of(yard: &Path, name: &str) -> PathBuf {
        yard.join(named(name, "VARS.fd"))
    }

    /// Start it, with this medium in its drive and this console on its serial
    /// line.
    ///
    /// **Nothing here says what to start.** What the firmware starts on the
    /// next restart is one of the things this task measures, so no boot order
    /// is given and the firmware's own variables decide.
    ///
    /// # Panics
    /// When the machine does not come up far enough to answer its monitor.
    #[must_use]
    pub fn start(
        yard: &Path,
        name: &str,
        medium: Option<&Path>,
        installing_from: Option<&Path>,
        console: &Console,
        chip: &SecurityChip,
    ) -> Self {
        Self::start_on(
            Path::new(THE_FIRMWARE),
            yard,
            name,
            medium,
            installing_from,
            console,
            chip,
        )
    }

    /// Start it on this firmware build rather than the walk's own — the road
    /// that has to see the environment start needs one that can start it
    /// (`crate::walking::firmware`).
    ///
    /// # Panics
    /// When the machine does not come up far enough to answer its monitor.
    #[must_use]
    pub fn start_on(
        firmware: &Path,
        yard: &Path,
        name: &str,
        medium: Option<&Path>,
        installing_from: Option<&Path>,
        console: &Console,
        chip: &SecurityChip,
    ) -> Self {
        Self::launch(
            firmware,
            yard,
            name,
            medium,
            installing_from,
            console,
            chip,
            &Starting::WindowsDiskFirst,
        )
    }

    /// Start it on this firmware with **no device put first**, so that every
    /// start — including the ones the machine makes itself — is decided by the
    /// firmware's own variables alone, as on a computer; and with these systemd
    /// credentials in the firmware's tables, which only a system that reads
    /// them acts on (the walk's installed alo OS).
    ///
    /// # Panics
    /// When the machine does not come up far enough to answer its monitor.
    #[must_use]
    pub fn start_as_its_variables_decide(
        firmware: &Path,
        yard: &Path,
        name: &str,
        medium: Option<&Path>,
        console: &Console,
        chip: &SecurityChip,
        credentials: &[(&str, &str)],
    ) -> Self {
        Self::launch(
            firmware,
            yard,
            name,
            medium,
            None,
            console,
            chip,
            &Starting::AsItsVariablesDecide(
                credentials
                    .iter()
                    .map(|(named, content)| ((*named).to_owned(), (*content).to_owned()))
                    .collect(),
            ),
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "each is one fact about the machine, and the two public ways in name them"
    )]
    fn launch(
        firmware: &Path,
        yard: &Path,
        name: &str,
        medium: Option<&Path>,
        installing_from: Option<&Path>,
        console: &Console,
        chip: &SecurityChip,
        starting: &Starting,
    ) -> Self {
        chip.answering();
        let first = match starting {
            Starting::WindowsDiskFirst => ",bootindex=1",
            Starting::AsItsVariablesDecide(_) => "",
        };
        let monitor = yard.join("monitor.sock");
        let _ = std::fs::remove_file(&monitor);
        let mut arguments: Vec<String> = [
            "-name",
            "alo-walk",
            "-accel",
            "kvm",
            "-cpu",
            "host",
            "-smp",
            "4",
            "-m",
            THE_MEMORY,
            "-machine",
            "q35,smm=on",
            "-global",
            "driver=cfi.pflash01,property=secure,value=on",
        ]
        .iter()
        .map(|word| (*word).to_owned())
        .collect();
        arguments.extend([
            "-drive".to_owned(),
            format!(
                "if=pflash,format=raw,unit=0,readonly=on,file={}",
                firmware.display()
            ),
            "-drive".to_owned(),
            format!(
                "if=pflash,format=raw,unit=1,file={}",
                Self::variables_of(yard, name).display()
            ),
            "-device".to_owned(),
            "ich9-ahci,id=ahci".to_owned(),
            "-drive".to_owned(),
            format!(
                "file={},if=none,id=d0,format=qcow2,cache=writeback",
                Self::windows_of(yard, name).display()
            ),
            "-device".to_owned(),
            format!("ide-hd,drive=d0,bus=ahci.0,serial={THE_WINDOWS_DISKS_SERIAL}{first}"),
            "-drive".to_owned(),
            format!(
                "file={},if=none,id=d1,format=qcow2,cache=writeback",
                Self::second_of(yard, name).display()
            ),
            "-device".to_owned(),
            format!("ide-hd,drive=d1,bus=ahci.1,serial={THE_SECOND_DISKS_SERIAL}"),
        ]);
        if let Some(windows) = installing_from {
            arguments.extend([
                "-drive".to_owned(),
                format!(
                    "file={},if=none,id=cd0,media=cdrom,readonly=on",
                    windows.display()
                ),
                "-device".to_owned(),
                "ide-cd,drive=cd0,bus=ahci.2,bootindex=0".to_owned(),
            ]);
        }
        if let Some(medium) = medium {
            arguments.extend([
                "-drive".to_owned(),
                format!(
                    "file={},if=none,id=cd1,media=cdrom,readonly=on",
                    medium.display()
                ),
                "-device".to_owned(),
                "ide-cd,drive=cd1,bus=ahci.3".to_owned(),
            ]);
        }
        arguments.extend(
            [
                "-netdev",
                "user,id=n0",
                "-device",
                "e1000e,netdev=n0",
                "-chardev",
            ]
            .iter()
            .map(|word| (*word).to_owned()),
        );
        arguments.extend([
            format!("socket,id=tpmc,path={}", chip.socket.display()),
            "-tpmdev".to_owned(),
            "emulator,id=tpm0,chardev=tpmc".to_owned(),
            "-device".to_owned(),
            "tpm-tis,tpmdev=tpm0".to_owned(),
            "-serial".to_owned(),
            format!("file:{}", console.path().display()),
            "-display".to_owned(),
            "none".to_owned(),
            "-monitor".to_owned(),
            format!("unix:{},server,nowait", monitor.display()),
        ]);
        if let Starting::AsItsVariablesDecide(credentials) = starting {
            for (named, content) in credentials {
                arguments.extend([
                    "-smbios".to_owned(),
                    format!(
                        "type=11,value=io.systemd.credential.binary:{named}={}",
                        base64(content.as_bytes())
                    ),
                ]);
            }
        }
        let log = std::fs::File::create(yard.join("qemu.log")).expect("a place for QEMU's own log");
        let process = Command::new("qemu-system-x86_64")
            .args(&arguments)
            .stdin(std::process::Stdio::null())
            .stdout(log.try_clone().expect("the log again"))
            .stderr(log)
            .spawn()
            .expect("the machine");
        let until = Instant::now() + Duration::from_secs(30);
        while Instant::now() < until && !monitor.exists() {
            std::thread::sleep(Duration::from_millis(250));
        }
        assert!(monitor.exists(), "the machine never answered its monitor");
        Self {
            yard: yard.to_path_buf(),
            monitor,
            process,
        }
    }

    /// One word to the machine's monitor, and what it printed.
    pub fn told(&self, what: &str) -> String {
        let printed = Command::new("socat")
            .arg("-")
            .arg(format!("UNIX-CONNECT:{}", self.monitor.display()))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut talking| {
                use std::io::Write as _;
                talking
                    .stdin
                    .as_mut()
                    .expect("the monitor's ear")
                    .write_all(format!("{what}\n").as_bytes())?;
                talking.wait_with_output()
            });
        printed.map_or_else(
            |_| String::new(),
            |out| String::from_utf8_lossy(&out.stdout).into_owned(),
        )
    }

    /// **Hold a key down from the first second.** The Windows media's
    /// *press any key to boot from CD* times out in about five seconds and the
    /// firmware then falls through to the network, where it waits for ever.
    pub fn hold_a_key_down(&self, seconds: u64) {
        let until = Instant::now() + Duration::from_secs(seconds);
        while Instant::now() < until {
            let _ = self.told("sendkey ret");
            std::thread::sleep(Duration::from_millis(250));
        }
    }

    /// What the screen shows, kept beside the logs as a picture.
    pub fn screen(&self, called: &str) -> PathBuf {
        let shot = self.yard.join(format!("{called}.ppm"));
        let _ = std::fs::remove_file(&shot);
        let _ = self.told(&format!("screendump {}", shot.display()));
        let until = Instant::now() + Duration::from_secs(10);
        while Instant::now() < until
            && std::fs::metadata(&shot).map(|it| it.len()).unwrap_or(0) == 0
        {
            std::thread::sleep(Duration::from_millis(250));
        }
        shot
    }

    /// Start it again from the firmware, the way its reset button does.
    pub fn reset(&self) {
        let _ = self.told("system_reset");
    }

    /// Ask it to shut down the way its power button does, and wait.
    ///
    /// A walk's guest shuts itself down when it has done what it was told, so
    /// this is usually a request to a machine already on its way off.
    pub fn shut_down(&mut self, patience: Duration) -> bool {
        let _ = self.told("system_powerdown");
        self.has_stopped_within(patience)
    }

    /// Wait for the machine to stop by itself, and say whether it did.
    pub fn has_stopped_within(&mut self, patience: Duration) -> bool {
        let until = Instant::now() + patience;
        while Instant::now() < until {
            if self.process.try_wait().is_ok_and(|ended| ended.is_some()) {
                return true;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        false
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Whether a machine is running.
///
/// **`pkill -x qemu-system-x86_64` matches nothing**: Linux truncates a
/// process's `comm` to fifteen characters, so the name to match is
/// `qemu-system-x86` (`docs/quirks.md`). `pkill -f` is worse than useless —
/// it matches the shell running the `pkill`.
#[must_use]
pub fn is_running() -> bool {
    Command::new("pgrep")
        .args(["-x", "qemu-system-x86"])
        .output()
        .is_ok_and(|ran| ran.status.success())
}

/// Stop whatever machine is running, so the next start is not refused the
/// write lock on a disk.
pub fn stop() {
    let _ = Command::new("pkill")
        .args(["-x", "qemu-system-x86"])
        .output();
    let until = Instant::now() + Duration::from_secs(20);
    while Instant::now() < until && is_running() {
        std::thread::sleep(Duration::from_millis(500));
    }
    if is_running() {
        let _ = Command::new("pkill")
            .args(["-9", "-x", "qemu-system-x86"])
            .output();
        std::thread::sleep(Duration::from_secs(2));
    }
}

/// The guest's own security chip, which Windows 11 asks for.
///
/// **`swtpm` ends when the machine it served disconnects** — at every
/// shutdown — so a chip is started again, on the same state, before each
/// machine that uses it ([`Self::answering`]). Measured on 2026-09-22: a test
/// that started one `swtpm` and two machines on it found the second refused
/// with *Failed to connect to tpm.sock: No such file or directory*.
#[derive(Debug)]
pub struct SecurityChip {
    /// Where QEMU talks to it.
    pub socket: PathBuf,
    /// Where it keeps itself between machines.
    state: PathBuf,
    /// `swtpm` itself, while one is running.
    process: std::sync::Mutex<Option<Child>>,
}

impl Drop for SecurityChip {
    fn drop(&mut self) {
        if let Ok(mut running) = self.process.lock()
            && let Some(mut process) = running.take()
        {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

impl SecurityChip {
    /// A chip of its own, made fresh, and answering.
    ///
    /// # Panics
    /// When it does not come up.
    #[must_use]
    pub fn fresh(yard: &Path) -> Self {
        let state = yard.join("tpm");
        let socket = yard.join("tpm.sock");
        // A chip left over from a machine stopped hard held the next machine
        // before its firmware printed a line (2026-09-21), so every machine
        // gets its own. `pkill -x`, never `-f`: `-f` matches the shell that
        // runs it.
        let _ = Command::new("pkill").args(["-x", "swtpm"]).output();
        std::thread::sleep(Duration::from_secs(1));
        let _ = std::fs::remove_dir_all(&state);
        let _ = std::fs::remove_file(&socket);
        std::fs::create_dir_all(&state).expect("somewhere for the chip to keep itself");
        run(
            "swtpm_setup",
            &[
                "--tpm2",
                "--tpmstate",
                &state.display().to_string(),
                "--create-ek-cert",
                "--create-platform-cert",
                "--lock-nvram",
            ],
        );
        let chip = Self {
            socket,
            state,
            process: std::sync::Mutex::new(None),
        };
        chip.answering();
        chip
    }

    /// Make sure `swtpm` is running on this chip's state, starting it again
    /// when the last machine's shutdown ended it.
    ///
    /// # Panics
    /// When it does not come up.
    pub fn answering(&self) {
        let mut running = self.process.lock().expect("the chip");
        let alive = running
            .as_mut()
            .is_some_and(|process| process.try_wait().is_ok_and(|ended| ended.is_none()));
        if alive && self.socket.exists() {
            return;
        }
        if let Some(mut process) = running.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        let _ = std::fs::remove_file(&self.socket);
        let process = Command::new("swtpm")
            .args([
                "socket",
                "--tpmstate",
                &format!("dir={}", self.state.display()),
                "--ctrl",
                &format!("type=unixio,path={}", self.socket.display()),
                "--tpm2",
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("the chip");
        let until = Instant::now() + Duration::from_secs(20);
        while Instant::now() < until && !self.socket.exists() {
            std::thread::sleep(Duration::from_millis(250));
        }
        assert!(self.socket.exists(), "the security chip never answered");
        *running = Some(process);
    }
}

/// A file of the machine called `name`.
fn named(name: &str, what: &str) -> String {
    if name.is_empty() {
        what.to_owned()
    } else {
        format!("{name}-{what}")
    }
}

/// Run a program, and fail with what it said when it fails.
///
/// # Panics
/// When the program cannot be started or does not succeed.
pub fn run(program: &str, arguments: &[&str]) -> String {
    let ran = Command::new(program)
        .args(arguments)
        .output()
        .unwrap_or_else(|why| panic!("{program} could not be started: {why}"));
    assert!(
        ran.status.success(),
        "{program} {arguments:?} failed: {}{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8_lossy(&ran.stdout).into_owned()
}
