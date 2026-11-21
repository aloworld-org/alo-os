//! The set is the Demo 2 roads, every name in it is one the machine declares,
//! and its VSCodium request is the walk's own.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_adapters::fallback_verbs::{READ_WINDOW, TEXT, TYPE_TEXT};
use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_publishing::PUBLISH_TO_GITHUB_PAGES;
use alo_walking::writing_in_vscodium::{THE_FILE, THE_TEXT, VSCODIUM, the_ask};

use super::{THE_HOME, THE_VERSION, the_set};
use crate::a_road::{ARequest, Step};
use crate::testing::the_verbs;

/// A request's road and its other roads: every check of a road holds the
/// others too.
fn every_road_of(request: &ARequest) -> Vec<&[Step]> {
    std::iter::once(request.road())
        .chain(request.other_roads().iter().map(Vec::as_slice))
        .collect()
}

/// Every step of every road.
fn every_step_of(request: &ARequest) -> impl Iterator<Item = &Step> {
    every_road_of(request).into_iter().flatten()
}

fn the_road_of(named: &str) -> Vec<&'static str> {
    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == named)
        .unwrap_or_else(|| panic!("the set has {named}"));
    request
        .road()
        .iter()
        .map(crate::a_road::Step::verb)
        .collect()
}

#[test]
fn the_set_is_versioned_and_holds_a_request_for_every_verb() {
    assert_eq!(THE_VERSION, 35);
    let set = the_set().expect("the set builds");
    let named: Vec<&str> = set
        .requests()
        .iter()
        .map(|request| request.named())
        .collect();
    assert_eq!(
        named,
        [
            "write in VSCodium",
            "publish to GitHub Pages",
            "open an application by name",
            "list a folder",
            "read a file",
            "find files by name",
            "rename a file",
            "move a file",
            "archive a folder",
            "see what can be installed",
            "install an application",
            "bring an application to the front",
            "close an application",
            "ask the record about a file",
            "press a button in an application",
            "scroll to a named thing in an application",
            "scroll a page in an application",
            "move a slider in an application",
            "choose an option in a drop-down",
            "select a file in the file manager",
            "open a file in the file manager",
            "open a thing's right-click menu in an application",
            "press a key in an application",
            "type into a search box without saving",
            "answer the file chooser",
        ]
    );
}

/// **Every verb a model is offered is asked for by at least one request**, so
/// a change to any verb's words is tried against the model (law 9). A verb
/// added to the machine's list with no request here turns this red.
#[test]
fn every_verb_the_machine_offers_is_on_some_road() {
    let set = the_set().expect("the set builds");
    let asked: Vec<&str> = set
        .requests()
        .iter()
        .flat_map(|request| request.road().iter().map(crate::a_road::Step::verb))
        .collect();
    let missing: Vec<String> = the_verbs()
        .all()
        .map(|verb| verb.name().to_owned())
        .filter(|name| !asked.contains(&name.as_str()))
        .collect();
    assert!(missing.is_empty(), "no request asks for {missing:?}");
}

/// **Every step's scripted answer is one the protocol writes**, so the model
/// is never shown a line a machine could not send.
#[test]
fn every_step_is_answered_in_a_line_the_protocol_writes() {
    let verbs = the_verbs();
    let words = crate::testing::the_words();
    for request in the_set().expect("the set builds").requests() {
        for step in every_step_of(request) {
            let declared = verbs
                .of(step.verb())
                .unwrap_or_else(|| panic!("{} is offered", step.verb()));
            let given: Vec<(&str, alo_capability::Given)> = declared
                .args()
                .iter()
                .map(|arg| {
                    let decided = step
                        .expected()
                        .iter()
                        .find(|(named, _)| *named == arg.name())
                        .map(|(_, is)| is.clone());
                    (arg.name(), crate::testing::a_value_for(arg, decided))
                })
                .collect();
            let call = verbs
                .call(step.verb(), &given)
                .unwrap_or_else(|why| panic!("{}: {why:?}", request.named()));
            let answer = crate::the_machine_answers::answering(step.answered(), &call, &words);
            let line = answer.written().unwrap_or_else(|why| {
                panic!(
                    "{}: {} is not written: {why:?}",
                    request.named(),
                    step.verb()
                )
            });
            // A word the vocabulary lacks is shown as its key in guillemets:
            // the model would read a key where the machine says a sentence.
            assert!(
                !line.contains('«'),
                "{}: {} is answered with a key, not a sentence: {line}",
                request.named(),
                step.verb()
            );
        }
    }
}

