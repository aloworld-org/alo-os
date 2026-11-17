//! What a person can find and install: the decided list, read.
//!
//! [`crate::Shipped`] is what a machine has so it is not helpless. This is the
//! next question and the one that decides whether a machine is a workstation:
//! **what does a person work in all day?** A fresh machine can browse, read
//! and look at a picture, and until this list existed it could not make one.
//!
//! The list itself is data — `crates/alo-software/catalogue.toml`
//! ([`WHERE_IT_IS`]) — because a market screen, the agent's installing verb and
//! anything auditing what this product recommends all have to read one list
//! rather than three that could disagree. This file is what holds the list to
//! being one.
//!
//! # What a catalogue has to be, and how that differs from the shipped list
//!
//! [`Shipped`] holds a **fixed shape**: eight roles, each filled exactly once,
//! because a machine missing one of them is helpless in a way that can be
//! named. A catalogue holds a **growing list**: a category may have two
//! applications in it one day, and adding one is not a change of shape. So this
//! is its own type rather than a second use of that one — the invariant is
//! different, and a type that held both would hold neither.
//!
//! [`Catalogue::read`] refuses ([`NotCatalogued`]) a list unless every entry
//! names an identifier the rented tool could be handed, a source by a name it
//! could be handed, the upstream release it was listed at and its licence —
//! and four things more:
//!
//! - **nothing already shipped.** A screen offering to install Firefox on a
//!   machine that has Firefox is a screen telling a person something untrue, so
//!   an identifier [`Shipped`] already names is refused here. This is the check
//!   that makes the two lists one story instead of two.
//! - **no application twice**, under one category or two.
//! - **nothing a person's own** (`alo_capability::A_PERSONS_OWN`, ADR 0043).
//!   Every [`Category`] is a making-category and none is a terminal; a terminal
//!   reaches a machine through the shipped list, which is where a reader looks
//!   for it. Nobody is stopped from installing one — [`Wanted::by_hand`] takes
//!   any identifier, and a curated list is not the only road.
//! - **no empty category.** A [`Category`] with no entry is a shelf a market
//!   screen would draw with nothing on it, which is a defect a person sees.
//!
//! And **no key this reader does not know**, refused rather than ignored: a key
//! meaning *recommended* or *cannot be removed* would be a promise nothing here
//! keeps.
//!
//! # Installed the way a person installs one
//!
//! [`Listed::wanted`] is a [`Wanted`] like any other, so a catalogued
//! application goes through `installing` and `install`, arrives granted nothing,
//! is offered updates and is removed exactly as anything else is. There is no
//! second road here, and nothing in this crate knows an application came from
//! this list once it is installed.
//!
//! # A name, and no other words
//!
//! No summary and no description, in the data or in this type: what a person
//! reads is their own language, a summary belongs to the application's
//! **source**, already translated by the people who publish it, and an English
//! one written here would stand in twenty-four languages' way and go stale the
//! week the upstream reworded its own.
//!
//! **The name is the one exception, since 2026-11-17.** Until then this said
//! *no name* too, and the market drew identifiers — `org.gimp.GIMP` where a
//! person expects *GNU Image Manipulation Program*. The owner, shown that:
//! *real application names, never identifiers.* A name is not a sentence to
//! translate; it is the makers' own, so each entry carries the one its source
//! publishes, read from Flathub's `name` the day it was written.
//!
//! The obligation that leaves for the screen, when it is built: a category is
//! drawn through `crate::words` like every other string this crate shows, never
//! by [`Category::named`].
//!
//! # English in the refusals, and why
//!
//! [`NotCatalogued`] keeps its English and a `Display`, like [`NotShipped`]: a
//! list that does not hold is this repository's own file contradicting itself,
//! read by whoever is editing it, and never by a person using a machine.

use alo_applications::Application;
use alo_capability::is_a_persons_own;
use serde::Deserialize;

use crate::category::Category;
use crate::fields::{Licence, is_a_release};
use crate::installing::Wanted;
use crate::installing_a_signed_release::{NotASignedRelease, SignedRelease};
use crate::label::Label;
use crate::maker::Maker;
use crate::reach::Reach;
use crate::shipped::{NotShipped, Shipped};
use crate::source::SourceName;

/// The decided list, as it is built into this crate.
pub const THE_CATALOGUE: &str = include_str!("../catalogue.toml");

/// Where the decided list is, from the repository's root — for anything that
/// reads the file rather than this crate.
pub const WHERE_IT_IS: &str = "crates/alo-software/catalogue.toml";

/// The longest name an entry may have, in characters. The longest Flathub
/// name in the catalogue on 2026-11-17 was 30 (*GNU Image Manipulation
/// Program*); anything near this is a sentence, not a name.
pub const THE_LONGEST_NAME: usize = 64;

