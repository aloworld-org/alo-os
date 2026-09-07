//! Output constraints respect client flags and committed parent coordinate spaces.
use super::{mapped, presentation::fixture};
use wayland_client::Proxy;
use wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment as Adjust;

#[test]
fn initial_constraints_apply_only_permitted_adjustments() {
    for (extent, offset, flags, expected) in [
        ((20, 20), 1, Adjust::empty(), (7, 10, 16, 16)),
        ((20, 20), 1, Adjust::SlideX | Adjust::SlideY, (4, 4, 16, 16)),
        (
            (20, 20),
            1,
            Adjust::ResizeX | Adjust::ResizeY,
            (7, 10, 13, 10),
        ),
        ((20, 20), 1, Adjust::FlipX | Adjust::FlipY, (7, 10, 16, 16)),
        (
            (32, 32),
            16,
            Adjust::FlipX | Adjust::SlideX,
            (2, 10, 16, 16),
        ),
        ((20, 20), -20, Adjust::SlideX, (0, 10, 16, 16)),
        (
            (8, 8),
            1,
            Adjust::SlideX | Adjust::SlideY | Adjust::ResizeX | Adjust::ResizeY,
            (0, 0, 8, 8),
        ),
    ] {
        let f = fixture();
        assert!(f.render(extent, false, 0).is_ok());
        let mut app = mapped(&f);
        let (surface, xdg, _) = app.popup_adjusted(Some(&app.xdg), offset, flags);
        surface.commit();
        app.sync();
        assert_eq!(app.events.popups.geometry.last(), Some(&expected));
        app.ack_popup(&xdg);
        app.attach_popup(&surface);
        app.sync();
        assert_eq!(
            f.backend(|s| s.popup_surfaces().first().map(|p| (
                p.geometry.loc.x,
                p.geometry.loc.y,
                p.geometry.size.w,
                p.geometry.size.h
            ))),
            Some(expected)
        );
        assert_eq!(app.events.popups.done, 0);
    }
}

#[test]
fn nested_constraints_use_parent_window_origin_and_commit_atomically() {
    let f = fixture();
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.surface.commit();
    let (parent, parent_xdg, _) = app.popup(true, 1);
    parent.commit();
    app.sync();
    app.ack_popup(&parent_xdg);
    parent_xdg.set_window_geometry(1, 2, 12, 12);
    app.attach_popup(&parent);
    app.sync();
    let (child, child_xdg, role) =
        app.popup_adjusted(Some(&parent_xdg), 40, Adjust::SlideX | Adjust::SlideY);
    child.commit();
    app.sync();
    // Parent window origin is (9,13), regardless of its shadow/buffer offset.
    assert_eq!(app.events.popups.geometry.last(), Some(&(15, 10, 16, 16)));
    app.ack_popup(&child_xdg);
    app.attach_popup(&child);
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(25.0, 24.0, 1)).is_ok());
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(child.id().protocol_id(), 1.0, 1.0))
    );
    let position = || f.backend(|s| s.popup_surfaces().last().map(|p| p.geometry.loc));
    // Failed submission still announces its valid extent. Empty extents do not.
    assert!(f.render((32, 32), true, 2).is_err());
    assert!(f.render((0, 0), false, 3).is_err());
    app.reposition_adjusted(&role, 40, 19, Adjust::SlideX | Adjust::SlideY);
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(7, -6, 16, 16)));
    assert_eq!(position(), Some((15, 10).into()));
    app.ack_popup(&child_xdg);
    app.sync();
    assert_eq!(position(), Some((15, 10).into()));
    child.commit();
    app.sync();
    assert_eq!(position(), Some((7, -6).into()));
    assert!(app.events.frames.is_empty());
    assert_eq!(app.events.popups.repositioned, [19]);
}

#[test]
fn unsafe_constraint_arithmetic_dismisses_only_its_popup_tree() {
    let f = fixture();
    let mut healthy = mapped(&f);
    let mut app = mapped(&f);
    assert!(f.render((i32::MAX, 200), false, 0).is_ok());
    let (refused, _, _) = app.popup_adjusted(Some(&app.xdg), 1, Adjust::SlideX);
    refused.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(app.events.popups.geometry.is_empty());
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    assert!(f.render((320, 200), false, 1).is_ok());
    let (surface, xdg, role) = app.popup_adjusted(Some(&app.xdg), 1, Adjust::SlideX);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    app.reposition_adjusted(&role, i32::MAX, 21, Adjust::SlideX);
    app.sync();
    assert_eq!(app.events.popups.done, 2);
    assert!(app.events.popups.repositioned.is_empty());
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    healthy.sync();
    f.wait_for((2, 2));
}

#[test]
fn extreme_parent_origin_refuses_constraints_and_outputless_fixture_stays_explicit() {
    let f = fixture();
    let mut app = mapped(&f);
    let (surface, xdg, role) = app.popup_adjusted(Some(&app.xdg), 40, Adjust::SlideX);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(46, 10, 16, 16)));
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert!(f.render((320, 200), true, 0).is_err());
    let (_far, _subsurface) = app.child_on(&app.surface, (i32::MAX, 0));
    app.xdg.set_window_geometry(i32::MAX, 0, 16, 16);
    app.surface.commit();
    app.sync();
    app.reposition_adjusted(&role, 40, 22, Adjust::SlideX);
    app.sync();
    assert_eq!(app.events.popups.done, 1);
    assert!(app.events.popups.repositioned.is_empty());
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    f.wait_for((1, 1));
}
