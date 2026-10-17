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
//! [`Shipped`] holds a **fixed shape**: seven roles, each filled exactly once,
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
//! # No English, here or in the file
//!
//! No name, no summary, no description — not in the data and not in this type.
//! What a person reads is their own language, and the name and summary of an
//! application belong to its **source**, already translated by the people who
//! publish it and fetched when a person browses. An English summary written
//! into this repository would be a hardcoded string standing in twenty-four
//! languages' way, and it would go stale the week the upstream reworded its
//! own. What this list holds is only what the machine has to *decide* with.
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
use crate::fields::{is_a_licence, is_a_release};
use crate::installing::Wanted;
use crate::shipped::{NotShipped, Shipped};
use crate::source::SourceName;

/// The decided list, as it is built into this crate.
pub const THE_CATALOGUE: &str = include_str!("../catalogue.toml");

/// Where the decided list is, from the repository's root — for anything that
/// reads the file rather than this crate.
pub const WHERE_IT_IS: &str = "crates/alo-software/catalogue.toml";

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
    /// Where it is installed from.
    source: SourceName,
    /// The upstream release it was listed at.
    version: String,
    /// Its licence, as its upstream states it.
    licence: String,
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
    /// The place it is installed from.
    source: String,
    /// The upstream release it was listed at.
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
    if !is_a_licence(&entry.licence) {
        return Err(NotCatalogued::NoLicence {
            identifier: entry.identifier,
        });
    }
    if is_a_persons_own(application.identifier()) {
        return Err(NotCatalogued::APersonsOwn {
            identifier: entry.identifier,
        });
    }
    Ok(Listed {
        category,
        application,
        source,
        version: entry.version,
        licence: entry.licence,
    })
}

#[cfg(test)]
#[path = "catalogue_tests.rs"]
mod tests;
