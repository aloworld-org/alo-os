//! The record's edge: its icon on its tile, drawn over the body where the last
//! frame drew it, and Close on it shutting the window through the pointer road.

use std::os::unix::fs::PermissionsExt as _;

use alo_strings::Direction;
use smithay::backend::input::ButtonState;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Rectangle, Size};

use super::the_records_icon;
use crate::Server;
use crate::record_palette::RecordPalette;
use crate::record_testing::{an_afternoon_kept, light, words};

/// **The icon is the record's clock on its `bg/cool` tile**, drawn at the
/// display's own scale.
#[test]
fn the_icon_is_the_clock_on_its_tile() {
    let palette = RecordPalette::of(light().scheme, light().contrast);
    let icon = the_records_icon(32, palette).expect("a tile");
    assert_eq!((icon.across(), icon.down()), (32, 32));
    let at = |x: u32, y: u32| -> [u8; 4] {
        let i = usize::try_from((y * 32 + x) * 4).expect("a place");
        let mut out = [0; 4];
        out.copy_from_slice(icon.pixels().get(i..i + 4).expect("a pixel"));
        out
    };
    let [r, g, b, a] = at(1, 16);
    assert_eq!(([r, g, b], a), (palette.cool, 255), "the tile is bg/cool");
    let inked = (0..32)
        .flat_map(|y| (0..32).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let [r, g, b, _] = at(*x, *y);
            [r, g, b] == palette.ink
        })
        .count();
    assert!(inked > 10, "the clock is not drawn: {inked} pixels");
}

/// **Close on the edge shuts the record**, pressed where the edge is drawn over
/// the body — and an edge is drawn only while the body has been.
#[test]
fn close_on_the_edge_shuts_the_record() {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "record-edge-close",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    server.surfaces.popups.output_size = Some(Size::from((1440, 960)));
    let kept = an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    let _ = server.the_record_is_opened();
    let _ = server.the_record_is_placed((880, 796), (1440, 960), Direction::LeftToRight);
    let mut fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces")
        .fonts;
    assert!(
        server
            .the_records_edge_picture(&mut fonts, &words(), ((1440, 960), 100), light())
            .is_none(),
        "an edge was drawn over a body not drawn yet"
    );
    let body = Rectangle::<i32, Physical>::new((280, 100).into(), (880, 752).into());
    server.where_the_records_body_was_drawn = Some(body);
    assert!(
        server
            .the_records_edge_picture(&mut fonts, &words(), ((1440, 960), 100), light())
            .is_some()
    );
    let edge = crate::an_alo_windows_edge::the_edge_over(body, Size::from((10_000, 10_000)), 100);
    let close = edge
        .controls
        .iter()
        .find(|control| control.does == crate::window_edge::OnTheEdge::Close)
        .expect("Close on the edge")
        .target;
    let (x, y) = (
        close.loc.x + close.size.w / 2,
        close.loc.y + close.size.h / 2,
    );
    // The control: Close is where the window's own strip drew its Close, 4
    // from the window's edge, in the band above the body.
    assert_eq!(close.loc.x + close.size.w, 280 + 880 - 4);
    assert_eq!(close.loc.y, 56);
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    let _ = server
        .pointer_button(0x110, ButtonState::Pressed, 2)
        .expect("the press was handled");
    assert!(!server.record.is_open(), "Close left the record open");
}
