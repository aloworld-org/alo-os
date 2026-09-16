//! What alo OS may set in the browser it ships, and what it never sets.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 3: *nothing alo OS
//! ships sets its home page, search engine or telemetry … with its telemetry
//! **off** where the upstream allows a policy to turn it off.* Two closed lists,
//! and between them they are the whole of the decision:
//!
//! - [`Setting`] — the three things that are set, each with the reason it is.
//! - [`NeverSet`] — the things that are **not**, each with what setting it would
//!   take away from the person.
//!
//! The reader ([`crate::Configuration`]) refuses anything that is not a
//! [`Setting`], so [`NeverSet`] changes no outcome: a key on it and a key nobody
//! ever thought of are both refused. What it adds is the *reason*, at the place
//! somebody will be standing when they want to add one — and a refusal that
//! names what the key would have cost rather than only that it is unknown.
//!
//! # Why anything is set at all
//!
//! A rented browser is configured and never patched (`CLAUDE.md`, ADR 0005), and
//! configured through the upstream's own published policy document, which is the
//! one surface it offers. Two of the three are `docs/features.md`'s promise of
//! *no telemetry* carried as far into a rented application as its own maker
//! allows. The third is not a preference about content at all: it is what makes
//! *the folder a person chose* true of a download, because a browser that does
//! not ask writes to a folder it picked.
//!
//! # What could not be turned off
//!
//! A policy turns off what its maker made a policy for, and no more. The browser
//! still keeps its own block lists up to date, still asks whether this network
//! wants a sign-in first, and still has a crash reporter it offers a person after
//! a crash. None of that is usage telemetry, none of it has a policy, and none of
//! it is claimed away here: it is a person's browser reaching the web on that
//! person's behalf, which is what a browser is. The four laws' *zero inference
//! egress* is measured of an **agent**, and a rented browser is not one.
//! `docs/autonomy/updates/the-web-browser.md` names each of them.

/// One thing alo OS sets in the browser it ships.
///
/// A closed list, and the whole of what is set. Each is a key in the browser's
/// own published policy document, whose value is `true`; there is no setting here
/// whose value is text, because text is where a home page or a search engine
/// would arrive wearing another key's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Setting {
    /// The browser sends its maker no usage data.
    TelemetryOff,
    /// The browser enrols the person in none of its maker's experiments.
    ExperimentsOff,
    /// The browser asks where each download is to be saved.
    AskWhereEachDownloadGoes,
}

impl Setting {
    /// Every setting, in the order the shipped document writes them.
    pub const EVERY: [Self; 3] = [
        Self::TelemetryOff,
        Self::ExperimentsOff,
        Self::AskWhereEachDownloadGoes,
    ];

    /// The key the browser's own policy document knows it by.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::TelemetryOff => "DisableTelemetry",
            Self::ExperimentsOff => "DisableFirefoxStudies",
            Self::AskWhereEachDownloadGoes => "PromptForDownloadLocation",
        }
    }

    /// Why alo OS sets it — for whoever is reading the shipped document, which
    /// is written in a notation that has nowhere to say this.
    #[must_use]
    pub const fn why(self) -> &'static str {
        match self {
            Self::TelemetryOff => {
                "docs/features.md promises no telemetry, and this is how far that reaches into an \
                 application somebody else makes"
            }
            Self::ExperimentsOff => {
                "an experiment changes how a person's browser behaves without their having chosen \
                 it, and reports on what happened"
            }
            Self::AskWhereEachDownloadGoes => {
                "a download goes to the folder the person chose in the moment, rather than to a \
                 folder the browser picked; the choice is theirs to make and the browser has no \
                 grant over the folder until they have made it"
            }
        }
    }
}

