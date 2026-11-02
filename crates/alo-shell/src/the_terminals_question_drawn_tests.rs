//! The question closing asks, as drawn: inside the transcript, two buttons a
//! press finds where they are seen at any zoom, and a ring on the one the
//! keyboard is on.

use smithay::utils::{Physical, Point, Rectangle};

use super::{ItsWords, WhereTheQuestionIs, the_question};
use crate::closing_the_terminal::OnTheQuestion;
use crate::desktop_look::Measure;

/// A transcript the size of the design's, in a picture's pixels.
fn a_transcript() -> Rectangle<i32, Physical> {
    Rectangle::new((37, 93).into(), (1038, 706).into())
}

/// The question in English, drawn at the ordinary size.
fn drawn(on: OnTheQuestion) -> super::TheQuestionDrawn {
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    the_question(
        &mut labels.fonts,
        ItsWords {
            question: "A command is still running. Close anyway?",
            close_anyway: "Close anyway",
            cancel: "Cancel",
        },
        on,
        a_transcript(),
        (Measure::of(alo_appearance::TextScale::ordinary()), 1),
    )
    .expect("a transcript this wide has room for it")
}

/// **Inside the transcript, with the buttons side by side and 48 tall**, the
/// record confirmation's buttons, which this question is drawn as.
#[test]
fn it_is_drawn_inside_the_transcript_with_two_buttons() {
    let it = drawn(OnTheQuestion::Cancel);
    let transcript = a_transcript();
    assert!(transcript.contains_rect(it.raster.area), "it spills out of the transcript");
    assert!(it.raster.area.contains_rect(it.close_anyway));
    assert!(it.raster.area.contains_rect(it.cancel));
    assert_eq!(it.close_anyway.size.h, 48);
    assert_eq!(it.cancel.size.h, 48);
    assert!(
        it.close_anyway.loc.x + it.close_anyway.size.w <= it.cancel.loc.x,
        "the two buttons overlap"
    );
    assert_eq!(it.close_anyway.loc.y, it.cancel.loc.y);
}

/// **The ring goes where the keyboard is**: a pixel just outside Cancel is
/// inked while the keyboard is on Cancel, and clear while it is on Close
/// anyway.
#[test]
fn the_ring_is_round_the_button_the_keyboard_is_on() {
    let pixel = |drawn: &super::TheQuestionDrawn, x: i32, y: i32| -> [u8; 4] {
        let area = drawn.raster.area;
        let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4)
            .expect("inside");
        let mut out = [0; 4];
        out.copy_from_slice(drawn.raster.pixels.get(at..at + 4).expect("a pixel"));
        out
    };
    let on_cancel = drawn(OnTheQuestion::Cancel);
    let on_the_other = drawn(OnTheQuestion::CloseAnyway);
    // Three pixels left of Cancel's middle: in the ring's two-thick band two
    // off the button.
    let (x, y) = (
        on_cancel.cancel.loc.x - 3,
        on_cancel.cancel.loc.y + on_cancel.cancel.size.h / 2,
    );
    let ringed = pixel(&on_cancel, x, y);
    let clear = pixel(&on_the_other, x, y);
    assert_ne!(ringed, clear, "the ring did not move with the keyboard");
    assert_eq!(&ringed[..3], &[0x10, 0x2A, 0x43], "the ring is not navy");
}

/// **A press finds a button where it is seen**: with the picture drawn at half
/// its size and moved, a point in the middle of Cancel as drawn is Cancel, and
/// a point where Cancel would be at full size is not.
#[test]
fn a_press_finds_a_button_where_the_camera_drew_it() {
    let where_it_is = WhereTheQuestionIs {
        drawn_at: Rectangle::new((300, 200).into(), (600, 450).into()),
        picture: (1200, 900),
        close_anyway: Rectangle::new((100, 100).into(), (200, 48).into()),
        cancel: Rectangle::new((320, 100).into(), (120, 48).into()),
    };
    // Cancel's middle, (380, 124), halved and moved by (300, 200).
    assert_eq!(
        where_it_is.the_button_under(Point::from((490, 262))),
        Some(OnTheQuestion::Cancel)
    );
    assert_eq!(
        where_it_is.the_button_under(Point::from((400, 262))),
        Some(OnTheQuestion::CloseAnyway)
    );
    assert_eq!(
        where_it_is.the_button_under(Point::from((380, 124))),
        None,
        "a press found the button where it would be unzoomed"
    );
}
