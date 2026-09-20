//! The laid-out approval surface, read as text, boxes and pixels: what is on
//! it, exactly, and what never is.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use super::*;
use crate::approval_testing::{archiving, hour, noon, on_a_machine, words};
use crate::painted_text::sentence as shaped_sentence;
use crate::{ApprovalKey, ApprovalScreen};
use alo_appearance::Token;
use alo_capability::Given;

/// The ordinary look in `scheme`, read `reading`.
fn look(scheme: Scheme, reading: Direction) -> ApprovalLook {
    ApprovalLook {
        contrast: Contrast::AsDesigned,
        scheme,
        scale: TextScale::ordinary(),
        reading,
    }
}

/// The ordinary light look, read left to right.
fn light() -> ApprovalLook {
    look(Scheme::Light, Direction::LeftToRight)
}

/// The surface as it stands, drawn on a 1920×1080 output.
fn drawn(screen: &ApprovalScreen, look: ApprovalLook) -> ApprovalPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(screen.shows(), &words(), &mut labels, (1920, 1080), look).unwrap()
}

/// **The text drawn is byte-for-byte the text the turn wrote.** The sentence
/// on the panel is the proposal's own sentence as `alo-turn` renders it — not
/// shortened, not headed, not summarised — and the pixels inked for it are
/// exactly the pixels of that text shaped on its own, so nothing was added to
/// or taken from it between the turn and the screen.
#[test]
fn the_text_drawn_is_byte_for_byte_the_text_the_turn_wrote() {
    on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let written = turning
            .proposed(march)
            .unwrap()
            .proposal
            .sentence(turning.strings())
            .text()
            .to_owned();
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        let picture = drawn(&screen, light());

        let (text, area) = picture.sentence.clone().unwrap();
        assert_eq!(text.as_bytes(), written.as_bytes());
        assert!(text.contains(&places.invoice("march").display().to_string()));

        let mut labels = WindowControlLabels::new().unwrap();
        let palette = Palette::of(Scheme::Light, Contrast::AsDesigned);
        let alone = shaped_sentence(
            &mut labels.fonts,
            &written,
            area.size.w,
            Measure::of(TextScale::ordinary()).metrics(),
            palette.ground,
            palette.ink,
        );
        let inked = picture
            .inked
            .iter()
            .find(|inked| inked.area == area)
            .expect("the sentence was not inked where it was laid out");
        assert_eq!(inked.area.size.h, alone.height, "the sentence was cut");
        assert_eq!(
            inked.pixels, alone.pixels,
            "the pixels are not that sentence"
        );

        // Whose question it is, and the two answers, are the only other words
        // on the panel — no heading, no summary, nothing of this crate's.
        assert_eq!(picture.agent.as_deref(), Some("@files"));
        assert_eq!(picture.inked.len(), 1 + 1 + 2);
    });
}

/// **A long sentence is drawn whole**, its lines broken to the panel's width
/// and nothing else: every byte of a change with a long path in it is inked,
/// and the panel grows to hold it.
#[test]
fn a_long_sentence_is_drawn_whole() {
    on_a_machine(|turning, grants, places| {
        let deep = places
            .invoices
            .join("a folder with a long name that goes on for some time")
            .join("and another inside it that nobody would call short");
        std::fs::create_dir_all(&deep).unwrap();
        let file = deep.join("the quarterly invoice nobody wants to read again.pdf");
        std::fs::write(&file, "long").unwrap();
        let id = turning
            .proposing(
                "move_file",
                &[
                    ("file", Given::text(file.to_string_lossy().into_owned())),
                    (
                        "into",
                        Given::text(places.archive.to_string_lossy().into_owned()),
                    ),
                ],
                grants,
                hour(),
                noon(),
            )
            .unwrap();
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, id, noon());
        let picture = drawn(&screen, light());
        let (text, area) = picture.sentence.unwrap();
        assert!(text.ends_with(&places.archive.display().to_string()));
        assert!(text.contains("nobody wants to read again.pdf"));
        assert!(
            area.size.h >= 3 * 24,
            "a long sentence was not given its lines"
        );
        let panel = picture.panel.unwrap();
        assert!(panel.contains_rect(area));
    });
}

