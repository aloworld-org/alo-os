use std::io::Read as _;
use std::os::unix::net::UnixStream;

use alo_acting::{Clicked, ToClick};
use alo_applications::Application;

use super::AskedToClick;

/// **The answer is one word and a newline, on the connection it was asked
/// on**, and the connection is let go after it.
#[test]
fn a_click_is_answered_in_one_word_on_its_own_connection() {
    for answer in [Clicked::Clicked, Clicked::ThePersonIsPressing] {
        let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
        let asked = ToClick::at(
            Application::identified("net.supertuxkart.SuperTuxKart").expect("an identifier"),
            500,
            400,
        )
        .expect("a point");
        let one = AskedToClick::of(asked.clone(), ours);
        assert_eq!(one.asked(), &asked);
        one.answered(answer);
        let mut heard = String::new();
        (&theirs)
            .read_to_string(&mut heard)
            .expect("the answer is read whole");
        assert_eq!(heard, format!("{}\n", answer.written()));
    }
}
