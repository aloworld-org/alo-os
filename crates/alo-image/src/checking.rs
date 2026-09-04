//! Every promise the image's files make to the two daemons, checked against
//! each other.
//!
//! A build cannot catch any of this. `docker build` will produce an image whose
//! machine description names a login the image never creates, whose loader is
//! in root's group, or whose agent service has quietly been given a capability
//! — and every one of those is a machine that boots to a daemon that stops, or
//! worse, one that runs while a sentence in `docs/` about it has become untrue.
//!
//! # Everything wrong at once, rather than the first thing wrong
//!
//! [`everything_wrong_with`] answers with all of them, which is
//! `alo-strings`' `Vocabulary::check` argument met a second time: being told
//! about the next mistake each time you try again is how somebody gives up. An
//! image is edited in one sitting and there are five files in it.
//!
//! # What is checked, and what deliberately is not
//!
//! Every check here is a sentence in an ADR or a contract, named in the
//! [`Wrong`] it produces. What is **not** here is anything about whether the
//! image boots: that is the machine half of `ROADMAP.md`'s image line, it needs
//! a machine, and an image that builds is not an image that boots.

use std::path::Path;

use alo_keeping::Keeping;

use crate::image::Image;
use crate::service::ROOT;
use crate::wrong::Wrong;

/// The directory every person's door goes in (ADR 0017).
pub const THE_DOOR: &str = "/run/alo";

/// The mode that directory is made with: anybody may walk through, nobody but
/// root may put a name in it.
const THE_DOORS_MODE: u32 = 0o755;

/// The two capabilities ADR 0018 gives the loader, and the whole of what makes
/// one privileged component acceptable.
const WHAT_THE_LOADER_MAY_HOLD: [&str; 2] = ["CAP_BPF", "CAP_SYS_ADMIN"];

/// What a unit says where it says nothing, in a sentence somebody reads.
const NOTHING: &str = "-";

/// Everything this image's files disagree with each other about.
///
/// An empty answer is an image whose five files say one thing. It is not a
/// claim that the machine boots.
#[must_use]
pub fn everything_wrong_with(image: &Image) -> Vec<Wrong> {
    let mut wrong = Vec::new();
    the_loader_runs_first(image, &mut wrong);
    the_loader_holds_two_things(image, &mut wrong);
    the_agent_holds_nothing(image, &mut wrong);
    the_logins_are_the_ones_this_image_makes(image, &mut wrong);
    the_directories_are_made(image, &mut wrong);
    nobody_chose_a_retention(image, &mut wrong);
    both_units_are_pulled_in(image, &mut wrong);
    wrong
}

/// ADR 0018 and ADR 0015: the boundary is on the kernel before the agent's
/// service exists, and it stays there after the loader has gone.
fn the_loader_runs_first(image: &Image, wrong: &mut Vec<Wrong>) {
    let loader = image.loader().called().to_owned();
    let agent = image.agent().called().to_owned();

    if !image.agent().needs().contains(&loader.as_str())
        || !image.agent().after().contains(&loader.as_str())
    {
        wrong.push(Wrong::TheAgentDoesNotWaitForTheBoundary {
            agent: agent.clone(),
            loader: loader.clone(),
        });
    }
    if image.loader().kind() != Some("oneshot") || !image.loader().stays_after_exiting() {
        wrong.push(Wrong::TheLoaderDoesNotStay {
            loader,
            kind: image.loader().kind().unwrap_or(NOTHING).to_owned(),
        });
    }
}

/// ADR 0018: root, in the agent's group, holding exactly two capabilities.
fn the_loader_holds_two_things(image: &Image, wrong: &mut Vec<Wrong>) {
    let loader = image.loader().called().to_owned();

    if image.loader().as_login() != Some(ROOT) {
        wrong.push(Wrong::TheLoaderIsNotRoot {
            loader: loader.clone(),
            as_login: image.loader().as_login().unwrap_or(NOTHING).to_owned(),
        });
    }

    let group = image.loader().in_group().unwrap_or(NOTHING);
    if image.group_called(group) != Some(image.description().group()) {
        wrong.push(Wrong::TheLoaderIsInTheWrongGroup {
            loader: loader.clone(),
            group: group.to_owned(),
        });
    }

    let mut held: Vec<String> = image
        .loader()
        .bounded_to()
        .into_iter()
        .map(str::to_owned)
        .collect();
    held.sort_unstable();
    if held != WHAT_THE_LOADER_MAY_HOLD || !image.loader().given().is_empty() {
        wrong.push(Wrong::TheLoaderHoldsSomethingElse { loader, held });
    }
}

