//! **The Accounts page**, as the owner approved it on 2026-11-13
//! (`accounts-signed-out` and `accounts-signed-in`, measured in
//! `docs/design/github-sign-in.md`): *Services you sign in to*, a card with
//! one row — *GitHub*, who is signed in, *Sign in* or *Sign out* at its end,
//! and *What it risks* under it in the owner's words.
//!
//! **The risk is said whether or not anybody is signed in**, as both pictures
//! draw it: a person deciding whether to sign in reads it before they do, and
//! a person signed in reads what they are living with.

use alo_strings::Strings;

use crate::settings_page::{ABlock, ARisk, ARow, ASection, TheKind, said};
use crate::settings_window::{Open, SettingsRow};
use crate::the_settings_words as w;

/// The page's blocks.
pub(crate) fn the_blocks(open: &Open, strings: &Strings) -> Vec<ABlock> {
    let github = &open.github;
    let mut blocks: Vec<ABlock> = Vec::new();
    // **What the last thing asked came to**, above the card, as every page
    // says a refusal; gh's own line, where it said one, is the second line.
    blocks.extend(github.told.iter().cloned().map(ABlock::Said));
    let mut row = ARow::named(None, TheKind::Plain, said(strings, w::GITHUB));
    row.lines.push(github.line(strings));
    row.button = github.rows().into_iter().next().map(|button| {
        let label = match button {
            SettingsRow::SignOutOfGitHub => w::SIGN_OUT,
            _ => w::SIGN_IN,
        };
        (said(strings, label), button)
    });
    row.risk = Some(ARisk {
        heading: said(strings, w::WHAT_IT_RISKS),
        words: said(strings, w::GITHUB_RISKS),
    });
    blocks.push(ABlock::Section(ASection {
        heading: Some(said(strings, w::SERVICES_YOU_SIGN_IN_TO)),
        rows: vec![row],
        // **No footnote**: neither approved picture draws one.
        footnote: None,
    }));
    blocks
}
