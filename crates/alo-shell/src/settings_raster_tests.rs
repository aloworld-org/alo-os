//! Laid-out Settings, read as text, boxes and colours: the window the frames
//! draw, where the keyboard is, and what scrolling keeps in view.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::settings_testing::{
    CountingDoor, a_persons_machine, doors, light, noon, settings_words,
};
use crate::{SettingsKey, SettingsPress};
use alo_appearance::Token;

/// Settings as it stands, drawn on an output of `size`.
fn drawn_at(window: &SettingsWindow, size: (i32, i32), look: SettingsLook) -> SettingsPicture {
    drawn_at_scale(window, size, look, DisplayScale::ONE)
}

/// Settings as it stands, drawn on an output of `size` at `display`.
fn drawn_at_scale(
    window: &SettingsWindow,
    size: (i32, i32),
    look: SettingsLook,
    display: DisplayScale,
) -> SettingsPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        window,
        &settings_words(),
        &mut labels,
        size,
        look,
        noon(),
        None,
        display,
    )
    .unwrap()
}

/// A key that means `key`.
fn key(key: SettingsKey) -> SettingsPress {
    SettingsPress {
        key,
        chord: None,
        filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
    }
}

/// Press `pressed` once.
fn press(window: &mut SettingsWindow, pressed: SettingsKey) {
    let strings = settings_words();
    let door = CountingDoor::revoking();
    window.pressed(key(pressed), doors(&strings, &door));
}

/// The pixel at (`x`, `y`) of a picture's raster, as red, green, blue, alpha.
fn pixel(drawn: &SettingsPicture, x: i32, y: i32) -> [u8; 4] {
    let washed = drawn.washed.as_ref().unwrap();
    let at = usize::try_from(
        ((y - washed.area.loc.y) * washed.area.size.w + (x - washed.area.loc.x)) * 4,
    )
    .unwrap();
    [
        washed.pixels[at],
        washed.pixels[at + 1],
        washed.pixels[at + 2],
        washed.pixels[at + 3],
    ]
}

/// **Closed Settings draws nothing.**
#[test]
fn closed_settings_draws_nothing() {
    let window = SettingsWindow::closed();
    let drawn = drawn_at(&window, (1920, 1080), light());
    assert!(drawn.is_empty());
}

/// **The window is where the frame draws it, at both sizes the design gives**:
/// 1200 × 816 at (120, 72) on 1440 × 960 and 1152 × 688 at (64, 56) on
/// 1280 × 800 — and the side column's eight categories are 48 tall, in two
/// groups, starting where `550:105` does.
#[test]
fn the_window_is_where_both_frames_draw_it() {
    let machine = a_persons_machine("raster-frame");
    let window = machine.opened();
    let large = drawn_at(&window, (1440, 960), light());
    let panel = large.panel.unwrap();
    assert_eq!(
        (panel.loc.x, panel.loc.y, panel.size.w, panel.size.h),
        (120, 72, 1200, 816)
    );
    let small = drawn_at(&window, (1280, 800), light());
    let panel = small.panel.unwrap();
    assert_eq!(
        (panel.loc.x, panel.loc.y, panel.size.w, panel.size.h),
        (64, 56, 1152, 688)
    );
    let categories: Vec<SettingsCategory> =
        large.categories.iter().map(|(which, _)| *which).collect();
    assert_eq!(categories, SettingsCategory::ALL);
    let (_, first) = large.categories[0];
    // `550:105` is at (16, 158) inside the side column, which is one border
    // inside the window: (120 + 1 + 16, 72 + 1 + 158).
    assert_eq!((first.loc.x, first.loc.y), (137, 231));
    assert!(large.categories.iter().all(|(_, at)| at.size.h == 48));
}

