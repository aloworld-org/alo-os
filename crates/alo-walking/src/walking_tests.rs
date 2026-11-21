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
    assert_eq!(
        machine.kept,
        vec![
            "with-the-edge-when-first-looked-for".to_owned(),
            "with-the-edge".to_owned()
        ],
        "the screen when the edge was first looked for was not kept"
    );

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
    assert_eq!(
        machine.kept,
        vec![
            "with-the-edge-when-first-looked-for".to_owned(),
            "with-the-edge".to_owned()
        ],
        "the screen when the edge was first looked for was not kept"
    );
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
        vec![
            "with-the-edge-when-first-looked-for".to_owned(),
            "with-the-edge".to_owned(),
            "bare".to_owned()
        ]
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

/// **A look that may find the screen as it was passes once it settles** — for
/// a key whose effect may already be on the screen, like bringing forward a
/// window already in front — and keeps its picture, so the report can still be
/// checked against it.
#[test]
fn a_look_that_may_find_the_screen_as_it_was_passes_once_settled() {
    let mut machine = PicturesAndLines::showing(vec![Frame::filled(4, 4, [0, 0, 0])]);
    let walked = Walk::named("w")
        .press(Chord::key("ret"))
        .look_as_it_settles("01-as-it-was", LOOK);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("the screen 01-as-it-was".to_owned(), Verdict::Passed)]
    );
    assert_eq!(machine.kept, vec!["01-as-it-was".to_owned()]);
    assert!(
        report.as_text().contains("settled after"),
        "{}",
        report.as_text()
    );
}

/// **But one that never settles fails**, so the look can still fail: a screen
/// that keeps changing for its whole time is not a terminal waiting for keys.
#[test]
fn a_look_that_may_find_the_screen_as_it_was_fails_if_it_never_settles() {
    let mut machine = PicturesAndLines::showing(
        (0..20_u8)
            .map(|shade| Frame::filled(4, 4, [shade, shade, shade]))
            .collect(),
    );
    let walked = Walk::named("w")
        .press(Chord::key("ret"))
        .look_as_it_settles("01-moving", LOOK)
        .expect_line("a line", "x", Since::Boot, Duration::from_secs(1));
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the screen 01-moving".to_owned(), Verdict::Failed),
            ("a line".to_owned(), Verdict::NotReached),
        ]
    );
}

/// Three pictures the walks kept, as the edge's checks would see them.
fn pictures(names: &[&str]) -> Vec<Frame> {
    names
        .iter()
        .map(|name| crate::the_edge_seen::tests::a_kept_picture(name))
        .collect()
}

/// **The edge is followed when its window moves, and gone is judged where it
/// went.** The walk of 2026-11-16 on `main` at 19202288 (run 43967583290)
/// first saw the edge round VSCodium before VSCodium had drawn, from (16, 16)
/// to (1151, 699) with the window's band shown (its picture 58); VSCodium then
/// drew from x −24 and y 70 to (1111, 699) — where the walk at a5f5ffb8 (run
/// 43933773704, its picture 58) saw the edge round it — and, the edge gone,
/// picture 60 has VSCodium there. A walk that looked for the window only where
/// the edge was first seen said it had gone too.
#[test]
fn the_edge_is_followed_where_its_window_goes() {
    let mut machine = PicturesAndLines::showing(pictures(&[
        "walk-19202288-58-vscodium-wearing-alo.png",
        "walk-a5f5ffb8-58-vscodium-wearing-alo.png",
        "walk-19202288-60-vscodium-after-alo.png",
    ]));
    let walked = the_edge_gone(the_edge_drawn(Walk::named("w"), 5), 30);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert!(report.passed(), "{}", report.as_text());
    assert!(
        report.as_text().contains("it went with its window"),
        "{}",
        report.as_text()
    );
}

/// **But a window gone from every place the edge was seen is not still
/// there**: the same edge, followed, and then the person's terminal in front
/// with VSCodium nowhere near (the walk at a5f5ffb8, picture 63).
#[test]
fn a_window_gone_from_every_place_the_edge_was_seen_fails() {
    let mut machine = PicturesAndLines::showing(pictures(&[
        "walk-19202288-58-vscodium-wearing-alo.png",
        "walk-a5f5ffb8-58-vscodium-wearing-alo.png",
        "walk-a5f5ffb8-63-the-terminal-and-no-vscodium.png",
    ]));
    let walked = the_edge_gone(the_edge_drawn(Walk::named("w"), 5), 5);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the window wears the edge".to_owned(), Verdict::Passed),
            ("the edge is gone".to_owned(), Verdict::Failed),
        ],
        "{}",
        report.as_text()
    );
}