#[test]
fn writing_in_vscodium_opens_reads_and_types_which_also_saves() {
    assert_eq!(
        the_road_of("write in VSCodium"),
        [OPEN_FILE_IN_APPLICATION, READ_WINDOW, TYPE_TEXT]
    );
}

#[test]
fn publishing_is_one_verb() {
    assert_eq!(
        the_road_of("publish to GitHub Pages"),
        [PUBLISH_TO_GITHUB_PAGES]
    );
}

/// **A window read after an application opens is aside of the road**, at both
/// levels: the try still has to open it and then answer. Opening it again
/// after the read is not the road, and neither is the read alone.
#[test]
fn opening_an_application_may_read_its_window_after_and_must_still_answer() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, proposing, reading, the_words};
    use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};

    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "open an application by name")
        .expect("the set has it");
    let browser = request
        .road()
        .first()
        .and_then(|open| {
            open.expected()
                .iter()
                .find(|(arg, _)| *arg == APPLICATION)
                .map(|(_, is)| is.clone())
        })
        .expect("the open names its application");
    let asides: Vec<&str> = request.aside().iter().map(|aside| aside.verb()).collect();
    assert_eq!(asides, [READ_WINDOW], "one read aside, of the window");

    let open = proposing(OPEN_APPLICATION, &[(APPLICATION, &browser)]);
    let look = reading(
        READ_WINDOW,
        &[(alo_adapters::fallback_verbs::APPLICATION, &browser)],
    );
    let answer = answering("Firefox is open.");
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let try_these = |lines: &[String]| {
            one_try_at(
                level,
                request,
                set.may_reach(),
                &the_verbs(),
                &the_words(),
                &mut Writing::these(lines),
            )
        };
        let looked = try_these(&[open.clone(), look.clone(), answer.clone()]);
        assert_eq!(looked.went(), &Went::TheRoad, "{level:?}: {looked:?}");
        assert_eq!(looked.looked_aside(), 1, "{level:?}");
        let straight = try_these(&[open.clone(), answer.clone()]);
        assert_eq!(straight.went(), &Went::TheRoad, "{level:?}: {straight:?}");
        assert_eq!(straight.looked_aside(), 0, "{level:?}");
        // **The refusals**: opening again after the read, and reading
        // instead of opening, both leave the road.
        let again = try_these(&[open.clone(), look.clone(), open.clone()]);
        assert!(!again.took_the_road(), "{level:?}: {again:?}");
        let only_looked = try_these(&[look.clone(), answer.clone()]);
        assert!(!only_looked.took_the_road(), "{level:?}: {only_looked:?}");
    }
}

#[test]
fn opening_an_application_is_one_verb() {
    assert_eq!(
        the_road_of("open an application by name"),
        [alo_applications::verbs::OPEN_APPLICATION]
    );
}

/// **The check and the walk ask the same thing**: the request is the walk's
/// own, and the application and the text the road expects are the walk's.
#[test]
fn the_vscodium_request_is_the_walks_own() {
    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "write in VSCodium")
        .expect("the set has it");
    assert_eq!(request.words(), the_ask());
    let expected: Vec<(&str, &str)> = request
        .road()
        .iter()
        .flat_map(|step| step.expected().iter().map(|(arg, is)| (*arg, is.as_str())))
        .collect();
    assert!(expected.contains(&(alo_applications::verbs::APPLICATION, VSCODIUM)));
    assert!(expected.contains(&(TEXT, THE_TEXT)));
    assert!(expected.contains(&(
        alo_applications::opening_a_file::PATH,
        format!("{THE_HOME}/{THE_FILE}").as_str()
    )));
}

/// **Every name the set expects is one the machine offers**: each verb is on
/// the list a model is given, and each argument a step decides is one that
/// verb declares. A rename on either side turns this red.
#[test]
fn every_verb_and_argument_the_set_expects_is_on_the_machines_list() {
    let verbs = the_verbs();
    for request in the_set().expect("the set builds").requests() {
        for step in every_step_of(request) {
            let verb = verbs
                .of(step.verb())
                .unwrap_or_else(|| panic!("{} is offered", step.verb()));
            for (argument, _) in step.expected() {
                assert!(
                    verb.args().iter().any(|arg| arg.name() == *argument),
                    "{} takes {argument}",
                    step.verb()
                );
            }
        }
    }
}

