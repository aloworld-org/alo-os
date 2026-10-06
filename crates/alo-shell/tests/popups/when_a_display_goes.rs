//! A popup is not left constrained to a screen that has gone.
//!
//! `docs/autonomy/more-than-one-display-plan.md` task 6's second clause.

use super::{mapped, presentation::fixture};
use alo_shell::{FrameTarget, RenderError};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};

/// A display that can actually be retired.
///
/// The shared fixture's target does not override `retire`, so it answers
/// `RetirementUnsupported` and `retire_output` refuses before the display is
/// ever recorded as gone. This one retires, which is what a real backend
/// does and what this test is about.
struct ADisplayThatGoes;

impl FrameTarget for ADisplayThatGoes {
    fn size(&self) -> Size<i32, Physical> {
        (320, 200).into()
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        Ok(roots.to_vec())
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
}

/// **A popup whose display goes is dismissed.**
///
/// A popup is constrained to the screen its parent is on, so a screen that
/// has gone leaves its menus bound to a rectangle that no longer exists — and
/// a menu held open across that is one a person can neither reach nor close.
///
/// The two assertions before the retirement are the point of the test as much
/// as the one after it: without them this would pass against a compositor
/// that dismissed every popup the moment it opened.
#[test]
fn a_popup_is_dismissed_when_the_display_it_is_on_goes() {
    let f = fixture();
    f.backend(|s| s.enable_popup_protocol());
    let mut app = mapped(&f);
    // **A frame first, so there is a display to lose.** A display is recorded
    // when it submits, so a session that has never drawn has none to go —
    // and this test would then pass by having nothing to dismiss.
    assert!(
        f.render((320, 200), false, 0).is_ok(),
        "the fixture drew no frame, so no display was ever recorded"
    );
    // The full handshake, as `chains.rs` does it: a popup only reaches the
    // server once it has been acked and given a buffer.
    let (surface, xdg, _role) = app.popup_on(Some(&app.xdg.clone()), 1);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();

    assert_eq!(
        app.events.popups.done, 0,
        "the popup was dismissed before anything happened to its display"
    );
    assert!(
        f.backend(|s| !s.popup_surfaces().is_empty()),
        "the popup never reached the server, so this test could not fail"
    );

    // Asserted rather than unwrapped: a refusal here means the display never
    // went, and this test would then be reporting that a popup survived an
    // event that did not happen.
    assert!(
        f.backend(|s| s.retire_output(&mut ADisplayThatGoes).is_ok()),
        "the display refused to retire, so nothing was tested"
    );
    app.sync();

    assert_eq!(
        app.events.popups.done, 1,
        "a popup outlived the display it was constrained to"
    );
    assert!(
        f.backend(|s| s.popup_surfaces().is_empty()),
        "the server still holds a popup for a display that has gone"
    );
}
