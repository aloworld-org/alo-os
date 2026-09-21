//! Everything the broker does before its door opens, in the order it does it.
//!
//! 1. **Who the door is for**, from the machine description — and never root.
//! 2. **Which group the door and the key go to**: the one this process runs in,
//!    which the unit makes the person's own. Never root's, which would be a
//!    door nobody but root can reach, and **never the agent's**, which would
//!    hand the agent's own login the key it must never hold.
//! 3. **The record is opened**, because a broker with nowhere to write down
//!    what it answered must answer nothing.
//! 4. **The folder a person hands a proxy or an update over in is made**, in
//!    that group and nobody else's, and **the folder the broker hands one on
//!    in is made**, root's alone — a privileged unit reading from a folder the
//!    person can write is a unit acting on bytes nobody approved.
//! 5. **A fresh key is handed over**, so nothing issued before this start is
//!    genuine and the turn reads the key this broker holds.
//! 6. **Only then the door opens.**
//!
//! Each step refuses before the next is reached, and a refusal leaves no door:
//! a socket standing open in front of a broker that could not keep its record,
//! or could not prove an approval, would be a door that answers without either.

use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use alo_broker::handing_over::{NotHandedOver, hand_over_a_fresh_key};
use alo_broker::listening::Listening;
use alo_broker::{Broker, Carrying, Door, NotADoor};
use alo_keeping::Writing;

use crate::describing::{Logins, NotDescribed, logins};
use crate::recording::MachinesRecord;

/// Where the machine description is.
pub const THE_DESCRIPTION: &str = "/etc/alo/agentd.toml";

/// The group that is root's.
const ROOTS_GROUP: u32 = 0;

/// The mode of the folder a proxy is handed over in: root and the person's
/// group may write there, nobody else may enter.
const THE_WANTED_FOLDERS_MODE: u32 = 0o770;

/// Where the broker finds and puts things.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Places {
    /// The machine description.
    pub description: PathBuf,
    /// The broker's record.
    pub record: PathBuf,
    /// Where the approving key is handed over.
    pub key: PathBuf,
    /// The door.
    pub door: PathBuf,
    /// The folder a person hands a proxy or an update over in.
    pub wanted: PathBuf,
    /// The folder the broker hands a checked update on to a unit in, which is
    /// root's alone.
    pub approved: PathBuf,
}

impl Places {
    /// Where they are on a machine.
    #[must_use]
    pub fn on_this_machine() -> Self {
        let wanted = PathBuf::from(alo_networks::proxy_file::THE_WANTED_PROXY);
        Self {
            description: PathBuf::from(THE_DESCRIPTION),
            record: PathBuf::from(crate::recording::THE_RECORD),
            key: PathBuf::from(alo_broker::THE_KEY),
            door: PathBuf::from(alo_broker::THE_DOOR),
            wanted: wanted.parent().map(PathBuf::from).unwrap_or(wanted),
            approved: PathBuf::from(crate::for_the_unit::THE_FOLDER),
        }
    }
}

/// Why the broker did not open its door.
#[derive(Debug)]
pub enum NotStarted {
    /// The machine description could not be read, or does not say who
    /// `alo-agentd` runs as.
    NotDescribed(NotDescribed),
    /// It says `alo-agentd` runs as root.
    NotADoor(NotADoor),
    /// This process runs in root's group.
    InRootsGroup,
    /// This process runs in the agent's own group.
    InTheAgentsGroup {
        /// That group.
        group: u32,
    },
    /// The record could not be opened.
    NoRecord(alo_keeping::NotKept),
    /// The folder a proxy is handed over in could not be made.
    NoFolderForAProxy(std::io::Error),
    /// The folder a checked update is handed on to a unit in could not be
    /// made, root's alone.
    NoFolderForTheUnits(std::io::Error),
    /// The key could not be handed over.
    NoKey(NotHandedOver),
    /// The door would not open.
    NoDoor(std::io::Error),
}

