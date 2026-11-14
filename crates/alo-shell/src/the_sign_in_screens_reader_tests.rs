//! The sign-in screen's door to its reader: the sign-in login is let in and
//! told the screen; **anybody else is shut out before a byte is sent**; a
//! reader that is not there is said to be, and is told everything when it
//! comes.

use std::io::{BufRead as _, BufReader};
use std::path::PathBuf;

use alo_strings::Strings;

use super::*;
use crate::SignInShows;
use crate::the_first_start::TheFirstStart;

/// A folder of this test's own.
fn a_folder() -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-sign-in-reading-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// This test's own uid, which is who every connection here comes from.
fn me() -> u32 {
    rustix::process::getuid().as_raw()
}

/// The first screen of a fresh machine, as a reader is told it.
fn the_first_screen() -> ReadAloudTree {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::sign_in_words::declare_sign_in_words(&mut vocabulary).unwrap();
    crate::first_start_words::declare_first_start_words(&mut vocabulary).unwrap();
    let first_start = TheFirstStart::arriving(None);
    ReadAloudTree::before_sign_in(
        &Strings::of(vocabulary),
        SignInShows::FirstStart(&first_start),
    )
}

/// A reader connected at `door`, reading with a short patience.
fn a_reader(door: &Path) -> BufReader<UnixStream> {
    let stream = UnixStream::connect(door).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .unwrap();
    BufReader::new(stream)
}

/// The next line the reader is sent, or [`None`] where nothing comes.
fn the_next(reader: &mut BufReader<UnixStream>) -> Option<ToTheReader> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(ToTheReader::read(&line).unwrap()),
    }
}

/// **The sign-in login is let in and told the screen** — and told it again
/// only when it changes.
#[test]
fn the_sign_in_login_is_let_in_and_told_the_screen() {
    let folder = a_folder();
    let door = folder.join("reading");
    let mut screens = TheSignInScreensReader::at(&door, me());
    let mut reader = a_reader(&door);
    let tree = the_first_screen();
    screens.following(|| tree.clone());
    assert_eq!(
        the_next(&mut reader),
        Some(ToTheReader::TheScreen(tree.clone()))
    );
    screens.following(|| tree.clone());
    assert_eq!(
        the_next(&mut reader),
        None,
        "the same screen is not sent twice"
    );
    let _ = std::fs::remove_dir_all(folder);
}

/// **Anybody but the sign-in login is shut out**, before a byte is sent.
#[test]
fn anybody_else_is_shut_out() {
    let folder = a_folder();
    let door = folder.join("reading");
    let mut screens = TheSignInScreensReader::at(&door, me().wrapping_add(1));
    let mut reader = a_reader(&door);
    let tree = the_first_screen();
    screens.following(|| tree.clone());
    screens.turned_over(Some("de".to_owned()));
    assert_eq!(
        the_next(&mut reader),
        None,
        "nothing was sent to a stranger"
    );
    assert_eq!(
        screens.answered(),
        Some(TheScreenReaderAnswered::DidNotStart),
        "with nobody let in, the reader is said not to have started"
    );
    let _ = std::fs::remove_dir_all(folder);
}

/// **Turned on with no reader there, it is said not to have started — and the
/// choice stands**, so a reader that comes later is told to read, in the
/// language chosen, and is sent the whole screen.
#[test]
fn a_reader_that_comes_late_is_told_everything() {
    let folder = a_folder();
    let door = folder.join("reading");
    let mut screens = TheSignInScreensReader::at(&door, me());
    screens.turned_over(Some("fr".to_owned()));
    assert_eq!(
        screens.answered(),
        Some(TheScreenReaderAnswered::DidNotStart)
    );
    assert!(
        screens.is_reading(),
        "the person's choice stays as they made it"
    );

    let mut reader = a_reader(&door);
    let tree = the_first_screen();
    screens.following(|| tree.clone());
    assert_eq!(
        the_next(&mut reader),
        Some(ToTheReader::Read {
            on: true,
            language: Some("fr".to_owned()),
        })
    );
    assert_eq!(the_next(&mut reader), Some(ToTheReader::TheScreen(tree)));
}

