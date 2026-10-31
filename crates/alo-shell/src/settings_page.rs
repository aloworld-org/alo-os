//! One page of Settings as the things drawn on it, in reading order: the
//! page's heading and introduction, then its sections — each a heading, a card
//! of rows and a footnote — and every sentence a section has to say.
//!
//! **This is the shape both the drawing and the keyboard read.** A row a person
//! can act on carries its [`SettingsRow`]; the keyboard's order is the order of
//! those rows here ([`APage::rows`]), and `crate::settings_raster` draws exactly
//! these. One list read by two, so a row cannot be drawn and not reachable, or
//! reachable and not drawn — the fault `every_setting_the_machine_has_is_in_one_window`
//! has caught three times.
//!
//! **Nothing is worded here.** Every string arrives said: a crate's word for a
//! row, `crate::the_settings_words` for the window's own.

use alo_strings::{Filling, Strings};

use crate::settings_categories::SettingsCategory;
use crate::settings_lines::{LineWords, SectionWords};
use crate::settings_window::{Open, SettingsRow, SettingsSection};

/// One page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct APage {
    /// Which category it is.
    pub(crate) category: SettingsCategory,
    /// Its heading, which is the category's name.
    pub(crate) heading: String,
    /// The sentence under the heading.
    pub(crate) introduction: String,
    /// What it holds, in reading order.
    pub(crate) blocks: Vec<ABlock>,
}

/// One thing on a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ABlock {
    /// A sentence the page has to say: a change was refused, a file did not
    /// read, a folder was just given — drawn in a box of its own above the
    /// sections.
    Said(String),
    /// A heading, a card of rows and a footnote.
    Section(ASection),
}

/// One section: a heading over a card of rows, and a footnote under it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ASection {
    /// The heading, in sentence case.
    pub(crate) heading: Option<String>,
    /// The rows, in the card.
    pub(crate) rows: Vec<ARow>,
    /// The sentence under the card.
    pub(crate) footnote: Option<String>,
}

/// What kind of row it is, which decides what is drawn at its start and end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TheKind {
    /// One of several choices: a ring, filled when it is chosen.
    AChoice {
        /// Whether it is what the person's settings are now.
        chosen: bool,
    },
    /// One of several choices under another: the smaller ring, further in.
    ANestedChoice {
        /// Whether it is what the person's settings are now.
        chosen: bool,
    },
    /// Something that opens something else: a plus before it, under another.
    AnAddingRow,
    /// Something that opens something else: a chevron at its end.
    AnOpeningRow,
    /// On or off, with the word beside it.
    ASwitch(SwitchIs),
    /// Words, and nothing to choose.
    Plain,
}

/// Which way a switch is, as the person's settings say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwitchIs {
    /// On.
    On,
    /// Off.
    Off,
    /// The person has never said, which is neither — `alo-choosing`'s *absent
    /// is stronger than defaulted*.
    NotSet,
}

/// A note under a row's words, with the small sign before it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ANote {
    /// The words.
    pub(crate) words: String,
    /// Whether it asks for attention, which is drawn in the warning colour.
    pub(crate) warns: bool,
}

/// What a choice risks: a heading and a sentence, in a box of their own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ARisk {
    /// *What it risks*.
    pub(crate) heading: String,
    /// The risk, in plain words.
    pub(crate) words: String,
}

/// One row in a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ARow {
    /// What a person acts on when the keyboard is on it, or [`None`] for a
    /// row that only says something.
    pub(crate) row: Option<SettingsRow>,
    /// What kind of row.
    pub(crate) kind: TheKind,
    /// Its name.
    pub(crate) title: String,
    /// The lines under its name.
    pub(crate) lines: Vec<String>,
    /// A note under those.
    pub(crate) note: Option<ANote>,
    /// What choosing it risks, in a box under its words.
    pub(crate) risk: Option<ARisk>,
    /// A button at its end, with what pressing it acts on.
    pub(crate) button: Option<(String, SettingsRow)>,
    /// Words at the row's end: a switch's *On* or *Off*, or a shortcut's keys.
    pub(crate) at_its_end: Option<String>,
}

impl ARow {
    /// A row with a name and nothing else, of `kind`, acting on `row`.
    pub(crate) fn named(row: Option<SettingsRow>, kind: TheKind, title: String) -> Self {
        Self {
            row,
            kind,
            title,
            lines: Vec::new(),
            note: None,
            risk: None,
            button: None,
            at_its_end: None,
        }
    }

