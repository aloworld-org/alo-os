//! The keepers Settings calls, each exactly as its own crate offers it.
//!
//! `alo_appearance::keeping`, `alo_dock::keeping`, `alo_shortcuts::keeping`,
//! `alo_formats::keeping`, `alo_access::keeping` and `alo_applications::keeping`
//! have one shape and six sets of types. This names that shape once, so
//! `crate::settings_kept` walks all six the same road, and
//! holds nothing of its own: every function body is one call into the crate
//! that owns the file.

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

/// What a person turned on for access: `alo_access::keeping`, the person's own
/// `access.toml` and never the machine's sign-in copy.
///
/// **Reduced motion is the one setting Settings changes here**, the owner's yes
/// of 2026-11-07; the rest of the file is read and written back whole, so a
/// switch somebody set some other way is kept as they set it.
#[derive(Debug)]
pub(crate) struct AccessKept;

impl Keeper for AccessKept {
    type Drawn = alo_access::TurnedOn;
    /// `alo_access`' reason, as its words say it: its error carries an
    /// `io::Error`, which cannot be held twice.
    type NotRead = String;
    type NotWritten = alo_access::keeping::FileNotWritten;

    fn shipped() -> alo_access::TurnedOn {
        alo_access::TurnedOn::nothing()
    }

    fn at_sign_in(at: &Path) -> (alo_access::TurnedOn, Option<String>) {
        // **A person who has kept nothing of their own yet starts with what
        // the sign-in screen turns on** (ADR 0155 §6), so a reader turned on
        // at the first start is on here too; once they keep anything, theirs.
        let (turned_on, why) = alo_access::keeping::for_this_person(
            its_folder(at),
            Path::new(alo_access::keeping::THE_MACHINES_FOLDER),
        );
        (turned_on, why.map(|why| why.to_string()))
    }

    fn keep(at: &Path, drawn: &alo_access::TurnedOn) -> Result<(), Self::NotWritten> {
        alo_access::keeping::keep(its_folder(at), drawn)
    }

    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten> {
        alo_access::keeping::keep(its_folder(at), &alo_access::TurnedOn::nothing())
    }

    fn did_not_read(_refused: &Self::NotWritten) -> Option<String> {
        // `alo_access::keeping::keep` writes without reading, so a write it
        // refused never says anything about the file as it was.
        None
    }

    fn not_read_said(why: &String, strings: &Strings) -> String {
        crate::the_settings_words::access_not_read(strings, why)
    }

    fn not_written_said(why: &Self::NotWritten, strings: &Strings) -> String {
        crate::the_settings_words::access_not_written(strings, &why.to_string())
    }
}

/// **The folder `access.toml` is in**: `alo_access::keeping` is handed the
/// folder and names the file itself, where every other keeper here is handed
/// the file. Settings knows a section by its file, so the folder is taken off
/// it here, once.
fn its_folder(at: &Path) -> &Path {
    at.parent().unwrap_or(at)
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

/// What opens what for this person — their default browser and mail
/// application among it: `alo_applications::keeping`, `what-opens-what.toml`.
///
/// **The whole file is read and written back**, so a choice of what opens a
/// kind of file, made some other way, is kept as it was when a person changes
/// their default browser here.
#[derive(Debug)]
pub(crate) struct LinksKept;

impl Keeper for LinksKept {
    type Drawn = alo_applications::Chosen;
    type NotRead = alo_applications::FileNotRead;
    type NotWritten = alo_applications::FileNotWritten;

    fn shipped() -> alo_applications::Chosen {
        alo_applications::Chosen::untouched()
    }

    fn at_sign_in(at: &Path) -> (alo_applications::Chosen, Option<Self::NotRead>) {
        alo_applications::keeping::at_sign_in(at)
    }

    fn keep(at: &Path, drawn: &alo_applications::Chosen) -> Result<(), Self::NotWritten> {
        alo_applications::keeping::keep(at, drawn)
    }

    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten> {
        alo_applications::keeping::put_back_as_shipped(at)
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

/// How this person's dates and times are written: `alo_formats::keeping`.
#[derive(Debug)]
pub(crate) struct FormatsKept;

impl Keeper for FormatsKept {
    type Drawn = alo_formats::Chosen;
    type NotRead = alo_formats::FileNotRead;
    type NotWritten = alo_formats::FileNotWritten;

    fn shipped() -> alo_formats::Chosen {
        alo_formats::Chosen::default()
    }

    fn at_sign_in(at: &Path) -> (alo_formats::Chosen, Option<Self::NotRead>) {
        alo_formats::keeping::at_sign_in(at)
    }

    fn keep(at: &Path, drawn: &alo_formats::Chosen) -> Result<(), Self::NotWritten> {
        alo_formats::keeping::keep(at, drawn)
    }

    fn put_back_as_shipped(at: &Path) -> Result<(), Self::NotWritten> {
        alo_formats::keeping::put_back_as_shipped(at)
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
