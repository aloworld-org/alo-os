//! The converting service's side: one request at a time, from descriptors to
//! a report.
//!
//! ADR 0039 §A, in order, and the first *no* is the answer:
//!
//! 1. **Understand the request** — a line of [`crate::wire`] with exactly the
//!    descriptors it carries.
//! 2. **Read the original** from its descriptor, up to [`LARGEST_ORIGINAL`].
//! 3. **Check it is what the request named**, from its bytes, with
//!    `alo-opening` — the verb decided that already, and the service does not
//!    take a client's word for it.
//! 4. **Inventory the original**, before the copy is made. One that cannot be
//!    inventoried is not converted. An original whose format nothing here reads
//!    is rendered by the engine first and the rendering is inventoried —
//!    `inventory::read_from` says which originals those are, and why.
//! 5. **Convert** in a scratch folder of the service's own, with the engine's
//!    one argument list.
//! 6. **Inventory the copy**, before a byte of it is written anywhere the person
//!    can see. One that cannot be inventoried is not written.
//! 7. **Write the copy** to its descriptor, and say what it carried.
//!
//! The scratch folder is emptied whatever happened, when the conversion ends.

use std::fs::{self, File};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::os::fd::OwnedFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use alo_opening::{Decided, Kind, Outcome, ThisMachine, decide};

use crate::conversion::Conversion;
use crate::engine::{self, Export};
use crate::inventory::{Copy, Original, ReadFrom, carried, read_from};
use crate::passing::receive;
use crate::wire::{Answer, LONGEST_REQUEST, Refusal, Request};

/// The largest original the service converts, in bytes.
pub const LARGEST_ORIGINAL: u64 = 128 * 1024 * 1024;

/// The largest copy the service writes, in bytes.
pub const LARGEST_COPY: u64 = 512 * 1024 * 1024;

/// How long a client has to send its request.
const SENDING: Duration = Duration::from_secs(10);

/// What a service runs with.
#[derive(Debug, Clone)]
pub struct Serving {
    /// The engine it starts.
    engine: PathBuf,
    /// Where its scratch folders are made.
    scratch: PathBuf,
}

impl Serving {
    /// A service that starts this engine and works in this folder.
    #[must_use]
    pub fn with(engine: &Path, scratch: &Path) -> Self {
        Self {
            engine: engine.to_owned(),
            scratch: scratch.to_owned(),
        }
    }

    /// Answer every connection, one at a time, for as long as the listener
    /// lasts.
    ///
    /// # Errors
    /// Whatever the machine said about the listener itself. A connection that
    /// goes wrong ends that connection and nothing else.
    pub fn serve(&self, listener: &UnixListener) -> io::Result<()> {
        loop {
            let (stream, _) = listener.accept()?;
            let _answered = self.answer(&stream);
        }
    }

    /// Answer one connection.
    fn answer(&self, stream: &UnixStream) -> io::Result<()> {
        stream.set_read_timeout(Some(SENDING))?;
        let answer = match receive(stream, LONGEST_REQUEST) {
            Ok((line, descriptors)) => {
                let request = std::str::from_utf8(&line).ok().and_then(Request::read);
                self.carry_out(request, descriptors)
            }
            Err(_) => Answer::Refused(Refusal::NotUnderstood),
        };
        let mut stream = stream;
        stream.write_all(answer.written().as_bytes())
    }

    /// Carry out one request.
    fn carry_out(&self, request: Option<Request>, descriptors: Vec<OwnedFd>) -> Answer {
        match (request, <[OwnedFd; 2]>::try_from(descriptors)) {
            (Some(Request::Ready), Err(none)) if none.is_empty() => Answer::Ready,
            (Some(Request::Convert(conversion)), Ok([original, copy])) => {
                match self.convert(conversion, File::from(original), File::from(copy)) {
                    Ok(carried) => Answer::Converted(carried),
                    Err(refusal) => Answer::Refused(refusal),
                }
            }
            _ => Answer::Refused(Refusal::NotUnderstood),
        }
    }

    /// Convert an original into a copy.
    fn convert(
        &self,
        conversion: Conversion,
        mut original: File,
        mut copy: File,
    ) -> Result<crate::carried::Carried, Refusal> {
        let bytes = read_capped(&mut original, LARGEST_ORIGINAL)?;
        let macros = what_it_is(&bytes, conversion)?;

        let scratch = Scratch::made_in(&self.scratch).map_err(|_| Refusal::CouldNotConvert)?;
        fs::write(scratch.path().join(conversion.scratch_name()), &bytes)
            .map_err(|_| Refusal::CouldNotConvert)?;

        // Step 4, and for one conversion in seven it runs the engine: an
        // original this crate does not read is inventoried out of a rendering
        // the engine makes of it. The rendering never becomes the copy.
        let before = match read_from(conversion) {
            ReadFrom::ItsOwnBytes => Original::of(&bytes, conversion, macros),
            ReadFrom::WhatTheEngineReadsOfIt(as_if) => {
                let rendering =
                    self.exported(&scratch, conversion, Export::SomethingThisMachineReads)?;
                Original::of(&rendering, as_if, macros)
            }
        }
        .map_err(|_| Refusal::OriginalNotChecked)?;

        let pdf = self.exported(&scratch, conversion, Export::TheCopy)?;
        let after = Copy::of(&pdf).map_err(|_| Refusal::CopyNotChecked)?;
        let carried = carried(&before, &after);

        copy.set_len(0)
            .and_then(|()| copy.seek(SeekFrom::Start(0)))
            .and_then(|_| copy.write_all(&pdf))
            .and_then(|()| copy.sync_all())
            .map_err(|_| Refusal::CopyNotWritten)?;
        Ok(carried)
    }

