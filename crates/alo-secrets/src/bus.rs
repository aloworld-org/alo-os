//! Where the person's session bus is, and whether it is theirs.

use std::os::unix::fs::FileTypeExt as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use crate::refusing::NotStored;

/// Where `logind` puts a person's runtime directory.
///
/// Named rather than written inline because it is the one path in this crate
/// that is a fact about the machine rather than about alo OS, and because a
/// reader looking for *what does this open* should find it in one line.
pub const WHERE_SESSIONS_ARE: &str = "/run/user";

/// The person's session bus, as this process may reach it.
///
/// Holds a path that has been **checked**, so that holding one is the fact that
/// there is a bus at it and that it is the person's. An unchecked path is a
/// `PathBuf` and this is not one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBus {
    /// Where it is.
    at: PathBuf,

    /// Whose it is, which is who this was derived for.
    whose: u32,
}

impl TheBus {
    /// Where the bus for this uid would be, whether or not it is there.
    ///
    /// **Takes a uid and nothing else.** There is no argument an environment
    /// variable could arrive through, which is how this crate keeps
    /// `DBUS_SESSION_BUS_ADDRESS` out of a decision the daemon makes on the
    /// person's behalf — a property of the signature rather than a promise in a
    /// comment.
    #[must_use]
    pub fn of(uid: u32) -> PathBuf {
        Path::new(WHERE_SESSIONS_ARE)
            .join(uid.to_string())
            .join("bus")
    }

    /// The bus this process may reach, checked.
    ///
    /// The uid is the one the kernel says this process is running as. On a
    /// machine that has booted, `alo-agentd` is `User=alo` — so this is the
    /// person's own bus, and it is the one directory `/run/user/<uid>` being
    /// `0700` and theirs does *not* keep this process out of.
    ///
    /// # Errors
    /// [`NotStored::Unavailable`] when there is no bus this process may use:
    /// nobody has signed in, nothing is listening at that name, or what is there
    /// belongs to somebody else.
    pub fn of_this_process() -> Result<Self, NotStored> {
        Self::found(rustix::process::getuid().as_raw())
    }

    /// The same, for a uid a caller names.
    ///
    /// # Errors
    /// [`NotStored::Unavailable`], with the three checks in `lib.rs`.
    pub fn found(uid: u32) -> Result<Self, NotStored> {
        Self::at(&Self::of(uid), uid)
    }

    /// The same, at a path a caller names — which is how the three checks are
    /// tested against sockets a test owns rather than against whatever
    /// `/run/user` happens to hold on the machine running them.
    ///
    /// # Errors
    /// [`NotStored::Unavailable`] when it is not there, is not a socket, or is
    /// not this uid's.
    pub fn at(at: &Path, whose: u32) -> Result<Self, NotStored> {
        // `symlink_metadata`, not `metadata`: a symlink at this name pointing
        // at somebody else's socket would otherwise be followed and then
        // checked at the far end, which answers a question about the wrong
        // file. What is wanted is *this name is a socket of theirs*.
        // **And it has to be sayable as an address**, which is checked before
        // anything is opened rather than after. `libsecret` cannot be told
        // which bus to use at all — no API of it accepts a connection — so the
        // client that eventually does is given [`TheBus::as_an_address`] and
        // nothing else, and a path that does not survive that grammar would
        // quietly become a different address.
        if !can_be_said_as_an_address(at) {
            return Err(NotStored::Unavailable);
        }
        let it = std::fs::symlink_metadata(at).map_err(|_| NotStored::Unavailable)?;
        if !it.file_type().is_socket() {
            return Err(NotStored::Unavailable);
        }
        if it.uid() != whose {
            return Err(NotStored::Unavailable);
        }
        Ok(Self {
            at: at.to_owned(),
            whose,
        })
    }

    /// Where it is.
    #[must_use]
    pub fn at_path(&self) -> &Path {
        &self.at
    }

    /// Whose it is.
    #[must_use]
    pub const fn whose(&self) -> u32 {
        self.whose
    }

    /// The address a D-Bus client is given for it.
    ///
    /// The one shape this crate hands outward, so that whatever binds to
    /// `libsecret` later is given **this** rather than left to whatever
    /// `g_bus_get` would have chosen from the environment. ADR 0022's
    /// *verify the bus actually selected* begins here: a client handed an
    /// address cannot silently use another one.
    #[must_use]
    pub fn as_an_address(&self) -> String {
        format!("unix:path={}", self.at.display())
    }
}

