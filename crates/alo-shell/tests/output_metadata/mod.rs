//! Real wire metadata with failed submission and identity replacement refusal.
use super::support::{Application, Fixture};
use alo_shell::{FrameTarget, OutputMetadata, RenderError};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Size},
};
use wayland_client::protocol::wl_output;

struct Target {
    metadata: OutputMetadata,
    size: (i32, i32),
    fail: bool,
    calls: usize,
}
impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        self.size.into()
    }
    fn metadata(&self) -> Result<OutputMetadata, RenderError> {
        Ok(self.metadata.clone())
    }
    fn submit(&mut self, roots: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.calls += 1;
        if self.fail {
            Err(RenderError::Submission("injected".into()))
        } else {
            Ok(roots.to_vec())
        }
    }
}

fn metadata() -> OutputMetadata {
    OutputMetadata {
        name: "test-panel-1".into(),
        make: "Test make".into(),
        model: "Test model".into(),
        physical_size: (310, 170),
        refresh: 59_940,
    }
}

fn render(
    fixture: &Fixture,
    metadata: OutputMetadata,
    size: (i32, i32),
    fail: bool,
    time: u32,
) -> (Result<usize, RenderError>, usize) {
    fixture.backend(move |server| {
        let mut target = Target {
            metadata,
            size,
            fail,
            calls: 0,
        };
        let result = server.render(&mut target, time);
        (result, target.calls)
    })
}

#[test]
fn output_wire_metadata_and_modes_are_published_only_after_success() -> Result<(), RenderError> {
    let fixture = Fixture::new();
    assert!(render(&fixture, metadata(), (320, 200), true, 0).0.is_err());
    let mut app = Application::new(&fixture);
    app.sync();
    assert_eq!(app.events.outputs, 0);
    app.configure();
    app.surface.frame(&app.queue.handle(), ());
    app.attach();
    app.sync();
    assert_eq!(render(&fixture, metadata(), (320, 200), false, 1).0?, 1);
    app.sync();
    // Binding a newly published global takes another roundtrip.
    app.sync();
    assert_eq!(app.events.outputs, 1);
    assert_eq!(app.events.frames, [1]);
    assert_eq!(app.events.membership, (1, 0));
    assert!(
        app.events
            .output_events
            .iter()
            .any(|e| matches!(e, wl_output::Event::Name { name } if name == "test-panel-1"))
    );
    assert!(app.events.output_events.iter().any(|e| matches!(e, wl_output::Event::Geometry { physical_width: 310, physical_height: 170, make, model, .. } if make == "Test make" && model == "Test model")));
    assert!(app.events.output_events.iter().any(|e| matches!(
        e,
        wl_output::Event::Mode {
            width: 320,
            height: 200,
            refresh: 59_940,
            ..
        }
    )));
    let before = app.events.output_events.len();
    app.surface.frame(&app.queue.handle(), ());
    app.surface.commit();
    app.sync();
    assert!(
        render(
            &fixture,
            OutputMetadata {
                refresh: 60_000,
                ..metadata()
            },
            (640, 480),
            true,
            2
        )
        .0
        .is_err()
    );
    // **An identity that changed under the same name is still refused**, and
    // the name is no longer one of them.
    //
    // This loop included `name` until 2026-10-05, and that entry was the
    // single-output assumption being enforced by an identity guard: a target
    // calling itself something else is a **second display**, not the same one
    // claiming a new identity. `more-than-one-display-plan.md` task 3 gives it
    // a presentation of its own, so it is accepted below rather than refused
    // here.
    //
    // The other three are the fault this guard was written for and keep
    // refusing: one connector reporting a different monitor is something to
    // stop, whatever else is plugged in.
    for changed in [
        OutputMetadata {
            physical_size: (400, 300),
            ..metadata()
        },
        OutputMetadata {
            model: "replacement".into(),
            ..metadata()
        },
        OutputMetadata {
            make: "replacement".into(),
            ..metadata()
        },
    ] {
        let (result, calls) = render(&fixture, changed, (640, 480), false, 3);
        assert!(matches!(result, Err(RenderError::OutputIdentityChanged)));
        assert_eq!(calls, 0);
    }
    let (result, calls) = render(
        &fixture,
        OutputMetadata {
            name: "bad\0name".into(),
            ..metadata()
        },
        (640, 480),
        false,
        4,
    );
    assert!(matches!(result, Err(RenderError::InvalidOutputMetadata)));
    assert_eq!(calls, 0);
    app.sync();
    assert_eq!(app.events.output_events.len(), before);
    assert_eq!(app.events.frames, [1]);
    assert_eq!(app.events.membership, (1, 0));
    assert_eq!(
        render(
            &fixture,
            OutputMetadata {
                refresh: 60_000,
                ..metadata()
            },
            (640, 480),
            false,
            5
        )
        .0?,
        1
    );
    app.sync();
    assert_eq!(app.events.outputs, 1);
    assert_eq!(app.events.frames, [1, 5]);
    assert_eq!(app.events.membership, (1, 0));
    assert!(
        app.events
            .output_events
            .iter()
            .skip(before)
            .any(|e| matches!(
                e,
                wl_output::Event::Mode {
                    width: 640,
                    height: 480,
                    refresh: 60_000,
                    ..
                }
            ))
    );
    // **A second display draws rather than being turned away, and is
    // advertised.** The same call refused as `OutputIdentityChanged` above
    // until 2026-10-05 — that entry was the single-output assumption being
    // enforced by an identity guard. `more-than-one-display-plan.md` task 3
    // gives a target with a name of its own a presentation of its own.
    //
    // **Last in this test on purpose.** It adds a `wl_output`, so every count
    // above it is a count of one display's events; asserting them after a
    // second display exists would be asserting about two.
    let outputs_before = app.events.outputs;
    let (result, calls) = render(
        &fixture,
        OutputMetadata {
            name: "HDMI-A-1".into(),
            ..metadata()
        },
        (640, 480),
        false,
        6,
    );
    assert!(
        result.is_ok(),
        "a display with a name of its own was refused: {result:?}"
    );
    assert_eq!(calls, 1, "a second display was accepted and never drawn");
    app.sync();
    assert_eq!(
        app.events.outputs,
        outputs_before + 1,
        "a second display did not reach the client as a second wl_output"
    );
    Ok(())
}
