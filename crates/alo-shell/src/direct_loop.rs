//! Session-driven synchronous dispatch, submission and ordered output retirement.

use crate::{FrameTarget, RenderError, Server, SessionError};

/// Trusted scheduler decision; even idle iterations poll the seat and dispatch.
#[derive(Debug, Clone, Copy)]
pub enum DirectFrame {
    /// Dispatch clients without drawing a frame.
    Idle,
    /// Submit with a wrapping monotonic timestamp in milliseconds.
    Render(u32),
    /// End this output lifetime and retire before closing its descriptor.
    Stop,
}

/// Cause that stopped the synchronous compositor loop.
#[derive(Debug, thiserror::Error)]
pub enum DirectLoopError {
    /// Seat pause or notification failure; no further dispatch/submission occurs.
    #[error(transparent)]
    Session(#[from] SessionError),
    /// Client transport failed.
    #[error("direct client dispatch failed: {0}")]
    Dispatch(#[from] std::io::Error),
    /// Frame submission failed.
    #[error(transparent)]
    Render(#[from] RenderError),
    /// Fresh DRM discovery failed before target construction.
    #[error(transparent)]
    Discovery(#[from] crate::AtomicOutputError),
}

/// Loop failure and retirement failure are independent; neither hides the other.
#[derive(Debug)]
#[must_use = "inspect loop, retirement and flush results as well as session cleanup"]
pub struct DirectLoopResult {
    /// Clean scheduler stop or the first runtime failure.
    pub outcome: Result<(), DirectLoopError>,
    /// Output disable/withdrawal result; absent if discovery never made a target.
    pub retirement: Option<Result<(), RenderError>>,
    /// Flush queued output events without dispatch; absent before target creation.
    /// Would-block clients still require later dispatch or server teardown.
    pub flush: Option<Result<(), std::io::Error>>,
}

impl crate::DirectSession {
    /// Run one direct output lifetime with a caller-owned current GLES renderer.
    ///
    /// Fresh atomic discovery and the target use this scope's seat descriptor.
    /// The renderer must support offscreen readback and must not retain/duplicate
    /// that descriptor. Renderer initialization and direct input are separate.
    /// `next` is trusted pacing/stop control, never an agent callback; it should
    /// return promptly so idle sessions keep polling. Seat state is checked again
    /// after it returns, before any client dispatch or rendering. No retry or
    /// automatic resume occurs. Retire and drop precede session descriptor close,
    /// including when submission fails; inspect all independent outcomes. Queued
    /// input releases/leaves and output events are flushed without dispatching
    /// more requests after pause. Input is cleared before acquisition/discovery
    /// and on every loop exit, including stop and failed submission/retirement.
    /// If acquisition or discovery fails before a target exists, cleanup events
    /// remain queued for the caller to flush or dispatch.
    /// Failed retirement preserves advertised output, so discard the server
    /// before recovery. Upstream libseat acknowledgement ordering still applies.
    pub fn run_compositor(
        &mut self,
        server: &mut Server,
        renderer: &mut smithay::backend::renderer::gles::GlesRenderer,
        mut next: impl FnMut() -> DirectFrame,
    ) -> Result<crate::ActiveSessionResult<DirectLoopResult>, SessionError> {
        server.clear_input();
        self.with_active_device(|fd, poll| {
            let output = match crate::discover_atomic_output(fd) {
                Ok(output) => output,
                Err(error) => {
                    return DirectLoopResult {
                        outcome: Err(error.into()),
                        retirement: None,
                        flush: None,
                    };
                }
            };
            let target = crate::DirectTarget::new(renderer, fd, output);
            run(server, target, poll, &mut next)
        })
    }
}

/// Own the target so its destructor runs before the enclosing device scope closes.
pub(crate) fn run(
    server: &mut Server,
    mut target: impl LoopTarget,
    poll: &mut dyn FnMut() -> Result<(), SessionError>,
    next: &mut dyn FnMut() -> DirectFrame,
) -> DirectLoopResult {
    let outcome = (|| {
        loop {
            poll()?;
            let frame = next();
            poll()?;
            if matches!(frame, DirectFrame::Stop) {
                return Ok(());
            }
            server.dispatch()?;
            if let DirectFrame::Render(time) = frame {
                server.render(&mut target, time)?;
                target.check()?;
            }
        }
    })();
    server.clear_input();
    let retirement = Some(server.retire_output(&mut target));
    let flush = Some(server.flush());
    DirectLoopResult {
        outcome,
        retirement,
        flush,
    }
}

/// A committed frame can still require immediate retirement after cleanup fails.
pub(crate) trait LoopTarget: FrameTarget {
    /// Refuse continuation without waiting for the next scheduled frame.
    fn check(&self) -> Result<(), RenderError>;
}

impl LoopTarget for crate::DirectTarget<'_, '_> {
    fn check(&self) -> Result<(), RenderError> {
        if self.retirement_error().is_some() {
            Err(RenderError::DirectHalted)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "direct_input_retirement_tests.rs"]
mod input_retirement_tests;
