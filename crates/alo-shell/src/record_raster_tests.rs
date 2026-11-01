//! The laid-out record window, read as text, boxes and pixels: what is in it,
//! in which order, where the approved frames put it, and what never is.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use super::*;
use crate::Contrast;
use crate::record_testing::{a_clock, a_long_record, an_afternoon_kept, light, words};
use crate::settings_type::{TheType, written};
use crate::{RecordKey, RecordOpened, RecordWindow};
use alo_appearance::{Scheme, TextScale};
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
    picture(
        window.shows(),
        &words(),
        &mut labels,
        size,
        look,
        a_clock(),
        crate::display_scale::DisplayScale::ONE,
    )
    .unwrap()
}

/// The window as it stands, on the approved frame's 1440 × 960.
fn drawn(window: &RecordWindow) -> RecordPicture {
    drawn_at(window, (1440, 960), light())
}

/// **Every entry of `kept`, as the window draws it**, newest first: the view
/// walked down the record one entry at a time, since no view holds a whole
/// afternoon.
fn every_entry(kept: &crate::record_testing::Kept) -> Vec<EntryDrawn> {
    let mut window = opened(kept);
    let recounting = kept.recounting();
    let mut all = Vec::new();
    loop {
        let picture = drawn(&window);
        if !picture.older_out_of_view {
            all.extend(picture.entries);
            return all;
        }
        all.push(picture.entries[0].clone());
        let _ = window.pressed(RecordKey::Older, &recounting);
    }
}

/// The account in the window.
fn account_of(window: &RecordWindow) -> &alo_recounting::Account {
    match window.shows() {
        RecordShows::Account { account, .. } => account,
        other => panic!("no account: {other:?}"),
    }
}

/// The colour at (`x`, `y`) on the window's raster.
fn colour_at(picture: &RecordPicture, x: i32, y: i32) -> [u8; 4] {
    let washed = picture.washed.as_ref().unwrap();
    let area = washed.area;
    let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4).unwrap();
    [
        washed.pixels[at],
        washed.pixels[at + 1],
        washed.pixels[at + 2],
        washed.pixels[at + 3],
    ]
}

