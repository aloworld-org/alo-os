//! A try passes only when the model takes every step and then answers; the
//! first step off the road ends it there, and nothing after it is asked.

use alo_adapters::fallback_verbs::{APPLICATION, NAME, READ_WINDOW, TEXT, TYPE_TEXT};
use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_applications::opening_a_file::PATH;
use alo_instructing::{Instructions, WhatItMayReach, the_products_words};

use super::{Went, one_try};
use crate::a_road::{ARequest, Answered, Step};
use crate::testing::{Down, Writing, answering, proposing, reading, the_verbs, the_words};
use crate::went_wrong::WentWrong;

const EDITOR: &str = "com.vscodium.codium";
const FILE: &str = "/var/home/alo/hello.txt";
const THE_TEXT: &str = "Written by alo";

fn writing_in_the_editor() -> ARequest {
    ARequest::asking(
        "write in the editor",
        "Open hello.txt in VSCodium and write Written by alo in it",
        vec![
            Step::of(OPEN_FILE_IN_APPLICATION, Answered::AFileOpened)
                .expecting(APPLICATION, EDITOR)
                .expecting(PATH, FILE),
            Step::of(
                READ_WINDOW,
                Answered::AWindowShowing {
                    title: "hello.txt - VSCodium".to_owned(),
                    field: "hello.txt".to_owned(),
                },
            )
            .expecting(APPLICATION, EDITOR),
            Step::of(TYPE_TEXT, Answered::TypedAndSaved)
                .expecting(APPLICATION, EDITOR)
                .expecting(NAME, "hello.txt")
                .expecting(TEXT, THE_TEXT),
        ],
    )
}

fn opening() -> String {
    proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, EDITOR), (PATH, FILE)],
    )
}

fn looking() -> String {
    reading(READ_WINDOW, &[(APPLICATION, EDITOR)])
}

fn typing() -> String {
    proposing(
        TYPE_TEXT,
        &[(APPLICATION, EDITOR), (NAME, "hello.txt"), (TEXT, THE_TEXT)],
    )
}

fn tried(lines: &[String]) -> (super::Tried, Writing) {
    let mut model = Writing::these(lines);
    let tried = one_try(
        &writing_in_the_editor(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    (tried, model)
}

#[test]
fn every_step_and_then_an_answer_is_the_road() {
    let (tried, _) = tried(&[opening(), looking(), typing(), answering("It is written.")]);
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    assert_eq!(
        tried.chose(),
        [
            OPEN_FILE_IN_APPLICATION,
            READ_WINDOW,
            TYPE_TEXT,
            "an answer in words"
        ]
    );
    assert_eq!(tried.asked(), 4);
    assert_eq!(tried.looked_aside(), 0, "no read was taken aside");
}

/// **The refusal case**: a model that types before it reads the window has
/// left the road at step 2, the try says so, and the model is not asked again.
#[test]
fn a_step_off_the_road_fails_the_try_there_and_asks_nothing_more() {
    let (tried, model) = tried(&[opening(), typing(), answering("Done.")]);
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::AnotherVerb { step: 2, expected, asked, given })
                if expected == READ_WINDOW && asked == TYPE_TEXT && given.contains("text = ")
        ),
        "{tried:?}"
    );
    assert!(!tried.took_the_road());
    assert_eq!(tried.chose(), [OPEN_FILE_IN_APPLICATION, TYPE_TEXT]);
    assert_eq!(
        tried.said_last(),
        Some(typing().as_str()),
        "the line that left the road is kept"
    );
    assert_eq!(
        model.shown.len(),
        2,
        "nothing is asked after the road is left"
    );
}

#[test]
fn an_answer_before_the_road_is_finished_is_stopping_short() {
    let (tried, _) = tried(&[opening(), answering("I opened it.")]);
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::StoppedShort { done: 1, of: 3, .. })
        ),
        "{tried:?}"
    );
}

#[test]
fn asking_for_more_after_the_whole_road_is_going_on() {
    let (tried, _) = tried(&[opening(), looking(), typing(), looking()]);
    assert_eq!(
        tried.went(),
        &Went::WentWrong(WentWrong::WentOn {
            asked: READ_WINDOW.to_owned()
        }),
        "{tried:?}"
    );
}