/// The machine is the walk's: full access, the person's home, and the
/// applications a fresh machine installs, VSCodium among them.
#[test]
fn the_machine_is_told_full_access_the_home_and_the_applications() {
    let set = the_set().expect("the set builds");
    let told = set.may_reach().told();
    assert!(told.contains(THE_HOME), "{told}");
    assert!(told.contains(VSCODIUM), "{told}");
    assert!(told.contains("org.mozilla.firefox"), "{told}");
    let at_full_access = alo_instructing::WhatItMayReach::nothing().at_full_access(true);
    let full_access_told = at_full_access.told();
    assert!(!full_access_told.is_empty());
    assert!(told.contains(full_access_told.trim()), "{told}");
}

#[test]
fn one_request_can_be_tried_alone_and_an_unknown_one_is_none() {
    let set = the_set().expect("the set builds");
    let only = set
        .clone()
        .only("publish to GitHub Pages")
        .expect("it is in the set");
    assert_eq!(only.requests().len(), 1);
    assert!(set.only("nothing called this").is_none());
}

/// **A look is a read, and never a road's last step**: a change cannot be
/// skipped, and a road ending on a look could not tell skipping it from
/// stopping short.
#[test]
fn every_look_is_a_read_and_not_last() {
    let verbs = the_verbs();
    for request in the_set().expect("the set builds").requests() {
        for road in every_road_of(request) {
            for (at, step) in road.iter().enumerate() {
                if !step.is_a_look() {
                    continue;
                }
                let declared = verbs
                    .of(step.verb())
                    .unwrap_or_else(|| panic!("{} is offered", step.verb()));
                assert_eq!(
                    declared.effect(),
                    alo_capability::Effect::Read,
                    "{}: a look at {} is a read",
                    request.named(),
                    step.verb()
                );
                assert!(
                    at + 1 < road.len(),
                    "{}: a look is not last",
                    request.named()
                );
            }
        }
    }
}

/// **The scrolling request's other roads are pages down and then the heading
/// on the screen** — read there, or scrolled to by name — parting from the
/// road where the road scrolls to it.
#[test]
fn scrolling_down_until_a_heading_shows_may_go_a_page_at_a_time() {
    use alo_adapters::fallback_verbs::{SCROLL_PAGE, SCROLL_TO};
    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "scroll to a named thing in an application")
        .expect("the set scrolls to a thing");
    let verbs = |road: &[Step]| road.iter().map(Step::verb).collect::<Vec<_>>();
    assert_eq!(verbs(request.road()), [READ_WINDOW, SCROLL_TO]);
    let [read, named, read_and_named] = request.other_roads() else {
        panic!("three other roads: {:?}", request.other_roads());
    };
    assert_eq!(verbs(read), [READ_WINDOW, SCROLL_PAGE, READ_WINDOW]);
    assert_eq!(verbs(named), [READ_WINDOW, SCROLL_PAGE, SCROLL_TO]);
    assert_eq!(
        verbs(read_and_named),
        [READ_WINDOW, SCROLL_PAGE, READ_WINDOW, SCROLL_TO]
    );
    for other in [read, named, read_and_named] {
        assert!(
            other
                .iter()
                .any(|step| step.verb() == SCROLL_PAGE && step.is_taken_as_often_as_asked()),
            "{other:?}"
        );
        assert!(
            matches!(
                other.last().map(Step::answered),
                Some(crate::a_road::Answered::With(done))
                    if format!("{done:?}").contains(crate::every_other_verb::FURTHER_DOWN)
            ),
            "{other:?}"
        );
    }
}

/// **The machine installs from the places a fresh machine is set up with**:
/// every place the shipped applications come from (`shipped.toml`), once
/// each, and the model's answers are held to exactly those, as a booted
/// machine's are (`alo_turn::the_shape_a_provider_is_held_to`).
#[test]
fn the_machine_installs_from_the_places_a_fresh_machine_has() {
    let set = the_set().expect("the set builds");
    let shipped = alo_software::Shipped::decided().expect("the shipped list reads");
    let mut expected: Vec<String> = shipped
        .every()
        .iter()
        .map(|pinned| pinned.source().as_str().to_owned())
        .collect();
    expected.sort_unstable();
    expected.dedup();
    assert_eq!(set.places(), expected.as_slice());
    assert!(
        !set.places().is_empty(),
        "a fresh machine installs from somewhere"
    );
    let held = alo_turn::the_shape_a_provider_is_held_to::the_envelope_naming_the_verbs_and_places(
        &the_verbs(),
        set.places(),
    )
    .to_string();
    assert!(
        held.contains(&format!("\"enum\":{}", serde_json::json!(expected))),
        "{held}"
    );
}

