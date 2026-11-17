//! **One picture of the window in front, of that window alone** (ADR 0147 §1),
//! against real clients on a real display.
//!
//! What is asserted is the PNG itself, decoded: its size is the window's, and
//! its pixels are the ones that client drew — so *the picture is of that one
//! window* is about the bytes a reader would be handed, not about this crate's
//! own bookkeeping. And every refusal is a refusal with **no bytes at all**.
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or answer is the failure being reported"
)]

use super::support::{Application, Fixture};
use alo_acting::{Pictured, ToPicture};
use alo_applications::Application as Named;
use alo_shell::typing_into_the_window_in_front::MayTypeInto;

/// The application the window in front belongs to.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

/// Answers the same for every process.
struct Answers(bool);

impl MayTypeInto for Answers {
    fn may_type_into(&mut self, _: &str, _: u32) -> bool {
        self.0
    }
}

fn mapped(f: &Fixture, app_id: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

fn of(application: &str) -> ToPicture {
    ToPicture {
        application: Named::identified(application).expect("an identifier"),
    }
}

fn pictured(f: &Fixture, asked: ToPicture, may: bool) -> (Pictured, Vec<u8>) {
    f.backend(move |s| s.picture_of_the_window_in_front(&asked, &mut Answers(may)))
}

/// **The window in front, drawn alone**: a PNG as big as the window, every
/// pixel the client's own white, and a head that says how many bytes follow.
#[test]
fn the_window_in_front_is_drawn_alone_as_a_png() {
    let f = Fixture::keyboard();
    let _game = mapped(&f, A_GAME);
    f.focus(Some(0)).expect("the window takes the keyboard");
    let (answer, png) = pictured(&f, of(A_GAME), true);
    let Pictured::Picture { bytes, wide, tall } = answer else {
        panic!("no picture: {answer:?}");
    };
    assert_eq!(bytes, png.len(), "the head says another length");
    let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .expect("a PNG")
        .to_rgb8();
    assert_eq!((decoded.width(), decoded.height()), (wide, tall));
    assert_eq!((wide, tall), (16, 16), "not the window's own size");
    assert!(
        decoded.pixels().all(|pixel| pixel.0 == [255, 255, 255]),
        "a pixel is not the client's own"
    );
}

/// **No window has the keyboard: no picture, and no bytes.**
#[test]
fn no_picture_when_no_window_has_the_keyboard() {
    let f = Fixture::keyboard();
    let _game = mapped(&f, A_GAME);
    assert_eq!(
        pictured(&f, of(A_GAME), true),
        (Pictured::NotInFront, Vec::new())
    );
}

/// **Another application's window in front: no picture of it**, however the
/// question is put.
#[test]
fn no_picture_of_another_applications_window() {
    let f = Fixture::keyboard();
    let _browser = mapped(&f, "org.mozilla.firefox");
    f.focus(Some(0)).expect("the window takes the keyboard");
    assert_eq!(
        pictured(&f, of(A_GAME), true),
        (Pictured::NotInFront, Vec::new())
    );
}

/// **A process alo may not reach at the person's setting: no picture.**
#[test]
fn no_picture_of_a_window_alo_may_not_reach() {
    let f = Fixture::keyboard();
    let _game = mapped(&f, A_GAME);
    f.focus(Some(0)).expect("the window takes the keyboard");
    assert_eq!(
        pictured(&f, of(A_GAME), false),
        (Pictured::NotAlosCopy, Vec::new())
    );
}
