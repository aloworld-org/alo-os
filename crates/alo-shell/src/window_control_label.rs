//! **The shell's font system**, and the one refusal loading it can produce.
//!
//! # Its name is wrong and the rename is a change of its own
//!
//! This file was *externalized control names shaped without consulting host
//! fonts or clients* — the label machinery for the three window control tiles.
//! The tiles are gone, and what the tiles happened to own turned out to be the
//! only [`FontSystem`] in this crate: `grep -rn "FontSystem"
//! crates/alo-shell/src/*.rs` finds one holder and it is here.
//!
//! So `WindowControlLabels` is threaded as `&mut` through sign-in, the lock
//! screen, recovery, settings, the record window, notifications, the egress
//! indicator, the approval surface, booting and every nested surface — **220
//! mentions across 85 files**, almost none of them about window controls. A
//! name that says *window control* on the thing that draws every word on the
//! machine is a trap: anybody retiring the window controls from the name alone
//! deletes the text machinery.
//!
//! **The rename is owed and is deliberately not in this change.** 85 files of
//! mechanical renaming inside a deletion would make the deletion unreviewable,
//! and the same is true of `NestedControlInput` beside it. Both belong in one
//! change about names.
//!
//! # What went
//!
//! `WindowControlLabel`, `LabelGeometry`, `prepare`, `prepare_said`, `shape`
//! and `shape_said` prepared and rastered one control's wording into a bounded
//! box, for a tile too small to show it. The external window edge draws its
//! title through `crate::painted_text` and keeps the whole string for a reader
//! — `crate::window_edge_title` — so none of it has a caller.

use cosmic_text::FontSystem;

/// Refusal before a prepared label can be painted. Diagnostic, not UI wording.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowControlLabelError {
    /// No valid font face was supplied.
    #[error("invalid native label font")]
    Font,
}

/// The shell's only font database, loaded once from faces it was given.
pub struct WindowControlLabels {
    /// Loaded once, with only explicitly supplied fonts.
    pub(crate) fonts: FontSystem,
}

impl WindowControlLabels {
    /// Use the bundled, OFL-licensed Inter face specified by the design brief.
    pub fn new() -> Result<Self, WindowControlLabelError> {
        Self::from_fonts([include_bytes!("../fonts/Inter.ttf").to_vec()])
    }

    /// Supply an explicit primary font followed by fallback fonts. Every entry
    /// must parse into at least one face. The first face's family is the default.
    /// This permits additional scripts without reading unrelated host fonts.
    pub fn from_fonts(
        fonts: impl IntoIterator<Item = Vec<u8>>,
    ) -> Result<Self, WindowControlLabelError> {
        let mut db = cosmic_text::fontdb::Database::new();
        for data in fonts {
            let before = db.faces().count();
            db.load_font_data(data);
            if db.faces().count() == before {
                return Err(WindowControlLabelError::Font);
            }
        }
        let family = db
            .faces()
            .next()
            .and_then(|face| face.families.first())
            .map(|(name, _)| name.clone())
            .ok_or(WindowControlLabelError::Font)?;
        db.set_sans_serif_family(family);
        Ok(Self {
            fonts: FontSystem::new_with_locale_and_db("en-US".into(), db),
        })
    }
}