/// **Every read aside of a road is a read**: a change is never aside.
#[test]
fn every_read_aside_is_a_read() {
    let verbs = the_verbs();
    for request in the_set().expect("the set builds").requests() {
        for aside in request.aside() {
            let declared = verbs
                .of(aside.verb())
                .unwrap_or_else(|| panic!("{} is offered", aside.verb()));
            assert_eq!(
                declared.effect(),
                alo_capability::Effect::Read,
                "{}: {} aside is a read",
                request.named(),
                aside.verb()
            );
        }
    }
}

/// **Each shipped application is told as a booted machine tells it**: by the
/// name its package gives it and, in brackets, its identifier — the names
/// pinned from the packages (`alo_software::ShippedNames`), not typed here.
#[test]
fn every_shipped_application_is_told_by_its_packages_name() {
    let names = alo_software::ShippedNames::pinned().expect("the pinned names read");
    assert!(!names.every().is_empty(), "no name is pinned");
    let told = the_set().expect("the set builds").may_reach().told();
    for (identifier, name) in names.every() {
        assert!(told.contains(&format!("{name} ({identifier})")), "{told}");
    }
}

/// **Typing in VSCodium names the field as the machine finds it**: the entry
/// named after the file, in its window, with the keyboard — not one spelling
/// of its name.
#[test]
fn typing_in_vscodium_expects_the_field_the_window_shows() {
    let set = the_set().expect("the set builds");
    let writing = set
        .requests()
        .iter()
        .find(|request| request.named() == "write in VSCodium")
        .expect("the set writes in VSCodium");
    let typing = writing
        .road()
        .iter()
        .find(|step| step.verb() == TYPE_TEXT)
        .expect("the road types");
    assert!(
        typing
            .expected()
            .iter()
            .all(|(argument, _)| *argument != alo_adapters::fallback_verbs::NAME)
    );
    assert_eq!(
        typing.fields(),
        [(
            alo_adapters::fallback_verbs::NAME,
            alo_adapters::places_for_text::APlaceForText::as_shown(
                THE_FILE,
                &format!("{THE_FILE} - VSCodium"),
                true
            )
        )]
    );
}

/// **Opening a file the file manager shows, as a person asks it**: with
/// Dolphin's window offered, and a path guessed where nothing is refused as
/// the machine refuses it before asking — so a try that guesses, is refused,
/// reads the window and opens the item takes the road, at both levels, and
/// says it was refused first. Opening it by the path that is there, in the
/// file manager, is the other road: the machine opens it as a double-click
/// there would (the owner, 2026-11-17).
#[test]
fn opening_a_shown_file_goes_on_from_a_wrong_path_and_holds_the_verb() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, proposing, reading, the_words};
    use alo_adapters::fallback_verbs::{APPLICATION as IN, NAME, OPEN_ITEM};
    use alo_applications::opening_a_file::PATH;
    use alo_applications::verbs::APPLICATION;

    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "open a file in the file manager")
        .expect("the set has it");
    let window = request.window().expect("Dolphin's window is offered");
    let dolphin = window.application().to_owned();
    assert_eq!(window.title(), "Downloads");
    let there = format!("{THE_HOME}/Downloads/report.pdf");
    assert!(
        request
            .there()
            .iter()
            .any(|at| at.display().to_string() == there),
        "{:?}",
        request.there()
    );

    let guessed = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[
            (APPLICATION, &dolphin),
            (PATH, &format!("{THE_HOME}/Documents/report.pdf")),
        ],
    );
    let by_its_real_path = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, &dolphin), (PATH, &there)],
    );
    let look = reading(READ_WINDOW, &[(IN, &dolphin)]);
    let open = proposing(OPEN_ITEM, &[(IN, &dolphin), (NAME, "report.pdf")]);
    let answer = answering("report.pdf is open.");
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let try_these = |lines: &[String]| {
            let mut model = Writing::these(lines);
            let tried = one_try_at(
                level,
                request,
                set.may_reach(),
                &the_verbs(),
                &the_words(),
                &mut model,
            );
            (tried, model.shown)
        };
        let (went_on, shown) =
            try_these(&[guessed.clone(), look.clone(), open.clone(), answer.clone()]);
        assert_eq!(went_on.went(), &Went::TheRoad, "{level:?}: {went_on:?}");
        assert_eq!(went_on.refused_before_asking(), 1, "{level:?}");
        let first = shown.first().expect("the model was asked");
        assert!(first.contains(&window.told()), "{level:?}: {first}");
        assert!(
            shown
                .get(1)
                .is_some_and(|second| second.contains("there is nothing at")),
            "{level:?}: the refusal is shown: {shown:?}"
        );
        // **By the path that is there is the other road** since the machine
        // opens it as a double-click in the file manager would (2026-11-17),
        // and so is guessing, being refused, and then naming it rightly.
        let (by_path, _) = try_these(&[by_its_real_path.clone(), answer.clone()]);
        assert_eq!(by_path.went(), &Went::TheRoad, "{level:?}: {by_path:?}");
        let (guessed_then_right, _) =
            try_these(&[guessed.clone(), by_its_real_path.clone(), answer.clone()]);
        assert_eq!(
            guessed_then_right.went(),
            &Went::TheRoad,
            "{level:?}: {guessed_then_right:?}"
        );
        // **The refusal case**: guessing and then answering is not the road.
        let (guessing, _) = try_these(&[guessed.clone(), answer.clone()]);
        assert!(!guessing.took_the_road(), "{level:?}: {guessing:?}");
    }
}

