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

/// Why nothing was put back, before it is said.
#[derive(Debug)]
pub(crate) enum NotPutBack {
    /// The service refused, in the person's words already.
    Refused(String),
    /// The service could not be asked, or answered something that was not an
    /// answer to this — said with the sentence `alo-back` says for the same
    /// thing, and what the system said, if anything, filled in.
    NotReached(String),
}

impl NotPutBack {
    /// The sentence the confirmation shows, in `strings`' language.
    pub(crate) fn said(self, strings: &alo_strings::Strings) -> String {
        match self {
            Self::Refused(said) => said,
            Self::NotReached(why) => strings
                .say(
                    &alo_as_it_was::words::NOT_WITHOUT_THE_RECORD.key(),
                    &alo_strings::Filling::of("why", why),
                )
                .into_text(),
        }
    }
}

/// Ask the service to put `folders` back from `place`, and the channel its
/// answer comes over: the sentences it said, or why nothing was put back.
pub(crate) fn asked(
    place: String,
    folders: Vec<String>,
) -> Receiver<Result<Vec<String>, NotPutBack>> {
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
) -> Result<Vec<String>, NotPutBack> {
    let unreached = |why: std::io::Error| NotPutBack::NotReached(why.to_string());
    let stream = UnixStream::connect(door).map_err(unreached)?;
    stream.set_read_timeout(Some(PATIENCE)).map_err(unreached)?;
    let line = FromAPerson::PutBack { place, folders }
        .written()
        .map_err(|why| NotPutBack::NotReached(why.to_string()))?;
    (&stream)
        .write_all(line.as_bytes())
        .and_then(|()| (&stream).write_all(b"\n"))
        .map_err(unreached)?;
    let mut answer = String::new();
    BufReader::new(&stream)
        .read_line(&mut answer)
        .map_err(unreached)?;
    // An answer that does not read, or is not about putting back, says
    // nothing a person could act on, so the sentence stands without one.
    match ToAPerson::read(answer.trim_end()) {
        Ok(ToAPerson::PutBack { said }) => Ok(said
            .iter()
            .map(|wording| wording.text().to_owned())
            .collect()),
        Ok(ToAPerson::Refused(wording)) => Err(NotPutBack::Refused(wording.text().to_owned())),
        Ok(_) | Err(_) => Err(NotPutBack::NotReached(String::new())),
    }
}
