//! The broker's verbs: a closed list, and the only list there is.
//!
//! Twelve verbs across the operations ADR 0001 §2 calls genuinely privileged —
//! printers, the network, updates, storage, and which system this machine
//! starts next — and each takes exactly one argument of one of
//! `crate::arguments`' two shapes. This file is the list's **shape**; none of
//! the verbs is carried out by this crate. What carries each out is written by
//! the task that owns it (`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`,
//! tasks 2 to 4), behind [`crate::Carrying`], and it receives a [`SystemVerb`]
//! that has already been proven to be exactly what a person approved.
//!
//! # What is deliberately not on it
//!
//! **No verb formats, repartitions or erases anything**, and none writes to a
//! partition table (task 4's constraint). **No verb takes a password**: joining
//! a protected network asks the person in a surface the agent cannot read (task
//! 3), and [`SystemVerb::JoinNetwork`] has nowhere to put one. **No VPN
//! configuration** in v0.5. **No read**: checking a disk's health is a read, and
//! a read answers inside a turn without an approval (ADR 0001 §5). Task 4 found
//! it needs no privilege at all: the disk service answers it to anybody on the
//! system bus (`alo-drives`), so putting it here would only have made every read
//! wait for an approval it should not need. And, above
//! all, **no verb runs anything, and none takes a path, a command, a device
//! name or a line of configuration.**
//!
//! # The twelfth, and why it belongs on a list that had eleven
//!
//! [`SystemVerb::RestartIntoWindows`] sets the firmware's next start to the
//! Windows already on the disk, for **one** start, and leaves the machine's
//! ordinary start-up order exactly as it was
//! ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
//! *what stays as it was*). Writing a firmware variable is privileged, which is
//! what ADR 0001 §2 puts behind this list; the argument is the identity of a
//! start-up entry **the firmware itself reported**, so nothing here names a
//! loader, a path or a disk. And it **restarts nothing**: the restart is the
//! person's own, afterwards, exactly as it is for the update verbs.
//!
//! # The names are a contract
//!
//! [`SystemVerb::name`] is what crosses the door and what the record keeps.
//! They change additively, like every agent verb (`docs/contracts/agent-verbs.md`).

use crate::arguments::{Argument, Identity, Switch};

/// One of the broker's verbs, with the one argument it takes.
///
/// `Copy`, and held to it below: a `String`, `PathBuf` or `Vec` anywhere in a
/// verb's argument would stop this crate compiling, which is the plan's *no
/// free string* walked by the compiler over every argument type there is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemVerb {
    /// Add a printer the print service found.
    AddPrinter(Identity),
    /// Remove a printer this machine has.
    RemovePrinter(Identity),
    /// Make a printer this machine has the one it prints to by default.
    SetDefaultPrinter(Identity),
    /// Join a network the machine can see, by the identity the network manager
    /// reported for it.
    JoinNetwork(Identity),
    /// Forget a network this machine has joined.
    ForgetNetwork(Identity),
    /// Turn the machine's wireless radio on or off.
    SetRadio(Switch),
    /// Use the proxy setting the proxy service holds, by its identity.
    SetProxy(Identity),
    /// Apply an update that is already staged, by the digest of its build.
    ApplyStagedUpdate(Identity),
    /// Go back to the build the machine ran before, by the digest of that build.
    RollBack(Identity),
    /// Mount a removable drive a person plugged in, for the signed-in person.
    MountDrive(Identity),
    /// Eject a removable drive.
    EjectDrive(Identity),
    /// Start the Windows this machine already has, once, at the next start —
    /// by the identity the firmware reported for its start-up entry.
    RestartIntoWindows(Identity),
}

/// The compiler, walking every argument of every verb: nothing that is not
/// `Copy` — no `String`, no `PathBuf`, no `Vec` — can be inside one.
const _EVERY_ARGUMENT_IS_COPY: () = {
    /// Compiles only for a type that holds nothing owned.
    const fn held_to_copy<T: Copy>() {}
    held_to_copy::<SystemVerb>();
};

/// Every name on the list, in the order the enum declares the verbs.
pub const EVERY_NAME: [&str; 12] = [
    "printers.add",
    "printers.remove",
    "printers.set-default",
    "network.join",
    "network.forget",
    "network.radio",
    "network.set-proxy",
    "updates.apply",
    "updates.roll-back",
    "storage.mount",
    "storage.eject",
    "starting.windows-next",
];