/// **The chosen category is on the accent's ground, the page's words are the
/// crates' and the window's own, and nothing is deep teal.**
#[test]
fn the_chosen_category_is_on_the_accent_and_nothing_is_deep_teal() {
    let machine = a_persons_machine("raster-colours");
    let window = machine.opened();
    let drawn = drawn_at(&window, (1440, 960), light());
    let (_, chosen) = drawn.categories[0];
    // Indigo at twelve per cent over the side column, inside the item.
    assert_eq!(
        pixel(&drawn, chosen.loc.x + 4, chosen.loc.y + 24),
        [227, 228, 233, 255]
    );
    let (_, other) = drawn.categories[1];
    assert_eq!(
        pixel(&drawn, other.loc.x + 4, other.loc.y + 24),
        [0xFA, 0xF7, 0xF2, 255]
    );
    let teal = Token::DeepTeal.colour();
    let teal = [teal.red(), teal.green(), teal.blue(), 255];
    let washed = drawn.washed.as_ref().unwrap();
    assert!(
        !washed.pixels.chunks(4).any(|it| it == teal),
        "deep teal is alo acting, and Settings is a person choosing"
    );
    let texts: Vec<&str> = drawn.texts.iter().map(|it| it.text.as_str()).collect();
    for expected in [
        "Settings",
        "alo and AI",
        "Where your questions are answered",
    ] {
        assert!(
            texts.contains(&expected),
            "{expected} is not drawn: {texts:?}"
        );
    }
    assert!(
        drawn
            .texts
            .iter()
            .all(|it| !it.text.is_empty() && it.area.size.h > 0)
    );
}

/// **Where the keyboard is, is drawn before any key is pressed**, and it moves
/// into the page with Tab.
///
/// `alo_access::Setting::FocusAlwaysVisible` asks for *where the keyboard is,
/// always drawn*: the window opens with the keyboard on the chosen category
/// and draws its outline there with no key pressed at all.
#[test]
fn where_the_keyboard_is_is_drawn_before_any_key_is_pressed() {
    let machine = a_persons_machine("focus-at-once");
    let mut window = machine.opened();
    let drawn = drawn_at(&window, (1440, 960), light());
    let (_, chosen) = drawn.categories[0];
    assert_eq!(drawn.focused, Some(chosen));
    press(&mut window, SettingsKey::NextSection);
    let inside = drawn_at(&window, (1440, 960), light());
    let row = window.focused(noon()).unwrap();
    let target = inside.targets.iter().find(|it| it.row == row).unwrap();
    assert_eq!(inside.focused, Some(target.area));
}

/// **The row the keyboard is on is always in view, whole**: on a short screen,
/// End scrolls the page so the last row is drawn inside the window; and a
/// screen too small for the window is refused.
#[test]
fn the_focused_row_is_always_in_view_and_a_frame_too_small_is_refused() {
    let machine = a_persons_machine("raster-view");
    let mut window = machine.opened();
    press(&mut window, SettingsKey::NextSection);
    press(&mut window, SettingsKey::Last);
    let last = window.focused(noon()).unwrap();
    let drawn = drawn_at(&window, (1280, 800), light());
    assert!(drawn.out_of_view);
    let target = drawn.targets.iter().find(|it| it.row == last).unwrap();
    let panel = drawn.panel.unwrap();
    assert!(target.area.loc.y >= panel.loc.y);
    assert!(target.area.loc.y + target.area.size.h <= panel.loc.y + panel.size.h);
    assert_eq!(drawn.focused, Some(target.area));

    let mut labels = WindowControlLabels::new().unwrap();
    assert!(matches!(
        picture(
            &window,
            &settings_words(),
            &mut labels,
            (1920, 90),
            light(),
            noon(),
            None,
            DisplayScale::ONE
        ),
        Err(RenderError::SettingsScene)
    ));
}

/// **A right-to-left reader gets the side column on the right**, and the same
/// words.
#[test]
fn a_right_to_left_reader_gets_the_side_column_on_the_right() {
    let machine = a_persons_machine("raster-rtl");
    let window = machine.opened();
    let left = drawn_at(&window, (1440, 960), light());
    let right = drawn_at(
        &window,
        (1440, 960),
        SettingsLook {
            reading: Direction::RightToLeft,
            ..light()
        },
    );
    let (_, l) = left.categories[0];
    let (_, r) = right.categories[0];
    assert!(r.loc.x > 720 && l.loc.x < 720, "{l:?} {r:?}");
    let words = |drawn: &SettingsPicture| -> Vec<String> {
        let mut them: Vec<String> = drawn.texts.iter().map(|it| it.text.clone()).collect();
        them.sort();
        them
    };
    assert_eq!(words(&left), words(&right));
}

