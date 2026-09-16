//! The configuration alo OS ships with the browser, read and held to being that
//! and nothing more.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 3: *a test reads the
//! shipped configuration and finds the upstream's own defaults, with its
//! telemetry off where the upstream allows a policy to turn it off.* The document
//! is data beside this crate's manifest — `crates/alo-software/browser.json`
//! ([`WHERE_IT_IS`]) — because it is a file the browser reads rather than
//! anything this code executes, and because the installer plan can put a file on
//! a machine and cannot run a function in this crate.
//!
//! # Held to the two closed lists, and to nothing else being there
//!
//! [`Configuration::read`] refuses ([`NotConfigured`]) a document that
//!
//! - is not `{"policies": {…}}` and only that;
//! - names a key that is not a [`Setting`] — and where that key is one of
//!   [`NeverSet`]'s, says what setting it would have taken from the person;
//! - gives any of them a value other than `true`, so no setting arrives as text
//!   in which an address, a search engine or a home page could be written;
//! - leaves one of the three out, because *telemetry off* is a promise and a
//!   document that quietly stopped making it is the thing this reader exists to
//!   catch.
//!
//! # The document has no comments, and that is the notation's fault
//!
//! It is written in the browser's own published format, which has nowhere to put
//! one. [`Setting::why`] is where each reason is instead, and it is beside the
//! reader rather than inside the file for that reason alone.
//!
//! # Where it goes on a machine
//!
//! [`WHERE_THE_BROWSER_READS_IT`] is the path the browser reads a machine-wide
//! policy document from. Whether the sandboxed build of it sees that path is a
//! question about the rented tool on a real machine, which is outstanding for
//! this task exactly as it is for the two before it; the task's report says what
//! that acceptance is, and `docs/quirks.md` is where the answer goes once a
//! machine has been asked.
//!
//! # English, and why
//!
//! [`NotConfigured`] keeps its English and a `Display`, like [`crate::NotShipped`]:
//! a document that does not hold is this repository's own file contradicting
//! itself, read by whoever is editing it and never by a person using a machine.

use std::collections::BTreeMap;

use crate::browser_settings::{NeverSet, Setting};

/// The configuration, as it is built into this crate.
pub const THE_CONFIGURATION: &str = include_str!("../browser.json");

/// Where the configuration is, from the repository's root — for the installer
/// plan, which reads the file rather than this crate.
pub const WHERE_IT_IS: &str = "crates/alo-software/browser.json";

/// Where the browser reads a machine-wide policy document from.
///
/// The upstream's own documented path for one. See the section above on what is
/// still to be asked of a real machine.
pub const WHERE_THE_BROWSER_READS_IT: &str = "/etc/firefox/policies/policies.json";

/// Why the configuration alo OS ships with the browser was not believed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotConfigured {
    /// It is not a document in this shape at all.
    #[error("the browser's configuration could not be read: {why}")]
    NotRead {
        /// What the reader said.
        why: String,
    },
    /// A key alo OS has decided it never sets.
    #[error("the browser's configuration sets {}, which would set {}", .0.key(), .0.takes())]
    NeverSet(NeverSet),
    /// A key nothing here decided.
    #[error(
        "the browser's configuration sets {key}, which is not one of the settings alo OS ships \
         with the browser"
    )]
    NotDecided {
        /// The key it names.
        key: String,
    },
    /// A decided key with a value that is not `true`.
    #[error("the browser's configuration sets {key} to something other than true")]
    NotTrue {
        /// The key.
        key: String,
    },
    /// A decided key that is not there.
    #[error("the browser's configuration does not set {key}, and alo OS ships it set")]
    Missing {
        /// The key.
        key: String,
    },
}

/// The configuration alo OS ships with the browser: the settings it sets, and
/// the proof that it sets nothing else.
///
/// Built only by [`Configuration::read`], so holding one is the proof that the
/// refusals above were made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    /// Each setting, in [`Setting::EVERY`]'s order — not the document's, which
    /// is a notation where key order means nothing.
    sets: Vec<Setting>,
}

