//! Isolated upstream CUPS and IPP Everywhere processes for runtime acceptance.

#![expect(clippy::unwrap_used, reason = "fixture failures must fail acceptance")]

use std::fs::{self, File};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Private configuration and processes; never uses the host's printer queues.
pub struct Running {
    pub at: PathBuf,
    pub socket: PathBuf,
    pub printer: String,
    cups: Option<Child>,
    device: Option<Child>,
}

impl Running {
    /// Start the unmodified upstream server and printer emulator.
    pub fn start() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let at = std::env::temp_dir().join(format!("alo-cups-{}-{stamp}", std::process::id()));
        fs::create_dir(&at).unwrap();
        fs::set_permissions(&at, fs::Permissions::from_mode(0o755)).unwrap();
        for part in [
            "config", "spool", "cache", "state", "temp", "printer", "keys",
        ] {
            fs::create_dir(at.join(part)).unwrap();
            fs::set_permissions(at.join(part), fs::Permissions::from_mode(0o755)).unwrap();
        }
        let socket = at.join("state/cups.sock");
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let printer = format!("ipp://localhost:{port}/ipp/print");
        let mut this = Self {
            at,
            socket,
            printer,
            cups: None,
            device: None,
        };
        let log = File::create(this.at.join("printer.log")).unwrap();
        this.device = Some(
            Command::new("ippeveprinter")
                .args([
                    "-r",
                    "off",
                    "-n",
                    "localhost",
                    "-p",
                    &port.to_string(),
                    "-f",
                    "application/pdf,image/pwg-raster",
                    "-d",
                ])
                .arg(this.at.join("printer"))
                .arg("-K")
                .arg(this.at.join("keys"))
                .arg("alo runtime printer")
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        while TcpStream::connect(("127.0.0.1", port)).is_err() {
            assert!(Instant::now() < deadline, "{}", this.logs());
            std::thread::sleep(Duration::from_millis(50));
        }

        let config = fs::read_to_string("/etc/cups/cupsd.conf").unwrap();
        let mut private = String::new();
        for line in config.lines() {
            if line.trim_start().starts_with("Listen ") || line.trim_start().starts_with("Port ") {
                continue;
            }
            private.push_str(line);
            private.push('\n');
        }
        private.push_str(&format!(
            "\nListen {}\nWebInterface No\nLogLevel debug\n",
            this.socket.display()
        ));
        fs::write(this.at.join("cupsd.conf"), private).unwrap();
        let root = this.at.display();
        fs::write(
            this.at.join("cups-files.conf"),
            format!(
                "User lp\nGroup lp\nSystemGroup root\n\
             ServerRoot {root}/config\nRequestRoot {root}/spool\n\
             CacheDir {root}/cache\nStateDir {root}/state\nTempDir {root}/temp\n\
             ServerBin /usr/lib/cups\nDataDir /usr/share/cups\n\
             DocumentRoot /usr/share/cups/doc-root\n\
             ErrorLog {root}/error.log\nAccessLog {root}/access.log\nPageLog {root}/page.log\n"
            ),
        )
        .unwrap();
        let log = File::create(this.at.join("cups-process.log")).unwrap();
        this.cups = Some(
            Command::new("/usr/sbin/cupsd")
                .arg("-f")
                .arg("-c")
                .arg(this.at.join("cupsd.conf"))
                .arg("-s")
                .arg(this.at.join("cups-files.conf"))
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        while UnixStream::connect(&this.socket).is_err() {
            assert!(Instant::now() < deadline, "{}", this.logs());
            std::thread::sleep(Duration::from_millis(50));
        }
        this
    }

    /// Run the same acceptance binary without capabilities, optionally as nobody.
    pub fn check(&self, phase: &str, nobody: bool, test: &str) {
        // /root is not traversable by nobody; this private copy is executable
        // without granting access to the build directory.
        let binary = self.at.join("acceptance");
        if !binary.exists() {
            fs::copy(std::env::current_exe().unwrap(), &binary).unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut command = Command::new("setpriv");
        command.args([
            "--bounding-set=-all",
            "--inh-caps=-all",
            "--ambient-caps=-all",
            "--no-new-privs",
        ]);
        if nobody {
            command.args(["--reuid=65534", "--regid=65534", "--clear-groups"]);
        }
        let result = command
            .arg(&binary)
            .args(["--exact", test, "--ignored", "--nocapture"])
            .env("ALO_CUPS_RUNTIME_PHASE", phase)
            .env("ALO_CUPS_RUNTIME_SOCKET", &self.socket)
            .env("ALO_CUPS_RUNTIME_PRINTER", &self.printer)
            .current_dir(&self.at)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "phase {phase}: {}\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
            self.logs()
        );
        println!("capability-free {phase}: passed");
    }

    fn logs(&self) -> String {
        ["printer.log", "cups-process.log", "error.log"]
            .iter()
            .map(|name| {
                format!(
                    "{name}:\n{}",
                    fs::read_to_string(self.at.join(name)).unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        for process in [&mut self.cups, &mut self.device].into_iter().flatten() {
            let _ = process.kill();
            let _ = process.wait();
        }
        // Only this fixture's newly-created private directory is removed.
        let _ = fs::remove_dir_all(&self.at);
    }
}