#[test]
fn a_line_that_is_never_a_request_runs_out_of_rounds() {
    let broken: Vec<String> = (0..20)
        .map(|_| r#"{"format":1,"asks":{}}"#.to_owned())
        .collect();
    let (tried, _) = tried(&broken);
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::RanOutOfRounds { done: 0, of: 3 })
        ),
        "{tried:?}"
    );
}

#[test]
fn a_model_that_cannot_be_asked_is_not_a_model_that_chose_wrongly() {
    let tried = one_try(
        &writing_in_the_editor(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut Down,
    );
    assert!(matches!(tried.went(), Went::NotAnswered(_)), "{tried:?}");
}

/// **The model is shown the real turn's text**: it begins with the product's
/// own words for this registry, which is what makes the daemon pass it on
/// unwrapped, and the request is in it; the second round carries what the
/// machine answered to the first.
#[test]
fn the_model_is_shown_the_text_a_turn_shows() {
    let (_, model) = tried(&[opening(), looking(), typing(), answering("Done.")]);
    // **What the turn offers**: no read here finds a window that describes
    // nothing, so the screen road is not in it.
    let products = the_products_words(
        Instructions::SHOWN_TO_A_TURN,
        &alo_instructing::the_verbs_offered(&the_verbs(), false),
    );
    for shown in &model.shown {
        assert!(shown.starts_with(&products));
        assert_eq!(shown, shown.trim(), "shown as the daemon shows it: trimmed");
        assert!(shown.contains("Open hello.txt in VSCodium and write Written by alo in it"));
    }
    assert!(
        model.shown.get(1).is_some_and(
            |second| second.contains(EDITOR) && second.contains(OPEN_FILE_IN_APPLICATION)
        ),
        "{:?}",
        model.shown.get(1)
    );
    assert!(
        model
            .shown
            .get(2)
            .is_some_and(|third| third.contains("hello.txt - VSCodium")),
        "the third round is shown the window the second read"
    );
}

/// **Law 8, on the check itself**: a road that expects the wrong thing fails
/// a model that did exactly what the request asked. Were this green, the
/// check could not tell a right road from a wrong one.
#[test]
fn a_road_that_expects_the_wrong_application_fails_a_model_that_did_right() {
    let wrong = ARequest::asking(
        "write in the wrong editor",
        "Open hello.txt in VSCodium and write Written by alo in it",
        vec![
            Step::of(OPEN_FILE_IN_APPLICATION, Answered::AFileOpened)
                .expecting(APPLICATION, "org.gnome.TextEditor")
                .expecting(PATH, FILE),
        ],
    );
    let mut model = Writing::these(&[opening(), answering("Opened.")]);
    let tried = one_try(
        &wrong,
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::AnotherValue { step: 1, .. })
        ),
        "{tried:?}"
    );
}

/// The editor road with its read made **a look**: a step the model may skip
/// when it already knows, or take more than once.
fn writing_with_a_look() -> ARequest {
    let road = writing_in_the_editor()
        .road()
        .iter()
        .map(|step| {
            if step.verb() == READ_WINDOW {
                step.clone().a_look()
            } else {
                step.clone()
            }
        })
        .collect();
    ARequest::asking("write with a look", "the same words", road)
}

fn tried_on(request: &ARequest, lines: &[String]) -> super::Tried {
    let mut model = Writing::these(lines);
    one_try(
        request,
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    )
}

/// **A look may be skipped**: a model that knew what it needed and went
/// straight on took the road.
#[test]
fn a_look_may_be_skipped() {
    let tried = tried_on(
        &writing_with_a_look(),
        &[opening(), typing(), answering("It is written.")],
    );
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    // And the skip is counted, so a summary can say how often it happened.
    assert_eq!(tried.skipped_looks(), 1, "{tried:?}");
    let looked = tried_on(
        &writing_with_a_look(),
        &[opening(), looking(), typing(), answering("It is written.")],
    );
    assert_eq!(looked.skipped_looks(), 0, "{looked:?}");
}

/// **A look may be taken twice**, and the road goes on after it.
#[test]
fn a_look_may_be_taken_again() {
    let tried = tried_on(
        &writing_with_a_look(),
        &[
            opening(),
            looking(),
            looking(),
            typing(),
            answering("It is written."),
        ],
    );
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    assert_eq!(
        tried.chose(),
        [
            OPEN_FILE_IN_APPLICATION,
            READ_WINDOW,
            READ_WINDOW,
            TYPE_TEXT,
            "an answer in words"
        ]
    );
}