impl Configuration {
    /// The configuration built into this crate.
    ///
    /// # Errors
    /// [`NotConfigured`], which the document as written cannot cause — a test
    /// holds it.
    pub fn decided() -> Result<Self, NotConfigured> {
        Self::read(THE_CONFIGURATION)
    }

    /// A configuration, from its text.
    ///
    /// # Errors
    /// [`NotConfigured`], naming the first thing that does not hold.
    pub fn read(text: &str) -> Result<Self, NotConfigured> {
        let written: Written =
            serde_json::from_str(text).map_err(|why| NotConfigured::NotRead {
                why: why.to_string(),
            })?;
        let mut sets = Vec::with_capacity(Setting::EVERY.len());
        for (key, value) in &written.policies {
            let Some(setting) = Setting::EVERY.into_iter().find(|one| one.key() == key) else {
                return Err(match NeverSet::named(key) {
                    Some(never) => NotConfigured::NeverSet(never),
                    None => NotConfigured::NotDecided { key: key.clone() },
                });
            };
            if value != &serde_json::Value::Bool(true) {
                return Err(NotConfigured::NotTrue { key: key.clone() });
            }
            sets.push(setting);
        }
        for one in Setting::EVERY {
            if !sets.contains(&one) {
                return Err(NotConfigured::Missing {
                    key: one.key().to_owned(),
                });
            }
        }
        sets.sort();
        Ok(Self { sets })
    }

    /// Every setting it sets, in [`Setting::EVERY`]'s order.
    #[must_use]
    pub fn sets(&self) -> &[Setting] {
        &self.sets
    }

    /// Whether the document names this key at all.
    #[must_use]
    pub fn names(&self, key: &str) -> bool {
        self.sets.iter().any(|one| one.key() == key)
    }

    /// Whether the browser sends its maker no usage data.
    #[must_use]
    pub fn telemetry_is_off(&self) -> bool {
        self.sets.contains(&Setting::TelemetryOff) && self.sets.contains(&Setting::ExperimentsOff)
    }

    /// Whether the browser asks where each download is to be saved, which is
    /// what makes the folder the person's own choice rather than one it picked.
    #[must_use]
    pub fn asks_where_each_download_goes(&self) -> bool {
        self.sets.contains(&Setting::AskWhereEachDownloadGoes)
    }

    /// Whether the browser takes the machine's own road out rather than one
    /// written in here.
    ///
    /// One proxy, machine-wide, is an organisation's or the person's setting, and
    /// a browser holding a second copy of it is a machine with two answers. The
    /// document names none, so whatever the machine publishes is what the browser
    /// uses — which is the browser's own default behaviour, left alone.
    #[must_use]
    pub fn takes_the_machines_proxy(&self) -> bool {
        !self.names(NeverSet::Proxy.key())
    }

    /// Whether the person's own home page, new tab and start page are left as the
    /// upstream set them.
    #[must_use]
    pub fn sets_no_home_page(&self) -> bool {
        !self.names(NeverSet::HomePage.key())
            && !self.names(NeverSet::NewTab.key())
            && !self.names(NeverSet::StartPage.key())
    }

    /// Whether which search engine the browser uses is left as the upstream set
    /// it.
    #[must_use]
    pub fn sets_no_search_engine(&self) -> bool {
        !self.names(NeverSet::SearchEngines.key())
    }

    /// Whether the folder downloads go to is left to the person.
    #[must_use]
    pub fn sets_no_download_folder(&self) -> bool {
        !self.names(NeverSet::DownloadFolder.key())
            && !self.names(NeverSet::DefaultDownloadFolder.key())
    }
}

