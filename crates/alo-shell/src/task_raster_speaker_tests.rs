//! **Every message in a task conversation carries its speaker's label** — *You*
//! above each thing the person said, *alo* above each thing alo replied, in
//! alo's teal — and an account of what was done carries none.
//!
//! The design draws both in every state of the window (`417:27689`, the
//! conversation of `417:27666` *Working*; `docs/design/the-ask-alo-task-window.md`,
//! *Typography* and *Teal is conditional*): *You* inside the person's bubble,
//! above the words, and *alo* above the reply, in `accent/default` in every
//! variant because it says who is speaking. Until this file nothing drew either.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on a window that will not rasterise is the failure being reported, \
              and a formatted panic names the case it was on"
)]

use alo_tasks::WhoseBlock;
use smithay::utils::Rectangle;

use super::furniture;
use super::tests::a_look;
use crate::display_scale::DisplayScale;
use crate::painted::Inked;
use crate::task_text::{TheWordsInATaskWindow, WhatWasSaid};

/// One thing said, nothing waiting on it.
const fn said(whose: WhoseBlock, words: &'static str) -> WhatWasSaid<'static> {
    WhatWasSaid {
        whose,
        words,
        marks: &[],
        answers: None,
    }
}

/// A conversation of two turns each, and an account of what was done.
fn a_conversation() -> Vec<WhatWasSaid<'static>> {
    vec![
        said(
            WhoseBlock::ThePersons,
            "Make a six-slide launch presentation from these two files.",
        ),
        said(
            WhoseBlock::Alos,
            "I\u{2019}ll read the two files, shape the story, and build a draft.",
        ),
        said(WhoseBlock::ThePersons, "Keep it clear and concise."),
        said(WhoseBlock::Alos, "Here is the draft, in six slides."),
        said(WhoseBlock::AnAccount, "Reading your documents"),
    ]
}

/// Whether `kind` is a message, which carries a speaker's label, or an account
/// of what was done, which does not. Every kind named with no catch-all.
const fn is_a_message(kind: WhoseBlock) -> bool {
    match kind {
        WhoseBlock::ThePersons | WhoseBlock::Alos => true,
        WhoseBlock::AnAccount => false,
    }
}