/// **A slider named as the request says it, not as the window shows it, is
/// refused before the card**, as the machine now refuses it at both levels, in
/// the move's own words — and a try that names it again as the window shows it
/// takes the road, said to have been refused first. Naming it wrongly until the
/// end is not the road.
#[test]
fn a_slider_named_wrongly_is_refused_before_the_card_and_the_try_goes_on() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, proposing, the_words};
    use alo_adapters::fallback_verbs::{APPLICATION, NAME, SET_VALUE, VALUE};

    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "move a slider in an application")
        .expect("the set has it");
    let moving = request
        .road()
        .iter()
        .find(|step| step.verb() == SET_VALUE)
        .expect("the road moves the slider");
    assert!(moving.is_the_only_control_of_its_kind());
    let expected = |argument: &str| {
        moving
            .expected()
            .iter()
            .find(|(arg, _)| *arg == argument)
            .map(|(_, is)| is.clone())
            .expect("the road names it")
    };
    let (viewer, zoom, to) = (expected(APPLICATION), expected(NAME), expected(VALUE));
    let wrongly = format!("{zoom} slider");
    let as_said = proposing(
        SET_VALUE,
        &[(APPLICATION, &viewer), (NAME, &wrongly), (VALUE, &to)],
    )
    .replace(&format!("\"{to}\""), &to);
    let as_shown = proposing(
        SET_VALUE,
        &[(APPLICATION, &viewer), (NAME, &zoom), (VALUE, &to)],
    )
    .replace(&format!("\"{to}\""), &to);
    let answer = answering("The zoom is at 200.");
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let try_these = |lines: &[String]| {
            let mut model = Writing::these(lines);
            let tried = one_try_at(
                level,
                request,
                set.may_reach(),
                &the_verbs(),
                &the_words(),
                &mut model,
            );
            (tried, model.shown)
        };
        let (went_on, shown) = try_these(&[as_said.clone(), as_shown.clone(), answer.clone()]);
        assert_eq!(went_on.went(), &Went::TheRoad, "{level:?}: {went_on:?}");
        assert_eq!(went_on.refused_before_asking(), 1, "{level:?}");
        let refusal = format!("shows no slider or number box named “{wrongly}” now");
        assert!(
            shown.get(1).is_some_and(|second| second.contains(&refusal)),
            "{level:?}: the refusal is shown: {shown:?}"
        );
        // **The refusal case**: naming it wrongly and then answering is not
        // the road.
        let (stopped, _) = try_these(&[as_said.clone(), answer.clone()]);
        assert!(!stopped.took_the_road(), "{level:?}: {stopped:?}");
    }
}

