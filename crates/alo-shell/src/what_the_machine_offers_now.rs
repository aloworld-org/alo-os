//! **What the machine offers, told to a seat that is already running.**
//!
//! `crate::stand_the_desktop_up` is told once what the machine has — the
//! Dock's roles, the terminal, the artwork — and until this file that was the
//! only time it was told. A fresh machine installs its applications **after**
//! its first boot, for minutes on real hardware and for hours in a virtual
//! machine, and a person can be signed in the whole time. On 2026-10-31 a
//! booted machine's desktop came up at 669 s, the browser arrived at 1429 s,
//! and the person never saw it: the Dock, the launcher and the terminal chord
//! all kept the answer from 669 s, which was *nothing*.
//!
//! So the binary that reads the machine reads it again when it changes, and
//! hands the whole answer here through `crate::TheDesktop::what_this_machine_offers_now`.
//! This crate still reads nothing — the rule `what_the_dock_is_filled_with`
//! states — it is told again.
//!
//! # What a new answer changes, and what it may not
//!
//! The artwork, the terminal and the list the alo Bar searches are replaced
//! whole: each is a reading of the machine and the newest is the true one.
//!
//! **The Dock is worked out again only until it is settled.** A Dock filled
//! from a discovery that has not finished is drawn and not kept
//! (`alo_dock::pinning::WhatToPin::ForNow`), so the next answer may replace
//! it. Once the person's own pins are read, or the defaults are written down,
//! no answer may — that would put back a pin somebody had just taken off.

use alo_dock::AppId;
use alo_dock::artwork::TheArtwork;
use alo_dock::pinning::Discovery;

use crate::server::Server;

/// One application the machine has, as its desktop entry names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnApplicationHere {
    /// What the machine knows it by — the desktop file identifier, which is
    /// what an `AppId` and the bus name are built from.
    pub identifier: String,
    /// What it calls itself, where its entry said.
    pub called: Option<String>,
    /// What it is, in its entry's own words and the person's language —
    /// `GenericName`, `Keywords` and `Categories` — for the alo Bar to find it
    /// by. Empty where the entry says nothing about it.
    pub is: Vec<String>,
}

/// Everything one reading of the machine tells a running seat.
#[derive(Debug, Clone)]
pub struct WhatTheMachineOffersNow {
    /// Which applications fill the Dock's default roles, and whether the
    /// reading is finished.
    pub roles: Discovery,
    /// This machine's terminal, when one is installed.
    pub terminal: Option<AppId>,
    /// What each application draws on the Dock.
    pub artwork: TheArtwork,
    /// Every application a person could open, for the alo Bar to search.
    pub installed: Vec<AnApplicationHere>,
    /// What each application's entry declares it opens, first declared first —
    /// what Settings → Default apps offers a person to choose among, read from
    /// the same entries the portal answers a link from.
    pub what_they_open: alo_applications::Declared,
}

impl Server {
    /// Take a new reading of the machine.
    ///
    /// `desktop` is asked for the pins the person kept, and told which to keep
    /// when the defaults are final — the same two calls
    /// `crate::stand_the_desktop_up` makes, for the same reason: this crate
    /// keeps no file of its own.
    pub fn the_machine_now_offers(
        &mut self,
        now: WhatTheMachineOffersNow,
        desktop: &mut dyn crate::TheDesktop,
    ) {
        self.the_terminal = now.terminal;
        self.its_applications_draw(now.artwork);
        self.what_they_open = now.what_they_open;
        self.what_this_machine_has = now
            .installed
            .iter()
            .map(|application| application.identifier.clone())
            .collect();
        self.what_each_is = now
            .installed
            .iter()
            .filter(|application| !application.is.is_empty())
            .map(|application| (application.identifier.clone(), application.is.clone()))
            .collect();
        self.what_each_is_called = now
            .installed
            .into_iter()
            .filter_map(|application| {
                application
                    .called
                    .map(|called| (application.identifier, called))
            })
            .collect();
        // **A change the person made and the disk has not had yet is theirs**,
        // so it settles the Dock before a reading can work it out again; it is
        // written on this frame and read back as their own on the next.
        if self.the_dock_is_settled || self.pins_owed_to_the_disk.is_some() {
            return;
        }
        let kept = desktop.the_pins_they_kept();
        if let Some(owed) = self.its_dock_is_filled_from_what_they_kept(&kept, &now.roles) {
            desktop.keep_these_pins(&owed);
        }
    }

    /// **Settings is told what could open a link**, from what this seat was
    /// last told: every application with the name its entry gives, what each
    /// declares it opens, and what each draws. Called when Settings opens, so
    /// Default apps lists what is installed at that moment.
    pub(crate) fn settings_is_told_what_could_open_links(&mut self) {
        let installed = alo_applications::Installed::holding(
            self.what_this_machine_has.iter().filter_map(|identifier| {
                match self.what_each_is_called.get(identifier) {
                    Some(called) => alo_applications::Application::called(identifier, called)
                        .or_else(|_| alo_applications::Application::identified(identifier))
                        .ok(),
                    None => alo_applications::Application::identified(identifier).ok(),
                }
            }),
        );
        let here = crate::settings_default_apps::WhatCouldOpenLinks {
            installed,
            declared: self.what_they_open.clone(),
            artwork: std::sync::Arc::clone(&self.what_each_application_draws),
        };
        self.settings.the_applications_are(here);
    }
}

#[cfg(test)]
#[path = "what_the_machine_offers_now_tests.rs"]
mod tests;
