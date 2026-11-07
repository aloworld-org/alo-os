//! Batch disjoint opaque pixel runs without changing native layer order.

use super::Inked;
use crate::RenderError;
use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::{Physical, Rectangle},
};
use std::collections::BTreeMap;

/// Bound temporary geometry even for an image with a different colour per pixel.
const MAX_RUNS: usize = 4096;

/// Rectangles are relative to one inked image, as both renderers require.
type Damage = Rectangle<i32, Physical>;
/// Only disjoint runs within the same image may be reordered by colour.
type Colours = BTreeMap<[u8; 3], Vec<Damage>>;

/// Complete one image before the next overlapping native layer can be drawn.
pub(super) fn paint(frame: &mut impl Frame, inked: &Inked) -> Result<(), RenderError> {
    batches(inked, |colour, damage| {
        let [red, green, blue] = colour;
        frame
            .draw_solid(
                inked.area,
                damage,
                Color32F::new(
                    f32::from(red) / 255.0,
                    f32::from(green) / 255.0,
                    f32::from(blue) / 255.0,
                    1.0,
                ),
            )
            .map_err(|error| RenderError::Submission(format!("paint native pixels: {error}")))
    })
}

/// Submit bounded groups; every source pixel belongs to exactly one rectangle.
fn batches(
    inked: &Inked,
    mut draw: impl FnMut([u8; 3], &[Damage]) -> Result<(), RenderError>,
) -> Result<(), RenderError> {
    let width = inked.area.size.w.max(1) as usize;
    let mut colours = Colours::new();
    let mut retained = 0;
    for (row, pixels) in inked.pixels.chunks(width).enumerate() {
        let mut start = 0;
        while let Some(first) = pixels.get(start) {
            let run = pixels.get(start..).map_or(1, |rest| {
                rest.iter().take_while(|pixel| *pixel == first).count()
            });
            colours.entry(*first).or_default().push(Rectangle::new(
                (start as i32, row as i32).into(),
                (run as i32, 1).into(),
            ));
            start += run;
            retained += 1;
            if retained == MAX_RUNS {
                flush(&mut colours, &mut draw)?;
                retained = 0;
            }
        }
    }
    flush(&mut colours, &mut draw)
}

/// Release each batch before accumulating more geometry; stop at the first error.
fn flush(
    colours: &mut Colours,
    draw: &mut impl FnMut([u8; 3], &[Damage]) -> Result<(), RenderError>,
) -> Result<(), RenderError> {
    for (colour, damage) in std::mem::take(colours) {
        draw(colour, &damage)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "painted_runs_tests.rs"]
mod tests;
