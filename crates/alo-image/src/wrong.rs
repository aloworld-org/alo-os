//! What the image's files say that they cannot all be saying at once.
//!
//! `crate::refusing` is the other half: those are files that would not read at
//! all, and every one of these is a file that reads perfectly and disagrees with
//! another one. That is the failure worth catching here, because it is the one a
//! build cannot see — a Containerfile will happily produce an image whose
//! machine description names a login the image never creates, and the machine
//! boots to a daemon that stops with a sentence about a number.
//!
//! Each variant names one promise made somewhere in `docs/`, and the sentence
//! says which. They are English for `crate::refusing`'s reason: nobody using alo
//! OS ever reads one.

use std::path::PathBuf;

use thiserror::Error;

/// One thing the image's files disagree about.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Wrong {
    /// The agent's service would start on a machine with no boundary.
    #[error(
        "{agent} does not wait for {loader} — ADR 0015 says a turn that cannot be bounded does \
         not run, so a machine whose boundary never loaded must not reach a service that would \
         refuse every turn on it"
    )]
    TheAgentDoesNotWaitForTheBoundary {
        /// The agent's unit.
        agent: String,
        /// The loader's unit.
        loader: String,
    },
    /// The loader's unit does not stay active once the process has gone.
    #[error(
        "{loader} is `Type={kind}`, and the boundary outlives the process that loaded it \
         (ADR 0018) — a unit that went inactive would tell `systemctl status` that this machine \
         has no boundary while it has one"
    )]
    TheLoaderDoesNotStay {
        /// The loader's unit.
        loader: String,
        /// What it says it is, or `-` where it says nothing.
        kind: String,
    },
    /// The loader would run as somebody who cannot load a BPF LSM programme.
    #[error(
        "{loader} runs as `{as_login}`, and only root can load a BPF LSM programme — the loader \
         refuses to start rather than fail inside the verifier"
    )]
    TheLoaderIsNotRoot {
        /// The loader's unit.
        loader: String,
        /// The login it names, or `-` where it names none.
        as_login: String,
    },
    /// The loader would pin the map of turns where the agent cannot write it.
    #[error(
        "{loader} runs in group `{group}`, which is not the group the machine description gives \
         the agent — the map of turns is handed to whatever group this process is in, so this is \
         a boundary loaded for a daemon that can never bound a turn"
    )]
    TheLoaderIsInTheWrongGroup {
        /// The loader's unit.
        loader: String,
        /// The group it names, or `-` where it names none.
        group: String,
    },
    /// The loader holds something ADR 0018 did not give it.
    #[error(
        "{loader} is bounded to {held:?}, and ADR 0018 gives it CAP_BPF and CAP_SYS_ADMIN — the \
         one privileged component alo OS has is acceptable because of how little it is trusted \
         with, and this line is where that is true or is prose"
    )]
    TheLoaderHoldsSomethingElse {
        /// The loader's unit.
        loader: String,
        /// What the unit bounds it to.
        held: Vec<String>,
    },
    /// The agent's service holds a capability, or does not say it holds none.
    #[error(
        "{agent} does not say it holds nothing — ADR 0001 §2 says the agent service never runs \
         with authority the person does not have, and ADR 0018 moved the one privileged act out \
         of it; both capability lines must be there and both must be empty (bounded to {bounded:?}, \
         given {given:?})"
    )]
    TheAgentHoldsSomething {
        /// The agent's unit.
        agent: String,
        /// What the unit bounds it to.
        bounded: Vec<String>,
        /// What the unit gives it.
        given: Vec<String>,
    },
    /// The agent's service runs as somebody who is not the person the machine
    /// description names.
    #[error(
        "{agent} runs as `{as_login}`, which this image does not make with number {person} — the \
         machine description says that number is the signed-in person, and the kernel's answer \
         about who is on the socket is compared against it"
    )]
    TheAgentIsNotThePerson {
        /// The agent's unit.
        agent: String,
        /// The login it names, or `-` where it names none.
        as_login: String,
        /// The number the description says the person is.
        person: u32,
    },
    /// The agent's service is not in the group the socket is handed to.
    #[error(
        "{agent} is not in group `{group}` — the socket is handed to that group, and changing a \
         file's group is only allowed to a member of it, so this service would bind a door it \
         cannot give the agent"
    )]
    TheAgentIsNotInTheGroup {
        /// The agent's unit.
        agent: String,
        /// The group it should be in.
        group: String,
    },
    /// The person's login is not in the agent's group outside the unit either.
    #[error(
        "this image does not put login `{login}` in group `{group}` — the unit puts the service \
         in it, and a person who is only in it while systemd is starting them is a person for \
         whom the same daemon run by hand behaves differently"
    )]
    ThePersonIsNotInTheGroup {
        /// The person's login.
        login: String,
        /// The group.
        group: String,
    },
    /// The agent's own login is not one this image makes.
    #[error(
        "this image makes no login with number {agent} — the machine description says that is the \
         agent, ADR 0001 §5 makes it a login of its own, and SO_PEERCRED is the whole of the \
         division between the two doors on the socket"
    )]
    TheAgentHasNoLogin {
        /// The number the description says the agent is.
        agent: u32,
    },
    /// A directory the daemon refuses to make is one nothing makes.
    #[error(
        "nothing in this image makes {at} at boot, and `alo-agentd` refuses to make it — the \
         service would name the directory and stop"
    )]
    NotMadeAtBoot {
        /// The directory.
        at: PathBuf,
    },
    /// The directory every person's door goes in is not what ADR 0017 says.
    #[error(
        "{at} is made {mode:04o} {owner}:{group}, and ADR 0017 says 0755 root:root — every \
         person's door goes in it, so its mode is not one person's service's to choose"
    )]
    TheDoorIsNotWhatWasDecided {
        /// The directory.
        at: PathBuf,
        /// The mode it is made with.
        mode: u32,
        /// The login that owns it.
        owner: String,
        /// The group it is in.
        group: String,
    },
    /// The folder the record goes in belongs to somebody else.
    #[error(
        "{at} is made for `{owner}`, and the record is written by `{person}` — what an agent did \
         on somebody's machine is theirs, and a folder the service cannot write is a service that \
         will not start"
    )]
    TheRecordFolderIsNotThePersons {
        /// The directory.
        at: PathBuf,
        /// The login it is made for.
        owner: String,
        /// The login the service runs as.
        person: String,
    },
    /// The image ships a number of days nobody chose.
    #[error(
        "this image ships a record kept for {days} days — ADR 0004 gives retention to the \
         organisation that manages the machine, and the only rule an image may write is `forever`"
    )]
    ARetentionNobodyChose {
        /// How many days it ships.
        days: u32,
    },
    /// The agent's service could not make the control group a turn runs in.
    #[error(
        "{agent} does not delegate its control group — a turn is a cgroup made under the \
         service's own (ADR 0015), a service running as a person can only make one where systemd \
         has handed the subtree over, and without `Delegate=` this machine boots to a daemon that \
         stops on the boundary it cannot make"
    )]
    TheAgentCannotMakeATurnsControlGroup {
        /// The agent's unit.
        agent: String,
    },
    /// Nothing makes the person's own door, which the daemon cannot make itself.
    #[error(
        "{agent} does not make {expected} — ADR 0017 puts every person's door in a directory that \
         is 0755 root:root, so the service cannot make its own name there, and this is the \
         directory the socket for person {person} goes in (it names {declared:?})"
    )]
    ThePersonsDoorIsNotMade {
        /// The agent's unit.
        agent: String,
        /// The runtime directory it should name, relative to `/run`.
        expected: String,
        /// The number the description says the person is.
        person: u32,
        /// What the unit names instead.
        declared: Vec<String>,
    },
    /// The person's door would be made where anybody can look in.
    #[error(
        "{agent} makes the person's door {mode} — ADR 0017 says 0750, the person and the agent's \
         group and nobody else, and a unit that leaves it to systemd's default gets 0755 rather \
         than a decision"
    )]
    ThePersonsDoorIsNotShut {
        /// The agent's unit.
        agent: String,
        /// The mode the unit names, or `-` where it names none.
        mode: String,
    },
    /// The agent's service would be started by booting rather than by signing
    /// in.
    #[error(
        "{agent} is pulled in by `{wanted_by}` rather than by `{manager}` — a daemon started at \
         boot is a daemon running before anybody has signed in, with no session, no bus and no \
         person, which is not what ADR 0001 §2 means by running as the signed-in person"
    )]
    TheAgentIsNotStartedBySigningIn {
        /// The agent's unit.
        agent: String,
        /// What pulls it in, or `-` where nothing does.
        wanted_by: String,
        /// The person's own systemd manager, which is their session.
        manager: String,
    },
    /// The agent's service would outlive the session it belongs to.
    #[error(
        "{agent} is not bound to `{manager}` and ordered after it — signing out would leave the \
         agent service holding somebody's door with nobody signed in, and starting it before \
         their session would leave it beside the session rather than inside it"
    )]
    TheAgentDoesNotStopWithTheSession {
        /// The agent's unit.
        agent: String,
        /// The person's own systemd manager.
        manager: String,
    },
    /// The agent's service would run with an environment that is not the
    /// person's session.
    #[error(
        "{agent} states `{variable}={named}`, and the session of the person this machine \
         describes is `{variable}={theirs}` — a system unit inherits no session, so what this \
         line says is the whole of what the service is told about one"
    )]
    TheAgentsEnvironmentIsNotTheSessions {
        /// The agent's unit.
        agent: String,
        /// Which variable.
        variable: String,
        /// What the unit says, or `-` where it says nothing.
        named: String,
        /// What the person's session is.
        theirs: String,
    },
    /// The image ships the accounts a person signs in with.
    #[error(
        "this image ships {at} — the accounts a person signs in with are the machine's, made on \
         the machine, and an image that carried one would carry a login whose password is known \
         to everybody who has the image; `alo-accounts` reads no store as first boot, which is \
         the state an image ships in"
    )]
    AnAccountShippedWithTheImage {
        /// Where it ships.
        at: PathBuf,
    },
    /// A unit nothing pulls in at boot.
    #[error("nothing pulls {unit} in at boot — it has no [Install] section that wants it")]
    NothingPullsItIn {
        /// The unit.
        unit: String,
    },
    /// The image does not land the model runtime where it belongs.
    #[error(
        "this image does not land the model runtime at {at}, copied out of a build stage that \
         fetched it — ADR 0025 promises the local model is what the machine arrives ready to \
         run, and ADR 0006 says the runtime arrives as a pinned upstream artefact, never as a \
         source tree or a binary committed here; an image without it is a machine whose \
         sovereignty is an option to find"
    )]
    TheRuntimeIsNotOnTheImage {
        /// Where the artefact should land, and does not.
        at: PathBuf,
    },
    /// The model runtime's version floats.
    #[error(
        "the model runtime's version is `{version}`, which is not one exact release — ADR 0006 \
         pins the runtime in the image, and a floating or partial version is a different \
         runtime on two builds of one image, moved by nobody and tested beside nothing; write \
         it `major.minor.patch`, the way every other pin in the Containerfile is written"
    )]
    TheRuntimesVersionIsNotPinned {
        /// What the recipe says, or `-` where it says nothing.
        version: String,
    },
    /// The model runtime's artefact is not held to a digest the build checks.
    #[error(
        "the model runtime arrives unverified (digest: {digest}) — the version says what was \
         asked for, and only a whole sha256 that the build checks before unpacking says what \
         arrived; without it, a release published again under the same number becomes a \
         different runtime on a certified machine, and ADR 0006's pin is prose"
    )]
    TheRuntimeArrivesUnverified {
        /// The digest the recipe names, or `-` where it names none.
        digest: String,
    },
    /// The image carries no weights, so the runtime aboard it has nothing to
    /// load.
    #[error(
        "this image does not land any weights at {at}, copied out of a build stage that fetched \
         and checked them — ADR 0025 promises the local model is what the machine arrives ready \
         to run, and a model runtime with nothing to load answers exactly as little as no runtime \
         at all; a machine that has to fetch a model before it can do anything is a machine whose \
         sovereignty is a download"
    )]
    TheWeightsAreNotOnTheImage {
        /// Where the weights should land, and do not.
        at: PathBuf,
    },
    /// The weights are fetched from a name that means something else tomorrow.
    #[error(
        "the weights are fetched from `{from}`, which is a moving name rather than one exact \
         revision — ADR 0006's pin said about a file: a branch is a different artefact on two \
         builds of one image, and the digest beside it turns that into a release that stopped \
         building rather than a mistake anybody caught"
    )]
    TheWeightsAreNotPinned {
        /// Where the recipe fetches them from, or `-` where it says nothing.
        from: String,
    },
    /// The weights are not held to a digest checked before anything reads them.
    #[error(
        "the weights arrive unverified (digest: {digest}) — only a whole sha256, checked before \
         any other step reads the file, says what arrived; weights are imported by the runtime \
         rather than unpacked, so a recipe that checked afterwards would have built its model \
         store out of whatever the network gave it and gone red with those bytes already in a \
         layer"
    )]
    TheWeightsArriveUnverified {
        /// The digest the recipe names, or `-` where it names none.
        digest: String,
    },
    /// The recipe carries weights without saying which catalogue entry they are.
    #[error(
        "this image carries weights and does not say which model they are — the catalogue is what \
         states a model's licence, its cost and whether anybody measured it (docs/features.md), \
         and weights nothing can look up are weights nobody can answer a question about"
    )]
    TheImageDoesNotSayWhichModelItCarries,
    /// The recipe names a model the catalogue does not have.
    #[error(
        "this image carries `{model}`, which `crates/alo-models/data/catalogue.toml` does not \
         have — a model on the disk of every machine we ship is one whose licence, cost and \
         measurement a person can read, and an entry that exists only in a build argument is none \
         of those"
    )]
    TheWeightsNameAModelTheCatalogueDoesNotHave {
        /// What the recipe names.
        model: String,
    },
    /// The recipe names a model nobody has measured driving the verbs.
    #[error(
        "this image carries `{model}`, which nobody has put to `alo-driving` — `docs/features.md` \
         promises a catalogue measured by us rather than claimed by the publisher, and the one \
         model every machine arrives with is the last place to take a publisher's word for it \
         (ADR 0007)"
    )]
    TheWeightsWereNeverMeasured {
        /// What the recipe names.
        model: String,
    },
    /// The recipe's quantisation and artefact are not the ones the catalogue
    /// states for that model.
    #[error(
        "this image carries `{model}` as `{said}`, and the catalogue states `{catalogue}` — the \
         entry a person reads and the name their machine answers to are one string or they are \
         two models, and \"it worked for me\" is not a useful report without the quantisation"
    )]
    TheWeightsAreNotTheArtefactTheCatalogueNames {
        /// What the recipe names.
        model: String,
        /// The quantisation and artefact the recipe says, or `-` where it says
        /// nothing.
        said: String,
        /// What the catalogue says, or `-` where it states none.
        catalogue: String,
    },
    /// The model the image carries is more than the machine it ships on can
    /// run.
    #[error(
        "this image carries `{model}`, which the catalogue says needs {needs_gb} GB and runs \
         `{on_cpu}` — the machine that decides whether this product has a market is an ordinary \
         business laptop with {machine_gb} GB and no card (docs/hardware.md, ADR 0007), and a \
         model it cannot drive is a machine that arrives ready to wait"
    )]
    TheWeightsAreMoreThanTheMachineCanDrive {
        /// What the recipe names.
        model: String,
        /// What the catalogue says it needs, in gigabytes.
        needs_gb: String,
        /// How it behaves with no graphics card, as the catalogue grades it.
        on_cpu: String,
        /// What the certified laptop has, in gigabytes.
        machine_gb: String,
    },
    /// The image would redistribute weights under a licence that is not ours to
    /// hand on.
    #[error(
        "this image carries `{model}`, whose licence `{licence}` does not permit commercial use \
         outright — carrying weights in an image *is* redistribution, so its terms would travel \
         with every copy of alo OS and bind everybody who received one; the catalogue may offer \
         such a model, and the machine may not arrive with it"
    )]
    TheWeightsCarryALicenceThatWasNotOursToHandOn {
        /// What the recipe names.
        model: String,
        /// The licence the catalogue states.
        licence: String,
    },
    /// The catalogue itself would not read, so nothing here can be checked
    /// against it.
    #[error(
        "the catalogue this system ships did not read ({why}) — nothing about the weights this \
         image carries can be checked against it, and a check that quietly passed when it could \
         not look would be worse than no check"
    )]
    TheCatalogueDidNotRead {
        /// What `alo_models::Catalogue` said.
        why: String,
    },

    /// The model service would run as a login this image never creates.
    #[error(
        "{server} runs as `{as_login}`, which this image does not make — the one thing that serves \
         the model runs as a login of its own (ADR 0025), and a name nothing creates is a service \
         that will not start on a machine that otherwise boots perfectly"
    )]
    TheServerIsNotALoginThisImageMakes {
        /// The model service's unit.
        server: String,
        /// The login it names, or `-` where it names none.
        as_login: String,
    },
    /// The model service would run as the person or as the agent.
    #[error(
        "{server} runs as `{as_login}`, which is {whose} own login — the process that serves the \
         model is up before anybody signs in and reads every question put to this machine, and \
         ADR 0001 §2 and §5 are the division it would be standing inside rather than outside"
    )]
    TheServerIsSomebodyElse {
        /// The model service's unit.
        server: String,
        /// The login it names.
        as_login: String,
        /// Whose login that is.
        whose: String,
    },
    /// The model service is in a group this image does not make.
    #[error(
        "{server} runs in group `{group}`, which this image does not make — the model service is a \
         login and a group of its own (ADR 0025), and a group nothing creates is a service that \
         will not start"
    )]
    TheServerIsNotInAGroupOfItsOwn {
        /// The model service's unit.
        server: String,
        /// The group it names, or `-` where it names none.
        group: String,
    },
    /// The model service shares a group with the person, the agent or the
    /// greeter.
    #[error(
        "{server} runs in group `{group}`, which is {whose} — on a machine where what a file is \
         reachable by is a group, the service that holds the model may not be inside the division \
         ADR 0001 §5 draws around the agent"
    )]
    TheServerSharesItsGroup {
        /// The model service's unit.
        server: String,
        /// The group it names.
        group: String,
        /// Whose group that is.
        whose: String,
    },
    /// The model service holds a capability, or does not say it holds none.
    #[error(
        "{server} does not say it holds nothing (bounded to {bounded:?}, given {given:?}) — \
         serving a model needs no privilege at all, and ADR 0018's argument is that the one \
         privileged component is acceptable because of how little it is trusted with; a capability \
         added here to make something work is that argument quietly stopping being true"
    )]
    TheServerHoldsSomething {
        /// The model service's unit.
        server: String,
        /// What the unit bounds it to.
        bounded: Vec<String>,
        /// What the unit gives it.
        given: Vec<String>,
    },
    /// The model service looks for the model somewhere the weights did not land.
    #[error(
        "{server} is pointed at `{store}` and the weights land in `{weights}` — the runtime's own \
         default is a home directory this image does not make, so a machine carrying 2.23 GiB of \
         model would serve nothing and report nothing installed, which looks from the outside \
         exactly like a machine nobody put a model on (ADR 0025)"
    )]
    TheServersStoreIsNotTheWeights {
        /// The model service's unit.
        server: String,
        /// What the unit points it at, or `-` where it points it nowhere.
        store: String,
        /// Where the weights land.
        weights: String,
    },
    /// The model service answers somewhere this machine does not look — or
    /// somewhere the network can reach.
    #[error(
        "{server} answers at `{address}` and this machine looks at `{looks}` (ADR 0019) — an \
         address naming anything but the loopback interface offers this machine's model to the \
         network it is plugged into, with no grant, no record and nothing on the egress indicator, \
         because nothing left"
    )]
    TheServerIsNotWhereTheMachineLooks {
        /// The model service's unit.
        server: String,
        /// What the unit says, or `-` where it says nothing.
        address: String,
        /// Where `alo-models` knocks.
        looks: String,
    },
    /// The model service may make connections of its own.
    #[error(
        "{server} may reach `{allowed}` and is denied `{denied}` — law 1 says a working day with a \
         local model produces zero inference egress, and the one process holding the model is the \
         one that has to be silent for that to be true; an update check, a telemetry call or a \
         registry pull must not fail politely, it must not leave, which takes both an allow list \
         naming this machine alone and a deny of everywhere under it"
    )]
    TheServerMayReachTheNetwork {
        /// The model service's unit.
        server: String,
        /// What the unit allows, or `-` where it allows nothing.
        allowed: String,
        /// What it denies, or `-` where it denies nothing.
        denied: String,
    },
    /// The opener would run as somebody `logind` will not open a session for.
    ///
    /// ADR 0024 measured it twice, on two systemds: `CreateSession` answers
    /// root with *Leader PID is not valid* — the call authorised and only its
    /// contents refused — and an unprivileged caller with *Access denied*. So
    /// an opener that is not root is a machine nobody can sign in to, and the
    /// failure arrives as a screen that does nothing.
    #[error(
        "{opener} runs as {as_login} and systemd-logind opens a session only for a privileged \
         caller (ADR 0024, and the measurement in docs/quirks.md) — a machine whose opener is not \
         root is a machine nobody can sign in to"
    )]
    TheOpenerIsNotRoot {
        /// The opener's unit.
        opener: String,
        /// Who it says it runs as, or `-` where it says nothing.
        as_login: String,
    },

    /// The opener holds a capability.
    ///
    /// **The line this whole component's acceptability rests on.** ADR 0018
    /// gives the loader two capabilities and argues at length for them; ADR
    /// 0024 adds a second privileged component and its price is that this one
    /// holds *none* — what it needs is a uid, which is what `logind` decides
    /// on. A capability added here is that argument undone in one line, in the
    /// file nobody reviews.
    #[error(
        "{opener} holds capabilities (bounded to {bounded:?}, given {given:?}) — the second \
         privileged component alo OS has holds none at all, because what systemd-logind decides \
         CreateSession on is a uid; both lines exist and are empty, or this is no longer the \
         component ADR 0024 accepted"
    )]
    TheOpenerHoldsSomething {
        /// The opener's unit.
        opener: String,
        /// The most it may ever hold, as the unit says.
        bounded: Vec<String>,
        /// What it is given to start with.
        given: Vec<String>,
    },

    /// The opener is not in the group its door is handed to.
    ///
    /// The door is handed to whatever group the process is in, and that is the
    /// whole of who may ask for a session. A group this image does not make is
    /// a service that will not start; root's group is a door the sign-in
    /// surface can never reach.
    #[error(
        "{opener} runs in group `{group}`, which is not a login group this image makes — its door \
         is handed to whatever group it is in, so a group that is not the greeter's is either a \
         service that will not start or a door nobody can knock on"
    )]
    TheOpenerIsNotInTheGreetersGroup {
        /// The opener's unit.
        opener: String,
        /// The group it says it runs in, or `-` where it says nothing.
        group: String,
    },

    /// The greeter is one of the two logins that already exist.
    ///
    /// A greeter that is the person would mean anything running as the person
    /// could ask for a session; a greeter that is the agent would put the
    /// sign-in door inside the reach of the thing ADR 0001 §2 spends its length
    /// keeping authority away from.
    #[error(
        "this image's greeter is login {greeter}, which is also {whose} — the sign-in surface is a \
         login of its own (ADR 0024), because a door handed to the person's group or the agent's \
         is a door those can knock on"
    )]
    TheGreeterIsSomebodyElse {
        /// The number the greeter's group has.
        greeter: u32,
        /// Who else has it.
        whose: String,
    },

    /// The opener's door is not in the directory the opener looks in.
    #[error(
        "{opener} makes /run/{made} and alo-sessiond opens its door in {looks} — a service whose \
         runtime directory is not where its own code binds is a door that is never opened, and \
         nothing but this notices"
    )]
    TheOpenersDoorIsNotWhereItLooks {
        /// The opener's unit.
        opener: String,
        /// What the unit makes, relative to `/run`, or `-` where it makes
        /// nothing.
        made: String,
        /// Where the code opens its door.
        looks: String,
    },

    /// The opener's door directory is open to more than the greeter.
    #[error(
        "{opener} makes its runtime directory `{mode}` — the sign-in door goes in it, and \
         `{wanted}` is the mode that lets the greeter's group in and nobody else"
    )]
    TheOpenersDoorIsNotShut {
        /// The opener's unit.
        opener: String,
        /// The mode it says, or `-` where it says nothing.
        mode: String,
        /// The mode that was decided.
        wanted: String,
    },

    /// The recipe does not say what disk this image becomes.
    #[error(
        "this image's recipe does not say `{label}` — an image is not a disk, and the three \
         `alo.disk.*` labels are where it says which tool writes one, which firmware that disk is \
         installed for, and what the file is called; without them `docs/booting.md` is a document \
         held to nothing"
    )]
    TheImageDoesNotSayWhatDiskItBecomes {
        /// The label that is missing.
        label: String,
    },
    /// The disk would be written by something other than the pinned base's own
    /// tool.
    #[error(
        "this image's disk would be written by `{tool}` out of a base pinned `{pinned}` — the disk \
         is written by `bootc install to-disk` run out of the base image, so that base's digest is \
         the version of the tool (ADR 0011 rents the base rather than trusting what a tag meant \
         this morning); anything else is a partitioner of ours, laying out a disk nobody upstream \
         ever tested"
    )]
    TheDiskIsNotWrittenByThePinnedBase {
        /// The tool the recipe names, or `-` where it names none.
        tool: String,
        /// Whether the base the tool comes out of is pinned by content.
        pinned: bool,
    },
    /// A disk somebody lays out by hand.
    #[error(
        "{at} names `{by}` — a disk this repository ships is written by the upstream tool the base \
         carries, never assembled here; `CLAUDE.md` says engines are configured and never written \
         in, and a partition table of our own is that rule broken in the one file whose mistakes \
         only appear on somebody's machine"
    )]
    TheDiskWouldBeLaidOutByHand {
        /// Where the partitioner is named.
        at: String,
        /// What is named.
        by: String,
    },
    /// The document tells a person something the recipe does not say.
    #[error(
        "`docs/booting.md` says `{fact}: {said}` and the image's recipe says `{recipe}` — the disk \
         is declared in two places, and the one a person reads is the one nothing would have \
         caught; a document that drifted from the recipe is somebody following it to a machine \
         that does not boot"
    )]
    TheDocumentDoesNotSayWhatTheRecipeDoes {
        /// Which fact.
        fact: String,
        /// What the document says, or `-` where it says nothing.
        said: String,
        /// What the recipe says, or `-` where it says nothing.
        recipe: String,
    },
    /// The firmware the disk is installed for is not the one the document tells
    /// a person to select.
    #[error(
        "this image's disk is installed for `{firmware}` and `docs/booting.md` tells a person to \
         select a generation {generation} virtual machine, which is the {other} one — a person \
         follows the document, and the two are one line apart in two files that nothing else reads \
         together"
    )]
    TheFirmwareIsNotWhatAPersonIsToldToSelect {
        /// The firmware the recipe names, or `-` where it names none.
        firmware: String,
        /// The generation the document names, or `-` where it names none.
        generation: String,
        /// The firmware that generation really is.
        other: String,
    },
    /// The document does not carry the command it exists to carry.
    #[error(
        "`docs/booting.md` never runs `{tool}` — naming a tool is not telling anybody how to use \
         it, and one documented command turning the image into a disk is the whole of what that \
         document is for"
    )]
    TheDocumentDoesNotGiveTheCommand {
        /// The tool the recipe names.
        tool: String,
    },
    /// The document is missing a section it is answerable for.
    #[error(
        "`docs/booting.md` has no section `{heading}` — a person doing this once needs what it \
         produces, what they must install, what to attach it to, and what it cannot show, and a \
         document missing one of those is a document that answers a question it was not asked"
    )]
    TheDocumentIsMissingASection {
        /// The heading that is missing.
        heading: String,
    },
    /// The document does not say what a virtual machine cannot show.
    #[error(
        "`docs/booting.md` does not say, under `{heading}`, what a virtual machine cannot show \
         about `{about}` — a virtual GPU is not the GPU working on first boot and tame virtual \
         firmware is not a certified machine's, so this section is what stops somebody quoting a \
         virtual machine as the hardware acceptance in phase 8"
    )]
    TheDocumentDoesNotSayWhatADiskCannotShow {
        /// The heading it should be under.
        heading: String,
        /// What it does not name.
        about: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sentence names the promise it is about, because the person reading
    /// it is looking at a diff rather than at this repository's documentation.
    #[test]
    fn a_disagreement_names_the_decision_it_is_about() {
        assert!(
            Wrong::TheLoaderHoldsSomethingElse {
                loader: "alo-boundaryd.service".to_owned(),
                held: vec!["CAP_SYS_ADMIN".to_owned()],
            }
            .to_string()
            .contains("ADR 0018")
        );
        assert!(
            Wrong::TheDoorIsNotWhatWasDecided {
                at: PathBuf::from("/run/alo"),
                mode: 0o777,
                owner: "alo".to_owned(),
                group: "alo".to_owned(),
            }
            .to_string()
            .contains("ADR 0017")
        );
    }

    /// A version that floats is refused **in words that say why**: the reader
    /// is somebody who wrote `latest` to get a build going, and the sentence
    /// has to give them the reason it cannot ship, not only the rule.
    #[test]
    fn a_floating_version_is_refused_in_words_that_say_why() {
        let said = Wrong::TheRuntimesVersionIsNotPinned {
            version: "latest".to_owned(),
        }
        .to_string();
        assert!(said.contains("latest"), "{said}");
        assert!(said.contains("ADR 0006"), "{said}");
        assert!(
            said.contains("a different runtime on two builds of one image"),
            "{said}"
        );

        let said = Wrong::TheRuntimeArrivesUnverified {
            digest: "-".to_owned(),
        }
        .to_string();
        assert!(said.contains("what arrived"), "{said}");
        assert!(said.contains("ADR 0006"), "{said}");
    }

    /// A mode is said the way somebody wrote it, in octal with its leading zero,
    /// rather than as the number it happens to be.
    #[test]
    fn a_mode_is_said_in_octal() {
        let said = Wrong::TheDoorIsNotWhatWasDecided {
            at: PathBuf::from("/run/alo"),
            mode: 0o777,
            owner: "alo".to_owned(),
            group: "alo".to_owned(),
        }
        .to_string();
        assert!(said.contains("0777"), "{said}");
    }
}
