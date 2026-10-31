//! The alo and AI page: where questions are answered, and what happens when
//! that place cannot be reached — frame `550:97` and its three siblings.
//!
//! # The four ways, always, with what is under each
//!
//! The four are `alo_setting_up::THE_FOUR`, in setup's order, each with the
//! name and line its crate declares. Under each is what the person's settings
//! hold: models they brought, computers they paired, providers they added.
//! **Under the providers are alo's own service and *Add a provider***, as the
//! design draws them — the owner's decision of 2026-10-31 confirming the ruling
//! of 2026-10-06: *alo's own service sits inside "With a provider you add",
//! with no exemption*.
//!
//! **alo's own service is not on this computer yet**, and nor is a way to say
//! where a provider runs. Both are drawn as the design draws them, and choosing
//! either says so in `crate::the_settings_words::NOT_ON_THIS_COMPUTER_YET`.
//! Nothing here pretends.
//!
//! # What happens when it cannot be reached
//!
//! One switch, *Let alo use another model or provider*, with what it risks in a
//! box under it — and only for somebody who chose a provider or a computer on
//! their network, because for anybody else there is nothing to cover for. **A
//! person who has never said is drawn as neither On nor Off**, because their
//! machine does neither: it answers where it can and says plainly that the
//! chosen place was not used (`docs/features.md`, 2026-10-18).

use std::time::SystemTime;

use alo_setting_up::Offered;
use alo_strings::{Filling, Strings};

use crate::settings_answering::SettingsChoice;
use crate::settings_page::{
    ABlock, ANote, ARisk, ARow, ASection, SwitchIs, TheKind, said, switch_said,
};
use crate::settings_type::as_a_name;
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings, now: SystemTime) -> Vec<ABlock> {
    let section = &open.answering;
    let mut blocks = Vec::new();
    if let Some(why) = section.not_read() {
        blocks.push(ABlock::Said(why.said(strings).into_text()));
    }
    blocks.extend(section.told().map(|it| ABlock::Said(it.to_owned())));

    let ways = section.every_way(&open.pairings(), now);
    if !ways.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::WHERE_QUESTIONS_ARE_ANSWERED)),
            rows: the_ways(open, ways, strings),
            footnote: Some(said(strings, w::EVERY_ANSWER_SAYS_WHERE)),
        }));
    }
    if section.another_is_a_question_at_all() {
        let is = match section.may_use_another() {
            Some(alo_choosing::MayUseAnother::Yes) => SwitchIs::On,
            Some(alo_choosing::MayUseAnother::No) => SwitchIs::Off,
            None => SwitchIs::NotSet,
        };
        let mut row = ARow::named(
            Some(SettingsRow::UsingAnother),
            TheKind::ASwitch(is),
            said(strings, w::LET_ALO_USE_ANOTHER),
        );
        row.lines.push(said(strings, w::WITH_IT_OFF));
        row.risk = Some(ARisk {
            heading: said(strings, w::WHAT_IT_RISKS),
            words: said(strings, w::ANOTHER_RISKS),
        });
        row.at_its_end = Some(switch_said(is, strings));
        blocks.push(ABlock::Section(ASection {
            heading: Some(said(strings, w::WHEN_IT_IS_NOT_AVAILABLE)),
            rows: vec![row],
            footnote: None,
        }));
    }
    blocks
}

/// The four ways, each with what is under it.
fn the_ways(
    open: &Open,
    ways: Vec<(Offered, Vec<SettingsChoice>)>,
    strings: &Strings,
) -> Vec<ARow> {
    let section = &open.answering;
    let chosen_way = section.chosen_way();
    let chosen = section.chosen();
    let mut rows = Vec::new();
    for (way, choices) in ways {
        let row = if way == Offered::NotAtAll {
            SettingsRow::Answer(SettingsChoice::NotAtAll)
        } else {
            SettingsRow::Way(way)
        };
        let mut head = ARow::named(
            Some(row),
            TheKind::AChoice {
                chosen: chosen_way == Some(way),
            },
            as_a_name(&way.called(strings).into_text()),
        );
        head.lines
            .push(as_a_name(&way.described(strings).into_text()));
        rows.push(head);
        if way == Offered::NotAtAll {
            continue;
        }
        for choice in choices {
            rows.push(a_choice_under(open, choice, chosen.as_ref(), strings));
        }
        if way == Offered::FromAProvider {
            let mut ours = ARow::named(
                Some(SettingsRow::AlosOwnService),
                TheKind::ANestedChoice { chosen: false },
                said(strings, w::ALOS_OWN_SERVICE),
            );
            ours.lines.push(said(strings, w::ALOS_OWN_SERVICE_IS));
            rows.push(ours);
            if section.a_provider_can_be_added() {
                rows.push(ARow::named(
                    Some(SettingsRow::AddAProvider),
                    TheKind::AnAddingRow,
                    strings
                        .say(
                            &crate::the_providers_words::ADDING_A_PROVIDER.key(),
                            &Filling::nothing(),
                        )
                        .into_text(),
                ));
            }
        }
    }
    rows
}

/// One choice under a way: a model, a computer or a provider.
fn a_choice_under(
    open: &Open,
    choice: SettingsChoice,
    chosen: Option<&SettingsChoice>,
    strings: &Strings,
) -> ARow {
    let is_chosen = chosen == Some(&choice);
    let kind = TheKind::ANestedChoice { chosen: is_chosen };
    match &choice {
        SettingsChoice::Provider { provider, model } => {
            let mut row = ARow::named(
                Some(SettingsRow::Answer(choice.clone())),
                kind,
                provider.clone(),
            );
            let added = open.answering.the_provider(provider);
            let with_a_key = added.is_some_and(|it| it.key.is_some());
            row.lines.push(
                strings
                    .say(
                        &if with_a_key {
                            w::YOU_ADDED_THIS_WITH_A_KEY
                        } else {
                            w::YOU_ADDED_THIS
                        }
                        .key(),
                        &Filling::of(w::THE_MODEL, model.clone()),
                    )
                    .into_text(),
            );
            match added.map(|it| &it.region) {
                Some(alo_models::Region::Declared(region)) => {
                    row.note = Some(ANote {
                        words: strings
                            .say(
                                &w::IT_SAYS_IT_RUNS_IN.key(),
                                &Filling::of(w::THE_REGION, region.clone()),
                            )
                            .into_text(),
                        warns: false,
                    });
                }
                Some(alo_models::Region::Unknown) | None => {
                    row.note = Some(ANote {
                        words: said(strings, w::HAS_NOT_SAID_WHERE),
                        warns: true,
                    });
                    row.button = Some((
                        said(strings, w::SAY_WHERE_IT_RUNS),
                        SettingsRow::SayWhereItRuns(provider.clone()),
                    ));
                }
            }
            row
        }
        SettingsChoice::Brought(name) | SettingsChoice::Paired(name) => ARow::named(
            Some(SettingsRow::Answer(choice.clone())),
            kind,
            name.clone(),
        ),
        SettingsChoice::NotAtAll => ARow::named(
            Some(SettingsRow::Answer(SettingsChoice::NotAtAll)),
            kind,
            String::new(),
        ),
    }
}