/// ADR 0001 §2 and ADR 0018: the service that talks to the agent holds nothing,
/// and says so.
fn the_agent_holds_nothing(image: &Image, wrong: &mut Vec<Wrong>) {
    if !image.agent().holds_nothing() {
        wrong.push(Wrong::TheAgentHoldsSomething {
            agent: image.agent().called().to_owned(),
            bounded: image
                .agent()
                .bounded_to()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            given: image
                .agent()
                .given()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        });
    }
}

/// The machine description's three numbers are the accounts this image creates,
/// and the person can reach the group the socket is handed to.
fn the_logins_are_the_ones_this_image_makes(image: &Image, wrong: &mut Vec<Wrong>) {
    let person = image.agent().as_login().unwrap_or(NOTHING);
    if image.login_called(person) != Some(image.description().person()) {
        wrong.push(Wrong::TheAgentIsNotThePerson {
            agent: image.agent().called().to_owned(),
            as_login: person.to_owned(),
            person: image.description().person(),
        });
    }

    // The agent's login is the one thing the description names that no unit file
    // does, so what can be asked here is whether the image makes it at all. The
    // person and the agent being one login is the description's own refusal and
    // is tested where that refusal is made.
    if !image.makes_a_login_numbered(image.description().agent()) {
        wrong.push(Wrong::TheAgentHasNoLogin {
            agent: image.description().agent(),
        });
    }

    let group = image.loader().in_group().unwrap_or(NOTHING);
    if !image.agent().beside_groups().contains(&group) {
        wrong.push(Wrong::TheAgentIsNotInTheGroup {
            agent: image.agent().called().to_owned(),
            group: group.to_owned(),
        });
    }
    if !image.puts(person, group) {
        wrong.push(Wrong::ThePersonIsNotInTheGroup {
            login: person.to_owned(),
            group: group.to_owned(),
        });
    }
}

/// ADR 0017 and the machine description: the two directories `alo-agentd`
/// refuses to make are made by something.
fn the_directories_are_made(image: &Image, wrong: &mut Vec<Wrong>) {
    match image.directory_at(Path::new(THE_DOOR)) {
        None => wrong.push(Wrong::NotMadeAtBoot {
            at: Path::new(THE_DOOR).to_owned(),
        }),
        Some(door) => {
            if door.mode() != THE_DOORS_MODE || door.owner() != ROOT || door.group() != ROOT {
                wrong.push(Wrong::TheDoorIsNotWhatWasDecided {
                    at: door.at().to_owned(),
                    mode: door.mode(),
                    owner: door.owner().to_owned(),
                    group: door.group().to_owned(),
                });
            }
        }
    }

    let Some(folder) = image.description().record_folder() else {
        // A record path with no folder above it is a relative one, which the
        // daemon refuses; there is nothing here for an image to make.
        return;
    };
    let person = image.agent().as_login().unwrap_or(NOTHING);
    match image.directory_at(folder) {
        None => wrong.push(Wrong::NotMadeAtBoot {
            at: folder.to_owned(),
        }),
        Some(made) => {
            if made.owner() != person {
                wrong.push(Wrong::TheRecordFolderIsNotThePersons {
                    at: made.at().to_owned(),
                    owner: made.owner().to_owned(),
                    person: person.to_owned(),
                });
            }
        }
    }
}

/// ADR 0004: how long a record is kept is the organisation's to name, so the
/// only rule an image may ship is everything.
fn nobody_chose_a_retention(image: &Image, wrong: &mut Vec<Wrong>) {
    if let Keeping::ForDays(days) = image.description().keeping() {
        wrong.push(Wrong::ARetentionNobodyChose { days: days.get() });
    }
}