/// **A file named by a path where nothing is is refused as the machine
/// refuses it**, and the try goes on: a read of a path that names nothing is
/// answered by `alo_files::Touching` with *there is nothing at …*, at both
/// levels, and a model that then reads the file the person named takes the
/// road, said to have been refused first. Run 44793910551: one try at ask
/// each time named `notes.txt},` and the check ended it where the machine
/// would have answered.
#[test]
fn reading_a_file_goes_on_from_a_path_where_nothing_is() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, reading, the_words};
    use alo_files::verbs::{FILE, READ_FILE};

    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "read a file")
        .expect("the set has it");
    let notes = format!("{THE_HOME}/Documents/notes.txt");
    let garbled = reading(READ_FILE, &[(FILE, &format!("{notes}}},"))]);
    let read = reading(READ_FILE, &[(FILE, &notes)]);
    let answer = answering("It says to buy bread and call Anna on Friday.");
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let try_these = |lines: &[String]| {
            let mut model = Writing::these(lines);
            let tried = one_try_at(
                level,
                request,
                set.may_reach(),
                &the_verbs(),
                &the_words(),
                &mut model,
            );
            (tried, model.shown)
        };
        let (went_on, shown) = try_these(&[garbled.clone(), read.clone(), answer.clone()]);
        assert_eq!(went_on.went(), &Went::TheRoad, "{level:?}: {went_on:?}");
        assert_eq!(went_on.refused_before_asking(), 1, "{level:?}");
        assert!(
            shown
                .get(1)
                .is_some_and(|second| second.contains("there is nothing at")),
            "{level:?}: the refusal is shown: {shown:?}"
        );
        let (straight, _) = try_these(&[read.clone(), answer.clone()]);
        assert_eq!(straight.went(), &Went::TheRoad, "{level:?}: {straight:?}");
        assert_eq!(straight.refused_before_asking(), 0, "{level:?}");
        // **The refusal case**: the wrong path and then an answer is not
        // the road.
        let (stopped, _) = try_these(&[garbled.clone(), answer.clone()]);
        assert!(!stopped.took_the_road(), "{level:?}: {stopped:?}");
    }
}

/// **A path through `/home` is the same place at full access**: on this image
/// `/home` is a link to `/var/home` (`image/usr/lib/tmpfiles.d/alo.conf`, the
/// walk of 2026-10-26 in `alo_capability::Grant::really`), every path verb
/// resolves what it is given through every link (`alo_files::Touching`), and
/// full access answers every path (`alo_capability::full_access`). So at full
/// access a booted machine opens `/home/alo/hello.txt` as the file the walk
/// made, and the check takes it as that file. At ask each time the grants are
/// asked about the path as it was written, so it is held as before.
#[test]
fn a_path_through_home_is_the_same_file_at_full_access() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, proposing, reading, the_words};
    use alo_adapters::fallback_verbs::{APPLICATION as IN, NAME};
    use alo_applications::opening_a_file::PATH;
    use alo_applications::verbs::APPLICATION;
    use alo_files::verbs::{FILE, READ_FILE};

    let set = the_set().expect("the set builds");
    let named = |called: &str| {
        set.requests()
            .iter()
            .find(|request| request.named() == called)
            .expect("the set has it")
    };
    let writing = named("write in VSCodium");
    let editor = writing
        .road()
        .first()
        .and_then(|open| {
            open.expected()
                .iter()
                .find(|(arg, _)| *arg == APPLICATION)
                .map(|(_, is)| is.clone())
        })
        .expect("the open names its application");
    let through_the_link = format!("/home/alo/{THE_FILE}");
    let open = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, &editor), (PATH, &through_the_link)],
    );
    let typing = proposing(
        TYPE_TEXT,
        &[(IN, &editor), (NAME, THE_FILE), (TEXT, THE_TEXT)],
    );
    let answer = answering("It is written and saved.");
    let try_at = |level: TheLevel, request: &ARequest, lines: &[String]| {
        one_try_at(
            level,
            request,
            set.may_reach(),
            &the_verbs(),
            &the_words(),
            &mut Writing::these(lines),
        )
    };
    let opened = try_at(
        TheLevel::FullAccess,
        writing,
        &[open.clone(), typing.clone(), answer.clone()],
    );
    assert_eq!(opened.went(), &Went::TheRoad, "{opened:?}");
    assert_eq!(opened.refused_before_asking(), 0, "{opened:?}");
    let read = try_at(
        TheLevel::FullAccess,
        named("read a file"),
        &[
            reading(READ_FILE, &[(FILE, "/home/alo/Documents/notes.txt")]),
            answering("It says to buy bread and call Anna on Friday."),
        ],
    );
    assert_eq!(read.went(), &Went::TheRoad, "{read:?}");
    // **The refusal cases**: at ask each time it is held as before, and a
    // path that is not the link's is not taken for the file.
    let asked = try_at(
        TheLevel::AskEachTime,
        writing,
        &[open.clone(), typing.clone(), answer.clone()],
    );
    assert!(!asked.took_the_road(), "{asked:?}");
    let elsewhere = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[
            (APPLICATION, &editor),
            (PATH, &format!("/homes/alo/{THE_FILE}")),
        ],
    );
    let wrong = try_at(
        TheLevel::FullAccess,
        writing,
        &[elsewhere, typing.clone(), answer.clone()],
    );
    assert!(!wrong.took_the_road(), "{wrong:?}");
}

