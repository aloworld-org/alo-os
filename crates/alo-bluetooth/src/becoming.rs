//! **What a paired device becomes on this machine**, which is not the same
//! question as what it is allowed to do.
//!
//! A headset becomes a place sound comes out of and goes into — and it becomes
//! it **in `alo-sound`'s list**, beside the laptop's own speakers, with the same
//! pin, the same volume and the same mute as anything else there. This crate
//! keeps no second list of volumes and no second idea of which device is in use;
//! a person who pins their headphones pins them once.
//!
//! A keyboard becomes a keyboard, a mouse becomes a mouse, and everything else
//! becomes nothing in particular, which is honest and common: a phone this
//! machine has paired with is a phone this machine has paired with.
//!
//! None of it is permission. See [`crate::granting`], which is the file about
//! that.

use crate::reported::Kind;

/// **What a device of this kind becomes.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Becomes {
    /// One of the machine's sound devices, held by `alo-sound` and not here.
    ASoundDevice,
    /// Something a person types on.
    AKeyboard,
    /// Something a person points with.
    APointer,
    /// Nothing in particular: paired, and that is all.
    NothingInParticular,
}

impl Becomes {
    /// What a device of this kind becomes.
    #[must_use]
    pub const fn of(kind: Kind) -> Self {
        match kind {
            Kind::Audio => Self::ASoundDevice,
            Kind::Keyboard => Self::AKeyboard,
            Kind::Mouse => Self::APointer,
            Kind::Other => Self::NothingInParticular,
        }
    }

    /// **Whether this is another crate's to hold from here on.**
    ///
    /// True for sound, and the reason is the one that matters: two lists of
    /// audio devices would disagree, and the day they disagreed a person's call
    /// would come out of the wrong one.
    #[must_use]
    pub const fn is_held_elsewhere(self) -> bool {
        matches!(self, Self::ASoundDevice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The kind decides what it becomes**, and sound is somebody else's list.
    #[test]
    fn what_a_device_becomes_follows_from_what_it_is() {
        assert_eq!(Becomes::of(Kind::Audio), Becomes::ASoundDevice);
        assert_eq!(Becomes::of(Kind::Keyboard), Becomes::AKeyboard);
        assert_eq!(Becomes::of(Kind::Mouse), Becomes::APointer);
        assert_eq!(Becomes::of(Kind::Other), Becomes::NothingInParticular);

        assert!(Becomes::of(Kind::Audio).is_held_elsewhere());
        for kind in [Kind::Keyboard, Kind::Mouse, Kind::Other] {
            assert!(
                !Becomes::of(kind).is_held_elsewhere(),
                "{kind:?} was handed to a crate that does not want it"
            );
        }
    }
}