/// Two independent sections after a shared start: the first fails its look,
/// and the second must still run.
fn two_sections(second_needs_the_first: bool) -> Walk {
    let walk = Walk::named("w")
        .press(Chord::key("tab"))
        .look("00-start", LOOK)
        .section("the first", &[], |walk| {
            walk.press(Chord::key("ret"))
                .look("01-nothing", LOOK)
                .expect_line("the first's line", "x", Since::Boot, Duration::from_secs(1))
        });
    let needs: &[&'static str] = if second_needs_the_first {
        &["the first"]
    } else {
        &[]
    };
    walk.section("the second", needs, |walk| {
        walk.expect_line(
            "the second's line",
            "the desktop is up",
            Since::Boot,
            Duration::from_secs(10),
        )
    })
}

/// A machine whose screen changes once, then never again: the shared start's
/// look passes and the first section's fails.
fn changes_once() -> PicturesAndLines {
    PicturesAndLines::showing(screens()).saying(Duration::from_secs(1), "the desktop is up")
}

/// **A failure in one section does not stop the next**: the first section's
/// later checks are not reached, the second section runs and passes, and the
/// walk still fails.
#[test]
fn a_failing_section_does_not_stop_the_next() {
    let mut machine = changes_once();
    let report = walk(&mut machine, &two_sections(false), &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the screen 00-start".to_owned(), Verdict::Passed),
            ("the screen 01-nothing".to_owned(), Verdict::Failed),
            ("the first's line".to_owned(), Verdict::NotReached),
            ("the second's line".to_owned(), Verdict::Passed),
        ],
        "{}",
        report.as_text()
    );
    assert!(!report.passed(), "a walk with a failed section passed");
    // **And the report gives each section its own result.**
    assert_eq!(
        report.sections(),
        vec![
            ("the first".to_owned(), Verdict::Failed),
            ("the second".to_owned(), Verdict::Passed),
        ]
    );
    let text = report.as_text();
    assert!(text.contains("section the first: FAIL"), "{text}");
    assert!(text.contains("section the second: PASS"), "{text}");
    assert!(
        report.as_markdown().contains("| FAIL | the first |"),
        "{}",
        report.as_markdown()
    );
}

/// **A section that needs an earlier one is not reached when that one
/// failed**, and says which.
#[test]
fn a_section_that_needs_a_failed_one_is_not_reached() {
    let mut machine = changes_once();
    let report = walk(&mut machine, &two_sections(true), &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report).last(),
        Some(&("the second's line".to_owned(), Verdict::NotReached)),
        "{}",
        report.as_text()
    );
    assert!(
        report.as_text().contains("needs the section the first"),
        "{}",
        report.as_text()
    );
}

/// **A failure before any section stops every section**: they all stand on
/// what came before them.
#[test]
fn a_failure_before_the_sections_stops_them_all() {
    let mut machine = PicturesAndLines::showing(vec![Frame::filled(4, 4, [0, 0, 0])])
        .saying(Duration::from_secs(1), "the desktop is up");
    let report = walk(&mut machine, &two_sections(false), &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![
            ("the screen 00-start".to_owned(), Verdict::Failed),
            ("the screen 01-nothing".to_owned(), Verdict::NotReached),
            ("the first's line".to_owned(), Verdict::NotReached),
            ("the second's line".to_owned(), Verdict::NotReached),
        ]
    );
}

/// A check on the line that says what alo did, which may pass after a retry.
fn typed_and_saved(retry_may_pass: bool) -> Walk {
    let walk = Walk::named("w").mark();
    if retry_may_pass {
        walk.expect_line_saying_or_after_a_retry(
            "alo types and saves",
            "alo in an application: accessible.type_text in org.gnome.TextEditor",
            "done, and saved",
            Since::Mark,
            Duration::from_secs(30),
        )
    } else {
        walk.expect_line_saying(
            "alo types and saves",
            "alo in an application: accessible.type_text in org.gnome.TextEditor",
            "done, and saved",
            Since::Mark,
            Duration::from_secs(30),
        )
    }
}

