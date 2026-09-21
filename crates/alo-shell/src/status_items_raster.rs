//! Where the four status items sit inside the status area, and what is painted
//! for each.
//!
//! The status area is `crate::dock_raster`'s segment at the far end of the
//! band. This divides it among the items there are to show and paints each in
//! the person's ink — and it decides nothing about what they say, which is
//! `crate::status_items`' half and the owning crates' before that.
//!
//! # It reflows because the dock does
//!
//! `alo_dock::Along` says which way the band runs: a dock across the screen
//! gets a **row** of items, a dock down either side gets a **column**. Nothing
//! here chooses an edge or a direction — both are read off the layout the dock
//! was already laid out with, so the status area cannot come to disagree with
//! the band it is inside.
//!
//! # An item the machine does not have takes no room
//!
//! A desktop has no battery, so there are three cells rather than four and each
//! is wider. The alternative — a cell left blank — would be a battery drawn as
//! an empty outline, which reads as *flat* rather than as *absent*.
//!
//! # It cannot cover the egress indicator
//!
//! The indicator's lines grow **away from the dock**, out of the band and onto
//! the screen (`crate::egress_status_place`); everything here stays **inside**
//! the status area, which is inside the band. The two therefore cannot overlap
//! by construction rather than by arithmetic, and the test below holds that
//! against the place the indicator really starts from.

use alo_dock::Along;
use smithay::utils::{Physical, Point, Rectangle};

use crate::dock_raster::DockPicture;
use crate::painted::Solid;
use crate::status_items::StatusItems;

/// The least a cell may be, in pixels, before an item is not worth drawing.
///
/// A status area too small for its items draws none rather than a row of marks
/// too small to read: a smear a person cannot interpret is worse than a space
/// they can ask about.
const SMALLEST_CELL: i32 = 12;

/// Which item a cell holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Which {
    /// The time.
    Clock,
    /// The battery, where there is one.
    Battery,
    /// How far the machine reaches.
    Network,
    /// How loud it is.
    Volume,
}

/// One item's place inside the status area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cell {
    /// Which item.
    pub(crate) which: Which,
    /// Where it sits.
    pub(crate) area: Rectangle<i32, Physical>,
}

/// The status items on one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StatusPicture {
    /// Each item and its place, in the order they are laid out.
    pub(crate) cells: Vec<Cell>,
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
}

/// Lay `items` out inside `dock`'s status area and rasterise them.
pub(crate) fn picture(
    items: &StatusItems,
    dock: &DockPicture,
    ink: [u8; 3],
    ground: [u8; 3],
) -> StatusPicture {
    let area = dock.status_area;
    let along = dock.layout.along();
    let mut which: Vec<Which> = Vec::with_capacity(4);
    which.push(Which::Clock);
    if items.battery().is_some() {
        which.push(Which::Battery);
    }
    which.push(Which::Network);
    which.push(Which::Volume);

    let how_many = i32::try_from(items.how_many()).unwrap_or(1);
    let (whole, thickness) = match along {
        Along::Across => (area.size.w, area.size.h),
        Along::Down => (area.size.h, area.size.w),
    };
    let each = whole / how_many.max(1);
    if each < SMALLEST_CELL || thickness < SMALLEST_CELL {
        return StatusPicture {
            cells: Vec::new(),
            solids: Vec::new(),
        };
    }

    let mut cells = Vec::with_capacity(which.len());
    let mut solids = Vec::new();
    for (at, item) in which.into_iter().enumerate() {
        let step = each * i32::try_from(at).unwrap_or(0);
        let origin: Point<i32, Physical> = match along {
            Along::Across => (area.loc.x + step, area.loc.y).into(),
            Along::Down => (area.loc.x, area.loc.y + step).into(),
        };
        let size = match along {
            Along::Across => (each, thickness),
            Along::Down => (thickness, each),
        };
        let cell = Rectangle::new(origin, size.into());
        cells.push(Cell {
            which: item,
            area: cell,
        });
        solids.extend(marks_for(item, items, cell, ink, ground));
    }
    StatusPicture { cells, solids }
}

