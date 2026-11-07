//! **The real binary, against a desktop door of the test's own and a record
//! file of the test's own**: an act writes exactly one entry with what the
//! application sent, a look writes none, and a refusal writes one that keeps
//! nothing of what arrived.
//!
//! The door is a Unix socket answering the contract's words
//! (`docs/contracts/application-door.md`) — `counting`, `sent 4096`, `acted` —
//! and keeping every line it was asked, so the order is read off what crossed
//! the socket rather than off this crate. `ALO_APP_DOORS_IN` points the binary
//! at it; nothing else about the command is changed for the test.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

/// A folder of the test's own.
fn a_folder(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-app-{what}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// A desktop door that answers every act `acted` and every count `sent 4096`,
/// and keeps every line it was asked.
fn a_door_in(doors: &Path) -> Arc<Mutex<Vec<String>>> {
    let listener = UnixListener::bind(doors.join("shell.sock")).unwrap();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let keeping = Arc::clone(&heard);
    std::thread::spawn(move || {
        // **One thread per connection**, because a count holds its connection
        // open while the act it counts arrives on another.
        for stream in listener.incoming() {
            let Ok(stream) = stream else { break };
            let keeping = Arc::clone(&keeping);
            std::thread::spawn(move || {
                let mut reading = BufReader::new(stream.try_clone().unwrap());
                let mut writing = stream;
                let mut line = String::new();
                while reading.read_line(&mut line).unwrap_or(0) > 0 {
                    let asked = line.trim_end().to_owned();
                    keeping.lock().unwrap().push(asked.clone());
                    let answer = match asked.split(' ').next() {
                        Some("count") => "counting",
                        Some("done") => "sent 4096",
                        _ => "acted",
                    };
                    let _ = writeln!(writing, "{answer}");
                    line.clear();
                }
            });
        }
    });
    heard
}

/// A machine with the text editor installed, its doors, and a home.
struct AMachine {
    /// Where everything is.
    place: PathBuf,
    /// What the door heard.
    heard: Arc<Mutex<Vec<String>>>,
}

impl AMachine {
    /// A machine of the test's own, its door already answering.
    fn started(what: &str) -> Self {
        let place = a_folder(what);
        std::fs::create_dir_all(place.join("home")).unwrap();
        std::fs::create_dir_all(place.join("doors")).unwrap();
        let applications = place.join("share").join("applications");
        std::fs::create_dir_all(&applications).unwrap();
        std::fs::write(
            applications.join("org.gnome.TextEditor.desktop"),
            "[Desktop Entry]\nType=Application\nName=Text Editor\nExec=gnome-text-editor %U\n",
        )
        .unwrap();
        let heard = a_door_in(&place.join("doors"));
        Self { place, heard }
    }

    /// Run the command a person would type.
    fn typed(&self, line: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_alo-app"))
            .args(line)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", self.place.join("home"))
            .env("XDG_DATA_HOME", self.place.join("nowhere"))
            .env("XDG_DATA_DIRS", self.place.join("share"))
            .env("XDG_CONFIG_HOME", self.place.join("config"))
            .env("ALO_APP_DOORS_IN", self.place.join("doors"))
            .current_dir(&self.place)
            .output()
            .expect("the command this crate builds runs")
    }

    /// Every entry in the person's record.
    fn entries(&self) -> Vec<String> {
        let record = self.place.join("home/.local/state/alo/commands.jsonl");
        std::fs::read_to_string(record)
            .unwrap_or_default()
            .lines()
            .filter(|line| line.contains("asked-an-application"))
            .map(str::to_owned)
            .collect()
    }
}

impl Drop for AMachine {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.place);
    }
}

#[test]
fn an_act_writes_one_entry_with_what_was_sent_and_a_look_writes_none() {
    let machine = AMachine::started("act");

    let opened = machine.typed(&["texteditor", "open"]);
    assert_eq!(
        opened.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&opened.stderr)
    );
    let entries = machine.entries();
    assert_eq!(entries.len(), 1, "{entries:?}");
    let entry = entries.first().unwrap();
    assert!(
        entry.contains(r#""application":"org.gnome.TextEditor""#),
        "{entry}"
    );
    assert!(entry.contains(r#""road":"window""#), "{entry}");
    assert!(entry.contains("4096"), "the count was not kept: {entry}");
    assert!(
        !entry.contains("agent"),
        "a person's own act named an agent: {entry}"
    );

    // **Around the act, in this order**: the count begins, the act, the count ends.
    let heard = machine.heard.lock().unwrap().clone();
    assert_eq!(
        heard,
        [
            "count org.gnome.TextEditor",
            "open org.gnome.TextEditor",
            "done"
        ]
    );

    // **A look writes nothing.**
    for look in [&["list"][..], &["show", "texteditor"]] {
        let looked = machine.typed(look);
        assert_eq!(looked.status.code(), Some(0), "{look:?}");
        assert!(!looked.stdout.is_empty(), "{look:?} said nothing");
    }
    assert_eq!(machine.entries().len(), 1, "a look was written down");
    let listed = machine.typed(&["list"]);
    assert_eq!(
        String::from_utf8_lossy(&listed.stdout),
        "org.gnome.TextEditor\n"
    );
}

#[test]
fn a_refusal_writes_one_entry_with_nothing_of_what_arrived() {
    let machine = AMachine::started("refusal");
    let refused = machine.typed(&["texteditor", "press", "slider", "Volume-Secret"]);
    assert_eq!(refused.status.code(), Some(65));
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("is not a kind of control"),
        "{}",
        String::from_utf8_lossy(&refused.stderr)
    );
    let entries = machine.entries();
    assert_eq!(entries.len(), 1, "{entries:?}");
    let entry = entries.first().unwrap();
    assert!(entry.contains(r#""ability":"press""#), "{entry}");
    assert!(
        !entry.contains("Volume-Secret"),
        "a refusal kept what arrived: {entry}"
    );
    assert!(
        machine.heard.lock().unwrap().is_empty(),
        "the desktop was asked about a command that was refused"
    );

    let shapeless = machine.typed(&[]);
    assert_eq!(shapeless.status.code(), Some(64));
    assert_eq!(
        machine.entries().len(),
        1,
        "nothing was asked, and it was written down"
    );
}
