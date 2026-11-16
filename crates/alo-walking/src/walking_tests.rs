//! A walk carried out against a machine of pictures and log lines: what it
//! presses, what it checks, where it stops, and what it never says.

use std::time::Duration;

use super::walk;
use crate::a_machine_of_pictures::PicturesAndLines;
use crate::frame::Frame;
use crate::keys::Chord;
use crate::report::{Check, Report, Verdict};
use crate::settling::Settling;
use crate::step::{Since, Walk};
use crate::strokes::Pace;

/// No waiting between keys, so a test's clock moves only for looks and logs.
const QUICK: Pace = Pace {
    hold: Duration::ZERO,
    between: Duration::ZERO,
};

/// A short look.
const LOOK: Settling = Settling {
    every: Duration::from_secs(1),
    matching: 2,
    noise: 0,
    at_most: Duration::from_secs(5),
};

/// One screen, then another.
fn screens() -> Vec<Frame> {
    vec![
        Frame::filled(4, 4, [0, 0, 0]),
        Frame::filled(4, 4, [9, 9, 9]),
    ]
}

/// No secret is set.
fn nothing_set(_: &str) -> Option<String> {
    None
}

/// The checks, as (name, verdict).
fn verdicts(report: &Report) -> Vec<(String, Verdict)> {
    report
        .checks()
        .into_iter()
        .map(|Check { name, verdict, .. }| (name.clone(), *verdict))
        .collect()
}

/// Keys are pressed in order, a look that sees the change passes, and a log
/// line said in time passes with the line as its evidence.
#[test]
fn a_walk_presses_looks_and_reads() {
    let mut machine =
        PicturesAndLines::showing(screens()).saying(Duration::from_secs(3), "the desktop is up");
    let walked = Walk::named("w")
        .press(Chord::key("tab"))
        .type_text("Ab")
        .look("02-after", LOOK)
        .expect_line(
            "the desktop",
            "desktop is up",
            Since::Boot,
            Duration::from_secs(10),
        );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(machine.keys_pressed(), vec!["tab", "shift", "a", "b"]);
    assert_eq!(machine.kept, vec!["02-after".to_owned()]);
    assert!(report.passed(), "{}", report.as_text());
    let text = report.as_text();
    // The look ended at 2 s and the line arrived at 3 s; the log is read
    // every 2 s, so it is found at 4 s, 2 s after the check began.
    assert!(text.contains("after 2s: the desktop is up"), "{text}");
}

/// **A key with no visible effect fails its look, and every check after it is
/// not reached** — never quietly skipped, never guessed at.
#[test]
fn an_unchanged_screen_fails_and_stops_the_walk() {
    let mut machine = PicturesAndLines::showing(vec![Frame::filled(4, 4, [0, 0, 0])]);
    let walked = Walk::named("w")
        .press(Chord::key("ret"))
        .look("01-nothing", LOOK)
        .press(Chord::key("tab"))
        .look("02-later", LOOK)
        .expect_line("a line", "x", Since::Boot, Duration::from_secs(1));
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the screen 01-nothing".to_owned(), Verdict::Failed),
            ("the screen 02-later".to_owned(), Verdict::NotReached),
            ("a line".to_owned(), Verdict::NotReached),
        ]
    );
    assert_eq!(machine.keys_pressed(), vec!["ret"], "a key after a failure");
    assert!(!report.passed());
}

/// **A line never said fails at its time**, and one said before the mark does
/// not answer a check since the mark.
#[test]
fn a_line_not_said_since_the_mark_fails() {
    let mut machine = PicturesAndLines::showing(screens())
        .saying(Duration::ZERO, "Started alo-screen-reader.service");
    let walked = Walk::named("w")
        .wait(Duration::from_secs(1))
        .mark()
        .expect_line(
            "it started",
            "Started alo-screen-reader.service",
            Since::Mark,
            Duration::from_secs(6),
        );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("it started".to_owned(), Verdict::Failed)]
    );
    assert!(report.as_text().contains("was not said within 6s"));
}