/// **The refusal case**: skipping a look does not excuse the step after it —
/// a wrong verb there still leaves the road, at that step.
#[test]
fn skipping_a_look_still_holds_the_next_step() {
    let tried = tried_on(
        &writing_with_a_look(),
        &[opening(), opening(), answering("Opened.")],
    );
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::AnotherVerb { step: 3, expected, asked, given })
                if expected == TYPE_TEXT
                    && asked == OPEN_FILE_IN_APPLICATION
                    && given.contains(EDITOR)
        ),
        "{tried:?}"
    );
}

/// **A file named in the wrong folder is answered as the machine answers it**:
/// refused, with where one of that name is in the home, and a model that then
/// names it there takes the road (the owner's small-by-default answer).
#[test]
fn a_file_named_in_the_wrong_folder_is_told_where_it_is_and_may_be_named_again() {
    let wrong = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[
            (APPLICATION, EDITOR),
            (PATH, "/var/home/alo/Documents/hello.txt"),
        ],
    );
    let (tried, model) = tried(&[wrong, opening(), looking(), typing(), answering("Done.")]);
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    let told = model.shown.get(1).cloned().unwrap_or_default();
    assert!(
        told.contains("but one of that name is at /var/home/alo/hello.txt"),
        "{told}"
    );
}

/// **The refusal cases**: a model told where the file is that never names it
/// there does not take the road, and a file of another name is never answered
/// this way.
#[test]
fn a_file_is_told_where_it_is_only_for_its_own_name_and_must_be_named_there() {
    let wrong = || {
        proposing(
            OPEN_FILE_IN_APPLICATION,
            &[
                (APPLICATION, EDITOR),
                (PATH, "/var/home/alo/Documents/hello.txt"),
            ],
        )
    };
    let (twice, _) = tried(&[wrong(), wrong(), answering("Done.")]);
    assert!(!twice.took_the_road(), "{twice:?}");
    let another = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, EDITOR), (PATH, "/var/home/alo/notes.txt")],
    );
    let (other, _) = tried(&[another, answering("Done.")]);
    assert!(
        matches!(
            other.went(),
            Went::WentWrong(WentWrong::AnotherValue { step: 1, .. })
        ),
        "{other:?}"
    );
}

/// The editor road, and a read of the file it opens that the machine would
/// answer at any point: the file is there, and empty, as the walk makes it.
fn writing_or_reading_the_file_first() -> ARequest {
    writing_in_the_editor().or_looking_at(
        Step::of(
            alo_files::verbs::READ_FILE,
            Answered::With(alo_protocol::Done::Read {
                text: String::new(),
            }),
        )
        .expecting(alo_files::verbs::FILE, FILE),
    )
}

fn reading_the_file() -> String {
    reading(
        alo_files::verbs::READ_FILE,
        &[(alo_files::verbs::FILE, FILE)],
    )
}

fn tried_on_the_road(request: &ARequest, lines: &[String]) -> super::Tried {
    let mut model = Writing::these(lines);
    one_try(
        request,
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    )
}

/// **A read the machine would answer is a look aside**, and the road goes on
/// after it: the try still has to open the file, read the window, type and
/// save — and it says it took a look, so a pass that needed one is seen.
#[test]
fn a_read_aside_is_answered_and_the_whole_road_must_still_be_taken() {
    let tried = tried_on_the_road(
        &writing_or_reading_the_file_first(),
        &[
            reading_the_file(),
            opening(),
            looking(),
            typing(),
            answering("It is written."),
        ],
    );
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    assert_eq!(tried.looked_aside(), 1);
    // Looking aside and then stopping is not the road.
    let stopped = tried_on_the_road(
        &writing_or_reading_the_file_first(),
        &[reading_the_file(), answering("It is empty.")],
    );
    assert!(!stopped.took_the_road(), "{stopped:?}");
}

/// **The refusal cases**: looks aside are capped, and a read the road does
/// not declare is still off the road.
#[test]
fn looks_aside_are_capped_and_an_undeclared_read_is_off_the_road() {
    let mut lines: Vec<String> = (0..=super::THE_MOST_LOOKS_ASIDE)
        .map(|_| reading_the_file())
        .collect();
    lines.extend([opening(), looking(), typing(), answering("Done.")]);
    let capped = tried_on_the_road(&writing_or_reading_the_file_first(), &lines);
    assert!(!capped.took_the_road(), "{capped:?}");
    let undeclared = tried_on_the_road(
        &writing_in_the_editor(),
        &[
            reading_the_file(),
            opening(),
            looking(),
            typing(),
            answering("Done."),
        ],
    );
    assert!(!undeclared.took_the_road(), "{undeclared:?}");
}