/// The two lines main's walk at 71932e07 (run 44161055698) saw: the first try
/// not done (827.96 s) and the second done and saved (832.15 s).
fn a_first_try_that_failed_then_one_that_held() -> PicturesAndLines {
    PicturesAndLines::showing(screens())
        .saying(
            Duration::from_secs(2),
            "alo-agentd: alo in an application: accessible.type_text in org.gnome.TextEditor, not done",
        )
        .saying(
            Duration::from_secs(6),
            "alo-agentd: alo in an application: accessible.type_text in org.gnome.TextEditor, done, and saved",
        )
}

/// **The outcome right after a failed first try passes, counted apart**, with
/// both lines as its evidence, and the walk passes.
#[test]
fn a_check_that_allows_a_retry_passes_after_one_and_says_so() {
    let mut machine = a_first_try_that_failed_then_one_that_held();
    let report = walk(&mut machine, &typed_and_saved(true), &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("alo types and saves".to_owned(), Verdict::PassedAfterARetry)],
        "{}",
        report.as_text()
    );
    assert!(report.passed(), "{}", report.as_text());
    let text = report.as_text();
    assert!(
        text.contains("not done") && text.contains("done, and saved"),
        "{text}"
    );
    assert!(text.contains("1 passed after a retry"), "{text}");
}

/// **But a failed try with nothing after it still fails**, once its time is up.
#[test]
fn a_check_that_allows_a_retry_still_fails_with_no_good_line() {
    let mut machine = PicturesAndLines::showing(screens()).saying(
        Duration::from_secs(2),
        "alo-agentd: alo in an application: accessible.type_text in org.gnome.TextEditor, not done",
    );
    let report = walk(&mut machine, &typed_and_saved(true), &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("alo types and saves".to_owned(), Verdict::Failed)],
        "{}",
        report.as_text()
    );
    assert!(!report.passed());
}

/// **And a check that does not allow a retry is judged on the first line**, as
/// every check was: the card checks depend on it.
#[test]
fn a_check_that_does_not_allow_a_retry_fails_on_the_first_line() {
    let mut machine = a_first_try_that_failed_then_one_that_held();
    let report = walk(&mut machine, &typed_and_saved(false), &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![("alo types and saves".to_owned(), Verdict::Failed)],
        "{}",
        report.as_text()
    );
}

/// A machine whose Settings says the keyboard moved to `names`, one every four
/// seconds from the third — later than one read of the log after a press, so
/// each press sees its own line.
fn settings_saying(names: &[&str]) -> PicturesAndLines {
    names.iter().enumerate().fold(
        PicturesAndLines::showing(screens()),
        |machine, (at, name)| {
            machine.saying(
                Duration::from_secs(3 + 4 * u64::try_from(at).unwrap_or(0)),
                &alo_acting::said_in_settings::the_keyboard_is_on(name),
            )
        },
    )
}

/// **A row is found by its name**: Down is pressed until Settings says the
/// keyboard is on *Add a provider*, and no more.
#[test]
fn a_row_is_found_by_its_name() {
    let mut machine = settings_saying(&["alo", alo_acting::said_in_settings::ADD_A_PROVIDER]);
    let walked = Walk::named("w").mark().press_until_the_keyboard_is_on(
        &Chord::key("down"),
        alo_acting::said_in_settings::ADD_A_PROVIDER,
        8,
    );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![(
            "the keyboard is on \"Add a provider\"".to_owned(),
            Verdict::Passed
        )],
        "{}",
        report.as_text()
    );
    assert_eq!(machine.keys_pressed(), vec!["down", "down"]);
}

/// **But a row never named fails**, after the presses it was given, saying
/// where the keyboard went instead.
#[test]
fn a_row_never_named_fails_saying_where_the_keyboard_went() {
    let mut machine = settings_saying(&["alo", "Not at all"]);
    let walked = Walk::named("w").mark().press_until_the_keyboard_is_on(
        &Chord::key("down"),
        alo_acting::said_in_settings::ADD_A_PROVIDER,
        3,
    );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![(
            "the keyboard is on \"Add a provider\"".to_owned(),
            Verdict::Failed
        )],
        "{}",
        report.as_text()
    );
    assert!(
        report.as_text().contains("Not at all"),
        "{}",
        report.as_text()
    );
    assert_eq!(machine.keys_pressed().len(), 3);
}

