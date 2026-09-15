//! What a fresh machine has, so it is not helpless: the decided list, read.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 2. The list itself is
//! data — `crates/alo-software/shipped.toml` ([`WHERE_IT_IS`]) — because the
//! installer plan reads it to name the fresh machine's applications to the
//! image, and a list copied into `image/` would be a second list that could
//! disagree with this one. This file is what holds the list to being one.
//!
//! # What a list has to be
//!
//! [`Shipped::read`] refuses ([`NotShipped`]) a list that does not say, for
//! **every** [`Role`] and exactly once, an application by an identifier the
//! rented tool could be handed, a source by a name it could be handed, the
//! upstream version it was decided at and its licence. And two things more:
//!
//! - **the terminal is a person's own** — on `alo_capability::A_PERSONS_OWN`,
//!   so no agent can be granted it and no call can name it (ADR 0043) — and
//!   nothing a person's own is shipped under another role, where a reader of
//!   the list would not look for it;
//! - **no key it does not know.** There is no key meaning *kept* or *cannot be
//!   removed*, and one written in is refused rather than ignored, because a
//!   person may remove any of these, the browser included.
//!
//! # Installed the way a person installs one
//!
//! [`Pinned::wanted`] is a [`Wanted`] like any other, so a fresh machine's
//! applications go through `installing` and `install`, arrive granted nothing,
//! are offered updates and are removed exactly as any application is. There is
//! no second road here, and nothing in this crate knows an application came
//! from this list once it is installed.
//!
//! # English, and why
//!
//! [`NotShipped`] keeps its English and a `Display`, like [`crate::NotShown`]: a
//! list that does not hold is this repository's own file contradicting itself,
//! read by whoever is editing it, and never by a person using a machine.

use alo_applications::Application;
use alo_capability::is_a_persons_own;
use serde::Deserialize;

use crate::installing::Wanted;
use crate::role::Role;
use crate::source::SourceName;

/// The decided list, as it is built into this crate.
pub const THE_LIST: &str = include_str!("../shipped.toml");

/// Where the decided list is, from the repository's root — for the installer
/// plan, which reads the file rather than this crate.
pub const WHERE_IT_IS: &str = "crates/alo-software/shipped.toml";

/// The only `format` this reader understands.
pub const FORMAT: u32 = 1;

/// Why a list of what a fresh machine ships was not believed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotShipped {
    /// It is not a list in this shape at all, or holds a key nothing reads.
    #[error("the list of what a fresh machine ships could not be read: {why}")]
    NotRead {
        /// What the reader said.
        why: String,
    },
    /// A `format` this reader does not understand.
    #[error("the list is in format {format}, and this reader understands only format {FORMAT}")]
    Format {
        /// The format it says it is in.
        format: u32,
    },
    /// A role with nothing to fill it.
    #[error("the list ships no {}", .role.named())]
    Missing {
        /// The role.
        role: Role,
    },
    /// A role filled twice.
    #[error("the list ships two of {}", .role.named())]
    Twice {
        /// The role.
        role: Role,
    },
    /// One application under two roles.
    #[error("{identifier} is shipped twice")]
    SameApplication {
        /// The application.
        identifier: String,
    },
    /// An identifier no verb or rented tool could be handed.
    #[error("the {} is named {identifier:?}, which is not an application's identifier", .role.named())]
    NotAnIdentifier {
        /// The role.
        role: Role,
        /// What it was named.
        identifier: String,
    },
    /// A source name the rented tool could not be handed safely.
    #[error("the {} comes from {place:?}, which is not a place's name", .role.named())]
    NotASource {
        /// The role.
        role: Role,
        /// What the place was named.
        place: String,
    },
    /// A version that is not an upstream release number.
    #[error("the {} is pinned at {version:?}, which is not a release number", .role.named())]
    NotAVersion {
        /// The role.
        role: Role,
        /// What it was pinned at.
        version: String,
    },
    /// No licence, or one that is not a licence expression.
    #[error("the {} names no licence it can be shipped under", .role.named())]
    NoLicence {
        /// The role.
        role: Role,
    },
    /// A terminal an agent could be granted.
    #[error(
        "the terminal {identifier} is not a person's own application, so an agent could be \
         granted it — add it to alo_capability::A_PERSONS_OWN (ADR 0043)"
    )]
    TerminalAnAgentCouldReach {
        /// The terminal.
        identifier: String,
    },
    /// A person's own application shipped under a role other than the terminal.
    #[error("{identifier} is a person's own application and is shipped as the {}", .role.named())]
    APersonsOwnElsewhere {
        /// The role it is shipped under.
        role: Role,
        /// The application.
        identifier: String,
    },
}

