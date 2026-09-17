//! What a system verb may be given, and the reason there are only two shapes.
//!
//! ADR 0001 §2: the broker has *no free-form parameters*. The plan for this
//! crate makes that sharper — **no string that becomes a path, a command, a
//! configuration line or a device name** — and the way to keep a rule like that
//! is to have nowhere a string could go. So an argument is one of two things,
//! and neither of them holds text:
//!
//! | | |
//! |---|---|
//! | [`Identity`] | Thirty-two bytes naming something the machine itself reported |
//! | [`Switch`] | On, or off |
//!
//! # An identity is compared, never interpreted
//!
//! A printer is added, a network joined, a drive mounted — each of them
//! *something the machine already found*: the print service reported the
//! printer, the network manager reported the network, the kernel reported the
//! drive. The broker is never told what the thing is called. It is told the
//! SHA-256 of the identity the rented service reported, and the verb that
//! carries it out asks the rented service for what it has **now** and acts on
//! the one whose identity digests to the same bytes — or on nothing.
//!
//! That is what makes a hostile argument harmless rather than merely checked:
//! thirty-two bytes can name nothing the machine did not report, cannot be a
//! path, cannot be spliced into a command, and cannot be a device name, because
//! nothing ever reads them as anything but bytes to compare.
//!
//! Every type here is `Copy`, and `crate::verbs` asks the compiler to hold the
//! verb list to that. A `String`, a `PathBuf` or a `Vec` anywhere inside an
//! argument would stop it compiling.

use ring::digest;

/// How many bytes an [`Identity`] is.
pub const IDENTITY_BYTES: usize = 32;

/// Something the machine itself reported, named by the SHA-256 of the identity
/// it was reported under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identity([u8; IDENTITY_BYTES]);

impl Identity {
    /// The identity of what a rented service reported, as it reported it.
    ///
    /// Whoever proposes a system verb makes one of these from the service's own
    /// words — `alo-printing`'s found printer, the network manager's connection
    /// — and whoever carries the verb out makes one the same way from what the
    /// service reports at that moment, and compares.
    #[must_use]
    pub fn of_what_was_reported(reported: &[u8]) -> Self {
        let mut bytes = [0; IDENTITY_BYTES];
        bytes.copy_from_slice(digest::digest(&digest::SHA256, reported).as_ref());
        Self(bytes)
    }

    /// The bytes themselves.
    #[must_use]
    pub const fn bytes(&self) -> &[u8; IDENTITY_BYTES] {
        &self.0
    }

    /// Read an identity as the door writes it: sixty-four lowercase hexadecimal
    /// characters and nothing else.
    ///
    /// One spelling only. Upper case, a prefix or a separator is refused rather
    /// than normalised, because a request that can be written two ways is a
    /// request whose proof has to be checked against both.
    #[must_use]
    pub fn read(written: &str) -> Option<Self> {
        let mut bytes = [0; IDENTITY_BYTES];
        crate::hex::read_into(written, &mut bytes).then_some(Self(bytes))
    }

    /// As the door writes it.
    #[must_use]
    pub fn written(&self) -> String {
        crate::hex::written(&self.0)
    }
}

/// On, or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Switch {
    /// On.
    On,
    /// Off.
    Off,
}

impl Switch {
    /// Read a switch as the door writes it: `on` or `off`.
    #[must_use]
    pub fn read(written: &str) -> Option<Self> {
        match written {
            "on" => Some(Self::On),
            "off" => Some(Self::Off),
            _ => None,
        }
    }

    /// As the door writes it.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

/// Whichever of the two a verb takes.
///
/// What a proof is made over, and what the door writes. There is no third
/// variant, and adding one is adding a shape of argument to a privileged
/// component — a decision, with `tests/every_argument_is_a_closed_type.rs` in
/// its way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Argument {
    /// Something the machine reported.
    Identity(Identity),
    /// On, or off.
    Switch(Switch),
}

impl Argument {
    /// The bytes a proof is made over: one byte saying which shape, then the
    /// value. Two arguments of different shapes can never prove each other.
    #[must_use]
    pub fn proven_as(&self) -> Vec<u8> {
        match self {
            Self::Identity(identity) => [&[1_u8][..], identity.bytes()].concat(),
            Self::Switch(Switch::On) => vec![2, 1],
            Self::Switch(Switch::Off) => vec![2, 0],
        }
    }

    /// As the door writes it.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Identity(identity) => identity.written(),
            Self::Switch(switch) => switch.written().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **An identity reads back as it was written**, and is the digest of what
    /// was reported rather than what was reported.
    #[test]
    fn an_identity_is_a_digest_and_reads_back_as_written() {
        let printer = Identity::of_what_was_reported(b"ipp://printer.local/ipp/print");
        assert_eq!(Identity::read(&printer.written()), Some(printer));
        assert_eq!(printer.written().len(), 64);
        assert!(!printer.written().contains("printer"));
    }

    /// **Every other spelling is refused**: upper case, too short, too long, a
    /// prefix, a path, a command.
    #[test]
    fn an_identity_has_one_spelling_and_nothing_else_reads_as_one() {
        let printer = Identity::of_what_was_reported(b"a printer").written();
        for written in [
            printer.to_uppercase(),
            printer.chars().skip(1).collect(),
            format!("{printer}0"),
            format!("0x{printer}"),
            "/dev/sda".to_owned(),
            "; rm -rf /".to_owned(),
            String::new(),
        ] {
            assert_eq!(Identity::read(&written), None, "{written}");
        }
    }

    /// A switch is two words and no others.
    #[test]
    fn a_switch_is_on_or_off_and_nothing_else() {
        assert_eq!(Switch::read("on"), Some(Switch::On));
        assert_eq!(Switch::read("off"), Some(Switch::Off));
        for written in ["On", "1", "true", "", "on "] {
            assert_eq!(Switch::read(written), None, "{written}");
        }
    }

    /// **Different shapes never prove each other**, whatever bytes they hold.
    #[test]
    fn arguments_of_different_shapes_are_proven_as_different_bytes() {
        let on = Argument::Switch(Switch::On).proven_as();
        let off = Argument::Switch(Switch::Off).proven_as();
        let identity = Argument::Identity(Identity::of_what_was_reported(b"x")).proven_as();
        assert_ne!(on, off);
        assert_ne!(on, identity);
        assert_ne!(off, identity);
    }
}