/// **A call the machine turns away as it is written is refused in its words,
/// and the try goes on**: a booted machine builds every call through
/// `Verbs::call` before anything else (`alo_turn::Turning::calling`), turns a
/// text with nothing in it away there, at both levels, and the agent's loop
/// reads the refusal as a step (`alo_agentd`'s `not_done`). Runs 44793910551,
/// 44803684908 and 44793268747: the model sent `accessible.type_text` with an
/// empty text to the Reload button in five tries, and the check ended each
/// where the machine would have answered.
#[test]
fn a_call_turned_away_as_written_is_refused_and_the_try_goes_on() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, proposing, the_words};
    use alo_adapters::fallback_verbs::{ACTIVATE_CONTROL, APPLICATION, KIND, NAME};

    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "press a button in an application")
        .expect("the set has it");
    let pressing = request
        .road()
        .iter()
        .find(|step| step.verb() == ACTIVATE_CONTROL)
        .expect("the road presses");
    let expected = |argument: &str| {
        pressing
            .expected()
            .iter()
            .find(|(arg, _)| *arg == argument)
            .map(|(_, is)| is.clone())
            .expect("the road names it")
    };
    let (browser, kind, reload) = (expected(APPLICATION), expected(KIND), expected(NAME));
    let nothing_typed = proposing(
        TYPE_TEXT,
        &[(APPLICATION, &browser), (NAME, &reload), (TEXT, "")],
    );
    let press = proposing(
        ACTIVATE_CONTROL,
        &[(APPLICATION, &browser), (KIND, &kind), (NAME, &reload)],
    );
    let answer = answering("Reload is pressed.");
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let try_these = |lines: &[String]| {
            let mut model = Writing::these(lines);
            let tried = one_try_at(
                level,
                request,
                set.may_reach(),
                &the_verbs(),
                &the_words(),
                &mut model,
            );
            (tried, model.shown)
        };
        let (went_on, shown) = try_these(&[nothing_typed.clone(), press.clone(), answer.clone()]);
        assert_eq!(went_on.went(), &Went::TheRoad, "{level:?}: {went_on:?}");
        assert_eq!(went_on.refused_before_asking(), 1, "{level:?}");
        let said = alo_capability::CallError::Argument(alo_capability::ArgError::Empty {
            argument: TEXT.to_owned(),
        })
        .said(&the_words())
        .into_text();
        assert!(
            shown.get(1).is_some_and(|second| second.contains(&said)),
            "{level:?}: the machine's own refusal is shown, {said:?}: {shown:?}"
        );
        // **The refusal cases**: turned away and then answering is not the
        // road, and a call the machine takes, on another verb, still leaves it.
        let (stopped, _) = try_these(&[nothing_typed.clone(), answer.clone()]);
        assert!(!stopped.took_the_road(), "{level:?}: {stopped:?}");
        let typed = proposing(
            TYPE_TEXT,
            &[(APPLICATION, &browser), (NAME, &reload), (TEXT, "x")],
        );
        let (wrong_verb, _) = try_these(&[typed, press.clone(), answer.clone()]);
        assert!(!wrong_verb.took_the_road(), "{level:?}: {wrong_verb:?}");
    }
}

