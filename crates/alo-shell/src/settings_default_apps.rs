//! Settings → Default apps, as the section holds it: which installed
//! application opens a web link and which an email address, chosen by the
//! person and kept in their `what-opens-what.toml` (frame `25:2`,
//! `docs/design/default-apps-and-open-with.md`, approved by the owner on
//! 2026-11-12).
//!
//! # One answer to *which browser*
//!
//! What a row offers is `alo_applications::WhatOpensWhat::what_could_open_links`
//! and which row is ringed is `…::what_opens_links` — the very two answers the
//! portal opens a link with. So the browser ringed here is the browser a link
//! opens in, whatever the person's file says: a choice that is gone, or that
//! is a terminal, is passed over there and so is not ringed here.
//!
//! # The machine is told, and never read here
//!
//! What is installed, what each declares it opens and what each draws arrive
//! from the seat ([`WhatCouldOpenLinks`]), which was told them by the binary
//! that reads the machine — `crate::what_the_machine_offers_now`. This file
//! reads only the person's own file, through its keeper, as every other
//! section does.
//!
//! # Choosing writes `[links]`, and nothing else
//!
//! A choice is `alo_applications::Chosen::choose_for_links` on a copy, kept
//! whole through `crate::settings_kept`; the `[kinds]` the file holds are
//! written back as they were read.

use std::path::PathBuf;
use std::sync::Arc;

use alo_applications::{Application, Declared, Installed, LinkKind, WhatOpensWhat};
use alo_dock::artwork::{APicture, TheArtwork};
use alo_strings::Strings;

use crate::settings_keepers::LinksKept;
use crate::settings_kept::{KeptSection, SettingsKept};
use crate::settings_window::SettingsRow;

/// What this machine has that could open a link, as the seat was last told.
#[derive(Debug, Clone, Default)]
pub(crate) struct WhatCouldOpenLinks {
    /// Every application installed, with the name its entry gives.
    pub(crate) installed: Installed,
    /// What each declares it opens, first declared first.
    pub(crate) declared: Declared,
    /// What each draws on the Dock.
    pub(crate) artwork: Arc<TheArtwork>,
}

/// The Default apps section.
#[derive(Debug)]
pub(crate) struct DefaultAppsHere {
    /// The person's `what-opens-what.toml`.
    kept: KeptSection<LinksKept>,
    /// What the machine has.
    here: WhatCouldOpenLinks,
    /// What the page says about the last choice that changed nothing.
    told: Option<String>,
}

impl DefaultAppsHere {
    /// The section at sign-in: the person's file read once, and nothing yet
    /// known about the machine.
    pub(crate) fn at_sign_in(at: Option<PathBuf>) -> Self {
        Self {
            kept: KeptSection::at_sign_in(at),
            here: WhatCouldOpenLinks::default(),
            told: None,
        }
    }

    /// The machine has these applications, as the seat was told.
    pub(crate) fn the_machine_has(&mut self, here: WhatCouldOpenLinks) {
        self.here = here;
    }

    /// The person's file, as its keeper holds it.
    pub(crate) const fn kept(&self) -> &KeptSection<LinksKept> {
        &self.kept
    }

    /// The person's file, to put back as shipped.
    pub(crate) const fn kept_mut(&mut self) -> &mut KeptSection<LinksKept> {
        &mut self.kept
    }

    /// What the page says about the last choice that changed nothing.
    pub(crate) fn told(&self) -> Option<&str> {
        self.told.as_deref()
    }

    /// Whether this section's choices can be offered at all: only where the
    /// session has a folder to keep them in.
    pub(crate) fn is_offered(&self) -> bool {
        self.kept.at().is_some()
    }

    /// The applications a person may choose among for this kind, first
    /// declared first.
    pub(crate) fn could_open(&self, kind: LinkKind) -> Vec<Application> {
        WhatOpensWhat::on(&self.here.installed, &self.here.declared, self.kept.drawn())
            .what_could_open_links(kind)
            .into_iter()
            .cloned()
            .collect()
    }

    /// The identifier of what opens this kind of link now, if anything does.
    pub(crate) fn opens_now(&self, kind: LinkKind) -> Option<String> {
        WhatOpensWhat::on(&self.here.installed, &self.here.declared, self.kept.drawn())
            .what_opens_links(kind)
            .ok()
            .map(|opener| opener.application().identifier().to_owned())
    }

    /// What this application draws, where the machine has a picture for it.
    pub(crate) fn picture_of(&self, identifier: &str) -> Option<APicture> {
        alo_dock::AppId::named(identifier)
            .ok()
            .and_then(|app| self.here.artwork.of(&app).cloned())
    }

    /// The rows, in the order the page draws them: web links, then email
    /// addresses.
    pub(crate) fn rows(&self) -> Vec<SettingsRow> {
        if !self.is_offered() {
            return Vec::new();
        }
        LinkKind::EVERY
            .into_iter()
            .flat_map(|kind| {
                self.could_open(kind).into_iter().map(move |application| {
                    SettingsRow::DefaultFor(kind, application.identifier().to_owned())
                })
            })
            .collect()
    }

    /// The person chose this application for this kind of link.
    ///
    /// **Only an application this page offers can be chosen**: one that left
    /// the machine since the page was drawn is refused in words, and the file
    /// is not touched.
    pub(crate) fn choose(
        &mut self,
        kind: LinkKind,
        identifier: &str,
        strings: &Strings,
    ) -> SettingsKept {
        let Some(application) = self
            .could_open(kind)
            .into_iter()
            .find(|it| it.identifier() == identifier)
        else {
            self.told = Some(crate::settings_page::said(
                strings,
                crate::the_settings_words::THAT_APP_IS_GONE,
            ));
            return SettingsKept::Refused;
        };
        self.told = None;
        self.kept.changed(
            |chosen| {
                chosen.choose_for_links(kind, &application);
                Ok(())
            },
            strings,
        )
    }
}

#[cfg(test)]
#[path = "settings_default_apps_tests.rs"]
mod tests;