/// A machine the size of a walk's screen, saying `lines` one every four
/// seconds from the third.
fn a_screen_saying(lines: &[String]) -> PicturesAndLines {
    lines.iter().enumerate().fold(
        PicturesAndLines::showing(vec![
            Frame::filled(1280, 800, [0, 0, 0]),
            Frame::filled(1280, 800, [9, 9, 9]),
        ]),
        |machine, (at, line)| {
            machine.saying(
                Duration::from_secs(3 + 4 * u64::try_from(at).unwrap_or(0)),
                line,
            )
        },
    )
}

/// A machine the size of a walk's screen, saying every one of `lines` at the
/// third second, together.
fn a_screen_saying_at_once(lines: &[String]) -> PicturesAndLines {
    lines.iter().fold(
        PicturesAndLines::showing(vec![Frame::filled(1280, 800, [0, 0, 0])]),
        |machine, line| machine.saying(Duration::from_secs(3), line),
    )
}

/// The terminal's band, and its three controls.
fn the_terminals_band() -> (alo_acting::said_of_a_band::OnTheScreen, Vec<String>) {
    use alo_acting::said_of_a_band::{self as band, OnTheScreen, TheKind};
    let at = |x: i32| OnTheScreen {
        x,
        y: 51,
        w: 44,
        h: 44,
    };
    let drawn = OnTheScreen {
        x: 100,
        y: 46,
        w: 640,
        h: 54,
    };
    (
        drawn,
        vec![
            band::a_band_is_drawn(TheKind::Terminal, "terminal", drawn),
            band::a_bands_controls_are_drawn(
                TheKind::Terminal,
                "terminal",
                &[
                    (band::WINDOW_ACTIONS, at(580)),
                    (band::MINIMISE, at(628)),
                    (band::CLOSE, at(676)),
                ],
            ),
        ],
    )
}

/// **A band is found where the shell says it is drawn, and pointed at in its
/// middle; its controls are read, and a control is pressed in its middle.**
#[test]
fn a_band_is_found_and_its_control_pressed_where_they_are_drawn() {
    use alo_acting::said_of_a_band::{self as band, TheKind};
    let (_, lines) = the_terminals_band();
    let mut machine = a_screen_saying(&lines);
    let walked = Walk::named("w")
        .mark()
        .point_at_the_band(TheKind::Terminal, "terminal", Duration::from_secs(30))
        .the_controls_are(
            TheKind::Terminal,
            "terminal",
            &[band::WINDOW_ACTIONS, band::MINIMISE, band::CLOSE],
            Duration::from_secs(30),
        )
        .click_the_control(band::CLOSE);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert!(report.passed(), "{}", report.as_text());
    assert_eq!(
        machine.pointed,
        vec![(420, 73), (698, 73)],
        "the band's middle, then Close's"
    );
    assert_eq!(machine.buttons, vec![true, false]);
}

/// **A band nothing says is drawn fails**, and so does one said and then no
/// longer drawn, and one drawn off the screen: three ways the band is not on
/// the screen at all, which is what Text Editor's was.
#[test]
fn a_band_not_on_the_screen_fails() {
    use alo_acting::said_of_a_band::{self as band, OnTheScreen, TheKind};
    let (drawn, lines) = the_terminals_band();
    let off = OnTheScreen { x: 2000, ..drawn };
    for said in [
        Vec::new(),
        vec![
            lines.first().cloned().unwrap_or_default(),
            band::a_band_is_no_longer_drawn(TheKind::Terminal, "terminal"),
        ],
        vec![band::a_band_is_drawn(TheKind::Terminal, "terminal", off)],
    ] {
        let mut machine = a_screen_saying_at_once(&said);
        let walked = Walk::named("w").mark().point_at_the_band(
            TheKind::Terminal,
            "terminal",
            Duration::from_secs(20),
        );
        let report = walk(&mut machine, &walked, &nothing_set, QUICK);
        assert_eq!(
            verdicts(&report),
            vec![(
                "the band of terminal \"terminal\" is on the screen".to_owned(),
                Verdict::Failed
            )],
            "{said:?}\n{}",
            report.as_text()
        );
        assert!(machine.pointed.is_empty(), "pointed at a band not there");
    }
}

