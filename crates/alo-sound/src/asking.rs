//! **What a machine's sound can be asked and told**, as one small surface.
//!
//! Everything above this line — pins, what a person set, which device was meant
//! — is decided against [`Sound`] rather than against a machine, so that the
//! deciding is tested on every machine the gate runs on and the reaching is
//! tested on a machine that has sound. [`crate::server::TheAudioServer`] is the
//! one implementation that talks to a real one.

use crate::device::{Identity, Kind, Volume};
use crate::heard::Heard;
use crate::mute::Mute;
use crate::refusing::{NotDone, NotHeard};

/// **A machine's sound**, asked and told.
pub trait Sound {
    /// Everything the machine says about its sound right now.
    ///
    /// # Errors
    /// [`NotHeard`] where there is no media server, it would not answer, or it
    /// answered something this crate cannot read. Never an empty list standing
    /// in for a question that could not be asked.
    fn heard_now(&mut self) -> Result<Heard, NotHeard>;

    /// **Use this device for this kind from now on.**
    ///
    /// A call already running moves with it: what moves the sound is the rented
    /// server's own policy, which this asks rather than reimplements.
    ///
    /// # Errors
    /// [`NotDone`] where the device is not here, or the machine refused.
    fn choose(&mut self, identity: &Identity, kind: Kind) -> Result<(), NotDone>;

    /// **Set how loud one device is.**
    ///
    /// # Errors
    /// [`NotDone`] where the device is not here, or the machine refused.
    fn set_volume(&mut self, identity: &Identity, volume: Volume) -> Result<(), NotDone>;

    /// **Mute or unmute one device**, at the source.
    ///
    /// # Errors
    /// [`NotDone`] where the device is not here, or the machine refused.
    fn set_mute(&mut self, identity: &Identity, mute: Mute) -> Result<(), NotDone>;
}
