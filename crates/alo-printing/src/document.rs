//! What may be printed, decided from the file's bytes before anything is sent.
//!
//! Task 1 of the documents-and-paper plan already answers *what is this file*
//! from its content, and printing asks it first. A printer takes a PDF, a
//! picture or plain text, and the printing service turns each of those into
//! what the particular printer speaks; anything else is refused here, with the
//! sentence `alo-opening` says about the file beside this crate's, before a
//! single byte reaches a printer.
//!
//! **A file whose name lies is not printed**, whatever it really is. It is the
//! finding `alo-opening` puts first, and printing a file named `invoice.pdf`
//! that is really a picture would be paper nobody asked for. So is a program,
//! a damaged file, and an office document — those wait on converting (task 2),
//! and until then are not a kind printers take as they are.

use std::ffi::OsStr;
use std::io::{Read, Seek, SeekFrom};

use alo_opening::{Cannot, Decided, Kind, Outcome, ThisMachine, decide};
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// The kinds of file a printer takes as they are, and how each is named to the
/// printing service.
const PRINTS: [(Kind, &str); 4] = [
    (Kind::Pdf, "application/pdf"),
    (Kind::PngImage, "image/png"),
    (Kind::JpegImage, "image/jpeg"),
    (Kind::Text, "text/plain"),
];

/// A file that can be printed, as it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Printable {
    /// What it is.
    kind: Kind,
    /// How it is named to the printing service.
    format: &'static str,
    /// How many bytes it is.
    length: u64,
}

impl Printable {
    /// What it is.
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// How it is named to the printing service.
    pub(crate) fn format(&self) -> &'static str {
        self.format
    }

    /// How many bytes it is.
    pub(crate) fn length(&self) -> u64 {
        self.length
    }
}

/// Why a file is not printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotPrintable {
    /// It is a kind of file this machine does not print as it is.
    NotAKindPrintersTake(Kind),
    /// It cannot be opened at all.
    CannotBeOpened(Cannot),
    /// Its name says it is something it is not.
    NotWhatItsNameSays(Decided),
    /// It could not be read.
    Unreadable,
}

impl NotPrintable {
    /// What a person reads, in order: what `alo-opening` says about the file
    /// where it has something to say, then that nothing was printed.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::NotAKindPrintersTake(kind) => vec![strings.say(
                &words::NOT_A_KIND_PRINTERS_TAKE.key(),
                &Filling::nothing().and_said(words::WHAT, &kind.said(strings)),
            )],
            Self::CannotBeOpened(cannot) => {
                let mut said = Outcome::CannotOpen(cannot).said(strings);
                said.push(strings.say(&words::CANNOT_BE_OPENED.key(), &Filling::nothing()));
                said
            }
            Self::NotWhatItsNameSays(decided) => {
                let mut said = decided.said(strings);
                said.truncate(1);
                said.push(strings.say(&words::NOT_WHAT_ITS_NAME_SAYS.key(), &Filling::nothing()));
                said
            }
            Self::Unreadable => vec![strings.say(&words::UNREADABLE.key(), &Filling::nothing())],
        }
    }
}

/// Whether this file can be printed as it is, decided from its bytes; the file
/// is left at its start, ready to be sent.
///
/// # Errors
/// [`NotPrintable`].
pub fn printable<F: Read + Seek>(file: &mut F, name: &OsStr) -> Result<Printable, NotPrintable> {
    let mut machine = ThisMachine::with_nothing();
    for (kind, _) in PRINTS {
        machine = machine
            .opens(kind)
            .map_err(|_| NotPrintable::NotAKindPrintersTake(kind))?;
    }
    let decided = decide(file, name, &machine).map_err(|_| NotPrintable::Unreadable)?;
    let kind = match decided {
        Decided::NotWhatItsNameSays { .. } => {
            return Err(NotPrintable::NotWhatItsNameSays(decided));
        }
        Decided::AsItIs(Outcome::OpensAsItIs { kind, .. }) => kind,
        Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(kind)))
        | Decided::AsItIs(Outcome::Converts { from: kind, .. }) => {
            return Err(NotPrintable::NotAKindPrintersTake(kind));
        }
        Decided::AsItIs(Outcome::CannotOpen(cannot)) => {
            return Err(NotPrintable::CannotBeOpened(cannot));
        }
    };
    let format = PRINTS
        .iter()
        .find(|(prints, _)| *prints == kind)
        .map(|(_, format)| *format)
        .ok_or(NotPrintable::NotAKindPrintersTake(kind))?;
    let length = file
        .seek(SeekFrom::End(0))
        .and_then(|length| file.seek(SeekFrom::Start(0)).map(|_| length))
        .map_err(|_| NotPrintable::Unreadable)?;
    Ok(Printable {
        kind,
        format,
        length,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A PDF, a picture and text print, and the file is left at its start.
    #[test]
    fn a_pdf_a_picture_and_text_print_as_they_are() {
        let mut pdf = Cursor::new(b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n".to_vec());
        let printable = printable(&mut pdf, OsStr::new("letter.pdf")).unwrap();
        assert_eq!(printable.kind(), Kind::Pdf);
        assert_eq!(printable.format(), "application/pdf");
        assert_eq!(printable.length(), 36);
        assert_eq!(pdf.position(), 0);

        let mut text = Cursor::new(b"Dear Anna,\nthe invoice is attached.\n".to_vec());
        assert_eq!(
            super::printable(&mut text, OsStr::new("note.txt"))
                .unwrap()
                .format(),
            "text/plain"
        );
    }

    /// **A program named as a PDF never reaches a printer**, and nor does a
    /// file that is not what its name says, or nothing at all.
    #[test]
    fn a_program_a_lying_name_and_an_empty_file_are_not_printed() {
        let mut program = Cursor::new(b"\x7fELF\x02\x01\x01\0".to_vec());
        assert!(matches!(
            printable(&mut program, OsStr::new("invoice.pdf")),
            Err(NotPrintable::NotWhatItsNameSays(_))
        ));
        let mut program = Cursor::new(b"\x7fELF\x02\x01\x01\0".to_vec());
        assert_eq!(
            printable(&mut program, OsStr::new("invoice")),
            Err(NotPrintable::CannotBeOpened(Cannot::AProgram))
        );
        let mut empty = Cursor::new(Vec::new());
        assert_eq!(
            printable(&mut empty, OsStr::new("empty")),
            Err(NotPrintable::CannotBeOpened(Cannot::Empty))
        );
    }
}
