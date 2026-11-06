//! Settings' edge: drawn over the window where the last frame drew it, with
//! *Settings* as its title, in the focused look while it has the keys.

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_strings::Direction;
use smithay::utils::Size;

use crate::display_scale::DisplayScale;
use crate::settings_testing::{a_persons_machine, light, settings_words};

/// **The edge is drawn once Settings has been drawn**, and not before or after
/// it is shut — an edge with no window under it would be a Close that reaches
/// nothing.
#[test]
fn the_edge_is_drawn_with_settings_and_not_without_it() {
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = crate::Server::bind(directory.path(), "settings-edge").expect("a socket");
    server.surfaces.popups.output_size = Some(Size::from((1440, 960)));
    let machine = a_persons_machine("settings-edge");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    let mut fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces")
        .fonts;
    let words = settings_words();
    assert!(
        server
            .the_settings_edge_picture(&mut fonts, &words, (1440, 960), 100)
            .is_none(),
        "an edge was drawn over a window not drawn yet"
    );

    let sizes = crate::settings_on_the_plane::how_big_settings_is(
        (1440, 960),
        TextScale::ordinary(),
        DisplayScale::ONE,
    )
    .expect("room");
    let _ = server.settings_is_placed(sizes.window, (1440, 960), Direction::LeftToRight);
    let room = server
        .where_the_settings_room_is_drawn("", sizes)
        .expect("placed");
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    let picture = crate::settings_raster::picture(
        server.the_settings_window(),
        &words,
        &mut labels,
        sizes.room,
        light(),
        std::time::SystemTime::now(),
        None,
        DisplayScale::ONE,
    )
    .expect("drawn");
    let drawn = crate::settings_as_drawn::SettingsAsDrawn::of(&picture, room);
    server.settings_was_drawn(drawn);
    assert!(
        server
            .the_settings_edge_picture(&mut fonts, &words, (1440, 960), 100)
            .is_some()
    );

    server.settings.close();
    assert!(
        server
            .the_settings_edge_picture(&mut fonts, &words, (1440, 960), 100)
            .is_none(),
        "the edge outlived the window"
    );
}
