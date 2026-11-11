#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// **Every bundled face loads, and Manrope is the sans-serif.**
#[test]
fn every_bundled_face_loads_and_manrope_is_the_sans_serif() {
    let fonts = alos_own_fonts().expect("the bundled faces");
    let sans = fonts
        .db()
        .family_name(&cosmic_text::fontdb::Family::SansSerif);
    assert_eq!(sans, "Manrope");
    assert!(fonts.db().faces().count() >= ALOS_OWN_FACES.len());
}

/// **A file that is not a font is refused**, rather than a database with a
/// silent hole in it.
#[test]
fn a_file_that_is_not_a_font_is_refused() {
    assert_eq!(
        a_font_system([MANROPE.to_vec(), b"not a font".to_vec()]).err(),
        Some(NotAFace)
    );
    assert_eq!(a_font_system(Vec::<Vec<u8>>::new()).err(), Some(NotAFace));
}
