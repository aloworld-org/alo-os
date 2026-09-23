//! The privileged broker as a machine runs it.
//!
//! `alo-broker` is the closed list and the door, and it is held small enough to
//! audit in an afternoon. This crate is the process around it and everything
//! the door deliberately does not decide:
//!
//! | | |
//! |---|---|
//! | [`started`], [`Places`], [`NotStarted`] | What happens before the door opens, in order, each step refusing before the next |
//! | [`logins`] | Who the door is for, from the machine description |
//! | [`MachinesRecord`] | The broker's record file, written before every answer |
//! | [`Carriers`] | Which verbs this machine carries out, and the refusal for every other, by name |
//! | [`Network`] | The network's three verbs, carried out against what the network manager reports now |
//! | [`Proxy`] | Setting the machine's proxy, and the password it signs in with, to the ones a person handed over, exactly |
//! | [`Storage`] | A removable drive mounted for the signed-in person, or ejected, against what the disk service reports now |
//! | [`Printers`], [`PrintService`] | The printers' three verbs, carried out against what the printing service reports now |
//! | [`Updates`], [`StartingUnits`], [`TheUnit`] | The two update verbs, carried out by starting a unit that holds what the base asks for |
//! | [`NextStart`], [`the_identity_of`] | *Restart into Windows*, carried out by setting the firmware's next start — once, and leaving the order alone |
//! | [`ByDefault`] | *Start this system when nobody chooses*, carried out by changing the one place that answer is kept — and leaving the next start alone |
//! | [`AnUpdate`], [`GoingBackApproved`], [`Handing`] | What a person approved about an update, and the folder only root can read that carries it to the unit |
//! | [`stage_the_update_approved`], [`set_going_back`] | What the two units' own programs do, which is every decision in them |
//! | `alo-brokerd.service` | The unit, beside this manifest, held to what the process expects by a test |
//! | `alo-applying-an-update.service`, `alo-going-back.service` | The two units the broker starts, held to their fixed command line and their capabilities line by line |
//!
//! # Root, holding nothing
//!
//! The broker runs as root because the network manager and the disk service
//! decide who may change the machine's network, and who may mount a drive for
//! another login, from the credentials of whoever asks; and because the
//! machine's proxy file is root's to write. The printing service also checks
//! the credentials of whoever asks to configure a printer. The broker holds
//! **no capability** — both capability lines in its unit are empty — and runs in the person's own group,
//! so its door and its approving key are theirs to reach and not the agent's.
//! `tests/the_unit_is_the_process.rs` holds the unit to that.
//!
//! # Carried out, and nothing else
//!
//! What carries a verb out is handed the verb only after the door has decided
//! it is exactly one a person approved, once, and written that down. The network
//! and storage, and the printers supplied to the carriers, are carried out here.
//! **The updates are carried out by a unit this process starts and waits for**
//! — ADR 0053, accepted option B — because the base's own program changes the
//! machine only for a process holding `CAP_SYS_ADMIN`, and this one holds none
//! and is not given one. The units' own programs are this crate's second and
//! third binaries, and they are where the base is named. Each answer is written
//! down like every other.
//!
//! # And it runs on Linux
//!
//! The door is a Unix socket, and the network manager and the disk service are
//! spoken to over the system bus. On any other host the process says so and ends in failure.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(unix)]
pub mod approved;
mod by_default;
#[cfg(unix)]
mod carrying;
#[cfg(unix)]
mod describing;
#[cfg(unix)]
pub mod for_the_unit;
#[cfg(unix)]
mod handed_over;
mod network;
mod next_start;
mod printers;
mod printing_service;
#[cfg(unix)]
mod proxy;
#[cfg(unix)]
mod recording;
#[cfg(unix)]
mod returning;
#[cfg(unix)]
mod staging_an_update;
#[cfg(unix)]
mod starting;
mod storage;
#[cfg(unix)]
pub mod the_machine_now;
#[cfg(unix)]
mod the_road_out;
#[cfg(unix)]
pub mod units;
#[cfg(unix)]
mod updates;

#[cfg(unix)]
pub use approved::{AnUpdate, GoingBackApproved, NotAnUpdate};
pub use by_default::ByDefault;
#[cfg(unix)]
pub use carrying::{Carriers, NoFirmware, NoLoader, NoUnits};
#[cfg(unix)]
pub use describing::{Logins, NotDescribed, logins};
#[cfg(unix)]
pub use for_the_unit::{Handing, NotHanded};
pub use network::Network;
pub use next_start::{NextStart, the_identity_of};
pub use printers::{PrintService, Printers, Reported};
#[cfg(unix)]
pub use proxy::Proxy;
#[cfg(unix)]
pub use recording::{MachinesRecord, THE_RECORD};
#[cfg(unix)]
pub use returning::{NotSetByTheUnit, THE_MACHINES_RECORD, carried_out as set_going_back};
#[cfg(unix)]
pub use staging_an_update::{NotStagedByTheUnit, carried_out as stage_the_update_approved};
#[cfg(unix)]
pub use starting::{NotStarted, Places, Started, THE_DESCRIPTION, started};
pub use storage::{Storage, THE_ACCOUNTS};
#[cfg(unix)]
pub use the_machine_now::TheMachineNow;
#[cfg(unix)]
pub use the_road_out::{NoRoad, the_road_out};
#[cfg(target_os = "linux")]
pub use units::systemd::{OnThisMachine as TheSystemManager, Systemd};
#[cfg(unix)]
pub use units::{EVERY_UNIT, StartingUnits, TheUnit};
#[cfg(unix)]
pub use updates::{THE_WANTED_UPDATE, Updates};
