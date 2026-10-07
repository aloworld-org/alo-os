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

use alo_image::{
    Image, NO_PARTITIONER, ROOT, THE_AGENT, THE_DOOR, THE_IMAGE, THE_LOADER, THE_ONLY_TOOL,
    THE_OPENER, THE_RUNTIMES_BINARY, THE_SERVER, THE_STORE_IS_AT, everything_wrong_with,
};

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

/// **Every login and group this image declares is one its own build asserts.**
///
/// The number in a `sysusers.d` line is a request rather than a declaration: on
/// the pinned base `systemd-sysusers` answers a number somebody else has by
/// taking a different one, or by putting the login into the group that already
/// holds it — which is how alo OS's agent was put into `systemd-resolve`'s group
/// the first time this recipe was built (`docs/quirks.md`). The `test` lines at
/// the foot of `image/Containerfile` are where a request becomes a fact, and
/// this is what keeps the two files from drifting apart: a login added here and
/// asserted nowhere is one whose number the machine may not have.
///
/// Written after building the recipe, because building it is what shows that
/// those assertions are the only thing standing between a green build and a
/// machine whose description names a number a login does not hold.
#[test]
fn the_build_holds_every_login_to_the_number_the_image_declares() {
    let image = the_image();
    let asserted = image.asserted();

    for (login, number) in [
        ("alo", 1000),
        ("alo-agent", 60989),
        ("alo-greeter", 60990),
        ("alo-model", 60991),
    ] {
        assert_eq!(image.login_called(login), Some(number));
        assert_eq!(
            asserted.login_called(login),
            Some(number),
            "the recipe does not hold `{login}` to {number} after `systemd-sysusers` has run"
        );
    }
    for (group, number) in [
        ("alo-agent", 60989),
        ("alo-greeter", 60990),
        ("alo-model", 60991),
    ] {
        assert_eq!(image.group_called(group), Some(number));
        assert_eq!(
            asserted.group_called(group),
            Some(number),
            "the recipe does not hold group `{group}` to {number}"
        );
    }
    assert!(
        asserted.puts("alo", "alo-agent"),
        "the membership is what lets the socket be handed over at all, and it is the one the \
         first build of this image got silently wrong"
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
        "the per-person directory is made for one session and taken away with it, so `tmpfiles.d` \
         is the wrong maker of it: that file runs at boot, and a daemon that stopped once would \
         never get its door back"
    );
}