/// One application a fresh machine ships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pinned {
    /// What it is there to do.
    role: Role,
    /// The application, by its source's identifier.
    application: Application,
    /// Where it is installed from.
    source: SourceName,
    /// The upstream release it was decided at.
    version: String,
    /// Its licence, as its upstream states it.
    licence: String,
}

impl Pinned {
    /// What it is there to do.
    #[must_use]
    pub const fn role(&self) -> Role {
        self.role
    }

    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// Where it is installed from.
    #[must_use]
    pub const fn source(&self) -> &SourceName {
        &self.source
    }

    /// The upstream release it was decided at.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Its licence.
    #[must_use]
    pub fn licence(&self) -> &str {
        &self.licence
    }

    /// The installation, as a person asks for any application: the same value,
    /// through the same two steps, arriving granted nothing.
    #[must_use]
    pub fn wanted(&self) -> Wanted {
        Wanted::by_hand(self.application.clone(), self.source.as_str())
    }
}

/// What a fresh machine ships: one application for every [`Role`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shipped {
    /// In [`Role::EVERY`]'s order, one each.
    every: [Pinned; 7],
}

impl Shipped {
    /// The decided list, built into this crate.
    ///
    /// # Errors
    /// [`NotShipped`], which the list as written cannot cause — a test holds it.
    pub fn decided() -> Result<Self, NotShipped> {
        Self::read(THE_LIST)
    }

    /// A list, from its text.
    ///
    /// # Errors
    /// [`NotShipped`], naming the first thing that does not hold.
    pub fn read(text: &str) -> Result<Self, NotShipped> {
        let written: Written = toml::from_str(text).map_err(|why| NotShipped::NotRead {
            why: why.message().to_owned(),
        })?;
        if written.format != FORMAT {
            return Err(NotShipped::Format {
                format: written.format,
            });
        }
        let mut filled: [Option<Pinned>; 7] = Default::default();
        for entry in written.application {
            let pinned = checked(entry)?;
            if filled
                .iter()
                .flatten()
                .any(|already| already.application == pinned.application)
            {
                return Err(NotShipped::SameApplication {
                    identifier: pinned.application.identifier().to_owned(),
                });
            }
            let Some(slot) = filled.get_mut(pinned.role.place()) else {
                return Err(NotShipped::Missing { role: pinned.role });
            };
            if slot.is_some() {
                return Err(NotShipped::Twice { role: pinned.role });
            }
            *slot = Some(pinned);
        }
        let mut every = Vec::with_capacity(Role::EVERY.len());
        for (role, slot) in Role::EVERY.into_iter().zip(filled) {
            every.push(slot.ok_or(NotShipped::Missing { role })?);
        }
        let every = every
            .try_into()
            .map_err(|_: Vec<Pinned>| NotShipped::Missing {
                role: Role::Terminal,
            })?;
        Ok(Self { every })
    }

    /// Every application, in [`Role::EVERY`]'s order.
    #[must_use]
    pub const fn every(&self) -> &[Pinned; 7] {
        &self.every
    }