    /// Everything on this row a person can act on, in the order the keyboard
    /// reaches them: the row, then its button.
    pub(crate) fn targets(&self) -> impl Iterator<Item = &SettingsRow> {
        self.row
            .iter()
            .chain(self.button.as_ref().map(|(_, row)| row))
    }
}

impl APage {
    /// Every row on the page a person can act on, in the keyboard's order —
    /// read by the tests that hold it to `crate::settings_window::Open::rows_on`.
    #[cfg(test)]
    pub(crate) fn rows(&self) -> Vec<SettingsRow> {
        self.blocks
            .iter()
            .filter_map(|block| match block {
                ABlock::Section(section) => Some(section),
                ABlock::Said(_) => None,
            })
            .flat_map(|section| section.rows.iter().flat_map(ARow::targets))
            .cloned()
            .collect()
    }
}

/// The page for `category`, as `open` holds it now.
pub(crate) fn the_page(
    open: &Open,
    category: SettingsCategory,
    strings: &Strings,
    now: std::time::SystemTime,
) -> APage {
    let blocks = match category {
        SettingsCategory::AloAndAi => crate::settings_page_alo::the_blocks(open, strings, now),
        other => the_sections_of(other, open, strings, now),
    };
    APage {
        category,
        heading: category.said(strings),
        introduction: said(strings, category.is()),
        blocks,
    }
}

/// The page while a person is adding a provider: the form takes the alo and
/// AI page over, because every key is the form's while it is open and rows
/// behind it would be rows nobody can reach. See
/// `crate::a_provider_being_added_is_drawn`.
pub(crate) fn the_form_page(form: SectionWords, strings: &Strings) -> APage {
    let category = SettingsCategory::AloAndAi;
    let mut blocks: Vec<ABlock> = form.said.into_iter().map(ABlock::Said).collect();
    let rows: Vec<ARow> = form
        .lines
        .into_iter()
        .map(|line| {
            let mut parts = line.parts.into_iter();
            let mut row = ARow::named(None, TheKind::Plain, parts.next().unwrap_or_default());
            row.lines = parts.collect();
            row
        })
        .collect();
    if !rows.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: None,
            rows,
            footnote: None,
        }));
    }
    APage {
        category,
        heading: category.said(strings),
        introduction: said(strings, category.is()),
        blocks,
    }
}

