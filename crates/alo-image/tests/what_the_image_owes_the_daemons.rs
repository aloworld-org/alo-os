//! The image this repository ships, held to the sentences the rest of it makes.
//!
//! `crate::checking`'s own tests break one line of a copy of this image and ask
//! whether that is noticed — which is the half that proves the checks work.
//! This is the other half, and it is written against the real files rather than
//! against a copy: each test below is one promise in an ADR or a contract, read
//! back out of `image/`, so that a change to the image which nobody meant fails
//! beside the sentence it made untrue.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::{Image, ROOT, THE_AGENT, THE_DOOR, THE_IMAGE, THE_LOADER, everything_wrong_with};

/// The image this repository ships.
///
/// A `panic!` on a missing image is the failure being reported: every test here
/// is about a file in it.
fn the_image() -> Image {
    match Image::at(Path::new(THE_IMAGE)) {
        Ok(image) => image,
        Err(why) => panic!("the image this repository ships did not read: {why}"),
    }
}

/// **Everything the image says agrees with everything else it says.**
///
/// The whole crate in one line. Every check has a twin in `checking.rs` that
/// breaks a copy of this image and is caught, so a green here is a green that
/// has been shown to be able to go red.
#[test]
fn the_image_agrees_with_itself() {
    let wrong = everything_wrong_with(&the_image());
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// **The boundary is on the kernel before the agent service exists.** ADR 0015:
/// a turn that cannot be bounded does not run, and ADR 0018 moved the loading
/// into a component of its own that runs at boot.
#[test]
fn the_loader_runs_before_the_agent_service_and_stays() {
    let image = the_image();

    assert!(image.agent().needs().contains(&THE_LOADER));
    assert!(image.agent().after().contains(&THE_LOADER));
    assert!(image.loader().before().contains(&THE_AGENT));
    assert_eq!(image.loader().kind(), Some("oneshot"));
    assert!(
        image.loader().stays_after_exiting(),
        "the boundary outlives the process that loaded it, and the unit has to say so"
    );
}

/// **The loader is the only privileged component, and it holds two things.**
/// What makes one privileged component acceptable is the size of what it is
/// trusted with; this is that, read off the unit rather than off an ADR.
#[test]
fn the_loader_holds_exactly_what_adr_0018_gave_it() {
    let image = the_image();

    assert_eq!(image.loader().as_login(), Some(ROOT));
    let mut held = image.loader().bounded_to();
    held.sort_unstable();
    assert_eq!(held, vec!["CAP_BPF", "CAP_SYS_ADMIN"]);
    assert!(
        image.loader().given().is_empty(),
        "it is root, so it needs nothing made ambient"
    );
}

/// **The agent service holds nothing, and says so.** ADR 0001 §2, and the shape
/// the mistake would really arrive in: not a privileged daemon, one directive
/// added to an ordinary unit to make something work.
#[test]
fn the_agent_service_holds_nothing_at_all() {
    let image = the_image();

    assert!(image.agent().holds_nothing());
    assert_ne!(image.agent().as_login(), Some(ROOT));
    assert_eq!(
        image.agent().as_login(),
        Some("alo"),
        "it runs as the person, which is the whole of ADR 0001 §5's two sides"
    );
}

/// **The three numbers the socket's two doors are decided by are the accounts
/// this image makes.** Nothing on the wire says who a caller is; `SO_PEERCRED`
/// does, and it is compared against these.
#[test]
fn the_machine_description_names_logins_this_image_makes() {
    let image = the_image();
    let described = image.description();

    assert_eq!(image.login_called("alo"), Some(described.person()));
    assert_eq!(image.login_called("alo-agent"), Some(described.agent()));
    assert_eq!(image.group_called("alo-agent"), Some(described.group()));
    assert_ne!(
        described.person(),
        described.agent(),
        "the side that proposes a change would also be the side that approves it"
    );
    assert!(image.puts("alo", "alo-agent"));
}

/// **The agent's login is out of the range the base allocates from**, which was
/// found by running `systemd-sysusers` against the pinned base rather than by
/// reading anything: 989 — the number every example in this repository uses — is
/// systemd-resolve's group there, and the first build of this image put alo OS's
/// agent into it.
#[test]
fn the_agents_login_cannot_be_taken_by_the_base() {
    let image = the_image();
    let described = image.description();

    assert!(
        described.agent() > 1000 && described.group() > 1000,
        "a system login number is one a base update can allocate from underneath us"
    );
}

/// **ADR 0017's directory is made by the image and is what the ADR says.**
/// `alo-agentd` refuses to make it, and every person's door goes in it — so its
/// mode is not one person's service's to choose.
#[test]
fn the_door_every_person_goes_through_is_the_images() {
    let image = the_image();
    let door = match image.directory_at(Path::new(THE_DOOR)) {
        Some(door) => door,
        None => panic!("nothing in the image makes {THE_DOOR}"),
    };

    assert_eq!(door.mode(), 0o755);
    assert_eq!(door.owner(), ROOT);
    assert_eq!(door.group(), ROOT);
    assert!(
        image.directory_at(Path::new("/run/alo/1000")).is_none(),
        "the per-person directory is the daemon's, made when a session starts and taken away \
         when it ends"
    );
}

/// **The folder the record goes in is made, and it is the person's.** The
/// machine description says the file is made on the first start and the folder
/// above it is not.
#[test]
fn the_record_has_a_folder_and_it_belongs_to_the_person() {
    let image = the_image();
    let folder = match image.description().record_folder() {
        Some(folder) => folder,
        None => panic!("the record path names no folder"),
    };
    let made = match image.directory_at(folder) {
        Some(made) => made,
        None => panic!("nothing in the image makes {}", folder.display()),
    };

    assert_eq!(made.owner(), "alo");
    assert_eq!(made.mode(), 0o700);
}

/// **The image ships no retention rule of its own.** ADR 0004 gives how long a
/// record is kept to the organisation that manages the machine, and a number of
/// days that sounded reasonable is exactly what `CLAUDE.md` refuses to ship.
#[test]
fn the_image_keeps_everything_because_that_is_not_ours_to_decide() {
    assert_eq!(the_image().description().keeping().days(), None);
}

/// **Both units are pulled in at boot.** The failure this catches looks most
/// like success: the file is in the image, `systemctl cat` shows it, and nothing
/// ever starts it.
#[test]
fn both_units_are_started_by_something() {
    let image = the_image();

    assert!(!image.loader().wanted_by().is_empty());
    assert!(!image.agent().wanted_by().is_empty());
}

/// The two processes the units start are the two binaries this repository
/// builds, at the paths the Containerfile installs them to.
#[test]
fn the_units_start_the_binaries_the_image_installs() {
    let image = the_image();

    assert_eq!(image.loader().runs(), "/usr/libexec/alo-boundaryd");
    assert_eq!(image.agent().runs(), "/usr/bin/alo-agentd");
}
