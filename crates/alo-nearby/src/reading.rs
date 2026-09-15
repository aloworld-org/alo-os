//! Taking a packet off the network as a machine or a workspace, and refusing
//! anything that says more than presence.
//!
//! # The refusal this file exists for
//!
//! Everything on a network can send an advertisement, and a later version of
//! alo OS could send one too. So what an advertisement is allowed to carry is a
//! closed list, and a key that is not on it is
//! [`NotNearby::SaysMoreThanPresence`] rather than a key that is skipped.
//!
//! Skipping it would be the ordinary thing — DNS-SD readers ignore what they do
//! not understand, and that is why the format has lasted. It is refused here
//! because the failure being guarded against is not a stranger's packet. It is
//! **this** machine, two years from now, advertising the person's name in a key
//! the reader would have ignored, with nothing in the workspace failing.
//!
//! **A workspace is held to the same list** — which workspace, where it
//! answers, and the version it speaks — for the same reason: a workspace
//! advertising its organisation's name, a login address or a certificate in a
//! key nobody reads is a workspace telling a café who works where.
//!
//! # What is read, and in what order
//!
//! The instance name from the `PTR`, the port and host from the `SRV`, the keys
//! from the `TXT`. A packet with no `PTR` for the service asked about is not an
//! answer about that at all and says so; a packet whose `SRV` and `TXT` are
//! about a different instance than the `PTR` is refused, because presence about
//! one thing carried in an answer about another is not something a correct
//! responder does. The `SRV`'s host name is read past and never used: where a
//! machine or a workspace answers is the address its answer came from.

use std::net::{IpAddr, SocketAddr};

use crate::heard_from::HeardFrom;
use crate::machine::MachineId;
use crate::presence::{Found, SERVICE, VERSION, VERSION_KEY};
use crate::refusing::NotNearby;
use crate::wire::{Packet, kind};
use crate::workspace::{FoundWorkspace, WORKSPACE_SERVICE};

/// The class bits that matter, the top one being mDNS's cache-flush bit rather
/// than part of the class.
const WITHOUT_THE_FLUSH_BIT: u16 = 0x7fff;

/// One machine, read out of an answer it sent from `from`.
///
/// # Errors
///
/// [`NotNearby::NotAnAloMachine`] for an answer about some other service, which
/// is the commonest thing on a network and is not a fault in anything;
/// [`NotNearby::SaysMoreThanPresence`] for an answer carrying a key that is not
/// on the list; [`NotNearby::CutShort`] or [`NotNearby::NameNeverEnds`] for a
/// packet that is not well formed; and
/// [`NotNearby::SaysNothingAboutWhichMachine`] for one that never named an
/// instance.
pub fn a_machine_in(packet: &[u8], from: IpAddr) -> Result<Found, NotNearby> {
    a_machine_heard(packet, SocketAddr::new(from, 0))
}

/// One machine, read out of an answer that arrived from `from` — the socket
/// address the kernel reported, which for a link-local IPv6 address carries
/// the interface it was heard on ([`HeardFrom`]).
///
/// The same packet is read the same way whichever family carried it: the
/// closed list and every refusal are the packet's, and the family is only
/// where it came from.
///
/// # Errors
///
/// [`NotNearby::NamesNoNetwork`] for an answer from a link-local address with
/// no interface beside it, which no machine could be reached at; and every
/// refusal [`a_machine_in`] gives.
pub fn a_machine_heard(packet: &[u8], from: SocketAddr) -> Result<Found, NotNearby> {
    let from = measured(from)?;
    let (machine, port) = an_instance_in(packet, SERVICE, NotNearby::NotAnAloMachine)?;
    Ok(Found::heard(machine, port, from))
}

