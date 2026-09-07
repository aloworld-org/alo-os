//! Reactive placement preserves opt-in, configure ordering and committed scenes.
use super::{mapped, presentation::fixture};
use wayland_client::Proxy;
use wayland_protocols::xdg::shell::client::xdg_positioner::ConstraintAdjustment as Adjust;

#[test]
fn reactive_output_changes_coalesce_and_wait_for_acknowledged_commit() {
    let f = fixture();
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    let mut fixed = mapped(&f);
    let (fixed_surface, fixed_xdg, _) = fixed.popup_adjusted(Some(&fixed.xdg), 40, Adjust::SlideX);
    fixed_surface.commit();
    fixed.sync();
    fixed.ack_popup(&fixed_xdg);
    fixed.attach_popup(&fixed_surface);
    fixed.sync();
    let (surface, xdg, _) =
        app.popup_reactive(Some(&app.xdg), 40, Adjust::SlideX | Adjust::SlideY, true);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    let position = || f.backend(|s| s.popup_surfaces().last().map(|popup| popup.geometry.loc));
    assert_eq!(position(), Some((24, 10).into()));
    assert!(f.render((32, 32), true, 1).is_err());
    app.sync();
    assert!(app.events.popups.serial.is_some());
    let first = app.events.popups.serial.unwrap_or_default();
    assert_eq!(app.events.popups.geometry.last(), Some(&(16, 10, 16, 16)));
    assert!(f.render((32, 32), true, 2).is_err());
    assert!(f.render((0, 0), false, 3).is_err());
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), 2);
    assert!(f.render((20, 20), true, 4).is_err());
    app.sync();
    assert!(app.events.popups.serial.is_some());
    let latest = app.events.popups.serial.unwrap_or_default();
    assert_eq!(app.events.popups.geometry.last(), Some(&(4, 4, 16, 16)));
    assert_eq!(position(), Some((24, 10).into()));
    xdg.ack_configure(first);
    surface.commit();
    app.sync();
    assert_eq!(position(), Some((16, 10).into()));
    xdg.ack_configure(latest);
    app.sync();
    assert_eq!(position(), Some((16, 10).into()));
    surface.commit();
    app.sync();
    assert_eq!(position(), Some((4, 4).into()));
    assert_eq!(app.events.popups.geometry.len(), 3);
    assert_eq!(app.events.popups.order, ["geometry", "surface"].repeat(3));
    assert!(app.events.frames.is_empty());
    fixed.sync();
    assert_eq!(fixed.events.popups.geometry.len(), 1);
    assert_eq!(fixed.events.popups.done, 0);
}

#[test]
fn reactive_descendant_uses_only_committed_parent_geometry_and_routes_input() {
    let f = fixture();
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    let (parent, parent_xdg, parent_role) = app.popup(true, 1);
    parent.commit();
    app.sync();
    app.ack_popup(&parent_xdg);
    app.attach_popup(&parent);
    app.sync();
    let (child, child_xdg, _) =
        app.popup_reactive(Some(&parent_xdg), 40, Adjust::SlideX | Adjust::SlideY, true);
    child.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(17, 10, 16, 16)));
    app.ack_popup(&child_xdg);
    app.attach_popup(&child);
    app.sync();
    app.reposition_popup_to(&parent_role, 20, 51);
    app.sync();
    let configs = app.events.popups.geometry.len();
    app.ack_popup(&parent_xdg);
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), configs);
    parent.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(10, 10, 16, 16)));
    app.ack_popup(&child_xdg);
    child.commit();
    app.sync();
    assert!(f.backend(|s| s.pointer_motion(25.0, 5.0, 1)).is_ok());
    app.sync();
    assert_eq!(
        app.events.pointer.enters.last(),
        Some(&(child.id().protocol_id(), 1.0, 1.0))
    );
    // Toplevel window geometry is double-buffered too, and propagates through chains.
    app.xdg.set_window_geometry(2, 3, 12, 12);
    app.sync();
    let configs = app.events.popups.geometry.len();
    app.surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), configs + 1);
    assert_eq!(app.events.popups.geometry.last(), Some(&(8, 10, 16, 16)));
    assert_eq!(app.events.popups.repositioned, [51]);
}

