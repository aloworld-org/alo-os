//! **The record window drawn as a window on the canvas**, for one frame on the
//! display the loop holds (ADR 0145, stage 4).
//!
//! Placed once where a new window goes; laid out in a room of its own the size
//! the approved rule gives this display (`crate::record_on_the_plane`), with its
//! title strip left to the canvas edge and its body in the look it wears; drawn
//! where the camera puts that room; and its places — Put back, the
//! confirmation's buttons, the window itself — carried onto the display for the
//! press after the frame.
//!
//! **A display with no room for the record by the rule keeps the old road**:
//! the window laid out on the display and drawn fixed to it, refusing the frame
//! as it always has where even that cannot be done.
//!
//! **On the display the loop holds only**, as Settings and the task windows
//! are: one window is on one display, and which display a window of alo's is on
//! is a question nothing here answers yet.

use smithay::utils::{Physical, Rectangle};

use crate::a_press_on_the_record::TheRecordsPlaces;
use crate::record_frame_drawn::TheStrip;
use crate::record_raster::{RecordPicture, TheRecordsRoom, picture_with};

/// What a frame drew of the record.
pub(crate) struct TheRecordDrawn {
    /// The picture, for the painter.
    pub(crate) picture: RecordPicture,
    /// Where its places are on the display, for the press after the frame.
    pub(crate) places: Option<TheRecordsPlaces>,
    /// Where its body is on the display, for its edge — none for the record
    /// fixed to the display, which draws its own strip.
    pub(crate) body: Option<Rectangle<i32, Physical>>,
}

/// Draw the record for this frame, if it is open and on the Place being
/// looked at.
///
/// # Errors
/// [`crate::RenderError::RecordScene`] where the record cannot be laid out.
pub(crate) fn the_record_drawn(
    server: &mut crate::Server,
    labels: &mut crate::WindowControlLabels,
    frame: crate::DesktopFrame<'_>,
    (size, named): ((i32, i32), &str),
) -> Result<Option<TheRecordDrawn>, crate::RenderError> {
    let look = crate::RecordLook {
        scheme: frame.look.scheme(),
        scale: frame.look.scale(),
        reading: frame.look.reading(),
        contrast: frame.look.contrast(),
    };
    let display = crate::display_scale::DisplayScale::of_per_cent(frame.display_scale);
    let Some(window) = crate::record_on_the_plane::how_big_the_record_is(size, look, display)
    else {
        // **The old road**, fixed to the display.
        let Some(record) = server.the_records_frame(frame.strings, frame.look) else {
            return Ok(None);
        };
        let picture = crate::record_raster::picture(
            record.window.shows(),
            record.strings,
            labels,
            size,
            record.look,
            record.clock,
            display,
        )?;
        let places = TheRecordsPlaces::of(&picture);
        return Ok(Some(TheRecordDrawn {
            picture,
            places,
            body: None,
        }));
    };
    let _ = server.the_record_is_placed(window, size, look.reading);
    if !server.the_record_is_open_here() {
        return Ok(None);
    }
    let focused = server.the_record_is_taking_the_keys();
    let Some(record) = server.the_records_frame(frame.strings, frame.look) else {
        return Ok(None);
    };
    let mut picture = picture_with(
        record.window.shows(),
        record.strings,
        labels,
        (TheRecordsRoom::ItsOwn(window), record.look, display),
        record.clock,
        TheStrip::TheCanvasEdge { focused },
    )?;
    let Some(room) = server.where_the_records_room_is_drawn(named, picture.size) else {
        return Ok(None);
    };
    picture.drawn_at = Some(room);
    let places = TheRecordsPlaces::of(&picture).map(|places| places.as_seen(picture.size, room));
    let body = picture
        .body
        .map(|body| crate::a_room_through_the_camera::on_the_screen(body, picture.size, room));
    Ok(Some(TheRecordDrawn {
        picture,
        places,
        body,
    }))
}
