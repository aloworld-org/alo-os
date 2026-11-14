//! **The applications installed on this computer, by name**, as one line
//! between the person's shell and the agent, and as the words a model is told.
//!
//! The walk of 2026-11-14: VSCodium was installed as `com.vscodium.codium`, a
//! model asked to open it sent `vscodium`, and the call was refused as *not
//! installed*; asked again, alo installed something from Flathub. #1249 made a
//! name reach the application it means. This is the other half: **the model is
//! told what is installed, by the name a person calls each application and the
//! identifier the machine knows it by**, so it names the right one the first
//! time.
//!
//! ```text
//! alo-their-applications:{"com.vscodium.codium":"VSCodium","org.gnome.Geary":"Geary"}
//! ```
//!
//! It is the seventh line on the asking socket, after whether the person chose
//! full access (`docs/contracts/asking-socket.md`), for the reason every line
//! is after the last one an older agent reads.
//!
//! # Names, and nothing else
//!
//! *The owner, 2026-11-14: "Yes, send app names."* Each entry is an
//! application's identifier and the name its desktop entry gives it, and
//! **nothing else**: no file, no window, no document, no icon, nothing about
//! when it was last used. With an online provider this list leaves the machine
//! inside the question the turn asks, as the folders granted and the person's
//! home already do, so it is under the indicator like every question.
//!
//! # Offered at invocation, never watched
//!
//! It is what the person's shell already holds to draw the Dock and the
//! launcher, read **at the moment they send a request** and sent with that
//! request only — once per turn, like the window offered and the folders. It
//! is not a verb an agent can call whenever it likes, and nothing reads it in
//! the background. [ADR 0001](../../../docs/decisions/0001-the-capability-model.md)'s
//! amendment of 2026-11-14 holds the argument.
//!
//! # A hint, never a capability
//!
//! **Knowing an application is installed allows nothing.** Every application
//! verb is still decided by `alo-agentd` against the grants it reads itself,
//! and the installed list it reads itself.

use serde_json::{Map, Value};

/// The marker the line begins with.
pub const THE_APPLICATIONS_INSTALLED: &str = "alo-their-applications:";

/// **The most applications a line names.** A machine with more is told the
/// first this many, by identifier; the bound refuses a line no shell writes.
pub const THE_MOST_APPLICATIONS: usize = 512;

/// The longest identifier a line names, in bytes: a desktop file ID is a bus
/// name at most, and a bus name is at most 255 bytes.
pub const THE_LONGEST_IDENTIFIER: usize = 255;

/// The longest name a line gives an application, in characters.
pub const THE_LONGEST_NAME: usize = 128;

/// The most a line may be before it is read at all, in bytes: every
/// identifier and name at its longest, escaped, and the envelope.
pub const THE_LONGEST_APPLICATIONS_LINE: usize =
    THE_MOST_APPLICATIONS * (2 * THE_LONGEST_IDENTIFIER + 6 * 4 * THE_LONGEST_NAME + 8) + 64;

/// The applications installed on this computer: each by its identifier, and
/// the name a person calls it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheApplicationsInstalled {
    /// Each application's identifier and name, sorted by identifier.
    applications: Vec<(String, String)>,
}

/// Why a line could not be read as the applications installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTheApplications {
    /// The line does not begin with [`THE_APPLICATIONS_INSTALLED`].
    NotMarked,
    /// Longer than a shell writes.
    TooLong(usize),
    /// Not the JSON a shell writes.
    NotReadable(String),
}

impl std::fmt::Display for NotTheApplications {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMarked => write!(
                f,
                "the line after full access is not the applications installed"
            ),
            Self::TooLong(bytes) => write!(
                f,
                "the applications installed are described in {bytes} bytes, more than a shell \
                 writes, so they were not read"
            ),
            Self::NotReadable(why) => {
                write!(f, "the applications installed could not be read: {why}")
            }
        }
    }
}

impl std::error::Error for NotTheApplications {}

/// Whether `identifier` is one a line may name: one word, within the bound.
fn an_identifier_a_line_may_name(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.len() <= THE_LONGEST_IDENTIFIER
        && !identifier
            .chars()
            .any(|it| it.is_whitespace() || it.is_control())
}

/// Whether `name` is one a line may give an application: one line, within the
/// bound.
fn a_name_a_line_may_give(name: &str) -> bool {
    !name.trim().is_empty()
        && name.chars().count() <= THE_LONGEST_NAME
        && !name.chars().any(char::is_control)
}