/// A line that is found but does not also say what it must is a failure,
/// with the line as evidence — *8 arrived, 1 still owed* is not *all arrived*.
#[test]
fn a_line_that_does_not_also_say_what_it_must_fails() {
    let mut machine = PicturesAndLines::showing(screens()).saying(
        Duration::ZERO,
        "7 of this machine's applications arrived, 1 still owed",
    );
    let walked = Walk::named("w").expect_line_saying(
        "the applications arrived",
        "of this machine's applications arrived",
        ", 0 still owed",
        Since::Boot,
        Duration::from_secs(5),
    );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert!(!report.passed());
    assert!(report.as_text().contains("1 still owed"));
}

/// **A line that must not appear fails the check when it does**, and the
/// offending line is the evidence.
#[test]
fn a_forbidden_line_fails_its_watch() {
    let mut machine = PicturesAndLines::showing(screens()).saying(
        Duration::from_secs(4),
        "orca: AT-SPI: Called _atspi_dbus_return_accessible_from_message with strange signature so",
    );
    let quiet = Walk::named("w").expect_no_line(
        "no strange signature",
        &["strange signature", "Traceback"],
        Since::Boot,
        Duration::from_secs(10),
    );
    let report = walk(&mut machine, &quiet, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("no strange signature".to_owned(), Verdict::Failed)]
    );
    assert!(report.as_text().contains("1 lines in 10s"));

    let mut calm = PicturesAndLines::showing(screens());
    let report = walk(&mut calm, &quiet, &nothing_set, QUICK);
    assert!(report.passed(), "{}", report.as_text());
}

/// **Without the secret, the rest of the walk is skipped and says why** — and
/// nothing after the gate is pressed.
#[test]
fn without_its_secret_the_rest_is_skipped() {
    let mut machine = PicturesAndLines::showing(screens());
    let walked = Walk::named("w")
        .press(Chord::key("tab"))
        .look("01", LOOK)
        .only_with("ALO_WALK_PROVIDER_KEY")
        .type_secret("ALO_WALK_PROVIDER_KEY")
        .look("02-filled", LOOK);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the screen 01".to_owned(), Verdict::Passed),
            ("the screen 02-filled".to_owned(), Verdict::Skipped),
        ]
    );
    assert!(report.passed(), "{}", report.as_text());
    assert_eq!(machine.keys_pressed(), vec!["tab"]);
    assert!(report.as_text().contains("needs ALO_WALK_PROVIDER_KEY"));
}

/// A machine that says two lines: one a check inside a section looks for,
/// one a check after it looks for.
fn saying_inside_and_after() -> PicturesAndLines {
    PicturesAndLines::showing(screens())
        .saying(Duration::ZERO, "inside the section")
        .saying(Duration::ZERO, "after the section")
}

/// **Without its variable, a section is skipped and the walk goes on after
/// it** — what a section gated on a hand-run choice needs, so the sections
/// placed after it still run on every walk after a merge. Nothing inside the
/// section is pressed; the step after it is. Under the gate this replaced
/// (`Walk::only_with`), the check after it was skipped too.
#[test]
fn without_its_variable_only_the_section_is_skipped() {
    let mut machine = saying_inside_and_after();
    let walked = Walk::named("w")
        .section_only_with("ALO_WALK_EVERY_APPLICATION", |section| {
            section.press(Chord::key("esc")).expect_line(
                "inside",
                "inside the section",
                Since::Boot,
                Duration::from_secs(1),
            )
        })
        .press(Chord::key("tab"))
        .expect_line(
            "after",
            "after the section",
            Since::Boot,
            Duration::from_secs(1),
        );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("inside".to_owned(), Verdict::Skipped),
            ("after".to_owned(), Verdict::Passed),
        ]
    );
    assert!(report.passed(), "{}", report.as_text());
    assert_eq!(
        machine.keys_pressed(),
        vec!["tab"],
        "a key inside the section was pressed"
    );
    assert!(
        report
            .as_text()
            .contains("needs ALO_WALK_EVERY_APPLICATION"),
        "{}",
        report.as_text()
    );
}