/// **The agent service can make the two things it is not given.** It holds no
/// capability (ADR 0018) and is not root (ADR 0001 §2), so a turn's control
/// group and the person's own door both depend on whoever starts it having
/// arranged them — and the first image to boot proved that by stopping thirty
/// milliseconds in, on the first of the two.
#[test]
fn the_agent_service_can_make_a_turn_and_a_door() {
    let image = the_image();

    assert!(
        image.agent().delegates_control_groups(),
        "a turn is a control group under the service's own, and a service running as a person \
         can only make one where systemd has handed the subtree over"
    );
    assert_eq!(
        image.agent().runtime_directories(),
        vec![format!("alo/{}", image.description().person())],
        "every person's door goes in a directory that is 0755 root:root, so the person's own \
         daemon cannot make its name there and something that is root must"
    );
    assert_eq!(image.agent().runtime_directory_mode(), Some("0750"));
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

/// The processes the units start are the binaries the image installs, at the
/// paths the Containerfile installs them to.
///
/// Three of them are built here and the fourth is the pinned runtime the recipe
/// fetches (ADR 0006, ADR 0025) — which is why the fourth is checked against
/// where `crate::runtime` says that artefact lands rather than against a fifth
/// spelling of it.
#[test]
fn the_units_start_the_binaries_the_image_installs() {
    let image = the_image();

    assert_eq!(image.loader().runs(), "/usr/libexec/alo-boundaryd");
    assert_eq!(image.agent().runs(), "/usr/bin/alo-agentd");
    assert_eq!(image.opener().runs(), "/usr/libexec/alo-sessiond");
    assert!(
        image.server().runs().starts_with(THE_RUNTIMES_BINARY),
        "{THE_SERVER} starts `{}`, which is not the runtime this image carries",
        image.server().runs()
    );
}

/// **The second privileged component alo OS has holds no capability at all.**
///
/// ADR 0024 accepted a second privileged component beside ADR 0018's loader,
/// and priced it honestly: it takes a number that has already been
/// authenticated, asks `systemd-logind` to open that person's session, and can
/// do nothing else. This is what that costs the machine, read off the unit —
/// root, because `logind` decides `CreateSession` on a uid and the measurement
/// in `docs/quirks.md` says so on two systemds; the greeter's group, because
/// the door is handed to whatever group it is in; and both capability lines
/// present and empty, because a root process that never mentions capabilities
/// keeps every one of them.
#[test]
fn the_opener_is_root_and_holds_nothing_at_all() {
    let image = the_image();

    assert_eq!(image.opener().called(), THE_OPENER);
    assert_eq!(image.opener().as_login(), Some(ROOT));
    assert_eq!(image.opener().in_group(), Some("alo-greeter"));
    assert!(
        image.opener().holds_nothing(),
        "the opener holds {:?} and is given {:?}",
        image.opener().bounded_to(),
        image.opener().given()
    );
}

/// **The sign-in door is the greeter's, and the greeter is nobody else.**
///
/// `alo-sessiond` hands its door to its own group and refuses every caller the
/// kernel says is in another one, so this `Group=` line is the whole of who may
/// ask this machine for a session. A greeter that were the person or the agent
/// would be a door those could knock on, and the image makes the login rather
/// than leaving it to whatever a surface is configured with later.
#[test]
fn the_greeter_is_a_login_of_its_own_and_the_door_is_its_group() {
    let image = the_image();
    let greeter = image.group_called("alo-greeter");

    assert_eq!(greeter, Some(60990));
    assert_eq!(image.login_called("alo-greeter"), Some(60990));
    assert_ne!(greeter, Some(image.description().person()));
    assert_ne!(greeter, Some(image.description().agent()));
    assert_ne!(greeter, Some(image.description().group()));
    assert_eq!(
        image.opener().runtime_directories(),
        vec!["alo-sessiond"],
        "the unit makes a directory that is not where alo-sessiond opens its door"
    );
    assert_eq!(image.opener().runtime_directory_mode(), Some("0750"));
}

/// **The model runtime the machine arrives with is aboard, pinned and
/// verified.** ADR 0025 made *the local model is what the machine arrives
/// ready to run* the promise, and ADR 0006 said how the runtime gets there: a
/// pinned upstream artefact, its version written where the other pins are.
/// This is that, read off the recipe rather than off an ADR. The weights it
/// loads are the test below; what is still not here is a unit that starts it,
/// because who that process runs as and what it may reach are a decision of
/// their own (ADR 0019).
#[test]
fn the_model_runtime_is_aboard_pinned_and_verified() {
    let image = the_image();
    let runtime = image.runtime();

    assert!(
        runtime.lands_its_binary() && runtime.lands_its_libraries(),
        "the recipe does not land the runtime: {runtime:?}"
    );
    assert!(
        runtime.is_pinned(),
        "the runtime's version floats: {:?}",
        runtime.version()
    );
    assert!(
        runtime.is_verified(),
        "the runtime's artefact is not held to a digest the build checks: {:?}",
        runtime.digest()
    );
}

/// **The image carries no weights at all**, which since
/// [ADR 0095](../../../docs/decisions/0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
/// is what a correct image does.
///
/// Two tests stood here. One held the weights to being aboard, pinned,
/// verified and the catalogue's own recommendation; the other held the store to
/// carrying them exactly once. Both were about 4.87 GiB that no longer ships: a
/// person brings weights they already have, uses a provider, or works without
/// one, and a model we chose would be a model chosen for them.
///
/// What replaces them is the opposite question, asked of the real recipe,
/// because weights can return in a single `COPY` line and nobody reviewing a
/// recipe notices a layer getting bigger.
#[test]
fn the_image_carries_no_weights() {
    let image = the_image();
    assert_eq!(
        image.lands_weights(),
        Vec::<String>::new(),
        "the recipe lands model weights, and ADR 0095 says the release carries none"
    );
}

/// **It looks for the model where the weights landed, and answers where this
/// machine knocks.**
///
/// Two `Environment=` lines and each is a different kind of wrong. The store is
/// the quiet one: the runtime's own default is a home directory this image does
/// not make, so a machine carrying the model would report nothing installed and
/// look from the outside exactly like a machine nobody put a model on.
///
/// The address is the expensive one. `alo-models` knocks at exactly one place
/// (ADR 0019) and the address is read from that crate rather than spelled here,
/// so the unit and the knock cannot drift apart — and an address naming every
/// interface rather than the loopback one would offer this machine's model to
/// whatever network it is plugged into, with nothing on the egress indicator,
/// because nothing left.
#[test]
fn the_model_service_is_pointed_at_a_writable_store_and_at_the_loopback_address() {
    let image = the_image();
    let stated = image.server().environment();

    let store: Vec<&str> = assigned(&stated, "OLLAMA_MODELS");
    assert_eq!(
        store,
        vec![THE_STORE_IS_AT.trim_end_matches('/')],
        "the model service is not pointed at the store a person's own weights go in"
    );
    assert!(
        alo_image::a_machine_may_write(THE_STORE_IS_AT),
        "the store is somewhere the machine cannot write, so nobody can bring weights"
    );

    let address: Vec<&str> = assigned(&stated, "OLLAMA_HOST");
    assert_eq!(
        address,
        vec![alo_models::ollama::DEFAULT_ENDPOINT],
        "the model service does not answer where this machine looks for a runtime"
    );
}

/// **And it reaches nothing off this machine — enforced, not asserted.**
///
/// Law 1: with a local model a working day produces zero inference egress,
/// measured at the network boundary, and the one process holding the model is
/// the one that has to be silent for that sentence to be true. systemd's IP
/// access list is a kernel-side filter on this unit's own control group, so an
/// update check, a telemetry call or a registry pull does not fail politely; it
/// does not leave.
///
/// **This reads a setting, and the setting has been watched working — once,
/// under the image's own systemd in a container, not at a boot.** On
/// 2026-09-12 the login this unit runs as attempted sixteen packets to its
/// publisher's port 443 and none reached the host side of the container's
/// bridge, while an unfiltered process in the same container was answered
/// (`docs/quirks.md`). A boot is still owed, and
/// `docs/autonomy/evidence-it-boots-and-the-agent-acts.md` is where that stays.
#[test]
fn the_model_service_may_reach_nothing_off_this_machine() {
    let image = the_image();

    assert_eq!(
        image.server().may_reach(),
        vec!["localhost"],
        "the model service is allowed somewhere beyond this machine"
    );
    assert!(
        image.server().may_not_reach().contains(&"any"),
        "an allow list with no deny under it filters nothing at all"
    );
}

/// **And it does not ask its publisher in the first place.** The filter above
/// is the lock that cannot be switched off by a runtime update; this is the
/// runtime's own, measured on 2026-09-12 to stop both start-up requests from
/// being made at all — so a machine's journal carries no line naming the
/// publisher and no retry every five minutes. Configuration, never a patch
/// (ADR 0011), and exactly one assignment, because two would be a unit where
/// the second silently wins.
#[test]
fn the_model_service_does_not_ask_its_publisher() {
    let image = the_image();
    let stated = image.server().environment();

    let switched: Vec<&str> = assigned(&stated, "OLLAMA_NO_CLOUD");
    assert_eq!(
        switched,
        vec!["1"],
        "the model service does not set the runtime's own switch beside the filter"
    );
}

/// Every value a unit's environment gives this variable, in the order it gives
/// them — a list, because *exactly one assignment* is the question.
fn assigned<'a>(stated: &[&'a str], variable: &str) -> Vec<&'a str> {
    stated
        .iter()
        .filter_map(|pair| pair.strip_prefix(variable))
        .filter_map(|rest| rest.strip_prefix('='))
        .collect()
}

