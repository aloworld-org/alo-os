//! **Put back**, from the record window, on this machine: where this person's
//! places to return to are, and the request to the service that writes the
//! record (`put-back` on the person's own door), asked on a thread of its own.
//!
//! **On a thread**, because the service copies each folder out of the place
//! before it goes back, and a large one takes as long as its copy takes — a
//! frame loop that waited would stop every window on the machine drawing. The
//! answer comes back over a channel the frame loop asks once a frame.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use alo_protocol::{FromAPerson, ToAPerson};

/// How long the service is waited for: a folder's copy, not a network.
const PATIENCE: Duration = Duration::from_secs(30 * 60);

/// Where this person's places are kept, and their home — [`None`] on a
/// session with no home, which then offers no *Put back*.
pub(crate) fn where_the_places_are() -> Option<(PathBuf, PathBuf)> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    home.is_absolute().then(|| {
        (
            alo_as_it_was::where_this_person_keeps(alo_remembering::this_person()),
            home,
        )
    })
}

/// Ask the service to put `folders` back from `place`, and the channel its
/// answer comes over: the sentences it said, or why nothing was put back.
pub(crate) fn asked(place: String, folders: Vec<String>) -> Receiver<Result<Vec<String>, String>> {
    let (tell, told) = channel();
    let door = alo_changing::this_persons_door();
    let started = std::thread::Builder::new()
        .name("putting-back".to_owned())
        .spawn(move || {
            let _ = tell.send(the_service_puts_back(&door, place, folders));
        });
    if let Err(why) = started {
        eprintln!("alo-desktop: nothing could ask the service to put folders back: {why}");
    }
    told
}

/// One conversation on the person's door: the request, and its answer.
fn the_service_puts_back(
    door: &Path,
    place: String,
    folders: Vec<String>,
) -> Result<Vec<String>, String> {
    let stream = UnixStream::connect(door).map_err(|why| why.to_string())?;
    stream
        .set_read_timeout(Some(PATIENCE))
        .map_err(|why| why.to_string())?;
    let line = FromAPerson::PutBack { place, folders }
        .written()
        .map_err(|why| why.to_string())?;
    (&stream)
        .write_all(line.as_bytes())
        .and_then(|()| (&stream).write_all(b"\n"))
        .map_err(|why| why.to_string())?;
    let mut answer = String::new();
    BufReader::new(&stream)
        .read_line(&mut answer)
        .map_err(|why| why.to_string())?;
    match ToAPerson::read(answer.trim_end()).map_err(|why| why.to_string())? {
        ToAPerson::PutBack { said } => Ok(said
            .iter()
            .map(|wording| wording.text().to_owned())
            .collect()),
        ToAPerson::Refused(wording) => Err(wording.text().to_owned()),
        other => Err(format!("the service answered something else: {other:?}")),
    }
}