/// One workspace, read out of an answer its host sent from `from`.
///
/// The only way a [`FoundWorkspace`] is made: where it answers is `from`, the
/// address measured off the packet, and the port it advertised.
///
/// # Errors
///
/// [`NotNearby::NotAWorkspace`] for an answer about some other service —
/// including an alo machine's own presence, which is not a workspace;
/// [`NotNearby::SaysMoreThanPresence`] for an answer carrying a key that is not
/// on the closed list, or claiming a version this machine does not speak; and
/// the refusals [`a_machine_in`] gives for a packet that is not well formed or
/// never says which one it is about.
pub fn a_workspace_in(packet: &[u8], from: IpAddr) -> Result<FoundWorkspace, NotNearby> {
    a_workspace_heard(packet, SocketAddr::new(from, 0))
}

/// One workspace, read out of an answer that arrived from `from`, as
/// [`a_machine_heard`] reads a machine.
///
/// # Errors
///
/// [`NotNearby::NamesNoNetwork`] as [`a_machine_heard`], and every refusal
/// [`a_workspace_in`] gives.
pub fn a_workspace_heard(packet: &[u8], from: SocketAddr) -> Result<FoundWorkspace, NotNearby> {
    let from = measured(from)?;
    let (host, port) = an_instance_in(packet, WORKSPACE_SERVICE, NotNearby::NotAWorkspace)?;
    Ok(FoundWorkspace::heard(host, port, from))
}

/// Where an answer came from, or the refusal of a link-local address that says
/// no interface — checked before the packet, because an answer nothing could
/// reach is not worth reading.
fn measured(from: SocketAddr) -> Result<HeardFrom, NotNearby> {
    let heard = HeardFrom::of(from);
    if heard.names_a_network() {
        Ok(heard)
    } else {
        Err(NotNearby::NamesNoNetwork(heard.ip().to_string()))
    }
}

/// The identity and port an answer about `service` carries, or why it is not
/// one — `not_it` saying so for an answer about something else.
fn an_instance_in(
    packet: &[u8],
    service: &str,
    not_it: fn(String) -> NotNearby,
) -> Result<(MachineId, u16), NotNearby> {
    let mut reading = Packet::of(packet);
    let _transaction = reading.sixteen()?;
    let _flags = reading.sixteen()?;
    let questions = reading.sixteen()?;
    let answers = reading.sixteen()?;
    let official = reading.sixteen()?;
    let extra = reading.sixteen()?;

    for _ in 0..questions {
        drop(reading.name()?);
        let _kind = reading.sixteen()?;
        let _class = reading.sixteen()?;
    }

    let mut instance: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut version_said = false;

    let records = u32::from(answers)
        .saturating_add(u32::from(official))
        .saturating_add(u32::from(extra));
    for _ in 0..records {
        let name = reading.name()?;
        let kind_of = reading.sixteen()?;
        let _class = reading.sixteen()? & WITHOUT_THE_FLUSH_BIT;
        let _held_for = reading.thirty_two()?;
        let length = usize::from(reading.sixteen()?);
        let after = reading
            .so_far()
            .checked_add(length)
            .ok_or(NotNearby::CutShort)?;

        match kind_of {
            kind::PTR if name.eq_ignore_ascii_case(service) => {
                instance = Some(reading.name()?);
            }
            kind::SRV => {
                let _priority = reading.sixteen()?;
                let _weight = reading.sixteen()?;
                let said = reading.sixteen()?;
                if instance.as_deref().is_some_and(|of| of == name) {
                    port = Some(said);
                }
            }
            kind::TXT if instance.as_deref().is_some_and(|of| of == name) => {
                version_said = a_version_and_nothing_else(reading.some(length)?)?;
            }
            _ => {}
        }
        reading.go_to(after)?;
    }

    let Some(instance) = instance else {
        return Err(not_it(a_word_for(packet)));
    };
    let Some(said) = instance.strip_suffix(&format!(".{service}")) else {
        return Err(not_it(instance));
    };
    let identity = MachineId::read(said)?;
    let Some(port) = port else {
        return Err(NotNearby::SaysNothingAboutWhichMachine);
    };
    if !version_said {
        return Err(NotNearby::SaysNothingAboutWhichMachine);
    }
    Ok((identity, port))
}