/// What is painted for one item, in its own cell.
///
/// Every number here is the owning crate's, taken through
/// `crate::status_items`: the battery's fill is its charge, the volume's is its
/// own loudness, and the network is a mark that is either there or not because
/// *reaches anything* is a yes or a no.
fn marks_for(
    which: Which,
    items: &StatusItems,
    cell: Rectangle<i32, Physical>,
    ink: [u8; 3],
    ground: [u8; 3],
) -> Vec<Solid> {
    let inset = 2;
    let inner_w = (cell.size.w - inset * 2).max(1);
    let inner_h = (cell.size.h - inset * 2).max(1);
    let at: Point<i32, Physical> = (cell.loc.x + inset, cell.loc.y + inset).into();
    match which {
        // The clock is text, and text is painted by `crate::painted_text` from
        // the string `alo-formats` wrote. Its cell is reserved here and nothing
        // is invented to stand in for the characters.
        Which::Clock => Vec::new(),
        // An outline, hollowed, then filled as far as the charge goes — the
        // three `crate::lock_battery` draws, and for its reason. An outline
        // that was a filled block would make a flat battery and a full one the
        // same shape, which is the one thing this item exists to tell apart.
        Which::Battery => {
            let mut solids = vec![
                Solid {
                    area: Rectangle::new(at, (inner_w, inner_h).into()),
                    colour: ink,
                },
                Solid {
                    area: Rectangle::new(
                        (at.x + 1, at.y + 1).into(),
                        ((inner_w - 2).max(1), (inner_h - 2).max(1)).into(),
                    ),
                    colour: ground,
                },
            ];
            if let Some(hundredths) = items.battery_hundredths() {
                let room = (inner_w - 4).max(0);
                let filled = room * i32::from(hundredths) / 100;
                if filled > 0 {
                    solids.push(Solid {
                        area: Rectangle::new(
                            (at.x + 2, at.y + 2).into(),
                            (filled, (inner_h - 4).max(1)).into(),
                        ),
                        colour: ink,
                    });
                }
            }
            // Power going in, marked without colour — a bar across the middle,
            // the same idea `crate::lock_battery` draws for the same reason: a
            // state told only by hue is a state somebody cannot see.
            if items.is_charging() {
                let middle = at.y + (inner_h / 2) - 1;
                solids.push(Solid {
                    area: Rectangle::new(
                        (at.x + (inner_w / 4), middle).into(),
                        ((inner_w / 2).max(1), 2).into(),
                    ),
                    colour: ground,
                });
            }
            solids
        }
        // Reaching something is a mark; holding off is a notch cut out of it.
        // **A machine that was told nothing is holding off**, because that is
        // what `alo_networks::Metered::should_hold_off` counts *nothing said*
        // as — so this shows the notch rather than a clear connection it cannot
        // vouch for.
        Which::Network => {
            if !items.reaches_anything() {
                return Vec::new();
            }
            let mut solids = vec![Solid {
                area: Rectangle::new(at, (inner_w, inner_h).into()),
                colour: ink,
            }];
            if items.should_hold_off() {
                solids.push(Solid {
                    area: Rectangle::new(
                        (at.x + (inner_w / 3), at.y + (inner_h / 3)).into(),
                        ((inner_w / 3).max(1), (inner_h / 3).max(1)).into(),
                    ),
                    colour: ground,
                });
            }
            solids
        }
        Which::Volume => {
            let loud = i32::from(items.volume_hundredths());
            let filled = inner_w * loud / i32::from(alo_sound::Volume::LOUDEST);
            if filled > 0 {
                vec![Solid {
                    area: Rectangle::new(at, (filled, inner_h).into()),
                    colour: ink,
                }]
            } else {
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_dock::{Dock, Edge};
    use alo_networks::{HowFar, Metered, Reaching};
    use alo_power::Reading;
    use alo_power::reading::{Charge, Charging};
    use alo_sound::Volume;
    use alo_strings::Direction;

    use super::*;

    /// A laptop's four readings.
    fn a_laptop() -> StatusItems {
        StatusItems::shown(
            "09:41".to_owned(),
            Some(Reading::taken(
                Charge::reported(50).expect("a charge"),
                Charging::Discharging,
                None,
                std::time::SystemTime::UNIX_EPOCH,
            )),
            Reaching::reported(HowFar::AllOfIt, Metered::NotSaid),
            Volume::of(50).expect("a volume"),
        )
    }

    /// A desktop's three.
    fn a_desktop() -> StatusItems {
        StatusItems::shown(
            "09:41".to_owned(),
            None,
            Reaching::reported(HowFar::AllOfIt, Metered::NotSaid),
            Volume::of(50).expect("a volume"),
        )
    }

    /// The ink a mark is painted in. Which colour is the look's business and
    /// not this file's, so the tests name one rather than resolve a palette.
    const INK: [u8; 3] = [17, 17, 17];

    /// The ground a hollow is painted in.
    const GROUND: [u8; 3] = [238, 238, 238];

    /// The dock on one edge of a screen of this size, laid out exactly as
    /// `crate::dock_raster`'s own tests lay one out.
    fn a_dock(edge: Edge, size: (i32, i32)) -> DockPicture {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        let look = crate::desktop_testing::noon_look(
            &crate::desktop_testing::an_appearance(),
            Direction::LeftToRight,
        );
        crate::dock_raster::picture(&dock, look, size).expect("a dock on this screen")
    }

    /// Whether one rectangle is wholly inside another.
    fn inside(inner: Rectangle<i32, Physical>, outer: Rectangle<i32, Physical>) -> bool {
        inner.loc.x >= outer.loc.x
            && inner.loc.y >= outer.loc.y
            && inner.loc.x + inner.size.w <= outer.loc.x + outer.size.w
            && inner.loc.y + inner.size.h <= outer.loc.y + outer.size.h
    }

    /// **Every item sits inside the status area, on every edge**, and the items
    /// do not overlap each other.
    #[test]
    fn every_item_is_inside_the_status_area_and_none_overlaps_another() {
        for edge in Edge::ALL {
            let dock = a_dock(edge, (1920, 1080));
            let drawn = picture(&a_laptop(), &dock, INK, GROUND);
            assert_eq!(drawn.cells.len(), 4, "{edge:?}");
            for cell in &drawn.cells {
                assert!(
                    inside(cell.area, dock.status_area),
                    "{edge:?}: {:?} left the status area",
                    cell.which
                );
            }
            for (at, cell) in drawn.cells.iter().enumerate() {
                for other in drawn.cells.iter().skip(at + 1) {
                    assert!(
                        cell.area.intersection(other.area).is_none(),
                        "{edge:?}: {:?} and {:?} overlap",
                        cell.which,
                        other.which
                    );
                }
            }
            for solid in &drawn.solids {
                assert!(
                    inside(solid.area, dock.status_area),
                    "{edge:?}: a mark was painted outside the status area"
                );
            }
        }
    }

    /// **A dock across the screen gets a row; a dock down it gets a column.**
    ///
    /// Read off the same layout the band was laid out with, so the two cannot
    /// disagree about which way the status area runs.
    #[test]
    fn the_items_run_the_way_the_dock_does() {
        let across = a_dock(Edge::Bottom, (1920, 1080));
        let row = picture(&a_laptop(), &across, INK, GROUND);
        let xs: Vec<i32> = row.cells.iter().map(|cell| cell.area.loc.x).collect();
        let ys: Vec<i32> = row.cells.iter().map(|cell| cell.area.loc.y).collect();
        assert!(rising(&xs), "{xs:?}");
        assert!(level(&ys), "{ys:?}");

        let down = a_dock(Edge::Left, (1920, 1080));
        let column = picture(&a_laptop(), &down, INK, GROUND);
        let xs: Vec<i32> = column.cells.iter().map(|cell| cell.area.loc.x).collect();
        let ys: Vec<i32> = column.cells.iter().map(|cell| cell.area.loc.y).collect();
        assert!(rising(&ys), "{ys:?}");
        assert!(level(&xs), "{xs:?}");
    }

    /// **A machine with no battery has three cells, not four with one blank**,
    /// and no battery is painted anywhere.
    #[test]
    fn a_desktop_leaves_the_battery_out_rather_than_drawing_an_empty_one() {
        let dock = a_dock(Edge::Bottom, (1920, 1080));
        let drawn = picture(&a_desktop(), &dock, INK, GROUND);
        assert_eq!(drawn.cells.len(), 3);
        assert!(
            !drawn.cells.iter().any(|cell| cell.which == Which::Battery),
            "a machine with no battery was given a battery's place"
        );

        // And the three it does have are wider for it, rather than three in a
        // row with a gap where the fourth would have been.
        let laptop = picture(&a_laptop(), &dock, INK, GROUND);
        let desktop_cell = drawn.cells.first().expect("a clock").area.size.w;
        let laptop_cell = laptop.cells.first().expect("a clock").area.size.w;
        assert!(
            desktop_cell > laptop_cell,
            "three items did not take the room four would have: {desktop_cell} vs {laptop_cell}"
        );
    }

    /// **Nothing drawn here can reach the egress indicator's lines.**
    ///
    /// The indicator grows away from the dock, out of the band; every mark here
    /// is inside the status area, which is inside the band. This holds that
    /// against the band the dock was really laid out with rather than against
    /// an arithmetic repeat of it.
    #[test]
    fn the_status_items_never_reach_the_indicators_lines() {
        for edge in Edge::ALL {
            let dock = a_dock(edge, (1920, 1080));
            let drawn = picture(&a_laptop(), &dock, INK, GROUND);
            for cell in &drawn.cells {
                assert!(
                    inside(cell.area, dock.band),
                    "{edge:?}: {:?} left the band the indicator grows away from",
                    cell.which
                );
            }
        }
    }

    /// **A status area too small for its items draws none**, rather than a row
    /// of marks too small to read.
    #[test]
    fn a_status_area_too_small_draws_nothing_rather_than_a_smear() {
        let dock = a_dock(Edge::Bottom, (1920, 1080));
        let mut cramped = dock.clone();
        cramped.status_area = Rectangle::new(dock.status_area.loc, (8, 8).into());
        let drawn = picture(&a_laptop(), &cramped, INK, GROUND);
        assert!(drawn.cells.is_empty());
        assert!(drawn.solids.is_empty());
    }

    /// **The battery's fill is its charge and the volume's is its loudness**,
    /// each taken from the owning crate rather than computed here.
    ///
    /// The assertion that matters is that a **flat battery and a full one do
    /// not render the same**. An earlier version of this test counted the
    /// shapes drawn, which is a stand-in for the picture rather than the
    /// picture — and it passed against a battery whose outline was a filled
    /// block, so nought and a hundred looked identical.
    #[test]
    fn what_is_filled_is_what_the_crate_said() {
        let dock = a_dock(Edge::Bottom, (1920, 1080));

        let flat = picture(&a_machine_at(0), &dock, INK, GROUND);
        let full = picture(&a_machine_at(100), &dock, INK, GROUND);
        assert_ne!(
            ink_inside(&flat, Which::Battery),
            ink_inside(&full, Which::Battery),
            "a flat battery and a full one were drawn the same"
        );
        assert!(
            ink_inside(&flat, Which::Battery) < ink_inside(&full, Which::Battery),
            "a flat battery was not drawn emptier than a full one"
        );

        // A machine that reaches nothing has no network mark at all, rather
        // than a shape invented to mean "none".
        let nowhere = StatusItems::shown(
            "09:41".to_owned(),
            None,
            Reaching::reported(HowFar::NotSaid, Metered::NotSaid),
            Volume::of(0).expect("a volume"),
        );
        let drawn = picture(&nowhere, &dock, INK, GROUND);
        assert_eq!(
            ink_inside(&drawn, Which::Network),
            0,
            "a machine that reaches nothing was drawn as connected"
        );
        assert_eq!(
            ink_inside(&drawn, Which::Volume),
            0,
            "a silent machine was drawn as making a sound"
        );

        // And a machine that does reach something is.
        let somewhere = picture(&a_machine_at(50), &dock, INK, GROUND);
        assert!(ink_inside(&somewhere, Which::Network) > 0);
        assert!(ink_inside(&somewhere, Which::Volume) > 0);
    }

    /// **Charging is drawn, and a battery that is merely full is not.**
    ///
    /// The mark is a shape rather than a colour, for `crate::lock_battery`'s
    /// reason: a state told only by hue is a state somebody cannot see.
    #[test]
    fn power_going_in_is_drawn_and_a_full_battery_is_not_mistaken_for_it() {
        let dock = a_dock(Edge::Bottom, (1920, 1080));
        let at_half = |charging| {
            StatusItems::shown(
                "09:41".to_owned(),
                Some(Reading::taken(
                    Charge::reported(50).expect("a charge"),
                    charging,
                    None,
                    std::time::SystemTime::UNIX_EPOCH,
                )),
                Reaching::reported(HowFar::AllOfIt, Metered::NotSaid),
                Volume::of(50).expect("a volume"),
            )
        };
        let filling = picture(&at_half(Charging::Charging), &dock, INK, GROUND);
        let emptying = picture(&at_half(Charging::Discharging), &dock, INK, GROUND);
        let full = picture(&at_half(Charging::Full), &dock, INK, GROUND);

        assert_ne!(
            ink_inside(&filling, Which::Battery),
            ink_inside(&emptying, Which::Battery),
            "a battery filling and one emptying were drawn the same"
        );
        assert_eq!(
            ink_inside(&full, Which::Battery),
            ink_inside(&emptying, Which::Battery),
            "a full battery on the mains was drawn as filling"
        );
    }

    /// **A connection this machine should hold off on is drawn differently
    /// from one it should not** — including a machine that was told nothing,
    /// which `alo-networks` counts as holding off.
    ///
    /// A status area that drew *nothing said* the same as *not metered* would
    /// be adding a claim about somebody's data allowance.
    #[test]
    fn holding_off_is_drawn_and_nothing_said_holds_off() {
        let dock = a_dock(Edge::Bottom, (1920, 1080));
        let reaching = |metered| {
            StatusItems::shown(
                "09:41".to_owned(),
                None,
                Reaching::reported(HowFar::AllOfIt, metered),
                Volume::of(50).expect("a volume"),
            )
        };
        let free = picture(&reaching(Metered::Unmetered), &dock, INK, GROUND);
        let metered = picture(&reaching(Metered::Metered), &dock, INK, GROUND);
        let unknown = picture(&reaching(Metered::NotSaid), &dock, INK, GROUND);

        assert_ne!(
            ink_inside(&free, Which::Network),
            ink_inside(&metered, Which::Network),
            "a metered connection was drawn as a free one"
        );
        assert_eq!(
            ink_inside(&unknown, Which::Network),
            ink_inside(&metered, Which::Network),
            "a machine told nothing was drawn as a free connection"
        );
    }

    /// A laptop at this charge, reaching the internet, half as loud as it goes.
    fn a_machine_at(hundredths: u8) -> StatusItems {
        StatusItems::shown(
            "09:41".to_owned(),
            Some(Reading::taken(
                Charge::reported(hundredths).expect("a charge"),
                Charging::Discharging,
                None,
                std::time::SystemTime::UNIX_EPOCH,
            )),
            Reaching::reported(HowFar::AllOfIt, Metered::NotSaid),
            Volume::of(50).expect("a volume"),
        )
    }

    /// Whether every step is greater than the one before.
    fn rising(steps: &[i32]) -> bool {
        steps
            .windows(2)
            .all(|two| matches!((two.first(), two.get(1)), (Some(a), Some(b)) if a < b))
    }

    /// Whether every step is the same.
    fn level(steps: &[i32]) -> bool {
        steps
            .windows(2)
            .all(|two| matches!((two.first(), two.get(1)), (Some(a), Some(b)) if a == b))
    }

    /// How many pixels of ink are painted inside one item's cell.
    ///
    /// Counted rather than the shapes, because two pictures with the same
    /// number of shapes can look entirely different and it is the look that is
    /// under test. Ground painted over ink is subtracted, which is what makes a
    /// hollow outline read as hollow.
    fn ink_inside(drawn: &StatusPicture, which: Which) -> i32 {
        let Some(cell) = drawn.cells.iter().find(|cell| cell.which == which) else {
            return 0;
        };
        drawn
            .solids
            .iter()
            .filter(|solid| inside(solid.area, cell.area))
            .map(|solid| {
                let area = solid.area.size.w * solid.area.size.h;
                if solid.colour == INK { area } else { -area }
            })
            .sum()
    }
}
