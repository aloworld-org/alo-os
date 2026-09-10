//! The store as it is written down, and read back believed or refused.
//!
//! TOML, because `/etc/alo/agentd.toml` beside it is TOML and one machine
//! should not need two notations for the two files that say who it belongs
//! to. Unlike the description, this file is written by a program —
//! `crate::keeping` — and a person edits it only to break it, so every way a
//! hand-edit goes wrong is a refusal with the file's own words in it.
//!
//! ```toml
//! format = 1
//!
//! [[account]]
//! name = "alo"
//! uid = 1000
//! password = "$argon2id$v=19$m=19456,t=2,p=1$…"
//! ```
//!
//! The `password` key holds a **hash** and refuses anything else — a
//! password typed in raw is the likeliest hand-edit, and it is refused
//! loudly rather than kept as an account nothing could ever sign in to.

use serde::{Deserialize, Serialize};

use crate::account::Account;
use crate::hashing::Hashed;
use crate::refusing::NotKept;
use crate::store::Accounts;

/// Which shape of store this alo OS writes and reads.
pub const THE_FORMAT: u32 = 1;

/// The file's shape, as serde sees it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kept {
    /// Which shape this store is in.
    format: u32,
    /// The accounts, one table each.
    #[serde(rename = "account", default)]
    accounts: Vec<KeptAccount>,
}

/// One account's line in the file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeptAccount {
    /// What it is called.
    name: String,
    /// The login number it signs in as.
    uid: u32,
    /// The argon2id hash of its password — never the password.
    password: String,
}

impl Accounts {
    /// The store this text holds, believed only whole.
    ///
    /// # Errors
    ///
    /// [`NotKept::NotTheShape`] for text that is not this file,
    /// [`NotKept::AnotherFormat`] for a store from an alo OS this is not —
    /// answered before any account is looked at — [`NotKept::NotAHash`] for
    /// a password field that does not hold an argon2id hash, and
    /// [`NotKept::NotAnAccount`] for a name or number this crate would not
    /// create, duplicates included.
    pub fn read(text: &str) -> Result<Self, NotKept> {
        let kept: Kept = toml::from_str(text).map_err(|why| NotKept::NotTheShape {
            why: why.to_string(),
        })?;
        if kept.format != THE_FORMAT {
            return Err(NotKept::AnotherFormat {
                format: kept.format,
            });
        }
        let mut accounts = Vec::with_capacity(kept.accounts.len());
        for one in kept.accounts {
            let secret = Hashed::read(&one.password).ok_or(NotKept::NotAHash {
                name: one.name.clone(),
            })?;
            accounts.push(Account::read(&one.name, one.uid, secret).map_err(NotKept::from)?);
        }
        Self::holding(accounts)
    }

    /// The store as the file that keeps it.
    ///
    /// # Errors
    ///
    /// [`NotKept::NotTheShape`] if the value would not serialise — which this
    /// shape cannot cause, and is handed back rather than sworn about.
    pub fn written(&self) -> Result<String, NotKept> {
        let kept = Kept {
            format: THE_FORMAT,
            accounts: self
                .everybody()
                .iter()
                .map(|account| KeptAccount {
                    name: account.name().to_owned(),
                    uid: account.uid(),
                    password: account.secret().as_written().to_owned(),
                })
                .collect(),
        };
        toml::to_string(&kept).map_err(|why| NotKept::NotTheShape {
            why: why.to_string(),
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::refusing::{NotCreated, NotSignedIn};

    /// A store with one account, written down.
    fn adas_machine_written_down() -> String {
        let mut store = Accounts::none().unwrap();
        store
            .created("ada", 1000, "correct horse battery staple")
            .unwrap();
        store.written().unwrap()
    }

    /// **What was written reads back and still signs in** — the round trip a
    /// reboot is.
    #[test]
    fn a_store_written_down_reads_back_and_signs_in() {
        let text = adas_machine_written_down();
        assert!(text.contains("format = 1"));
        assert!(text.contains("[[account]]"));

        let read = Accounts::read(&text).unwrap();
        assert_eq!(read.how_many(), 1);
        let signed_in = read
            .signs_in("ada", "correct horse battery staple")
            .unwrap();
        assert_eq!(signed_in.uid(), 1000);
        assert_eq!(
            read.signs_in("ada", "wrong"),
            Err(NotSignedIn::Refused),
            "a round trip loosened the password"
        );
    }

    /// **An empty store round-trips too**, because first boot writes one
    /// before the first account exists.
    #[test]
    fn a_store_with_nobody_in_it_round_trips() {
        let text = Accounts::none().unwrap().written().unwrap();
        let read = Accounts::read(&text).unwrap();
        assert_eq!(read.how_many(), 0);
    }

    /// **Text that is not this file is refused**, in the parser's words.
    #[test]
    fn what_is_not_a_store_is_refused() {
        for wrong in ["=", "format = \"one\"", "just some text"] {
            assert!(
                matches!(Accounts::read(wrong), Err(NotKept::NotTheShape { .. })),
                "`{wrong}` was read as a store"
            );
        }
    }

    /// **A key this reader does not know is refused**, not skipped — the
    /// machine description's rule, because a line quietly passed over is a
    /// thing somebody believes is doing something.
    #[test]
    fn a_key_nobody_declared_is_refused() {
        let with_extra = adas_machine_written_down() + "\ntenant = \"contoso\"\n";
        assert!(matches!(
            Accounts::read(&with_extra),
            Err(NotKept::NotTheShape { .. })
        ));
    }

    /// **A store from a newer alo OS is refused as one**, before any of its
    /// accounts is looked at, rather than as whichever key happened not to
    /// parse.
    #[test]
    fn a_store_from_an_alo_os_this_is_not_is_refused() {
        // With an account this reader would happily hold: the refusal has to
        // be about the number, not about a line further down.
        let newer = adas_machine_written_down().replace("format = 1", "format = 2");
        assert!(matches!(
            Accounts::read(&newer),
            Err(NotKept::AnotherFormat { format: 2 })
        ));
    }

    /// **A password typed into the store raw is refused, naming the
    /// account** — the hand-edit that really happens.
    #[test]
    fn a_password_written_raw_is_refused() {
        let raw = "format = 1\n\n[[account]]\nname = \"ada\"\nuid = 1000\npassword = \"hunter2\"\n";
        assert!(matches!(
            Accounts::read(raw),
            Err(NotKept::NotAHash { name }) if name == "ada"
        ));
    }

    /// **A file holding one name twice is refused whole**, rather than read
    /// as whichever of the two came first.
    #[test]
    fn a_store_naming_somebody_twice_is_refused() {
        let text = adas_machine_written_down();
        let account_table = text
            .split("[[account]]")
            .nth(1)
            .map(|rest| format!("[[account]]{rest}"))
            .unwrap();
        let doubled = text.clone() + "\n" + &account_table;
        assert!(matches!(
            Accounts::read(&doubled),
            Err(NotKept::NotAnAccount(NotCreated::TakenName { .. }))
        ));
    }

    /// **No password ever appears in the file** — only hashes do, and the
    /// test would catch a serializer that wrote the wrong field.
    #[test]
    fn what_is_written_holds_the_hash_and_never_the_password() {
        let text = adas_machine_written_down();
        assert!(!text.contains("correct horse battery staple"));
        assert!(text.contains("$argon2id$"));
    }
}
