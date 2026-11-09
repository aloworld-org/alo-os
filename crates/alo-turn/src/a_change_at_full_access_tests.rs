//! **At full access a change runs once, with the snapshot asked for first; on
//! every other list it waits** (ADR 0149) — the yes, and every refusal around
//! it, each written down.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

use alo_capability::{AnswerError, Grants};
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_record::Record;

use super::*;
use crate::machine::Machine;
use crate::testing::{
    NothingIsBounded, a_folder_of_our_own, as_given, files, granting, hour, in_english, noon,
    offering,
};

/// A folder with March's invoice in it.
fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    let folder = a_folder_of_our_own(what);
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

/// What renaming the invoice is asked for with.
fn renaming(invoice: &Path, to: &str) -> Vec<(&'static str, Given)> {
    vec![("file", as_given(invoice)), ("name", Given::text(to))]
}

/// A turn on a machine whose record is `record`, with nothing granted, handed
/// the turn, the folder and the invoice.
fn on_a_machine<T>(
    what: &str,
    record: &mut Record,
    doing: impl FnOnce(&mut Turning<'_, '_>, &Path, &Path) -> T,
) -> T {
    let strings = in_english();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut grants = Grants::default();
    let mut turning = Turning::beginning(
        offering(&invoice),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    doing(&mut turning, &folder, &invoice)
}

/// A `before` that remembers every sentence it was handed, whether the invoice
/// was still where it was at that moment, and answers with a kept place.
fn keeping<'k>(
    told: &'k RefCell<Vec<(String, bool)>>,
    invoice: &'k Path,
) -> impl FnMut(&Said) -> Option<String> + 'k {
    move |sentence| {
        told.borrow_mut()
            .push((sentence.text().to_owned(), invoice.is_file()));
        Some("1793502334-812-1".to_owned())
    }
}

/// **At full access the change runs at once and exactly once**, with no
/// approval and nothing left waiting; the snapshot is asked for first, while
/// the file is still where it was; and the record says it ran at full access,
/// from no approval, with the place kept before it.
#[test]
fn at_full_access_a_change_runs_once_with_the_snapshot_first() {
    let told = RefCell::new(Vec::new());
    let mut record = Record::default();
    let renamed = on_a_machine(
        "full-access-runs",
        &mut record,
        |turning, folder, invoice| {
            let at = Grants::default().at_full_access_for(&files());
            let changed = turning
                .changing(
                    "rename_file",
                    &renaming(invoice, "march-final.pdf"),
                    &at,
                    hour(),
                    &mut keeping(&told, invoice),
                    noon(),
                )
                .unwrap();
            assert_eq!(
                turning.waiting_at(noon()).count(),
                0,
                "a change was left waiting"
            );
            let Changed::Done { did, sentence } = changed else {
                return None;
            };
            assert!(sentence.text().contains("march-final.pdf"), "{sentence}");
            assert!(folder.join("march-final.pdf").is_file());
            did.to_files()
                .and_then(|did| did.now_at())
                .map(Path::to_path_buf)
        },
    );
    assert!(renamed.is_some_and(|at| at.ends_with("march-final.pdf")));
    let told = told.into_inner();
    assert_eq!(
        told.len(),
        1,
        "the snapshot was asked for {} times",
        told.len()
    );
    assert!(
        told.first().is_some_and(|(_, still_there)| *still_there),
        "the snapshot was asked for after the file had moved"
    );

    assert_eq!(
        record.len(),
        1,
        "{:?}",
        record.everything().collect::<Vec<_>>()
    );
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), None);
    assert!(entry.happened().by_full_access());
    assert!(entry.was_at_full_access());
    assert_eq!(
        entry.kept_before_it().map(alo_record::Line::as_str),
        Some("1793502334-812-1")
    );
}

