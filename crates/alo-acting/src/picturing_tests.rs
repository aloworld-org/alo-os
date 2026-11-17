use super::*;

/// A game: an application that describes nothing about its window.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

fn a_game() -> Application {
    Application::identified(A_GAME).unwrap()
}

/// **The line names one application and nothing else**, and reads back as
/// what was written.
#[test]
fn the_line_names_one_application_and_reads_back() {
    let asked = ToPicture {
        application: a_game(),
    };
    assert_eq!(asked.written(), "picture-of net.supertuxkart.SuperTuxKart");
    assert_eq!(ToPicture::read(&asked.written()), Some(Ok(asked.clone())));
    assert_eq!(
        ToPicture::read("picture-of net.supertuxkart.SuperTuxKart\n"),
        Some(Ok(asked))
    );
}

/// **A line about anything else is not this act's**, so the door reads it as
/// one of the others.
#[test]
fn a_line_about_something_else_is_not_read_as_a_picture() {
    for line in [
        "type-into net.supertuxkart.SuperTuxKart 3",
        "may-type-into net.supertuxkart.SuperTuxKart",
        "pictures-of net.supertuxkart.SuperTuxKart",
        "",
    ] {
        assert_eq!(ToPicture::read(line), None, "{line:?}");
    }
}

/// **No application, two, one nothing could be, or a position after it is
/// refused** — the act takes no place on the screen and no second window.
#[test]
fn anything_but_exactly_one_application_is_refused() {
    for line in [
        "picture-of",
        "picture-of ",
        "picture-of net.supertuxkart.SuperTuxKart org.mozilla.firefox",
        "picture-of net.supertuxkart.SuperTuxKart 0 0 100 100",
        "picture-of a/b",
    ] {
        assert_eq!(
            ToPicture::read(line),
            Some(Err(NotToPicture::NotOneApplication)),
            "{line:?}"
        );
    }
    let long = format!("picture-of {}", "a".repeat(Request::THE_MOST));
    assert_eq!(ToPicture::read(&long), Some(Err(NotToPicture::TooLong)));
}

/// **A picture's head says how many bytes follow and how big it is**, and
/// reads back exactly; every refusal is one word.
#[test]
fn every_answer_reads_back_as_written() {
    for answer in [
        Pictured::Picture {
            bytes: 1234,
            wide: 800,
            tall: 600,
        },
        Pictured::NotInFront,
        Pictured::NotAlosCopy,
        Pictured::CouldNotDraw,
    ] {
        assert_eq!(
            Pictured::read(&answer.written()),
            Some(answer),
            "{answer:?}"
        );
    }
    assert_eq!(
        Pictured::Picture {
            bytes: 1234,
            wide: 800,
            tall: 600
        }
        .written(),
        "picture 1234 800 600"
    );
}

/// **A head the door would never write is not read as a picture** — above all
/// one with no length, which would leave the reader not knowing where the
/// picture ends.
#[test]
fn a_head_the_door_would_not_write_is_not_an_answer() {
    for line in [
        "picture",
        "picture 0 800 600",
        "picture 12 0 600",
        "picture 12 800 0",
        "picture +12 800 600",
        "picture 12 800",
        "picture 12 800 600 1",
        "pictured",
        "typed",
        "",
    ] {
        assert_eq!(Pictured::read(line), None, "{line:?}");
    }
}

/// **The most a picture may be is said once**, and a head over it is not
/// read: a caller is never made to hold an unbounded picture.
#[test]
fn a_picture_over_the_most_is_not_read() {
    let over = format!("picture {} 800 600", Pictured::THE_MOST + 1);
    assert_eq!(Pictured::read(&over), None);
    let most = format!("picture {} 800 600", Pictured::THE_MOST);
    assert!(Pictured::read(&most).is_some());
}
