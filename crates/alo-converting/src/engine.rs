//! The rented engine, and the one argument list it is ever started with.
//!
//! **The only file in alo OS that names it.** ADR 0039 chose LibreOffice, in
//! its headless mode, pinned in the image and unmodified (ADR 0011): the only
//! office engine that converts all three formats on Linux under a licence the
//! image can carry. A person never learns that name (`docs/features.md`), and a
//! test reads this crate's source for it anywhere but here.
//!
//! # Started by the service, never by a turn
//!
//! This is reached from `serving.rs` and from nothing else. The service is a
//! system unit of its own with no network and no view of any home folder; what
//! it starts runs with that and nothing more. `alo-agentd` never starts a
//! program, and `alo-bounding` says why.
//!
//! # A fixed argument list from a closed set
//!
//! What varies between two conversions is which of the three it is — which
//! decides the export filter — and the scratch folder, which the service made.
//! Nothing a request carries is an argument: the document's own name is never
//! passed on, because a name is whatever the sender wrote.
//!
//! # Twice, for one conversion out of seven
//!
//! An original whose format nothing here reads is inventoried out of a
//! rendering the engine makes of it (`crate::inventory::read_from`), so that
//! conversion starts the engine twice: once for the rendering, once for the
//! copy. Both are the same fixed argument list with a different export, and
//! [`LONGEST_ALTOGETHER`] is what the service waits for either way.
//!
//! The copy is always made from the **original**, never from the rendering, so
//! nobody's PDF is a conversion of a conversion.
//!
//! # And a clean environment
//!
//! The engine is started with **no inherited environment**: its home is the
//! scratch folder, so its profile is written there and emptied with it, and
//! nothing of the service's own environment reaches a program that reads
//! documents from strangers.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::conversion::Conversion;

/// Where the pinned engine is on a machine: the path its release installs to,
/// on the image and on the machine this repository is gated on.
pub const THE_ENGINE: &str = "/opt/libreoffice26.2/program/soffice";

/// The longest one run of the engine may take before it is stopped.
pub const LONGEST: Duration = Duration::from_secs(120);

/// The most times the engine is started for one conversion: once for a
/// rendering the inventory reads, and once for the copy.
pub const MOST_RUNS: u64 = 2;

/// The longest all of one conversion's runs of the engine may take.
pub const LONGEST_ALTOGETHER: Duration = Duration::from_secs(LONGEST.as_secs() * MOST_RUNS);

/// How often a running conversion is looked at.
const LOOKING: Duration = Duration::from_millis(50);

/// Why the engine did not produce a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotConverted {
    /// It is not on this machine, or could not be started.
    #[error("the engine could not be started")]
    NotStarted,
    /// It ran past [`LONGEST`] and was stopped.
    #[error("the engine took too long and was stopped")]
    TooSlow,
    /// It finished without writing a copy.
    #[error("the engine wrote no copy")]
    NoCopy,
}

/// What the engine is asked to produce out of a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Export {
    /// The copy a person keeps: a PDF (ADR 0039 §1), through the writer the
    /// document's own kind belongs to.
    TheCopy,
    /// A rendering this crate can inventory, of an original whose own format it
    /// does not read: the engine's own text document.
    ///
    /// One value and not one per kind, because one conversion needs it and it
    /// is a text document. A rendering of another shape is another value here
    /// and another line in `crate::inventory::read_from`, and that crate's
    /// `every_rendering_the_engine_is_asked_for_is_a_text_document` is what
    /// fails until both exist.
    SomethingThisMachineReads,
}

/// What the engine is asked to export each conversion with.
///
/// **A filter names the export, not the document.** Which reader opens the
/// document is decided by the document, from the ending its scratch name
/// carries; this says which of the engine's writers produces what comes out. So
/// two conversions of text documents share one filter, and that is not a
/// collision: a Pages document and a Word document are both pages of text, and
/// both come out of the same writer.
/// What the engine is asked to export each conversion with.
const fn filter(conversion: Conversion, export: Export) -> &'static str {
    match export {
        Export::SomethingThisMachineReads => the_rendering(conversion),
        Export::TheCopy => match the_shape(conversion) {
            Shape::Prose => "pdf:writer_pdf_Export",
            Shape::Spreadsheet => "pdf:calc_pdf_Export",
            Shape::Slides => "pdf:impress_pdf_Export",
        },
    }
}

