//! The FileChooser portal on a real bus, written from the promise in
//! `docs/contracts/portals.md` (the row *file chooser and documents*, and *How
//! an answer arrives*) by a reader who has not read the implementation: an
//! application asks, **alo's own sheet** is shown, the person's choice is the
//! grant, the choice is recorded before it is handed over, and **that file and
//! no other** is exported to the asking application. The refusals are held as
//! carefully: nothing chosen, a choice that cannot be believed, an export that
//! fails, what is not offered yet, and a caller with no sandbox each export
//! nothing and leave a line saying why.
//!
//! The desktop ([`ADesktop`]) and the documents portal ([`AnExporter`]) are
//! stand-ins that keep everything they were asked and answer as each test says.
//! The files are real, in a folder of this test's own under the temporary
//! directory, because a choice must be a file that is there. What this cannot
//! show is a real sheet on a booted machine and a real `/run/user/<uid>/doc/`
//! mount; that is a walk, and it is owed rather than implied.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use alo_capability::{Applicant, Grants};
use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_portals::{
    Allowed, Answered, Appearance, Backend, ChoosesFiles, ExportedFor, ExportsDocuments,
    FileSheetAnswer, KeepsSecrets, Kept, KeptOutcome, NoFile, NotExported, NotKept, Request,
    Sandboxes, Served, TheMachine, TimeOfDay, ToChoose, Unanswered, file_uri,
};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const EDITOR: &str = "org.gnome.TextEditor";

const DESKTOP: &str = "org.freedesktop.portal.Desktop";
const PORTALS: &str = "/org/freedesktop/portal/desktop";
const FILE_CHOOSER: &str = "org.freedesktop.portal.FileChooser";

/// Where the stand-in documents portal says an exported file is opened from.
const DOCUMENTS: &str = "/run/user/1000/doc/a1b2c3";

/// A machine with nothing granted to anybody: the person's choice on the sheet
/// is the only grant this portal needs.
struct AMachine;

impl TheMachine for AMachine {
    fn grants(&self) -> Option<Grants> {
        Some(Grants::default())
    }

    fn applications(&self) -> Option<alo_portals::Applications> {
        None
    }

    fn appearance(&self) -> Option<Appearance> {
        Some(Appearance::shipped())
    }

    fn time_of_day(&self) -> Option<TimeOfDay> {
        TimeOfDay::checked(12, 0).ok()
    }

    fn reaching(&self) -> Option<alo_portals::Reaching> {
        None
    }
}

/// No keyring: nothing here asks the Secret portal.
struct NoKeyring;

impl KeepsSecrets for NoKeyring {
    fn hand_over(
        &self,
        _request: &Request,
        _grants: &Grants,
        _now: SystemTime,
        _into: &mut dyn std::io::Write,
    ) -> Result<Allowed, NotKept> {
        Err(NotKept::Unavailable)
    }
}

/// One sheet the desktop was asked to show: who asked, what, and where it
/// starts.
type Sheet = (String, ToChoose, Option<PathBuf>);

/// A desktop that keeps every sheet it was asked to show, and answers with
/// what the person does, as the test sets it.
struct ADesktop {
    shown: Mutex<Vec<Sheet>>,
    the_person: Mutex<FileSheetAnswer>,
}

impl ChoosesFiles for ADesktop {
    fn chosen(&self, asking: &str, to: &ToChoose, starting_in: Option<&Path>) -> FileSheetAnswer {
        self.shown.lock().unwrap().push((
            asking.to_owned(),
            to.clone(),
            starting_in.map(Path::to_path_buf),
        ));
        self.the_person.lock().unwrap().clone()
    }
}

/// One export the documents portal was asked for, and what the record held at
/// that moment — which is how *recorded before it is handed over* is checked.
struct Export {
    application: String,
    file: PathBuf,
    what: ExportedFor,
    recorded_by_then: Vec<KeptOutcome>,
}

/// A documents portal that keeps every export it was asked for and answers as
/// the test sets: `Ok(folder)` exports into that folder under the file's own
/// name, `Err` refuses.
struct AnExporter {
    record: Arc<Kept>,
    asked: Mutex<Vec<Export>>,
    answering: Mutex<Result<PathBuf, NotExported>>,
}