/// **The controls must be the ones expected, every one**: Minimise missing
/// fails, and a control not drawn cannot be pressed.
#[test]
fn a_control_missing_fails_and_cannot_be_pressed() {
    use alo_acting::said_of_a_band::{self as band, OnTheScreen, TheKind};
    let (drawn, _) = the_terminals_band();
    let at = OnTheScreen {
        x: 580,
        y: 51,
        w: 44,
        h: 44,
    };
    let lines = vec![
        band::a_band_is_drawn(TheKind::Terminal, "terminal", drawn),
        band::a_bands_controls_are_drawn(
            TheKind::Terminal,
            "terminal",
            &[(band::WINDOW_ACTIONS, at)],
        ),
    ];
    let mut machine = a_screen_saying(&lines);
    let walked = Walk::named("w")
        .mark()
        .point_at_the_band(TheKind::Terminal, "terminal", Duration::from_secs(30))
        .the_controls_are(
            TheKind::Terminal,
            "terminal",
            &[band::WINDOW_ACTIONS, band::MINIMISE, band::CLOSE],
            Duration::from_secs(30),
        );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report).last().map(|(_, verdict)| *verdict),
        Some(Verdict::Failed),
        "{}",
        report.as_text()
    );

    let mut machine = a_screen_saying(&lines);
    let walked = Walk::named("w")
        .mark()
        .point_at_the_band(TheKind::Terminal, "terminal", Duration::from_secs(30))
        .click_the_control(band::MINIMISE);
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report).last().map(|(_, verdict)| *verdict),
        Some(Verdict::Failed),
        "{}",
        report.as_text()
    );
    assert!(
        machine.buttons.is_empty(),
        "a control not drawn was pressed"
    );
}

/// **A task window is found by its kind, whatever its number**: the walk
/// cannot know which task the shell numbers it, so `"task"` answers to any
/// `"task <n>"`.
#[test]
fn a_task_window_is_found_whatever_its_number() {
    use alo_acting::said_of_a_band::{self as band, TheKind};
    let (drawn, _) = the_terminals_band();
    let lines = vec![band::a_band_is_drawn(TheKind::Task, "task 3", drawn)];
    let mut machine = a_screen_saying(&lines);
    let walked =
        Walk::named("w")
            .mark()
            .point_at_the_band(TheKind::Task, "task", Duration::from_secs(30));
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert!(report.passed(), "{}", report.as_text());
}

/// A machine whose market says its keyboard moved to `identifiers`, and whose
/// Settings says the last of them too — one every four seconds from the third.
fn the_market_saying(identifiers: &[&str]) -> PicturesAndLines {
    identifiers.iter().enumerate().fold(
        PicturesAndLines::showing(screens()),
        |machine, (at, identifier)| {
            machine.saying(
                Duration::from_secs(3 + 4 * u64::try_from(at).unwrap_or(0)),
                &alo_acting::said_in_the_market::in_the_market_the_keyboard_is_on(identifier),
            )
        },
    )
}

/// **A card is found by the phrase its line begins with**, not only a
/// Settings row: Down is pressed until the market says Kate, and no more.
#[test]
fn a_card_in_the_market_is_found_by_its_name() {
    let mut machine = the_market_saying(&["org.gimp.GIMP", "org.kde.kate"]);
    let walked = Walk::named("w").mark().press_until_it_says(
        &Chord::key("down"),
        alo_acting::said_in_the_market::IN_THE_MARKET_THE_KEYBOARD_IS_ON,
        "org.kde.kate",
        8,
    );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![(
            "the keyboard is on \"org.kde.kate\"".to_owned(),
            Verdict::Passed
        )],
        "{}",
        report.as_text()
    );
    assert_eq!(machine.keys_pressed(), vec!["down", "down"]);
}

/// **A line with another phrase does not answer**, even naming the same
/// thing: Settings saying *Applications* is not the Dock saying it.
#[test]
fn a_line_with_another_phrase_does_not_answer() {
    let mut machine = settings_saying(&["Applications", "Applications"]);
    let walked = Walk::named("w").mark().press_until_it_says(
        &Chord::key("tab"),
        alo_acting::said_in_the_market::ON_THE_DOCK_THE_KEYBOARD_IS_ON,
        "Applications",
        2,
    );
    let report = walk(&mut machine, &walked, &nothing_set, QUICK);
    assert_eq!(
        verdicts(&report),
        vec![(
            "the keyboard is on \"Applications\"".to_owned(),
            Verdict::Failed
        )],
        "{}",
        report.as_text()
    );
}
