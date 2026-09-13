//! What the person on this machine called the other machine, for the door.
//!
//! A pairing row names the other machine by its identity and nothing else
//! (`alo_nearby::Pairing::with`); what a person called it when they paired is
//! theirs, kept wherever the shell keeps it, and it decides nothing — a
//! caller that handed over the wrong name would mislabel a record, and only
//! the identity is what the proof is about (`alo_nearby::origin`). So the
//! door asks for it through this trait at the moment a verb arrives, and a
//! shell or a daemon answers from its own list.
//!
//! A closure of the right shape is one, so a test needs no type of its own.

use alo_nearby::MachineId;

/// What names a paired machine the way this machine's person named it.
pub trait Naming {
    /// The name the person here gave this machine, if they gave one.
    ///
    /// `None` and an empty name both mean *nothing*: the origin is then
    /// named by its identity, which `alo_nearby::Origin::proven` decides.
    fn called(&self, machine: &MachineId) -> Option<String>;
}

impl<F: Fn(&MachineId) -> Option<String>> Naming for F {
    fn called(&self, machine: &MachineId) -> Option<String> {
        self(machine)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use alo_nearby::MachineId;

    use super::Naming;

    /// A closure names a machine, and one that names nothing names nothing.
    #[test]
    fn a_closure_is_a_naming() {
        let reception = MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap();
        let naming = |machine: &MachineId| {
            (machine == &reception).then(|| "the reception machine".to_owned())
        };
        assert_eq!(
            naming.called(&reception).as_deref(),
            Some("the reception machine")
        );
        let nobody = MachineId::read("99998888777766665555444433332222").unwrap();
        assert_eq!(naming.called(&nobody), None);
    }
}
