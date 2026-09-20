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

/// The longest a conversion may take before it is stopped.
pub const LONGEST: Duration = Duration::from_secs(120);

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

/// What the engine is asked to export each conversion with.
///
/// **A filter names the export, not the document.** Which reader opens the
/// document is decided by the document, from the ending its scratch name
/// carries; this says which of the engine's writers produces the PDF. So two
/// conversions of text documents share one filter, and that is not a collision:
/// a Pages document and a Word document are both pages of text, and both come
/// out of the same writer.
const fn filter(conversion: Conversion) -> &'static str {
    match conversion {
        Conversion::WordDocument | Conversion::OpenDocumentText | Conversion::PagesDocument => {
            "pdf:writer_pdf_Export"
        }
        Conversion::ExcelWorkbook | Conversion::OpenDocumentSpreadsheet => "pdf:calc_pdf_Export",
        Conversion::PowerPointPresentation | Conversion::OpenDocumentPresentation => {
            "pdf:impress_pdf_Export"
        }
    }
}

/// Convert the document at `scratch`/[`Conversion::scratch_name`] into a PDF
/// beside it, and say where the PDF is.
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
        .arg(filter(conversion))
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

    let copy = into.join(
        Path::new(conversion.scratch_name())
            .with_extension("pdf")
            .file_name()
            .unwrap_or_default(),
    );
    if copy.is_file() {
        Ok(copy)
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
        let filters: Vec<&str> = Conversion::EVERY.into_iter().map(filter).collect();
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

    /// **A held-back conversion already has its filter, and it is one this
    /// engine is asked for anyway.**
    ///
    /// The argument list is the whole of what `convert` varies between two
    /// conversions, so deciding it now is what makes offering this one later a
    /// one-line change rather than a design. Asserting it is a filter already
    /// in use is what stops a plausible-looking name nobody has run being
    /// written here: `pdf:pages_pdf_Export` would pass a test that only
    /// checked the prefix, and there is no such filter.
    #[test]
    fn a_held_back_conversion_exports_through_a_filter_already_in_use() {
        let in_use: Vec<&str> = Conversion::EVERY.into_iter().map(filter).collect();
        for held in Conversion::HELD_BACK {
            let exports_with = filter(held);
            assert!(exports_with.starts_with("pdf:"), "{held:?}");
            assert!(
                in_use.contains(&exports_with),
                "{held:?} exports with {exports_with}, which no conversion this machine \
                 makes has ever asked the engine for"
            );
        }
        assert_eq!(
            filter(Conversion::PagesDocument),
            filter(Conversion::WordDocument),
            "both are text documents and come out of the same writer"
        );
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
                Conversion::WordDocument
            ),
            Err(NotConverted::NotStarted)
        );
        std::fs::remove_dir_all(&scratch).unwrap();
    }
}