/// The window with only the conversation and the two labels to say, so every
/// piece of ink is a block's words or a label.
fn drawn(said: &[WhatWasSaid<'_>], scale: DisplayScale) -> Vec<Inked> {
    let words = TheWordsInATaskWindow {
        title: "",
        kind: "",
        status: "",
        alo_is_working: false,
        model: "",
        destination: "",
        invitation: "",
        scope_note: "",
        files: &[],
        how_many_files: "",
        you: "You",
        alo: "alo",
    };
    let per_cent = i32::from(scale.per_cent());
    furniture(
        Rectangle::new(
            (0, 0).into(),
            (1040 * per_cent / 100, 800 * per_cent / 100).into(),
        ),
        a_look(),
        0,
        &words,
        said,
        alo_tasks::TheScroll::at_the_newest(),
        alo_tasks::TheOneButton::Stop,
        false,
        &mut crate::WindowControlLabels::new()
            .expect("the shell's own fonts")
            .fonts,
        scale,
    )
    .expect("the design's size")
    .inked
}

/// Whether any pixel of `ink` is exactly `colour`.
fn has(ink: &Inked, colour: [u8; 3]) -> bool {
    ink.pixels.contains(&colour)
}

/// **The rule**: each message has a label of its own, at the left of its words
/// and above them, narrower than they are — alo's in teal, the person's not —
/// and an account has none.
#[test]
fn every_message_in_a_task_conversation_carries_its_speakers_label() {
    let teal = crate::task_look::TaskPalette::of(a_look())
        .expect("a palette")
        .alo;
    let said = a_conversation();
    let messages = said.iter().filter(|one| is_a_message(one.whose)).count();

    for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
        let inked = drawn(&said, scale);
        let mut labels_tall: Vec<(WhoseBlock, i32)> = Vec::new();
        assert_eq!(
            inked.len(),
            said.len() + messages,
            "at {scale:?}: {} things said, {messages} of them messages, so {} pieces of ink, \
             and {} were drawn",
            said.len(),
            said.len() + messages,
            inked.len()
        );

        // **Each piece of ink, grouped by where it starts across**: a label
        // starts where its own message's words start, and the three kinds of
        // block start in three different places.
        let mut by_where: Vec<(i32, Vec<&Inked>)> = Vec::new();
        for one in &inked {
            let x = one.area.loc.x;
            match by_where.iter_mut().find(|(at, _)| (*at - x).abs() <= 2) {
                Some((_, group)) => group.push(one),
                None => by_where.push((x, vec![one])),
            }
        }
        for (_, group) in &mut by_where {
            group.sort_by_key(|one| one.area.loc.y);
        }

        for kind in [
            WhoseBlock::ThePersons,
            WhoseBlock::Alos,
            WhoseBlock::AnAccount,
        ] {
            let how_many = said.iter().filter(|one| one.whose == kind).count();
            let wanted = if is_a_message(kind) {
                how_many * 2
            } else {
                how_many
            };
            let group = by_where
                .iter()
                .map(|(_, group)| group)
                .find(|group| group.len() == wanted && wanted > 0)
                .unwrap_or_else(|| {
                    panic!(
                        "at {scale:?}: no column of {wanted} pieces for {kind:?}; the columns \
                         hold {:?}",
                        by_where
                            .iter()
                            .map(|(_, group)| group.len())
                            .collect::<Vec<_>>()
                    )
                });
            if !is_a_message(kind) {
                assert!(
                    group.iter().all(|one| !has(one, teal)),
                    "at {scale:?}: an account of what was done carries alo's label"
                );
                continue;
            }
            for pair in group.chunks(2) {
                let [label, words] = pair else {
                    panic!("at {scale:?}: {kind:?}'s column does not pair up");
                };
                assert!(
                    label.area.loc.y + label.area.size.h <= words.area.loc.y,
                    "at {scale:?}: {kind:?}'s label at {:?} is not above its words at {:?}",
                    label.area,
                    words.area
                );
                assert!(
                    label.area.size.w < words.area.size.w,
                    "at {scale:?}: {kind:?}'s first piece at {:?} is as wide as its words at \
                     {:?}, so it is not a label",
                    label.area,
                    words.area
                );
                assert_eq!(
                    has(label, teal),
                    kind == WhoseBlock::Alos,
                    "at {scale:?}: {kind:?}'s label is {}teal, and only alo's is",
                    if has(label, teal) { "" } else { "not " }
                );
                assert!(
                    !has(words, teal),
                    "at {scale:?}: {kind:?}'s words are drawn in teal"
                );
                // **The room under it is the design's**: 6 under *You*, 8
                // under *alo*, measured on `417:27691` and `417:27694`.
                let under = scale.px(if kind == WhoseBlock::Alos {
                    alo_tasks::measures::UNDER_ALOS_LABEL
                } else {
                    alo_tasks::measures::UNDER_YOUR_LABEL
                });
                assert_eq!(
                    words.area.loc.y - (label.area.loc.y + label.area.size.h),
                    under,
                    "at {scale:?}: {kind:?}'s label at {:?} and its words at {:?} are not the \
                     design's room apart",
                    label.area,
                    words.area
                );
                labels_tall.push((kind, label.area.size.h));
            }
        }
        // **And the two labels are the two styles**: *You* in Label/Small,
        // shorter than *alo* in Label/Medium.
        let tallest = |of: WhoseBlock| {
            labels_tall
                .iter()
                .filter(|(kind, _)| *kind == of)
                .map(|(_, tall)| *tall)
                .max()
                .unwrap_or(0)
        };
        assert!(
            tallest(WhoseBlock::ThePersons) < tallest(WhoseBlock::Alos),
            "at {scale:?}: You is {} tall and alo {}, so they are not Label/Small and \
             Label/Medium",
            tallest(WhoseBlock::ThePersons),
            tallest(WhoseBlock::Alos)
        );
    }
}