/// One thing alo OS does not set in the browser it ships, and what setting it
/// would take from the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NeverSet {
    /// The page the browser opens with.
    HomePage,
    /// What a new tab shows.
    NewTab,
    /// What the browser's own start page shows.
    StartPage,
    /// Which search engines the browser has.
    SearchEngines,
    /// Which road out of the machine the browser takes.
    Proxy,
    /// The folder every download goes to.
    DownloadFolder,
    /// The folder every download goes to, said the other way the browser says
    /// it.
    DefaultDownloadFolder,
    /// Bookmarks the person did not make.
    Bookmarks,
    /// Which sites the person may read.
    WhichSitesArePermitted,
    /// Any single setting the browser has, reached by its internal name — the
    /// door every other key on this list could be opened through.
    AnySettingAtAll,
}

impl NeverSet {
    /// Every one, in no order a person reads.
    pub const EVERY: [Self; 10] = [
        Self::HomePage,
        Self::NewTab,
        Self::StartPage,
        Self::SearchEngines,
        Self::Proxy,
        Self::DownloadFolder,
        Self::DefaultDownloadFolder,
        Self::Bookmarks,
        Self::WhichSitesArePermitted,
        Self::AnySettingAtAll,
    ];

    /// The key in the browser's own policy document that would set it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::HomePage => "Homepage",
            Self::NewTab => "NewTabPage",
            Self::StartPage => "FirefoxHome",
            Self::SearchEngines => "SearchEngines",
            Self::Proxy => "Proxy",
            Self::DownloadFolder => "DownloadDirectory",
            Self::DefaultDownloadFolder => "DefaultDownloadDirectory",
            Self::Bookmarks => "Bookmarks",
            Self::WhichSitesArePermitted => "WebsiteFilter",
            Self::AnySettingAtAll => "Preferences",
        }
    }

    /// Which key this is, if it is one of these.
    #[must_use]
    pub fn named(key: &str) -> Option<Self> {
        Self::EVERY.into_iter().find(|never| never.key() == key)
    }

    /// What setting it would take from the person.
    #[must_use]
    pub const fn takes(self) -> &'static str {
        match self {
            Self::HomePage => "the page a person's own browser opens with",
            Self::NewTab => "what a person sees when they open a tab",
            Self::StartPage => "what a person's browser shows them when it starts",
            Self::SearchEngines => "who a person's questions about the web are sent to",
            Self::Proxy => {
                "which road out of the machine the browser takes, which is one machine-wide \
                 setting an organisation or the person sets, never a browser's own"
            }
            Self::DownloadFolder | Self::DefaultDownloadFolder => {
                "where a person's downloads go, which is the folder they chose in the moment"
            }
            Self::Bookmarks => "which sites are in a person's own list of them",
            Self::WhichSitesArePermitted => "which parts of the web a person may read",
            Self::AnySettingAtAll => {
                "every other setting on this list, reached by its internal name instead of its own \
                 key — so it is refused even where the setting behind it would have been harmless"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **Each list is as long as it says and names each key once**, and the two
    /// lists never name the same key.
    #[test]
    fn each_key_is_named_once_and_by_one_list() {
        let set: BTreeSet<&str> = Setting::EVERY.iter().map(|one| one.key()).collect();
        assert_eq!(set.len(), Setting::EVERY.len());
        let never: BTreeSet<&str> = NeverSet::EVERY.iter().map(|one| one.key()).collect();
        assert_eq!(never.len(), NeverSet::EVERY.len());
        assert!(set.is_disjoint(&never));
        for one in NeverSet::EVERY {
            assert_eq!(NeverSet::named(one.key()), Some(one), "{}", one.key());
        }
        assert_eq!(NeverSet::named("DisableTelemetry"), None);
    }

    /// **Every setting says why it is set, and every refused key says what it
    /// would have taken** — a sentence each, for whoever is about to add one.
    #[test]
    fn every_key_carries_its_reason() {
        for one in Setting::EVERY {
            assert!(one.why().len() > 40, "{}", one.key());
        }
        for one in NeverSet::EVERY {
            assert!(one.takes().len() > 20, "{}", one.key());
        }
    }
}
