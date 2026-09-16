//! The person's door says what this machine tells the local network about
//! itself, and changes none of it.
//!
//! An alo machine says two things to everything on the link: that it exists,
//! under its identity and the wire's port, and — where root installed a
//! workspace server — that it hosts a workspace at a port. *Nothing leaves
//! silently* is law 1's reason, and it is a person's to check; this is the
//! request they check it with — `alo_protocol::FromAPerson::Advertised`,
//! answered here — so that seeing what their machine tells the office does not
//! take a packet capture, and a workspace file that was refused is a sentence
//! in their language rather than a line in a service log they do not read.
//!
//! # What the running service holds, never the file read again
//!
//! [`Advertising`] is taken off [`crate::wire::Wire`] — the identity and port
//! its responder answers with, the workspace port it was handed at start, and
//! the group a refused file was put in ([`crate::unhosted`]) — and nothing in
//! this file opens anything. So the answer describes **the service that is
//! running**: a file changed since the start is not what the network is being
//! told, and saying so would be a second, wrong account of it.
//!
//! # It shows presence; it does not change it
//!
//! No record is written, because nothing happened on the machine; nothing is
//! advertised that was not; no file is read. There is no request beside this
//! one that sets what is advertised — no *advertise as*, no *discovery off*,
//! no setting (ADR 0003) — and this one carries no field for a port or a path.
//! An agent asking is refused by `alo-protocol` before anything reaches this
//! file, in the words an agent approving something gets.

use alo_nearby::MachineId;
use alo_protocol::{Advertised, HostedWorkspace, ToAPerson};
use alo_strings::Strings;

use crate::hosting::Hosted;

/// What this machine advertises on the local network, as the running service
/// holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advertising {
    /// The identity its presence is advertised under.
    machine: MachineId,
    /// The port its presence names.
    port: u16,
    /// The workspace it answers for, that it answers for none, or why one
    /// installed is not advertised.
    workspace: Hosted,
}

impl Advertising {
    /// What a machine advertises: its identity, the port its presence names,
    /// and what its start read about a workspace.
    #[must_use]
    pub const fn of(machine: MachineId, port: u16, workspace: Hosted) -> Self {
        Self {
            machine,
            port,
            workspace,
        }
    }

    /// The identity its presence is advertised under.
    #[must_use]
    pub const fn machine(&self) -> &MachineId {
        &self.machine
    }

    /// The port its presence names.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// What its start read about a workspace.
    #[must_use]
    pub const fn workspace(&self) -> Hosted {
        self.workspace
    }
}

