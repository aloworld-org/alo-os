//! The person's door names a machine this one is paired with, or takes its
//! name away.
//!
//! Every surface the local network built named the other machine by its
//! identity, because nothing kept the name a person gives one. This is the
//! request that gives one — `alo_protocol::FromAPerson::NameMachine` and
//! `ClearMachineName`, answered here — and [`crate::names::TheNames`] is where
//! it is held and written down, and what the indicator, the record and the
//! list read it from.
//!
//! # In this order, and every refusal changes nothing
//!
//! 1. **An identity**, or the refusal a pairing request gets for something
//!    that is not one. A machine is named *by* its identity: nothing on this
//!    door finds a machine by a name, so a name typed where the identity goes
//!    is refused rather than looked up.
//! 2. **A name the rule allows** (`alo_remembering::MachineName`), each part of
//!    the rule refused in its own sentence — before any lock is taken.
//! 3. **A pairing with that machine, standing now**, asked under the network's
//!    lock — the very list the next verb and the next question from that
//!    machine are judged against. A name is for a machine this one is paired
//!    with, and a machine merely discovered, revoked or run out is refused.
//! 4. **The name, held and written** while that lock is held, so a revocation
//!    cannot land between the question and the write. A file that could not be
//!    written is said as *until a restart*, the true sentence, rather than
//!    reversing what the person did.
//!
//! # A name decides nothing, and goes nowhere
//!
//! Nothing here sends anything, looks anything up on the network, or touches a
//! pairing or a grant: the name is kept on this machine and read where this
//! machine speaks of the other one. An agent sending either request is refused
//! by `alo-protocol` before anything reaches this file, in the words an agent
//! approving something gets.

use std::time::SystemTime;

use alo_nearby::MachineId;
use alo_protocol::{AfterNaming, ToAPerson};
use alo_remembering::{MachineName, NotAName};
use alo_strings::{Filling, Strings};

use crate::network::TheNetwork;
use crate::words::{
    A_NAME_CANNOT_BE_AN_IDENTITY, A_NAME_HAS_A_LINE_BREAK, A_NAME_HAS_NOTHING_IN_IT,
    A_NAME_IS_TOO_LONG, ONLY_A_PAIRED_MACHINE_IS_NAMED, THAT_IS_NOT_A_MACHINE, Word,
};