/// **At 200 per cent Settings is twice its size and place**: the window
/// exactly, and the categories within a pixel.
#[test]
fn at_two_hundred_settings_is_twice_its_size() {
    let machine = a_persons_machine("raster-two");
    let window = machine.opened();
    let one = drawn_at(&window, (1440, 960), light());
    let two = drawn_at_scale(
        &window,
        (2880, 1920),
        light(),
        DisplayScale::of_per_cent(200),
    );
    let (a, b) = (one.panel.unwrap(), two.panel.unwrap());
    assert_eq!(
        (2 * a.loc.x, 2 * a.loc.y, 2 * a.size.w, 2 * a.size.h),
        (b.loc.x, b.loc.y, b.size.w, b.size.h)
    );
    for ((_, small), (_, large)) in one.categories.iter().zip(&two.categories) {
        assert!(
            (2 * small.loc.y - large.loc.y).abs() <= 2,
            "{small:?} {large:?}"
        );
        assert_eq!(2 * small.size.h, large.size.h);
    }
}

/// **Every page draws**, at both sizes the design gives, and when asked, each
/// is written out as a picture to set beside the design's frames.
///
/// The pictures are written only when `ALO_SETTINGS_PICTURES` names a folder,
/// which is a check somebody runs once by hand; what runs on every build is
/// that every page lays out and draws at both sizes.
#[test]
fn every_page_draws_and_can_be_looked_at() {
    let machine = a_persons_machine("raster-every-page");
    let mut window = machine.opened();
    let folder = std::env::var_os("ALO_SETTINGS_PICTURES");
    for (which, category) in SettingsCategory::ALL.into_iter().enumerate() {
        assert_eq!(window.category(), Some(category));
        for size in [(1440, 960), (1280, 800)] {
            let drawn = drawn_at(&window, size, light());
            assert!(!drawn.is_empty(), "{category:?} at {size:?}");
            if let Some(folder) = &folder {
                let named = format!("settings-{which}-{category:?}-{}x{}.png", size.0, size.1);
                written_out(&drawn, size, &std::path::Path::new(folder).join(named));
            }
        }
        press(&mut window, SettingsKey::Next);
    }

    // **And the country list open, with *Bel* typed** — frame `551:751`.
    let mut window = machine.opened();
    for _ in 0..5 {
        press(&mut window, SettingsKey::Next);
    }
    press(&mut window, SettingsKey::Choose);
    press(&mut window, SettingsKey::Choose);
    let strings = settings_words();
    let door = CountingDoor::revoking();
    for letter in ['b', 'e', 'l'] {
        window.pressed(
            SettingsPress {
                key: SettingsKey::Nothing,
                chord: None,
                filling: crate::a_key_fills_in_a_provider::FillingInKey::Typed(letter),
            },
            doors(&strings, &door),
        );
    }
    let drawn = drawn_at(&window, (1440, 960), light());
    let list = drawn.list.unwrap();
    assert!(drawn.entries.len() > 3, "{:?}", drawn.entries);
    assert!(
        drawn
            .entries
            .iter()
            .all(|(_, area)| list.contains_rect(*area)
                || area.loc.y + area.size.h > list.loc.y + list.size.h)
    );
    if let Some(folder) = &folder {
        written_out(
            &drawn,
            (1440, 960),
            &std::path::Path::new(folder).join("settings-5-RegionAndTime-choosing-1440x960.png"),
        );
    }
}

/// A picture of `drawn` on the canvas colour, written as a PNG.
fn written_out(drawn: &SettingsPicture, size: (i32, i32), to: &std::path::Path) {
    let (wide, tall) = (
        u32::try_from(size.0).unwrap(),
        u32::try_from(size.1).unwrap(),
    );
    let mut image = image::RgbaImage::from_pixel(wide, tall, image::Rgba([0xFA, 0xF7, 0xF2, 255]));
    let washed = drawn.washed.as_ref().unwrap();
    for (at, it) in washed.pixels.chunks(4).enumerate() {
        let at = i32::try_from(at).unwrap();
        let (x, y) = (
            washed.area.loc.x + at % washed.area.size.w,
            washed.area.loc.y + at / washed.area.size.w,
        );
        if x < 0 || y < 0 || x >= size.0 || y >= size.1 {
            continue;
        }
        let under = image
            .get_pixel(u32::try_from(x).unwrap(), u32::try_from(y).unwrap())
            .0;
        let alpha = u32::from(it[3]);
        let mut out = [0u8; 4];
        for channel in 0..3 {
            out[channel] =
                ((u32::from(it[channel]) * alpha + u32::from(under[channel]) * (255 - alpha) + 127)
                    / 255) as u8;
        }
        out[3] = 255;
        image.put_pixel(
            u32::try_from(x).unwrap(),
            u32::try_from(y).unwrap(),
            image::Rgba(out),
        );
    }
    image.save(to).unwrap();
}
