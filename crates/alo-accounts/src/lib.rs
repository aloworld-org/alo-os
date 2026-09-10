//! The local account that needs no tenant, and the session a sign-in opens.
//!
//! v0.01's exit gate opens with *sign in*, and this crate is the account and
//! the authentication behind that word: a local account, against the
//! machine's own store — **no identity provider, no tenant, no network**.
//! Those are the other half of phase 4 (`docs/features.md`, *Sign-in with an
//! alo identity*) and are deliberately absent here rather than stubbed.
//!
//! The entry *surface* — the screen a person types into — is the compositor
//! lane's and lives in `crates/alo-shell`; this crate is what that surface
//! calls. Nothing here draws, prompts, or reads a keyboard.
//!
//! # The three promises, and where each is kept
//!
//! **An account is created and authenticated against the machine's own
//! store.** [`Accounts`] holds them; `keeping` (Unix only) puts the store at
//! [`THE_ACCOUNTS`] on the disk, believed under the same ownership rules as
//! the machine description, and replaced whole or not at all.
//!
//! **A wrong password is refused in words, and is not distinguishable by
//! timing from an unknown name.** [`Accounts::signs_in`] answers both with
//! the one value [`NotSignedIn::Refused`], whose sentence
//! ([`NotSignedIn::said`]) is one sentence for both facts — and an unknown
//! name is verified against a decoy hash whose parameters match every real
//! one, so the two refusals cost the same work. `store.rs` says why the
//! obvious shortcut is an enumeration attack; `tests/a_person_signs_in.rs`
//! measures it.
//!
//! **The session carries the uid `alo-agentd` is told about.** [`Session`]
//! can only be constructed by [`Session::opened`], which takes the
//! authenticated account *and* the described person's uid
//! (`/etc/alo/agentd.toml`, `[logins].person`) and refuses to exist where
//! they differ — so the machine description and the running session cannot
//! disagree about who is signed in.
//!
//! # Map
//!
//! | | |
//! |---|---|
//! | `account` | One account: name, number, and the hash of a password |
//! | `hashing` | Argon2id, and the decoy that keeps an unknown name honest |
//! | `store` | The accounts, creation, and the sign-in they answer |
//! | `written` | The store as TOML, read back believed or refused |
//! | `place` | Where the store is, which every host can read |
//! | `keeping` | The file on the disk, and who may have written it |
//! | `session` | Who signed in, and the uid agreement with the description |
//! | `refusing` | Every refusal, and who each is written for |
//! | `words` | The two sentences a person reads, named for translation |

mod account;
mod hashing;
#[cfg(unix)]
mod keeping;
mod place;
mod refusing;
mod session;
mod store;
mod words;
mod written;

pub use account::Account;
#[cfg(unix)]
pub use keeping::{found, kept};
pub use place::THE_ACCOUNTS;
pub use refusing::{CannotHash, NotCreated, NotKept, NotSignedIn};
pub use session::{Session, SignedIn};
pub use store::Accounts;
pub use words::{
    EVERY_WORD, NOT_SIGNED_IN, NOT_THIS_MACHINES_PERSON, Word, WordsError, accounts_words,
    declare_into,
};
pub use written::THE_FORMAT;
