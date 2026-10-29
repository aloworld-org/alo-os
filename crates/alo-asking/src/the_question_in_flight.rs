//! **The question on its way to a model, and the one way to stop it.**
//!
//! [ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md):
//! when a person presses Stop, a question already on its way to a provider is
//! cancelled, so **nothing more leaves after Stop**. A request that is blocked
//! waiting for a provider cannot be interrupted from inside itself, so its
//! connection is opened here, by `Stoppable`, and kept where another thread
//! can shut it: [`stop_the_question_in_flight`].
//!
//! # One at a time, and why that is enough
//!
//! `alo-agentd` serves one turn at a time, and a turn asks one question at a
//! time, so this holds one connection. A process that put two questions at
//! once would have the second replace the first here, and a Stop would shut
//! only the later one — which is why this is used by the daemon's road and
//! stated, rather than being a general facility.
//!
//! # What stopping does, exactly
//!
//! It shuts the connection both ways, so the provider sees it close and the
//! request in this process fails at once; and it is remembered until the next
//! question begins, so a connection the request was about to open is refused
//! rather than opened after Stop. The question then ends as
//! `alo_answering::WentWrong::Stopped`, and its departure — and with it the
//! egress indicator's line — ends with it.

use std::io::{self, Read as _, Write as _};
use std::net::{Shutdown, TcpStream};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use ureq::unversioned::transport::{
    Buffers, ConnectionDetails, Connector, Either, LazyBuffers, NextTimeout, Transport,
};

/// What is in flight, and whether a Stop has arrived for it.
#[derive(Debug, Default)]
struct InFlight {
    /// A second handle on the connection the question went out on.
    connection: Option<TcpStream>,
    /// Whether a Stop arrived since this question began.
    stopped: bool,
}

/// The one question this process has in flight.
static IN_FLIGHT: Mutex<InFlight> = Mutex::new(InFlight {
    connection: None,
    stopped: false,
});

/// How long connecting may take, as the connector `ureq` ships allows.
const WHILE_CONNECTING: Duration = Duration::from_secs(10);

/// **Stop the question in flight**, if there is one: shut its connection, and
/// refuse any connection it was about to open.
///
/// Answers whether a connection was open to shut. Called by `alo-agentd` when
/// the agent whose question it is hangs up (ADR 0139); safe to call from any
/// thread, and at any time — with nothing in flight it only marks the next
/// connection of this question refused, and the next question clears that.
pub fn stop_the_question_in_flight() -> bool {
    let mut held = IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner);
    held.stopped = true;
    held.connection
        .take()
        .is_some_and(|connection| connection.shutdown(Shutdown::Both).is_ok())
}

/// A question that has begun: clears what an earlier one left, and on drop
/// lets its connection go.
#[derive(Debug)]
pub(crate) struct Begun(());

/// **A question begins.** Whatever an earlier one left — its connection, a
/// Stop that arrived for it — is cleared.
pub(crate) fn begun() -> Begun {
    let mut held = IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner);
    *held = InFlight::default();
    Begun(())
}

impl Begun {
    /// Whether a Stop arrived while this question was in flight.
    pub(crate) fn was_stopped(&self) -> bool {
        IN_FLIGHT
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .stopped
    }
}

impl Drop for Begun {
    fn drop(&mut self) {
        IN_FLIGHT
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .connection = None;
    }
}

/// **A TCP connector whose connection can be shut from another thread**: the
/// one `ureq` ships, but keeping a second handle on the socket in
/// [`IN_FLIGHT`]. Chained where the shipped TCP connector would be, so a proxy
/// and TLS go round it exactly as before.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Stoppable;

impl<In: Transport> Connector<In> for Stoppable {
    type Out = Either<In, StoppableConnection>;

    fn connect(
        &self,
        details: &ConnectionDetails,
        chained: Option<In>,
    ) -> Result<Option<Self::Out>, ureq::Error> {
        if let Some(chained) = chained {
            return Ok(Some(Either::A(chained)));
        }
        let within = details
            .timeout
            .not_zero()
            .map_or(WHILE_CONNECTING, |left| (*left).min(WHILE_CONNECTING));
        let mut refused = None;
        for address in details.addrs.iter() {
            if IN_FLIGHT
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .stopped
            {
                return Err(ureq::Error::Io(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "the person stopped this before it was sent",
                )));
            }
            match TcpStream::connect_timeout(address, within) {
                Ok(stream) => {
                    let second = stream.try_clone()?;
                    let mut held = IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner);
                    if held.stopped {
                        drop(stream.shutdown(Shutdown::Both));
                        return Err(ureq::Error::Io(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "the person stopped this as it was being sent",
                        )));
                    }
                    held.connection = Some(second);
                    let buffers = LazyBuffers::new(
                        details.config.input_buffer_size(),
                        details.config.output_buffer_size(),
                    );
                    return Ok(Some(Either::B(StoppableConnection { stream, buffers })));
                }
                Err(why) if why.kind() == io::ErrorKind::TimedOut => {
                    return Err(ureq::Error::Timeout(ureq::Timeout::Connect));
                }
                Err(why) => refused = Some(why),
            }
        }
        Err(ureq::Error::Io(refused.unwrap_or_else(|| {
            io::Error::new(io::ErrorKind::ConnectionRefused, "nowhere was registered")
        })))
    }
}

/// The connection [`Stoppable`] opened.
#[derive(Debug)]
pub(crate) struct StoppableConnection {
    /// The socket.
    stream: TcpStream,
    /// What `ureq` reads into and writes from.
    buffers: LazyBuffers,
}

impl Transport for StoppableConnection {
    fn buffers(&mut self) -> &mut dyn Buffers {
        &mut self.buffers
    }

    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), ureq::Error> {
        self.stream
            .set_write_timeout(timeout.not_zero().map(|left| *left))?;
        let output = self
            .buffers
            .output()
            .get(..amount)
            .ok_or_else(|| io::Error::other("the client asked for more than it buffered"))?;
        self.stream
            .write_all(output)
            .map_err(|why| timed_out_or(why, timeout))
    }

    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, ureq::Error> {
        self.stream
            .set_read_timeout(timeout.not_zero().map(|left| *left))?;
        let read = self
            .stream
            .read(self.buffers.input_append_buf())
            .map_err(|why| timed_out_or(why, timeout))?;
        self.buffers.input_appended(read);
        Ok(read > 0)
    }

    fn is_open(&mut self) -> bool {
        // One request per client (ADR 0020): never handed back to a pool.
        false
    }
}

/// A timeout the socket reported, as the client's own kind of timeout.
fn timed_out_or(why: io::Error, timeout: NextTimeout) -> ureq::Error {
    match why.kind() {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => ureq::Error::Timeout(timeout.reason),
        _ => ureq::Error::Io(why),
    }
}