/// The engine's own format for a document of this shape, and the filter that
/// writes one — prose as prose, a spreadsheet as a spreadsheet, slides as
/// slides.
///
/// **Shaped like the document rather than always text, and that was measured.**
/// Rendering `sample.xls` through the writer produced an OpenDocument carrying
/// the substituted family and the comment and **no formula at all**: a
/// spreadsheet read by the prose reader has had its cells fixed to their
/// values before anything could count them. So a copy of somebody's workbook
/// would have reported a lost font and a lost comment and said nothing about
/// `=NOW()` becoming a number — the exact thing ADR 0039 §5 exists to prevent.
/// The same file through the spreadsheet's own writer keeps all three.
/// Measured on the pinned engine, 2026-09-25.
const fn the_rendering(conversion: Conversion) -> &'static str {
    match the_shape(conversion) {
        Shape::Prose => "odt:writer8",
        Shape::Spreadsheet => "ods:calc8",
        Shape::Slides => "odp:impress8",
    }
}

/// What an export leaves behind it, as a file ending.
const fn ending(conversion: Conversion, export: Export) -> &'static str {
    match export {
        Export::TheCopy => "pdf",
        Export::SomethingThisMachineReads => match the_shape(conversion) {
            Shape::Prose => "odt",
            Shape::Spreadsheet => "ods",
            Shape::Slides => "odp",
        },
    }
}

/// Which of the engine's three readers and writers a document belongs to.
///
/// The one place this is decided. Both the filter a copy is exported with and
/// the format a rendering is made in follow from it, so the two cannot come to
/// disagree about what kind of document is being handled.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// Pages of text.
    Prose,
    /// Rows and columns.
    Spreadsheet,
    /// Slides.
    Slides,
}

/// The shape of the document a conversion is of.
const fn the_shape(conversion: Conversion) -> Shape {
    match conversion {
        Conversion::WordDocument
        | Conversion::OpenDocumentText
        | Conversion::PagesDocument
        | Conversion::OlderWordDocument => Shape::Prose,
        Conversion::ExcelWorkbook
        | Conversion::OpenDocumentSpreadsheet
        | Conversion::OlderExcelWorkbook => Shape::Spreadsheet,
        Conversion::PowerPointPresentation
        | Conversion::OpenDocumentPresentation
        | Conversion::OlderPowerPointPresentation => Shape::Slides,
    }
}