/// The only `format` this reader understands.
pub const FORMAT: u32 = 1;

/// Why a catalogue of what a person can install was not believed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotCatalogued {
    /// It is not a list in this shape at all, or holds a key nothing reads.
    #[error("the catalogue of what a person can install could not be read: {why}")]
    NotRead {
        /// What the reader said.
        why: String,
    },
    /// A `format` this reader does not understand.
    #[error(
        "the catalogue is in format {format}, and this reader understands only format {FORMAT}"
    )]
    Format {
        /// The format it says it is in.
        format: u32,
    },
    /// A category with nothing in it.
    #[error("the catalogue offers nothing under {}", .category.named())]
    NothingUnder {
        /// The category.
        category: Category,
    },
    /// One application offered twice.
    #[error("{identifier} is in the catalogue twice")]
    Twice {
        /// The application.
        identifier: String,
    },
    /// An application a fresh machine already has.
    #[error(
        "{identifier} is already shipped, so the catalogue would offer to install what this \
         machine has"
    )]
    AlreadyShipped {
        /// The application.
        identifier: String,
    },
    /// An identifier no verb or rented tool could be handed.
    #[error("the {} is named {identifier:?}, which is not an application's identifier", .category.named())]
    NotAnIdentifier {
        /// The category it is under.
        category: Category,
        /// What it was named.
        identifier: String,
    },
    /// A source name the rented tool could not be handed safely.
    #[error("{identifier} comes from {place:?}, which is not a place's name")]
    NotASource {
        /// The application.
        identifier: String,
        /// What the place was named.
        place: String,
    },
    /// A version that is not an upstream release number.
    #[error("{identifier} is listed at {version:?}, which is not a release number")]
    NotAVersion {
        /// The application.
        identifier: String,
        /// What it was listed at.
        version: String,
    },
    /// A name that is empty, longer than a row holds, or more than one line.
    #[error("{identifier} is called {name:?}, which is not a name a row can show")]
    NotAName {
        /// The application.
        identifier: String,
        /// What it was called.
        name: String,
    },
    /// No licence, or one that is not a licence expression.
    #[error("{identifier} names no licence it can be offered under")]
    NoLicence {
        /// The application.
        identifier: String,
    },
    /// A terminal, which belongs in the shipped list and nowhere else.
    #[error(
        "{identifier} is a person's own application (ADR 0043) and the catalogue has no terminal \
         category — a terminal reaches a machine through the shipped list"
    )]
    APersonsOwn {
        /// The application.
        identifier: String,
    },
    /// An entry whose maker's signed release does not hold (ADR 0158): a maker
    /// with no bundle or checksum, an address that is not the maker's, or a
    /// checksum that is not one.
    #[error("{identifier} is from its maker's signed release, which does not hold: {why}")]
    NotASignedRelease {
        /// The application.
        identifier: String,
        /// What was wrong with it.
        why: NotASignedRelease,
    },
    /// A bundle or a checksum on an entry from a place set up in the rented
    /// tool, where neither means anything.
    #[error("{identifier} comes from a place, not a maker, so it may name no bundle or checksum")]
    NotFromAMaker {
        /// The application.
        identifier: String,
    },
    /// The shipped list this catalogue is checked against does not itself hold.
    ///
    /// Unreachable for the two decided lists as they are written — a test in
    /// `crate::shipped` holds the shipped one — and a variant rather than a
    /// panic because this crate does not unwrap outside its tests.
    #[error("the catalogue could not be checked, because the shipped list does not hold: {why}")]
    TheShippedListDoesNotHold {
        /// What was wrong with the shipped list.
        why: NotShipped,
    },
}

/// One application a person can find and install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    /// What a person would make with it.
    category: Category,
    /// The application, by its source's identifier.
    application: Application,
    /// What its makers call it.
    name: String,
    /// Where it is installed from.
    source: SourceName,
    /// The upstream release it was listed at.
    version: String,
    /// Its licence, as its upstream states it, read.
    licence: Licence,
    /// Whether its makers published it at its source themselves.
    verified: bool,
    /// Whether it is an early version, still changing a great deal.
    early: bool,
    /// Its maker's signed release, when it is installed from one (ADR 0158).
    signed: Option<SignedRelease>,
    /// What its makers ask for that reaches past its box.
    reaches: Vec<Reach>,
}

impl Listed {
    /// What a person would make with it.
    #[must_use]
    pub const fn category(&self) -> Category {
        self.category
    }

    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// What its makers call it — **the name a person reads, never the
    /// identifier** (the owner, 2026-11-17).
    ///
    /// As its source publishes it in its own default language: Flathub's
    /// `name` for the entries from Flathub. A product's name is its makers'
    /// and is not translated here.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Where it is installed from.
    #[must_use]
    pub const fn source(&self) -> &SourceName {
        &self.source
    }

