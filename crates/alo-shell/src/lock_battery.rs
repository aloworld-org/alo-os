//! Battery outline, charge fill and a non-colour charging mark.
use crate::painted::Solid;
use smithay::utils::Rectangle;
/// Draw the supplied reading without measuring or inferring power state here.
pub(crate) fn draw(
    battery: alo_locking::Battery,
    size: (i32, i32),
    margin: i32,
    ground: [u8; 3],
    ink: [u8; 3],
) -> Vec<Solid> {
    let mut solids = Vec::new();
    // Outline + fill indicate charge; a plus distinguishes charging without hue.
    let x = margin;
    let y = size.1 - margin - 22;
    solids.push(Solid {
        area: Rectangle::new((x, y).into(), (54, 22).into()),
        colour: ink,
    });
    solids.push(Solid {
        area: Rectangle::new((x + 2, y + 2).into(), (50, 18).into()),
        colour: ground,
    });
    solids.push(Solid {
        area: Rectangle::new(
            (x + 4, y + 4).into(),
            (46 * i32::from(battery.percent()) / 100, 14).into(),
        ),
        colour: ink,
    });
    if battery.is_charging() {
        solids.push(Solid {
            area: Rectangle::new((x + 64, y + 3).into(), (3, 16).into()),
            colour: ink,
        });
        solids.push(Solid {
            area: Rectangle::new((x + 58, y + 9).into(), (15, 3).into()),
            colour: ink,
        });
    }
    solids
}