/// Give the machine the person named by its identity the name they gave it, or
/// take its name away when `called` is nothing — or say in one sentence why
/// nothing changed.
#[must_use]
pub fn named(
    machine: &str,
    called: Option<&str>,
    network: &TheNetwork,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    let say = |word: Word| ToAPerson::refused(&strings.say(&word.key(), &Filling::nothing()));

    // 1. An identity.
    let Ok(identity) = MachineId::read(machine.trim()) else {
        return say(THAT_IS_NOT_A_MACHINE);
    };

    // 2. A name the rule allows.
    let name = match called.map(MachineName::checked).transpose() {
        Ok(name) => name,
        Err(why) => {
            return say(match why {
                NotAName::Empty => A_NAME_HAS_NOTHING_IN_IT,
                NotAName::TooLong { .. } => A_NAME_IS_TOO_LONG,
                NotAName::AControlCharacter => A_NAME_HAS_A_LINE_BREAK,
                NotAName::AnIdentity => A_NAME_CANNOT_BE_AN_IDENTITY,
            });
        }
    };

    // 3. A pairing, standing now, under the lock.
    let shared = network.locked();
    if !shared.pairings().paired_with(&identity, now) {
        return say(ONLY_A_PAIRED_MACHINE_IS_NAMED);
    }

    // 4. The name, held and written while the lock is held.
    let now_called = name.as_ref().map(|name| name.as_str().to_owned());
    let became = match network
        .names()
        .named(&identity, name, shared.pairings(), now)
    {
        Ok(()) => AfterNaming::Kept,
        Err(why) => {
            eprintln!(
                "alo-agentd: a machine's name was changed and could not be written down: {why}"
            );
            AfterNaming::KeptUntilARestart
        }
    };
    ToAPerson::machine_named(identity.as_str(), now_called.as_deref(), became)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use alo_capability::Grants;
    use alo_corridor::Naming as _;
    use alo_nearby::{MayAskIts, Pairings};
    use alo_protocol::{AfterRevoking, FromAPerson, ToAPerson};
    use alo_record::Record;
    use alo_remembering::{MachineNames, NotRemembered};

    use super::*;
    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::holding::Holding;
    use crate::keeping_names::TheNamesFile;
    use crate::names::{KeepingNames, TheNames};
    use crate::pairing::Nearby;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NobodyIsNearby, NothingIsRemembered, TheStudioIsAt, a_directory_of_our_own, a_message,
        hour, noon, nothing_has_been_chosen, on_a_machine_that_answers, on_a_machine_with_no_turn,
        paired_between, reception, the_studio,
    };

    /// A machine nobody here is paired with.
    fn the_warehouse() -> MachineId {
        MachineId::read("11112222333344445555666677778888").unwrap()
    }

    /// Naming `machine` `called`, as the person's shell writes it.
    fn naming(machine: &str, called: &str) -> String {
        format!(
            r#"{{"name-machine":{{"machine":"{machine}","called":{}}}}}"#,
            serde_json::to_string(called).unwrap()
        )
    }

    /// Taking `machine`'s name away.
    fn clearing(machine: &str) -> String {
        format!(r#"{{"clear-machine-name":{{"machine":"{machine}"}}}}"#)
    }

    /// Reception's network, paired with the studio for its models from noon
    /// for a day, holding names written through `keeping`.
    fn reception_paired_with_the_studio(keeping: Box<dyn KeepingNames>) -> TheNetwork {
        let network = TheNetwork::on(reception())
            .calling(TheNames::remembering(MachineNames::none(), keeping));
        let (on_reception, _) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        network.locked().pairings_mut().keep(on_reception);
        network
    }

    /// Somewhere that refuses every write, as a full disk does.
    #[derive(Debug)]
    struct NoRoom;

    impl KeepingNames for NoRoom {
        fn keep(&self, _: &MachineNames) -> Result<(), NotRemembered> {
            Err(NotRemembered::NotWritten {
                at: "/var/lib/alo/machine-names.toml".into(),
                why: "no space left on device".to_owned(),
            })
        }
    }

    /// One line on the person's door, on a machine with no turn.
    fn the_person_says(line: &str, network: &TheNetwork, now: SystemTime) -> ToAPerson {
        let mut record = Record::default();
        on_a_machine_with_no_turn("naming-door", &mut record, |machine, _, strings, _, _| {
            let mut grants = Grants::default();
            what_a_person_said(
                &a_message(line),
                &mut Holding::Nobody(machine),
                &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                &Nearby {
                    network,
                    looking: &NobodyIsNearby,
                },
                strings,
                now,
            )
            .unwrap()
        })
    }

    /// The sentence this crate says for `word`.
    fn saying(word: Word) -> String {
        crate::testing::in_english()
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    }

    /// **A machine this one is paired with is named on the person's door**,
    /// the name is what the network's `Naming` answers and what the list
    /// carries beside the identity, and taking it away speaks of the machine by
    /// its identity again.
    #[test]
    fn a_paired_machine_is_named_and_the_list_carries_the_name_beside_the_identity() {
        let network = reception_paired_with_the_studio(Box::new(crate::names::NothingKeepsNames));
        let said = the_person_says(
            &naming(the_studio().as_str(), "  the studio machine "),
            &network,
            noon(),
        );
        assert_eq!(
            said.became_of_naming(),
            Some((the_studio().as_str(), AfterNaming::Kept)),
            "{said:?}"
        );
        assert_eq!(said.called(), Some("the studio machine"));
        assert_eq!(
            network.names().called(&the_studio()).as_deref(),
            Some("the studio machine")
        );

        let listed = the_person_says(r#"{"pairings":{}}"#, &network, noon());
        let paired = listed.paired().unwrap();
        assert_eq!(paired.len(), 1);
        assert_eq!(paired.first().unwrap().machine(), the_studio().as_str());
        assert_eq!(paired.first().unwrap().called(), Some("the studio machine"));

        let cleared = the_person_says(&clearing(the_studio().as_str()), &network, noon());
        assert_eq!(cleared.called(), None, "{cleared:?}");
        assert!(cleared.became_of_naming().is_some());
        assert_eq!(network.names().called(&the_studio()), None);
        let listed = the_person_says(r#"{"pairings":{}}"#, &network, noon());
        assert_eq!(listed.paired().unwrap().first().unwrap().called(), None);
    }

    /// **Something that is not an identity is refused, and nothing is
    /// named** — including the name of a machine typed where the identity
    /// goes, because nothing on this door finds a machine by its name.
    #[test]
    fn naming_something_that_is_not_an_identity_is_refused_and_names_nothing() {
        let network = reception_paired_with_the_studio(Box::new(crate::names::NothingKeepsNames));
        the_person_says(
            &naming(the_studio().as_str(), "the studio machine"),
            &network,
            noon(),
        );
        for line in [
            naming("the studio machine", "the front desk"),
            naming("192.168.1.20", "the front desk"),
            naming("", "the front desk"),
            clearing("the studio machine"),
        ] {
            let said = the_person_says(&line, &network, noon());
            assert_eq!(
                said.refusal().unwrap().text(),
                saying(THAT_IS_NOT_A_MACHINE),
                "{line}"
            );
        }
        assert_eq!(
            network.names().called(&the_studio()).as_deref(),
            Some("the studio machine"),
            "a refused request changed a name"
        );
    }

    /// **A machine this one is not paired with is not named, and its name is
    /// not taken away**: never paired, paired and revoked, and a pairing that
    /// ran out — each refused in words, with nothing written.
    #[test]
    fn naming_a_machine_this_one_is_not_paired_with_is_refused_and_writes_nothing() {
        let written = std::sync::Arc::new(std::sync::Mutex::new(0_usize));
        #[derive(Debug)]
        struct Counting(std::sync::Arc<std::sync::Mutex<usize>>);
        impl KeepingNames for Counting {
            fn keep(&self, _: &MachineNames) -> Result<(), NotRemembered> {
                *self.0.lock().unwrap() += 1;
                Ok(())
            }
        }
        let network =
            reception_paired_with_the_studio(Box::new(Counting(std::sync::Arc::clone(&written))));
        let revoked =
            reception_paired_with_the_studio(Box::new(Counting(std::sync::Arc::clone(&written))));
        assert!(revoked.locked().pairings_mut().revoke(&the_studio()));
        let two_days_later = noon() + Duration::from_secs(2 * 86_400);
        for (what, network, machine, now) in [
            ("never paired", &network, the_warehouse(), noon()),
            ("revoked", &revoked, the_studio(), noon()),
            ("ran out", &network, the_studio(), two_days_later),
        ] {
            for line in [
                naming(machine.as_str(), "a name"),
                clearing(machine.as_str()),
            ] {
                let said = the_person_says(&line, network, now);
                assert_eq!(
                    said.refusal().unwrap().text(),
                    saying(ONLY_A_PAIRED_MACHINE_IS_NAMED),
                    "{what}: {line}"
                );
            }
            assert_eq!(network.names().called(&machine), None, "{what}");
        }
        assert_eq!(*written.lock().unwrap(), 0, "a refused name was written");
    }

    /// **A name the rule does not allow is refused in its own words, and
    /// nothing is named**: nothing in it, too long, a line break, and a
    /// machine's identity.
    #[test]
    fn a_name_the_rule_does_not_allow_is_refused_in_its_own_words() {
        let network = reception_paired_with_the_studio(Box::new(crate::names::NothingKeepsNames));
        for (called, word) in [
            ("   ".to_owned(), A_NAME_HAS_NOTHING_IN_IT),
            ("x".repeat(65), A_NAME_IS_TOO_LONG),
            ("the studio\nmachine".to_owned(), A_NAME_HAS_A_LINE_BREAK),
            (
                the_warehouse().as_str().to_owned(),
                A_NAME_CANNOT_BE_AN_IDENTITY,
            ),
            (
                format!("machine:{}", reception().as_str()),
                A_NAME_CANNOT_BE_AN_IDENTITY,
            ),
        ] {
            let said = the_person_says(&naming(the_studio().as_str(), &called), &network, noon());
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert_eq!(refusal.text(), saying(word), "{called:?}");
        }
        assert_eq!(network.names().called(&the_studio()), None);
    }

    /// **A file that cannot be written leaves the name standing for the
    /// session, and says so** rather than reversing what the person did.
    #[test]
    fn a_name_that_could_not_be_written_stands_until_a_restart_and_says_so() {
        let network = reception_paired_with_the_studio(Box::new(NoRoom));
        let said = the_person_says(
            &naming(the_studio().as_str(), "the studio machine"),
            &network,
            noon(),
        );
        assert_eq!(
            said.became_of_naming(),
            Some((the_studio().as_str(), AfterNaming::KeptUntilARestart))
        );
        assert_eq!(
            network.names().called(&the_studio()).as_deref(),
            Some("the studio machine")
        );
    }

    /// **An agent naming a machine, or taking its name away, is refused in the
    /// words an agent approving something gets**, and nothing is named.
    #[test]
    fn an_agent_naming_a_machine_is_refused_as_an_approval_would_be_and_names_nothing() {
        let network = reception_paired_with_the_studio(Box::new(crate::names::NothingKeepsNames));
        network
            .names()
            .named(
                &the_studio(),
                Some(MachineName::checked("the studio machine").unwrap()),
                network.locked().pairings(),
                noon(),
            )
            .unwrap();
        let looking = TheStudioIsAt {
            at: "127.0.0.1:9".parse().unwrap(),
            looked: Cell::new(0),
        };
        let corridor = Corridor {
            network: &network,
            looking: &looking,
            naming: network.names(),
        };
        let mut questions = nothing_has_been_chosen();
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, grants, strings| {
            for line in [
                naming(the_studio().as_str(), "the reception machine"),
                clearing(the_studio().as_str()),
            ] {
                let said = what_an_agent_said(
                    &a_message(&line),
                    turning,
                    &mut questions,
                    Some(&corridor),
                    grants,
                    strings,
                    hour(),
                    noon(),
                );
                let refusal = said.refusal().unwrap();
                assert!(!refusal.is_a_bug(), "{refusal:?}");
                assert!(
                    refusal
                        .text()
                        .contains("an agent cannot answer a question that was put to a person"),
                    "{refusal:?}"
                );
            }
        });
        assert_eq!(
            network.names().called(&the_studio()).as_deref(),
            Some("the studio machine"),
            "an agent's request changed a name"
        );
        assert_eq!(looking.looked.get(), 0);
    }

    /// **A revoked pairing's name goes with it**: revoked on the person's door,
    /// the name is gone from the list, from `Naming` and from the file — and a
    /// pairing made again with the same machine comes back nameless.
    #[test]
    fn a_revoked_pairings_name_goes_with_it() {
        let at = a_directory_of_our_own("names-revoked").join("machine-names.toml");
        let network = reception_paired_with_the_studio(Box::new(TheNamesFile::at(&at)));
        the_person_says(
            &naming(the_studio().as_str(), "the studio machine"),
            &network,
            noon(),
        );
        assert!(
            std::fs::read_to_string(&at)
                .unwrap()
                .contains("the studio machine")
        );

        let revoked = the_person_says(
            &format!(
                r#"{{"revoke-pairing":{{"machine":"{}"}}}}"#,
                the_studio().as_str()
            ),
            &network,
            noon(),
        );
        assert_eq!(revoked.became_of_revoking(), Some(AfterRevoking::Revoked));
        assert_eq!(network.names().called(&the_studio()), None);
        assert!(
            !std::fs::read_to_string(&at)
                .unwrap()
                .contains("the studio machine"),
            "a revoked pairing's name stayed in the file"
        );

        let (again, _) = paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        network.locked().pairings_mut().keep(again);
        let listed = the_person_says(r#"{"pairings":{}}"#, &network, noon());
        assert_eq!(listed.paired().unwrap().first().unwrap().called(), None);
    }

    /// **A name outlives a restart, beside its pairing**: named through the
    /// door into a file of this test's own, read back by `alo-remembering` as
    /// `src/main.rs` reads it, and a machine started from nothing but that
    /// speaks of the studio by its name — on the list and through `Naming`.
    #[test]
    fn a_name_given_on_the_door_is_read_again_at_start() {
        let at = a_directory_of_our_own("names-restart").join("machine-names.toml");
        let network = reception_paired_with_the_studio(Box::new(TheNamesFile::at(&at)));
        the_person_says(
            &naming(the_studio().as_str(), "the studio machine"),
            &network,
            noon(),
        );
        let pairings: Pairings = network.locked().pairings().clone();
        drop(network);

        // The restart.
        let later = noon() + Duration::from_secs(60);
        let names = alo_remembering::machine_names_remembered(&at, &pairings, later).unwrap();
        let restarted = TheNetwork::remembering(
            reception(),
            pairings,
            Box::new(crate::network::NothingKeepsPairings),
        )
        .calling(TheNames::remembering(
            names,
            Box::new(TheNamesFile::at(&at)),
        ));
        assert_eq!(
            restarted.names().called(&the_studio()).as_deref(),
            Some("the studio machine")
        );
        let listed = the_person_says(r#"{"pairings":{}}"#, &restarted, later);
        assert_eq!(
            listed.paired().unwrap().first().unwrap().called(),
            Some("the studio machine")
        );
    }

    /// The requests read as a person's, for the dispatch's sake.
    #[test]
    fn both_requests_are_a_persons_and_about_a_name() {
        for line in [
            naming(the_studio().as_str(), "s"),
            clearing(the_studio().as_str()),
        ] {
            assert!(
                FromAPerson::read(&a_message(&line))
                    .unwrap()
                    .is_about_a_name()
            );
        }
    }
}