/// **With its variable, the section is walked like any other**, and the
/// steps after it too.
#[test]
fn with_its_variable_the_section_is_walked() {
    let mut machine = saying_inside_and_after();
    let walked = Walk::named("w")
        .section_only_with("ALO_WALK_EVERY_APPLICATION", |section| {
            section.press(Chord::key("esc")).expect_line(
                "inside",
                "inside the section",
                Since::Boot,
                Duration::from_secs(1),
            )
        })
        .press(Chord::key("tab"))
        .expect_line(
            "after",
            "after the section",
            Since::Boot,
            Duration::from_secs(1),
        );
    let set = |name: &str| (name == "ALO_WALK_EVERY_APPLICATION").then(|| "yes".to_owned());
    let report = walk(&mut machine, &walked, &set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("inside".to_owned(), Verdict::Passed),
            ("after".to_owned(), Verdict::Passed),
        ]
    );
    assert_eq!(machine.keys_pressed(), vec!["esc", "tab"]);
}

/// **A secret typed is never said**: it reaches the machine as keys, and the
/// report names the variable and the length only.
#[test]
fn a_secret_is_typed_and_never_said() {
    let mut machine = PicturesAndLines::showing(screens());
    let walked = Walk::named("w")
        .only_with("THE_KEY")
        .type_secret("THE_KEY")
        .look("01", LOOK);
    let secret = |name: &str| (name == "THE_KEY").then(|| "sk-Zq9".to_owned());
    let report = walk(&mut machine, &walked, &secret, QUICK);
    assert_eq!(
        machine.keys_pressed(),
        vec!["s", "k", "minus", "shift", "z", "q", "9"]
    );
    let text = report.as_text();
    assert!(!text.contains("sk-Zq9"), "{text}");
    assert!(text.contains("typed THE_KEY, 6 characters"), "{text}");
    assert_eq!(walked.secrets(), vec!["THE_KEY"]);
}

/// A secret with a character the walk cannot type is refused by position,
/// and nothing of it is typed.
#[test]
fn an_untypable_secret_is_refused_by_position_only() {
    let mut machine = PicturesAndLines::showing(screens());
    let walked = Walk::named("w").type_secret("THE_KEY");
    let secret = |_: &str| Some("ab\u{e9}".to_owned());
    let report = walk(&mut machine, &walked, &secret, QUICK);
    assert!(machine.keys_pressed().is_empty());
    let text = report.as_text();
    assert!(text.contains("character 2 of THE_KEY has no key"), "{text}");
    assert!(!text.contains("ab"), "{text}");
}

/// A screen with a window on it — its ground dark, with `marked` pixels of
/// something written in it — and alo's teal edge round it, or not.
fn a_window(edged: bool, marked: usize) -> Frame {
    let mut frame = Frame::filled(320, 240, [250, 247, 242]);
    for y in 12..228 {
        for x in 16..304 {
            frame = frame.with_pixel(x, y, [24, 24, 27]);
        }
    }
    for x in 0..marked {
        frame = frame.with_pixel(40 + x % 200, 40 + x / 200, [200, 200, 200]);
    }
    if edged {
        let teal = crate::the_edge_seen::TEAL;
        for x in 30..290 {
            for y in [10, 11, 228, 229] {
                frame = frame.with_pixel(x, y, teal);
            }
        }
        for y in 26..214 {
            for x in [14, 15, 304, 305] {
                frame = frame.with_pixel(x, y, teal);
            }
        }
    }
    frame
}

/// The edge drawn, looked for for `seconds`.
fn the_edge_drawn(walk: Walk, seconds: u64) -> Walk {
    walk.the_edge(
        "the window wears the edge",
        "with-the-edge",
        true,
        Duration::from_secs(seconds),
    )
}

/// The edge gone, looked for for `seconds`.
fn the_edge_gone(walk: Walk, seconds: u64) -> Walk {
    walk.the_edge(
        "the edge is gone",
        "bare",
        false,
        Duration::from_secs(seconds),
    )
}

