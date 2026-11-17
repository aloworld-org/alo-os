//! **A picture asked for at the compositor's door, and read back whole**
//! (`picture-of`, `docs/contracts/application-door.md`) — against a stand-in
//! compositor on a real socket.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use alo_acting::{Pictured, ToPicture};
use alo_applications::Application;
use alo_turn::{NotAtTheDoor, TheDoorTheCompositorServes as _};

use super::{AtTheCompositorsDoor, THE_COMPOSITORS_SOCKET};

/// A directory of this test's own, shallow for the socket path's bound.
fn a_directory(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-pic-{}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

fn of_the_game() -> ToPicture {
    ToPicture {
        application: Application::identified("net.supertuxkart.SuperTuxKart").unwrap(),
    }
}

/// A stand-in compositor that answers one connection with these bytes, and
/// reports the line it was handed.
fn a_compositor_answering(at: &Path, answer: Vec<u8>) -> mpsc::Receiver<String> {
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
        let _ = writing.write_all(&answer);
        let _ = writing.flush();
    });
    heard
}

/// **The line is the contract's, and the PNG comes back byte for byte**, as
/// long as the head says — even when it is larger than a socket holds.
#[test]
fn a_picture_comes_back_whole_after_its_head() {
    let at = a_directory("whole");
    let png: Vec<u8> = (0..3 * 1024 * 1024_u32)
        .map(|at| u8::try_from(at % 251).unwrap())
        .collect();
    let mut answer = format!("picture {} 1280 800\n", png.len()).into_bytes();
    answer.extend_from_slice(&png);
    let heard = a_compositor_answering(&at, answer);
    let mut door = AtTheCompositorsDoor::beside(&at);
    let (pictured, bytes) = door.picturing(&of_the_game()).unwrap();
    assert_eq!(
        heard.recv().unwrap(),
        "picture-of net.supertuxkart.SuperTuxKart\n"
    );
    assert_eq!(
        pictured,
        Pictured::Picture {
            bytes: png.len(),
            wide: 1280,
            tall: 800
        }
    );
    assert!(bytes == png, "the picture did not come back whole");
}

/// **A refusal comes back with no bytes.**
#[test]
fn a_refusal_comes_back_with_no_bytes() {
    let at = a_directory("refused");
    let _heard = a_compositor_answering(&at, b"not-in-front\n".to_vec());
    let mut door = AtTheCompositorsDoor::beside(&at);
    assert_eq!(
        door.picturing(&of_the_game()).unwrap(),
        (Pictured::NotInFront, Vec::new())
    );
}

/// **A picture cut short is not a picture**: fewer bytes than the head said
/// is an answer that did not finish, never a smaller picture.
#[test]
fn a_picture_cut_short_is_not_answered() {
    let at = a_directory("short");
    let _heard = a_compositor_answering(&at, b"picture 100 10 10\nonly this".to_vec());
    let mut door = AtTheCompositorsDoor::beside(&at);
    assert!(matches!(
        door.picturing(&of_the_game()),
        Err(NotAtTheDoor::NotAnswered { .. })
    ));
}

/// **A head this machine does not know is not read as a picture.**
#[test]
fn an_unknown_head_is_not_answered() {
    let at = a_directory("unknown");
    let _heard = a_compositor_answering(&at, b"picture lots\n".to_vec());
    let mut door = AtTheCompositorsDoor::beside(&at);
    assert!(matches!(
        door.picturing(&of_the_game()),
        Err(NotAtTheDoor::NotAnswered { .. })
    ));
}

/// **No compositor is no compositor**, not a failure.
#[test]
fn no_compositor_is_said_as_such() {
    let at = a_directory("none");
    let mut door = AtTheCompositorsDoor::beside(&at);
    assert_eq!(
        door.picturing(&of_the_game()),
        Err(NotAtTheDoor::NoCompositor)
    );
}
