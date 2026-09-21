//! The converting service, as the verb reaches it: on this machine, over its
//! own socket, and nowhere else.
//!
//! [`ConvertingService`] is a path to a Unix socket. There is no address form,
//! so there is no way to point converting at another machine: ADR 0039 rules
//! out converting anywhere but here, with no fallback and no setting.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::carried::Carried;
use crate::conversion::Conversion;
use crate::engine::LONGEST_ALTOGETHER;
use crate::wire::Refusal;

/// Where the service listens on a machine, as its socket unit says.
pub const THE_SOCKET: &str = "/run/alo-convertd/socket";

/// How long the service may take to say it is there.
const ASKING_IF_READY: Duration = Duration::from_secs(5);

/// How long a conversion may take to be answered: every run of the engine it
/// makes, at the engine's own limit each, and time around them for the
/// inventories and the copying.
const ASKING_TO_CONVERT: Duration = LONGEST_ALTOGETHER.saturating_add(Duration::from_secs(60));

/// This machine's converting service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertingService {
    /// Its socket.
    socket: PathBuf,
}

/// Why the service wrote nothing to the copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unconverted {
    /// Nothing answered, or what answered did not speak the protocol.
    NotAnswering,
    /// It answered, and refused.
    Refused(Refusal),
}

impl ConvertingService {
    /// The service where a machine runs it.
    #[must_use]
    pub fn this_machines() -> Self {
        Self::at(Path::new(THE_SOCKET))
    }

    /// The service at a socket — for a test, which puts one somewhere of its
    /// own.
    #[must_use]
    pub fn at(socket: &Path) -> Self {
        Self {
            socket: socket.to_owned(),
        }
    }

    /// Whether the service is there and speaks this protocol.
    #[must_use]
    pub fn answers(&self) -> bool {
        platform::ready(&self.socket, ASKING_IF_READY)
    }

    /// Convert the open original into the open, empty copy.
    ///
    /// # Errors
    /// [`Unconverted`]; the copy may have been created and must be removed by
    /// whoever created it.
    pub fn convert(
        &self,
        conversion: Conversion,
        original: &File,
        copy: &File,
    ) -> Result<Carried, Unconverted> {
        platform::convert(&self.socket, ASKING_TO_CONVERT, conversion, original, copy)
    }
}

#[cfg(target_os = "linux")]
mod platform {
    //! Speaking to the service over its socket.

    use std::fs::File;
    use std::io::Read as _;
    use std::os::fd::AsFd as _;
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::time::Duration;

    use super::Unconverted;
    use crate::carried::Carried;
    use crate::conversion::Conversion;
    use crate::passing::send;
    use crate::wire::{Answer, LONGEST_ANSWER, Request};

    /// Ask one thing and read the answer.
    fn ask(socket: &Path, waiting: Duration, request: Request, files: &[&File]) -> Option<Answer> {
        let stream = UnixStream::connect(socket).ok()?;
        stream.set_read_timeout(Some(waiting)).ok()?;
        stream.set_write_timeout(Some(waiting)).ok()?;
        let descriptors: Vec<_> = files.iter().map(|file| file.as_fd()).collect();
        send(&stream, request.written().as_bytes(), &descriptors).ok()?;
        let mut sent = Vec::new();
        (&stream)
            .take(u64::try_from(LONGEST_ANSWER).ok()?)
            .read_to_end(&mut sent)
            .ok()?;
        Answer::read(std::str::from_utf8(&sent).ok()?)
    }

    /// Whether the service answers that it is ready.
    pub(super) fn ready(socket: &Path, waiting: Duration) -> bool {
        ask(socket, waiting, Request::Ready, &[]) == Some(Answer::Ready)
    }

    /// Ask the service to convert.
    pub(super) fn convert(
        socket: &Path,
        waiting: Duration,
        conversion: Conversion,
        original: &File,
        copy: &File,
    ) -> Result<Carried, Unconverted> {
        match ask(
            socket,
            waiting,
            Request::Convert(conversion),
            &[original, copy],
        ) {
            Some(Answer::Converted(carried)) => Ok(carried),
            Some(Answer::Refused(refusal)) => Err(Unconverted::Refused(refusal)),
            Some(Answer::Ready) | None => Err(Unconverted::NotAnswering),
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    //! A host with no converting service: nothing answers, which is true.

    use std::fs::File;
    use std::path::Path;
    use std::time::Duration;

    use super::Unconverted;
    use crate::carried::Carried;
    use crate::conversion::Conversion;

    /// Nothing is ready.
    pub(super) fn ready(_: &Path, _: Duration) -> bool {
        false
    }

    /// Nothing converts.
    pub(super) fn convert(
        _: &Path,
        _: Duration,
        _: Conversion,
        _: &File,
        _: &File,
    ) -> Result<Carried, Unconverted> {
        Err(Unconverted::NotAnswering)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A service that is not there does not answer**, and converting through
    /// it is not answering rather than an error nobody can read.
    #[test]
    fn a_service_that_is_not_there_does_not_answer() {
        let nowhere = std::env::temp_dir().join(format!(
            "alo-converting-no-service-{}/socket",
            std::process::id()
        ));
        let service = ConvertingService::at(&nowhere);
        assert!(!service.answers());
        assert_eq!(
            ConvertingService::this_machines().socket,
            Path::new(THE_SOCKET)
        );
    }
}