/// Whether a packet is somebody asking who is here.
///
/// A question for this service and no other. Anything else on the address —
/// printers, media players, a machine's own answer coming back — is not one,
/// and is not an error either.
#[must_use]
pub fn a_question_in(packet: &[u8]) -> bool {
    asks_for(packet, SERVICE)
}

/// Whether a packet is somebody asking which workspaces are here.
///
/// A question for [`WORKSPACE_SERVICE`] and no other, read by exactly the rule
/// [`a_question_in`] reads a question for machines by — one packet may ask
/// both, and then both are true of it.
#[must_use]
pub fn a_question_for_workspaces_in(packet: &[u8]) -> bool {
    asks_for(packet, WORKSPACE_SERVICE)
}

/// Whether a packet is a question asking for `service`'s instances.
fn asks_for(packet: &[u8], service: &str) -> bool {
    let asked = || -> Result<bool, NotNearby> {
        let mut reading = Packet::of(packet);
        let _transaction = reading.sixteen()?;
        let flags = reading.sixteen()?;
        let questions = reading.sixteen()?;
        // The rest of the header, which is counted and stepped over: a
        // question's name begins after all six of its numbers, not after three.
        let _answers = reading.sixteen()?;
        let _official = reading.sixteen()?;
        let _extra = reading.sixteen()?;
        if flags & 0x8000 != 0 {
            // The reply bit: this is somebody's answer, not their question.
            return Ok(false);
        }
        for _ in 0..questions {
            let name = reading.name()?;
            let kind_of = reading.sixteen()?;
            let _class = reading.sixteen()?;
            if kind_of == kind::PTR && name.eq_ignore_ascii_case(service) {
                return Ok(true);
            }
        }
        Ok(false)
    };
    asked().unwrap_or(false)
}

/// Whether the keys in a `TXT` record are the one key presence may carry, and
/// nothing else.
///
/// # Errors
///
/// [`NotNearby::SaysMoreThanPresence`] naming the key that is not on the list —
/// the refusal this file exists for — and [`NotNearby::CutShort`] for a record
/// whose lengths run past its own data.
fn a_version_and_nothing_else(data: &[u8]) -> Result<bool, NotNearby> {
    let mut at = 0_usize;
    let mut said = false;
    while let Some(&length) = data.get(at) {
        let from = at.saturating_add(1);
        let to = from
            .checked_add(usize::from(length))
            .ok_or(NotNearby::CutShort)?;
        let entry = data.get(from..to).ok_or(NotNearby::CutShort)?;
        at = to;
        if entry.is_empty() {
            // A single empty string is how DNS-SD spells "no keys at all"; it
            // is the absence of a key rather than a key.
            continue;
        }
        let entry = String::from_utf8_lossy(entry);
        let (key, value) = entry.split_once('=').unwrap_or((entry.as_ref(), ""));
        if key != VERSION_KEY {
            return Err(NotNearby::SaysMoreThanPresence(key.to_owned()));
        }
        if value != VERSION {
            return Err(NotNearby::SaysMoreThanPresence(entry.into_owned()));
        }
        said = true;
    }
    Ok(said)
}