/// A unit nothing pulls in at boot is a unit that is shipped and never runs.
fn both_units_are_pulled_in(image: &Image, wrong: &mut Vec<Wrong>) {
    for service in [image.loader(), image.agent()] {
        if service.wanted_by().is_empty() {
            wrong.push(Wrong::NothingPullsItIn {
                unit: service.called().to_owned(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        THE_AGENTS_UNIT, THE_DESCRIPTION_FILE, THE_LOADERS_UNIT, THE_SYSUSERS, THE_TMPFILES,
        a_copy_of_the_image, edited, image_at,
    };

    /// **The image this repository ships says one thing.** Everything below
    /// breaks one file of it and asks whether that is noticed, and none of those
    /// tests would mean anything without this one.
    #[test]
    fn the_image_this_repository_ships_agrees_with_itself() {
        let wrong = everything_wrong_with(&image_at(Path::new(crate::THE_IMAGE)));
        assert!(wrong.is_empty(), "{wrong:?}");
    }

    /// **An agent service that does not wait for the boundary is caught.** ADR
    /// 0015: a turn that cannot be bounded does not run, so a machine whose
    /// boundary never loaded must not reach a service that refuses every turn.
    #[test]
    fn an_agent_that_would_start_without_the_boundary_is_caught() {
        let root = a_copy_of_the_image("no-wait");
        edited(&root, THE_AGENTS_UNIT, "Requires=alo-boundaryd.service", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentDoesNotWaitForTheBoundary { .. })),
            "{wrong:?}"
        );
    }

    /// **A loader whose unit does not stay is caught.** The boundary outlives
    /// the process that loaded it, and a unit going inactive would tell
    /// `systemctl status` that a machine with a boundary has none.
    #[test]
    fn a_loader_that_does_not_stay_after_exiting_is_caught() {
        let root = a_copy_of_the_image("does-not-stay");
        edited(&root, THE_LOADERS_UNIT, "RemainAfterExit=yes", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheLoaderDoesNotStay { .. })),
            "{wrong:?}"
        );
    }

    /// **A loader in root's group is caught**, which is the mistake the loader
    /// itself refuses to start on: the map of turns is handed to whatever group
    /// this process is in, so root's group is a map only root could write.
    #[test]
    fn a_loader_in_roots_group_is_caught() {
        let root = a_copy_of_the_image("roots-group");
        edited(&root, THE_LOADERS_UNIT, "Group=alo-agent", "Group=root");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheLoaderIsInTheWrongGroup { .. })),
            "{wrong:?}"
        );
    }

    /// **A loader given a third capability is caught.** What makes one
    /// privileged component acceptable is the size of what it is trusted with,
    /// and this is the line where that is true rather than written down.
    #[test]
    fn a_loader_holding_more_than_adr_0018_gave_it_is_caught() {
        let root = a_copy_of_the_image("a-third");
        edited(
            &root,
            THE_LOADERS_UNIT,
            "CapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN",
            "CapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN CAP_NET_ADMIN",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheLoaderHoldsSomethingElse { .. })),
            "{wrong:?}"
        );
    }

    /// **An agent service quietly given a capability is caught**, which is the
    /// exact shape ADR 0018 exists to prevent: not a privileged daemon, a
    /// directive added to an ordinary one.
    #[test]
    fn an_agent_service_given_a_capability_is_caught() {
        let root = a_copy_of_the_image("a-capability");
        edited(
            &root,
            THE_AGENTS_UNIT,
            "AmbientCapabilities=",
            "AmbientCapabilities=CAP_BPF",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **And an agent service that merely stops saying it holds nothing is
    /// caught too.** The two empty lines are the claim; deleting them leaves a
    /// service that holds nothing today and nothing saying it must.
    #[test]
    fn an_agent_service_that_stops_saying_so_is_caught() {
        let root = a_copy_of_the_image("stopped-saying");
        edited(&root, THE_AGENTS_UNIT, "CapabilityBoundingSet=", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **A description naming a person the image never creates is caught.**
    /// This is the disagreement a build cannot see and a boot turns into a
    /// daemon that stops.
    #[test]
    fn a_description_naming_a_login_nobody_makes_is_caught() {
        let root = a_copy_of_the_image("no-person");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "person = 1000",
            "person = 1001",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentIsNotThePerson { .. })),
            "{wrong:?}"
        );
    }

    /// **And a description whose agent number nothing makes is caught**, which
    /// is ADR 0001 §5: the agent is a login of its own, and SO_PEERCRED is the
    /// whole of the division between the socket's two doors.
    #[test]
    fn a_description_whose_agent_is_no_login_is_caught() {
        let root = a_copy_of_the_image("no-agent-login");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "agent = 60989",
            "agent = 60991",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentHasNoLogin { .. })),
            "{wrong:?}"
        );
    }

    /// **A person left out of the agent's group is caught.** The socket is
    /// handed to that group, and changing a file's group is only allowed to a
    /// member of it — so this is a service that binds a door it cannot give away.
    #[test]
    fn a_person_who_cannot_reach_the_group_is_caught() {
        let root = a_copy_of_the_image("not-in-the-group");
        edited(&root, THE_AGENTS_UNIT, "SupplementaryGroups=alo-agent", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentIsNotInTheGroup { .. })),
            "{wrong:?}"
        );
    }

    /// And the same login outside the unit, which is the half a `systemd`
    /// directive does not cover.
    #[test]
    fn a_login_the_image_never_puts_in_the_group_is_caught() {
        let root = a_copy_of_the_image("no-membership");
        edited(&root, THE_SYSUSERS, "m alo alo-agent", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ThePersonIsNotInTheGroup { .. })),
            "{wrong:?}"
        );
    }

    /// **A door nothing makes is caught**, which is ADR 0017's whole
    /// consequence: `alo-agentd` refuses to make `/run/alo` and names it, and an
    /// image that forgot is a machine that boots to that sentence.
    #[test]
    fn a_door_nothing_makes_is_caught() {
        let root = a_copy_of_the_image("no-door");
        edited(&root, THE_TMPFILES, "d /run/alo 0755 root root -", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::NotMadeAtBoot { at } if at == Path::new(THE_DOOR))),
            "{wrong:?}"
        );
    }

    /// **A door left open is caught.** Every person's door goes in it, so
    /// anybody who could write it could replace anybody's socket.
    #[test]
    fn a_door_anybody_could_write_is_caught() {
        let root = a_copy_of_the_image("open-door");
        edited(
            &root,
            THE_TMPFILES,
            "d /run/alo 0755 root root -",
            "d /run/alo 0777 root root -",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDoorIsNotWhatWasDecided { mode: 0o777, .. })),
            "{wrong:?}"
        );
    }

    /// **A record folder nothing makes is caught**, which is the machine
    /// description's rule: the file is made on first start and the folder above
    /// it is not, so a typo there would become a second record nobody reads.
    #[test]
    fn a_record_folder_nothing_makes_is_caught() {
        let root = a_copy_of_the_image("no-record-folder");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "/var/lib/alo/record.jsonl",
            "/var/lib/somewhere-else/record.jsonl",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::NotMadeAtBoot { .. })),
            "{wrong:?}"
        );
    }

    /// **A record folder belonging to somebody else is caught.** What an agent
    /// did on somebody's machine is theirs, and a folder the service cannot
    /// write is a service that will not start.
    #[test]
    fn a_record_folder_that_is_not_the_persons_is_caught() {
        let root = a_copy_of_the_image("someone-elses-record");
        edited(
            &root,
            THE_TMPFILES,
            "d /var/lib/alo 0700 alo alo -",
            "d /var/lib/alo 0700 root root -",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheRecordFolderIsNotThePersons { .. })),
            "{wrong:?}"
        );
    }

    /// **A retention alo OS chose for somebody is caught.** ADR 0004 gives it to
    /// the organisation, and a number of days that sounded reasonable is exactly
    /// what `CLAUDE.md` refuses to ship.
    #[test]
    fn a_number_of_days_shipped_in_the_image_is_caught() {
        let root = a_copy_of_the_image("a-retention");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "keeping = \"forever\"",
            "keeping = { for-days = 90 }",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ARetentionNobodyChose { days: 90 })),
            "{wrong:?}"
        );
    }

    /// **A unit nothing pulls in is caught**, which is the failure that looks
    /// most like success: the file is in the image, `systemctl cat` shows it,
    /// and nothing ever starts it.
    #[test]
    fn a_unit_nothing_pulls_in_is_caught() {
        let root = a_copy_of_the_image("never-started");
        edited(&root, THE_LOADERS_UNIT, "WantedBy=multi-user.target", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::NothingPullsItIn { .. })),
            "{wrong:?}"
        );
    }

    /// **Everything wrong comes back at once**, rather than the first of it.
    /// An image is edited in one sitting and has five files in it.
    #[test]
    fn two_things_wrong_are_two_answers() {
        let root = a_copy_of_the_image("two-things");
        edited(&root, THE_LOADERS_UNIT, "WantedBy=multi-user.target", "");
        edited(&root, THE_TMPFILES, "d /run/alo 0755 root root -", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(wrong.len() >= 2, "{wrong:?}");
    }
}
