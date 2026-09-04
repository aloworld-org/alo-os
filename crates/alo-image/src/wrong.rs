//! What the image's files say that they cannot all be saying at once.
//!
//! `crate::refusing` is the other half: those are files that would not read at
//! all, and every one of these is a file that reads perfectly and disagrees with
//! another one. That is the failure worth catching here, because it is the one a
//! build cannot see — a Containerfile will happily produce an image whose
//! machine description names a login the image never creates, and the machine
//! boots to a daemon that stops with a sentence about a number.
//!
//! Each variant names one promise made somewhere in `docs/`, and the sentence
//! says which. They are English for `crate::refusing`'s reason: nobody using alo
//! OS ever reads one.

use std::path::PathBuf;

use thiserror::Error;

/// One thing the image's files disagree about.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Wrong {
    /// The agent's service would start on a machine with no boundary.
    #[error(
        "{agent} does not wait for {loader} — ADR 0015 says a turn that cannot be bounded does \
         not run, so a machine whose boundary never loaded must not reach a service that would \
         refuse every turn on it"
    )]
    TheAgentDoesNotWaitForTheBoundary {
        /// The agent's unit.
        agent: String,
        /// The loader's unit.
        loader: String,
    },
    /// The loader's unit does not stay active once the process has gone.
    #[error(
        "{loader} is `Type={kind}`, and the boundary outlives the process that loaded it \
         (ADR 0018) — a unit that went inactive would tell `systemctl status` that this machine \
         has no boundary while it has one"
    )]
    TheLoaderDoesNotStay {
        /// The loader's unit.
        loader: String,
        /// What it says it is, or `-` where it says nothing.
        kind: String,
    },
    /// The loader would run as somebody who cannot load a BPF LSM programme.
    #[error(
        "{loader} runs as `{as_login}`, and only root can load a BPF LSM programme — the loader \
         refuses to start rather than fail inside the verifier"
    )]
    TheLoaderIsNotRoot {
        /// The loader's unit.
        loader: String,
        /// The login it names, or `-` where it names none.
        as_login: String,
    },
    /// The loader would pin the map of turns where the agent cannot write it.
    #[error(
        "{loader} runs in group `{group}`, which is not the group the machine description gives \
         the agent — the map of turns is handed to whatever group this process is in, so this is \
         a boundary loaded for a daemon that can never bound a turn"
    )]
    TheLoaderIsInTheWrongGroup {
        /// The loader's unit.
        loader: String,
        /// The group it names, or `-` where it names none.
        group: String,
    },
    /// The loader holds something ADR 0018 did not give it.
    #[error(
        "{loader} is bounded to {held:?}, and ADR 0018 gives it CAP_BPF and CAP_SYS_ADMIN — the \
         one privileged component alo OS has is acceptable because of how little it is trusted \
         with, and this line is where that is true or is prose"
    )]
    TheLoaderHoldsSomethingElse {
        /// The loader's unit.
        loader: String,
        /// What the unit bounds it to.
        held: Vec<String>,
    },
    /// The agent's service holds a capability, or does not say it holds none.
    #[error(
        "{agent} does not say it holds nothing — ADR 0001 §2 says the agent service never runs \
         with authority the person does not have, and ADR 0018 moved the one privileged act out \
         of it; both capability lines must be there and both must be empty (bounded to {bounded:?}, \
         given {given:?})"
    )]
    TheAgentHoldsSomething {
        /// The agent's unit.
        agent: String,
        /// What the unit bounds it to.
        bounded: Vec<String>,
        /// What the unit gives it.
        given: Vec<String>,
    },
    /// The agent's service runs as somebody who is not the person the machine
    /// description names.
    #[error(
        "{agent} runs as `{as_login}`, which this image does not make with number {person} — the \
         machine description says that number is the signed-in person, and the kernel's answer \
         about who is on the socket is compared against it"
    )]
    TheAgentIsNotThePerson {
        /// The agent's unit.
        agent: String,
        /// The login it names, or `-` where it names none.
        as_login: String,
        /// The number the description says the person is.
        person: u32,
    },
    /// The agent's service is not in the group the socket is handed to.
    #[error(
        "{agent} is not in group `{group}` — the socket is handed to that group, and changing a \
         file's group is only allowed to a member of it, so this service would bind a door it \
         cannot give the agent"
    )]
    TheAgentIsNotInTheGroup {
        /// The agent's unit.
        agent: String,
        /// The group it should be in.
        group: String,
    },
    /// The person's login is not in the agent's group outside the unit either.
    #[error(
        "this image does not put login `{login}` in group `{group}` — the unit puts the service \
         in it, and a person who is only in it while systemd is starting them is a person for \
         whom the same daemon run by hand behaves differently"
    )]
    ThePersonIsNotInTheGroup {
        /// The person's login.
        login: String,
        /// The group.
        group: String,
    },
    /// The agent's own login is not one this image makes.
    #[error(
        "this image makes no login with number {agent} — the machine description says that is the \
         agent, ADR 0001 §5 makes it a login of its own, and SO_PEERCRED is the whole of the \
         division between the two doors on the socket"
    )]
    TheAgentHasNoLogin {
        /// The number the description says the agent is.
        agent: u32,
    },
    /// A directory the daemon refuses to make is one nothing makes.
    #[error(
        "nothing in this image makes {at} at boot, and `alo-agentd` refuses to make it — the \
         service would name the directory and stop"
    )]
    NotMadeAtBoot {
        /// The directory.
        at: PathBuf,
    },
    /// The directory every person's door goes in is not what ADR 0017 says.
    #[error(
        "{at} is made {mode:04o} {owner}:{group}, and ADR 0017 says 0755 root:root — every \
         person's door goes in it, so its mode is not one person's service's to choose"
    )]
    TheDoorIsNotWhatWasDecided {
        /// The directory.
        at: PathBuf,
        /// The mode it is made with.
        mode: u32,
        /// The login that owns it.
        owner: String,
        /// The group it is in.
        group: String,
    },
    /// The folder the record goes in belongs to somebody else.
    #[error(
        "{at} is made for `{owner}`, and the record is written by `{person}` — what an agent did \
         on somebody's machine is theirs, and a folder the service cannot write is a service that \
         will not start"
    )]
    TheRecordFolderIsNotThePersons {
        /// The directory.
        at: PathBuf,
        /// The login it is made for.
        owner: String,
        /// The login the service runs as.
        person: String,
    },
    /// The image ships a number of days nobody chose.
    #[error(
        "this image ships a record kept for {days} days — ADR 0004 gives retention to the \
         organisation that manages the machine, and the only rule an image may write is `forever`"
    )]
    ARetentionNobodyChose {
        /// How many days it ships.
        days: u32,
    },
    /// The agent's service could not make the control group a turn runs in.
    #[error(
        "{agent} does not delegate its control group — a turn is a cgroup made under the \
         service's own (ADR 0015), a service running as a person can only make one where systemd \
         has handed the subtree over, and without `Delegate=` this machine boots to a daemon that \
         stops on the boundary it cannot make"
    )]
    TheAgentCannotMakeATurnsControlGroup {
        /// The agent's unit.
        agent: String,
    },
    /// Nothing makes the person's own door, which the daemon cannot make itself.
    #[error(
        "{agent} does not make {expected} — ADR 0017 puts every person's door in a directory that \
         is 0755 root:root, so the service cannot make its own name there, and this is the \
         directory the socket for person {person} goes in (it names {declared:?})"
    )]
    ThePersonsDoorIsNotMade {
        /// The agent's unit.
        agent: String,
        /// The runtime directory it should name, relative to `/run`.
        expected: String,
        /// The number the description says the person is.
        person: u32,
        /// What the unit names instead.
        declared: Vec<String>,
    },
    /// The person's door would be made where anybody can look in.
    #[error(
        "{agent} makes the person's door {mode} — ADR 0017 says 0750, the person and the agent's \
         group and nobody else, and a unit that leaves it to systemd's default gets 0755 rather \
         than a decision"
    )]
    ThePersonsDoorIsNotShut {
        /// The agent's unit.
        agent: String,
        /// The mode the unit names, or `-` where it names none.
        mode: String,
    },
    /// A unit nothing pulls in at boot.
    #[error("nothing pulls {unit} in at boot — it has no [Install] section that wants it")]
    NothingPullsItIn {
        /// The unit.
        unit: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sentence names the promise it is about, because the person reading
    /// it is looking at a diff rather than at this repository's documentation.
    #[test]
    fn a_disagreement_names_the_decision_it_is_about() {
        assert!(
            Wrong::TheLoaderHoldsSomethingElse {
                loader: "alo-boundaryd.service".to_owned(),
                held: vec!["CAP_SYS_ADMIN".to_owned()],
            }
            .to_string()
            .contains("ADR 0018")
        );
        assert!(
            Wrong::TheDoorIsNotWhatWasDecided {
                at: PathBuf::from("/run/alo"),
                mode: 0o777,
                owner: "alo".to_owned(),
                group: "alo".to_owned(),
            }
            .to_string()
            .contains("ADR 0017")
        );
    }

    /// A mode is said the way somebody wrote it, in octal with its leading zero,
    /// rather than as the number it happens to be.
    #[test]
    fn a_mode_is_said_in_octal() {
        let said = Wrong::TheDoorIsNotWhatWasDecided {
            at: PathBuf::from("/run/alo"),
            mode: 0o777,
            owner: "alo".to_owned(),
            group: "alo".to_owned(),
        }
        .to_string();
        assert!(said.contains("0777"), "{said}");
    }
}
