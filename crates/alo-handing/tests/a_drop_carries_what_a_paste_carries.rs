//! A drop carries what copy and paste carries, and a sandboxed window receives
//! it the way a sandboxed window expects.
//!
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 4, first acceptance:
//! *a drop carries what copy and paste carries — text, images, files — through
//! `alo-clipboard`'s payload types rather than a second set, and the target
//! application receives it through the portal a sandboxed application expects.*
//!
//! The refusals are here beside the drops that work, because the promise is
//! about what arrives **and** what does not: a window that takes none of the
//! forms is not handed a converted one, and a list of files that cannot be read
//! delivers none of them rather than most of them.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_clipboard::{CouldNotGive, Form, Gives, Kind, Offer, Taking};
use alo_handing::{Application, Delivery, Drag, LetGo, NotHanded, Over, Target};

/// The window a person dragged something out of: it answers whatever it is
/// asked for with these bytes.
struct AWindow(Vec<u8>);

impl Gives for AWindow {
    fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        Ok(self.0.clone())
    }
}

/// A drag of these forms, out of a window holding these bytes.
fn dragging(forms: Vec<Kind>, holding: &[u8]) -> Drag {
    Drag::begun(
        Offer::copied(forms).expect("a form is an offer"),
        Box::new(AWindow(holding.to_vec())),
    )
}

/// A sandboxed window that takes these forms.
fn sandboxed(id: &str, takes: Vec<Kind>) -> Target {
    Target::AWindow(Application::sandboxed(id, takes))
}

/// **The promise's three are one mechanism.** Text, an image and a list of
/// files travel the same way, through the same types, with nothing anywhere
/// switching on what sort of thing is being moved — which is the whole reason
/// this crate borrows `alo-clipboard`'s payloads rather than writing a second
/// set of its own.
#[test]
fn a_drop_carries_the_same_text_images_and_files_a_paste_does() {
    for (form, family) in [
        (Kind::text(), Form::Text),
        (Kind::image_png(), Form::AnImage),
        (Kind::files(), Form::Files),
    ] {
        assert_eq!(form.form(), family);
        let drag = dragging(vec![form.clone()], b"https://example.org/a\n");
        let target = Target::AWindow(Application::outside_a_sandbox(
            "org.alo.Terminal",
            vec![form.clone()],
        ));

        assert_eq!(
            drag.over(&target),
            Over::WouldHandItOver {
                form: form.clone(),
                taking: Taking::Copied,
            }
        );
        let LetGo::Handed(handed) = drag.let_go_on(&target).expect("a window that takes it") else {
            unreachable!("a window received it")
        };
        assert_eq!(handed.form(), &form);
        assert_eq!(handed.to(), "org.alo.Terminal");
        assert_eq!(
            handed.delivery().bytes(),
            Some(b"https://example.org/a\n".as_slice()),
            "{form:?}"
        );
    }
}

/// **A file dropped on a sandboxed window arrives through the documents
/// portal**, which is what a sandboxed application expects — and what crosses
/// the boundary is an export of each file, never a location on the person's
/// disk. A path handed into a sandbox would name a file that is not there, and
/// a compositor that made it be there would have turned a drag of the wrist
/// into a grant over somebody's home folder.
#[test]
fn a_file_dropped_on_a_sandboxed_window_arrives_through_the_documents_portal() {
    let drag = dragging(
        vec![Kind::files()],
        b"file:///home/anna/M%C3%BCller.pdf\r\nfile:///home/anna/april.pdf\r\n",
    );
    let LetGo::Handed(handed) = drag
        .let_go_on(&sandboxed("org.alo.Mail", vec![Kind::files()]))
        .expect("a window that takes files")
    else {
        unreachable!("a window received it")
    };

    assert_eq!(handed.to(), "org.alo.Mail");
    assert_eq!(
        handed.delivery(),
        &Delivery::ThroughTheDocuments {
            files: vec![
                PathBuf::from("/home/anna/Müller.pdf"),
                PathBuf::from("/home/anna/april.pdf"),
            ],
        }
    );
    // And there is nothing in that delivery for a path to be carried in.
    assert_eq!(handed.delivery().bytes(), None);
}

