//! A text as the approval shows it: all of it, its length said, and what could
//! be missed marked.

use super::*;
use crate::testing::{in_english, translated};

/// The lines put back together, endings and all — what the person was shown,
/// read back into the text it was shown from.
fn rejoined(text: &TheWholeText) -> String {
    let mut whole = String::new();
    for line in text.lines() {
        whole.push_str(&line.as_given());
        whole.push_str(match line.ending() {
            Ending::Newline => "\n",
            Ending::CarriageReturnAndNewline => "\r\n",
            Ending::EndOfText => "",
        });
    }
    whole
}

/// **No limit, and nothing cut.** Two million characters on fifty thousand
/// lines are every one of them in what the approval is handed, and putting the
/// pieces back gives exactly the text that arrived.
#[test]
fn a_very_long_text_is_shown_whole() {
    let line = format!("{}\t// line\n", "x".repeat(32));
    let given = line.repeat(50_000);
    let shown = TheWholeText::of(&given);
    assert_eq!(shown.as_given(), given);
    assert_eq!(rejoined(&shown), given);
    assert_eq!(shown.characters(), given.chars().count());
    assert_eq!(shown.how_many_lines(), 50_000);
    assert_eq!(
        shown.lines().len(),
        50_001,
        "the empty line after the last newline is shown"
    );
    assert_eq!(shown.unseen(), 0);
    assert_eq!(
        shown.how_long(&in_english()).text(),
        "50000 lines, 2050000 characters"
    );
}

/// Newlines are lines; a Windows line ending is a line ending and not
/// something hidden; a carriage return anywhere else is marked.
#[test]
fn newlines_make_lines_and_a_stray_carriage_return_is_marked() {
    let shown = TheWholeText::of("first\nsecond\r\nthird\rstill third\n");
    let endings: Vec<Ending> = shown.lines().iter().map(ShownLine::ending).collect();
    assert_eq!(
        endings,
        [
            Ending::Newline,
            Ending::CarriageReturnAndNewline,
            Ending::Newline,
            Ending::EndOfText
        ]
    );
    assert_eq!(shown.how_many_lines(), 3);
    assert_eq!(shown.unseen(), 1);
    let third = &shown.lines()[2];
    assert_eq!(
        third.pieces()[1],
        Piece::Unseen(Unseen {
            character: '\r',
            why: WhyMarked::Control
        })
    );
    assert_eq!(rejoined(&shown), "first\nsecond\r\nthird\rstill third\n");
}

/// **Hidden characters are accepted and marked where they are**, in plain
/// words with their code points — and the approval says how many there are
/// before the person starts reading.
#[test]
fn what_a_person_could_miss_is_marked_where_it_is() {
    let shown = TheWholeText::of("let total = 1;\u{200B}\n// \u{202E}gnp.exe\n");
    assert_eq!(shown.unseen(), 2);
    let strings = in_english();
    let marks: Vec<String> = shown
        .lines()
        .iter()
        .flat_map(ShownLine::pieces)
        .filter_map(|piece| match piece {
            Piece::Unseen(unseen) => Some(unseen.said(&strings).text().to_owned()),
            Piece::Seen(_) => None,
        })
        .collect();
    assert_eq!(
        marks,
        [
            "invisible character U+200B",
            "character that changes the direction of the text U+202E"
        ]
    );
    assert_eq!(
        shown.lines()[0].pieces(),
        [
            Piece::Seen("let total = 1;".to_owned()),
            Piece::Unseen(Unseen {
                character: '\u{200B}',
                why: WhyMarked::Invisible
            })
        ]
    );
    assert_eq!(
        shown.what_is_marked(&strings).unwrap().text(),
        "2 characters in this text are easy to miss, and each is marked where it is."
    );
    assert_eq!(rejoined(&shown), shown.as_given());
}

/// One of each other kind, and a look-alike letter named by the letter itself.
#[test]
fn every_kind_of_mark_says_what_it_is() {
    let strings = in_english();
    let said = |character: char, why: WhyMarked| Unseen { character, why }.said(&strings);
    assert_eq!(
        said('\u{1b}', WhyMarked::Control).text(),
        "control character U+001B"
    );
    assert_eq!(
        said('\u{00A0}', WhyMarked::Space).text(),
        "unusual space U+00A0"
    );
    assert_eq!(
        said('\u{0430}', WhyMarked::OtherAlphabet).text(),
        "“\u{0430}” from another alphabet U+0430"
    );

    let shown = TheWholeText::of("p\u{0430}ypal.com");
    assert_eq!(shown.unseen(), 1);
    assert_eq!(
        shown.what_is_marked(&strings).unwrap().text(),
        "One character in this text is easy to miss, and it is marked where it is."
    );
}

/// Ordinary text, in any language, marks nothing and says nothing about marks.
#[test]
fn ordinary_text_marks_nothing() {
    let shown = TheWholeText::of("Grüße aus Αθήνα и Київ ❤\u{FE0F}\n\tfn main() {}\n");
    assert_eq!(shown.unseen(), 0);
    assert!(shown.what_is_marked(&in_english()).is_none());
    assert_eq!(
        shown.how_long(&in_english()).text(),
        "2 lines, 40 characters"
    );
}

/// One line and one character are counted the reader's own way.
#[test]
fn one_is_said_as_one() {
    assert_eq!(
        TheWholeText::of("a").how_long(&in_english()).text(),
        "one line, one character"
    );
}

/// The marks are read in the reader's language; the code point is not
/// translated, because it is the character's name everywhere.
#[test]
fn a_mark_is_read_in_the_readers_own_language() {
    let strings = translated(&[(
        crate::words::TEXT_HOLDS_AN_INVISIBLE,
        "unsichtbares Zeichen {code}",
    )]);
    let said = Unseen {
        character: '\u{200B}',
        why: WhyMarked::Invisible,
    }
    .said(&strings);
    assert!(said.is_translated());
    assert_eq!(said.text(), "unsichtbares Zeichen U+200B");
}