/// **The edge drawn is looked for until it comes, and kept**; one that never
/// comes fails at its time, keeps what the screen showed instead, says what
/// teal it did find, and stops the walk.
#[test]
fn the_edge_is_looked_for_until_it_comes_and_kept() {
    let mut machine = PicturesAndLines::showing(vec![
        a_window(false, 0),
        a_window(false, 0),
        a_window(true, 0),
    ]);
    let report = walk(
        &mut machine,
        &the_edge_drawn(Walk::named("w"), 10),
        &nothing_set,
        QUICK,
    );
    assert!(report.passed(), "{}", report.as_text());
    assert_eq!(machine.kept, vec!["with-the-edge".to_owned()]);

    let mut machine = PicturesAndLines::showing(vec![a_window(false, 0)]);
    let walked = the_edge_drawn(Walk::named("w"), 5).expect_line(
        "a line",
        "x",
        Since::Boot,
        Duration::from_secs(1),
    );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the window wears the edge".to_owned(), Verdict::Failed),
            ("a line".to_owned(), Verdict::NotReached),
        ]
    );
    assert_eq!(machine.kept, vec!["with-the-edge".to_owned()]);
    assert!(
        report.as_text().contains("no edge"),
        "the failure does not say what it found: {}",
        report.as_text()
    );
}

/// **The edge gone is looked for until it goes, with the window still there**;
/// one that stays fails.
#[test]
fn the_edge_gone_is_looked_for_until_it_goes() {
    let mut machine = PicturesAndLines::showing(vec![
        a_window(true, 0),
        a_window(true, 0),
        a_window(false, 300),
    ]);
    let walked = the_edge_gone(the_edge_drawn(Walk::named("w"), 5), 10);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert!(report.passed(), "{}", report.as_text());
    assert_eq!(
        machine.kept,
        vec!["with-the-edge".to_owned(), "bare".to_owned()]
    );

    let mut machine = PicturesAndLines::showing(vec![a_window(true, 0)]);
    let walked = the_edge_gone(the_edge_drawn(Walk::named("w"), 5), 5);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the window wears the edge".to_owned(), Verdict::Passed),
            ("the edge is gone".to_owned(), Verdict::Failed),
        ]
    );
}

/// **Gone is not the verdict of a screen with no window on it**: a check that
/// passed because the window itself went — a crash, a lock screen, a black
/// screen — would say nothing about the edge. So the place the edge was round
/// must still hold the window it was round, and *gone* needs an edge seen
/// before it.
#[test]
fn gone_needs_the_window_still_there_and_an_edge_seen_first() {
    let mut machine =
        PicturesAndLines::showing(vec![a_window(true, 0), Frame::filled(320, 240, [0, 0, 0])]);
    let walked = the_edge_gone(the_edge_drawn(Walk::named("w"), 5), 5);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the window wears the edge".to_owned(), Verdict::Passed),
            ("the edge is gone".to_owned(), Verdict::Failed),
        ],
        "a black screen read as the edge gone"
    );

    let mut machine = PicturesAndLines::showing(vec![a_window(false, 0)]);
    let report = walk(
        &mut machine,
        &the_edge_gone(Walk::named("w"), 5),
        &nothing_set,
        QUICK,
    );
    assert_eq!(
        verdicts(&report),
        vec![("the edge is gone".to_owned(), Verdict::Failed)],
        "gone was said of an edge never seen"
    );
}

/// A window whose application has drawn its content: many colours inside.
fn a_painted_window() -> Frame {
    let mut frame = a_window(false, 0);
    for y in 70..170 {
        for x in 70..250 {
            let shade = u8::try_from((x * 7 + y * 13) % 200).unwrap_or(0);
            frame = frame.with_pixel(x, y, [shade, 255 - shade, 90]);
        }
    }
    frame
}

/// **The edge seen round a window not yet painted, then gone once it is,
/// passes.** The walk of 2026-11-16 (run 43864271712) saw alo's edge round
/// VSCodium's window before VSCodium had drawn anything in it, and later saw
/// the edge gone with the editor drawn: the window had changed in every pixel
/// and was still there. A check that asked for the window's pixels to be as
/// they were failed it.
#[test]
fn the_edge_seen_round_an_unpainted_window_and_gone_once_it_is_painted_passes() {
    let mut machine = PicturesAndLines::showing(vec![a_window(true, 0), a_painted_window()]);
    let walked = the_edge_gone(the_edge_drawn(Walk::named("w"), 5), 5);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert!(report.passed(), "{}", report.as_text());
}