/// **Two answers and no third**: the panel holds exactly the two answers
/// `alo-approving` words, *no* first in reading order, and they stand at the
/// far end of a left-to-right line and are mirrored for right to left.
#[test]
fn two_answers_and_no_third_in_reading_order() {
    on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        let strings = words();
        let ApprovalShows::Question { asked, .. } = screen.shows() else {
            panic!("no question");
        };

        let left_to_right = drawn(&screen, light());
        let answers: Vec<_> = left_to_right
            .answers
            .iter()
            .map(|drawn| (drawn.answer, drawn.words.clone()))
            .collect();
        assert_eq!(
            answers,
            [
                (
                    ApprovalAnswer::No,
                    asked.no_said(&strings).text().to_owned()
                ),
                (
                    ApprovalAnswer::Approve,
                    asked.approve_said(&strings).text().to_owned()
                ),
            ]
        );
        let [no, approve] = [&left_to_right.answers[0], &left_to_right.answers[1]];
        assert!(no.area.loc.x < approve.area.loc.x);
        let panel = left_to_right.panel.unwrap();
        assert!(panel.contains_rect(no.area) && panel.contains_rect(approve.area));

        let right_to_left = drawn(&screen, look(Scheme::Light, Direction::RightToLeft));
        let [no, approve] = [&right_to_left.answers[0], &right_to_left.answers[1]];
        assert_eq!(no.answer, ApprovalAnswer::No);
        assert!(no.area.loc.x > approve.area.loc.x, "not mirrored");
    });
}

/// **Nothing is preselected, and a selected answer is never told apart by
/// colour alone.** With nothing selected the two boxes are drawn with the same
/// shapes; selected, one gets a thicker edge and a bar under its words — more
/// shapes, in the same two colours the rest of the panel uses.
#[test]
fn nothing_is_preselected_and_selection_is_never_colour_alone() {
    on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());

        let fresh = drawn(&screen, light());
        assert!(fresh.answers.iter().all(|answer| !answer.selected));
        let shapes_in = |picture: &ApprovalPicture, area: Rectangle<i32, Physical>| {
            picture
                .solids
                .iter()
                .filter(|solid| area.contains_rect(solid.area))
                .map(|solid| (solid.area.loc - area.loc, solid.area.size))
                .collect::<Vec<_>>()
        };
        let no = shapes_in(&fresh, fresh.answers[0].area);
        let approve = shapes_in(&fresh, fresh.answers[1].area);
        assert_eq!(no, approve, "an answer looks selected before anybody chose");

        screen.pressed(ApprovalKey::PreviousAnswer, turning, grants, noon());
        let chosen = drawn(&screen, light());
        let approve_box = &chosen.answers[1];
        assert!(approve_box.selected && !chosen.answers[0].selected);
        let selected_shapes = shapes_in(&chosen, approve_box.area);
        assert!(
            selected_shapes.len() > approve.len(),
            "selection is not an added shape"
        );
        assert_ne!(selected_shapes, approve);

        let palette = Palette::of(Scheme::Light, Contrast::AsDesigned);
        for solid in &chosen.solids {
            assert!(
                solid.colour == palette.ink || solid.colour == palette.ground,
                "a colour beyond the panel's two"
            );
        }
    });
}

/// **Terracotta is not on the surface**, in either scheme: it means the agent
/// acting, and a question is the moment it has not.
#[test]
fn terracotta_is_not_on_the_surface() {
    let terracotta = Token::Terracotta.colour();
    let terracotta = [terracotta.red(), terracotta.green(), terracotta.blue()];
    for scheme in [Scheme::Light, Scheme::Dark] {
        let palette = Palette::of(scheme, Contrast::AsDesigned);
        assert_ne!(palette.ground, terracotta);
        assert_ne!(palette.ink, terracotta);
    }
}

