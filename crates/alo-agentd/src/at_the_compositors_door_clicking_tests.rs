//! **A click asked for at the compositor's door, and its one word read back**
//! (`click-at`, `docs/contracts/application-door.md`) — against a stand-in
//! compositor on a real socket.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use alo_acting::{Clicked, ToClick};
use alo_applications::Application;
use alo_turn::{NotAtTheDoor, TheDoorTheCompositorServes as _};

use super::{AtTheCompositorsDoor, THE_COMPOSITORS_SOCKET};

/// A directory of this test's own, shallow for the socket path's bound.
fn a_directory(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-clk-{}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

fn in_the_game() -> ToClick {
    ToClick::at(
        Application::identified("net.supertuxkart.SuperTuxKart").unwrap(),
        500,
        400,
    )
    .unwrap()
}

/// A stand-in compositor that answers one connection with these bytes, and
/// reports the line it was handed.
fn a_compositor_answering(at: &Path, answer: &'static [u8]) -> mpsc::Receiver<String> {
    let listening = UnixListener::bind(at.join(THE_COMPOSITORS_SOCKET)).unwrap();
    let (telling, heard) = mpsc::channel();
    std::thread::spawn(move || {
        let Ok((talking, _)) = listening.accept() else {
            return;
        };
        let mut line = String::new();
        if BufReader::new(&talking).read_line(&mut line).is_err() {
            return;
        }
        let _ = telling.send(line);
        let mut writing = &talking;
        let _ = writing.write_all(answer);
        let _ = writing.flush();
    });
    heard
}

/// **The line is the contract's, and each word comes back as itself.**
#[test]
fn the_line_is_the_contracts_and_every_answer_is_read() {
    for (said, answer) in [
        (&b"clicked\n"[..], Clicked::Clicked),
        (b"not-in-front\n", Clicked::NotInFront),
        (b"not-alos-copy\n", Clicked::NotAlosCopy),
        (b"the-person-is-pressing\n", Clicked::ThePersonIsPressing),
        (b"could-not-click\n", Clicked::CouldNotClick),
    ] {
        let at = a_directory(answer.written());
        let heard = a_compositor_answering(&at, said);
        let mut door = AtTheCompositorsDoor::beside(&at);
        assert_eq!(door.clicking(&in_the_game()).unwrap(), answer);
        assert_eq!(
            heard.recv().unwrap(),
            "click-at net.supertuxkart.SuperTuxKart 500 400\n"
        );
    }
}

/// **A word this machine does not know is never read as a click**, and a
/// compositor that says nothing is not one that clicked.
#[test]
fn an_unknown_word_or_silence_is_not_a_click() {
    for (what, said) in [("unknown", &b"acted\n"[..]), ("silent", b"")] {
        let at = a_directory(what);
        let _heard = a_compositor_answering(&at, said);
        let mut door = AtTheCompositorsDoor::beside(&at);
        assert!(
            matches!(
                door.clicking(&in_the_game()),
                Err(NotAtTheDoor::NotAnswered { .. })
            ),
            "{what}"
        );
    }
}