/// **This image says what disk a machine boots from, and it is written by the
/// base's own tool.** An image is not a disk, and every promise in
/// `docs/autonomy/evidence-it-boots-and-the-agent-acts.md` that said *no machine has ever* was
/// waiting on nothing more exotic than that. The tool is not pinned separately
/// because it is already pinned: `bootc install to-disk` is run out of the base
/// image, so `THE_BASE`'s digest is the version of the partitioner.
#[test]
fn the_image_says_what_disk_it_becomes() {
    let image = the_image();
    let disk = image.disk();

    assert_eq!(disk.tool(), Some(THE_ONLY_TOOL));
    assert_eq!(disk.firmware(), Some("uefi"));
    assert_eq!(disk.file(), Some("alo-os.raw"));
    assert!(
        disk.on_a_pinned_base(),
        "the tool comes out of the base, so a base on a tag that moves is a partitioner nobody \
         chose"
    );
    assert!(disk.written_by_the_base());
    assert_eq!(
        disk.laid_out_by_hand(),
        None,
        "engines are configured and never written in, and a partition table of our own is that \
         rule broken where only somebody else's machine would find out"
    );
}

/// **The document a person follows says what the recipe says**, including the
/// firmware — which is the sharp one, because a Hyper-V generation 1 machine
/// pointed at a UEFI disk finds nothing at all to boot, and the difference is
/// one digit in one line of prose.
#[test]
fn the_document_tells_a_person_the_disk_this_image_really_makes() {
    let image = the_image();
    let document = image.document();

    assert_eq!(document.says("tool"), image.disk().tool());
    assert_eq!(document.says("firmware"), image.disk().firmware());
    assert_eq!(document.says("disk"), image.disk().file());
    assert_eq!(
        document.says("generation"),
        Some("2"),
        "generation 2 is the UEFI one; generation 1 is the BIOS one"
    );
    assert!(
        document.gives_the_command(THE_ONLY_TOOL),
        "naming a tool is not telling anybody how to run it, and one documented command is what \
         that document is for"
    );
    for by in NO_PARTITIONER {
        assert!(
            !document.names(by),
            "the document tells somebody to run {by}"
        );
    }
}

/// **And it says what a virtual machine cannot show.** A virtual GPU is not
/// *the GPU works on first boot* and tame virtual firmware is not a certified
/// machine's, so this paragraph is what stands between a disk booting in
/// Hyper-V and somebody quoting that as the hardware acceptance in phase 8.
#[test]
fn the_document_says_what_a_virtual_machine_cannot_show() {
    let image = the_image();
    let document = image.document();
    let heading = "What a virtual machine cannot show";

    assert!(document.has_a_section(heading));
    for about in ["GPU", "firmware", "hardware acceptance"] {
        assert!(
            document.names_under(heading, about),
            "the document does not say what a virtual machine cannot show about {about}"
        );
    }
}