#[test]
fn reactive_permission_changes_follow_explicit_reposition_and_map_lifetime() {
    let f = fixture();
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    let (surface, xdg, role) = app.popup_reactive(Some(&app.xdg), 40, Adjust::SlideX, true);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    // Before the first buffer, the initial configure remains the only configure.
    assert!(f.render((32, 32), true, 1).is_err());
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), 1);
    app.attach_popup(&surface);
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(16, 10, 16, 16)));
    app.ack_popup(&xdg);
    surface.commit();
    app.sync();
    // A newer explicit request withdraws reactive permission even before commit.
    app.reposition_adjusted(&role, 40, 61, Adjust::SlideX);
    app.sync();
    let configs = app.events.popups.geometry.len();
    assert!(f.render((20, 20), true, 2).is_err());
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), configs);
    app.ack_popup(&xdg);
    surface.commit();
    app.sync();
    app.reposition_reactive(&role, 40, 62, Adjust::SlideX, true);
    app.sync();
    app.ack_popup(&xdg);
    assert!(f.render((24, 24), true, 3).is_err());
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), configs + 1);
    surface.commit();
    app.sync();
    assert_eq!(app.events.popups.geometry.last(), Some(&(8, -6, 16, 16)));
    assert_eq!(app.events.popups.repositioned, [61, 62]);
    let configs = app.events.popups.geometry.len();
    surface.attach(None, 0, 0);
    surface.commit();
    app.sync();
    assert!(f.render((40, 40), false, 4).is_ok());
    app.sync();
    assert_eq!(app.events.popups.geometry.len(), configs);
    assert_eq!(app.events.popups.done, 1);
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}

#[test]
fn reactive_unsafe_output_dismisses_child_first_and_preserves_other_clients() {
    let f = fixture();
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut app = mapped(&f);
    let mut healthy = mapped(&f);
    let (parent, xdg, role) = app.popup_reactive(Some(&app.xdg), 40, Adjust::SlideX, true);
    parent.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&parent);
    app.sync();
    let (child, child_xdg, child_role) = app.popup_on(Some(&xdg), 1);
    child.commit();
    app.sync();
    app.ack_popup(&child_xdg);
    app.attach_popup(&child);
    app.sync();
    assert!(f.render((i32::MAX, 40), true, 1).is_err());
    app.sync();
    assert_eq!(
        app.events.popups.done_order,
        [child_role.id().protocol_id(), role.id().protocol_id()]
    );
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
    let configs = app.events.popups.geometry.len();
    assert!(f.render((40, 40), false, 2).is_ok());
    parent.commit();
    app.sync();
    assert_eq!(app.events.popups.done, 2);
    assert_eq!(app.events.popups.geometry.len(), configs);
    healthy.sync();
    f.wait_for((2, 2));
}

#[test]
fn reactive_superseded_acknowledgement_refuses_only_its_client() {
    let f = fixture();
    assert!(f.render((40, 40), false, 0).is_ok());
    let mut healthy = mapped(&f);
    let mut app = mapped(&f);
    let (surface, xdg, _) = app.popup_reactive(Some(&app.xdg), 40, Adjust::SlideX, true);
    surface.commit();
    app.sync();
    app.ack_popup(&xdg);
    app.attach_popup(&surface);
    app.sync();
    assert!(f.render((32, 32), true, 1).is_err());
    app.sync();
    assert!(app.events.popups.serial.is_some());
    let stale = app.events.popups.serial.unwrap_or_default();
    assert!(f.render((20, 20), true, 2).is_err());
    app.sync();
    app.ack_popup(&xdg);
    surface.commit();
    app.sync();
    xdg.ack_configure(stale);
    app.refused();
    healthy.sync();
    f.wait_for((1, 1));
    assert!(f.backend(|s| s.popup_surfaces().is_empty()));
}