/// The same drop onto a window that is **not** sandboxed goes straight over.
/// It already reaches the person's files — it is their own terminal — so
/// exporting them would be ceremony that changed nothing.
#[test]
fn the_same_files_dropped_outside_a_sandbox_go_straight_over() {
    let list = b"file:///home/anna/march.pdf\n";
    let drag = dragging(vec![Kind::files()], list);
    let LetGo::Handed(handed) = drag
        .let_go_on(&Target::AWindow(Application::outside_a_sandbox(
            "org.alo.Terminal",
            vec![Kind::files()],
        )))
        .expect("a window that takes files")
    else {
        unreachable!("a window received it")
    };

    assert_eq!(handed.delivery(), &Delivery::OnTheSpot(list.to_vec()));
}

/// A link dragged out of a browser is a `text/uri-list` with no file in it. It
/// is a perfectly good drop, and there is nothing to export.
#[test]
fn a_link_dragged_to_a_sandboxed_window_has_nothing_to_export() {
    let drag = dragging(vec![Kind::files()], b"https://example.org/march\n");
    let LetGo::Handed(handed) = drag
        .let_go_on(&sandboxed("org.alo.Mail", vec![Kind::files()]))
        .expect("a window that takes files")
    else {
        unreachable!("a window received it")
    };

    assert_eq!(
        handed.delivery(),
        &Delivery::OnTheSpot(b"https://example.org/march\n".to_vec())
    );
}

/// **A window that takes none of the forms is refused, and is never handed a
/// converted one** — and the window the drag came from is not asked for a byte.
/// Converting would hand an application bytes that the window which had them
/// never said it could produce.
#[test]
fn a_window_that_takes_none_of_the_forms_is_refused_rather_than_converted() {
    let drag = dragging(vec![Kind::image_png()], b"\x89PNG\r\n\x1a\n");
    let notes = sandboxed("org.alo.Notes", vec![Kind::text()]);

    assert_eq!(drag.over(&notes), Over::WouldNotTakeIt);
    assert_eq!(
        drag.let_go_on(&notes).unwrap_err(),
        NotHanded::WillNotTakeIt
    );
}

/// **A list of files that cannot be read delivers none of them.** Three files
/// out of four would be a drop that lost one without saying so, and the person
/// gets one sentence while what the application got wrong is kept beside it.
#[test]
fn a_list_of_files_that_cannot_be_read_delivers_none_of_them() {
    for list in [
        b"file://otherbox/home/anna/a.txt".as_slice(),
        b"file:///home/anna/../../etc/shadow".as_slice(),
        b"not a uri at all".as_slice(),
    ] {
        let drag = dragging(vec![Kind::files()], list);
        let refused = drag
            .let_go_on(&sandboxed("org.alo.Mail", vec![Kind::files()]))
            .unwrap_err();
        assert!(
            matches!(refused, NotHanded::NotAFileList { .. }),
            "{refused:?}"
        );
    }
}

/// Letting go over nothing at all leaves everything where it was.
#[test]
fn letting_go_over_nothing_leaves_everything_where_it_was() {
    let drag = dragging(vec![Kind::text()], b"the second paragraph");
    assert_eq!(drag.over(&Target::Nothing), Over::NothingHere);

    let drag = dragging(vec![Kind::text()], b"the second paragraph");
    assert_eq!(
        drag.let_go_on(&Target::Nothing).unwrap_err(),
        NotHanded::NothingHere
    );
}

/// **Everything this crate says is in the machine's one vocabulary.** A crate
/// whose words nothing collects reaches a real shell as a missing key, in
/// English and in every language somebody has translated.
#[test]
fn everything_this_crate_says_is_collected_into_the_machines_words() {
    let machine = alo_saying::everything_this_machine_can_say().expect("the machine's own words");
    for word in alo_handing::words::EVERY_WORD {
        assert!(
            machine.phrase(&word.key()).is_some(),
            "nothing collects {}",
            word.named()
        );
    }
}
