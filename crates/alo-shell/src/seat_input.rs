//! Owned libinput lifetime with seat checks at dispatch and delivery boundaries.

use crate::{SessionInput, SessionInputStatus};
use smithay::{
    backend::session::Session,
    reexports::input::{Event, Libinput},
};
use std::io;

#[cfg(test)]
#[path = "seat_input_tests.rs"]
mod tests;

/// A trusted compositor must clear keyboard and pointer state on reset.
pub enum InputUpdate<'a> {
    /// An event offered synchronously while the seat poll still permits access.
    Event(&'a Event),
    /// Context retirement; call `Server::clear_input` and flush client events.
    Reset,
}

/// Original dispatch/authority failure and independent device cleanup evidence.
#[derive(Debug, thiserror::Error)]
#[error("input context: {source}; cleanup: {cleanup:?}")]
pub struct InputDispatchError {
    /// First failure that stopped this context.
    pub source: io::Error,
    /// Restricted callback failure observed after suspension and destruction.
    pub cleanup: Option<io::Error>,
}

/// One seat's input context. No context or descriptor clone is exposed.
///
/// Call dispatch regularly, including while idle. The supplied poll must drain
/// seat notifications and refuse a paused or failed lifetime, even if activation
/// followed pause in the same batch. Retirement is terminal; build a fresh owner
/// after reactivation. This is trusted shell plumbing, never an agent interface.
/// Drop suspends devices; explicit shutdown additionally resets client input and
/// reports close errors. The seat notifier must outlive this owner.
pub struct SeatInput(Lifetime<Libinput>);

impl SeatInput {
    /// Assign the manager's seat through udev, refusing inactive or invalid seats.
    /// No device is opened directly, and assignment callback failures are fatal.
    pub fn new<S: Session + 'static>(session: S) -> Result<Self, InputDispatchError> {
        let seat = session.seat();
        if !session.is_active() || seat.is_empty() || seat.contains('\0') {
            return Err(InputDispatchError {
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "inactive or invalid input seat",
                ),
                cleanup: None,
            });
        }
        let (interface, status) = SessionInput::new(session);
        let mut context = Libinput::new_with_udev(interface);
        let assignment = context
            .udev_assign_seat(&seat)
            .map_err(|()| io::Error::other("assign input seat"));
        let mut owner = Lifetime {
            context: Some(context),
            status,
        };
        if let Err(source) = assignment.and_then(|()| owner.status.check()) {
            return Err(owner.fail(source));
        }
        Ok(Self(owner))
    }

    /// Poll before reading and before each event; reset on any failure.
    ///
    /// The handler translates keyboard/pointer events using existing Server
    /// validation. It must handle device removal; this owner provides lifetime
    /// enforcement, not per-device key accounting. A handler error also retires
    /// the context. Reset must be infallible (queue cleanup for a later flush).
    pub fn dispatch(
        &mut self,
        poll: impl FnMut() -> io::Result<()>,
        mut handle: impl FnMut(InputUpdate<'_>) -> io::Result<()>,
    ) -> Result<(), InputDispatchError> {
        self.0.dispatch(poll, |event| match event {
            Some(event) => handle(InputUpdate::Event(event)),
            None => handle(InputUpdate::Reset),
        })
    }

    /// Suspend and destroy before reporting callback errors; reset exactly once.
    pub fn shutdown(mut self, reset: impl FnOnce()) -> io::Result<()> {
        if self.0.context.is_some() {
            self.0.retire();
            reset();
        }
        self.0.status.check()
    }
}

/// Small injection boundary for ordering and dispatch-error tests.
trait Context: Iterator {
    /// Read ready input without blocking.
    fn dispatch(&mut self) -> io::Result<()>;
    /// Stop monitoring and close devices through restricted callbacks.
    fn suspend(&mut self);
}

impl Context for Libinput {
    fn dispatch(&mut self) -> io::Result<()> {
        Libinput::dispatch(self)
    }
    fn suspend(&mut self) {
        Libinput::suspend(self);
    }
}

/// No resumption or extraction API: retirement cannot be undone accidentally.
struct Lifetime<C: Context> {
    /// Taken before suspension, preventing repeated cleanup or resurrection.
    context: Option<C>,
    /// Callback errors remain observable after the context drops.
    status: SessionInputStatus,
}

impl<C: Context> Lifetime<C> {
    /// Explicit suspension closes devices even if an event retained upstream refs.
    fn retire(&mut self) {
        if let Some(mut context) = self.context.take() {
            context.suspend();
            drop(context);
        }
    }

    /// Preserve the triggering failure independently of suspension errors.
    fn fail(&mut self, source: io::Error) -> InputDispatchError {
        self.retire();
        InputDispatchError {
            source,
            cleanup: self.status.check().err(),
        }
    }

    /// Apply authority checks before reading or delivering, then reset on refusal.
    fn dispatch(
        &mut self,
        mut poll: impl FnMut() -> io::Result<()>,
        mut handle: impl FnMut(Option<&C::Item>) -> io::Result<()>,
    ) -> Result<(), InputDispatchError> {
        let was_live = self.context.is_some();
        let result = (|| {
            let context = self
                .context
                .as_mut()
                .ok_or_else(|| io::Error::other("input context retired"))?;
            poll()?;
            self.status.check()?;
            context.dispatch()?;
            self.status.check()?;
            // Poll even on an empty batch: dispatch can race with seat disable.
            loop {
                poll()?;
                self.status.check()?;
                let Some(event) = context.next() else {
                    break;
                };
                handle(Some(&event))?;
            }
            Ok(())
        })();
        result.map_err(|source| {
            let mut error = self.fail(source);
            if was_live && let Err(reset) = handle(None) {
                // Preserve both cleanup failures if a handler violates its contract.
                error.cleanup = Some(io::Error::other(format!(
                    "reset: {reset}; callbacks: {:?}",
                    error.cleanup
                )));
            }
            error
        })
    }
}

impl<C: Context> Drop for Lifetime<C> {
    fn drop(&mut self) {
        self.retire();
    }
}