/// **What the reader answers reaches the screen**, and a line that is not a
/// message is ignored rather than acted on.
#[test]
fn what_the_reader_answers_reaches_the_screen() {
    let folder = a_folder();
    let door = folder.join("reading");
    let mut screens = TheSignInScreensReader::at(&door, me());
    let reader = a_reader(&door);
    screens.following(the_first_screen);
    let mut stream = reader.into_inner();
    stream.write_all(b"{\"run\": \"/bin/sh\"}\n").unwrap();
    stream
        .write_all(
            FromTheReader::Answered(TheScreenReaderAnswered::Started)
                .written()
                .as_bytes(),
        )
        .unwrap();
    std::thread::sleep(Duration::from_millis(100));
    screens.following(the_first_screen);
    assert_eq!(screens.answered(), Some(TheScreenReaderAnswered::Started));
    assert_eq!(screens.answered(), None);
    let _ = std::fs::remove_dir_all(folder);
}

/// **At sign-in the reader is asked to step aside**: one that says it has is
/// waited for, one that never answers is waited for no longer than the
/// patience, and with no reader there is nothing to wait for.
#[test]
fn at_sign_in_the_reader_steps_aside_or_is_not_waited_for() {
    let folder = a_folder();
    let door = folder.join("reading");

    let mut nobody = TheSignInScreensReader::at(&door, me());
    assert!(nobody.steps_aside(Duration::from_secs(5)));
    drop(nobody);

    let mut screens = TheSignInScreensReader::at(&door, me());
    let mut reader = a_reader(&door);
    screens.following(the_first_screen);
    let _screen = the_next(&mut reader);
    let answering = std::thread::spawn(move || {
        let asked = the_next(&mut reader);
        let mut stream = reader.into_inner();
        stream
            .write_all(FromTheReader::SteppedAside.written().as_bytes())
            .unwrap();
        asked
    });
    assert!(screens.steps_aside(Duration::from_secs(5)));
    assert_eq!(
        answering.join().unwrap(),
        Some(ToTheReader::SomebodyIsSigningIn)
    );
    drop(screens);

    let mut screens = TheSignInScreensReader::at(&door, me());
    let _silent = a_reader(&door);
    screens.following(the_first_screen);
    let started = Instant::now();
    assert!(!screens.steps_aside(Duration::from_millis(300)));
    assert!(started.elapsed() < Duration::from_secs(3));
    let _ = std::fs::remove_dir_all(folder);
}

/// A file of the image, as the lines tmpfiles or sysusers act on.
fn of_the_image(path: &str) -> Vec<String> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../image")
        .join(path);
    std::fs::read_to_string(at)
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// **The uid let in is the sign-in login's, and the door is in the folder the
/// image makes for it** — two files that must agree with this one, held to it
/// here so a number changed in one place fails rather than locking the reader
/// out of a machine.
#[test]
fn the_door_and_the_login_are_the_ones_the_image_makes() {
    let login = of_the_image("usr/lib/sysusers.d/alo.conf");
    assert!(
        login
            .iter()
            .any(|line| line.starts_with(&format!("u alo-greeter {THE_SIGN_IN_LOGIN}:"))),
        "alo-greeter is not uid {THE_SIGN_IN_LOGIN} in the image"
    );
    let folder = Path::new(THE_READING_DOOR)
        .parent()
        .unwrap()
        .display()
        .to_string();
    assert!(
        of_the_image("usr/lib/tmpfiles.d/alo.conf")
            .iter()
            .any(|line| *line == format!("d {folder} 0750 root alo-greeter -")),
        "the image does not make {folder} for the sign-in login alone"
    );
}
