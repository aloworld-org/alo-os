//! **The screen reader the image ships, and the rule that what it says stays on
//! this computer** — [ADR 0151](../../../docs/decisions/0151-alo-ships-a-screen-reader-and-its-voices-stay-on-the-machine.md).
//!
//! It reads `image/Containerfile` and the two unit files and nothing else, so it
//! runs in the ordinary gates on any machine.
//!
//! # What this proves, and what it cannot
//!
//! It proves **the recipe still asks for the reader and still checks it the way
//! ADR 0151 says**: Orca loaded, and started the way the session starts it and
//! still running fifteen seconds later, a word spoken in each of the 24 languages, one
//! speech module and it local, and the unit unable to open an internet socket.
//! It cannot prove a machine speaks, or that nothing leaves it while the reader
//! runs: the image build runs the checks this test holds it to, and a booted
//! machine measured at its network boundary is the only proof of the second.

#![expect(
    clippy::panic,
    reason = "an unreadable file is the failure this test reports, and a formatted panic names \
              the path it looked at"
)]

use std::path::{Path, PathBuf};

use alo_image::THE_IMAGE;

/// The unit the desktop starts.
const THE_UNIT: &str = "usr/lib/systemd/user/alo-screen-reader.service";

/// Speech Dispatcher's own unit, held to the same rule.
const THE_SPEECH_DROP_IN: &str =
    "usr/lib/systemd/user/speech-dispatcher.service.d/50-alo-speech-stays-on-this-computer.conf";

/// A file of the image, as one string.
fn read(at: &Path) -> String {
    std::fs::read_to_string(at)
        .unwrap_or_else(|why| panic!("{} is not readable: {why}", at.display()))
}

/// Where a file of the image is.
fn of_the_image(path: &str) -> PathBuf {
    Path::new(THE_IMAGE).join(path)
}

/// The lines a shell or systemd would act on: comments and blank lines dropped,
/// so a sentence explaining a rule never stands in for the rule.
fn acted_on(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}

/// The recipe's block that installs the reader, from its `RUN` to the next.
fn the_readers_block() -> String {
    let recipe = read(&of_the_image("Containerfile"));
    let block = recipe
        .split("\nRUN ")
        .find(|block| block.contains("speech-dispatcher-espeak-ng"))
        .unwrap_or_else(|| panic!("the recipe no longer installs the screen reader's voice"));
    acted_on(block).join("\n")
}

/// **Every engine the reader needs is installed, by name** — including the two
/// that a successful `dnf install` leaves out: `python3-dbus`, without which
/// Orca does not start, and `pipewire-pulseaudio`, without which its voice has
/// nowhere to go.
#[test]
fn the_recipe_installs_the_reader_its_voice_and_where_the_sound_goes() {
    let block = the_readers_block();
    for package in [
        "orca",
        "python3-dbus",
        "espeak-ng",
        "speech-dispatcher",
        "speech-dispatcher-espeak-ng",
        "at-spi2-core",
        "pipewire-pulseaudio",
    ] {
        assert!(
            block.split_whitespace().any(|word| word == package),
            "the screen reader's block no longer installs `{package}`"
        );
    }
}

/// **Orca is loaded, not stat-ed**, and the floor is checked: 48 is the first
/// release whose keyboard is `Atspi.Device`'s. That it *starts* is the next
/// test's.
#[test]
fn orca_is_loaded_and_held_to_a_floor() {
    let block = the_readers_block();
    assert!(
        block.contains("orca --version"),
        "nothing runs Orca in the build, so a reader whose modules do not load would ship"
    );
    assert!(block.contains("least=48"), "the floor of Orca 48 is gone");
    assert!(
        !block.contains("test -x /usr/bin/orca"),
        "an executable bit stands in for Orca starting"
    );
}

/// The recipe's block that starts the reader as the session does.
fn the_starting_block() -> (String, usize) {
    let recipe = read(&of_the_image("Containerfile"));
    let at = recipe.find("--socket=alo-reader-check").unwrap_or_else(|| {
        panic!("the recipe no longer starts the screen reader as a session does")
    });
    let block = recipe
        .split("\nRUN ")
        .find(|block| block.contains("--socket=alo-reader-check"))
        .unwrap_or_else(|| panic!("the reader's start is not in a RUN of its own"));
    (acted_on(block).join("\n"), at)
}