/// Export the document at `scratch`/[`Conversion::scratch_name`] into `export`
/// beside it, and say where what came out is.
///
/// `engine` is [`THE_ENGINE`] on a machine; it is an argument so that the
/// refusal paths can be tested with one that is not there.
///
/// # Errors
/// [`NotConverted`]; a stopped engine has been killed and waited for.
pub fn convert(
    engine: &Path,
    scratch: &Path,
    conversion: Conversion,
    export: Export,
) -> Result<PathBuf, NotConverted> {
    let profile = scratch.join("profile");
    let into = scratch.join("out");
    let document = scratch.join(conversion.scratch_name());
    let mut started = Command::new(engine)
        .arg("--headless")
        .arg("--norestore")
        .arg("--nologo")
        .arg("--nodefault")
        .arg("--nolockcheck")
        .arg(format!(
            "-env:UserInstallation=file://{}",
            profile.display()
        ))
        .arg("--convert-to")
        .arg(filter(conversion, export))
        .arg("--outdir")
        .arg(&into)
        .arg(&document)
        .env_clear()
        .env("HOME", scratch)
        .env("PATH", "/usr/bin:/bin")
        .current_dir(scratch)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| NotConverted::NotStarted)?;

    let began = Instant::now();
    loop {
        match started.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if began.elapsed() < LONGEST => thread::sleep(LOOKING),
            Ok(None) | Err(_) => {
                // Stopped rather than left: a conversion nobody waits for is a
                // program reading a stranger's document with nobody watching.
                let _killed = started.kill();
                let _reaped = started.wait();
                return Err(NotConverted::TooSlow);
            }
        }
    }

    let came_out = into.join(
        Path::new(conversion.scratch_name())
            .with_extension(ending(conversion, export))
            .file_name()
            .unwrap_or_default(),
    );
    if came_out.is_file() {
        Ok(came_out)
    } else {
        Err(NotConverted::NoCopy)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every conversion exports into a PDF, through one of the engine's
    /// three writers.**
    ///
    /// The filters are not one per conversion and must not be: a filter names
    /// which writer produces the PDF, and the engine has exactly three —
    /// prose, a sheet, and slides. A `.docx` and an `.odt` are both prose and
    /// leave by the same one. What would be wrong is a fourth filter appearing,
    /// which would mean somebody had written down a writer this engine does not
    /// have.
    #[test]
    fn every_conversion_exports_into_a_pdf_through_one_of_three_writers() {
        let filters: Vec<&str> = Conversion::EVERY
            .into_iter()
            .map(|conversion| filter(conversion, Export::TheCopy))
            .collect();
        assert!(filters.iter().all(|filter| filter.starts_with("pdf:")));
        let mut writers = filters.clone();
        writers.sort_unstable();
        writers.dedup();
        assert_eq!(
            writers,
            [
                "pdf:calc_pdf_Export",
                "pdf:impress_pdf_Export",
                "pdf:writer_pdf_Export"
            ]
        );
    }

    /// **A Pages document comes out of the same writer a Word document does**,
    /// because both are pages of text — and there is no `pdf:pages_pdf_Export`,
    /// which is what a plausible-looking name nobody had run would have been.
    #[test]
    fn a_pages_document_exports_through_a_filter_already_in_use() {
        assert_eq!(
            filter(Conversion::PagesDocument, Export::TheCopy),
            filter(Conversion::WordDocument, Export::TheCopy),
            "both are text documents and come out of the same writer"
        );
    }

    /// **A rendering is one of the engine's own documents, shaped like the one
    /// it is of, and never a PDF.**
    ///
    /// A PDF is the copy: inventorying one as though it were the original
    /// would compare a document with itself and report that every conversion
    /// carried everything.
    ///
    /// Shaped like its document because a spreadsheet read by the prose reader
    /// arrives with its cells already fixed to their values — measured on the
    /// pinned engine, and `the_rendering` carries the measurement.
    #[test]
    fn a_rendering_is_one_of_the_engines_own_documents_shaped_like_it_and_never_a_pdf() {
        for conversion in Conversion::EVERY {
            let rendering = filter(conversion, Export::SomethingThisMachineReads);
            assert!(!rendering.starts_with("pdf:"), "{conversion:?}");
            assert_ne!(rendering, filter(conversion, Export::TheCopy));
            assert_eq!(
                rendering,
                the_rendering(conversion),
                "{conversion:?} is rendered by a writer that is not its shape's"
            );
            // Every rendering ends the way the OpenDocument kind of its own
            // shape ends, so the file the inventory opens is read by the
            // reader that wrote it.
            let ends = ending(conversion, Export::SomethingThisMachineReads);
            assert!(
                rendering.starts_with(ends),
                "{conversion:?} writes a {rendering} into a .{ends}"
            );
        }
        assert_eq!(ending(Conversion::WordDocument, Export::TheCopy), "pdf");
        assert_eq!(
            ending(Conversion::PagesDocument, Export::SomethingThisMachineReads),
            Path::new(Conversion::OpenDocumentText.scratch_name())
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap(),
            "a Pages document is read as an OpenDocument text document and must end like one"
        );
    }

    /// **All of one conversion's runs of the engine fit inside
    /// [`LONGEST_ALTOGETHER`]**, which is what the service waits for.
    ///
    /// A conversion that starts the engine twice inside a limit written for one
    /// run would be stopped halfway by the client rather than by the engine's
    /// own limit, and a person would be told the service did not answer about a
    /// conversion that was still going.
    #[test]
    fn what_the_service_waits_for_covers_every_run_of_the_engine() {
        assert_eq!(MOST_RUNS, 2);
        assert_eq!(LONGEST_ALTOGETHER, LONGEST * 2);
        assert!(LONGEST_ALTOGETHER >= LONGEST);
    }

    /// **An engine that is not there is a refusal**, not a copy.
    #[test]
    fn an_engine_that_is_not_there_converts_nothing() {
        let scratch =
            std::env::temp_dir().join(format!("alo-converting-no-engine-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        assert_eq!(
            convert(
                &scratch.join("no-such-engine"),
                &scratch,
                Conversion::WordDocument,
                Export::TheCopy
            ),
            Err(NotConverted::NotStarted)
        );
        assert_eq!(
            convert(
                &scratch.join("no-such-engine"),
                &scratch,
                Conversion::PagesDocument,
                Export::SomethingThisMachineReads
            ),
            Err(NotConverted::NotStarted),
            "a rendering nobody could make is a refusal too"
        );
        std::fs::remove_dir_all(&scratch).unwrap();
    }
}
