//! The laid-out record window, read as text, boxes and pixels: what is in it,
//! in which order, and what never is.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use super::*;
use crate::Contrast;
use crate::painted_text::sentence as shaped_sentence;
use crate::record_room::{Measure, Palette};
use crate::record_testing::{a_long_record, an_afternoon_kept, light, words};
use crate::{RecordKey, RecordOpened, RecordWindow};
use alo_appearance::{Scheme, TextScale, Token};
use alo_recounting::Outcome;
use alo_recounting::words::EVERY_OUTCOME;

/// A window opened by hand on `kept`.
fn opened(kept: &crate::record_testing::Kept) -> RecordWindow {
    let mut window = RecordWindow::on_an_output();
    assert_eq!(
        window.opened_by_hand(&kept.recounting()),
        RecordOpened::Shown
    );
    window
}

/// The window as it stands, drawn on an output of `size`.
fn drawn_at(window: &RecordWindow, size: (i32, i32), look: RecordLook) -> RecordPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(window.shows(), &words(), &mut labels, size, look).unwrap()
}

/// The window as it stands, drawn on a tall 1920×2160 output, which holds a
/// whole afternoon.
fn drawn(window: &RecordWindow) -> RecordPicture {
    drawn_at(window, (1920, 2160), light())
}

/// The account in the window.
fn account_of(window: &RecordWindow) -> &alo_recounting::Account {
    match window.shows() {
        RecordShows::Account { account, .. } => account,
        other => panic!("no account: {other:?}"),
    }
}

/// **Every clause is the vocabulary's, not this crate's.** Each entry's head is
/// exactly what `alo_recounting::Outcome::said` answers from the machine's one
/// vocabulary, declared, and one of the clauses `alo-recounting` declares — so
/// no clause on this window was written here.
#[test]
fn every_clause_drawn_is_the_vocabularys_and_none_is_this_crates() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let picture = drawn(&window);
    let strings = words();
    let declared: Vec<&str> = EVERY_OUTCOME.iter().map(|word| word.says()).collect();

    assert!(picture.entries.len() >= 7, "{:#?}", picture.entries);
    for entry in &picture.entries {
        let said = entry.words.outcome.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(entry.clause.text, said.text());
        assert!(
            declared.contains(&entry.clause.text.as_str()),
            "{} is not a clause alo-recounting declares",
            entry.clause.text
        );
    }

    // And what the record says about itself is its own sentences, in order.
    let remarks: Vec<String> = account_of(&window)
        .said(&strings)
        .into_iter()
        .map(alo_strings::Said::into_text)
        .collect();
    let drawn_remarks: Vec<String> = picture
        .remarks
        .iter()
        .map(|text| text.text.clone())
        .collect();
    assert_eq!(drawn_remarks, remarks);
    assert!(!remarks.is_empty());
}

/// **Newest first, in the order the record has them**: the entries down the
/// window are the record's entries reversed, one under another, each clause at
/// the head of its entry and everything else after it.
#[test]
fn entries_are_drawn_newest_first_in_the_order_the_record_has_them() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let picture = drawn(&window);
    let account = account_of(&window);

    let in_the_record: Vec<Outcome> = account
        .told()
        .iter()
        .rev()
        .map(|told| told.outcome())
        .collect();
    let down_the_window: Vec<Outcome> = picture
        .entries
        .iter()
        .map(|entry| entry.words.outcome)
        .collect();
    assert_eq!(down_the_window, in_the_record);
    assert_eq!(
        picture.entries.first().map(|entry| entry.words.outcome),
        Some(Outcome::LeftOnItsOwn)
    );

    let mut last_bottom = picture
        .remarks
        .last()
        .map_or(0, |remark| remark.area.loc.y + remark.area.size.h);
    for entry in &picture.entries {
        assert!(entry.clause.area.loc.y >= last_bottom, "{entry:#?}");
        let mut below = entry.clause.area.loc.y + entry.clause.area.size.h;
        for text in entry.agent.iter().chain(entry.after.iter()) {
            assert!(text.area.loc.y >= below, "{entry:#?}");
            below = text.area.loc.y + text.area.size.h;
        }
        last_bottom = below;
    }
    assert!(!picture.older_out_of_view && !picture.more_recent_out_of_view);
    assert!(picture.rail.is_none());
}