/// A word of this window's own, said.
pub(crate) fn said(strings: &Strings, word: alo_strings::Word) -> String {
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

/// The page's blocks from the sections `crate::settings_lines` words, for every
/// page whose rows are each one line of words: a section becomes a heading and
/// a card, and what it says goes above.
fn the_sections_of(
    category: SettingsCategory,
    open: &Open,
    strings: &Strings,
    now: std::time::SystemTime,
) -> Vec<ABlock> {
    let mut blocks = Vec::new();
    for section in crate::settings_lines::sections(open, strings, now) {
        if SettingsCategory::of(section.section) != category {
            continue;
        }
        blocks.extend(blocks_of(section, strings));
    }
    if blocks.is_empty() {
        blocks.push(ABlock::Said(said(
            strings,
            crate::the_settings_words::NOTHING_HERE_YET,
        )));
    }
    blocks
}

/// One section's words as the blocks a page draws.
fn blocks_of(section: SectionWords, strings: &Strings) -> Vec<ABlock> {
    let mut blocks: Vec<ABlock> = section.said.into_iter().map(ABlock::Said).collect();
    let which = section.section;
    let rows: Vec<ARow> = section
        .lines
        .into_iter()
        .map(|line| row_of(line, strings))
        .collect();
    if which == SettingsSection::Granted {
        blocks.extend(the_granted(rows, strings));
        return blocks;
    }
    if !rows.is_empty() {
        blocks.push(ABlock::Section(ASection {
            heading: heading_of(which).map(|word| said(strings, word)),
            rows,
            footnote: None,
        }));
    }
    blocks
}

/// What has been granted, divided the way the design draws it: the folders
/// given to alo, the row that gives it another, the record of what it did,
/// and the computers paired with this one.
///
/// **The order is `crate::settings_window::Open::rows`' order**, which the
/// keyboard follows, and `every_setting_the_machine_has_is_in_one_window`
/// holds the two to each other.
///
/// **Paired computers are here, under a heading of their own**, because they
/// were in this section's one list before the redesign and the redesign does
/// not draw them anywhere: a pairing is something granted, and it is revoked
/// here the same way a folder is. `docs/design/the-settings-window.md` names
/// this as the owner's to confirm.
fn the_granted(rows: Vec<ARow>, strings: &Strings) -> Vec<ABlock> {
    use crate::the_settings_words as w;
    // **While a folder is being picked, the picker is the whole section**, for
    // the reason `crate::settings_lines` gives.
    if rows
        .iter()
        .any(|row| matches!(row.row, Some(SettingsRow::Picking(_))))
    {
        return vec![ABlock::Section(ASection {
            heading: None,
            rows,
            footnote: None,
        })];
    }
    let (mut given, mut giving, mut record, mut paired) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for row in rows {
        match &row.row {
            Some(SettingsRow::Granted(alo_changing::Row::Pairing(_))) => paired.push(row),
            Some(SettingsRow::GrantAFolder) => giving.push(row),
            Some(SettingsRow::SeeWhatAloDid) => record.push(row),
            _ => given.push(row),
        }
    }
    [
        (Some(w::GIVEN_TO_ALO), given),
        (None, giving),
        (Some(w::WHAT_ALO_DID), record),
        (Some(w::PAIRED_MACHINES), paired),
    ]
    .into_iter()
    .filter(|(_, rows)| !rows.is_empty())
    .map(|(heading, rows)| {
        ABlock::Section(ASection {
            heading: heading.map(|word| said(strings, word)),
            rows,
            footnote: None,
        })
    })
    .collect()
}

/// The heading over a section, in sentence case.
const fn heading_of(section: SettingsSection) -> Option<alo_strings::Word> {
    use crate::the_settings_words as w;
    Some(match section {
        SettingsSection::Answering => w::WHERE_QUESTIONS_ARE_ANSWERED,
        SettingsSection::Appearance => w::LOOK,
        SettingsSection::Dock => w::THE_DOCK,
        SettingsSection::Formats => w::WHERE_THIS_COMPUTER_IS,
        SettingsSection::TimeZone => w::TIME_ZONE,
        SettingsSection::Shortcuts => w::SHORTCUTS,
        SettingsSection::Terminal => w::IN_THE_TERMINAL,
        SettingsSection::HowCodeRuns => w::ALO_RUNS_CODE,
        SettingsSection::Granted => w::GIVEN_TO_ALO,
    })
}

/// One line of a section as a row: its first piece the name, the rest the
/// lines under it, and what kind of row its [`SettingsRow`] makes it.
fn row_of(line: LineWords, strings: &Strings) -> ARow {
    let kind = match &line.row {
        Some(
            SettingsRow::Accent(_)
            | SettingsRow::WrittenAs(_)
            | SettingsRow::TimeZone(_)
            | SettingsRow::Hiding(_)
            | SettingsRow::HowCodeRuns(_),
        ) => TheKind::AChoice {
            chosen: line.chosen,
        },
        Some(SettingsRow::ShellIntegration | SettingsRow::APlaceToReturnTo) => {
            TheKind::ASwitch(if line.chosen {
                SwitchIs::On
            } else {
                SwitchIs::Off
            })
        }
        Some(
            SettingsRow::GrantAFolder
            | SettingsRow::SeeWhatAloDid
            | SettingsRow::AddAProvider
            | SettingsRow::Shortcut(_),
        ) => TheKind::AnOpeningRow,
        _ => TheKind::Plain,
    };
    let mut parts = line.parts.into_iter();
    let title = parts
        .next()
        .map(|it| crate::settings_type::as_a_name(&it))
        .unwrap_or_default();
    // **A shortcut's keys at the row's end**, where the design draws them, and
    // not as a line under the action's name.
    let (at_its_end, lines) = match (&line.row, kind) {
        (Some(SettingsRow::Shortcut(_)), _) => (parts.next(), Vec::new()),
        (_, TheKind::ASwitch(is)) => (
            Some(switch_said(is, strings)),
            parts
                .map(|it| crate::settings_type::as_a_name(&it))
                .collect(),
        ),
        _ => (
            None,
            parts
                .map(|it| crate::settings_type::as_a_name(&it))
                .collect(),
        ),
    };
    ARow {
        row: line.row,
        kind,
        title,
        lines,
        note: None,
        risk: None,
        button: None,
        at_its_end,
    }
}

/// The word beside a switch.
pub(crate) fn switch_said(is: SwitchIs, strings: &Strings) -> String {
    use crate::the_settings_words as w;
    said(
        strings,
        match is {
            SwitchIs::On => w::ON,
            SwitchIs::Off => w::OFF,
            SwitchIs::NotSet => w::NOT_SET,
        },
    )
}