/// **Every clause is the vocabulary's, not this crate's.** Each entry's head is
/// exactly what `alo_recounting::Outcome::said` answers from the machine's one
/// vocabulary, and one of the clauses `alo-recounting` declares — so no clause
/// on this window was written here.
#[test]
fn every_clause_drawn_is_the_vocabularys_and_none_is_this_crates() {
    let kept = an_afternoon_kept();
    let strings = words();
    let declared: Vec<&str> = EVERY_OUTCOME.iter().map(|word| word.says()).collect();
    let entries = every_entry(&kept);
    assert_eq!(entries.len(), 8, "{entries:#?}");
    for entry in &entries {
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
    let window = opened(&kept);
    let picture = drawn(&window);
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

/// **Newest first, in the order the record has them**, and inside one view each
/// entry below the last: its head row — the agent's name, then the clause —
/// over its lines, over its marks.
#[test]
fn entries_are_drawn_newest_first_in_the_order_the_record_has_them() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let in_the_record: Vec<Outcome> = account_of(&window)
        .told()
        .iter()
        .rev()
        .map(|told| told.outcome())
        .collect();
    let down_the_window: Vec<Outcome> = every_entry(&kept)
        .iter()
        .map(|entry| entry.words.outcome)
        .collect();
    assert_eq!(down_the_window, in_the_record);
    assert_eq!(down_the_window.first(), Some(&Outcome::LeftOnItsOwn));

    let picture = drawn(&window);
    let mut last_bottom = picture
        .counts
        .last()
        .map_or(0, |count| count.area.loc.y + count.area.size.h);
    for entry in &picture.entries {
        assert!(entry.area.loc.y >= last_bottom, "{entry:#?}");
        if let Some(agent) = &entry.agent {
            assert!(agent.area.loc.x < entry.clause.area.loc.x, "{entry:#?}");
        }
        let mut below = entry.clause.area.loc.y + entry.clause.area.size.h;
        for text in entry
            .after
            .iter()
            .chain(entry.approved.iter())
            .chain(entry.refused.iter())
        {
            assert!(text.area.loc.y >= below, "{entry:#?}");
            below = text.area.loc.y + text.area.size.h;
        }
        assert!(below <= entry.area.loc.y + entry.area.size.h, "{entry:#?}");
        last_bottom = entry.area.loc.y + entry.area.size.h;
    }
}

/// **What was refused is drawn as plainly as what ran.** Every clause —
/// carried out, declined, refused before anybody was asked, turned away, held
/// back — is exactly the pixels of that clause in the window's one style and
/// one ink at its width. A refusal adds the word *Refused* under its lines;
/// nothing about it is dimmer, smaller or folded away.
#[test]
fn what_was_refused_is_drawn_as_plainly_as_what_ran() {
    let kept = an_afternoon_kept();
    let mut window = opened(&kept);
    let recounting = kept.recounting();
    let palette = crate::record_palette::RecordPalette::of(Scheme::Light, Contrast::AsDesigned);
    let mut labels = WindowControlLabels::new().unwrap();
    let measure = crate::desktop_look::Measure::of(TextScale::ordinary());
    let mut seen = Vec::new();
    loop {
        let picture = drawn(&window);
        for entry in &picture.entries {
            let alone = written(
                &mut labels.fonts,
                &entry.clause.text,
                TheType::Name,
                entry.clause.area.size.w,
                measure,
                (palette.surface, palette.ink),
            );
            let area = entry.clause.area;
            for (at, expected) in alone.pixels.iter().enumerate().step_by(7) {
                let at = i32::try_from(at).unwrap();
                let (x, y) = (area.loc.x + at % area.size.w, area.loc.y + at / area.size.w);
                let [r, g, b, _] = colour_at(&picture, x, y);
                assert_eq!(
                    [r, g, b],
                    *expected,
                    "{:?} is drawn differently at ({x}, {y})",
                    entry.words.outcome
                );
            }
            let refused = EntryKind::of(entry.words.outcome) == EntryKind::Refused;
            assert_eq!(
                entry.refused.as_ref().map(|text| text.text.as_str()),
                refused.then_some("Refused"),
                "{:?}",
                entry.words.outcome
            );
            seen.push(entry.words.outcome);
        }
        if !picture.older_out_of_view {
            break;
        }
        let _ = window.pressed(RecordKey::Older, &recounting);
    }
    for refused in [
        Outcome::ThePersonSaidNo,
        Outcome::NeverBecameACall,
        Outcome::HeldBack,
        Outcome::Ran,
    ] {
        assert!(
            seen.contains(&refused),
            "{refused:?} is not drawn: {seen:?}"
        );
    }
}

/// **An entry that names no agent is drawn without one.** alo OS's own errand
/// has its clause and where it went, and no name; entries an agent caused carry
/// the record's name for it, in the agent's colour.
#[test]
fn an_entry_that_names_no_agent_is_drawn_without_inventing_one() {
    let kept = an_afternoon_kept();
    let entries = every_entry(&kept);
    let errand = entries
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
    for entry in entries
        .iter()
        .filter(|entry| entry.words.outcome != Outcome::LeftOnItsOwn)
    {
        let name = entry.agent.as_ref().map(|agent| agent.text.as_str());
        assert!(matches!(name, Some("@files" | "@mail")), "{entry:#?}");
    }
}

/// **Teal is the agent's name and nothing else.** The approved design draws
/// `accent/default` in one place — the name of the agent an entry was done
/// under, which is the agent acting — so every pixel exactly that colour is in
/// an entry's head row that names one.
#[test]
fn teal_is_only_the_agents_name() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    for scheme in [Scheme::Light, Scheme::Dark] {
        let picture = drawn_at(&window, (1440, 960), RecordLook { scheme, ..light() });
        let teal = crate::record_palette::RecordPalette::of(scheme, Contrast::AsDesigned).agent;
        let washed = picture.washed.as_ref().unwrap();
        let mut found = 0;
        for (at, pixel) in washed.pixels.chunks_exact(4).enumerate() {
            if pixel[..3] != teal[..] {
                continue;
            }
            found += 1;
            let at = i32::try_from(at).unwrap();
            let (x, y) = (
                washed.area.loc.x + at % washed.area.size.w,
                washed.area.loc.y + at / washed.area.size.w,
            );
            let in_a_named_head = picture.entries.iter().any(|entry| {
                entry.agent.is_some()
                    && entry.area.contains((x, y))
                    && y < entry.clause.area.loc.y + 22
                    && x < entry.clause.area.loc.x
            });
            assert!(in_a_named_head, "{scheme:?}: teal at ({x}, {y})");
        }
        assert!(found > 0, "{scheme:?}: no agent's name was drawn in teal");
    }
}

