//! The machine's media server, as one question.
//!
//! The plan's acceptance says where what is in use is read from, and it is a
//! sentence about provenance: **the media server's own record of open streams,
//! rather than what applications say they are doing.** An application that
//! wanted to use the camera without a line appearing would only have to stop
//! announcing itself, and a record kept by whatever the applications tell it
//! would have nothing to say about that. The server knows, because the stream
//! is open through it.
//!
//! [`Streams`] is that question, and it is a trait for one reason: the machine
//! reaches its media server by running a program, and a test cannot. The only
//! implementation alo OS ships is [`crate::TheMediaServer`], and
//! [`crate::heard`] is the one file that turns a record into values.
//!
//! # It answers everything, or it refuses
//!
//! There is no third answer. An implementation that could not reach the server
//! answers [`crate::NotHeard`] and the indicator says so; one that reached it
//! answers **every** stream it found. Nothing here filters, and nothing here
//! takes a list of what to leave out — the plan's acceptance is that there is
//! no variant that hides a use and no allow-list of trusted applications, and
//! the shape of this trait is the first place that has to be true.

use crate::refusing::NotHeard;
use crate::uses::Use;

/// What the machine's media server says is open right now.
pub trait Streams {
    /// Every current use of the screen, the camera and the microphone, as the
    /// server's own record has them.
    ///
    /// Takes `&mut self` because reaching a media server is not a pure
    /// question: the one alo OS ships starts a program and waits for it.
    ///
    /// # Errors
    /// [`NotHeard`], when the server is not there, does not answer, or answers
    /// something that cannot be read. Never an empty list standing in for one
    /// of those: a quiet room and an unanswerable question are different facts
    /// and a person is told which.
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard>;
}
