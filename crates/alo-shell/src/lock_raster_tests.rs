//! Compare actual opaque pixels while private session content changes.
#![allow(clippy::panic, clippy::indexing_slicing)]
#![expect(
    clippy::unwrap_used,
    reason = "a failed fixture or assertion is a failed test"
)]
use crate::Contrast;
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
/// Draw at the requested output and scale, retaining layout refusals.
fn try_draw(
    surface: &crate::LockSurface<String>,
    indicator: &Indicator,
    size: (i32, i32),
    scale: TextScale,
) -> Result<super::lock_raster::LockPicture, crate::RenderError> {
    let screen = snapshot(surface, indicator);
    let temp = tempfile::tempdir().unwrap();
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/artwork/wallpapers/alo-quiet-horizon.png"),
        temp.path().join("alo.png"),
    )
    .unwrap();
    let background = LockBackground::within(screen.image(), size, Duration::ZERO, temp.path())?;
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