/// The answer to `advertised`: what this machine says about itself, with why a
/// workspace installed is not advertised in the person's language.
#[must_use]
pub fn told(advertising: &Advertising, strings: &Strings) -> ToAPerson {
    let workspace = match advertising.workspace {
        Hosted::At(port) => HostedWorkspace::hosts(port),
        Hosted::Nothing => HostedWorkspace::none(),
        Hosted::Refused(why) => HostedWorkspace::not_advertised(&why.said(strings)),
    };
    ToAPerson::advertised(Advertised::of(
        advertising.machine.as_str(),
        advertising.port,
        workspace,
    ))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::Cell;
    use std::fs::Permissions;
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, UdpSocket};
    use std::num::NonZeroU16;
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use alo_capability::Grants;
    use alo_nearby::advertising::a_question_for_workspaces;
    use alo_record::Record;

    use super::*;
    use crate::answering::what_a_person_said;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::holding::Holding;
    use crate::hosting::{THE_HOSTED_WORKSPACE, advertised};
    use crate::network::TheNetwork;
    use crate::pairing::Nearby;
    use crate::rereading::WhatIsGranted;
    use crate::testing::{
        NothingIsRemembered, TheStudioIsAt, a_directory_of_our_own, a_message, hour, in_english,
        noon, nothing_has_been_chosen, on_a_machine_that_answers, on_a_machine_with_no_turn,
        reception,
    };
    use crate::unhosted::Unhosted;
    use crate::wire::Wire;

    /// What the person's shell sends to ask.
    const ASKING: &str = r#"{"advertised":{}}"#;

    /// The login a file is handed to so that it is not root's.
    const AN_AGENT: u32 = 989;

    /// A workspace file `text` would be, at `mode`, in a folder of this test's
    /// own — root's, because the loop runs these tests as root.
    fn the_workspace_file(what: &str, text: &str, mode: u32) -> PathBuf {
        let at = a_directory_of_our_own(what).join("workspace.toml");
        std::fs::write(&at, text).unwrap();
        std::fs::set_permissions(&at, Permissions::from_mode(mode)).unwrap();
        assert_eq!(
            std::fs::metadata(&at).unwrap().uid(),
            0,
            "these tests hold root's file and are run as root, as the loop runs them"
        );
        at
    }

    /// Reception, as `src/main.rs` starts it: bound on this host, told what the
    /// file at `file` says — and where its discovery is answered.
    fn reception_hosting(file: &Path) -> (Wire, SocketAddr) {
        let discovery = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let at = discovery.local_addr().unwrap();
        let wire = Wire::on(
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap(),
            discovery,
            reception(),
            0,
        )
        .unwrap()
        .hosting(advertised(file, |_| {}));
        (wire, at)
    }

    /// What `wire` answers a question for workspaces with, byte for byte —
    /// nothing, when it steps over the question.
    fn what_it_answers_workspaces_with(wire: &Wire, at: SocketAddr) -> Option<Vec<u8>> {
        let asking = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        asking
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        asking
            .send_to(&a_question_for_workspaces().unwrap(), at)
            .unwrap();
        wire.answer_discovery().unwrap();
        let mut heard = [0_u8; 1_500];
        asking
            .recv_from(&mut heard)
            .ok()
            .and_then(|(length, _)| heard.get(..length).map(<[u8]>::to_vec))
    }

    /// The person sends `line` on reception's door, and what was written down
    /// while they did comes back beside the answer.
    fn the_person_says(line: &str, wire: &Wire) -> (ToAPerson, Record) {
        let network = TheNetwork::on(reception());
        let advertising = wire.advertising();
        let mut record = Record::default();
        let said = on_a_machine_with_no_turn(
            "what-is-advertised",
            &mut record,
            |machine, _, strings, _, _| {
                let mut grants = Grants::default();
                what_a_person_said(
                    &a_message(line),
                    &mut Holding::Nobody(machine),
                    &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                    &Nearby {
                        network: &network,
                        looking: wire,
                        advertising: &advertising,
                    },
                    strings,
                    noon(),
                )
                .unwrap()
            },
        );
        (said, record)
    }

    /// **The person is told the identity, the port its presence names and the
    /// workspace it hosts — and the answer has no field for anything else.**
    /// Root's file saying a port is that port; no file is a machine hosting
    /// none; and the line on the wire carries exactly three keys.
    #[test]
    fn the_person_is_told_the_identity_the_port_and_the_workspace_and_nothing_else() {
        let file = the_workspace_file("advertised-hosting", "port = 8443\n", 0o644);
        let (wire, _) = reception_hosting(&file);
        let (said, record) = the_person_says(ASKING, &wire);
        let answer = said.advertisement().unwrap();
        assert_eq!(answer.machine(), reception().as_str());
        assert_eq!(answer.port(), wire.port());
        assert_eq!(answer.workspace().port(), NonZeroU16::new(8_443));
        assert!(record.is_empty());

        let written: serde_json::Value = serde_json::from_str(&said.written().unwrap()).unwrap();
        let advertised = written
            .get("tells")
            .and_then(|tells| tells.get("advertised"))
            .and_then(serde_json::Value::as_object)
            .unwrap();
        let mut keys: Vec<&str> = advertised.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["machine", "port", "workspace"]);
        assert_eq!(
            advertised.get("workspace").unwrap(),
            &serde_json::json!({"hosts": {"port": 8443}})
        );

        let nowhere = a_directory_of_our_own("advertised-nothing").join("workspace.toml");
        let (wire, _) = reception_hosting(&nowhere);
        let (said, _) = the_person_says(ASKING, &wire);
        let answer = said.advertisement().unwrap();
        assert_eq!(answer.workspace(), &HostedWorkspace::none());
        assert_eq!(answer.port(), wire.port());
    }

    /// **A refused workspace file is told to the person in their language,
    /// naming no path, owner or mode** — for real files refused each way a
    /// file on this host can be, in the group a person can act on; and the
    /// machine advertises no workspace for any of them.
    #[test]
    fn a_refused_workspace_file_is_told_in_the_persons_words_naming_no_path_owner_or_mode() {
        let strings = in_english();
        let not_roots = the_workspace_file("advertised-not-roots", "port = 8443\n", 0o644);
        std::os::unix::fs::chown(&not_roots, Some(AN_AGENT), Some(AN_AGENT)).unwrap();
        let real = the_workspace_file("advertised-linked", "port = 8443\n", 0o644);
        let link = real.with_file_name("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let cases = [
            (not_roots, Unhosted::NotTheSystems),
            (
                the_workspace_file("advertised-writable", "port = 8443\n", 0o666),
                Unhosted::NotTheSystems,
            ),
            (link, Unhosted::NotTheSystems),
            (
                a_directory_of_our_own("advertised-a-folder"),
                Unhosted::NotTheSystems,
            ),
            (
                the_workspace_file("advertised-a-name", "port = 8443\nname = \"Axon\"\n", 0o644),
                Unhosted::NoUsablePort,
            ),
            (
                the_workspace_file("advertised-no-port", "port = 65536\n", 0o644),
                Unhosted::NoUsablePort,
            ),
            (
                the_workspace_file("advertised-the-wire", "port = 7610\n", 0o644),
                Unhosted::NoUsablePort,
            ),
            (
                the_workspace_file("advertised-not-toml", "port: 8443\n", 0o644),
                Unhosted::NoUsablePort,
            ),
        ];
        for (file, group) in cases {
            let (wire, at) = reception_hosting(&file);
            assert_eq!(wire.hosts(), None, "{}", file.display());
            assert_eq!(
                what_it_answers_workspaces_with(&wire, at),
                None,
                "a refused file advertised a workspace: {}",
                file.display()
            );

            let (said, record) = the_person_says(ASKING, &wire);
            let answer = said.advertisement().unwrap();
            assert_eq!(answer.workspace().port(), None);
            let why = answer.workspace().why_not().unwrap();
            assert_eq!(
                why.text(),
                group.said(&strings).text(),
                "{}",
                file.display()
            );
            assert!(record.is_empty());

            // The machine's own port is advertised on purpose, and this machine
            // picks it: on 2026-09-16 it came up as 42989, which contains the
            // mode below, and the run failed for a coincidence rather than for a
            // leak — twice, so the loop read it as the work. It is taken out
            // before the line is searched, because what this looks for is the
            // refused file's path, owner and mode, and the port is none of them.
            let line = said
                .written()
                .unwrap()
                .replace(&format!("\"port\":{}", wire.port()), "\"port\":0");
            let folder = file.parent().unwrap().to_string_lossy().into_owned();
            for leaked in [
                folder.as_str(),
                THE_HOSTED_WORKSPACE,
                "workspace.toml",
                "uid",
                "989",
                "666",
                "mode",
            ] {
                assert!(!line.contains(leaked), "{leaked:?} in {line}");
            }
        }
    }

    /// **Asking changes nothing**: the file is not read again — changed and
    /// then removed after the start, the answer is still what the start read —
    /// nothing is advertised that was not, byte for byte, and nothing is
    /// written to the record. A refused file mended after the start is still
    /// told as refused, because the service running still advertises nothing.
    #[test]
    fn asking_reads_no_file_again_advertises_nothing_new_and_writes_nothing() {
        let file = the_workspace_file("advertised-unchanged", "port = 8443\n", 0o644);
        let (wire, at) = reception_hosting(&file);
        let before = wire.advertising();
        let answered_before = what_it_answers_workspaces_with(&wire, at).unwrap();

        std::fs::write(&file, "port = 9443\n").unwrap();
        let (said, record) = the_person_says(ASKING, &wire);
        assert_eq!(
            said.advertisement().unwrap().workspace().port(),
            NonZeroU16::new(8_443)
        );
        assert!(record.is_empty(), "asking wrote the record");
        std::fs::remove_file(&file).unwrap();
        let (said, record) = the_person_says(ASKING, &wire);
        assert_eq!(
            said.advertisement().unwrap().workspace().port(),
            NonZeroU16::new(8_443)
        );
        assert!(record.is_empty(), "asking wrote the record");

        assert_eq!(wire.advertising(), before);
        assert_eq!(
            what_it_answers_workspaces_with(&wire, at).unwrap(),
            answered_before,
            "asking changed what is advertised"
        );

        let loose = the_workspace_file("advertised-mended", "port = 8443\n", 0o666);
        let (wire, at) = reception_hosting(&loose);
        std::fs::set_permissions(&loose, Permissions::from_mode(0o644)).unwrap();
        let (said, record) = the_person_says(ASKING, &wire);
        let answer = said.advertisement().unwrap();
        assert_eq!(answer.workspace().port(), None);
        assert!(answer.workspace().why_not().is_some());
        assert!(record.is_empty());
        assert_eq!(wire.hosts(), None);
        assert_eq!(what_it_answers_workspaces_with(&wire, at), None);
    }

    /// **A request carrying a port or a path is not a request**, on the
    /// daemon's door as on the protocol's: each is refused, nothing is written
    /// down, and what is advertised is what it was.
    #[test]
    fn a_request_carrying_a_port_or_a_path_is_refused_and_changes_nothing() {
        let file = the_workspace_file("advertised-carrying", "port = 8443\n", 0o644);
        let (wire, at) = reception_hosting(&file);
        let before = wire.advertising();
        let answered_before = what_it_answers_workspaces_with(&wire, at).unwrap();
        for line in [
            r#"{"advertised":{"port":9443}}"#,
            r#"{"advertised":{"path":"/etc/alo/workspace.toml"}}"#,
            r#"{"advertised":{"workspace":{"hosts":{"port":9443}}}}"#,
            r#"{"advertise-as":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}}"#,
            r#"{"discovery":{"off":true}}"#,
        ] {
            let (said, record) = the_person_says(line, &wire);
            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{line}: {refusal:?}");
            assert!(said.advertisement().is_none(), "{line}");
            assert!(record.is_empty(), "{line}");
        }
        assert_eq!(wire.advertising(), before);
        assert_eq!(
            what_it_answers_workspaces_with(&wire, at).unwrap(),
            answered_before
        );
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "port = 8443\n");
    }

    /// **An agent asking what this machine advertises is refused in the words
    /// an agent approving something gets**, and the network is not asked.
    #[test]
    fn an_agent_asking_what_this_machine_advertises_is_refused_as_an_approval_would_be() {
        let network = TheNetwork::on(reception());
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
            let said = what_an_agent_said(
                &a_message(ASKING),
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
            assert!(!refusal.text().contains("7610"), "{refusal:?}");
        });
        assert_eq!(looking.looked.get(), 0);
    }
}