    /// The upstream release it was listed at.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Its licence, exactly as the catalogue writes it.
    #[must_use]
    pub fn licence(&self) -> &str {
        self.licence.written()
    }

    /// Whether every part of its licence is an open one.
    ///
    /// **False for anything Flathub records as `LicenseRef-proprietary`**, which
    /// the market labels *not open source* (the owner, 2026-11-17). Read from the
    /// licence by `crate::fields::Licence`, never written beside it, so the label
    /// and the licence cannot disagree.
    #[must_use]
    pub const fn is_open_source(&self) -> bool {
        self.licence.is_open_source()
    }

    /// Whether its makers published it at its source themselves — for Flathub,
    /// the verification Flathub shows on its page, read from
    /// `https://flathub.org/api/v2/verification/<id>/status` when it was listed.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.verified
    }

    /// Whether it is an early version, still changing a great deal.
    #[must_use]
    pub const fn is_early(&self) -> bool {
        self.early
    }

    /// What the market says beside its name, in [`Label::EVERY`]'s order —
    /// nothing, for an open-source application its makers published themselves
    /// that is not an early version.
    #[must_use]
    pub fn labels(&self) -> Vec<Label> {
        Label::EVERY
            .into_iter()
            .filter(|label| match label {
                Label::RunsProgramsOutsideItsBox => {
                    self.reaches.contains(&Reach::ProgramsOutsideItsBox)
                }
                // **Said only where the larger reach is not**: a program that
                // runs outside its box can reach the files as well, and a row
                // that said both would say the lesser twice.
                Label::ReachesYourFiles => {
                    self.reaches.contains(&Reach::YourFiles)
                        && !self.reaches.contains(&Reach::ProgramsOutsideItsBox)
                }
                Label::NotOpenSource => !self.is_open_source(),
                Label::NotVerified => !self.is_verified(),
                Label::EarlyVersion => self.is_early(),
                Label::NoAutomaticUpdates => self.signed_release().is_some(),
            })
            .collect()
    }

    /// What its makers ask for that reaches past its box, as its real metadata
    /// says — empty only for an application that asks for nothing past it.
    #[must_use]
    pub fn reaches(&self) -> &[Reach] {
        &self.reaches
    }

    /// Its maker's signed release, when it is installed from one rather than
    /// from a place set up in the rented tool (ADR 0158).
    ///
    /// **A caller that installs must ask this first**: [`Listed::wanted`] for
    /// such an entry names the maker as the place, which the rented tool does
    /// not know, so it is refused as a place this machine does not install
    /// from rather than installed some other way.
    #[must_use]
    pub const fn signed_release(&self) -> Option<&SignedRelease> {
        self.signed.as_ref()
    }

    /// The installation, as a person asks for any application: the same value,
    /// through the same two steps, arriving granted nothing.
    #[must_use]
    pub fn wanted(&self) -> Wanted {
        Wanted::by_hand(self.application.clone(), self.source.as_str())
            .reaching(self.reaches.clone())
    }
}

/// What a person can find and install, by what they would make with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalogue {
    /// Every entry, in [`Category::EVERY`]'s order and then the file's.
    listed: Vec<Listed>,
}

impl Catalogue {
    /// The decided list, built into this crate, checked against the decided
    /// shipped list.
    ///
    /// # Errors
    /// [`NotCatalogued`], which the two lists as written cannot cause — a test
    /// holds it.
    pub fn decided() -> Result<Self, NotCatalogued> {
        let shipped =
            Shipped::decided().map_err(|why| NotCatalogued::TheShippedListDoesNotHold { why })?;
        Self::read(THE_CATALOGUE, &shipped)
    }

    /// A catalogue, from its text, checked against what a machine ships.
    ///
    /// The shipped list is a parameter rather than read in here so that this
    /// reader can be measured against a list of either kind, and so that the
    /// *nothing already shipped* refusal is exercised by a test rather than
    /// assumed.
    ///
    /// # Errors
    /// [`NotCatalogued`], naming the first thing that does not hold.
    pub fn read(text: &str, shipped: &Shipped) -> Result<Self, NotCatalogued> {
        let written: Written = toml::from_str(text).map_err(|why| NotCatalogued::NotRead {
            why: why.message().to_owned(),
        })?;
        if written.format != FORMAT {
            return Err(NotCatalogued::Format {
                format: written.format,
            });
        }
        let mut listed: Vec<Listed> = Vec::with_capacity(written.application.len());
        for entry in written.application {
            let one = checked(entry)?;
            if listed
                .iter()
                .any(|already| already.application == one.application)
            {
                return Err(NotCatalogued::Twice {
                    identifier: one.application.identifier().to_owned(),
                });
            }
            if shipped
                .every()
                .iter()
                .any(|has| has.application() == &one.application)
            {
                return Err(NotCatalogued::AlreadyShipped {
                    identifier: one.application.identifier().to_owned(),
                });
            }
            listed.push(one);
        }
        for category in Category::EVERY {
            if !listed.iter().any(|one| one.category == category) {
                return Err(NotCatalogued::NothingUnder { category });
            }
        }
        // Kept in the categories' order so a screen drawing the shelves in
        // order does not sort the list itself, and stable within a category so
        // the file's order is the one a person sees.
        listed.sort_by_key(|one| one.category.place());
        Ok(Self { listed })
    }