/// **The machine's own words, as the record kept them.** The lines after a
/// clause are the record's sentence, the refusal's own reason and `alo-egress`'s
/// one rendering of where something went; that the person said yes is the
/// vocabulary's, under them.
#[test]
fn the_machines_own_words_are_drawn_as_the_record_kept_them() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let strings = words();
    let account = account_of(&window);
    let entries = every_entry(&kept);
    for (entry, told) in entries.iter().zip(account.told().iter().rev()) {
        let mut expected: Vec<String> = Vec::new();
        expected.extend(told.sentence().map(|line| line.as_str().to_owned()));
        expected.extend(told.because().map(|line| line.as_str().to_owned()));
        expected.extend(told.went_to().map(|to| to.shown(&strings)));
        let drawn: Vec<String> = entry.after.iter().map(|text| text.text.clone()).collect();
        assert_eq!(drawn, expected, "{:?}", told.outcome());
        assert_eq!(
            entry.approved.as_ref().map(|text| text.text.clone()),
            told.approved().map(|yes| yes.said(&strings).into_text()),
            "{:?}",
            told.outcome()
        );
    }
    let ran = entries
        .iter()
        .find(|entry| entry.words.outcome == Outcome::Ran)
        .unwrap();
    assert!(ran.after[0].text.contains("march.pdf"), "{ran:#?}");
}

/// **What ran says who said yes to it, and nothing refused does.** Demo 1's
/// fourteenth step opens the record on an approved rename and asks who approved
/// it; the mark beneath the sentence is the vocabulary's answer.
#[test]
fn what_ran_says_the_person_said_yes_and_nothing_refused_does() {
    let kept = an_afternoon_kept();
    let entries = every_entry(&kept);
    let yes = "the person at this machine was shown this and said yes before it happened";
    for entry in &entries {
        let said_yes = entry.approved.as_ref().map(|text| text.text.as_str());
        if entry.words.outcome == Outcome::Ran {
            assert_eq!(said_yes, Some(yes), "{entry:#?}");
        } else {
            assert_eq!(said_yes, None, "{:?}", entry.words.outcome);
        }
    }
    let declined = entries
        .iter()
        .find(|entry| entry.words.outcome == Outcome::ThePersonSaidNo)
        .unwrap();
    assert_eq!(
        declined.after.len(),
        1,
        "only what they declined: {declined:#?}"
    );
}

/// **What left says it is already sent**, at the end of its head row, and
/// nothing else does.
#[test]
fn what_left_says_it_is_already_sent() {
    let kept = an_afternoon_kept();
    for entry in every_entry(&kept) {
        let left = EntryKind::of(entry.words.outcome) == EntryKind::Left;
        let sent = entry.already_sent.as_ref();
        assert_eq!(
            sent.map(|text| text.text.as_str()),
            left.then_some("Already sent"),
            "{:?}",
            entry.words.outcome
        );
        if let Some(sent) = sent {
            assert!(sent.area.loc.x > entry.clause.area.loc.x);
            assert_eq!(
                sent.area.loc.x + sent.area.size.w,
                entry.area.loc.x + entry.area.size.w - 20
            );
        }
    }
}

/// **Today's counts are the record's, in the design's words**: the change
/// approved, the two things that left, the four refused.
#[test]
fn todays_counts_are_drawn_in_the_designs_words() {
    let kept = an_afternoon_kept();
    let picture = drawn(&opened(&kept));
    let counts: Vec<&str> = picture
        .counts
        .iter()
        .map(|count| count.text.as_str())
        .collect();
    assert_eq!(
        counts,
        [
            "1 change you allowed",
            "2 requests left this computer",
            "4 refused"
        ]
    );
}

/// **Where the approved frame puts things, at 1440 × 960**: the window at
/// (280, 56), 880 × 796; Close at its top right; the first day's card 40 in,
/// 800 wide; its first entry's time 20 into it, the icon's tile at 80, the
/// words at 128.
#[test]
fn at_1440_it_is_where_the_frame_draws_it() {
    let kept = an_afternoon_kept();
    let picture = drawn(&opened(&kept));
    assert_eq!(
        picture.panel,
        Some(Rectangle::new((280, 56).into(), (880, 796).into()))
    );
    assert_eq!(
        picture.close,
        Some(Rectangle::new((1112, 56).into(), (44, 44).into()))
    );
    let card = picture.cards[0];
    assert_eq!((card.loc.x, card.size.w), (320, 800));
    let first = &picture.entries[0];
    assert_eq!(first.area.loc.x, 320);
    assert_eq!(first.time.as_ref().unwrap().area.loc.x, 340);
    assert_eq!(first.icon.loc.x, 400);
    assert_eq!((first.icon.size.w, first.icon.size.h), (32, 32));
    // The agent's mark (12) and 6, the name and 6, where there is a name.
    let named = first
        .agent
        .as_ref()
        .map_or(0, |name| 12 + 6 + name.area.size.w + 6);
    assert_eq!(first.clause.area.loc.x, 448 + named);
    let day = first.day.as_ref().unwrap();
    assert_eq!(day.area.loc.x, 324);
    assert_eq!(
        day.area.loc.y + 28,
        card.loc.y,
        "18 tall and 10 above the card"
    );
}