/// **At *ask each time* the road is the same, one approval per change**: each
/// change waits, is approved, and the model is shown what the daemon tells the
/// person it did — the step the agent's turn goes on from
/// (`docs/contracts/asking-socket.md`, *The turn goes on after an answer*).
#[test]
fn asking_each_time_takes_the_same_road_one_approval_per_change() {
    let mut model = Writing::these(&[opening(), looking(), typing(), answering("It is written.")]);
    let tried = super::one_try_at(
        super::TheLevel::AskEachTime,
        &writing_in_the_editor(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    assert_eq!(tried.asked(), 4, "the model was not asked once per step");
    // **What the model was shown after an approved change is full access's
    // short shape, and true**: *Done: …*, under `done`, and nothing at *ask each
    // time* says it was done without asking (`alo_agent::shown_after_an_answer`).
    let last = model.shown.last().cloned().unwrap_or_default();
    assert!(
        last.contains("Done: "),
        "no approved change was shown: {last}"
    );
    assert!(
        !last.contains("without asking") && !last.contains("done-without-asking"),
        "the model was told something untrue at ask each time: {last}"
    );
}

/// The set's installing request: GIMP, by the identifier the catalogue gives.
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on a set that does not build is the failure being reported"
)]
fn installing_gimp() -> ARequest {
    crate::the_set()
        .expect("the set builds")
        .only("install an application")
        .expect("it is in the set")
        .requests()
        .first()
        .cloned()
        .expect("it is in the set")
}

/// An installation of `application` from Flathub, where the catalogue's GIMP
/// comes from.
fn installing(application: &str) -> String {
    proposing(
        alo_software::INSTALL_APPLICATION,
        &[
            (alo_software::words::APPLICATION, application),
            (alo_software::words::SOURCE, "flathub"),
        ],
    )
}

/// **A guessed identifier is answered as the machine answers it**: refused
/// before the person is asked, naming the catalogue's own, and a model that
/// then asks for that one takes the road — counted, so a pass that needed the
/// refusal is seen.
#[test]
fn a_guessed_identifier_is_refused_naming_the_catalogues_and_may_be_asked_again() {
    for level in [super::TheLevel::FullAccess, super::TheLevel::AskEachTime] {
        let mut model = Writing::these(&[
            installing("org.gnome.GIMP"),
            installing("org.gimp.GIMP"),
            answering("GIMP is installed."),
        ]);
        let tried = super::one_try_at(
            level,
            &installing_gimp(),
            &WhatItMayReach::nothing().at_full_access(true),
            &the_verbs(),
            &the_words(),
            &mut model,
        );
        assert_eq!(tried.went(), &Went::TheRoad, "{level:?}: {tried:?}");
        assert_eq!(tried.refused_before_asking(), 1, "{level:?}: {tried:?}");
        let told = model.shown.get(1).cloned().unwrap_or_default();
        assert!(
            told.contains("org.gnome.GIMP") && told.contains("org.gimp.GIMP"),
            "{level:?}: the refusal did not name both: {told}"
        );
    }
}

