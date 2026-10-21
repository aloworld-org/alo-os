//! That the typeface is really there, really Manrope, and really the weight
//! the design asks for — and that it can draw the letters this product
//! promises.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::AlosOwnFace;

/// The size the environment's body text is drawn at where the design was
/// drawn, from `where_a_screen_goes`.
const BODY: u32 = 16;

fn carried() -> AlosOwnFace<'static> {
    AlosOwnFace::carried().expect("the typeface compiled into this program")
}

#[test]
fn the_typeface_compiled_in_is_a_font_that_can_be_read() {
    // Not `test -x` on a file: the bytes are handed to the engine and it says
    // whether they are a font. This repository has shipped three releases of a
    // binary that was present and could not start, and the question to ask of
    // any green thing is what would have to be true for it to pass while the
    // product is broken. A font file that is present and unparseable is
    // exactly that.
    let _face = carried();
}

#[test]
fn it_is_manrope_and_it_is_semibold() {
    // **Both halves matter and the second is the one that gets lost.** This
    // repository has already found its own shell drawing ExtraLight while
    // every check about *which family* passed, because the weight lived on a
    // variable axis nobody set. A static cut cannot be wrong about its weight,
    // and this asserts that it is the static cut.
    let face = carried();
    let attributes = face.as_swash_reads_it().attributes();
    let weight = attributes.weight().0;
    assert!(
        (600..=700).contains(&weight),
        "the typeface carried is weight {weight}, and the design asks for SemiBold"
    );

    let named: Vec<String> = face
        .as_swash_reads_it()
        .localized_strings()
        .filter(|string| string.id() == swash::StringId::Family)
        .map(|string| string.chars().collect())
        .collect();
    assert!(
        named.iter().any(|name| name.contains("Manrope")),
        "the typeface carried calls itself {named:?}, and the design is drawn in Manrope"
    );
}

#[test]
fn it_can_draw_every_letter_the_environments_own_sentences_use() {
    // The sentences in `words.rs` are English, and the same file is what 24
    // languages are translated from — so this checks the letters those
    // languages need rather than only the ones English does. A face missing
    // one draws nothing where that letter goes, and a person reads a word with
    // a hole in it.
    let face = carried();
    let wanted = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 \
                  .,:;!?'\"()[]{}-–—/\\@&%+=<>*#_|~`^$£€";
    let and_the_accents = "àáâãäåæçèéêëìíîïðñòóôõöøùúûüýþÿāăąćčďēėęěğġħīįıķĺľłńņňōőœŕřśşšţťūůűųŵŷźżžßẞ\
                           ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÑÒÓÔÕÖØÙÚÛÜÝÞŸĀĂĄĆČĎĒĖĘĚĞĠĦĪĮİĶĹĽŁŃŅŇŌŐŒŔŘŚŞŠŢŤŪŮŰŲŴŶŹŻŽ\
                           αβγδεζηθικλμνξοπρστυφχψωΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ";

    let mut missing: Vec<char> = Vec::new();
    for character in wanted.chars().chain(and_the_accents.chars()) {
        if character != ' ' && face.the_glyph_for(character).is_none() {
            missing.push(character);
        }
    }
    assert!(
        missing.is_empty(),
        "the typeface has no glyph for {missing:?}, so those letters would be \
         drawn as nothing at all"
    );
}

#[test]
fn a_character_the_face_has_no_glyph_for_is_said_rather_than_drawn_as_a_box() {
    // Glyph nought is the empty box every font carries. Reporting it as a real
    // glyph would draw a row of boxes, which reads as the screen being broken
    // rather than as a letter being absent.
    let face = carried();
    // A Han character, which Manrope does not cover and none of the 24
    // languages needs.
    assert!(face.the_glyph_for('漢').is_none());
    assert!(face.the_glyph_for('a').is_some());
}

#[test]
fn a_wider_word_measures_wider_and_a_narrower_one_narrower() {
    // The control that the measure is the face's own and not a character
    // count: in a proportional face these three differ, and in a count they
    // would not.
    let face = carried();
    let narrow = face.how_wide("iiii", BODY);
    let wide = face.how_wide("mmmm", BODY);
    assert!(
        wide > narrow * 2,
        "four m measure {wide} and four i measure {narrow}, which is not a \
         proportional face"
    );
}

#[test]
fn nothing_measures_nothing_and_a_space_measures_something() {
    let face = carried();
    assert_eq!(face.how_wide("", BODY), 0);
    assert!(
        face.how_wide(" ", BODY) > 0,
        "a space has no width, so words would run together"
    );
}

#[test]
fn twice_the_em_is_about_twice_as_wide() {
    // Proportional in the size as well as in the letters, which is what makes
    // the layout's scaling honest: the block is a share of the display and the
    // text in it has to scale the same way.
    let face = carried();
    let at_sixteen = face.how_wide("Which disk should alo OS use?", 16);
    let at_thirty_two = face.how_wide("Which disk should alo OS use?", 32);
    let twice = at_sixteen * 2;
    let slack = twice / 20;
    assert!(
        at_thirty_two.abs_diff(twice) <= slack.max(2),
        "at 16 it is {at_sixteen} wide and at 32 it is {at_thirty_two}, which is \
         not twice"
    );
}

#[test]
fn the_longest_sentence_is_wider_than_the_block_it_goes_in() {
    // **The measurement the wrapping exists for, taken with the real face
    // rather than with a stand-in.** `wrapping.rs` is tested against a
    // deliberately proportional fake measure; this is the one place the claim
    // is made against Manrope itself. If this ever came out false, the
    // wrapping would be dead code and the finding behind the flow would have
    // been wrong.
    let face = carried();
    let longest = "You will type a network password and the name of a disk. Both have to be \
                   exact, so the keyboard has to be right first";
    let the_block_at_the_design_size = 600;
    let drawn = face.how_wide(longest, BODY);
    assert!(
        drawn > the_block_at_the_design_size,
        "the longest sentence is {drawn} wide in a block of \
         {the_block_at_the_design_size}, so it would fit on one line after all"
    );
}

#[test]
fn a_baseline_sits_inside_the_line_it_belongs_to() {
    // A baseline above the top or below the bottom of its own line draws the
    // text outside the area the layout gave it.
    let face = carried();
    for em in [8_u32, 12, 16, 24, 36, 48] {
        let baseline = face.the_baseline_in(em, (em * 25) / 16);
        assert!(
            baseline > 0 && baseline <= em * 2,
            "at {em} to the em the baseline is {baseline} down, which is outside \
             the line"
        );
    }
}
