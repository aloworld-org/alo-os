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
    THE_OPENER, THE_RUNTIMES_BINARY, THE_SERVER, THE_WEIGHTS, everything_wrong_with,
};
use alo_models::{Catalogue, OnCpu};

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

/// **The weights a machine arrives with are aboard, pinned, checked before
/// anything reads them, and a model somebody measured.**
///
/// ADR 0025's expensive half: a model on the disk of every machine we ship,
/// sized for that machine (ADR 0007). It is carried on the image rather than
/// fetched at setup, because a machine that fetches at setup has not arrived
/// ready when it is offline at setup.
///
/// The catalogue is the authority for which model, and this test reads both:
/// the entry has been measured by `alo-driving`, its quantisation and artefact
/// are the ones the recipe fetches and imports, it fits the ordinary business
/// laptop `docs/hardware.md` certifies, and its licence permits commercial use
/// outright — which matters here and nowhere else, because carrying weights in
/// an image is redistributing them.
///
/// **It does not say the machine can be given the agent.** Nothing in the
/// catalogue clears `Driving::Reliably` yet, this entry included; what is shown
/// is a machine that arrives with a model on its disk rather than one that
/// arrives with an agent that works.
#[test]
fn the_weights_a_machine_arrives_with_are_aboard_pinned_and_measured() {
    let image = the_image();
    let weights = image.weights();

    assert!(
        weights.land(),
        "the recipe lands no weights at {THE_WEIGHTS}: {weights:?}"
    );
    assert!(
        weights.is_pinned(),
        "the weights are fetched from a moving name: {:?}",
        weights.from()
    );
    assert!(
        weights.is_verified(),
        "the weights are not held to a digest checked before anything reads them: {:?}",
        weights.digest()
    );

    let named = match weights.model() {
        Some(model) => model,
        None => panic!("the recipe carries weights and does not say which model: {weights:?}"),
    };
    let catalogue = match Catalogue::built_in() {
        Ok(catalogue) => catalogue,
        Err(why) => panic!("the catalogue this system ships did not read: {why}"),
    };
    let entry = match catalogue.get(named) {
        Some(entry) => entry,
        None => panic!("the image carries `{named}`, which the catalogue does not have"),
    };

    assert!(
        entry.drives_verbs.has_been_measured(),
        "`{named}` was never put to alo-driving"
    );
    assert_eq!(
        entry.quantised_at(),
        weights.quantisation().zip(weights.artefact()),
        "`{named}` is carried as something the catalogue does not state"
    );
    assert!(
        entry.min_ram_gb <= 16.0 && entry.on_cpu != OnCpu::Slow,
        "`{named}` needs {} GB and is graded {:?} with no card, and the machine this image is \
         sized for has 16 GB and no card",
        entry.min_ram_gb,
        entry.on_cpu
    );
    assert!(
        entry.safe_default_for_business(),
        "`{named}` is under `{}`, and an image carrying weights hands their terms to everybody \
         who receives it",
        entry.licence.name
    );
}

/// **Something on this machine serves the model it arrived with, and it is a
/// login of its own that holds nothing.**
///
/// Until this unit existed the image carried a runtime and 2.23 GiB of weights
/// and started neither, so `alo-models` knocked at the loopback address and got
/// the answer a machine with no model at all gives. The four decisions in the
/// unit are read back here off the real file.
///
/// Not the person, whose session comes and goes; not the agent, which ADR 0001
/// §2 and §5 keep authority and identity away from; and no capability, because
/// serving a model needs none — ADR 0018's argument said in the third place it
/// has to be said.
#[test]
fn the_model_is_served_by_a_login_of_its_own_that_holds_nothing() {
    let image = the_image();
    let server = image.server();

    let as_login = match server.as_login() {
        Some(login) => login,
        None => panic!("{THE_SERVER} does not say which login it runs as"),
    };
    let number = match image.login_called(as_login) {
        Some(number) => number,
        None => panic!("{THE_SERVER} runs as `{as_login}`, which this image does not make"),
    };
    assert_ne!(number, image.description().person());
    assert_ne!(number, image.description().agent());

    let group = match server.in_group() {
        Some(group) => group,
        None => panic!("{THE_SERVER} does not say which group it runs in"),
    };
    assert!(
        image.group_called(group).is_some(),
        "{THE_SERVER} runs in `{group}`, which this image does not make"
    );
    assert_ne!(Some(group), image.loader().in_group(), "the agent's group");
    assert_ne!(Some(group), image.agent().in_group(), "the person's group");
    assert_ne!(
        Some(group),
        image.opener().in_group(),
        "the greeter's group"
    );

    assert!(
        server.holds_nothing(),
        "serving a model needs no capability, and both lines have to say so: {server:?}"
    );
    assert!(
        server.wanted_by().contains(&"multi-user.target"),
        "nothing starts {THE_SERVER}, which is the state the image shipped in until it existed"
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
fn the_model_service_is_pointed_at_the_weights_and_at_the_loopback_address() {
    let image = the_image();
    let stated = image.server().environment();

    let store: Vec<&str> = assigned(&stated, "OLLAMA_MODELS");
    assert_eq!(
        store,
        vec![THE_WEIGHTS.trim_end_matches('/')],
        "the model service is not pointed at the directory the weights landed in"
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
/// **This reads a setting. It is not a machine anybody watched.** No packet
/// counter has been put beside this image, because nothing in this lane has
/// booted it — `docs/autonomy/v0-01-evidence.md` is where that stays owed.
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
/// `docs/autonomy/v0-01-evidence.md` that said *no machine has ever* was
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