/// **The record's sentences about itself stay above every view**, and a view
/// that does not hold every entry says where it is with a rail — a shape, not
/// a sentence nobody declared. Every entry in view is whole, above the footer.
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
    let footer_top = panel.loc.y + panel.size.h - 52;
    for entry in &top.entries {
        assert!(entry.area.loc.y + entry.area.size.h <= footer_top - 24);
    }

    let _ = window.pressed(RecordKey::Oldest, &recounting);
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
                picture(
                    window.shows(),
                    &strings,
                    &mut labels,
                    size,
                    light(),
                    a_clock(),
                    crate::display_scale::DisplayScale::ONE,
                ),
                Err(RenderError::RecordScene)
            ),
            "{size:?}"
        );
    }
    let drawn = picture(
        window.shows(),
        &strings,
        &mut labels,
        (1920, 1080),
        light(),
        a_clock(),
        crate::display_scale::DisplayScale::ONE,
    )
    .unwrap();
    assert!(drawn.validate((1920, 1080).into()).is_ok());
    assert!(matches!(
        drawn.validate((1280, 720).into()),
        Err(RenderError::RecordScene)
    ));
}

/// **A closed window draws nothing**, and a refusal is drawn as the refuser's
/// sentence under the heading — no entries, no list, no remark, no count.
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
    assert!(picture.counts.is_empty() && picture.cards.is_empty());
    assert!(!picture.is_empty());
}

/// **Everything starts from the side the person starts reading at**: the rail
/// is on the trailing edge, and the time leads the clause in either direction.
#[test]
fn the_rail_and_the_time_follow_the_reading_direction() {
    let kept = a_long_record(80);
    let window = opened(&kept);
    let rtl = RecordLook {
        reading: Direction::RightToLeft,
        ..light()
    };
    let left_to_right = drawn(&window);
    let right_to_left = drawn_at(&window, (1440, 960), rtl);
    let (ltr_track, _) = left_to_right.rail.unwrap();
    let (rtl_track, _) = right_to_left.rail.unwrap();
    assert!(ltr_track.loc.x > left_to_right.entries[0].clause.area.loc.x);
    assert!(rtl_track.loc.x < right_to_left.entries[0].clause.area.loc.x);

    let ltr = &left_to_right.entries[0];
    assert!(ltr.time.as_ref().unwrap().area.loc.x < ltr.clause.area.loc.x);
    let rtl = &right_to_left.entries[0];
    assert!(rtl.time.as_ref().unwrap().area.loc.x > rtl.clause.area.loc.x);
    assert_eq!(left_to_right.panel, right_to_left.panel);
}

/// **At 200 per cent the record is drawn twice the size, in twice the
/// place**, on a display with twice the pixels each way: the window exactly,
/// the same remarks and entries in the same order, and each clause's box about
/// twice as large and as far from the corner.
///
/// *About*, because a line of type drawn twice the size is not exactly twice
/// the pixels of its box: a glyph's edge rounds to a pixel either way.
#[test]
fn at_two_hundred_the_record_is_drawn_twice_the_size() {
    let kept = an_afternoon_kept();
    let window = opened(&kept);
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let mut at = |size: (i32, i32), per_cent: u16| {
        picture(
            window.shows(),
            &strings,
            &mut labels,
            size,
            light(),
            a_clock(),
            crate::display_scale::DisplayScale::of_per_cent(per_cent),
        )
        .unwrap()
    };
    let sparse = at((1440, 960), 100);
    let dense = at((2880, 1920), 200);

    let (one, two) = (sparse.panel.unwrap(), dense.panel.unwrap());
    assert_eq!(
        (2 * one.loc.x, 2 * one.loc.y, 2 * one.size.w, 2 * one.size.h),
        (two.loc.x, two.loc.y, two.size.w, two.size.h),
        "the window is not exactly twice the size and place"
    );
    let said = |picture: &RecordPicture| -> Vec<String> {
        picture
            .remarks
            .iter()
            .map(|remark| remark.text.clone())
            .chain(
                picture
                    .entries
                    .iter()
                    .map(|entry| entry.clause.text.clone()),
            )
            .collect()
    };
    assert_eq!(said(&sparse), said(&dense), "different words at 200");

    let about_twice = |at_one: i32, at_two: i32| (at_two - 2 * at_one).abs() <= 4 + at_one / 5;
    for (small, large) in sparse.entries.iter().zip(&dense.entries) {
        let (a, b) = (small.clause.area, large.clause.area);
        assert!(
            about_twice(a.loc.x, b.loc.x)
                && about_twice(a.loc.y, b.loc.y)
                && about_twice(a.size.w, b.size.w)
                && about_twice(a.size.h, b.size.h),
            "{:?} is at {a:?} at 100 and {b:?} at 200, not twice",
            small.clause.text
        );
    }
}
