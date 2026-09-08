//! Connect the direct frame loop to a session-owned libinput context.

use crate::{DirectFrame, DirectLoopError, DirectLoopResult, SeatInput, Server, SessionError};
use std::io;

impl crate::DirectSession {
    /// Run one display/input lifetime using the same libseat connection.
    ///
    /// The caller supplies a current GLES renderer, as for `run_compositor`.
    /// Input acquisition follows fresh display discovery. Latched seat polling
    /// guards dispatch and every event, including idle iterations. Stop, pause or
    /// failure suspends input and flushes releases before attempting output
    /// retirement. Input cleanup, both flushes and display close remain independent
    /// results. There is no automatic resume. Drop the server after failed output
    /// retirement; upstream early-disable acknowledgement remains a limitation.
    /// Acquisition failure also clears and attempts to flush input. The outer
    /// SessionError preserves an acquisition failure; its diagnostic includes any
    /// accompanying flush failure. This is trusted shell plumbing, not agent IPC.
    pub fn run_compositor_with_input(
        &mut self,
        server: &mut Server,
        renderer: &mut smithay::backend::renderer::gles::GlesRenderer,
        mut next: impl FnMut() -> DirectFrame,
    ) -> Result<crate::ActiveSessionResult<DirectLoopResult>, SessionError> {
        server.clear_input();
        let manager = self.input_session();
        let result = self.with_active_device(|fd, poll| {
            let setup = (|| {
                let output = crate::discover_atomic_output(fd)?;
                poll()?;
                let (width, height) = output.output.mode.size();
                let input = RoutedInput {
                    owner: SeatInput::new(manager)?,
                    extent: (i32::from(width), i32::from(height)),
                };
                Ok::<_, DirectLoopError>((output, input))
            })();
            match setup {
                Ok((output, input)) => crate::direct_loop::run_with_input(
                    server,
                    crate::DirectTarget::new(renderer, fd, output),
                    poll,
                    &mut next,
                    input,
                ),
                Err(error) => DirectLoopResult {
                    outcome: Err(error),
                    input_cleanup: None,
                    input_flush: Some(server.flush()),
                    retirement: None,
                    flush: None,
                },
            }
        });
        match result {
            Err(error) => match server.flush() {
                Ok(()) => Err(error),
                Err(flush) => Err(SessionError::Backend {
                    stage: "acquire display and flush input reset",
                    source: io::Error::other(format!("{error}; flush: {flush}")),
                }),
            },
            result => result,
        }
    }
}

/// Loop injection boundary; production always uses the real SeatInput owner.
pub(crate) trait LoopInput {
    /// Route ready events synchronously under the active scope's latched poll.
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError>;
    /// Consume input before output retirement, retaining callback close errors.
    fn shutdown(self, server: &mut Server) -> Option<io::Result<()>>;
}

/// Display-only compatibility path does not acquire or poll input.
pub(crate) struct NoInput;
impl LoopInput for NoInput {
    fn dispatch(
        &mut self,
        _: &mut Server,
        _: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        Ok(())
    }
    fn shutdown(self, _: &mut Server) -> Option<io::Result<()>> {
        None
    }
}

/// Mode-sized routing owner; no libinput reference escapes its callback.
pub(crate) struct RoutedInput {
    /// Same-seat context, consumed at loop exit.
    pub(crate) owner: SeatInput,
    /// Scale-one dimensions from this lifetime's freshly discovered mode.
    pub(crate) extent: (i32, i32),
}
impl LoopInput for RoutedInput {
    fn dispatch(
        &mut self,
        server: &mut Server,
        poll: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        self.owner.dispatch(
            || poll().map_err(io::Error::other),
            |update| {
                server
                    .libinput_update(update, self.extent)
                    .map_err(io::Error::other)
            },
        )?;
        Ok(())
    }
    fn shutdown(self, server: &mut Server) -> Option<io::Result<()>> {
        Some(self.owner.shutdown(|| server.clear_input()))
    }
}

#[cfg(test)]
#[path = "direct_input_loop_tests.rs"]
mod tests;