/// **Orca is started the way the session starts it, and has to stay up.**
///
/// `orca --version` answered on every build while Orca 48.9 fell over at every
/// start on a booted machine (`docs/misreadings/orca-version-answered-is-not-orca-started.md`).
/// So the build runs the unit's own command with the unit's own environment,
/// under a Wayland compositor with no X display, a session bus and the
/// accessibility bus — and fails unless the reader is still running fifteen
/// seconds later with no traceback. And what it installed to do so is gone.
#[test]
fn orca_is_started_the_way_the_session_starts_it() {
    let (block, at) = the_starting_block();
    for (must, why) in [
        (
            "sed -n 's/^ExecStart=//p' \"${unit}\"",
            "the check does not run the command the unit runs",
        ),
        (
            "sed -n 's/^Environment=//p' \"${unit}\"",
            "the check does not give the reader the environment the unit gives it",
        ),
        (
            "unit=/usr/lib/systemd/user/alo-screen-reader.service",
            "the check reads some other unit than the one Settings starts",
        ),
        (
            "weston --backend=headless",
            "the reader is not started under a Wayland compositor",
        ),
        (
            "env -u DISPLAY",
            "the reader is started with an X display alo's session does not have",
        ),
        (
            "dbus-run-session --",
            "the reader is started with no session bus, so it cannot reach the accessibility bus",
        ),
        ("sleep 15;", "the reader is not given time to fall over"),
        (
            "kill -0 \"${reader}\"",
            "nothing asks whether the reader is still running",
        ),
        (
            "Traceback|Fatal Python error",
            "a reader that printed a traceback would pass",
        ),
        ("exit 1;", "a reader that fell over does not fail the build"),
        (
            "rpm -e ${added}",
            "what the check installed is left in the image",
        ),
        ("! command -v weston", "the check's compositor ships"),
    ] {
        assert!(block.contains(must), "{why}");
    }
    let recipe = read(&of_the_image("Containerfile"));
    let the_unit_arrives = recipe
        .find("COPY image/usr/lib/systemd/user/ /usr/lib/systemd/user/")
        .unwrap_or_else(|| panic!("the recipe no longer puts the user units in the image"));
    assert!(
        the_unit_arrives < at,
        "the reader is started before its unit is in the image, so the check reads nothing"
    );
}

/// **A word is spoken in each of the 24 languages**, and the 24 are the table's,
/// so a language the table says has a voice is a language the build hears.
#[test]
fn the_voice_speaks_each_language_the_table_says_it_has() {
    let block = the_readers_block();
    let the_loop = block
        .lines()
        .find(|line| line.starts_with("&& for language in"))
        .unwrap_or_else(|| panic!("the recipe no longer speaks each language"));
    let spoken: Vec<&str> = the_loop
        .trim_start_matches("&& for language in")
        .trim_end_matches("; do \\")
        .split_whitespace()
        .collect();
    let with_a_voice: Vec<&str> = alo_access::A_VOICE_FOR_EACH
        .iter()
        .filter(|(language, _)| alo_access::has_a_voice(language))
        .map(|(language, _)| *language)
        .collect();
    assert_eq!(spoken, with_a_voice);
    assert!(
        block.contains("espeak-ng -v \"${language}\" -w"),
        "each language is named in the loop and nothing makes it speak"
    );
    assert!(
        block.contains("test -s /tmp/alo-said.wav"),
        "a voice that wrote nothing would pass"
    );
}

/// **No voice that fetches speech from a server can ship**: exactly one speech
/// module, eSpeak NG, and Speech Dispatcher on a local socket.
#[test]
fn only_the_local_voice_is_loaded_and_speech_is_not_on_the_network() {
    let block = the_readers_block();
    assert!(
        block.contains(
            "test \"$(grep -cE '^[[:space:]]*AddModule' /etc/speech-dispatcher/speechd.conf)\" = 1"
        ),
        "nothing holds Speech Dispatcher to one module, so a network voice could be loaded"
    );
    assert!(
        block.contains("AddModule[[:space:]]+\"espeak-ng\""),
        "the one module is not checked to be eSpeak NG"
    );
    assert!(
        block.contains("! grep -qE '^[[:space:]]*CommunicationMethod[[:space:]]+\"?inet_socket'"),
        "nothing refuses Speech Dispatcher listening on the network"
    );
}

/// **The reader cannot open an internet socket**, and neither can Speech
/// Dispatcher when its own socket starts it.
///
/// This is the configuration ADR 0151 §3 uses instead of a patch: Orca fetches
/// a link's address to say its size, and with no internet socket that fetch
/// fails and nothing leaves.
#[test]
fn neither_the_reader_nor_its_speech_can_reach_a_network() {
    for path in [THE_UNIT, THE_SPEECH_DROP_IN] {
        let text = read(&of_the_image(path));
        let lines = acted_on(&text);
        assert!(
            lines.contains(&"RestrictAddressFamilies=AF_UNIX"),
            "{path} can open an internet socket, so what the reader fetches could leave"
        );
        assert!(
            !lines
                .iter()
                .any(|line| line.starts_with("RestrictAddressFamilies=")
                    && *line != "RestrictAddressFamilies=AF_UNIX"),
            "{path} widens what it may reach"
        );
    }
}

/// **The unit starts Orca and nothing starts the unit but the desktop**: no
/// `[Install]`, so no sign-in reads the screen aloud for somebody who did not
/// switch it on.
#[test]
fn the_unit_is_started_only_by_the_persons_switch() {
    let text = read(&of_the_image(THE_UNIT));
    let lines = acted_on(&text);
    assert!(lines.contains(&"ExecStart=/usr/bin/orca --replace"));
    assert!(
        !lines.contains(&"[Install]"),
        "the screen reader would start for everybody, switched on or not"
    );
    assert!(
        lines.contains(&"EnvironmentFile=-%t/alo-screen-reader.env"),
        "the language the person reads no longer reaches the reader"
    );
}

/// **The unit and its drop-in reach the machine**: made readable in the build
/// like every other unit, so a file in the tree is a file in the image.
#[test]
fn the_units_are_put_in_the_image() {
    let recipe = read(&of_the_image("Containerfile"));
    assert!(recipe.contains("COPY image/usr/lib/systemd/user/ /usr/lib/systemd/user/"));
    for path in [THE_UNIT, THE_SPEECH_DROP_IN] {
        assert!(
            recipe.contains(&format!("/{path}")),
            "the recipe does not name {path}"
        );
    }
}
