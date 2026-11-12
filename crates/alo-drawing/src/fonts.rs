//! **The faces alo draws in, bundled, and the one font database made of them.**
//!
//! Compiled into the binary from `crates/alo-drawing/fonts/`, whose
//! `README.md` carries each file's licence, upstream revision and hash. A
//! database made here **never reads the machine's own fonts**: a machine with
//! other fonts installed would otherwise draw other letters from the same code,
//! which is right for the desktop and right for a Flatpak too.
//!
//! Moved out of `alo-shell`'s `window_control_label.rs`, where the reasons for
//! each face were first written, by task 2 of
//! `docs/autonomy/the-alo-ui-kit-plan.md`.

use cosmic_text::FontSystem;

/// Manrope, variable: alo's own face (`crate::painted_text::ALOS_OWN_FACE`).
pub const MANROPE: &[u8] = include_bytes!("../fonts/Manrope.ttf");
/// Manrope at 600, the design's `Label/Medium`.
pub const MANROPE_SEMIBOLD: &[u8] = include_bytes!("../fonts/Manrope-SemiBold.ttf");
/// Manrope at 400, the design's `Body/Large`.
pub const MANROPE_REGULAR: &[u8] = include_bytes!("../fonts/Manrope-Regular.ttf");
/// Manrope at 500, the window panel's picture labels.
pub const MANROPE_MEDIUM: &[u8] = include_bytes!("../fonts/Manrope-Medium.ttf");
/// Inter, variable: the fallback for a script Manrope does not cover.
pub const INTER: &[u8] = include_bytes!("../fonts/Inter.ttf");
/// Geist Mono, variable: the terminal's face.
pub const GEIST_MONO: &[u8] = include_bytes!("../fonts/GeistMono.ttf");
/// Geist Mono at 500, the design's `Terminal/Command`.
pub const GEIST_MONO_MEDIUM: &[u8] = include_bytes!("../fonts/GeistMono-Medium.ttf");
/// Geist Mono at 700, what a bold terminal cell asks for.
pub const GEIST_MONO_BOLD: &[u8] = include_bytes!("../fonts/GeistMono-Bold.ttf");

/// **Every face alo draws in, in the order a database must load them.**
///
/// **The variable files first**, because the first face's family becomes the
/// database's sans-serif ([`a_font_system`]), and that must stay Manrope: a
/// monospace face in front of it would silently change the typeface of every
/// sentence that asks for sans-serif. Then **the static cuts the design asks
/// for**, because a variable font is registered as one face at its default
/// instance and cosmic-text honours a family only at an exact weight — so
/// without them neither of alo's typefaces could be drawn at the weight its own
/// design specifies (`docs/quirks.md`). `alo-shell`'s
/// `window_control_label.rs` keeps the measurements that found each one.
pub const ALOS_OWN_FACES: [&[u8]; 8] = [
    MANROPE,
    INTER,
    GEIST_MONO,
    MANROPE_SEMIBOLD,
    MANROPE_REGULAR,
    MANROPE_MEDIUM,
    GEIST_MONO_MEDIUM,
    GEIST_MONO_BOLD,
];

/// A font file that held no face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAFace;

/// **A font database of exactly these faces**, and nothing of the machine's.
///
/// Every entry must parse into at least one face, and the first face's family
/// becomes the database's sans-serif. Its locale is `en-US`, which decides
/// which fallback face a script gets; the plan's task 2 records that the kit
/// should take the person's language instead, and that is not changed by
/// moving it.
///
/// # Errors
///
/// [`NotAFace`] when an entry holds no face, or when there is no entry at all.
pub fn a_font_system(faces: impl IntoIterator<Item = Vec<u8>>) -> Result<FontSystem, NotAFace> {
    let mut db = cosmic_text::fontdb::Database::new();
    for data in faces {
        let before = db.faces().count();
        db.load_font_data(data);
        if db.faces().count() == before {
            return Err(NotAFace);
        }
    }
    let family = db
        .faces()
        .next()
        .and_then(|face| face.families.first())
        .map(|(name, _)| name.clone())
        .ok_or(NotAFace)?;
    db.set_sans_serif_family(family);
    Ok(FontSystem::new_with_locale_and_db("en-US".into(), db))
}

/// **alo's own font database**: [`ALOS_OWN_FACES`], in their order.
///
/// # Errors
///
/// [`NotAFace`] if a bundled file stopped being a font, which a build of this
/// crate's tests would already have said.
pub fn alos_own_fonts() -> Result<FontSystem, NotAFace> {
    a_font_system(ALOS_OWN_FACES.iter().map(|face| face.to_vec()))
}

#[cfg(test)]
#[path = "fonts_tests.rs"]
mod tests;