impl ExportsDocuments for AnExporter {
    fn exported(
        &self,
        application: &str,
        file: &Path,
        what: ExportedFor,
    ) -> Result<PathBuf, NotExported> {
        let recorded_by_then = self
            .record
            .everything()
            .iter()
            .map(|answered| KeptOutcome::from(answered.outcome()))
            .collect();
        self.asked.lock().unwrap().push(Export {
            application: application.to_owned(),
            file: file.to_path_buf(),
            what,
            recorded_by_then,
        });
        let answering = self.answering.lock().unwrap().clone();
        answering.map(|folder| folder.join(file.file_name().unwrap()))
    }
}

/// One backend on a bus of its own, a client to ask it from, and a folder of
/// real files for the person to choose from.
struct Serving {
    _bus: AKeyringOfOurOwn,
    place: PathBuf,
    processes: PathBuf,
    record: Arc<Kept>,
    desktop: Option<Arc<ADesktop>>,
    exporter: Option<Arc<AnExporter>>,
    client: Connection,
    _served: Served,
}

impl Serving {
    /// `desktop` is how the person answers the sheet, or [`None`] for a
    /// backend never told how to choose files at all.
    fn started(what: &str, desktop: Option<FileSheetAnswer>) -> Self {
        let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
        let place =
            std::env::temp_dir().join(format!("alo-portal-files-{what}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&place);
        let processes = place.join("proc");
        std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
            .unwrap();
        let files = place.join("files");
        std::fs::create_dir_all(files.join("photos")).unwrap();
        std::fs::write(files.join("the letter.txt"), "Dear Anna,\n").unwrap();
        std::fs::write(files.join("photos").join("beach.jpg"), [0xff, 0xd8, 0xff]).unwrap();

        let record = Arc::new(Kept::nothing());
        let backend = Backend::answering_from(
            Arc::new(AMachine),
            Arc::new(NoKeyring),
            Sandboxes::under(&processes),
            record.clone(),
        );
        let (backend, desktop, exporter) = match desktop {
            Some(the_person) => {
                let desktop = Arc::new(ADesktop {
                    shown: Mutex::new(Vec::new()),
                    the_person: Mutex::new(the_person),
                });
                let exporter = Arc::new(AnExporter {
                    record: record.clone(),
                    asked: Mutex::new(Vec::new()),
                    answering: Mutex::new(Ok(PathBuf::from(DOCUMENTS))),
                });
                (
                    backend.choosing_files_with(desktop.clone(), exporter.clone()),
                    Some(desktop),
                    Some(exporter),
                )
            }
            None => (backend, None, None),
        };
        let served = backend
            .serve_on(&bus.address())
            .expect("the portals are served on the test's bus");
        let client = zbus::blocking::connection::Builder::address(bus.address().as_str())
            .unwrap()
            .build()
            .unwrap();
        let serving = Self {
            _bus: bus,
            place,
            processes,
            record,
            desktop,
            exporter,
            client,
            _served: served,
        };
        serving.become_(Some(EDITOR));
        serving
    }

    /// The folder of real files the person chooses from.
    fn files(&self) -> PathBuf {
        self.place.join("files")
    }

    /// From now on this test process is `application`'s sandbox, or for
    /// [`None`] a program with none.
    fn become_(&self, application: Option<&str>) {
        let info = self
            .processes
            .join(std::process::id().to_string())
            .join("root")
            .join(".flatpak-info");
        match application {
            Some(name) => std::fs::write(info, format!("[Application]\nname={name}\n")).unwrap(),
            None => {
                let _ = std::fs::remove_file(info);
            }
        }
    }

    /// From now on the person answers the sheet with `chose`.
    fn the_person_answers(&self, chose: FileSheetAnswer) {
        *self.desktop.as_ref().unwrap().the_person.lock().unwrap() = chose;
    }

    /// From now on the documents portal answers `answering`.
    fn the_documents_portal_answers(&self, answering: Result<PathBuf, NotExported>) {
        *self.exporter.as_ref().unwrap().answering.lock().unwrap() = answering;
    }

    /// `method` of the FileChooser with `options` and `handle_token` `token`,
    /// listening on the predicted handle first; the response code and results.
    fn choose(
        &self,
        method: &str,
        token: &str,
        mut options: HashMap<&'static str, Value<'static>>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        let sender = self.client.unique_name().unwrap().as_str();
        let handle = format!(
            "{PORTALS}/request/{}/{token}",
            sender.trim_start_matches(':').replace('.', "_")
        );
        let request = Proxy::new(
            &self.client,
            DESKTOP,
            handle.as_str(),
            "org.freedesktop.portal.Request",
        )
        .unwrap();
        let mut responses = request.receive_signal("Response").unwrap();
        options.insert("handle_token", Value::from(token.to_owned()));
        let reply = self
            .client
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some(FILE_CHOOSER),
                method,
                &("", "Choose a file", options),
            )
            .expect("the file chooser answers the call");
        let returned: OwnedObjectPath = reply.body().deserialize().unwrap();
        assert_eq!(returned.as_str(), handle);
        let response = responses.next().expect("a response");
        response.body().deserialize().unwrap()
    }

    fn everything(&self) -> Vec<Answered> {
        self.record.everything()
    }

    fn kept(&self) -> Vec<KeptOutcome> {
        self.everything()
            .iter()
            .map(|answered| KeptOutcome::from(answered.outcome()))
            .collect()
    }

    fn sheets_shown(&self) -> Vec<Sheet> {
        self.desktop
            .as_ref()
            .map(|desktop| desktop.shown.lock().unwrap().clone())
            .unwrap_or_default()
    }

    fn exports_asked(&self) -> Vec<(String, PathBuf, ExportedFor)> {
        self.exporter
            .as_ref()
            .map(|exporter| {
                exporter
                    .asked
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|export| (export.application.clone(), export.file.clone(), export.what))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Drop for Serving {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.place);
    }
}

/// `path` as the specification's `ay`: its bytes, ending in a zero byte.
fn whole_path(path: &Path) -> Value<'static> {
    let mut bytes = path.as_os_str().as_bytes().to_vec();
    bytes.push(0);
    Value::from(bytes)
}

/// The `uris` of `results`, or [`None`] where there is none; anything that is
/// not an `as` fails the test.
fn uris(results: &HashMap<String, OwnedValue>) -> Option<Vec<String>> {
    let value = results.get("uris")?;
    match &**value {
        Value::Array(array) => Some(
            array
                .iter()
                .map(|one| match one {
                    Value::Str(uri) => uri.as_str().to_owned(),
                    other => panic!("a uri that is not a string: {other:?}"),
                })
                .collect(),
        ),
        other => panic!("uris that are not an array: {other:?}"),
    }
}

/// GTK 4.20.4 reads the URI array before examining the cancellation/refusal
/// code. No file must therefore be represented by a present, typed empty `as`,
/// without implying success or exporting anything. Other result fields remain
/// absent on these answers, and an untyped or differently typed empty fails.
fn only_empty_uris(results: &HashMap<String, OwnedValue>) -> bool {
    results.len() == 1
        && results
            .get("uris")
            .is_some_and(|value| value.value_signature().to_string() == "as")
        && uris(results).is_some_and(|files| files.is_empty())
}

/// Whether the record's `file` names `path`. The contract says the record
/// names *the file* and not in which form, so either its path or its address
/// is accepted.
fn names(file: &str, path: &Path) -> bool {
    file == path.to_string_lossy() || file == file_uri(path)
}

fn no_options() -> HashMap<&'static str, Value<'static>> {
    HashMap::new()
}

/// **A file is opened on alo's own sheet, with no grant asked**: the sheet is
/// asked for the application that asked, to open, starting in
/// `current_folder`; the choice is recorded **before** it is handed over;
/// **that file and no other** is exported, read and written; and the answer is
/// `0` with the exported file's address as the one entry of `uris`.
#[test]
fn an_opened_file_is_recorded_then_exported_and_answered_with_its_one_address() {
    let serving = Serving::started("open", Some(FileSheetAnswer::Cancelled));
    let letter = serving.files().join("the letter.txt");
    serving.the_person_answers(FileSheetAnswer::Picked(letter.clone()));
    let mut options = no_options();
    options.insert("current_folder", whole_path(&serving.files()));
    let (response, results) = serving.choose("OpenFile", "open", options);

    assert_eq!(
        serving.sheets_shown(),
        [(EDITOR.to_owned(), ToChoose::Open, Some(serving.files()))]
    );
    assert_eq!(response, 0);
    assert_eq!(
        serving.exports_asked(),
        [(
            EDITOR.to_owned(),
            letter.clone(),
            ExportedFor::Opening { writable: true }
        )]
    );
    let exported = PathBuf::from(DOCUMENTS).join("the letter.txt");
    assert_eq!(uris(&results), Some(vec![file_uri(&exported)]));
    assert_eq!(results.len(), 1, "only uris is answered: {results:?}");

    let exporter = serving.exporter.as_ref().unwrap();
    let asked = exporter.asked.lock().unwrap();
    match asked.first().unwrap().recorded_by_then.as_slice() {
        [KeptOutcome::FileChosen { file, saving }] => {
            assert!(names(file, &letter), "{file}");
            assert!(!saving);
        }
        other => panic!("not recorded before it was exported: {other:?}"),
    }
    drop(asked);

    let everything = serving.everything();
    assert_eq!(everything.len(), 1, "one choice, one line");
    assert_eq!(
        everything.first().unwrap().application(),
        Some(&Applicant::named(EDITOR))
    );
}

/// **A `current_folder` that is not a whole path starts the sheet in the
/// person's home**, because only a whole path is a folder anybody can name.
#[test]
fn a_current_folder_that_is_not_a_whole_path_starts_at_home() {
    let serving = Serving::started("relative-folder", Some(FileSheetAnswer::Cancelled));
    let mut options = no_options();
    options.insert("current_folder", whole_path(Path::new("files")));
    let (response, _) = serving.choose("OpenFile", "relative", options);
    assert_eq!(response, 1);
    assert_eq!(
        serving.sheets_shown(),
        [(EDITOR.to_owned(), ToChoose::Open, None)]
    );
}

/// **A save starts in `current_folder` under `current_name`, and
/// `current_file` wins over both, with its folder and its name**; what the
/// person names is recorded as a save and exported to save, and the answer is
/// `0` with its one address.
#[test]
fn a_save_starts_where_it_was_asked_and_current_file_wins() {
    let serving = Serving::started("save", Some(FileSheetAnswer::Cancelled));
    let saved = serving.files().join("minutes.odt");
    serving.the_person_answers(FileSheetAnswer::Picked(saved.clone()));
    let mut options = no_options();
    options.insert("current_folder", whole_path(&serving.files()));
    options.insert("current_name", Value::from("minutes.odt".to_owned()));
    let (response, results) = serving.choose("SaveFile", "save", options);

    assert_eq!(
        serving.sheets_shown(),
        [(
            EDITOR.to_owned(),
            ToChoose::Save {
                name: "minutes.odt".to_owned()
            },
            Some(serving.files())
        )]
    );
    assert_eq!(response, 0);
    assert_eq!(
        serving.exports_asked(),
        [(EDITOR.to_owned(), saved.clone(), ExportedFor::Saving)]
    );
    assert_eq!(
        uris(&results),
        Some(vec![file_uri(
            &PathBuf::from(DOCUMENTS).join("minutes.odt")
        )])
    );
    match serving
        .exporter
        .as_ref()
        .unwrap()
        .asked
        .lock()
        .unwrap()
        .first()
        .unwrap()
        .recorded_by_then
        .as_slice()
    {
        [KeptOutcome::FileChosen { file, saving }] => {
            assert!(names(file, &saved), "{file}");
            assert!(saving);
        }
        other => panic!("not recorded before it was exported: {other:?}"),
    }

    let photos = serving.files().join("photos");
    let mut options = no_options();
    options.insert("current_folder", whole_path(&serving.files()));
    options.insert("current_name", Value::from("minutes.odt".to_owned()));
    options.insert("current_file", whole_path(&photos.join("beach.jpg")));
    serving.the_person_answers(FileSheetAnswer::Cancelled);
    let (response, _) = serving.choose("SaveFile", "current_file", options);
    assert_eq!(response, 1);
    assert_eq!(
        serving.sheets_shown().get(1),
        Some(&(
            EDITOR.to_owned(),
            ToChoose::Save {
                name: "beach.jpg".to_owned()
            },
            Some(photos)
        ))
    );
}

/// **Nothing is exported when nothing was chosen**: cancel is `1`, and nobody
/// answering, a desktop that cannot show the sheet and no desktop at all are
/// `2` — one line each, saying which.
#[test]
fn nothing_chosen_exports_nothing_and_says_why() {
    for (what, chose, response, why) in [
        ("cancel", FileSheetAnswer::Cancelled, 1, NoFile::Cancelled),
        ("nobody", FileSheetAnswer::Unanswered, 2, NoFile::Unanswered),
        (
            "no-sheet",
            FileSheetAnswer::CannotShow,
            2,
            NoFile::CannotShow,
        ),
        ("gone", FileSheetAnswer::NoDesktop, 2, NoFile::NoDesktop),
        (
            "broken",
            FileSheetAnswer::CouldNotAsk,
            2,
            NoFile::CouldNotAsk,
        ),
    ] {
        let serving = Serving::started(what, Some(chose));
        for method in ["OpenFile", "SaveFile"] {
            let (answer, results) = serving.choose(method, "sheet", no_options());
            assert_eq!(answer, response, "{what} {method}");
            assert!(only_empty_uris(&results), "{what} {method}: {results:?}");
        }
        assert_eq!(serving.sheets_shown().len(), 2, "{what}");
        assert!(serving.exports_asked().is_empty(), "{what} exported");
        assert_eq!(
            serving.kept(),
            [
                KeptOutcome::FileUnchosen { why },
                KeptOutcome::FileUnchosen { why }
            ],
            "{what}"
        );
    }
}

/// **A file outside the person's pick is never exported**: what the desktop
/// answers that is not a file that is there, to open, or a name in a folder
/// that is there and is not itself a folder, to save, is not believed — `2`,
/// nothing exported, one line.
#[test]
fn a_choice_that_cannot_be_believed_exports_nothing() {
    let serving = Serving::started("unbelieved", Some(FileSheetAnswer::Cancelled));
    let files = serving.files();
    let by_way_of_dot_dot = files.join("photos").join("..").join("the letter.txt");
    assert!(by_way_of_dot_dot.exists(), "the test's own premise");
    for (method, picked) in [
        ("OpenFile", files.join("gone.txt")),
        ("OpenFile", files.join("photos")),
        ("OpenFile", PathBuf::from("the letter.txt")),
        ("OpenFile", by_way_of_dot_dot),
        ("SaveFile", files.join("photos")),
        ("SaveFile", files.join("nowhere").join("minutes.odt")),
        ("SaveFile", PathBuf::from("minutes.odt")),
        (
            "SaveFile",
            files.join("photos").join("..").join("minutes.odt"),
        ),
    ] {
        let lines = serving.everything().len();
        serving.the_person_answers(FileSheetAnswer::Picked(picked.clone()));
        let (response, results) = serving.choose(method, "unbelieved", no_options());
        assert_eq!(response, 2, "{method} {picked:?}");
        assert!(
            only_empty_uris(&results),
            "{method} {picked:?}: {results:?}"
        );
        assert!(
            serving.exports_asked().is_empty(),
            "{method} {picked:?} exported"
        );
        let kept = serving.kept();
        assert_eq!(kept.len(), lines + 1, "{method} {picked:?}: one line");
        assert_eq!(
            kept.last(),
            // The author changed `CouldNotAsk` to `NotBelieved` here, on this
            // file's own finding 1: the desktop was asked, and answered.
            Some(&KeptOutcome::FileUnchosen {
                why: NoFile::NotBelieved
            }),
            "{method} {picked:?}"
        );
    }
}

/// **A file that could not be handed over is `2`, with a second line saying
/// why** — the choice is still recorded first, and no address is answered.
#[test]
fn a_file_that_could_not_be_handed_over_is_refused_on_a_second_line() {
    for (what, refused) in [
        ("no-documents", NotExported::NoDocumentsPortal),
        ("not-there", NotExported::NotThere),
        ("refused", NotExported::Refused),
    ] {
        let serving = Serving::started(what, Some(FileSheetAnswer::Cancelled));
        let letter = serving.files().join("the letter.txt");
        serving.the_person_answers(FileSheetAnswer::Picked(letter.clone()));
        serving.the_documents_portal_answers(Err(refused));
        let (response, results) = serving.choose("OpenFile", "export", no_options());
        assert_eq!(response, 2, "{what}");
        assert_eq!(uris(&results), Some(Vec::new()), "{what}");
        assert!(only_empty_uris(&results), "{what}: {results:?}");
        assert_eq!(serving.exports_asked().len(), 1, "{what}");
        match serving.kept().as_slice() {
            [
                KeptOutcome::FileChosen { file, saving },
                KeptOutcome::FileUnchosen { why },
            ] => {
                assert!(names(file, &letter), "{what}: {file}");
                assert!(!saving, "{what}");
                assert_eq!(*why, NoFile::NotHandedOver(refused), "{what}");
            }
            other => panic!("{what}: {other:?}"),
        }
    }
}

/// **A folder (`directory`) and `SaveFiles` are not offered yet**: `2`, no
/// sheet shown, nothing exported, recorded as not offered.
#[test]
fn a_folder_and_save_files_are_not_offered_yet() {
    let serving = Serving::started("not-offered", Some(FileSheetAnswer::Cancelled));
    serving.the_person_answers(FileSheetAnswer::Picked(serving.files().join("photos")));
    let mut options = no_options();
    options.insert("directory", Value::from(true));
    let (response, results) = serving.choose("OpenFile", "folder", options);
    assert_eq!(response, 2);
    assert!(only_empty_uris(&results), "{results:?}");

    let mut options = no_options();
    options.insert("current_folder", whole_path(&serving.files()));
    let (response, results) = serving.choose("SaveFiles", "several", options);
    assert_eq!(response, 2);
    assert!(only_empty_uris(&results), "{results:?}");

    assert!(serving.sheets_shown().is_empty());
    assert!(serving.exports_asked().is_empty());
    assert_eq!(
        serving.kept(),
        [
            KeptOutcome::FileUnchosen {
                why: NoFile::NotOfferedYet
            },
            KeptOutcome::FileUnchosen {
                why: NoFile::NotOfferedYet
            }
        ]
    );
}

/// **`multiple` is answered with the one file the person chooses.**
#[test]
fn multiple_is_answered_with_exactly_one_file() {
    let serving = Serving::started("multiple", Some(FileSheetAnswer::Cancelled));
    let beach = serving.files().join("photos").join("beach.jpg");
    serving.the_person_answers(FileSheetAnswer::Picked(beach.clone()));
    let mut options = no_options();
    options.insert("multiple", Value::from(true));
    let (response, results) = serving.choose("OpenFile", "multiple", options);
    assert_eq!(response, 0);
    assert_eq!(
        uris(&results),
        Some(vec![file_uri(&PathBuf::from(DOCUMENTS).join("beach.jpg"))])
    );
    assert_eq!(
        serving.exports_asked(),
        [(
            EDITOR.to_owned(),
            beach,
            ExportedFor::Opening { writable: true }
        )]
    );
}

/// **A caller with no sandbox is refused before any sheet is shown**: `2`,
/// nothing exported, recorded as not identified and naming nobody.
#[test]
fn a_caller_with_no_sandbox_is_refused_before_any_sheet() {
    let serving = Serving::started("unnamed", Some(FileSheetAnswer::Cancelled));
    serving.the_person_answers(FileSheetAnswer::Picked(
        serving.files().join("the letter.txt"),
    ));
    serving.become_(None);
    for method in ["OpenFile", "SaveFile"] {
        let (response, results) = serving.choose(method, "unnamed", no_options());
        assert_eq!(response, 2, "{method}");
        assert!(only_empty_uris(&results), "{method}: {results:?}");
    }
    assert!(serving.sheets_shown().is_empty());
    assert!(serving.exports_asked().is_empty());
    let everything = serving.everything();
    assert_eq!(everything.len(), 2);
    for answered in &everything {
        assert_eq!(answered.application(), None);
        assert_eq!(
            KeptOutcome::from(answered.outcome()),
            KeptOutcome::Unanswered {
                why: Unanswered::NotIdentified
            }
        );
    }
}

/// **With no desktop at all, no sheet is shown and nothing is chosen**: a
/// backend never told how to choose files answers `2`, recorded as no desktop.
#[test]
fn with_no_desktop_nothing_is_chosen_and_the_record_says_so() {
    let serving = Serving::started("no-desktop", None);
    let (response, results) = serving.choose("OpenFile", "nobody", no_options());
    assert_eq!(response, 2);
    assert!(only_empty_uris(&results), "{results:?}");
    assert_eq!(
        serving.kept(),
        [KeptOutcome::FileUnchosen {
            why: NoFile::NoDesktop
        }]
    );
}

/// **A token no handle can be made from is refused on the call**, and no
/// sheet is shown for it.
#[test]
fn a_token_no_handle_can_be_made_from_is_refused_on_the_call() {
    let serving = Serving::started("bad-token", Some(FileSheetAnswer::Cancelled));
    let mut options = no_options();
    options.insert("handle_token", Value::from("not a token!".to_owned()));
    let refused = serving
        .client
        .call_method(
            Some(DESKTOP),
            PORTALS,
            Some(FILE_CHOOSER),
            "OpenFile",
            &("", "Choose a file", options),
        )
        .expect_err("a token with spaces is refused");
    match refused {
        zbus::Error::MethodError(name, _, _) => {
            assert_eq!(name.as_str(), "org.freedesktop.DBus.Error.InvalidArgs");
        }
        other => panic!("{other:?}"),
    }
    assert!(serving.sheets_shown().is_empty());
    assert!(serving.exports_asked().is_empty());
}

/// **Cancelling GTK's filtered SaveFile request is a complete non-success
/// response with a typed empty URI array.** The filter options do not turn a
/// cancellation into a chosen destination, and nothing is exported or created.
#[test]
fn a_cancelled_gtk_save_with_filters_answers_a_typed_empty_uri_array() {
    let serving = Serving::started("gtk-save-cancel", Some(FileSheetAnswer::Cancelled));
    let destination = serving.files().join("Screen demonstration.webm");
    let filter = (
        "WebM video".to_owned(),
        vec![(1_u32, "video/webm".to_owned())],
    );
    let mut options = no_options();
    options.insert("current_folder", whole_path(&serving.files()));
    options.insert(
        "current_name",
        Value::from("Screen demonstration.webm".to_owned()),
    );
    options.insert("filters", Value::from(vec![filter.clone()]));
    options.insert("current_filter", Value::from(filter));
    options.insert("multiple", Value::from(false));
    options.insert("modal", Value::from(true));
    let (response, results) = serving.choose("SaveFile", "gtk_save_cancel", options);

    assert_eq!(response, 1, "the person cancelled; no success is implied");
    assert!(
        only_empty_uris(&results),
        "GTK needs an explicit empty as: {results:?}"
    );
    assert_eq!(
        serving.sheets_shown(),
        [(
            EDITOR.to_owned(),
            ToChoose::Save {
                name: "Screen demonstration.webm".to_owned()
            },
            Some(serving.files())
        )]
    );
    assert_eq!(
        serving.kept(),
        [KeptOutcome::FileUnchosen {
            why: NoFile::Cancelled
        }]
    );
    assert!(serving.exports_asked().is_empty());
    assert!(
        !destination.exists(),
        "cancelling must not create a destination"
    );
}

/// **Failed SaveFile handover is safe for the same GTK consumer.** The
/// selected name is recorded, but the failure answers `2` with no URI and
/// cannot be mistaken for a successful file grant or a saved recording.
#[test]
fn a_refused_save_handover_answers_a_typed_empty_uri_array() {
    for (name, refusal) in [
        ("gtk-save-no-documents", NotExported::NoDocumentsPortal),
        ("gtk-save-destination-gone", NotExported::NotThere),
        ("gtk-save-export-refused", NotExported::Refused),
    ] {
        let serving = Serving::started(name, Some(FileSheetAnswer::Cancelled));
        let destination = serving.files().join("Screen demonstration.webm");
        serving.the_person_answers(FileSheetAnswer::Picked(destination.clone()));
        serving.the_documents_portal_answers(Err(refusal));
        let mut options = no_options();
        options.insert(
            "current_name",
            Value::from("Screen demonstration.webm".to_owned()),
        );
        let (response, results) = serving.choose("SaveFile", "gtk_save_refused", options);

        assert_eq!(response, 2, "{name}");
        assert!(
            only_empty_uris(&results),
            "{name}: GTK needs an explicit empty as: {results:?}"
        );
        assert_eq!(
            serving.exports_asked(),
            [(EDITOR.to_owned(), destination.clone(), ExportedFor::Saving)]
        );
        match serving.kept().as_slice() {
            [
                KeptOutcome::FileChosen { file, saving },
                KeptOutcome::FileUnchosen { why },
            ] => {
                assert!(names(file, &destination), "{name}: {file}");
                assert!(*saving, "{name}: this was a save, not an open");
                assert_eq!(*why, NoFile::NotHandedOver(refusal), "{name}");
            }
            other => panic!("{name}: the actual save/refusal was not recorded: {other:?}"),
        }
        assert!(
            !destination.exists(),
            "{name}: failed handover created a file"
        );
    }
}