    /// Every entry, in [`Category::EVERY`]'s order and then the file's.
    #[must_use]
    pub fn every(&self) -> &[Listed] {
        &self.listed
    }

    /// How many applications a person can find.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.listed.len()
    }

    /// What is offered under one category, in the file's order.
    pub fn under(&self, category: Category) -> impl Iterator<Item = &Listed> {
        self.listed
            .iter()
            .filter(move |one| one.category == category)
    }
}

/// One entry, as the file writes it.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    /// What a person would make with it.
    category: Category,
    /// The application's identifier at its source.
    identifier: String,
    /// What its makers call it, as its source publishes it.
    name: String,
    /// The place it is installed from.
    source: String,
    /// The upstream release it was listed at.
    version: String,
    /// Its licence.
    licence: String,
    /// Whether its makers published it at its source themselves. Written on
    /// every entry, so a forgotten one is a refused file rather than a
    /// missing label.
    verified: bool,
    /// Whether it is an early version.
    early: bool,
    /// What its makers ask for that reaches past its box. Written on every
    /// entry, an empty list included, so a forgotten one is a refused file.
    reaches: Vec<Reach>,
    /// Where a maker's own signed bundle is, for an entry whose `source` is a
    /// maker (ADR 0158), and nothing for any other.
    #[serde(default)]
    bundle: Option<String>,
    /// The bundle's SHA-256, beside it.
    #[serde(default)]
    sha256: Option<String>,
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
fn checked(entry: Entry) -> Result<Listed, NotCatalogued> {
    let category = entry.category;
    let not_an_identifier = || NotCatalogued::NotAnIdentifier {
        category,
        identifier: entry.identifier.clone(),
    };
    // Stricter than an argument, as the shipped list is: an identifier that
    // begins like an option is one no upstream publishes.
    if entry.identifier.starts_with('-') {
        return Err(not_an_identifier());
    }
    let application =
        Application::identified(&entry.identifier).map_err(|_| not_an_identifier())?;
    let source = SourceName::checked(&entry.source).ok_or_else(|| NotCatalogued::NotASource {
        identifier: entry.identifier.clone(),
        place: entry.source.clone(),
    })?;
    if !is_a_release(&entry.version) {
        return Err(NotCatalogued::NotAVersion {
            identifier: entry.identifier,
            version: entry.version,
        });
    }
    let name = entry.name.trim();
    let is_a_name = !name.is_empty()
        && name.chars().count() <= THE_LONGEST_NAME
        && !name.chars().any(char::is_control);
    if !is_a_name {
        return Err(NotCatalogued::NotAName {
            identifier: entry.identifier,
            name: entry.name,
        });
    }
    let name = name.to_owned();
    let Some(licence) = Licence::read(&entry.licence) else {
        return Err(NotCatalogued::NoLicence {
            identifier: entry.identifier,
        });
    };
    if is_a_persons_own(application.identifier()) {
        return Err(NotCatalogued::APersonsOwn {
            identifier: entry.identifier,
        });
    }
    let signed = match (Maker::named(&entry.source), entry.bundle, entry.sha256) {
        (None, None, None) => None,
        (None, _, _) => {
            return Err(NotCatalogued::NotFromAMaker {
                identifier: entry.identifier,
            });
        }
        (Some(_), bundle, sha256) => {
            let pinned = SignedRelease::pinned(
                application.clone(),
                &entry.source,
                bundle.as_deref().unwrap_or_default(),
                sha256.as_deref().unwrap_or_default(),
            );
            match pinned {
                Ok(release) => Some(release.reaching(entry.reaches.clone())),
                Err(why) => {
                    return Err(NotCatalogued::NotASignedRelease {
                        identifier: entry.identifier,
                        why,
                    });
                }
            }
        }
    };
    Ok(Listed {
        category,
        application,
        name,
        source,
        version: entry.version,
        licence,
        verified: entry.verified,
        early: entry.early,
        signed,
        reaches: entry.reaches,
    })
}

#[cfg(test)]
#[path = "catalogue_tests.rs"]
mod tests;