    /// The application shipped for this role.
    #[must_use]
    pub const fn the(&self, role: Role) -> &Pinned {
        let [
            browser,
            files,
            archives,
            editor,
            images,
            documents,
            terminal,
        ] = &self.every;
        match role {
            Role::WebBrowser => browser,
            Role::FileManager => files,
            Role::Archives => archives,
            Role::TextEditor => editor,
            Role::ImageViewer => images,
            Role::DocumentViewer => documents,
            Role::Terminal => terminal,
        }
    }
}

/// One entry, as the file writes it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    /// What it is there to do.
    role: Role,
    /// The application's identifier at its source.
    identifier: String,
    /// The place it is installed from.
    source: String,
    /// The upstream release it was decided at.
    version: String,
    /// Its licence.
    licence: String,
}

/// The file, as it is written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// Which shape the file is in.
    format: u32,
    /// Every entry, in the order written.
    #[serde(default)]
    application: Vec<Entry>,
}

/// One entry, held to what an entry has to be.
fn checked(entry: Entry) -> Result<Pinned, NotShipped> {
    let role = entry.role;
    let not_an_identifier = || NotShipped::NotAnIdentifier {
        role,
        identifier: entry.identifier.clone(),
    };
    // Stricter than an argument: the installer plan reads this file too, and an
    // identifier that begins like an option is one no upstream publishes.
    if entry.identifier.starts_with('-') {
        return Err(not_an_identifier());
    }
    let application =
        Application::identified(&entry.identifier).map_err(|_| not_an_identifier())?;
    let source = SourceName::checked(&entry.source).ok_or_else(|| NotShipped::NotASource {
        role,
        place: entry.source.clone(),
    })?;
    if !is_a_release(&entry.version) {
        return Err(NotShipped::NotAVersion {
            role,
            version: entry.version,
        });
    }
    if !is_a_licence(&entry.licence) {
        return Err(NotShipped::NoLicence { role });
    }
    let persons_own = is_a_persons_own(application.identifier());
    if role == Role::Terminal && !persons_own {
        return Err(NotShipped::TerminalAnAgentCouldReach {
            identifier: entry.identifier,
        });
    }
    if role != Role::Terminal && persons_own {
        return Err(NotShipped::APersonsOwnElsewhere {
            role,
            identifier: entry.identifier,
        });
    }
    Ok(Pinned {
        role,
        application,
        source,
        version: entry.version,
        licence: entry.licence,
    })
}

/// Whether this is a release number: digits in groups, separated by single dots.
fn is_a_release(version: &str) -> bool {
    !version.is_empty()
        && version
            .split('.')
            .all(|group| !group.is_empty() && group.chars().all(|c| c.is_ascii_digit()))
}