/// **The refusal cases**: a model that asks for the guess again is refused
/// again and has not taken the road, and a pass with no refusal counts none.
#[test]
fn a_guess_asked_for_again_never_takes_the_road() {
    let mut model = Writing::these(&[
        installing("org.gnome.GIMP"),
        installing("org.gnome.GIMP"),
        answering("GIMP is installed."),
    ]);
    let twice = super::one_try(
        &installing_gimp(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    assert!(!twice.took_the_road(), "{twice:?}");
    let mut model = Writing::these(&[installing("org.gimp.GIMP"), answering("Installed.")]);
    let straight = super::one_try(
        &installing_gimp(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    assert_eq!(straight.went(), &Went::TheRoad, "{straight:?}");
    assert_eq!(straight.refused_before_asking(), 0);
}

/// The window the other-road requests read.
fn the_editors_window() -> Step {
    Step::of(
        READ_WINDOW,
        Answered::AWindowShowing {
            title: "hello.txt - VSCodium".to_owned(),
            field: "hello.txt".to_owned(),
        },
    )
    .expecting(APPLICATION, EDITOR)
}

/// A request whose road reads the window and types, and whose **other road**
/// reads it, opens the file, and reads it again.
fn with_another_road() -> ARequest {
    ARequest::asking(
        "with another road",
        "the same words",
        vec![
            the_editors_window().a_look(),
            Step::of(TYPE_TEXT, Answered::TypedAndSaved)
                .expecting(APPLICATION, EDITOR)
                .expecting(NAME, "hello.txt")
                .expecting(TEXT, THE_TEXT),
        ],
    )
    .or_by_another_road(vec![
        the_editors_window().a_look(),
        Step::of(OPEN_FILE_IN_APPLICATION, Answered::AFileOpened)
            .expecting(APPLICATION, EDITOR)
            .expecting(PATH, FILE),
        the_editors_window(),
    ])
}

/// **The other road is a pass, counted apart**; the road is a pass that did
/// not take it.
#[test]
fn a_try_by_the_other_road_takes_the_road_and_says_which() {
    let other = tried_on(
        &with_another_road(),
        &[looking(), opening(), looking(), answering("Opened.")],
    );
    assert!(other.took_the_road(), "{other:?}");
    assert!(other.by_the_other_road(), "{other:?}");
    let road = tried_on(
        &with_another_road(),
        &[looking(), typing(), answering("Typed.")],
    );
    assert!(road.took_the_road(), "{road:?}");
    assert!(!road.by_the_other_road(), "{road:?}");
}

/// **The other road is held to its end**: a try that parts for it and answers
/// before its last read stopped short of it.
#[test]
fn a_try_by_the_other_road_must_take_it_to_its_end() {
    let short = tried_on(
        &with_another_road(),
        &[looking(), opening(), answering("Opened.")],
    );
    assert!(
        matches!(
            short.went(),
            Went::WentWrong(WentWrong::StoppedShort { done: 2, of: 3, .. })
        ),
        "{short:?}"
    );
}

/// **Without another road, the same steps leave the road** where they part.
#[test]
fn without_another_road_the_other_way_leaves_the_road() {
    let request = ARequest::asking(
        "with no other road",
        "the same words",
        with_another_road().road().to_vec(),
    );
    let tried = tried_on(
        &request,
        &[looking(), opening(), looking(), answering("Opened.")],
    );
    assert!(!tried.took_the_road(), "{tried:?}");
    assert!(!tried.by_the_other_road(), "{tried:?}");
}

/// The set's request to scroll down until a heading shows.
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on a set that does not build is the failure being reported"
)]
fn scrolling_to_the_heading() -> ARequest {
    crate::the_set()
        .expect("the set builds")
        .only("scroll to a named thing in an application")
        .expect("it is in the set")
        .requests()
        .first()
        .cloned()
        .expect("it is in the set")
}

const BROWSER: &str = "org.mozilla.firefox";

fn reading_the_browser() -> String {
    reading(READ_WINDOW, &[(APPLICATION, BROWSER)])
}

fn a_page_down() -> String {
    proposing(
        alo_adapters::fallback_verbs::SCROLL_PAGE,
        &[
            (APPLICATION, BROWSER),
            (alo_adapters::fallback_verbs::TOWARD, "down"),
        ],
    )
}

fn scrolling_to(to: &str) -> String {
    proposing(
        alo_adapters::fallback_verbs::SCROLL_TO,
        &[
            (APPLICATION, BROWSER),
            (alo_adapters::fallback_verbs::TO, to),
        ],
    )
}

/// `lines` tried on the scrolling request at `level`.
fn scrolled(level: super::TheLevel, lines: &[String]) -> super::Tried {
    let mut model = Writing::these(lines);
    super::one_try_at(
        level,
        &scrolling_to_the_heading(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    )
}

/// **Pages down, as often as the model asks, and then the heading reached —
/// read on the screen or scrolled to by name — is another road**, at both
/// levels, one card a page at ask each time; one scroll to it is the road.
#[test]
fn pages_down_until_the_heading_shows_take_another_road_at_both_levels() {
    let heading = crate::every_other_verb::FURTHER_DOWN;
    for level in [super::TheLevel::FullAccess, super::TheLevel::AskEachTime] {
        let by_name = [
            reading_the_browser(),
            a_page_down(),
            a_page_down(),
            scrolling_to(heading),
            answering("It is on the screen."),
        ];
        let tried = scrolled(level, &by_name);
        assert_eq!(tried.went(), &Went::TheRoad, "{level:?}: {tried:?}");
        assert!(tried.by_the_other_road(), "{level:?}: {tried:?}");
        assert_eq!(tried.asked(), by_name.len(), "{level:?}: {tried:?}");
        let read = [
            a_page_down(),
            a_page_down(),
            a_page_down(),
            reading_the_browser(),
            answering("It is on the screen."),
        ];
        let tried = scrolled(level, &read);
        assert_eq!(tried.went(), &Went::TheRoad, "{level:?}: {tried:?}");
        assert!(tried.by_the_other_road(), "{level:?}: {tried:?}");
        let road = scrolled(
            level,
            &[scrolling_to(heading), answering("It is on the screen.")],
        );
        assert_eq!(road.went(), &Went::TheRoad, "{level:?}: {road:?}");
        assert!(!road.by_the_other_road(), "{level:?}: {road:?}");
    }
}

/// **The refusal cases**: pages down that never reach the heading stopped
/// short, and a scroll to another name leaves every road.
#[test]
fn pages_down_that_never_reach_the_heading_are_not_a_pass() {
    for level in [super::TheLevel::FullAccess, super::TheLevel::AskEachTime] {
        let short = scrolled(
            level,
            &[a_page_down(), a_page_down(), answering("I scrolled down.")],
        );
        assert!(
            matches!(
                short.went(),
                Went::WentWrong(WentWrong::StoppedShort { done: 2, of: 3, .. })
            ),
            "{level:?}: {short:?}"
        );
        let elsewhere = scrolled(
            level,
            &[
                a_page_down(),
                scrolling_to("Somewhere else"),
                answering("Done."),
            ],
        );
        assert!(!elsewhere.took_the_road(), "{level:?}: {elsewhere:?}");
    }
}

/// **On a request's own road, a read declared aside is still answered aside**
/// even where the road's next step is a look at the same verb, as before other
/// roads existed: only the other road prefers its own next step.
#[test]
fn on_its_own_road_a_read_declared_aside_is_answered_aside() {
    let request = writing_with_a_look().or_looking_at(the_editors_window());
    let tried = tried_on(
        &request,
        &[opening(), looking(), typing(), answering("Typed.")],
    );
    assert!(tried.took_the_road(), "{tried:?}");
    assert_eq!(tried.looked_aside(), 1, "{tried:?}");
    assert!(!tried.by_the_other_road(), "{tried:?}");
}

/// **The seam: the agent's own loop and every-try's model are offered the same
/// verbs at every step**, both built by `alo_instructing::the_verbs_offered` —
/// the picture verb neither told to the model nor in what its answer is held to
/// before the read, and in both after a read of a window that describes nothing.
#[test]
fn the_agents_loop_and_the_check_offer_the_screen_road_alike() {
    let set = crate::the_set();
    assert!(set.is_ok(), "the set does not build");
    let Ok(set) = set else { return };
    let picturing = set
        .clone()
        .only("have a picture of a window that describes nothing read");
    assert!(picturing.is_some(), "the set has no picture request");
    let Some(request) = picturing.as_ref().and_then(|one| one.requests().first()) else {
        return;
    };
    let viewer = "org.gnome.Loupe";
    let mut model = Writing::these(&[
        reading(READ_WINDOW, &[(APPLICATION, viewer)]),
        proposing(
            alo_adapters::fallback_verbs::PICTURE_WINDOW,
            &[
                (APPLICATION, viewer),
                (alo_adapters::fallback_verbs::QUESTION, "What does it show?"),
            ],
        ),
        answering("It shows a beach."),
    ]);
    let tried = one_try(
        request,
        set.may_reach(),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    assert!(tried.took_the_road(), "{tried:?}");
    let picture = alo_adapters::fallback_verbs::PICTURE_WINDOW;
    let held = |at: usize| {
        model
            .held_to
            .get(at)
            .is_some_and(|verbs| verbs.iter().any(|verb| verb == picture))
    };
    let told = |at: usize| {
        model
            .shown
            .get(at)
            .is_some_and(|shown| shown.contains(picture))
    };
    assert!(!held(0) && !told(0), "offered before any read");
    assert!(
        held(1) && told(1),
        "not offered after a read of a window that describes nothing"
    );
}