/// **What was refused is drawn as plainly as what ran.** Every clause —
/// carried out, declined, refused before anybody was asked, turned away, held
/// back — is inked as exactly the pixels of that clause in the window's one
/// type and two colours at the one width, and every entry's words after it are
/// set in the same way. There is no dimmer, smaller or folded style for a
/// refusal.
#[test]
fn what_was_refused_is_drawn_as_plainly_as_what_ran() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let picture = drawn(&window);
    let palette = Palette::of(Scheme::Light, Contrast::AsDesigned);
    let mut labels = WindowControlLabels::new().unwrap();

    let outcomes: Vec<Outcome> = picture
        .entries
        .iter()
        .map(|entry| entry.words.outcome)
        .collect();
    for refused in [
        Outcome::ThePersonSaidNo,
        Outcome::NeverBecameACall,
        Outcome::HeldBack,
    ] {
        assert!(
            outcomes.contains(&refused),
            "{refused:?} is not drawn: {outcomes:?}"
        );
    }
    assert!(outcomes.contains(&Outcome::Ran));

    let clause_x = picture.entries[0].clause.area.loc.x;
    let clause_width = picture.entries[0].clause.area.size.w;
    let after_x = picture.entries[0].after[0].area.loc.x;
    for entry in &picture.entries {
        assert_eq!(entry.clause.area.loc.x, clause_x, "{entry:#?}");
        assert_eq!(entry.clause.area.size.w, clause_width, "{entry:#?}");
        for text in entry.agent.iter().chain(entry.after.iter()) {
            assert_eq!(text.area.loc.x, after_x, "{entry:#?}");
        }
        let alone = shaped_sentence(
            &mut labels.fonts,
            &entry.clause.text,
            clause_width,
            Measure::of(TextScale::ordinary()).metrics(),
            palette.ground,
            palette.ink,
        );
        let inked = picture
            .inked
            .iter()
            .find(|inked| inked.area == entry.clause.area)
            .expect("a clause was not inked where it was laid out");
        assert_eq!(
            inked.pixels, alone.pixels,
            "{:?} is drawn differently",
            entry.words.outcome
        );
    }
    for solid in &picture.solids {
        assert!(
            solid.colour == palette.ink || solid.colour == palette.ground,
            "a third colour: {solid:?}"
        );
    }
}

/// **An entry that names no agent is drawn without one.** alo OS's own errand
/// has its clause and where it went, and nothing in the place a name would be;
/// entries an agent caused carry the record's name for it.
#[test]
fn an_entry_that_names_no_agent_is_drawn_without_inventing_one() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let picture = drawn(&window);

    let errand = picture
        .entries
        .iter()
        .find(|entry| entry.words.outcome == Outcome::LeftOnItsOwn)
        .expect("alo OS's own errand is not drawn");
    assert_eq!(errand.agent, None);
    assert_eq!(errand.words.agent, None);
    assert_eq!(
        errand
            .after
            .iter()
            .map(|text| text.text.as_str())
            .collect::<Vec<_>>(),
        ["models.alo.example"]
    );
    let inked_in_it = picture
        .inked
        .iter()
        .filter(|inked| {
            inked.area.loc.y >= errand.clause.area.loc.y
                && inked.area.loc.y < errand.after[0].area.loc.y + errand.after[0].area.size.h
        })
        .count();
    assert_eq!(inked_in_it, 2, "something was drawn where a name would be");

    for entry in picture
        .entries
        .iter()
        .filter(|entry| entry.words.outcome != Outcome::LeftOnItsOwn)
    {
        let name = entry.agent.as_ref().map(|agent| agent.text.as_str());
        assert!(matches!(name, Some("@files" | "@mail")), "{entry:#?}");
    }
}

/// **The machine's own words, as the record kept them.** The sentence after a
/// carried-out change is byte for byte the record's; a refusal's reason is the
/// refusal's own; where something went is `alo-egress`'s one rendering.
#[test]
fn the_machines_own_words_are_drawn_as_the_record_kept_them() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let picture = drawn(&window);
    let strings = words();
    let account = account_of(&window);

    for (entry, told) in picture.entries.iter().zip(account.told().iter().rev()) {
        let mut expected: Vec<String> = Vec::new();
        expected.extend(told.sentence().map(|line| line.as_str().to_owned()));
        expected.extend(told.because().map(|line| line.as_str().to_owned()));
        expected.extend(told.went_to().map(|to| to.shown(&strings)));
        let drawn: Vec<String> = entry.after.iter().map(|text| text.text.clone()).collect();
        assert_eq!(drawn, expected, "{:?}", told.outcome());
    }
    let ran = picture
        .entries
        .iter()
        .find(|entry| entry.words.outcome == Outcome::Ran)
        .unwrap();
    assert!(ran.after[0].text.contains("march.pdf"), "{ran:#?}");
}