/// Whether this reads as a licence expression: letters, digits and the
/// punctuation such an expression uses, on one line.
fn is_a_licence(licence: &str) -> bool {
    !licence.trim().is_empty()
        && licence
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | ' ' | '(' | ')'))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The decided list with one line of it replaced.
    fn replacing(from: &str, to: &str) -> String {
        assert!(THE_LIST.contains(from), "{from}");
        THE_LIST.replacen(from, to, 1)
    }

    #[test]
    fn the_decided_list_holds() {
        let shipped = Shipped::decided().unwrap();
        for role in Role::EVERY {
            assert_eq!(shipped.the(role).role(), role);
        }
    }

    /// **A list missing a role is refused**, naming it — the terminal here,
    /// because a machine without one is the toy `docs/features.md` refuses.
    #[test]
    fn a_list_missing_a_role_is_refused() {
        let without_terminal = THE_LIST
            .split("[[application]]")
            .filter(|part| !part.contains("role = \"terminal\""))
            .collect::<Vec<_>>()
            .join("[[application]]");
        assert_eq!(
            Shipped::read(&without_terminal).unwrap_err(),
            NotShipped::Missing {
                role: Role::Terminal
            }
        );
    }

    #[test]
    fn a_role_filled_twice_or_an_application_shipped_twice_is_refused() {
        let twice = format!(
            "{THE_LIST}\n[[application]]\nrole = \"text-editor\"\nidentifier = \
             \"org.example.Editor\"\nsource = \"flathub\"\nversion = \"1.0\"\nlicence = \"MIT\"\n"
        );
        assert_eq!(
            Shipped::read(&twice).unwrap_err(),
            NotShipped::Twice {
                role: Role::TextEditor
            }
        );
        let same = replacing(
            "identifier = \"org.gnome.Loupe\"",
            "identifier = \"org.gnome.TextEditor\"",
        );
        assert_eq!(
            Shipped::read(&same).unwrap_err(),
            NotShipped::SameApplication {
                identifier: "org.gnome.TextEditor".to_owned()
            }
        );
    }

    /// **The terminal has to be one no agent can be granted** (ADR 0043), and a
    /// person's own application cannot hide under another role.
    #[test]
    fn a_terminal_an_agent_could_be_granted_is_refused() {
        let reachable = replacing(
            "identifier = \"app.devsuite.Ptyxis\"",
            "identifier = \"org.example.Terminal\"",
        );
        assert_eq!(
            Shipped::read(&reachable).unwrap_err(),
            NotShipped::TerminalAnAgentCouldReach {
                identifier: "org.example.Terminal".to_owned()
            }
        );
        let hidden = replacing(
            "identifier = \"org.gnome.TextEditor\"",
            "identifier = \"org.kde.konsole\"",
        );
        assert_eq!(
            Shipped::read(&hidden).unwrap_err(),
            NotShipped::APersonsOwnElsewhere {
                role: Role::TextEditor,
                identifier: "org.kde.konsole".to_owned()
            }
        );
    }

    /// **There is no key meaning kept**: one written in is refused, not
    /// ignored — and so is a role nobody decided.
    #[test]
    fn a_key_or_a_role_nothing_reads_is_refused() {
        let kept = replacing(
            "role = \"web-browser\"",
            "role = \"web-browser\"\nremovable = false",
        );
        assert!(matches!(
            Shipped::read(&kept).unwrap_err(),
            NotShipped::NotRead { .. }
        ));
        let unknown = replacing("role = \"archives\"", "role = \"mail\"");
        assert!(matches!(
            Shipped::read(&unknown).unwrap_err(),
            NotShipped::NotRead { .. }
        ));
        assert_eq!(
            Shipped::read(&replacing("format = 1", "format = 2")).unwrap_err(),
            NotShipped::Format { format: 2 }
        );
    }

    /// **Nothing shaped like an option, a path or a command reaches the rented
    /// tool from this file**, and every entry is pinned to a release under a
    /// licence.
    #[test]
    fn an_entry_that_could_not_be_handed_to_the_rented_tool_is_refused() {
        for (from, to, refused) in [
            (
                "identifier = \"org.gnome.Papers\"",
                "identifier = \"--system\"",
                "NotAnIdentifier",
            ),
            (
                "identifier = \"org.gnome.Papers\"",
                "identifier = \"/usr/bin/sh\"",
                "NotAnIdentifier",
            ),
            (
                "source = \"flathub\"",
                "source = \"--no-deps\"",
                "NotASource",
            ),
            ("source = \"flathub\"", "source = \"a/../b\"", "NotASource"),
            ("version = \"156.0\"", "version = \"latest\"", "NotAVersion"),
            ("version = \"156.0\"", "version = \"156..0\"", "NotAVersion"),
            ("version = \"156.0\"", "version = \"\"", "NotAVersion"),
            ("licence = \"MPL-2.0\"", "licence = \"\"", "NoLicence"),
            (
                "licence = \"MPL-2.0\"",
                "licence = \"see; rm\"",
                "NoLicence",
            ),
        ] {
            let refusal = Shipped::read(&replacing(from, to)).unwrap_err();
            assert!(
                format!("{refusal:?}").starts_with(refused),
                "{to}: {refusal:?}"
            );
        }
    }
}
