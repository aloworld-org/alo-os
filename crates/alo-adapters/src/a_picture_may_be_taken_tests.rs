use super::*;
use crate::shown::{Seen, ShownWindow};

/// A game's windows, as the tree shows them: each window and what is in it.
fn windows(seen: Vec<Vec<Seen>>) -> Shown {
    Shown::of(
        "net.supertuxkart.SuperTuxKart".to_owned(),
        seen.into_iter()
            .map(|seen| ShownWindow::titled("SuperTuxKart".to_owned(), seen))
            .collect(),
        false,
    )
}

fn nameless(role: Role) -> Seen {
    Seen::of(role, String::new(), None, 1)
}

/// **A window that describes nothing may be pictured**: a title, and inside
/// it only things with no name and no text — a game drawing its own canvas.
#[test]
fn a_window_that_describes_nothing_may_be_pictured() {
    assert_eq!(a_picture_may_be_taken(&windows(vec![vec![]])), Ok(()));
    assert_eq!(
        a_picture_may_be_taken(&windows(vec![vec![
            nameless(Role::Part),
            nameless(Role::Part)
        ]])),
        Ok(())
    );
    // **A canvas that names itself still describes nothing in it**: SDL and
    // most game engines give the one area they draw the game's own name.
    let canvas = Seen::of(Role::Canvas, "SuperTuxKart".to_owned(), None, 1);
    assert_eq!(a_picture_may_be_taken(&windows(vec![vec![canvas]])), Ok(()));
}

/// **A window that describes itself is read, never pictured** — the picture is
/// the last resort, for the windows nothing better reaches.
#[test]
fn a_window_that_describes_itself_is_not_pictured() {
    let named = Seen::of(Role::Button, "Start race".to_owned(), None, 2);
    assert_eq!(
        a_picture_may_be_taken(&windows(vec![vec![nameless(Role::Part), named]])),
        Err(NoPicture::ItDescribesItself)
    );
    let text = Seen::of(Role::Part, String::new(), Some("Lap 2 of 3".to_owned()), 2);
    assert_eq!(
        a_picture_may_be_taken(&windows(vec![vec![text]])),
        Err(NoPicture::ItDescribesItself)
    );
}

/// **A password field anywhere, in any window, means no picture at all** — even
/// one with no name, and even where everything else describes nothing.
#[test]
fn a_password_field_anywhere_means_no_picture() {
    assert_eq!(
        a_picture_may_be_taken(&windows(vec![
            vec![nameless(Role::Part)],
            vec![nameless(Role::PasswordField)],
        ])),
        Err(NoPicture::APasswordField)
    );
    // **And it outranks describing itself**: the person is told about the
    // password field, which no setting changes, rather than sent to read.
    let named = Seen::of(Role::Button, "Sign in".to_owned(), None, 2);
    assert_eq!(
        a_picture_may_be_taken(&windows(vec![vec![named, nameless(Role::PasswordField)]])),
        Err(NoPicture::APasswordField)
    );
}

/// **No window on the screen, nothing to picture.**
#[test]
fn no_window_is_no_picture() {
    assert_eq!(
        a_picture_may_be_taken(&windows(vec![])),
        Err(NoPicture::NoWindow)
    );
}
