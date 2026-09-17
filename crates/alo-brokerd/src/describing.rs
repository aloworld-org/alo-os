//! Who the door is for, from what this machine says about itself.
//!
//! The door hears one user: the one `alo-agentd` runs as, which on this image
//! is the signed-in person (ADR 0001 §2). That number is written once, in the
//! machine description (`docs/contracts/machine-description.md`), and the image
//! holds it to the logins it declares — so the broker reads it there rather
//! than being told a second copy that could disagree.
//!
//! Two numbers are read: the person, whom the door hears, and the group the
//! agent's own login is in, which the broker must never hand its door or its key
//! to. Everything else in the description is somebody else's and is ignored.

use serde::Deserialize;

/// The two logins the broker needs to know apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Logins {
    /// The user `alo-agentd` runs as, whom the door hears.
    pub person: u32,
    /// The group the agent's own login is in, which must never reach the door.
    pub agents_group: u32,
}

/// The part of the machine description this reads.
#[derive(Deserialize)]
struct Description {
    /// The `[logins]` table.
    logins: Written,
}

/// `[logins]`, as written.
#[derive(Deserialize)]
struct Written {
    /// `person`.
    person: u32,
    /// `group`.
    group: u32,
}

/// The machine description did not say who the door is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotDescribed(pub String);

impl std::fmt::Display for NotDescribed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the machine description does not say who alo-agentd runs as: {}",
            self.0
        )
    }
}

impl std::error::Error for NotDescribed {}

/// The logins, from the text of the machine description.
///
/// # Errors
/// [`NotDescribed`] when the text is not TOML or has no `[logins]` with a
/// `person` and a `group` in it.
pub fn logins(description: &str) -> Result<Logins, NotDescribed> {
    let read: Description =
        toml::from_str(description).map_err(|why| NotDescribed(why.message().to_owned()))?;
    Ok(Logins {
        person: read.logins.person,
        agents_group: read.logins.group,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The image's own description says who the door is for.**
    #[test]
    fn the_images_description_names_the_person_and_the_agents_group() {
        let written = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../image/etc/alo/agentd.toml"
        ))
        .unwrap_or_default();
        assert_eq!(
            logins(&written),
            Ok(Logins {
                person: 1000,
                agents_group: 60989,
            })
        );
    }

    /// A description that does not say is not guessed at.
    #[test]
    fn a_description_that_does_not_say_is_not_guessed_at() {
        for written in [
            "",
            "format = 1",
            "[logins]\nperson = 1000",
            "[logins]\nperson = \"alo\"\ngroup = 60989",
            "[logins]\nperson = -1\ngroup = 60989",
            "not toml at all [",
        ] {
            assert!(logins(written).is_err(), "{written:?}");
        }
    }
}