/// **On a list without full access the change waits for one OK**, granted or
/// not granted: nothing runs, no snapshot is asked for, and the one OK runs it
/// once. This is *ask me each time*, the sealed box, nothing chosen and a file
/// that would not read — the service hands each of them over as the list it
/// keeps, untouched.
#[test]
fn without_full_access_a_change_waits_and_nothing_runs() {
    let told = RefCell::new(Vec::new());
    let mut record = Record::default();
    let waited = on_a_machine("asks-first", &mut record, |turning, folder, invoice| {
        let grants = granting(&[folder]);
        let changed = turning
            .changing(
                "rename_file",
                &renaming(invoice, "march-final.pdf"),
                &grants,
                hour(),
                &mut keeping(&told, invoice),
                noon(),
            )
            .unwrap();
        let id = match changed {
            Changed::Waiting(id) => id,
            Changed::Done { .. } => return false,
        };
        assert!(invoice.is_file(), "a change ran before its OK");
        assert_eq!(turning.waiting_at(noon()).count(), 1);
        turning.approving(id, &grants, noon()).unwrap();
        assert!(matches!(
            turning.approving(id, &grants, noon()).unwrap_err(),
            NotDone::NotAnswered(AnswerError::NothingWaiting { .. })
        ));

        // And with nothing granted, refused before anybody is asked.
        let april = folder.join("april.pdf");
        fs::write(&april, "April").unwrap();
        let refused = turning
            .changing(
                "rename_file",
                &renaming(&april, "april-final.pdf"),
                &Grants::default(),
                hour(),
                &mut keeping(&told, invoice),
                noon(),
            )
            .unwrap_err();
        assert!(matches!(refused, NotDone::NeverAsked(_)), "{refused:?}");
        assert!(april.is_file());
        true
    });
    assert!(waited, "a change ran at once on a list without full access");
    assert!(
        told.borrow().is_empty(),
        "a snapshot was taken for a change that waited"
    );
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 2, "{entries:?}");
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.happened().ran())
            .count(),
        1,
        "one OK ran other than once"
    );
    assert!(entries.iter().all(|entry| !entry.was_at_full_access()));
    assert!(
        entries
            .first()
            .is_some_and(|entry| entry.happened().from_approval().is_some())
    );
}

/// **A change the disk refuses at full access is answered as not done and
/// written down as an attempt the machine said no to**, at full access, with
/// what the machine said kept beside it — so the record does not read as a
/// change that worked. The snapshot was asked for, because it was going to run.
#[test]
fn a_change_the_disk_refuses_at_full_access_is_written_down_as_not_done() {
    let told = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_machine("disk-says-no", &mut record, |turning, folder, invoice| {
        fs::write(folder.join("taken.pdf"), "already here").unwrap();
        let at = Grants::default().at_full_access_for(&files());
        let refused = turning
            .changing(
                "rename_file",
                &renaming(invoice, "taken.pdf"),
                &at,
                hour(),
                &mut keeping(&told, invoice),
                noon(),
            )
            .unwrap_err();
        assert!(
            matches!(refused, NotDone::MachineCouldNot(_)),
            "{refused:?}"
        );
        assert!(invoice.is_file());
    });
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 1, "{entries:?}");
    let entry = entries.first().unwrap();
    assert!(entry.was_at_full_access());
    assert!(
        entry
            .told()
            .iter()
            .any(|line| line.as_str().contains("taken.pdf")),
        "an attempt the disk refused was written down with nothing saying so: {:?}",
        entry.told()
    );
    assert_eq!(told.borrow().len(), 1);
}

/// **A change naming something that is not there is refused at full access
/// before any snapshot**, and written down; so is a read offered at the change
/// door, exactly as the proposing road refuses one.
#[test]
fn at_full_access_nothing_there_and_a_read_are_refused_and_written_down() {
    let told = RefCell::new(Vec::new());
    let mut record = Record::default();
    on_a_machine("nothing-there", &mut record, |turning, folder, invoice| {
        let at = Grants::default().at_full_access_for(&files());
        let gone = folder.join("gone.pdf");
        let refused = turning
            .changing(
                "rename_file",
                &renaming(&gone, "still-gone.pdf"),
                &at,
                hour(),
                &mut keeping(&told, invoice),
                noon(),
            )
            .unwrap_err();
        assert!(matches!(refused, NotDone::Refused(_)), "{refused:?}");

        let read = turning
            .changing(
                "list_folder",
                &[("folder", as_given(folder))],
                &at,
                hour(),
                &mut keeping(&told, invoice),
                noon(),
            )
            .unwrap_err();
        assert!(matches!(read, NotDone::NeverAsked(_)), "{read:?}");
    });
    assert!(
        told.borrow().is_empty(),
        "a snapshot was taken for a change that could not run"
    );
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(entries.len(), 2, "{entries:?}");
    assert!(entries.iter().all(|entry| !entry.happened().ran()));
}
