//! The name beside a window's square: inward of the rail, centred on the
//! square, navy, and nothing for a name with nothing in it.

use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use smithay::utils::{Physical, Rectangle};

use super::the_name_beside;

fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

/// The design's first window slot on a right-hand rail of a 1440 display.
fn a_slot() -> Rectangle<i32, Physical> {
    Rectangle::new((1360, 160).into(), (48, 48).into())
}

/// **Inward of a right-hand rail**: it ends 10 before the rail, which starts
/// 8 before the slot, and is centred down the square.
#[test]
fn on_a_right_hand_panel_the_name_is_to_its_left() {
    let pill = the_name_beside(a_slot(), WhichEdge::Right, "Home · Dolphin", 1440, &mut fonts())
        .expect("a name");
    assert_eq!(pill.area.loc.x + pill.area.size.w, 1360 - 8 - 10);
    let middle = pill.area.loc.y + pill.area.size.h / 2;
    assert!((middle - (160 + 24)).abs() <= 1, "centred on the square");
    let navy = alo_appearance::Role::TextPrimary.colour();
    assert!(
        pill.pixels
            .contains(&[navy.red(), navy.green(), navy.blue()]),
        "the pill is navy"
    );
    assert!(pill.area.size.w > 2 * 10, "words inside its padding");
}

/// **Mirrored on a left-hand panel**: it starts 10 after the rail.
#[test]
fn on_a_left_hand_panel_the_name_is_to_its_right() {
    let slot = Rectangle::new((32, 160).into(), (48, 48).into());
    let pill = the_name_beside(slot, WhichEdge::Left, "Home · Dolphin", 1440, &mut fonts())
        .expect("a name");
    assert_eq!(pill.area.loc.x, 32 + 48 + 8 + 10);
}

/// **The refusal path: a name with nothing in it draws no pill**, and nor does
/// a display with no room beside the rail.
#[test]
fn nothing_to_say_draws_nothing() {
    assert!(the_name_beside(a_slot(), WhichEdge::Right, "  ", 1440, &mut fonts()).is_none());
    let at_the_edge = Rectangle::new((10, 160).into(), (48, 48).into());
    assert!(
        the_name_beside(at_the_edge, WhichEdge::Right, "Home", 1440, &mut fonts()).is_none()
    );
}
