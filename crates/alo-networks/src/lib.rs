//! What a network is on this machine, as the rented network manager reports it.
//!
//! ADR 0011 rents NetworkManager, and the broker plan says so again: *configured,
//! never patched*. This crate is everything alo OS needs to know about what that
//! service reports, and the one client that asks it and tells it — read by the
//! person's side, which lists networks and says what a change would do, and by
//! the privileged broker, which carries a change out against what the service
//! reports at that moment.
//!
//! | | |
//! |---|---|
//! | [`NetworkName`], [`Visible`], [`Saved`], [`Protection`], [`Primary`], [`TheNetworks`] | What the network manager reported, and nothing it did not |
//! | [`Networks`], [`NetworkService`] | Asking what there is now, and the three changes the broker makes |
//! | `network_manager` | The client that speaks the network manager's own interface on the system bus (Linux only) |
//! | `secret_agent` | Where a Wi-Fi password is asked of a person, and handed to the network manager and nobody else (Linux only) |
//! | [`WifiPassword`] | A password a person typed, held for one answer |
//! | [`proxy_file`] | The machine's proxy file, and the wanted proxy a person hands the broker |
//! | [`proxy_password`] | The password a person hands the broker beside it, and the one identity they are approved under (ADR 0060) |
//!
//! # A network is compared by what was reported, never by what anybody typed
//!
//! Each reported thing has [`Visible::as_reported`] or [`Saved::as_reported`]:
//! the bytes the broker's `alo_broker::Identity` is digested from on both sides
//! of its door. A visible network is its name **and how it is protected**, so an
//! open network calling itself by a protected network's name is a different
//! network, never the one a person approved.
//!
//! # A Wi-Fi password never passes through the agent
//!
//! Joining a protected network hands the network manager no password at all. It
//! asks the secret agents registered on the system bus, and the one alo OS
//! registers runs in the person's own session, answers the network manager and
//! nobody else, and asks the person through a surface of their session that an
//! agent's login cannot read (`secret_agent`). The broker, its door, the agent's
//! verbs and the record have nowhere to put one.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(target_os = "linux")]
mod bus;
#[cfg(target_os = "linux")]
pub mod network_manager;
mod password;
pub mod proxy_file;
pub mod proxy_password;
mod reported;
#[cfg(target_os = "linux")]
pub mod secret_agent;
mod service;

pub use password::{NotAPassword, WifiPassword};
pub use reported::{LONGEST_NAME, NetworkName, Primary, Protection, Saved, TheNetworks, Visible};
pub use service::{NetworkService, Networks, NotAnswering, NotDone};
