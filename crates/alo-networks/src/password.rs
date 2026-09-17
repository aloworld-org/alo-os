//! A Wi-Fi password, held for exactly one answer to the network manager.
//!
//! **It goes in and does not come out**, except through the one function that
//! puts it where it belongs: no `Display`, no `Serialize`, no `Clone`, and a
//! `Debug` written by hand to say nothing. `crate::secret_agent` is the only
//! caller of [`WifiPassword::handed_to_the_network_manager`], so everything that
//! could ever carry one is what `grep` finds.
//!
//! **What this does not claim**, as `alo_proxy::Password` and
//! `alo_models::Secret` say for the same reason: the bytes are not scrubbed from
//! memory when the value is dropped, because doing that honestly needs `unsafe`
//! or a dependency.

use std::fmt;

/// The fewest characters a Wi-Fi passphrase may be (IEEE 802.11i).
const SHORTEST: usize = 8;

/// The most characters a Wi-Fi passphrase may be; sixty-four is a raw key.
const LONGEST: usize = 64;

/// Why what a person typed is not a Wi-Fi password.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPassword {
    /// Fewer than eight characters, or more than sixty-four.
    WrongLength,
    /// A character a Wi-Fi password cannot hold: anything but printable ASCII,
    /// or — at sixty-four characters — anything but hexadecimal.
    NotPrintable,
}

/// A Wi-Fi password a person typed.
pub struct WifiPassword(String);

impl WifiPassword {
    /// What a person typed, if a Wi-Fi network could accept it.
    ///
    /// # Errors
    /// [`NotAPassword`]. Refused rather than trimmed: a space a person typed is
    /// part of their password.
    pub fn typed(typed: &str) -> Result<Self, NotAPassword> {
        let length = typed.chars().count();
        if !(SHORTEST..=LONGEST).contains(&length) {
            return Err(NotAPassword::WrongLength);
        }
        let printable = if length == LONGEST {
            typed.chars().all(|letter| letter.is_ascii_hexdigit())
        } else {
            typed.chars().all(|letter| (' '..='~').contains(&letter))
        };
        if !printable {
            return Err(NotAPassword::NotPrintable);
        }
        Ok(Self(typed.to_owned()))
    }

    /// The password, for the one answer it is typed for.
    #[cfg(target_os = "linux")]
    pub(crate) fn handed_to_the_network_manager(self) -> String {
        self.0
    }
}

impl fmt::Debug for WifiPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WifiPassword(…)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A password a network could take is taken, and nothing else is.**
    #[test]
    fn a_password_is_what_a_network_could_take() {
        for typed in [
            "correct horse",
            "12345678",
            &"a".repeat(63),
            &"0f".repeat(32),
        ] {
            assert!(WifiPassword::typed(typed).is_ok(), "{typed}");
        }
        for (typed, why) in [
            ("short", NotAPassword::WrongLength),
            (&"a".repeat(65)[..], NotAPassword::WrongLength),
            ("pässwörter", NotAPassword::NotPrintable),
            ("new\nline!", NotAPassword::NotPrintable),
            (&"g".repeat(64)[..], NotAPassword::NotPrintable),
        ] {
            assert_eq!(WifiPassword::typed(typed).err(), Some(why), "{typed}");
        }
    }

    /// **Looking at one shows nothing of it.**
    #[test]
    fn looking_at_a_password_shows_nothing_of_it() {
        let Ok(password) = WifiPassword::typed("hunter2hunter2") else {
            return assert!(WifiPassword::typed("hunter2hunter2").is_ok());
        };
        assert!(!format!("{password:?}").contains("hunter2"));
    }
}
