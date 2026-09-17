//! The rented list of every keyboard this machine has, read rather than
//! written.
//!
//! `xkeyboard-config` ships the layouts, their variants and the options that
//! turn a key into a compose key or a layout switch, and it ships a list of all
//! of them for exactly this purpose. alo OS rents that list: **no layout name
//! is invented here, and a layout this file does not name cannot be added**
//! ([`crate::Refused::NotAKeyboardWeHave`]). That is what makes
//! [`crate::offering`]'s promise checkable rather than a table somebody typed
//! once.
//!
//! # The shape of the file
//!
//! Sections opened by a line beginning `!`, entries of a code and a
//! description:
//!
//! ```text
//! ! layout
//!   de              German
//!   gr              Greek
//!
//! ! variant
//!   intl            us: English (US, intl., with dead keys)
//!
//! ! option
//!   grp:alt_space_toggle Alt+Space
//! ```
//!
//! A variant's description begins with the layout it belongs to, which is how
//! `us(intl)` is told from `de(intl)` — one exists and one does not.
//!
//! # A file that is not there is refused and never guessed at
//!
//! If the list cannot be read, no layout can be checked, so nothing is added
//! and the person is told ([`crate::FileNotRead`] is about their own file;
//! this is about the machine's). Answering *yes, that layout exists* without
//! having read the list would be this crate inventing keyboard data, which is
//! the one thing its plan forbids.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::layout::Layout;

/// Where `xkeyboard-config` keeps the list on a machine alo OS ships.
pub const THE_RULES: &str = "/usr/share/X11/xkb/rules/evdev.lst";

/// Every keyboard, variant and option the rented data has on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rented {
    /// Every layout name.
    layouts: BTreeSet<String>,
    /// Every variant, as the layout it belongs to and its name.
    variants: BTreeSet<(String, String)>,
    /// Every option, such as `grp:alt_space_toggle`.
    options: BTreeSet<String>,
}

impl Rented {
    /// The list, read from where the rented package keeps it.
    ///
    /// # Errors
    /// [`NotRented`], naming the file, when it is not there, cannot be read, or
    /// holds no layouts at all.
    pub fn read() -> Result<Self, NotRented> {
        Self::read_at(Path::new(THE_RULES))
    }

    /// The list, read from a file named here.
    ///
    /// A path rather than only the constant above, because a test reads the
    /// one this machine actually has and a session on a machine that keeps it
    /// elsewhere is handed the path rather than patching a package.
    ///
    /// # Errors
    /// [`NotRented`], naming the file.
    pub fn read_at(at: &Path) -> Result<Self, NotRented> {
        match std::fs::read_to_string(at) {
            Ok(text) => Self::read_text(&text, at),
            Err(why) if why.kind() == io::ErrorKind::NotFound => Err(NotRented {
                at: at.to_owned(),
                why: NotAList::NotThere,
            }),
            Err(why) => Err(NotRented {
                at: at.to_owned(),
                why: NotAList::Disk(why.kind()),
            }),
        }
    }

    /// The list, read from text that came from `at`.
    ///
    /// # Errors
    /// [`NotRented`] when the text names no layouts, which is what a truncated
    /// or replaced file looks like.
    pub fn read_text(text: &str, at: &Path) -> Result<Self, NotRented> {
        let mut layouts = BTreeSet::new();
        let mut variants = BTreeSet::new();
        let mut options = BTreeSet::new();
        let mut section = None;
        for line in text.lines() {
            let line = line.trim_end();
            if let Some(named) = line.strip_prefix('!') {
                section = named.split_whitespace().next().map(ToOwned::to_owned);
                continue;
            }
            if !line.starts_with(char::is_whitespace) {
                continue;
            }
            let mut words = line.split_whitespace();
            let (Some(code), Some(section)) = (words.next(), section.as_deref()) else {
                continue;
            };
            match section {
                "layout" => {
                    layouts.insert(code.to_owned());
                }
                "variant" => {
                    let rest = words.collect::<Vec<_>>().join(" ");
                    if let Some((of, _)) = rest.split_once(':') {
                        variants.insert((of.trim().to_owned(), code.to_owned()));
                    }
                }
                "option" => {
                    options.insert(code.to_owned());
                }
                _ => {}
            }
        }
        if layouts.is_empty() {
            return Err(NotRented {
                at: at.to_owned(),
                why: NotAList::NoKeyboardsInIt,
            });
        }
        Ok(Self {
            layouts,
            variants,
            options,
        })
    }