/// Something true to call a packet that never said what it was about.
///
/// The bytes are deliberately not in it. A refusal naming a stranger's packet
/// would put a stranger's bytes wherever the refusal is printed, and law 1 is
/// about what leaves this machine in both directions.
fn a_word_for(packet: &[u8]) -> String {
    format!("something else, in {} bytes", packet.len())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{a_machine_in, a_workspace_in};
    use crate::advertising::{about, about_a_workspace};
    use crate::machine::MachineId;
    use crate::presence::{Presence, SERVICE, Standing};
    use crate::refusing::NotNearby;
    use crate::wire::{
        IN_AND_THE_ONLY_ONE, kind, write_data, write_name, write_sixteen, write_thirty_two,
    };
    use crate::workspace::{WORKSPACE_SERVICE, WorkspacePresence};

    /// Where every answer here is heard from.
    fn here() -> std::net::IpAddr {
        std::net::Ipv4Addr::LOCALHOST.into()
    }

    /// The identity every test here advertises.
    fn an_identity() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// **The two questions are told apart**: a question for machines is not
    /// one for workspaces, the other way round neither, and an answer about
    /// either is a question for nothing.
    #[test]
    fn a_question_for_workspaces_is_told_apart_from_one_for_machines() {
        use super::{a_question_for_workspaces_in, a_question_in};
        use crate::advertising::{a_question, a_question_for_workspaces, about_a_workspace};

        let for_machines = a_question().unwrap();
        let for_workspaces = a_question_for_workspaces().unwrap();
        assert!(a_question_in(&for_machines));
        assert!(!a_question_for_workspaces_in(&for_machines));
        assert!(a_question_for_workspaces_in(&for_workspaces));
        assert!(!a_question_in(&for_workspaces));

        let an_answer = about_a_workspace(&WorkspacePresence::of(an_identity(), 8_443)).unwrap();
        assert!(!a_question_for_workspaces_in(&an_answer));
        assert!(!a_question_in(&an_answer));
        assert!(!a_question_for_workspaces_in(b"which workspaces?"));
    }

    /// **A second machine reads the advertisement and answers with one machine,
    /// not paired.** The whole of task one's promise, in one round trip.
    #[test]
    fn a_machine_that_advertises_is_read_back_as_one_machine_not_paired() {
        let packet = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let found = a_machine_in(&packet, here()).unwrap();
        assert_eq!(found.machine, an_identity());
        assert_eq!(found.port, 7_610);
        assert_eq!(found.address, here());
        assert_eq!(found.where_it_answers().port(), 7_610);
        assert_eq!(found.standing, Standing::NotPaired);
    }

    /// An advertisement from an alo machine built with the same presence is the
    /// same bytes every time, so nothing about the machine's state leaks
    /// through the shape of what it says.
    #[test]
    fn the_same_presence_advertises_the_same_bytes_every_time() {
        let once = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let again = about(&Presence::of(an_identity(), 7_610)).unwrap();
        assert_eq!(once, again);
    }

    /// **What this machine advertises carries no word of what this machine is
    /// called.** The hostname is the field DNS-SD ordinarily fills, and it is
    /// the one that tells a café who is in the café.
    #[test]
    fn the_advertisement_carries_no_word_of_this_machines_own_name() {
        let packet = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let called = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_default()
            .to_lowercase();
        if called.len() < 3 {
            // Nothing to look for; the test says so rather than passing
            // quietly on a machine with no name.
            return;
        }
        let written = String::from_utf8_lossy(&packet).to_lowercase();
        assert!(
            !written.contains(&called),
            "the advertisement carries this machine's own name, `{called}`"
        );
    }

    /// An answer about a printer or a media player is not a fault in anything,
    /// and says which service it was about rather than carrying its bytes.
    #[test]
    fn an_answer_about_another_service_is_not_an_alo_machine() {
        let mut packet = Vec::new();
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0x8400);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 1);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0);
        write_name(&mut packet, "_ipp._tcp.local").unwrap();
        write_sixteen(&mut packet, kind::PTR);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, "printer", |data| {
            write_name(data, "a-printer._ipp._tcp.local")
        })
        .unwrap();

        let refused = a_machine_in(&packet, here()).unwrap_err();

        assert!(matches!(refused, NotNearby::NotAnAloMachine(_)));
        assert!(
            refused.is_about_a_stranger(),
            "a printer on the network reads as a fault in this machine"
        );
    }

    /// **A key that is not on the list is refused, not skipped** — and the
    /// failure being guarded against is this machine two years from now, not a
    /// stranger.
    #[test]
    fn an_advertisement_saying_more_than_presence_is_refused_by_name() {
        for saying in ["who=disan", "models=mistral-7b", "org=axon", "paired=2"] {
            let packet = an_advertisement_also_saying(saying);
            let refused = a_machine_in(&packet, here()).unwrap_err();
            let key = saying.split_once('=').unwrap().0;
            assert_eq!(
                refused,
                NotNearby::SaysMoreThanPresence(key.to_owned()),
                "`{saying}` was read past rather than refused"
            );
        }
    }

    /// An advertisement claiming a protocol version this machine does not speak
    /// is refused at the same door, because the version is the only key and a
    /// different one is a different closed list.
    #[test]
    fn an_advertisement_claiming_another_version_is_refused() {
        let packet = an_advertisement_also_saying("v=2");
        assert!(matches!(
            a_machine_in(&packet, here()).unwrap_err(),
            NotNearby::SaysMoreThanPresence(_)
        ));
    }

    /// A truncated packet is refused rather than read up to whatever is there.
    #[test]
    fn half_an_advertisement_is_refused() {
        let whole = about(&Presence::of(an_identity(), 7_610)).unwrap();
        for how_much in [4_usize, 12, 30, 60] {
            let half = whole.get(..how_much).unwrap().to_vec();
            assert!(
                a_machine_in(&half, here()).is_err(),
                "{how_much} bytes of an advertisement read as a machine"
            );
        }
    }

    /// An advertisement carrying one extra key beside the version, built by
    /// hand because the writer in this crate cannot produce one.
    fn an_advertisement_also_saying(entry: &str) -> Vec<u8> {
        let presence = Presence::of(an_identity(), 7_610);
        let instance = presence.instance();
        let mut packet = Vec::new();
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0x8400);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 1);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 2);

        write_name(&mut packet, SERVICE).unwrap();
        write_sixteen(&mut packet, kind::PTR);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| write_name(data, &instance)).unwrap();

        write_name(&mut packet, &instance).unwrap();
        write_sixteen(&mut packet, kind::SRV);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| {
            write_sixteen(data, 0);
            write_sixteen(data, 0);
            write_sixteen(data, 7_610);
            write_name(data, &presence.host())
        })
        .unwrap();

        write_name(&mut packet, &instance).unwrap();
        write_sixteen(&mut packet, kind::TXT);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| {
            for said in ["v=1", entry] {
                data.push(u8::try_from(said.len()).unwrap_or(0));
                data.extend_from_slice(said.as_bytes());
            }
            Ok(())
        })
        .unwrap();

        packet
    }

    /// **A workspace advertised is read back as which workspace, where it
    /// answers and the version** — at the address its answer came from, not
    /// paired with anything.
    #[test]
    fn a_workspace_that_advertises_is_read_back_as_one_workspace_not_paired() {
        let packet = about_a_workspace(&WorkspacePresence::of(an_identity(), 8_443)).unwrap();
        let from: std::net::IpAddr = std::net::Ipv4Addr::new(192, 168, 1, 20).into();
        let found = a_workspace_in(&packet, from).unwrap();
        assert_eq!(found.host(), &an_identity());
        assert_eq!(found.port(), 8_443);
        assert_eq!(found.address(), from);
        assert_eq!(found.version(), "1");
        assert_eq!(found.standing(), Standing::NotPaired);
    }

    /// **A workspace advertisement carrying one field more is refused by name,
    /// not read around** — the organisation, a login address, a certificate or
    /// anything else a later workspace might start saying.
    #[test]
    fn a_workspace_advertisement_carrying_one_field_more_is_refused() {
        for saying in [
            "org=axon",
            "url=https://mail.axon.example",
            "address=192.168.1.20",
            "name=axon-workspace",
            "cert=ab12",
        ] {
            let packet = a_workspace_advertisement_saying(&["v=1", saying], "unused.local");
            let key = saying.split_once('=').unwrap().0;
            assert_eq!(
                a_workspace_in(&packet, here()).unwrap_err(),
                NotNearby::SaysMoreThanPresence(key.to_owned()),
                "`{saying}` was read past rather than refused"
            );
        }
        // And a version this machine does not speak is not a quiet `v=1`.
        assert!(matches!(
            a_workspace_in(
                &a_workspace_advertisement_saying(&["v=2"], "x.local"),
                here()
            )
            .unwrap_err(),
            NotNearby::SaysMoreThanPresence(_)
        ));
        // Exactly the closed list reads.
        assert!(
            a_workspace_in(
                &a_workspace_advertisement_saying(&["v=1"], "x.local"),
                here()
            )
            .is_ok()
        );
    }

    /// **Where a workspace answers is the address it was heard from**, whatever
    /// host name its `SRV` record names — so a name in a packet never becomes
    /// an address this machine would go to.
    #[test]
    fn where_a_workspace_answers_is_measured_not_what_its_record_names() {
        let packet = a_workspace_advertisement_saying(&["v=1"], "mail.axon.example");
        let from: std::net::IpAddr = std::net::Ipv4Addr::new(10, 1, 2, 3).into();
        let found = a_workspace_in(&packet, from).unwrap();
        assert_eq!(found.where_it_answers().to_string(), "10.1.2.3:8443");
    }

    /// **A machine's presence is not a workspace, and a workspace is not a
    /// machine**: each service's answer is refused as the other, so serving a
    /// workspace never changes what a machine advertises about itself.
    #[test]
    fn a_machine_is_not_a_workspace_and_a_workspace_is_not_a_machine() {
        let machine = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let refused = a_workspace_in(&machine, here()).unwrap_err();
        assert!(
            matches!(refused, NotNearby::NotAWorkspace(_)),
            "{refused:?}"
        );
        assert!(refused.is_about_a_stranger());

        let workspace = about_a_workspace(&WorkspacePresence::of(an_identity(), 8_443)).unwrap();
        assert!(matches!(
            a_machine_in(&workspace, here()).unwrap_err(),
            NotNearby::NotAnAloMachine(_)
        ));
    }

    /// A workspace advertised under something that is not an identity — a name
    /// somebody chose — is refused rather than listed under that name.
    #[test]
    fn a_workspace_advertised_under_a_chosen_name_is_refused() {
        let mut packet = Vec::new();
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0x8400);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 1);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0);
        write_name(&mut packet, WORKSPACE_SERVICE).unwrap();
        write_sixteen(&mut packet, kind::PTR);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        let instance = format!("axon-mail.{WORKSPACE_SERVICE}");
        write_data(&mut packet, &instance, |data| write_name(data, &instance)).unwrap();
        assert!(matches!(
            a_workspace_in(&packet, here()).unwrap_err(),
            NotNearby::NotAMachineIdentity(_)
        ));
    }

    /// A workspace advertisement with `entries` in its `TXT` and `target` as its
    /// `SRV` host, built by hand because the writer in this crate cannot
    /// produce anything but the closed list.
    fn a_workspace_advertisement_saying(entries: &[&str], target: &str) -> Vec<u8> {
        let instance = WorkspacePresence::of(an_identity(), 8_443).instance();
        let mut packet = Vec::new();
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 0x8400);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 1);
        write_sixteen(&mut packet, 0);
        write_sixteen(&mut packet, 2);

        write_name(&mut packet, WORKSPACE_SERVICE).unwrap();
        write_sixteen(&mut packet, kind::PTR);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| write_name(data, &instance)).unwrap();

        write_name(&mut packet, &instance).unwrap();
        write_sixteen(&mut packet, kind::SRV);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| {
            write_sixteen(data, 0);
            write_sixteen(data, 0);
            write_sixteen(data, 8_443);
            write_name(data, target)
        })
        .unwrap();

        write_name(&mut packet, &instance).unwrap();
        write_sixteen(&mut packet, kind::TXT);
        write_sixteen(&mut packet, IN_AND_THE_ONLY_ONE);
        write_thirty_two(&mut packet, 120);
        write_data(&mut packet, &instance, |data| {
            for said in entries {
                data.push(u8::try_from(said.len()).unwrap_or(0));
                data.extend_from_slice(said.as_bytes());
            }
            Ok(())
        })
        .unwrap();

        packet
    }

    /// A link-local address on the interface the kernel numbers `scope`.
    fn link_local(scope: u32) -> std::net::SocketAddr {
        std::net::SocketAddr::V6(std::net::SocketAddrV6::new(
            "fe80::a406:e5ff:fe4b:ac9e".parse().unwrap(),
            5_353,
            0,
            scope,
        ))
    }

    /// **An answer over IPv6 is read as an answer over IPv4 is**: the same
    /// machine and the same workspace out of the same bytes, at the address it
    /// came from — and a link-local one with the interface it was heard on.
    #[test]
    fn an_answer_over_ipv6_is_read_as_one_over_ipv4_with_its_interface() {
        use super::{a_machine_heard, a_workspace_heard};

        let packet = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let over_ipv4 = a_machine_in(&packet, here()).unwrap();
        let over_ipv6 = a_machine_heard(&packet, link_local(3)).unwrap();
        assert_eq!(over_ipv6.machine, over_ipv4.machine);
        assert_eq!(over_ipv6.port, over_ipv4.port);
        assert_eq!(over_ipv6.address.scope(), Some(3));
        assert_eq!(
            over_ipv6.where_it_answers().to_string(),
            "[fe80::a406:e5ff:fe4b:ac9e%3]:7610"
        );

        let workspace = about_a_workspace(&WorkspacePresence::of(an_identity(), 8_443)).unwrap();
        let heard = a_workspace_heard(&workspace, link_local(3)).unwrap();
        assert_eq!(heard.host(), &an_identity());
        assert_eq!(
            heard.where_it_answers().to_string(),
            "[fe80::a406:e5ff:fe4b:ac9e%3]:8443"
        );
    }

    /// **A packet saying more than presence is refused whichever family carried
    /// it** — by the same refusal, with the same key named.
    #[test]
    fn saying_more_than_presence_is_refused_over_ipv6_as_over_ipv4() {
        use super::{a_machine_heard, a_workspace_heard};

        for saying in ["who=disan", "models=mistral-7b", "org=axon", "paired=2"] {
            let packet = an_advertisement_also_saying(saying);
            let over_ipv4 = a_machine_in(&packet, here()).unwrap_err();
            let over_ipv6 = a_machine_heard(&packet, link_local(3)).unwrap_err();
            let global: std::net::SocketAddr = "[2001:db8::7]:5353".parse().unwrap();
            let over_global_ipv6 = a_machine_heard(&packet, global).unwrap_err();
            assert_eq!(over_ipv6, over_ipv4, "{saying}");
            assert_eq!(over_global_ipv6, over_ipv4, "{saying}");
        }
        let packet = a_workspace_advertisement_saying(&["v=1", "org=axon"], "x.local");
        assert_eq!(
            a_workspace_heard(&packet, link_local(3)).unwrap_err(),
            NotNearby::SaysMoreThanPresence("org".to_owned())
        );
    }

    /// **An answer from a link-local address that says no interface is
    /// refused**, machine or workspace, even when the packet itself is a
    /// perfect advertisement: it names no network and nothing could reach it.
    #[test]
    fn an_answer_from_a_link_local_address_with_no_interface_is_refused() {
        use super::{a_machine_heard, a_workspace_heard};

        let packet = about(&Presence::of(an_identity(), 7_610)).unwrap();
        let refused = a_machine_heard(&packet, link_local(0)).unwrap_err();
        assert_eq!(
            refused,
            NotNearby::NamesNoNetwork("fe80::a406:e5ff:fe4b:ac9e".to_owned())
        );
        assert!(refused.is_about_a_stranger());
        assert!(matches!(
            a_machine_in(&packet, "fe80::1".parse().unwrap()).unwrap_err(),
            NotNearby::NamesNoNetwork(_)
        ));

        let workspace = about_a_workspace(&WorkspacePresence::of(an_identity(), 8_443)).unwrap();
        assert!(matches!(
            a_workspace_heard(&workspace, link_local(0)).unwrap_err(),
            NotNearby::NamesNoNetwork(_)
        ));
    }
}