/// Whether this path can be written as a D-Bus address without becoming a
/// different one.
///
/// D-Bus address grammar separates key–value pairs with `,` and whole addresses
/// with `;`. A socket path containing either would be **parsed as more than it
/// is** — `unix:path=/run/user/0/b,us` is a `path` of `/run/user/0/b` and a key
/// called `us`, and a `;` offers the client a second address to fall back to.
/// Neither is a path this could open, and both are a client connecting somewhere
/// this crate did not choose.
///
/// It must also be absolute: a relative path is resolved against whatever
/// directory the process happens to be in, which is not a decision anybody made.
///
/// Refused before the socket is looked at, because what is being refused is the
/// *name*, and looking first would answer a question about a file that is not
/// the one the client would open.
fn can_be_said_as_an_address(at: &Path) -> bool {
    let Some(said) = at.to_str() else {
        // A path that is not UTF-8 has no spelling in a D-Bus address, which is
        // text. Nothing on a machine `logind` made has one.
        return false;
    };
    at.is_absolute() && !said.contains(',') && !said.contains(';')
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A directory nothing else in this run is using.
    fn a_place_of_our_own(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-secrets-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).expect("a temporary directory can be made");
        at
    }

    /// **The path is the uid's, and a uid is all it takes.**
    ///
    /// `TheBus::of` has one argument and it is not a string somebody set. On the
    /// machine these tests run on `DBUS_SESSION_BUS_ADDRESS` really is set — to
    /// root's bus — and it reaches none of this, because there is no parameter
    /// it could arrive through.
    #[test]
    fn where_a_bus_is_comes_from_a_uid_and_nothing_else() {
        assert_eq!(TheBus::of(0), Path::new("/run/user/0/bus"));
        assert_eq!(TheBus::of(1000), Path::new("/run/user/1000/bus"));
        assert_eq!(TheBus::of(60989), Path::new("/run/user/60989/bus"));

        // Whatever the environment says, a different uid is a different path.
        // This is the assertion that would fail on the day somebody reads a
        // variable here, because the variable on this machine names uid 0.
        assert_ne!(TheBus::of(1000), TheBus::of(0));
    }

    /// **Before anybody signs in there is no bus**, and that is
    /// `Unavailable` rather than an error somebody has to interpret.
    #[test]
    fn a_uid_that_has_not_signed_in_has_no_bus() {
        let nobody = a_place_of_our_own("nobody").join("bus");
        assert_eq!(TheBus::at(&nobody, 0), Err(NotStored::Unavailable));

        // And a uid nobody on this machine has, through the real path.
        assert_eq!(TheBus::found(4_294_967), Err(NotStored::Unavailable));
    }

    /// **A file that is not a socket is not a bus.**
    ///
    /// A regular file at that name is a machine that is wrong, and opening it
    /// would be this crate believing a name rather than checking one.
    #[test]
    fn a_name_that_is_not_a_socket_is_not_a_bus() {
        let at = a_place_of_our_own("regular").join("bus");
        std::fs::write(&at, b"not a bus").expect("a file can be written");
        assert_eq!(TheBus::at(&at, 0), Err(NotStored::Unavailable));
    }

    /// **A socket somebody else owns is refused**, which is the cross-user
    /// check: `logind` makes `/run/user/<uid>` the person's, so a bus at that
    /// name owned by another login is a machine to stop on rather than connect
    /// through.
    ///
    /// The socket here is owned by whoever runs the tests; asking whether it
    /// belongs to a *different* uid is the same question from the other side,
    /// and it is the one a root test box can actually answer.
    #[test]
    fn a_bus_that_belongs_to_somebody_else_is_refused() {
        let at = a_place_of_our_own("theirs").join("bus");
        let listening = std::os::unix::net::UnixListener::bind(&at).expect("a socket of our own");

        let ours = rustix::process::getuid().as_raw();
        let found = TheBus::at(&at, ours).expect("a socket of ours is ours");
        assert_eq!(found.whose(), ours);
        assert_eq!(found.at_path(), at);

        // The same socket, asked whether it belongs to anybody else.
        let somebody_else = ours.wrapping_add(1);
        assert_eq!(TheBus::at(&at, somebody_else), Err(NotStored::Unavailable));

        drop(listening);
    }

    /// **A symlink at that name is not followed and then checked**, which would
    /// answer a question about the wrong file. What is asked is whether *this
    /// name* is a socket of theirs.
    #[test]
    fn a_symlink_pointing_at_a_socket_is_not_a_bus() {
        let place = a_place_of_our_own("pointing");
        let real = place.join("bus");
        let listening = std::os::unix::net::UnixListener::bind(&real).expect("a socket of our own");
        let pointed = place.join("pointed");
        std::os::unix::fs::symlink(&real, &pointed).expect("a symlink can be made");

        let ours = rustix::process::getuid().as_raw();
        assert!(TheBus::at(&real, ours).is_ok());
        assert_eq!(TheBus::at(&pointed, ours), Err(NotStored::Unavailable));

        drop(listening);
    }

    /// **A path that would say more than itself as an address is refused.**
    ///
    /// D-Bus separates key–value pairs with `,` and addresses with `;`, so
    /// `unix:path=/tmp/b,us` is a path of `/tmp/b` and a key called `us`, and a
    /// `;` hands the client a second address to try. Either is a client
    /// connecting somewhere this crate did not choose — which matters because
    /// **libsecret cannot be told which bus to use at all**, so the address is
    /// the only control there is.
    ///
    /// Refused on the name, before the socket is looked at.
    #[test]
    fn a_path_that_would_become_a_different_address_is_refused() {
        let place = a_place_of_our_own("ambiguous");
        let ours = rustix::process::getuid().as_raw();

        for named in ["b,us", "b;us"] {
            let at = place.join(named);
            let listening =
                std::os::unix::net::UnixListener::bind(&at).expect("a socket of our own");
            assert_eq!(
                TheBus::at(&at, ours),
                Err(NotStored::Unavailable),
                "{named} was accepted, and as an address it is not itself"
            );
            drop(listening);
        }

        // A relative name is refused too: it would be resolved against whatever
        // directory the process happens to be in.
        assert!(!can_be_said_as_an_address(Path::new("run/user/0/bus")));
        assert!(can_be_said_as_an_address(Path::new("/run/user/0/bus")));
    }

    /// **What a client is handed is an address for this bus**, so that whatever
    /// binds to a keyring later cannot quietly connect somewhere else.
    #[test]
    fn a_client_is_handed_this_bus_rather_than_left_to_choose() {
        let at = a_place_of_our_own("address").join("bus");
        let listening = std::os::unix::net::UnixListener::bind(&at).expect("a socket of our own");
        let ours = rustix::process::getuid().as_raw();

        let found = TheBus::at(&at, ours).expect("a socket of ours is ours");
        assert_eq!(found.as_an_address(), format!("unix:path={}", at.display()));
        drop(listening);
    }
}