    /// Whether this machine has that keyboard — the layout, and the variant
    /// when one is named.
    #[must_use]
    pub fn has(&self, layout: &Layout) -> bool {
        if !self.layouts.contains(layout.name()) {
            return false;
        }
        match layout.variant() {
            None => true,
            Some(variant) => self
                .variants
                .contains(&(layout.name().to_owned(), variant.to_owned())),
        }
    }

    /// Whether this machine has that option, such as `grp:alt_space_toggle`.
    #[must_use]
    pub fn has_option(&self, option: &str) -> bool {
        self.options.contains(option)
    }

    /// How many layouts the rented data names, for a test that wants to know it
    /// read a whole file rather than the first line of one.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.layouts.len()
    }
}

/// The machine's own list of keyboards could not be read, so no keyboard can be
/// checked and none is added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRented {
    /// The file.
    at: PathBuf,
    /// What was wrong with it.
    why: NotAList,
}

impl NotRented {
    /// The file that could not be read.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// What was wrong with it, for whoever is fixing the machine.
    #[must_use]
    pub fn why(&self) -> &NotAList {
        &self.why
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> crate::words::Word {
        crate::words::KEYBOARDS_NOT_THERE
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(
            &self.word().key(),
            &alo_strings::Filling::of("path", self.at.display().to_string()),
        )
    }
}

/// Why the machine's own list of keyboards was not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAList {
    /// The file is not on this machine.
    NotThere,
    /// The disk would not give it up.
    Disk(io::ErrorKind),
    /// It was read and names no keyboards, so it is not that list any more.
    NoKeyboardsInIt,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// A few lines shaped like the rented file.
    const A_LIST: &str = "\
! model
  pc105           Generic 105-key PC
! layout
  de              German
  gr              Greek
  us              English (US)
! variant
  intl            us: English (US, intl., with dead keys)
  polytonic       gr: Greek (polytonic)
! option
  grp:alt_space_toggle Alt+Space
  compose:menu         Menu
";

    fn somewhere() -> PathBuf {
        PathBuf::from("/usr/share/X11/xkb/rules/evdev.lst")
    }

    /// The sections are read apart, and a variant belongs to its own layout.
    #[test]
    fn layouts_variants_and_options_are_read_apart() {
        let rented = Rented::read_text(A_LIST, &somewhere()).unwrap();
        assert_eq!(rented.how_many(), 3);
        assert!(rented.has(&Layout::named("de").unwrap()));
        assert!(rented.has(&Layout::variant_of("us", "intl").unwrap()));
        assert!(rented.has(&Layout::variant_of("gr", "polytonic").unwrap()));
        assert!(rented.has_option("grp:alt_space_toggle"));
        assert!(rented.has_option("compose:menu"));
    }

    /// **A layout the rented data does not name is not one**, and neither is
    /// somebody else's variant worn by a layout that has no such variant.
    #[test]
    fn a_keyboard_the_rented_data_does_not_name_is_not_one() {
        let rented = Rented::read_text(A_LIST, &somewhere()).unwrap();
        assert!(!rented.has(&Layout::named("qwertz").unwrap()));
        assert!(!rented.has(&Layout::variant_of("de", "intl").unwrap()));
        assert!(!rented.has(&Layout::variant_of("us", "polytonic").unwrap()));
        assert!(!rented.has_option("grp:win_space_toggle"));
    }

    /// **A file that is not the list is refused rather than read as an empty
    /// machine**, because an empty answer would refuse every keyboard while
    /// looking like a machine that simply has none.
    #[test]
    fn a_file_that_names_no_keyboards_is_refused_and_says_where() {
        let refused = Rented::read_text("# nothing here\n", &somewhere()).unwrap_err();
        assert_eq!(refused.why(), &NotAList::NoKeyboardsInIt);
        assert_eq!(refused.at(), somewhere());
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug());
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(said.text().contains("evdev.lst"), "{said}");
    }

    /// A file that is not there is the same refusal, naming the same file.
    #[test]
    fn a_list_that_is_not_on_this_machine_is_refused() {
        let nowhere = somewhere().join("not-a-directory").join("evdev.lst");
        let refused = Rented::read_at(&nowhere).unwrap_err();
        assert!(matches!(
            refused.why(),
            NotAList::NotThere | NotAList::Disk(_)
        ));
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains(&nowhere.display().to_string())
        );
    }
}
