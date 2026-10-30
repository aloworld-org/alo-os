//! **Every character a program writes is drawn as itself, in its own cell.**
//!
//! The twelfth Demo 1 walk (2026-10-30) ran `ls -l` in alo's terminal and the
//! permissions came out as `-IW-F-F-.`: each `r` drawn as another shape, the
//! dashes beside it run together and across it, and the run shorter than its
//! eleven cells. `/var` on the line above, with no dash beside its `r`, was
//! drawn correctly.
//!
//! That is the face's own substitutions at work. A run is shaped whole, and
//! Geist Mono's ligatures and contextual alternates replace characters by their
//! neighbours — which is right in a paragraph of code and wrong in a terminal,
//! where a cell holds exactly the character the program put in it.
//!
//! So these shape a run the way a row is shaped and compare each glyph with the
//! same character shaped alone, and its place with the cell it belongs in.

use cosmic_text::{Buffer, FontSystem, Shaping, Wrap};

use crate::WindowControlLabels;

/// The font system this machine really builds, as the other terminal tests
/// take it.
fn the_shells_own_fonts() -> FontSystem {
    WindowControlLabels::new()
        .expect("the shell's bundled faces load")
        .fonts
}

/// Each glyph a run is shaped into: which glyph, and where it starts.
fn shaped(fonts: &mut FontSystem, text: &str) -> Vec<(u16, f32)> {
    let mut buffer = Buffer::new(fonts, super::the_designs_metrics(1.0));
    buffer.set_wrap(fonts, Wrap::None);
    buffer.set_size(fonts, None, None);
    buffer.set_text(
        fonts,
        text,
        &crate::painted_text::alos_own_mono(),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fonts, false);
    buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter().map(|glyph| (glyph.glyph_id, glyph.x)))
        .collect()
}

/// **The walk's own line, and what programs print beside it**: permissions,
/// arrows, comparisons and rules, every one a sequence a programming face
/// likes to join.
#[test]
fn each_character_is_drawn_as_itself_in_its_own_cell() {
    let mut fonts = the_shells_own_fonts();
    let cell = super::how_big_a_cell_is(&mut fonts, super::the_designs_metrics(1.0))
        .expect("the terminal's face measures a cell");

    for line in [
        "-rw-r--r--. 1 alo alo 0 Oct 30 17:08 march-invoice.pdf",
        "drwxr-xr-x. 2 alo alo 4096 Oct 30 /var",
        "a -> b => c == d != e <= f >= g || h && i :: j // k",
        "---------- ==== ~~~~ www ||| ...",
    ] {
        let together = shaped(&mut fonts, line);
        assert_eq!(
            together.len(),
            line.chars().count(),
            "{line:?} was shaped into {} glyphs for {} characters, so some \
             characters were joined or split",
            together.len(),
            line.chars().count()
        );
        for (column, (character, (glyph, x))) in line.chars().zip(together).enumerate() {
            let alone = shaped(&mut fonts, &character.to_string());
            assert_eq!(
                alone.first().map(|(glyph, _)| *glyph),
                Some(glyph),
                "{character:?} in column {column} of {line:?} was drawn as another \
                 glyph than {character:?} alone"
            );
            #[expect(
                clippy::cast_precision_loss,
                reason = "a column on a test line is a two-digit count"
            )]
            let its_cell = cell.advance * column as f32;
            assert!(
                (x - its_cell).abs() < 0.5,
                "{character:?} in column {column} of {line:?} starts at {x}, not at its \
                 cell's {its_cell}"
            );
        }
    }
}