/// **A call turned away leaves the try on the road it was on**: the refusal is
/// not a step, so a request with another road that the refused verb would
/// have parted onto is still held to its own, and the next correct step
/// passes (the rule #1393 set for every refusal before asking).
#[test]
fn a_call_turned_away_leaves_the_try_on_its_road() {
    use crate::a_road::Answered;
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::testing::{Writing, answering, proposing, the_words};
    use alo_adapters::fallback_verbs::{ACTIVATE_CONTROL, APPLICATION, KIND, NAME};

    let set = the_set().expect("the set builds");
    let firefox = "org.mozilla.firefox";
    let press = || {
        Step::of(ACTIVATE_CONTROL, Answered::TypedAndSaved)
            .expecting(APPLICATION, firefox)
            .expecting(NAME, "Reload")
    };
    let typing = || {
        Step::of(TYPE_TEXT, Answered::TypedAndSaved)
            .expecting(APPLICATION, firefox)
            .expecting(NAME, "Reload")
    };
    let request = ARequest::asking("press", "Press Reload in Firefox.", vec![press()])
        .or_by_another_road(vec![typing()]);
    let nothing_typed = proposing(
        TYPE_TEXT,
        &[(APPLICATION, firefox), (NAME, "Reload"), (TEXT, "")],
    );
    let pressed = proposing(
        ACTIVATE_CONTROL,
        &[(APPLICATION, firefox), (KIND, "button"), (NAME, "Reload")],
    );
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let tried = one_try_at(
            level,
            &request,
            set.may_reach(),
            &the_verbs(),
            &the_words(),
            &mut Writing::these(&[
                nothing_typed.clone(),
                pressed.clone(),
                answering("Pressed."),
            ]),
        );
        assert_eq!(tried.went(), &Went::TheRoad, "{level:?}: {tried:?}");
        assert_eq!(tried.refused_before_asking(), 1, "{level:?}");
        assert!(!tried.by_the_other_road(), "{level:?}: {tried:?}");
    }
}

/// **A heading read on the screen and then scrolled to by name reaches the same
/// place**: the machine answers a scroll to a thing already on the screen
/// (`alo_adapters::scrolling`, *already on the screen*, or scrolled), so a try
/// that pages down, reads the heading there, and scrolls to it anyway takes
/// the road, by another road. Run 44807502026 (10 tries on main): three of the
/// four misses were that scroll, after the whole of the other road.
#[test]
fn a_heading_read_on_the_screen_may_still_be_scrolled_to_by_name() {
    use crate::a_try::{TheLevel, Went, one_try_at};
    use crate::every_other_verb::FURTHER_DOWN;
    use crate::testing::{Writing, answering, proposing, reading, the_words};
    use alo_adapters::fallback_verbs::{APPLICATION, SCROLL_PAGE, SCROLL_TO, TO, TOWARD};

    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "scroll to a named thing in an application")
        .expect("the set scrolls to a thing");
    let browser = request
        .road()
        .first()
        .and_then(|look| {
            look.expected()
                .iter()
                .find(|(arg, _)| *arg == APPLICATION)
                .map(|(_, is)| is.clone())
        })
        .expect("the look names its application");
    let page = proposing(SCROLL_PAGE, &[(APPLICATION, &browser), (TOWARD, "down")]);
    let look = reading(READ_WINDOW, &[(APPLICATION, &browser)]);
    let to = |name: &str| proposing(SCROLL_TO, &[(APPLICATION, &browser), (TO, name)]);
    let answer = answering("The heading Downloads is on the screen.");
    for level in [TheLevel::FullAccess, TheLevel::AskEachTime] {
        let try_these = |lines: &[String]| {
            one_try_at(
                level,
                request,
                set.may_reach(),
                &the_verbs(),
                &the_words(),
                &mut Writing::these(lines),
            )
        };
        let scrolled_anyway =
            try_these(&[page.clone(), look.clone(), to(FURTHER_DOWN), answer.clone()]);
        assert_eq!(
            scrolled_anyway.went(),
            &Went::TheRoad,
            "{level:?}: {scrolled_anyway:?}"
        );
        let read_it = try_these(&[page.clone(), look.clone(), answer.clone()]);
        assert_eq!(read_it.went(), &Went::TheRoad, "{level:?}: {read_it:?}");
        // **The refusal cases**: scrolling to something else, or scrolling to
        // it twice, is not the road.
        let elsewhere = try_these(&[
            page.clone(),
            look.clone(),
            to("Getting started"),
            answer.clone(),
        ]);
        assert!(!elsewhere.took_the_road(), "{level:?}: {elsewhere:?}");
        let twice = try_these(&[
            page.clone(),
            look.clone(),
            to(FURTHER_DOWN),
            to(FURTHER_DOWN),
            answer.clone(),
        ]);
        assert!(!twice.took_the_road(), "{level:?}: {twice:?}");
    }
}