/// **Nothing open draws nothing**, on any output — no panel, no pixel — and a
/// refusal draws its sentence with no answer under it.
#[test]
fn nothing_open_draws_nothing_and_a_refusal_draws_no_answers() {
    on_a_machine(|turning, grants, places| {
        let screen = ApprovalScreen::on_an_output();
        assert!(drawn(&screen, light()).is_empty());
        let mut labels = WindowControlLabels::new().unwrap();
        for size in [(1, 1), (20_000, 20_000), (0, 0)] {
            assert!(
                picture(screen.shows(), &words(), &mut labels, size, light())
                    .unwrap()
                    .is_empty()
            );
        }

        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        screen.answer(
            crate::ApprovalAnswer::Approve,
            turning,
            &alo_capability::Grants::default(),
            noon(),
        );
        let ApprovalShows::Refusal(said) = screen.shows() else {
            panic!("no refusal is open");
        };
        let refusal = drawn(&screen, look(Scheme::Dark, Direction::LeftToRight));
        assert_eq!(refusal.sentence.as_ref().unwrap().0, said.text());
        assert!(refusal.answers.is_empty());
        assert_eq!(refusal.agent, None);
        assert_eq!(refusal.inked.len(), 1);
    });
}

/// **A panel that cannot hold the whole sentence refuses the frame** rather
/// than cutting it: too short, too narrow, or larger than any output.
#[test]
fn a_panel_that_cannot_hold_the_whole_sentence_refuses_the_frame() {
    on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        let mut labels = WindowControlLabels::new().unwrap();
        for size in [(1920, 120), (150, 1080), (20_000, 1080), (1920, 20_000)] {
            assert!(
                matches!(
                    picture(screen.shows(), &words(), &mut labels, size, light()),
                    Err(RenderError::ApprovalScene)
                ),
                "{size:?}"
            );
        }
        // A larger text scale is a taller panel, not a cut one.
        let large = ApprovalLook {
            contrast: Contrast::AsDesigned,
            scale: TextScale::percent(200).unwrap(),
            ..light()
        };
        let ordinary = picture(screen.shows(), &words(), &mut labels, (1920, 1080), light())
            .unwrap()
            .panel
            .unwrap();
        let larger = picture(screen.shows(), &words(), &mut labels, (1920, 1080), large)
            .unwrap()
            .panel
            .unwrap();
        assert!(larger.size.h > ordinary.size.h);

        let picture =
            picture(screen.shows(), &words(), &mut labels, (1920, 1080), light()).unwrap();
        assert!(picture.validate((1920, 1080).into()).is_ok());
        assert!(matches!(
            picture.validate((1280, 720).into()),
            Err(RenderError::ApprovalScene)
        ));
    });
}

/// **High contrast is the same surface in the other palette**: the sentence
/// and the two answers are where they were, and every flat colour is one
/// `alo_access::HighContrast` decided.
#[test]
fn high_contrast_draws_the_same_surface_in_the_palette_that_crate_decided() {
    on_a_machine(|turning, grants, places| {
        let march = archiving(turning, grants, places, "march");
        let mut screen = ApprovalScreen::on_an_output();
        screen.arrived(turning, march, noon());
        for scheme in [Scheme::Light, Scheme::Dark] {
            let designed = drawn(&screen, look(scheme, Direction::LeftToRight));
            let high = drawn(
                &screen,
                ApprovalLook {
                    contrast: Contrast::High,
                    ..look(scheme, Direction::LeftToRight)
                },
            );
            assert_eq!(
                designed.panel, high.panel,
                "{scheme:?}: high contrast moved the panel"
            );
            assert_eq!(
                designed
                    .answers
                    .iter()
                    .map(|answer| answer.area)
                    .collect::<Vec<_>>(),
                high.answers
                    .iter()
                    .map(|answer| answer.area)
                    .collect::<Vec<_>>(),
                "{scheme:?}: high contrast moved an answer"
            );
            let palette = crate::access_contrast::every_colour_of(
                Contrast::High,
                scheme,
                Token::Terracotta.colour(),
            );
            for solid in &high.solids {
                assert!(
                    palette.contains(&solid.colour),
                    "{scheme:?}: {:?} is drawn in {:?}, which is in no palette this crate may use",
                    solid.area,
                    solid.colour
                );
            }
        }
    });
}
