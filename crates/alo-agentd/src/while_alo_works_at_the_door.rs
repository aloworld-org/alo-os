//! **alo at work in one application, on the compositor's door**: the one act
//! on that door that holds its connection, for as long as alo works
//! (`alo_acting::working_in`), because the line the desktop puts up — the one
//! alo's border on the window reads — is up for exactly that long.
//!
//! It is still one connection per act, as the rest of the door: the act is
//! *alo working there*, from the first word to `done`, and the connection is
//! dropped with it. A turn that crashes drops it too, and the desktop takes the
//! line down when it sees the connection gone.

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;

use alo_acting::{ToWorkIn, Worked, Working, counting::DONE};
use alo_turn::{NotAtTheDoor, WhileAloWorks};

/// One work begun, its connection held.
pub(crate) struct AtWork {
    /// The connection, held until `done`.
    talking: UnixStream,
    /// What the door answered first.
    answered: Working,
}

impl AtWork {
    /// Say `asked` on a connection already made, and read the first answer.
    pub(crate) fn begun(mut talking: UnixStream, asked: &ToWorkIn) -> Result<Self, NotAtTheDoor> {
        say(&mut talking, &asked.written())?;
        let heard = hear(&talking)?;
        let answered = Working::read(&heard).ok_or_else(|| NotAtTheDoor::NotAnswered {
            why: format!("the compositor answered {heard:?}, which is no answer to work"),
        })?;
        Ok(Self { talking, answered })
    }
}

impl WhileAloWorks for AtWork {
    fn answered(&self) -> Working {
        self.answered
    }

    fn done(mut self: Box<Self>) -> Result<Worked, NotAtTheDoor> {
        say(&mut self.talking, DONE)?;
        let heard = hear(&self.talking)?;
        Worked::read(&heard).ok_or_else(|| NotAtTheDoor::NotAnswered {
            why: format!("the compositor answered {heard:?}, which is no answer to done"),
        })
    }
}

/// Write one line.
fn say(talking: &mut UnixStream, line: &str) -> Result<(), NotAtTheDoor> {
    talking
        .write_all(format!("{line}\n").as_bytes())
        .and_then(|()| talking.flush())
        .map_err(|why| NotAtTheDoor::NotAnswered {
            why: why.to_string(),
        })
}

/// Read one line, refusing silence.
fn hear(talking: &UnixStream) -> Result<String, NotAtTheDoor> {
    let mut heard = String::new();
    let read = BufReader::new(talking)
        .read_line(&mut heard)
        .map_err(|why| NotAtTheDoor::NotAnswered {
            why: why.to_string(),
        })?;
    if read == 0 {
        return Err(NotAtTheDoor::NotAnswered {
            why: "the compositor closed the connection without answering".to_owned(),
        });
    }
    Ok(heard)
}