/// The document, as it is written.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// Every setting, by the key the browser knows it by. A map rather than a
    /// struct, so a key nobody decided arrives here and is refused by name
    /// instead of being dropped by a deserializer.
    policies: BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A document setting exactly these keys to these values.
    fn document(keys: &[(&str, &str)]) -> String {
        let inside: Vec<String> = keys
            .iter()
            .map(|(key, value)| format!("    \"{key}\": {value}"))
            .collect();
        format!("{{\n  \"policies\": {{\n{}\n  }}\n}}\n", inside.join(",\n"))
    }

    /// The three decided keys, each set to `true`.
    fn the_three() -> Vec<(&'static str, &'static str)> {
        Setting::EVERY
            .into_iter()
            .map(|one| (one.key(), "true"))
            .collect()
    }

    /// **The shipped document holds**, and sets the three and only the three.
    #[test]
    fn the_shipped_configuration_holds() {
        let configuration = Configuration::decided().unwrap();
        assert_eq!(configuration.sets(), Setting::EVERY.as_slice());
        assert!(configuration.telemetry_is_off());
        assert!(configuration.asks_where_each_download_goes());
        assert!(configuration.takes_the_machines_proxy());
        assert!(configuration.sets_no_home_page());
        assert!(configuration.sets_no_search_engine());
        assert!(configuration.sets_no_download_folder());
    }

    /// **Every key alo OS has decided it never sets is refused, naming what it
    /// would have taken from the person** — the home page, the search engine, the
    /// proxy, the download folder, and the one key every other could arrive
    /// through.
    #[test]
    fn a_key_alo_os_never_sets_is_refused_naming_what_it_would_take() {
        for never in NeverSet::EVERY {
            let mut keys = the_three();
            keys.push((never.key(), "true"));
            let refused = Configuration::read(&document(&keys)).unwrap_err();
            assert_eq!(refused, NotConfigured::NeverSet(never), "{}", never.key());
            assert!(
                refused.to_string().contains(never.takes()),
                "{}: {refused}",
                never.key()
            );
        }
    }

    /// **A key nobody decided is refused rather than ignored**, and so is a
    /// top-level key that is not `policies`.
    #[test]
    fn a_key_nobody_decided_is_refused() {
        let mut keys = the_three();
        keys.push(("DisableProfileImport", "true"));
        assert_eq!(
            Configuration::read(&document(&keys)).unwrap_err(),
            NotConfigured::NotDecided {
                key: "DisableProfileImport".to_owned()
            }
        );
        for text in [
            "{\"policies\": {}, \"extra\": 1}",
            "{\"settings\": {}}",
            "not a document at all",
            "{}",
        ] {
            assert!(
                matches!(
                    Configuration::read(text),
                    Err(NotConfigured::NotRead { .. })
                ),
                "{text}"
            );
        }
    }

    /// **No setting arrives as text**: a value that is not `true` is refused, so
    /// an address or a search engine cannot ride in on a decided key.
    #[test]
    fn a_setting_that_is_not_true_is_refused() {
        for value in [
            "false",
            "\"https://example.com/\"",
            "1",
            "null",
            "[]",
            "{\"Default\": \"https://example.com/\"}",
        ] {
            let mut keys = the_three();
            keys.retain(|(key, _)| *key != Setting::TelemetryOff.key());
            keys.push((Setting::TelemetryOff.key(), value));
            assert_eq!(
                Configuration::read(&document(&keys)).unwrap_err(),
                NotConfigured::NotTrue {
                    key: Setting::TelemetryOff.key().to_owned()
                },
                "{value}"
            );
        }
    }

    /// **A document that quietly stopped turning telemetry off is refused**, and
    /// so is one missing either of the other two.
    #[test]
    fn a_document_missing_one_of_the_three_is_refused() {
        for one in Setting::EVERY {
            let keys: Vec<(&str, &str)> = the_three()
                .into_iter()
                .filter(|(key, _)| *key != one.key())
                .collect();
            assert_eq!(
                Configuration::read(&document(&keys)).unwrap_err(),
                NotConfigured::Missing {
                    key: one.key().to_owned()
                },
                "{}",
                one.key()
            );
        }
    }
}