impl TheApplicationsInstalled {
    /// **These applications**, each by its identifier and the name its entry
    /// gives it — or its identifier, where its entry gives none.
    ///
    /// One that could not travel is left out, and the others still go; at
    /// most [`THE_MOST_APPLICATIONS`], the first by identifier.
    #[must_use]
    pub fn of<'a>(applications: impl IntoIterator<Item = (&'a str, Option<&'a str>)>) -> Self {
        let mut applications: Vec<(String, String)> = applications
            .into_iter()
            .filter(|(identifier, _)| an_identifier_a_line_may_name(identifier))
            .map(|(identifier, name)| {
                let name = name
                    .map(str::trim)
                    .filter(|name| a_name_a_line_may_give(name))
                    .unwrap_or(identifier);
                (identifier.to_owned(), name.to_owned())
            })
            .collect();
        applications.sort_unstable();
        applications.dedup_by(|one, other| one.0 == other.0);
        applications.truncate(THE_MOST_APPLICATIONS);
        Self { applications }
    }

    /// Each application's identifier and name, sorted by identifier.
    #[must_use]
    pub fn applications(&self) -> &[(String, String)] {
        &self.applications
    }

    /// **The line a shell writes**, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        let named: Map<String, Value> = self
            .applications
            .iter()
            .map(|(identifier, name)| (identifier.clone(), Value::String(name.clone())))
            .collect();
        format!("{THE_APPLICATIONS_INSTALLED}{}", Value::Object(named))
    }

    /// **The line read back**, with or without its newline.
    ///
    /// # Errors
    /// [`NotTheApplications`], for anything a current shell would not write:
    /// not one object of names, an identifier or a name that is not one line
    /// within its bound, or more than [`THE_MOST_APPLICATIONS`].
    pub fn read(line: &str) -> Result<Self, NotTheApplications> {
        let line = line.trim_end_matches(['\n', '\r']);
        if line.len() > THE_LONGEST_APPLICATIONS_LINE {
            return Err(NotTheApplications::TooLong(line.len()));
        }
        let payload = line
            .strip_prefix(THE_APPLICATIONS_INSTALLED)
            .ok_or(NotTheApplications::NotMarked)?;
        let not = |why: &str| NotTheApplications::NotReadable(why.to_owned());
        let value: Value = serde_json::from_str(payload).map_err(|why| not(&why.to_string()))?;
        let named = value
            .as_object()
            .ok_or_else(|| not("it is not an object of names by identifier"))?;
        if named.len() > THE_MOST_APPLICATIONS {
            return Err(not("it names more applications than a shell writes"));
        }
        let mut applications = Vec::with_capacity(named.len());
        for (identifier, name) in named {
            if !an_identifier_a_line_may_name(identifier) {
                return Err(not("an identifier in it is not one a shell writes"));
            }
            let name = name
                .as_str()
                .filter(|name| a_name_a_line_may_give(name))
                .ok_or_else(|| not("a name in it is not one line a shell writes"))?;
            applications.push((identifier.clone(), name.to_owned()));
        }
        applications.sort_unstable();
        Ok(Self { applications })
    }

    /// **The line a shell writes for this request**, without its newline: the
    /// applications, or the marker alone where the shell knows none.
    ///
    /// **Always written**, as every line before it is, so an agent that reads
    /// the line finds one where a current shell sent it and never waits for a
    /// line that is not coming.
    #[must_use]
    pub fn the_line_for(installed: Option<&Self>) -> String {
        installed.map_or_else(|| THE_APPLICATIONS_INSTALLED.to_owned(), Self::written)
    }

    /// **The line read back**: the applications, or [`None`] for the marker
    /// alone.
    ///
    /// # Errors
    /// [`NotTheApplications`], as [`Self::read`].
    pub fn read_the_line(line: &str) -> Result<Option<Self>, NotTheApplications> {
        let bare = line.trim_end_matches(['\n', '\r']);
        if bare == THE_APPLICATIONS_INSTALLED {
            return Ok(None);
        }
        Self::read(bare).map(Some)
    }

    /// **The words a model is told**: each application by name and
    /// identifier, that the identifier is what to name, and that knowing an
    /// application is here allows nothing. Nothing at all where none is.
    #[must_use]
    pub fn told(&self) -> String {
        if self.applications.is_empty() {
            return String::new();
        }
        let mut text = String::from(
            "\n\nThe applications installed on this computer, each by the name the person calls \
             it and, in brackets, its identifier:\n",
        );
        for (identifier, name) in &self.applications {
            text.push_str("  ");
            text.push_str(name);
            text.push_str(" (");
            text.push_str(identifier);
            text.push_str(")\n");
        }
        text.push_str(
            "When the person names an application, or asks for one by what it does, name the \
             identifier of the one above that they mean. An application that is not listed is \
             not installed: do not install one in its place unless the person asks for that. \
             Knowing an application is installed does not allow reaching it: what you may reach \
             is decided exactly as for any other application.",
        );
        text
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// VSCodium and the mail client, the walk's machine.
    fn the_walks() -> TheApplicationsInstalled {
        TheApplicationsInstalled::of([
            ("org.gnome.Geary", Some("Geary")),
            ("com.vscodium.codium", Some("VSCodium")),
        ])
    }

    /// **What a shell writes is what an agent reads**, whole, and the marker
    /// alone is a shell that knows no applications.
    #[test]
    fn the_line_reads_back_as_it_was_written() {
        let written = the_walks().written();
        assert_eq!(
            written,
            r#"alo-their-applications:{"com.vscodium.codium":"VSCodium","org.gnome.Geary":"Geary"}"#
        );
        assert_eq!(
            TheApplicationsInstalled::read(&written).unwrap(),
            the_walks()
        );
        assert_eq!(
            TheApplicationsInstalled::read_the_line(&format!("{written}\n")).unwrap(),
            Some(the_walks())
        );
        assert_eq!(
            TheApplicationsInstalled::read_the_line(&format!("{THE_APPLICATIONS_INSTALLED}\n"))
                .unwrap(),
            None
        );
        assert_eq!(
            TheApplicationsInstalled::the_line_for(None),
            THE_APPLICATIONS_INSTALLED
        );
    }

    /// **The model is told each application by name and identifier**, to
    /// name the identifier, not to install a stand-in, and that knowing allows
    /// nothing.
    #[test]
    fn the_model_is_told_names_and_identifiers_and_that_knowing_allows_nothing() {
        let told = the_walks().told();
        assert!(
            told.contains("  VSCodium (com.vscodium.codium)\n"),
            "{told}"
        );
        assert!(told.contains("  Geary (org.gnome.Geary)\n"), "{told}");
        assert!(told.contains("name the identifier"), "{told}");
        assert!(told.contains("do not install one in its place"), "{told}");
        assert!(told.contains("does not allow reaching it"), "{told}");
        assert_eq!(TheApplicationsInstalled::of([]).told(), "");
    }

    /// **The whole text a turn shows carries the applications**, before what
    /// may be reached and before the request — and a turn whose shell said
    /// nothing about them is shown exactly what it was shown before.
    #[test]
    fn the_text_a_turn_shows_carries_the_applications_before_the_request() {
        let verbs = crate::testing::the_verbs();
        let request = "open hello.txt in vscodium";
        let granted = crate::WhatItMayReach::these([std::path::Path::new("/var/home/anna")]);
        let shown = crate::shown_to_a_turn(
            &verbs,
            &granted.clone().with_these_applications(Some(the_walks())),
            request,
        );
        let here = shown.find("VSCodium (com.vscodium.codium)").unwrap();
        let reach = shown.find("The folders you may reach").unwrap();
        let asked = shown.rfind(request).unwrap();
        assert!(here < reach && reach < asked, "{shown}");
        assert_eq!(
            crate::shown_to_a_turn(
                &verbs,
                &granted.clone().with_these_applications(None),
                request
            ),
            crate::shown_to_a_turn(&verbs, &granted, request),
            "a turn told no applications was shown something new"
        );
    }

    /// **Only names travel**: an entry with no name goes by its identifier,
    /// and one that is not one line is left out without costing the others.
    #[test]
    fn a_name_that_cannot_travel_falls_back_and_a_bad_identifier_is_left_out() {
        let some = TheApplicationsInstalled::of([
            ("org.kde.dolphin", None),
            ("org.gimp.GIMP", Some("GNU Image\nManipulation")),
            ("not one word", Some("Nothing")),
            ("", Some("Empty")),
        ]);
        assert_eq!(
            some.applications(),
            [
                ("org.gimp.GIMP".to_owned(), "org.gimp.GIMP".to_owned()),
                ("org.kde.dolphin".to_owned(), "org.kde.dolphin".to_owned()),
            ]
        );
    }

    /// **Refused, in words, rather than half read**: a line a shell would not
    /// write is not shown to a model as though it were the person's machine.
    #[test]
    fn a_line_no_shell_writes_is_refused() {
        for line in [
            "alo-full-access:yes",
            "alo-their-applications:not json",
            r#"alo-their-applications:["com.vscodium.codium"]"#,
            r#"alo-their-applications:{"com.vscodium.codium":1}"#,
            r#"alo-their-applications:{"com vscodium":"VSCodium"}"#,
            r#"alo-their-applications:{"com.vscodium.codium":"VS\nCodium"}"#,
            r#"alo-their-applications:{"com.vscodium.codium":"  "}"#,
        ] {
            assert!(TheApplicationsInstalled::read(line).is_err(), "{line}");
        }
        let long = format!(
            "{THE_APPLICATIONS_INSTALLED}{}",
            "x".repeat(THE_LONGEST_APPLICATIONS_LINE)
        );
        assert!(matches!(
            TheApplicationsInstalled::read(&long),
            Err(NotTheApplications::TooLong(_))
        ));
    }

    /// **The most a shell writes fits the bound a line is read within.**
    #[test]
    fn the_most_a_shell_writes_is_read() {
        let identifiers: Vec<String> = (0..THE_MOST_APPLICATIONS + 3)
            .map(|n| format!("org.example.A{n:0>240}"))
            .collect();
        let name = "\u{1F600}".repeat(THE_LONGEST_NAME);
        let many = TheApplicationsInstalled::of(
            identifiers
                .iter()
                .map(|identifier| (identifier.as_str(), Some(name.as_str()))),
        );
        assert_eq!(many.applications().len(), THE_MOST_APPLICATIONS);
        let written = many.written();
        assert!(written.len() <= THE_LONGEST_APPLICATIONS_LINE);
        assert_eq!(TheApplicationsInstalled::read(&written).unwrap(), many);
    }
}