impl SystemVerb {
    /// The verb's name, as the door writes it and the record keeps it.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::AddPrinter(_) => "printers.add",
            Self::RemovePrinter(_) => "printers.remove",
            Self::SetDefaultPrinter(_) => "printers.set-default",
            Self::JoinNetwork(_) => "network.join",
            Self::ForgetNetwork(_) => "network.forget",
            Self::SetRadio(_) => "network.radio",
            Self::SetProxy(_) => "network.set-proxy",
            Self::ApplyStagedUpdate(_) => "updates.apply",
            Self::RollBack(_) => "updates.roll-back",
            Self::MountDrive(_) => "storage.mount",
            Self::EjectDrive(_) => "storage.eject",
            Self::RestartIntoWindows(_) => "starting.windows-next",
        }
    }

    /// The one argument it takes.
    #[must_use]
    pub const fn argument(&self) -> Argument {
        match *self {
            Self::AddPrinter(identity)
            | Self::RemovePrinter(identity)
            | Self::SetDefaultPrinter(identity)
            | Self::JoinNetwork(identity)
            | Self::ForgetNetwork(identity)
            | Self::SetProxy(identity)
            | Self::ApplyStagedUpdate(identity)
            | Self::RollBack(identity)
            | Self::MountDrive(identity)
            | Self::EjectDrive(identity)
            | Self::RestartIntoWindows(identity) => Argument::Identity(identity),
            Self::SetRadio(switch) => Argument::Switch(switch),
        }
    }

    /// A verb, from a name on the list and an argument written in exactly the
    /// shape that verb takes — or nothing.
    ///
    /// A name not on the list is nothing. So is a name on it with an argument
    /// of the other shape, or of no shape at all: *one of its verbs, exactly*.
    #[must_use]
    pub fn read(name: &str, argument: &str) -> Option<Self> {
        let identity = || Identity::read(argument);
        match name {
            "printers.add" => identity().map(Self::AddPrinter),
            "printers.remove" => identity().map(Self::RemovePrinter),
            "printers.set-default" => identity().map(Self::SetDefaultPrinter),
            "network.join" => identity().map(Self::JoinNetwork),
            "network.forget" => identity().map(Self::ForgetNetwork),
            "network.radio" => Switch::read(argument).map(Self::SetRadio),
            "network.set-proxy" => identity().map(Self::SetProxy),
            "updates.apply" => identity().map(Self::ApplyStagedUpdate),
            "updates.roll-back" => identity().map(Self::RollBack),
            "storage.mount" => identity().map(Self::MountDrive),
            "storage.eject" => identity().map(Self::EjectDrive),
            "starting.windows-next" => identity().map(Self::RestartIntoWindows),
            _ => None,
        }
    }

    /// One of every verb there is, each holding `identity` or `switch`.
    ///
    /// For the tests that walk the whole list, here and in `tests/`. The match
    /// in [`SystemVerb::name`] is exhaustive and this is counted against
    /// [`EVERY_NAME`], so a verb added to the enum and not here is a test that
    /// fails rather than a verb nothing walked.
    #[must_use]
    pub const fn one_of_each(identity: Identity, switch: Switch) -> [Self; 12] {
        [
            Self::AddPrinter(identity),
            Self::RemovePrinter(identity),
            Self::SetDefaultPrinter(identity),
            Self::JoinNetwork(identity),
            Self::ForgetNetwork(identity),
            Self::SetRadio(switch),
            Self::SetProxy(identity),
            Self::ApplyStagedUpdate(identity),
            Self::RollBack(identity),
            Self::MountDrive(identity),
            Self::EjectDrive(identity),
            Self::RestartIntoWindows(identity),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Something reported, for a verb to hold.
    fn a_printer() -> Identity {
        Identity::of_what_was_reported(b"a printer")
    }

    /// **The list, the names and the reader agree**: every verb reads back from
    /// its own name and argument, in the order the names are listed, and no two
    /// share a name.
    #[test]
    fn every_verb_reads_back_from_its_name_and_argument() {
        let every = SystemVerb::one_of_each(a_printer(), Switch::Off);
        let names: Vec<&str> = every.iter().map(SystemVerb::name).collect();
        assert_eq!(names, EVERY_NAME);
        assert_eq!(
            EVERY_NAME.iter().collect::<BTreeSet<_>>().len(),
            EVERY_NAME.len()
        );
        for verb in every {
            assert_eq!(
                SystemVerb::read(verb.name(), &verb.argument().written()),
                Some(verb),
                "{}",
                verb.name()
            );
        }
    }

    /// **An argument of the wrong shape is not that verb.** A switch where an
    /// identity goes, an identity where a switch goes.
    #[test]
    fn an_argument_of_the_other_shape_is_not_the_verb() {
        let identity = a_printer().written();
        assert_eq!(SystemVerb::read("network.radio", &identity), None);
        assert_eq!(SystemVerb::read("printers.add", "on"), None);
    }

    /// **A name close to one on the list is not on it**, and neither is
    /// anything shaped like the escape hatch ADR 0001 §1 forbids.
    #[test]
    fn a_name_not_on_the_list_is_nothing() {
        let identity = a_printer().written();
        for name in [
            "printers.Add",
            "printers.add ",
            "printers",
            "exec",
            "run",
            "shell",
            "storage.format",
            "storage.erase",
            "storage.partition",
            "starting.windows",
            "starting.windows-next ",
            "starting",
            "",
        ] {
            assert_eq!(SystemVerb::read(name, &identity), None, "{name}");
        }
    }
}