impl std::fmt::Display for NotStarted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotDescribed(why) => write!(f, "{why}"),
            Self::NotADoor(why) => write!(f, "{why}"),
            Self::InRootsGroup => write!(
                f,
                "this process runs in root's group, so its door and its key would be handed to \
                 nobody but root; the unit's Group= line names the person's group"
            ),
            Self::InTheAgentsGroup { group } => write!(
                f,
                "this process runs in group {group}, which is the agent's own, so its door and \
                 its approving key would be handed to the agent (ADR 0001 §2)"
            ),
            Self::NoRecord(why) => write!(
                f,
                "the broker's record could not be opened, so it answers nothing: {why:?}"
            ),
            Self::NoFolderForAProxy(why) => write!(
                f,
                "the folder a person hands a proxy over in could not be made: {why}"
            ),
            Self::NoFolderForTheUnits(why) => write!(
                f,
                "the folder the broker hands a checked update on to a unit in could not be made: \
                 {why}"
            ),
            Self::NoKey(why) => write!(f, "{why}"),
            Self::NoDoor(why) => write!(f, "the broker's door would not open: {why}"),
        }
    }
}

impl std::error::Error for NotStarted {}

/// The broker, started: its door open, its key handed over, its record open.
pub struct Started<C> {
    /// The door.
    pub listening: Listening,
    /// What answers at it.
    pub broker: Broker<MachinesRecord, C>,
}

/// Start the broker in `group`, carrying verbs out with what `carrying` makes
/// once it knows who the door is for.
///
/// # Errors
/// [`NotStarted`], and then no door is open.
pub fn started<C: Carrying>(
    places: &Places,
    group: u32,
    carrying: impl FnOnce(&Logins) -> C,
) -> Result<Started<C>, NotStarted> {
    let description = std::fs::read_to_string(&places.description).map_err(|why| {
        NotStarted::NotDescribed(NotDescribed(format!(
            "{} could not be read: {why}",
            places.description.display()
        )))
    })?;
    let logins = logins(&description).map_err(NotStarted::NotDescribed)?;
    let door = Door::for_the_agent_service(logins.person).map_err(NotStarted::NotADoor)?;
    if group == ROOTS_GROUP {
        return Err(NotStarted::InRootsGroup);
    }
    if group == logins.agents_group {
        return Err(NotStarted::InTheAgentsGroup { group });
    }
    let record = Writing::opening(&places.record).map_err(NotStarted::NoRecord)?;
    folder_for_a_proxy(places, group).map_err(NotStarted::NoFolderForAProxy)?;
    folder_for_the_units(places).map_err(NotStarted::NoFolderForTheUnits)?;
    let key = hand_over_a_fresh_key(&places.key, group).map_err(NotStarted::NoKey)?;
    let listening = Listening::at(&places.door, group).map_err(NotStarted::NoDoor)?;
    Ok(Started {
        listening,
        broker: Broker::new(door, key, MachinesRecord::of(record), carrying(&logins)),
    })
}

/// The folder a proxy is handed over in: made, emptied of anything a previous
/// start left, at its mode, and in this group.
fn folder_for_a_proxy(places: &Places, group: u32) -> Result<(), std::io::Error> {
    drop(std::fs::remove_dir_all(&places.wanted));
    std::fs::create_dir(&places.wanted)?;
    std::fs::set_permissions(
        &places.wanted,
        std::fs::Permissions::from_mode(THE_WANTED_FOLDERS_MODE),
    )?;
    rustix::fs::chown(&places.wanted, None, Some(rustix::fs::Gid::from_raw(group)))?;
    Ok(())
}

/// The folder a checked update is handed on to a unit in: made, emptied of
/// anything a previous start left, at its mode, and in nobody's group but the
/// one this process already has.
///
/// **Not the person's group, and that is the whole reason it is a second
/// folder.** Between the broker digesting what a person handed over and a
/// privileged unit reading it, anything running as that person could write the
/// file again; a unit reading from there would act on bytes nobody approved.
fn folder_for_the_units(places: &Places) -> Result<(), std::io::Error> {
    drop(std::fs::remove_dir_all(&places.approved));
    std::fs::create_dir(&places.approved)?;
    std::fs::set_permissions(
        &places.approved,
        std::fs::Permissions::from_mode(crate::for_the_unit::THE_FOLDERS_MODE),
    )
}
