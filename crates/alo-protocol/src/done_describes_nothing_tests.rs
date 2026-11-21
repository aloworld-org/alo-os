//! **A read says whether its windows describe nothing**, which opens the
//! screen road for the rest of the turn.

use alo_adapters::{Role, Seen, Shown, ShownWindow};

use super::Done;

fn a_read_of(seen: Vec<Seen>) -> Done {
    Done::of_a_window_read(
        &Shown::of(
            "net.supertuxkart.SuperTuxKart".to_owned(),
            vec![ShownWindow::titled("SuperTuxKart".to_owned(), seen)],
            false,
        ),
        &[],
    )
}

#[test]
fn a_read_of_windows_that_describe_nothing_says_so_and_no_other_does() {
    assert!(
        a_read_of(vec![Seen::of(Role::Canvas, String::new(), None, 1)])
            .a_window_that_describes_nothing()
    );
    let describing: [fn() -> Seen; 3] = [
        || Seen::of(Role::Button, "Start race".to_owned(), None, 1),
        || Seen::of(Role::Part, String::new(), Some("Lap 2".to_owned()), 1),
        || Seen::of(Role::PasswordField, String::new(), None, 1),
    ];
    for seen in describing {
        assert!(
            !a_read_of(vec![seen()]).a_window_that_describes_nothing(),
            "{:?}",
            seen()
        );
    }
    assert!(!Done::of_a_key("org.mozilla.firefox", "enter").a_window_that_describes_nothing());
}
