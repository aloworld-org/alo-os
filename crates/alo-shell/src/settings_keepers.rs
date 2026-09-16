//! The three keepers Settings calls, each exactly as its own crate offers it.
//!
//! `alo_appearance::keeping`, `alo_dock::keeping` and `alo_shortcuts::keeping`
//! have one shape and three sets of types. This names that shape once, so
//! `crate::settings_kept` walks all three the same road, and holds nothing of
//! its own: every function body is one call into the crate that owns the file.

use std::fmt::Debug;
use std::path::Path;

use alo_appearance::Appearance;
use alo_dock::Dock;
use alo_shortcuts::Shortcuts;
use alo_strings::Strings;

/// A crate that keeps one section's file in the person's folder.
pub(crate) trait Keeper {
    /// What the section draws.
    type Drawn: Clone + Debug + PartialEq;
    /// The crate's refusal of a file that did not read.
    type NotRead: Clone + Debug + PartialEq;
    /// The crate's refusal of a change it did not write.
    type NotWritten: Debug;

    /// The section as the release ships it.
    fn shipped() -> Self::Drawn;

    /// What the section draws at sign-in, and why its file did not read.
    fn at_sign_in(at: &Path) -> (Self::Drawn, Option<Self::NotRead>);

    /// The whole of what is drawn, kept.
    fn keep(at: &Path, drawn: &Self::Drawn) -> Result<(), Self::NotWritten>;

    /// The file replaced with the release's, which is the one door that
    /// replaces a file that did not read.
    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten>;

    /// What is wrong with the file, when a change was not written because it
    /// did not read.
    fn did_not_read(refused: &Self::NotWritten) -> Option<Self::NotRead>;

    /// The file that did not read, in the crate's words.
    fn not_read_said(why: &Self::NotRead, strings: &Strings) -> String;

    /// The change that was not written, in the crate's words.
    fn not_written_said(why: &Self::NotWritten, strings: &Strings) -> String;
}

/// How this machine looks: `alo_appearance::keeping`.
#[derive(Debug)]
pub(crate) struct AppearanceKept;

impl Keeper for AppearanceKept {
    type Drawn = Appearance;
    type NotRead = alo_appearance::FileNotRead;
    type NotWritten = alo_appearance::FileNotWritten;

    fn shipped() -> Appearance {
        Appearance::shipped()
    }

    fn at_sign_in(at: &Path) -> (Appearance, Option<Self::NotRead>) {
        alo_appearance::keeping::at_sign_in(at)
    }

    fn keep(at: &Path, drawn: &Appearance) -> Result<(), Self::NotWritten> {
        alo_appearance::keeping::keep(at, drawn.changes())
    }

    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten> {
        alo_appearance::keeping::put_back_as_shipped(at)
    }

    fn did_not_read(refused: &Self::NotWritten) -> Option<Self::NotRead> {
        refused.did_not_read()
    }

    fn not_read_said(why: &Self::NotRead, strings: &Strings) -> String {
        why.said(strings).into_text()
    }

    fn not_written_said(why: &Self::NotWritten, strings: &Strings) -> String {
        why.said(strings).into_text()
    }
}

/// Which edge the dock is on: `alo_dock::keeping`.
#[derive(Debug)]
pub(crate) struct DockKept;

impl Keeper for DockKept {
    type Drawn = Dock;
    type NotRead = alo_dock::FileNotRead;
    type NotWritten = alo_dock::FileNotWritten;

    fn shipped() -> Dock {
        Dock::shipped()
    }

    fn at_sign_in(at: &Path) -> (Dock, Option<Self::NotRead>) {
        alo_dock::keeping::at_sign_in(at)
    }

    fn keep(at: &Path, drawn: &Dock) -> Result<(), Self::NotWritten> {
        alo_dock::keeping::keep(at, drawn.changes())
    }

    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten> {
        alo_dock::keeping::put_back_as_shipped(at)
    }

    fn did_not_read(refused: &Self::NotWritten) -> Option<Self::NotRead> {
        refused.did_not_read()
    }

    fn not_read_said(why: &Self::NotRead, strings: &Strings) -> String {
        why.said(strings).into_text()
    }

    fn not_written_said(why: &Self::NotWritten, strings: &Strings) -> String {
        why.said(strings).into_text()
    }
}

/// The shortcuts this person changed: `alo_shortcuts::keeping`.
#[derive(Debug)]
pub(crate) struct ShortcutsKept;

impl Keeper for ShortcutsKept {
    type Drawn = Shortcuts;
    type NotRead = alo_shortcuts::FileNotRead;
    type NotWritten = alo_shortcuts::FileNotWritten;

    fn shipped() -> Shortcuts {
        Shortcuts::shipped()
    }

    fn at_sign_in(at: &Path) -> (Shortcuts, Option<Self::NotRead>) {
        alo_shortcuts::keeping::at_sign_in(at)
    }

    fn keep(at: &Path, drawn: &Shortcuts) -> Result<(), Self::NotWritten> {
        alo_shortcuts::keeping::keep(at, drawn.changes())
    }

    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten> {
        alo_shortcuts::keeping::put_back_as_shipped(at)
    }

    fn did_not_read(refused: &Self::NotWritten) -> Option<Self::NotRead> {
        refused.did_not_read()
    }

    fn not_read_said(why: &Self::NotRead, strings: &Strings) -> String {
        why.said(strings).into_text()
    }

    fn not_written_said(why: &Self::NotWritten, strings: &Strings) -> String {
        why.said(strings).into_text()
    }
}