    /// Start the engine over the document already in `scratch` and read back
    /// what it exported, refusing more than [`LARGEST_COPY`] bytes of it.
    ///
    /// Both of a conversion's runs come through here, so a rendering and a copy
    /// are held to the same limits and the same refusals.
    fn exported(
        &self,
        scratch: &Scratch,
        conversion: Conversion,
        export: Export,
    ) -> Result<Vec<u8>, Refusal> {
        let came_out =
            engine::convert(&self.engine, scratch.path(), conversion, export).map_err(|not| {
                match not {
                    engine::NotConverted::TooSlow => Refusal::TooSlow,
                    engine::NotConverted::NotStarted | engine::NotConverted::NoCopy => {
                        Refusal::CouldNotConvert
                    }
                }
            })?;
        read_capped(
            &mut File::open(came_out).map_err(|_| Refusal::CouldNotConvert)?,
            LARGEST_COPY,
        )
    }
}

/// Whether the bytes are the kind a conversion is from, and the macros they
/// carry.
fn what_it_is(bytes: &[u8], conversion: Conversion) -> Result<alo_opening::Macros, Refusal> {
    let machine = ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .and_then(|machine| machine.converts(conversion.from(), Kind::Pdf))
        .map_err(|_| Refusal::NotTheKindAsked)?;
    // No name is handed over, so no name can decide.
    match decide(&mut Cursor::new(bytes), std::ffi::OsStr::new(""), &machine) {
        Ok(Decided::AsItIs(Outcome::Converts { from, costs, .. })) if from == conversion.from() => {
            Ok(costs.macros())
        }
        _ => Err(Refusal::NotTheKindAsked),
    }
}

/// Everything from the start of a file, refusing more than `most` bytes.
fn read_capped(file: &mut File, most: u64) -> Result<Vec<u8>, Refusal> {
    file.seek(SeekFrom::Start(0))
        .map_err(|_| Refusal::NotUnderstood)?;
    let mut bytes = Vec::new();
    file.take(most.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| Refusal::NotUnderstood)?;
    if u64::try_from(bytes.len()).map_or(true, |read| read > most) {
        return Err(Refusal::TooLarge);
    }
    Ok(bytes)
}

/// A scratch folder of one conversion's own, emptied when it is dropped.
struct Scratch {
    /// Where it is.
    path: PathBuf,
}

/// How many scratch folders this service has made, so no two are one.
static MADE: AtomicU64 = AtomicU64::new(0);

impl Scratch {
    /// Make one, private to this login.
    fn made_in(folder: &Path) -> io::Result<Self> {
        use std::os::unix::fs::DirBuilderExt as _;

        let number = MADE.fetch_add(1, Ordering::Relaxed);
        let path = folder.join(format!("conversion-{}-{number}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&path)?;
        Ok(Self { path })
    }

    /// Where it is.
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _emptied = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::the_document;

    /// **The service checks the bytes itself**: a workbook asked for as a
    /// Word document is refused, and a Word document is what it is.
    #[test]
    fn the_service_takes_no_clients_word_for_what_a_document_is() {
        let workbook = the_document("sample.xlsx");
        assert_eq!(
            what_it_is(&workbook, Conversion::WordDocument),
            Err(Refusal::NotTheKindAsked)
        );
        assert_eq!(
            what_it_is(&workbook, Conversion::ExcelWorkbook),
            Ok(alo_opening::Macros::NoneSeen)
        );
        assert_eq!(
            what_it_is(b"\x7fELF\x02\x01\x01\0", Conversion::WordDocument),
            Err(Refusal::NotTheKindAsked)
        );
    }

    /// **A request without exactly its descriptors is not understood**, and
    /// nothing is converted.
    #[test]
    fn a_request_without_its_descriptors_is_not_understood() {
        let serving = Serving::with(Path::new("/nowhere/engine"), &std::env::temp_dir());
        assert_eq!(
            serving.carry_out(Some(Request::Ready), Vec::new()),
            Answer::Ready
        );
        assert_eq!(
            serving.carry_out(Some(Request::Convert(Conversion::WordDocument)), Vec::new()),
            Answer::Refused(Refusal::NotUnderstood)
        );
        assert_eq!(
            serving.carry_out(None, Vec::new()),
            Answer::Refused(Refusal::NotUnderstood)
        );
    }

    /// A scratch folder is emptied when its conversion ends.
    #[test]
    fn a_scratch_folder_is_emptied_when_it_ends() {
        let scratch = Scratch::made_in(&std::env::temp_dir()).unwrap();
        let at = scratch.path().to_owned();
        fs::write(at.join("document.docx"), b"a stranger's").unwrap();
        drop(scratch);
        assert!(!at.exists());
    }
}