/// **The record's sentences about itself stay above every view**, and a view
/// that does not hold every entry says where it is with a rail — a shape, not
/// a sentence nobody declared. Every entry in view is whole.
#[test]
fn the_records_sentences_stay_above_every_view_and_no_entry_is_cut() {
    let kept = a_long_record(80);
    let mut window = opened(&kept);
    let recounting = kept.recounting();

    let top = drawn(&window);
    assert!(top.older_out_of_view && !top.more_recent_out_of_view);
    let (track, thumb) = top.rail.unwrap();
    assert_eq!(thumb.loc.y, track.loc.y);
    let panel = top.panel.unwrap();
    for entry in &top.entries {
        let last = entry.after.last().unwrap_or(&entry.clause);
        assert!(last.area.loc.y + last.area.size.h <= panel.loc.y + panel.size.h);
    }

    window.pressed(RecordKey::Oldest, &recounting);
    let bottom = drawn(&window);
    assert!(bottom.more_recent_out_of_view && !bottom.older_out_of_view);
    assert_eq!(
        bottom.remarks, top.remarks,
        "the record's own sentences moved with the view"
    );
    assert_eq!(bottom.entries.len(), 1);
    let (_, low_thumb) = bottom.rail.unwrap();
    assert!(low_thumb.loc.y > thumb.loc.y);
}

/// **A window that cannot hold an entry whole refuses the frame**, as does an
/// output larger than any the window is laid out for, and a picture laid out
/// for one output is refused on another.
#[test]
fn a_window_that_cannot_hold_an_entry_whole_refuses_rather_than_cutting() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    for size in [(1920, 90), (150, 1080), (20_000, 1080), (1920, 20_000)] {
        assert!(
            matches!(
                picture(window.shows(), &strings, &mut labels, size, light()),
                Err(RenderError::RecordScene)
            ),
            "{size:?}"
        );
    }
    let drawn = picture(window.shows(), &strings, &mut labels, (1920, 1080), light()).unwrap();
    assert!(drawn.validate((1920, 1080).into()).is_ok());
    assert!(matches!(
        drawn.validate((1280, 720).into()),
        Err(RenderError::RecordScene)
    ));
}

/// **A closed window draws nothing**, and a refusal is drawn as the refuser's
/// sentence and nothing else — no entries, no list, no remark.
#[test]
fn a_closed_window_draws_nothing_and_a_refusal_draws_only_its_sentence() {
    let window = RecordWindow::on_an_output();
    assert!(drawn(&window).is_empty());

    let kept = an_afternoon_kept();
    std::fs::remove_file(&kept.at).unwrap();
    let mut refused = RecordWindow::on_an_output();
    refused.opened_by_hand(&kept.recounting());
    let picture = drawn(&refused);
    let RecordShows::Refusal(why) = refused.shows() else {
        panic!("not a refusal");
    };
    assert_eq!(
        picture.refusal.as_ref().map(|text| text.text.as_str()),
        Some(why.said(&words()).text())
    );
    assert!(picture.entries.is_empty() && picture.remarks.is_empty());
    assert_eq!(picture.inked.len(), 1);
}

/// **The rail is on the trailing edge for the way the person reads**, and the
/// words after a clause are set in from the side they start reading at.
#[test]
fn the_rail_and_the_indent_follow_the_reading_direction() {
    let kept = a_long_record(80);
    let window = opened(&kept);
    let rtl = RecordLook {
        contrast: Contrast::AsDesigned,
        reading: Direction::RightToLeft,
        ..light()
    };
    let left_to_right = drawn(&window);
    let right_to_left = drawn_at(&window, (1920, 1080), rtl);
    let (ltr_track, _) = left_to_right.rail.unwrap();
    let (rtl_track, _) = right_to_left.rail.unwrap();
    assert!(ltr_track.loc.x > left_to_right.entries[0].clause.area.loc.x);
    assert!(rtl_track.loc.x < right_to_left.entries[0].clause.area.loc.x);

    let ltr = &left_to_right.entries[0];
    assert!(ltr.after[0].area.loc.x > ltr.clause.area.loc.x);
    let rtl = &right_to_left.entries[0];
    assert_eq!(rtl.after[0].area.loc.x, rtl.clause.area.loc.x);
    assert!(rtl.after[0].area.size.w < rtl.clause.area.size.w);
}

/// **Terracotta is never drawn**, in either scheme: it means the agent acting,
/// and this window is a person reading.
#[test]
fn terracotta_is_never_drawn() {
    let terracotta = Token::Terracotta.colour();
    let terracotta = [terracotta.red(), terracotta.green(), terracotta.blue()];
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    for scheme in [Scheme::Light, Scheme::Dark] {
        let palette = Palette::of(scheme, Contrast::AsDesigned);
        assert_ne!(palette.ground, terracotta);
        assert_ne!(palette.ink, terracotta);
        let picture = drawn_at(&window, (1920, 1080), RecordLook { scheme, ..light() });
        assert!(
            picture
                .solids
                .iter()
                .all(|solid| solid.colour != terracotta)
        );
        assert!(
            picture
                .inked
                .iter()
                .all(|inked| inked.pixels.iter().all(|pixel| *pixel != terracotta))
        );
    }
}
