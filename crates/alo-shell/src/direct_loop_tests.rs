//! Production target/transaction integration, with injected graphics and DRM.

use super::*;
use crate::{DirectFrame, DirectLoopError, Server, SessionError};
use std::os::unix::fs::PermissionsExt;

#[test]
fn direct_loop_stops_retires_and_withdraws_after_frames_and_idle()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind(dir.path(), "loop")?;
    let (device, log) = fixture(&[], 0);
    let painter = Painter::default();
    let paints = painter.calls.clone();
    let target = Target::new(painter, device, scanout_tests::output());
    let mut steps = [
        DirectFrame::Render(1),
        DirectFrame::Idle,
        DirectFrame::Render(2),
        DirectFrame::Stop,
    ]
    .into_iter();
    let mut polls = 0;
    let result = crate::direct_loop::run(
        &mut server,
        target,
        &mut || {
            polls += 1;
            Ok(())
        },
        &mut || steps.next().unwrap_or(DirectFrame::Stop),
    );
    result.outcome?;
    result.retirement.ok_or("missing retirement")??;
    result.flush.ok_or("missing flush")??;
    assert_eq!(polls, 8);
    assert_eq!(paints.get(), 2);
    assert!(server.presentation.output.is_none());
    assert!(server.presentation.global.is_none());
    assert_eq!(
        log.borrow()
            .calls
            .iter()
            .filter(|call| **call == "disable")
            .count(),
        1
    );
    Ok(())
}

#[test]
fn direct_loop_pause_after_scheduler_prevents_the_planned_frame()
-> Result<(), Box<dyn std::error::Error>> {
    for pause_at in [1, 2, 3, 4] {
        let dir = tempfile::tempdir()?;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
        let mut server = Server::bind(dir.path(), "pause")?;
        let (device, log) = fixture(&[], 0);
        let painter = Painter::default();
        let paints = painter.calls.clone();
        let target = Target::new(painter, device, scanout_tests::output());
        let mut polls = 0;
        let mut schedules = 0;
        let result = crate::direct_loop::run(
            &mut server,
            target,
            &mut || {
                polls += 1;
                if polls == pause_at {
                    Err(SessionError::Inactive)
                } else {
                    Ok(())
                }
            },
            &mut || {
                schedules += 1;
                DirectFrame::Render(1)
            },
        );
        assert!(matches!(
            result.outcome,
            Err(DirectLoopError::Session(SessionError::Inactive))
        ));
        result.retirement.ok_or("missing retirement")??;
        result.flush.ok_or("missing flush")??;
        assert_eq!(polls, pause_at);
        assert_eq!(schedules, pause_at / 2);
        assert_eq!(paints.get(), usize::from(pause_at > 2));
        assert_eq!(log.borrow().calls.contains(&"disable"), pause_at > 2);
        assert!(server.presentation.output.is_none());
    }
    Ok(())
}

#[test]
fn direct_loop_keeps_render_and_disable_errors_without_retry()
-> Result<(), Box<dyn std::error::Error>> {
    for failure in ["enable", "destroy framebuffer"] {
        let dir = tempfile::tempdir()?;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
        let mut server = Server::bind(dir.path(), "failure")?;
        let (device, log) = fixture(&[], 0);
        let painter = Painter::default();
        let paints = painter.calls.clone();
        let target = Target::new(painter, device, scanout_tests::output());
        let mut schedules = 0;
        let result = crate::direct_loop::run(&mut server, target, &mut || Ok(()), &mut || {
            schedules += 1;
            if schedules == 2 {
                log.borrow_mut().failures = vec![failure, "disable"];
            }
            assert!(schedules <= 2, "must stop before another scheduler call");
            DirectFrame::Render(1)
        });
        assert!(matches!(result.outcome, Err(DirectLoopError::Render(_))));
        assert!(matches!(
            result.retirement,
            Some(Err(RenderError::Retirement(_)))
        ));
        assert_eq!(schedules, 2);
        result.flush.ok_or("missing flush")??;
        assert_eq!(paints.get(), 2);
        assert!(server.presentation.output.is_some());
        assert_eq!(log.borrow().calls.last(), Some(&"disable"));
        assert_eq!(
            log.borrow()
                .calls
                .iter()
                .filter(|call| **call == "disable")
                .count(),
            1
        );
    }
    Ok(())
}
