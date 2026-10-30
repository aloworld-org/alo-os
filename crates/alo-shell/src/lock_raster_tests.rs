//! Compare actual opaque pixels while private session content changes.
#![allow(clippy::panic, clippy::indexing_slicing)]
#![expect(
    clippy::unwrap_used,
    reason = "a failed fixture or assertion is a failed test"
)]
use crate::Contrast;
use crate::display_scale::DisplayScale;
use crate::lock_testing::*;
use crate::{LockBackground, LockLook, SignInLook, WindowControlLabels};
use alo_appearance::{Appearance, DisplayId, Scheme, TextScale};
use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
use std::time::{Duration, SystemTime};

/// Snapshot.
fn snapshot(
    surface: &crate::LockSurface<String>,
    indicator: &Indicator,
) -> alo_locking::LockScreen {
    surface
        .snapshot(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000),
            &Appearance::shipped(),
            &DisplayId::named("eDP-1").unwrap(),
            Some(alo_locking::Battery::reading(72, true).unwrap()),
            indicator,
        )
        .unwrap()
}
/// Draw.
fn draw(
    surface: &crate::LockSurface<String>,
    indicator: &Indicator,
) -> super::lock_raster::LockPicture {
    try_draw(surface, indicator, (800, 600), TextScale::ordinary()).unwrap()
}
/// Draw at the requested output and text scale, at one-to-one.
fn try_draw(
    surface: &crate::LockSurface<String>,
    indicator: &Indicator,
    size: (i32, i32),
    scale: TextScale,
) -> Result<super::lock_raster::LockPicture, crate::RenderError> {
    try_draw_at(surface, indicator, size, scale, DisplayScale::ONE)
}
/// Draw at the requested output, text scale and display scale, retaining
/// layout refusals.
fn try_draw_at(
    surface: &crate::LockSurface<String>,
    indicator: &Indicator,
    size: (i32, i32),
    scale: TextScale,
    display: DisplayScale,
) -> Result<super::lock_raster::LockPicture, crate::RenderError> {
    let screen = snapshot(surface, indicator);
    let background = LockBackground::prepare(screen.image(), size)?;
    let region = alo_formats::Regionally::reading("en").unwrap();
    let timezone = alo_formats::Timezone::named("Europe/Berlin").unwrap();
    let strings = words();
    let look = LockLook {
        appearance: SignInLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Dark,
            scale,
        },
        region: &region,
        timezone: &timezone,
        strings: &strings,
    };
    super::lock_raster::picture(
        &screen,
        &background,
        surface.is_asking().then(|| surface.shows()),
        &mut WindowControlLabels::new().unwrap(),
        &look,
        display,
    )
}
/// Departure.
fn departure(destination: &str, agent: &str) -> Indicator {
    let mut indicator = Indicator::default();
    indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(
                &alo_capability::Grantee::named(agent),
                Why::Fetching,
                Destination::at(destination).unwrap(),
            ),
            SystemTime::UNIX_EPOCH,
        )
        .unwrap();
    indicator
}
#[test]
/// Private notifications and egress destinations never change locked pixels.
fn private_notifications_and_egress_destinations_never_change_locked_pixels() {
    let mut a = surface();
    let mut b = surface();
    a.arrives("medical report.txt: secret notification".into());
    b.arrives("other private message and agent answer".into());
    let first = draw(&a, &departure("first.example", "@files"));
    let second = draw(&b, &departure("different.example", "@mail"));
    assert_eq!(first, second);
    if let Some(path) = std::env::var_os("ALO_LOCK_PREVIEW") {
        image::save_buffer(
            path,
            &first.pixels,
            first.size.0 as u32,
            first.size.1 as u32,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
    assert!(
        first
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 255)
    );
    assert_ne!(
        first,
        draw(&a, &Indicator::default()),
        "real egress must visibly light the screen"
    );
}
#[test]
/// Unlock fields reuse sign in pixels and never reveal password length.
fn unlock_fields_reuse_sign_in_pixels_and_never_reveal_password_length() {
    assert_eq!(
        draw(&typed("ada", "x"), &Indicator::default()),
        draw(
            &typed("ada", "a much longer private password"),
            &Indicator::default()
        )
    );
}
#[test]
/// The clock uses civil time across daylight saving and refuses unknown zones.
fn the_clock_uses_civil_time_across_daylight_saving_and_refuses_unknown_zones() {
    let region = alo_formats::Regionally::reading("en").unwrap();
    let berlin = alo_formats::Timezone::named("Europe/Berlin").unwrap();
    for (instant, hour, minute) in [
        ("2026-03-29T00:30:00Z", 1, 30),
        ("2026-03-29T01:30:00Z", 3, 30),
    ] {
        let at = SystemTime::from(instant.parse::<jiff::Timestamp>().unwrap());
        assert_eq!(
            super::lock_clock::written(at, &berlin, &region).unwrap(),
            region.time(hour, minute).unwrap()
        );
    }
    assert!(
        super::lock_clock::written(
            SystemTime::UNIX_EPOCH,
            &alo_formats::Timezone::named("Made/Up").unwrap(),
            &region
        )
        .is_err()
    );
}

/// An authentication refusal is never silently dropped when large text needs space.
#[test]
fn a_refusal_that_does_not_fit_is_rejected_instead_of_disappearing() {
    let crate::LockPressed::Still(surface) =
        typed("ada", "x").pressed(crate::SignInKey::Enter, || {
            Err(alo_greeting::NotReadable {
                at: "/private/accounts".into(),
                why: "unreadable".into(),
            })
        })
    else {
        panic!("an unreadable store unlocked the screen")
    };
    let scale = TextScale::percent(200).unwrap();
    assert!(try_draw(&surface, &Indicator::default(), (800, 840), scale).is_err());
    assert!(try_draw(&surface, &Indicator::default(), (1200, 1800), scale).is_ok());
}

/// Everything a lock picture paints, hashed.
fn hash_of(drawn: &super::lock_raster::LockPicture) -> u64 {
    let mut hashed = crate::a_picture_hashed::Hashed::begun();
    hashed.raw(&drawn.pixels);
    hashed.done()
}

#[test]
/// **At 100 per cent the PIN form is the pixels main drew** before it followed
/// its display's scale: held to hashes taken from `main` by the same code,
/// twice, with the same answers. Bundled faces only.
fn at_one_hundred_the_pin_form_is_mains() {
    let asked = key(surface(), crate::SignInKey::Enter);
    let hashes = [
        hash_of(&draw(&asked, &Indicator::default())),
        hash_of(&draw(&typed("ada", "x"), &Indicator::default())),
        hash_of(
            &try_draw(
                &typed("ada", "x"),
                &Indicator::default(),
                (1920, 1080),
                TextScale::ordinary(),
            )
            .unwrap(),
        ),
    ];
    assert_eq!(hashes, [0_u64; 3], "{hashes:#x?}");
}

#[test]
/// **At 200 per cent the PIN form is twice its size and place**: every field
/// edge and the masked band, held against the same form at one on half the
/// room.
fn at_two_hundred_the_pin_form_is_twice_its_size() {
    let surface = typed("ada", "x");
    let strings = words();
    let look = SignInLook {
        contrast: Contrast::AsDesigned,
        scheme: Scheme::Dark,
        scale: TextScale::ordinary(),
    };
    let mut labels = WindowControlLabels::new().unwrap();
    let mut form = |size: (i32, i32), display: DisplayScale| {
        crate::sign_in_raster::picture(
            surface.shows(),
            &mut labels,
            &strings,
            size,
            look,
            crate::sign_in_raster::WhoseScreen::InsideAnother,
            display,
        )
        .unwrap()
    };
    let one = form((800, 200), DisplayScale::ONE);
    let two = form((1600, 400), DisplayScale::of_per_cent(200));
    assert_eq!(one.solids.len(), two.solids.len());
    for (small, large) in one.solids.iter().zip(&two.solids) {
        let (a, b) = (small.area, large.area);
        assert_eq!(
            (2 * a.loc.x, 2 * a.loc.y, 2 * a.size.w, 2 * a.size.h),
            (b.loc.x, b.loc.y, b.size.w, b.size.h)
        );
    }
}

#[test]
/// **A lock screen with too little room for the form at its scale draws it at
/// the largest that fits** rather than blanking: 800 × 600 at 200 leaves a
/// 200-pixel middle third, which holds the form at 125 and not at 150.
fn a_pin_form_with_too_little_room_is_held_to_what_fits() {
    let surface = typed("ada", "x");
    let at = |display: u16| {
        try_draw_at(
            &surface,
            &Indicator::default(),
            (800, 600),
            TextScale::ordinary(),
            DisplayScale::of_per_cent(display),
        )
    };
    let held = at(200).unwrap();
    assert_eq!(held, at(125).unwrap());
    assert_ne!(held, at(100).unwrap(), "and larger than one-to-one");
}

#[test]
/// **Keys reach the form at any scale**: what a press changes is the form's
/// state, which the picture at 200 shows as it does at one.
fn a_key_changes_the_form_at_two_hundred() {
    let at_two = |surface: &crate::LockSurface<String>| {
        try_draw_at(
            surface,
            &Indicator::default(),
            (1600, 1200),
            TextScale::ordinary(),
            DisplayScale::of_per_cent(200),
        )
        .unwrap()
    };
    let named = typed("ada", "");
    let pressed = key(typed("ada", ""), crate::SignInKey::Letter('1'));
    assert_ne!(at_two(&named), at_two(&pressed));
    assert_ne!(
        at_two(&pressed),
        try_draw(
            &pressed,
            &Indicator::default(),
            (1600, 1200),
            TextScale::ordinary()
        )
        .unwrap(),
        "the control: at two it is not the picture at one"
    );
}
