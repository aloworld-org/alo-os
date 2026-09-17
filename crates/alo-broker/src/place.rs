//! Where the broker's door and its approving key are on a machine.
//!
//! Both sides read these two names — the broker's process when it opens them,
//! and whatever issues tokens and asks at the door — so they are written once,
//! here. Both are in the broker's own runtime directory, which systemd makes
//! `0750 root:alo`: the person's own group, and nobody else's. Not under
//! `/run/alo`, which is every person's door (ADR 0017); this one is the
//! machine's.

/// The door: the one socket the broker listens at.
pub const THE_DOOR: &str = "/run/alo-broker/door.sock";

/// The approving key, handed over to the side that issues tokens.
pub const THE_KEY: &str = "/run/alo-broker/approving.key";
