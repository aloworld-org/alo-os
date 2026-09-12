//! Every promise the image's files make to the two daemons, checked against
//! each other.
//!
//! A build cannot catch any of this. `docker build` will produce an image whose
//! machine description names a login the image never creates, whose loader is
//! in root's group, or whose agent service has quietly been given a capability
//! — and every one of those is a machine that boots to a daemon that stops, or
//! worse, one that runs while a sentence in `docs/` about it has become untrue.
//!
//! # Everything wrong at once, rather than the first thing wrong
//!
//! [`everything_wrong_with`] answers with all of them, which is
//! `alo-strings`' `Vocabulary::check` argument met a second time: being told
//! about the next mistake each time you try again is how somebody gives up. An
//! image is edited in one sitting and there are five files in it.
//!
//! # What is checked, and what deliberately is not
//!
//! Every check here is a sentence in an ADR or a contract, named in the
//! [`Wrong`] it produces. What is **not** here is anything about whether the
//! image boots: that is the machine half of `ROADMAP.md`'s image line, it needs
//! a machine, and an image that builds is not an image that boots.

use std::path::Path;

use alo_entering::TheSessionsEnvironment;
use alo_keeping::Keeping;
use alo_models::{Catalogue, OnCpu};

use crate::image::Image;
use crate::service::ROOT;
use crate::wrong::Wrong;
use crate::{booting, disk};

/// The directory every person's door goes in (ADR 0017).
pub const THE_DOOR: &str = "/run/alo";

/// The mode that directory is made with: anybody may walk through, nobody but
/// root may put a name in it.
const THE_DOORS_MODE: u32 = 0o755;

/// Where systemd's `RuntimeDirectory=` is relative to, and the only reason this
/// crate knows the string: [`THE_DOOR`] is absolute and that setting is not.
const THE_RUN: &str = "/run";

/// The mode the person's own door is made with, as a unit spells it: the
/// person, the agent's group, and nobody else (ADR 0017, and
/// `alo-agentd`'s `place.rs`, which sets the same mode again at every start).
const THE_PERSONS_DOORS_MODE: &str = "0750";

/// The two capabilities ADR 0018 gives the loader, and the whole of what makes
/// one privileged component acceptable.
const WHAT_THE_LOADER_MAY_HOLD: [&str; 2] = ["CAP_BPF", "CAP_SYS_ADMIN"];

/// The mode the opener's door directory is made with: the greeter's group, and
/// nobody else (ADR 0024). The door itself is `0660` inside it, which
/// `alo-sessiond` sets and tests.
const THE_SIGN_IN_DOORS_MODE: &str = "0750";

/// What a unit says where it says nothing, in a sentence somebody reads.
const NOTHING: &str = "-";

/// The variable the model runtime reads its store from.
///
/// The rented runtime's own spelling, which the image has to write to put the
/// weights anywhere but that runtime's default — a home directory belonging to a
/// login this image does not make. ADR 0006's one-file rule is about how the
/// runtime is *spoken to*; where its files are is the image's own fact, and
/// `crate::runtime` says the same thing at more length.
const THE_STORE: &str = "OLLAMA_MODELS";

/// The variable it reads its address from.
const THE_ADDRESS: &str = "OLLAMA_HOST";

/// The runtime's own switch for the two questions it asks its publisher at
/// every start, and what it is set to when they are not asked.
///
/// Measured on 2026-09-12 (`docs/quirks.md`): with it set the runtime says
/// *cloud disabled: true* and makes neither request; without it, both, and a
/// retry every five minutes for as long as they fail. A second lock beside the
/// IP filter rather than instead of it — the filter is the kernel's and cannot
/// be switched off by a runtime update; this is the engine's, and is what keeps
/// a machine's journal free of lines naming the publisher.
const THE_RUNTIMES_OWN_SWITCH: &str = "OLLAMA_NO_CLOUD";

/// What that switch says when the questions are not asked.
const NOT_ASKED: &str = "1";

/// What an IP access list says to mean *this machine, and nothing else*.
const ONLY_THIS_MACHINE: &str = "localhost";

/// What it says to mean *everywhere*.
const ANYWHERE: &str = "any";

/// The memory of the machine the weights on this image are sized for.
///
/// `docs/hardware.md` certifies two machines and says which of them matters
/// more: *an ordinary business laptop — no discrete graphics, 16 GB of memory*,
/// because the Windows 10 fleet this product exists to catch has almost no
/// discrete GPUs in it. One image is built, so the model it carries is sized for
/// that machine rather than for the workstation; a GPU workstation runs the same
/// weights and can fetch something larger, which is a choice its owner makes.
const THE_CERTIFIED_LAPTOP_GB: f32 = 16.0;

/// The four questions `docs/booting.md` is answerable for, as its headings.
///
/// Somebody does this once, on a machine that has never run alo OS, and the
/// document is the whole of what they have.
const WHAT_THE_DOCUMENT_ANSWERS: [&str; 4] = [
    "What this produces",
    "What you need installed",
    "Attaching it to Hyper-V",
    "What a virtual machine cannot show",
];

/// The heading the limits go under, which is the last of the four.
const WHAT_IT_CANNOT_SHOW: &str = WHAT_THE_DOCUMENT_ANSWERS[3];

/// What that section has to name, by subject.
///
/// Each is a thing somebody could otherwise watch a virtual machine do and
/// believe they had watched a certified one do it. The plan this task came from
/// names the first two in as many words — *a virtual GPU is not the GPU works on
/// first boot*, and *tame virtual firmware is not firmware to sign-in* — and the
/// third is the sentence that keeps a `ROADMAP.md` line from being ticked.
const WHAT_A_VIRTUAL_MACHINE_CANNOT_SHOW: [&str; 3] = ["GPU", "firmware", "hardware acceptance"];

/// Which Hyper-V generation is which firmware.
///
/// The two are not a preference: a generation 2 machine is the UEFI one and a
/// generation 1 machine is the BIOS one, and a person pointing the wrong one at
/// this disk finds nothing to boot.
const THE_GENERATIONS: [(&str, &str); 2] = [("1", "bios"), ("2", "uefi")];

/// Everything this image's files disagree with each other about.
///
/// An empty answer is an image whose five files say one thing. It is not a
/// claim that the machine boots.
#[must_use]
pub fn everything_wrong_with(image: &Image) -> Vec<Wrong> {
    let mut wrong = Vec::new();
    the_loader_runs_first(image, &mut wrong);
    the_loader_holds_two_things(image, &mut wrong);
    the_opener_holds_nothing_at_all(image, &mut wrong);
    the_opener_can_be_knocked_on_by_the_greeter_alone(image, &mut wrong);
    the_agent_holds_nothing(image, &mut wrong);
    the_logins_are_the_ones_this_image_makes(image, &mut wrong);
    the_build_holds_every_login_to_its_number(image, &mut wrong);
    the_directories_are_made(image, &mut wrong);
    the_agent_can_make_what_it_is_not_given(image, &mut wrong);
    nobody_chose_a_retention(image, &mut wrong);
    both_units_are_pulled_in(image, &mut wrong);
    the_agent_runs_inside_the_persons_session(image, &mut wrong);
    nobody_signs_in_with_an_account_the_image_shipped(image, &mut wrong);
    the_model_runtime_is_aboard_and_pinned(image, &mut wrong);
    the_weights_are_aboard_pinned_and_measured(image, &mut wrong);
    the_model_is_served_by_a_login_of_its_own(image, &mut wrong);
    the_server_reaches_nothing_off_this_machine(image, &mut wrong);
    the_image_becomes_a_disk(image, &mut wrong);
    the_document_says_what_the_recipe_does(image, &mut wrong);
    wrong
}

/// **The image says what disk it becomes, and it is written by the base's own
/// tool.**
///
/// An image is not a disk, and until `docs/booting.md` existed nothing here
/// turned one into the other — so every promise that waited on *no machine has
/// ever* waited on this. The three `alo.disk.*` labels are where the recipe says
/// which tool writes the disk, which firmware it is installed for and what the
/// file is called, and they are in the recipe rather than beside it because a
/// second file would be the second recipe this task exists to prevent.
///
/// The tool is not pinned separately because it is already pinned: `bootc
/// install to-disk` is run out of the base image, so `THE_BASE`'s digest is the
/// version of the partitioner, and an unpinned base is an unpinned one.
fn the_image_becomes_a_disk(image: &Image, wrong: &mut Vec<Wrong>) {
    let declared = image.disk();

    for (label, said) in [
        (disk::THE_TOOL, declared.tool()),
        (disk::THE_FIRMWARE, declared.firmware()),
        (disk::THE_FILE, declared.file()),
    ] {
        if said.unwrap_or_default().is_empty() {
            wrong.push(Wrong::TheImageDoesNotSayWhatDiskItBecomes {
                label: label.to_owned(),
            });
        }
    }

    if !declared.written_by_the_base() {
        wrong.push(Wrong::TheDiskIsNotWrittenByThePinnedBase {
            tool: declared.tool().unwrap_or(NOTHING).to_owned(),
            pinned: declared.on_a_pinned_base(),
        });
    }

    if let Some(by) = declared.laid_out_by_hand() {
        wrong.push(Wrong::TheDiskWouldBeLaidOutByHand {
            at: "the image's recipe".to_owned(),
            by: by.to_owned(),
        });
    }
    for by in disk::NO_PARTITIONER {
        if image.document().names(by) {
            wrong.push(Wrong::TheDiskWouldBeLaidOutByHand {
                at: "`docs/booting.md`".to_owned(),
                by: by.to_owned(),
            });
        }
    }
}

/// **The document a person follows says what the recipe says.**
///
/// The disk is declared twice — once as labels a tool is given, once as a
/// sentence somebody reads before selecting a firmware in a dialog box — and the
/// second is the one nothing would otherwise catch. The way this document goes
/// wrong is not that somebody deletes it: it is that the image moves and the
/// document stays where it was, and the person following it reaches a machine
/// that does not boot and a repository that believed it did.
///
/// The firmware is the sharp one. A generation 2 virtual machine is the UEFI
/// one; a generation 1 machine pointed at a UEFI disk finds nothing to boot at
/// all, and the difference between those two is one digit in one line of prose.
fn the_document_says_what_the_recipe_does(image: &Image, wrong: &mut Vec<Wrong>) {
    let declared = image.disk();
    let document = image.document();

    for (fact, recipe) in [
        (booting::THE_TOOL, declared.tool()),
        (booting::THE_FIRMWARE, declared.firmware()),
        (booting::THE_DISK, declared.file()),
    ] {
        let said = document.says(fact);
        if said != recipe {
            wrong.push(Wrong::TheDocumentDoesNotSayWhatTheRecipeDoes {
                fact: fact.to_owned(),
                said: said.unwrap_or(NOTHING).to_owned(),
                recipe: recipe.unwrap_or(NOTHING).to_owned(),
            });
        }
    }

    let generation = document.says(booting::THE_GENERATION);
    let really = THE_GENERATIONS
        .into_iter()
        .find_map(|(number, firmware)| (Some(number) == generation).then_some(firmware));
    if really != declared.firmware() {
        wrong.push(Wrong::TheFirmwareIsNotWhatAPersonIsToldToSelect {
            firmware: declared.firmware().unwrap_or(NOTHING).to_owned(),
            generation: generation.unwrap_or(NOTHING).to_owned(),
            other: really.unwrap_or(NOTHING).to_owned(),
        });
    }

    if let Some(tool) = declared.tool()
        && !document.gives_the_command(tool)
    {
        wrong.push(Wrong::TheDocumentDoesNotGiveTheCommand {
            tool: tool.to_owned(),
        });
    }

    for heading in WHAT_THE_DOCUMENT_ANSWERS {
        if !document.has_a_section(heading) {
            wrong.push(Wrong::TheDocumentIsMissingASection {
                heading: heading.to_owned(),
            });
        }
    }
    for about in WHAT_A_VIRTUAL_MACHINE_CANNOT_SHOW {
        if !document.names_under(WHAT_IT_CANNOT_SHOW, about) {
            wrong.push(Wrong::TheDocumentDoesNotSayWhatADiskCannotShow {
                heading: WHAT_IT_CANNOT_SHOW.to_owned(),
                about: about.to_owned(),
            });
        }
    }
}

/// **The model runtime is on the image, pinned and verified.**
///
/// ADR 0025: the local model is what the machine arrives ready to run, and the
/// expensive half of that promise is an artefact on the disk. ADR 0006: it
/// arrives pinned. Neither can be caught by a build — a Containerfile that
/// dropped the runtime, floated its version to `latest`, or stopped checking
/// the digest builds green and ships a machine whose first promise is untrue.
///
/// Three separate disagreements rather than one, because they are fixed in
/// three different places and the person reading them is looking at a diff.
/// The weights are [`the_weights_are_aboard_pinned_and_measured`] beside this,
/// a promise of their own. What is still deliberately **not** here is the unit
/// that starts the runtime: which login it runs as, which group may reach its
/// socket and what it may reach off this machine are a decision of their own,
/// and a check written before that decision would be the decision.
fn the_model_runtime_is_aboard_and_pinned(image: &Image, wrong: &mut Vec<Wrong>) {
    let runtime = image.runtime();

    if !runtime.lands_its_binary() {
        wrong.push(Wrong::TheRuntimeIsNotOnTheImage {
            at: Path::new(crate::THE_RUNTIMES_BINARY).to_owned(),
        });
    }
    if !runtime.lands_its_libraries() {
        wrong.push(Wrong::TheRuntimeIsNotOnTheImage {
            at: Path::new(crate::THE_RUNTIMES_LIBRARIES).to_owned(),
        });
    }
    if !runtime.is_pinned() {
        wrong.push(Wrong::TheRuntimesVersionIsNotPinned {
            version: runtime.version().unwrap_or(NOTHING).to_owned(),
        });
    }
    if !runtime.is_verified() {
        wrong.push(Wrong::TheRuntimeArrivesUnverified {
            digest: runtime.digest().unwrap_or(NOTHING).to_owned(),
        });
    }
}

/// **The weights are aboard, pinned, checked before anything reads them, and a
/// model somebody actually measured.**
///
/// ADR 0025's expensive half: a model on the disk of every machine we ship,
/// sized for that machine (ADR 0007). A runtime with nothing to load answers
/// exactly as little as no runtime at all, and none of the ways this goes wrong
/// is visible at a build — a recipe that dropped the `COPY`, floated the fetch
/// to a branch, checked the digest after importing, or named a model nobody
/// measured all build green and all ship a machine whose first promise is
/// untrue.
///
/// **The catalogue is the authority for the last four**, rather than a list of
/// model names in a checker. `docs/features.md` promises entries *measured by
/// us, not claimed by the publisher* one line above the promise this function is
/// about, so the recipe's model is looked up and held to what
/// [`alo_models::Catalogue`] says: that somebody ran `alo-driving` against it,
/// that the quantisation and artefact are the entry's own, that it fits the
/// machine this product exists to reach, and that its licence was ours to hand
/// on at all.
///
/// That last one is not a formality. Carrying weights in an image **is**
/// redistribution: a licence with conditions would attach those conditions to
/// every holder of alo OS, which is a thing to do deliberately in an ADR and
/// never a thing to do by changing a build argument.
fn the_weights_are_aboard_pinned_and_measured(image: &Image, wrong: &mut Vec<Wrong>) {
    let weights = image.weights();

    if !weights.land() {
        wrong.push(Wrong::TheWeightsAreNotOnTheImage {
            at: Path::new(crate::THE_WEIGHTS).to_owned(),
        });
    }
    if !weights.is_pinned() {
        wrong.push(Wrong::TheWeightsAreNotPinned {
            from: weights.from().unwrap_or(NOTHING).to_owned(),
        });
    }
    if !weights.is_verified() {
        wrong.push(Wrong::TheWeightsArriveUnverified {
            digest: weights.digest().unwrap_or(NOTHING).to_owned(),
        });
    }
    if !weights.drops_the_source() {
        wrong.push(Wrong::TheWeightsAreCarriedTwice);
    }
    if !weights.holds_the_store_to_its_manifest() {
        wrong.push(Wrong::TheStoreIsHeldToNothing);
    }

    let Some(model) = weights.model() else {
        wrong.push(Wrong::TheImageDoesNotSayWhichModelItCarries);
        return;
    };
    let catalogue = match Catalogue::built_in() {
        Ok(catalogue) => catalogue,
        Err(why) => {
            wrong.push(Wrong::TheCatalogueDidNotRead {
                why: why.to_string(),
            });
            return;
        }
    };
    let Some(entry) = catalogue.get(model) else {
        wrong.push(Wrong::TheWeightsNameAModelTheCatalogueDoesNotHave {
            model: model.to_owned(),
        });
        return;
    };

    if !entry.drives_verbs.has_been_measured() {
        wrong.push(Wrong::TheWeightsWereNeverMeasured {
            model: model.to_owned(),
        });
    }

    let said = weights
        .quantisation()
        .zip(weights.artefact())
        .map(|(quantisation, artefact)| format!("{quantisation} / {artefact}"));
    let stated = entry
        .quantised_at()
        .map(|(quantisation, artefact)| format!("{quantisation} / {artefact}"));
    if said != stated {
        wrong.push(Wrong::TheWeightsAreNotTheArtefactTheCatalogueNames {
            model: model.to_owned(),
            said: said.unwrap_or_else(|| NOTHING.to_owned()),
            catalogue: stated.unwrap_or_else(|| NOTHING.to_owned()),
        });
    }

    if entry.min_ram_gb > THE_CERTIFIED_LAPTOP_GB || entry.on_cpu == OnCpu::Slow {
        wrong.push(Wrong::TheWeightsAreMoreThanTheMachineCanDrive {
            model: model.to_owned(),
            needs_gb: entry.min_ram_gb.to_string(),
            on_cpu: format!("{:?}", entry.on_cpu).to_lowercase(),
            machine_gb: THE_CERTIFIED_LAPTOP_GB.to_string(),
        });
    }

    if !entry.safe_default_for_business() {
        wrong.push(Wrong::TheWeightsCarryALicenceThatWasNotOursToHandOn {
            model: model.to_owned(),
            licence: entry.licence.name.clone(),
        });
    }
}

/// **The one thing that serves the model is a login of its own, holds nothing,
/// and is pointed at the weights and at the address this machine looks for a
/// runtime at.**
///
/// ADR 0025 put a runtime and 2.23 GiB of weights on every machine we build, and
/// until `alo-modeld.service` existed nothing started either of them — so
/// `alo-models` knocked at the loopback address and found what a machine with no
/// model at all would have offered it. A unit is not a `COPY` line's worth of
/// decision, and each of the four this function reads is one a build cannot see.
///
/// **Which login.** Not the person, whose session comes and goes and who would
/// otherwise have a process in their name running through a session they ended;
/// not the agent, which ADR 0001 §2 spends its length keeping authority away
/// from, and which is the last login to lend anything to a process that reads
/// every question put to this machine. A login of its own, made by the image, or
/// the `User=` line names somebody who cannot be told apart from one of those
/// two by the kernel.
///
/// **Which group**, for the same reason and with a second one: the group is who
/// this process *is* on a machine where files have groups, and a service sharing
/// the agent's would be inside ADR 0001 §5's division rather than outside it.
///
/// **Which store.** The runtime's own default is a home directory belonging to a
/// login this image does not make, so a unit that said nothing would serve
/// nothing off a machine carrying the model — the emptiest possible version of
/// this promise, and one that looks from the outside exactly like a machine
/// nobody put a model on.
///
/// **Which address**, which is the expensive one. `alo-models` knocks at exactly
/// one place (ADR 0019), and a `OLLAMA_HOST` that named every interface rather
/// than the loopback one would hand this machine's model to whatever network it
/// is plugged into — with no grant, no record and nothing on the egress
/// indicator, because nothing left. The address is read off
/// [`alo_models::ollama::DEFAULT_ENDPOINT`] rather than spelled here, so a
/// runtime that moves moves in one file.
///
/// # What this deliberately does not check, and what waits on a decision
///
/// *Who on this machine may ask the model anything.* A loopback TCP port has no
/// owner and no mode: every login on the machine can reach it, and no line in
/// any unit file changes that. `Group=` above says who **answers**, never who may
/// **ask**, and the two are not the same question —
/// [ADR 0027](../../../docs/decisions/0027-who-may-ask-the-model-anything.md) is
/// where the difference is argued and what it would take is priced. Nothing here
/// may be read as having answered it.
fn the_model_is_served_by_a_login_of_its_own(image: &Image, wrong: &mut Vec<Wrong>) {
    let server = image.server().called().to_owned();

    let as_login = image.server().as_login().unwrap_or(NOTHING);
    match image.login_called(as_login) {
        None => wrong.push(Wrong::TheServerIsNotALoginThisImageMakes {
            server: server.clone(),
            as_login: as_login.to_owned(),
        }),
        Some(number) => {
            for (theirs, whose) in [
                (image.description().person(), "the person"),
                (image.description().agent(), "the agent"),
            ] {
                if number == theirs {
                    wrong.push(Wrong::TheServerIsSomebodyElse {
                        server: server.clone(),
                        as_login: as_login.to_owned(),
                        whose: whose.to_owned(),
                    });
                }
            }
        }
    }

    let group = image.server().in_group().unwrap_or(NOTHING);
    if image.group_called(group).is_none() {
        wrong.push(Wrong::TheServerIsNotInAGroupOfItsOwn {
            server: server.clone(),
            group: group.to_owned(),
        });
    }
    for (theirs, whose) in [
        (image.agent().in_group(), "the person's"),
        (image.loader().in_group(), "the agent's"),
        (image.opener().in_group(), "the greeter's"),
    ] {
        if theirs == Some(group) {
            wrong.push(Wrong::TheServerSharesItsGroup {
                server: server.clone(),
                group: group.to_owned(),
                whose: whose.to_owned(),
            });
        }
    }

    if !image.server().holds_nothing() {
        wrong.push(Wrong::TheServerHoldsSomething {
            server: server.clone(),
            bounded: image
                .server()
                .bounded_to()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            given: image
                .server()
                .given()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        });
    }

    let stated = image.server().environment();

    let store = assigned(&stated, THE_STORE);
    let weights = crate::THE_WEIGHTS.trim_end_matches('/');
    if store.len() != 1 || store.first().map(|it| it.trim_end_matches('/')) != Some(weights) {
        wrong.push(Wrong::TheServersStoreIsNotTheWeights {
            server: server.clone(),
            store: said(&store),
            weights: weights.to_owned(),
        });
    }

    let looks = alo_models::ollama::DEFAULT_ENDPOINT;
    let address = assigned(&stated, THE_ADDRESS);
    if address != vec![looks] {
        wrong.push(Wrong::TheServerIsNotWhereTheMachineLooks {
            server,
            address: said(&address),
            looks: looks.to_owned(),
        });
    }
}

/// **And it makes no connection of its own — not an update check, not
/// telemetry, not a model registry.**
///
/// Law 1, said about the one process on the machine that holds the model: with a
/// local model a working day produces **zero** inference egress, measured at the
/// network boundary. A runtime that phoned home on start would be an egress
/// nobody asked for, on a machine sold on sovereignty, from the process whose
/// silence the whole claim rests on.
///
/// It is two lines rather than a comment because a comment is not a filter.
/// systemd's IP access list is applied in the kernel to this unit's own control
/// group, so a build of the runtime that grew an update check does not fail
/// politely — it does not leave. `IPAddressAllow=` without the deny beside it
/// filters nothing at all, which is why both are read and why an allow list
/// naming anything but this machine is a finding.
///
/// **And the runtime's own switch beside it**, as a second lock. Measured on
/// 2026-09-12 (`docs/quirks.md`): unset, the runtime asks its publisher two
/// questions at every start and retries every five minutes while they fail;
/// set, it says *cloud disabled: true* and asks neither. The filter is what
/// makes *nothing leaves* true; the switch is what keeps the journal free of a
/// line naming the publisher, and it is the engine's own documented setting, so
/// it is configuration rather than a patch (ADR 0011).
///
/// **What was watched, and what was not.** On 2026-09-12 this unit was started
/// by the image's own systemd, under a container whose init held the three
/// capabilities a machine's init holds anyway, with the image's own store and a
/// network: the login attempted sixteen packets to the publisher's port 443 and
/// none reached the host side of the container's bridge, while a request from
/// an unfiltered process in the same container went out and was answered
/// (`docs/quirks.md`). That is the filter refusing, watched at a boundary. It is
/// not a boot — nothing in this repository has yet booted the image — and
/// `docs/autonomy/v0-01-evidence.md` is where that stays owed.
fn the_server_reaches_nothing_off_this_machine(image: &Image, wrong: &mut Vec<Wrong>) {
    let server = image.server().called().to_owned();
    let allowed = image.server().may_reach();
    let denied = image.server().may_not_reach();

    if allowed != vec![ONLY_THIS_MACHINE] || !denied.contains(&ANYWHERE) {
        wrong.push(Wrong::TheServerMayReachTheNetwork {
            server: server.clone(),
            allowed: said(&allowed),
            denied: said(&denied),
        });
    }

    let switched = assigned(&image.server().environment(), THE_RUNTIMES_OWN_SWITCH);
    if switched != vec![NOT_ASKED] {
        wrong.push(Wrong::TheServerStillAsksItsPublisher {
            server,
            said: said(&switched),
        });
    }
}

/// Every value a unit's environment gives this variable, in the order it gives
/// them.
///
/// A list rather than the last one, because *exactly one assignment* is the
/// question everywhere this is used: two would be a unit where the second
/// silently wins and a reader believes the first.
fn assigned<'a>(stated: &[&'a str], variable: &str) -> Vec<&'a str> {
    stated
        .iter()
        .filter_map(|pair| pair.strip_prefix(variable))
        .filter_map(|rest| rest.strip_prefix('='))
        .collect()
}

/// **The image ships no accounts**, which is the state `alo-accounts` reads as
/// first boot.
///
/// The only thing an image is answerable for about the store a sign-in reads.
/// A password is typed on a machine by the person who owns it, so a store that
/// arrived in the image is a login every holder of that image can use — the
/// oldest mistake in shipped systems, and one a build cannot see, because a
/// file that is in the tree is a file that gets copied.
///
/// It is the one check here that passes by something being absent, and that is
/// why it is a check rather than a habit: nothing else would ever notice a
/// store committed beside the machine description, in the same folder, looking
/// exactly like the file that belongs there.
fn nobody_signs_in_with_an_account_the_image_shipped(image: &Image, wrong: &mut Vec<Wrong>) {
    if image.store().is_shipped() {
        wrong.push(Wrong::AnAccountShippedWithTheImage {
            at: image.store().at().to_owned(),
        });
    }
}

/// The daemon's environment is the session's: started by signing in, stopped by
/// signing out, and told where that session is.
///
/// The three states `crates/alo-secrets/tests/a_session_that_really_ended.rs`
/// measured and nothing was wired to. All three are decided in the unit file and
/// none of them can be caught by building it — an image whose agent service is
/// pulled in by `multi-user.target` boots perfectly and runs a daemon that has
/// no session, no bus and nobody signed in.
///
/// What the two variables must say is `alo-entering`'s, derived from the number
/// the machine description gives the person. That crate is also what the
/// **session** derives them from at a sign-in, so the unit shipped here and the
/// session a person really opens are held to one spelling rather than to two.
fn the_agent_runs_inside_the_persons_session(image: &Image, wrong: &mut Vec<Wrong>) {
    let agent = image.agent().called().to_owned();
    let theirs = TheSessionsEnvironment::for_person(image.description().person());
    let manager = theirs.their_manager();

    if !image.agent().wanted_by().contains(&manager.as_str()) {
        wrong.push(Wrong::TheAgentIsNotStartedBySigningIn {
            agent: agent.clone(),
            wanted_by: said(&image.agent().wanted_by()),
            manager: manager.clone(),
        });
    }
    if !image.agent().bound_to().contains(&manager.as_str())
        || !image.agent().after().contains(&manager.as_str())
    {
        wrong.push(Wrong::TheAgentDoesNotStopWithTheSession {
            agent: agent.clone(),
            manager,
        });
    }

    let stated = image.agent().environment();
    for (variable, value) in theirs.variables() {
        let named: Vec<&str> = stated
            .iter()
            .filter_map(|pair| pair.strip_prefix(variable))
            .filter_map(|rest| rest.strip_prefix('='))
            .collect();
        // Exactly one assignment, and it is the person's session. Two would be
        // a unit where the last one silently wins, and none is a service told
        // nothing about the session it is supposed to be inside.
        if named != vec![value.as_str()] {
            wrong.push(Wrong::TheAgentsEnvironmentIsNotTheSessions {
                agent: agent.clone(),
                variable: variable.to_owned(),
                named: said(&named),
                theirs: value,
            });
        }
    }
}

/// Whatever a unit named, as one sentence, or [`NOTHING`] where it named none.
fn said(named: &[&str]) -> String {
    if named.is_empty() {
        return NOTHING.to_owned();
    }
    named.join(" ")
}

/// ADR 0018 and ADR 0015: the boundary is on the kernel before the agent's
/// service exists, and it stays there after the loader has gone.
fn the_loader_runs_first(image: &Image, wrong: &mut Vec<Wrong>) {
    let loader = image.loader().called().to_owned();
    let agent = image.agent().called().to_owned();

    if !image.agent().needs().contains(&loader.as_str())
        || !image.agent().after().contains(&loader.as_str())
    {
        wrong.push(Wrong::TheAgentDoesNotWaitForTheBoundary {
            agent: agent.clone(),
            loader: loader.clone(),
        });
    }
    if image.loader().kind() != Some("oneshot") || !image.loader().stays_after_exiting() {
        wrong.push(Wrong::TheLoaderDoesNotStay {
            loader,
            kind: image.loader().kind().unwrap_or(NOTHING).to_owned(),
        });
    }
}

/// ADR 0018: root, in the agent's group, holding exactly two capabilities.
fn the_loader_holds_two_things(image: &Image, wrong: &mut Vec<Wrong>) {
    let loader = image.loader().called().to_owned();

    if image.loader().as_login() != Some(ROOT) {
        wrong.push(Wrong::TheLoaderIsNotRoot {
            loader: loader.clone(),
            as_login: image.loader().as_login().unwrap_or(NOTHING).to_owned(),
        });
    }

    let group = image.loader().in_group().unwrap_or(NOTHING);
    if image.group_called(group) != Some(image.description().group()) {
        wrong.push(Wrong::TheLoaderIsInTheWrongGroup {
            loader: loader.clone(),
            group: group.to_owned(),
        });
    }

    let mut held: Vec<String> = image
        .loader()
        .bounded_to()
        .into_iter()
        .map(str::to_owned)
        .collect();
    held.sort_unstable();
    if held != WHAT_THE_LOADER_MAY_HOLD || !image.loader().given().is_empty() {
        wrong.push(Wrong::TheLoaderHoldsSomethingElse { loader, held });
    }
}

/// **ADR 0024: the second privileged component holds no capability at all.**
///
/// It is root, because that is what `systemd-logind` decides `CreateSession`
/// on — measured on two systemds and written into `docs/quirks.md` — and it
/// holds nothing, because a uid is not a capability. That is the whole price of
/// the second privileged component this repository has, and this is the check
/// that keeps it at that price rather than at whatever somebody added to make
/// something work.
///
/// It is deliberately the *stronger* of the two forms `crate::Service` can ask
/// about: [`Service::holds_nothing`](crate::Service::holds_nothing) wants both
/// lines present and both empty, so a unit that merely stopped mentioning
/// capabilities is caught too. A service running as root with no
/// `CapabilityBoundingSet=` keeps every capability there is.
fn the_opener_holds_nothing_at_all(image: &Image, wrong: &mut Vec<Wrong>) {
    let opener = image.opener().called().to_owned();

    if image.opener().as_login() != Some(ROOT) {
        wrong.push(Wrong::TheOpenerIsNotRoot {
            opener: opener.clone(),
            as_login: image.opener().as_login().unwrap_or(NOTHING).to_owned(),
        });
    }

    if !image.opener().holds_nothing() {
        wrong.push(Wrong::TheOpenerHoldsSomething {
            opener,
            bounded: image
                .opener()
                .bounded_to()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            given: image
                .opener()
                .given()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        });
    }
}

/// **ADR 0024: the sign-in door is the greeter's, and the greeter is a login of
/// its own.**
///
/// `alo-sessiond` hands its door to whatever group it is running in and then
/// refuses every caller the kernel says is in another one, so the `Group=` line
/// is the whole of who may ask this machine for a session. Four ways that goes
/// wrong and none of them is visible at a build: a group the image does not
/// make, a group that is the person's or the agent's, a runtime directory that
/// is not where the code looks, and one open wider than the group.
fn the_opener_can_be_knocked_on_by_the_greeter_alone(image: &Image, wrong: &mut Vec<Wrong>) {
    let opener = image.opener().called().to_owned();

    let group = image.opener().in_group().unwrap_or(NOTHING);
    match image.group_called(group) {
        None => wrong.push(Wrong::TheOpenerIsNotInTheGreetersGroup {
            opener: opener.clone(),
            group: group.to_owned(),
        }),
        Some(greeter) => {
            for (number, whose) in [
                (image.description().person(), "the person"),
                (image.description().agent(), "the agent"),
                (image.description().group(), "the agent's group"),
            ] {
                if greeter == number {
                    wrong.push(Wrong::TheGreeterIsSomebodyElse {
                        greeter,
                        whose: whose.to_owned(),
                    });
                }
            }
        }
    }

    let looks = alo_sessiond::THE_DOORS_DIRECTORY;
    let made = image.opener().runtime_directories();
    if !made.iter().any(|it| looks == format!("{THE_RUN}/{it}")) {
        wrong.push(Wrong::TheOpenersDoorIsNotWhereItLooks {
            opener: opener.clone(),
            made: (*made.first().unwrap_or(&NOTHING)).to_owned(),
            looks: looks.to_owned(),
        });
    }

    if image.opener().runtime_directory_mode() != Some(THE_SIGN_IN_DOORS_MODE) {
        wrong.push(Wrong::TheOpenersDoorIsNotShut {
            opener,
            mode: image
                .opener()
                .runtime_directory_mode()
                .unwrap_or(NOTHING)
                .to_owned(),
            wanted: THE_SIGN_IN_DOORS_MODE.to_owned(),
        });
    }
}

/// ADR 0001 §2 and ADR 0018: the service that talks to the agent holds nothing,
/// and says so.
fn the_agent_holds_nothing(image: &Image, wrong: &mut Vec<Wrong>) {
    if !image.agent().holds_nothing() {
        wrong.push(Wrong::TheAgentHoldsSomething {
            agent: image.agent().called().to_owned(),
            bounded: image
                .agent()
                .bounded_to()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            given: image
                .agent()
                .given()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        });
    }
}

/// The machine description's three numbers are the accounts this image creates,
/// and the person can reach the group the socket is handed to.
fn the_logins_are_the_ones_this_image_makes(image: &Image, wrong: &mut Vec<Wrong>) {
    let person = image.agent().as_login().unwrap_or(NOTHING);
    if image.login_called(person) != Some(image.description().person()) {
        wrong.push(Wrong::TheAgentIsNotThePerson {
            agent: image.agent().called().to_owned(),
            as_login: person.to_owned(),
            person: image.description().person(),
        });
    }

    // The agent's login is the one thing the description names that no unit file
    // does, so what can be asked here is whether the image makes it at all. The
    // person and the agent being one login is the description's own refusal and
    // is tested where that refusal is made.
    if !image.makes_a_login_numbered(image.description().agent()) {
        wrong.push(Wrong::TheAgentHasNoLogin {
            agent: image.description().agent(),
        });
    }

    let group = image.loader().in_group().unwrap_or(NOTHING);
    if !image.agent().beside_groups().contains(&group) {
        wrong.push(Wrong::TheAgentIsNotInTheGroup {
            agent: image.agent().called().to_owned(),
            group: group.to_owned(),
        });
    }
    if !image.puts(person, group) {
        wrong.push(Wrong::ThePersonIsNotInTheGroup {
            login: person.to_owned(),
            group: group.to_owned(),
        });
    }
}

/// **Every number this image declares is one its own build asserts.**
///
/// This is the one check here that is about the build rather than about the
/// machine, and it is here because of what building the image measured. A
/// `sysusers.d` line is a **request**: on the pinned base `systemd-sysusers`
/// answers a number somebody else has by taking a different one, or by putting
/// the login into the group that already holds it — which is how alo OS's agent
/// was put into `systemd-resolve`'s group the first time this recipe was built,
/// with a green build and one line in a log (`docs/quirks.md`). The `test` lines
/// at the foot of `image/Containerfile` are the only place a request becomes a
/// fact, and nothing until now held the two files to each other: a sixth login
/// added to `sysusers.d` is asserted by nobody, and the failure it prevents is
/// silent by construction.
///
/// A number asserted as something **other** than what is declared is the same
/// finding and is worth as much: two files that disagree here are a build that
/// fails for a reason nobody can read, or worse, a machine description naming a
/// login the machine gave a different number to.
fn the_build_holds_every_login_to_its_number(image: &Image, wrong: &mut Vec<Wrong>) {
    for declared in image.declares() {
        match declared {
            crate::logins::Declared::Login { name, id, .. } => {
                let asserted = image.asserted().login_called(name);
                if asserted != Some(*id) {
                    wrong.push(Wrong::ALoginTheBuildDoesNotAssert {
                        login: name.clone(),
                        declared: *id,
                        asserted: said_number(asserted),
                    });
                }
            }
            crate::logins::Declared::Group { name, id } => {
                let asserted = image.asserted().group_called(name);
                if asserted != Some(*id) {
                    wrong.push(Wrong::AGroupTheBuildDoesNotAssert {
                        group: name.clone(),
                        declared: *id,
                        asserted: said_number(asserted),
                    });
                }
            }
            crate::logins::Declared::Member { login, group } => {
                if !image.asserted().puts(login, group) {
                    wrong.push(Wrong::AMembershipTheBuildDoesNotAssert {
                        login: login.clone(),
                        group: group.clone(),
                    });
                }
            }
        }
    }
}

/// A number the build insists on, in a sentence somebody reads — or the word
/// for a build that insists on nothing at all.
fn said_number(asserted: Option<u32>) -> String {
    asserted.map_or_else(|| "nothing".to_owned(), |number| number.to_string())
}

/// ADR 0017 and the machine description: the two directories `alo-agentd`
/// refuses to make are made by something.
fn the_directories_are_made(image: &Image, wrong: &mut Vec<Wrong>) {
    match image.directory_at(Path::new(THE_DOOR)) {
        None => wrong.push(Wrong::NotMadeAtBoot {
            at: Path::new(THE_DOOR).to_owned(),
        }),
        Some(door) => {
            if door.mode() != THE_DOORS_MODE || door.owner() != ROOT || door.group() != ROOT {
                wrong.push(Wrong::TheDoorIsNotWhatWasDecided {
                    at: door.at().to_owned(),
                    mode: door.mode(),
                    owner: door.owner().to_owned(),
                    group: door.group().to_owned(),
                });
            }
        }
    }

    let Some(folder) = image.description().record_folder() else {
        // A record path with no folder above it is a relative one, which the
        // daemon refuses; there is nothing here for an image to make.
        return;
    };
    let person = image.agent().as_login().unwrap_or(NOTHING);
    match image.directory_at(folder) {
        None => wrong.push(Wrong::NotMadeAtBoot {
            at: folder.to_owned(),
        }),
        Some(made) => {
            if made.owner() != person {
                wrong.push(Wrong::TheRecordFolderIsNotThePersons {
                    at: made.at().to_owned(),
                    owner: made.owner().to_owned(),
                    person: person.to_owned(),
                });
            }
        }
    }
}

/// The two places the daemon needs and cannot make, because it runs as a
/// person: a control group under its own, and the person's door.
///
/// Both are the same shape and both were found by booting the image rather than
/// by reading a file. `alo-agentd` holds no capability (ADR 0018) and is not
/// root (ADR 0001 §2), so everything it needs a privilege for is something
/// whoever starts it must have arranged — and a unit missing either line is a
/// machine that boots to a daemon that stops in milliseconds.
fn the_agent_can_make_what_it_is_not_given(image: &Image, wrong: &mut Vec<Wrong>) {
    let agent = image.agent().called().to_owned();

    if !image.agent().delegates_control_groups() {
        wrong.push(Wrong::TheAgentCannotMakeATurnsControlGroup {
            agent: agent.clone(),
        });
    }

    let person = image.description().person();
    let expected = the_persons_door(person);
    if !image
        .agent()
        .runtime_directories()
        .contains(&expected.as_str())
    {
        wrong.push(Wrong::ThePersonsDoorIsNotMade {
            agent: agent.clone(),
            expected,
            person,
            declared: image
                .agent()
                .runtime_directories()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        });
    }

    let mode = image.agent().runtime_directory_mode().unwrap_or(NOTHING);
    if mode != THE_PERSONS_DOORS_MODE {
        wrong.push(Wrong::ThePersonsDoorIsNotShut {
            agent,
            mode: mode.to_owned(),
        });
    }
}

/// Where this person's door goes, as `RuntimeDirectory=` spells it: relative to
/// `/run`, and named by the number the machine description gives the person.
fn the_persons_door(person: u32) -> String {
    format!(
        "{}/{person}",
        THE_DOOR.trim_start_matches(THE_RUN).trim_matches('/')
    )
}

/// ADR 0004: how long a record is kept is the organisation's to name, so the
/// only rule an image may ship is everything.
fn nobody_chose_a_retention(image: &Image, wrong: &mut Vec<Wrong>) {
    if let Keeping::ForDays(days) = image.description().keeping() {
        wrong.push(Wrong::ARetentionNobodyChose { days: days.get() });
    }
}

/// A unit nothing pulls in at boot is a unit that is shipped and never runs.
fn both_units_are_pulled_in(image: &Image, wrong: &mut Vec<Wrong>) {
    for service in [
        image.loader(),
        image.agent(),
        image.opener(),
        image.server(),
    ] {
        if service.wanted_by().is_empty() {
            wrong.push(Wrong::NothingPullsItIn {
                unit: service.called().to_owned(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{
        THE_AGENTS_UNIT, THE_BOOTING_DOCUMENT, THE_CONTAINERFILE, THE_DESCRIPTION_FILE,
        THE_LOADERS_UNIT, THE_OPENERS_UNIT, THE_SERVERS_UNIT, THE_SYSUSERS, THE_TMPFILES,
        a_copy_of_the_image, edited, image_at, the_store_file,
    };

    /// **The image this repository ships says one thing.** Everything below
    /// breaks one file of it and asks whether that is noticed, and none of those
    /// tests would mean anything without this one.
    #[test]
    fn the_image_this_repository_ships_agrees_with_itself() {
        let wrong = everything_wrong_with(&image_at(Path::new(crate::THE_IMAGE)));
        assert!(wrong.is_empty(), "{wrong:?}");
    }

    /// **An agent service that does not wait for the boundary is caught.** ADR
    /// 0015: a turn that cannot be bounded does not run, so a machine whose
    /// boundary never loaded must not reach a service that refuses every turn.
    #[test]
    fn an_agent_that_would_start_without_the_boundary_is_caught() {
        let root = a_copy_of_the_image("no-wait");
        edited(&root, THE_AGENTS_UNIT, "Requires=alo-boundaryd.service", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentDoesNotWaitForTheBoundary { .. })),
            "{wrong:?}"
        );
    }

    /// **A loader whose unit does not stay is caught.** The boundary outlives
    /// the process that loaded it, and a unit going inactive would tell
    /// `systemctl status` that a machine with a boundary has none.
    #[test]
    fn a_loader_that_does_not_stay_after_exiting_is_caught() {
        let root = a_copy_of_the_image("does-not-stay");
        edited(&root, THE_LOADERS_UNIT, "RemainAfterExit=yes", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheLoaderDoesNotStay { .. })),
            "{wrong:?}"
        );
    }

    /// **A loader in root's group is caught**, which is the mistake the loader
    /// itself refuses to start on: the map of turns is handed to whatever group
    /// this process is in, so root's group is a map only root could write.
    #[test]
    fn a_loader_in_roots_group_is_caught() {
        let root = a_copy_of_the_image("roots-group");
        edited(&root, THE_LOADERS_UNIT, "Group=alo-agent", "Group=root");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheLoaderIsInTheWrongGroup { .. })),
            "{wrong:?}"
        );
    }

    /// **A loader given a third capability is caught.** What makes one
    /// privileged component acceptable is the size of what it is trusted with,
    /// and this is the line where that is true rather than written down.
    #[test]
    fn a_loader_holding_more_than_adr_0018_gave_it_is_caught() {
        let root = a_copy_of_the_image("a-third");
        edited(
            &root,
            THE_LOADERS_UNIT,
            "CapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN",
            "CapabilityBoundingSet=CAP_BPF CAP_SYS_ADMIN CAP_NET_ADMIN",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheLoaderHoldsSomethingElse { .. })),
            "{wrong:?}"
        );
    }

    /// **An opener that is not root is caught.** `systemd-logind` answers an
    /// unprivileged caller `Access denied`, measured on two systemds, so an
    /// opener running as anybody else is a machine nobody can sign in to — and
    /// that arrives as a screen that does nothing rather than as an error.
    #[test]
    fn an_opener_that_is_not_root_is_caught() {
        let root = a_copy_of_the_image("opener-not-root");
        edited(&root, THE_OPENERS_UNIT, "User=root", "User=alo-greeter");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheOpenerIsNotRoot { .. })),
            "{wrong:?}"
        );
    }

    /// **An opener given a capability is caught.** This is the line ADR 0024's
    /// price is written on: the loader holds two capabilities and argues for
    /// them, and the second privileged component holds none — because what
    /// `logind` decides on is a uid.
    #[test]
    fn an_opener_given_a_capability_is_caught() {
        let root = a_copy_of_the_image("opener-a-capability");
        edited(
            &root,
            THE_OPENERS_UNIT,
            "AmbientCapabilities=",
            "AmbientCapabilities=CAP_SYS_ADMIN",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheOpenerHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **And an opener that merely stops saying it holds nothing is caught
    /// too.** A root process keeps every capability there is unless its unit
    /// says otherwise, so the two empty lines are the claim and deleting one is
    /// deleting it.
    #[test]
    fn an_opener_that_stops_saying_it_holds_nothing_is_caught() {
        let root = a_copy_of_the_image("opener-says-nothing");
        edited(
            &root,
            THE_OPENERS_UNIT,
            "CapabilityBoundingSet=",
            "# said nothing",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheOpenerHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **An opener in a group this image does not make is caught.** Its door is
    /// handed to whatever group it is in, so a name nothing creates is a
    /// service that will not start — and the failure arrives at a person's
    /// first sign-in rather than at a build.
    #[test]
    fn an_opener_in_a_group_this_image_does_not_make_is_caught() {
        let root = a_copy_of_the_image("opener-no-group");
        edited(
            &root,
            THE_OPENERS_UNIT,
            "Group=alo-greeter",
            "Group=alo-somebody",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheOpenerIsNotInTheGreetersGroup { .. })),
            "{wrong:?}"
        );
    }

    /// **A greeter that is the agent's group is caught.** The sign-in door
    /// would then be reachable by the agent's own service, which is ADR 0001
    /// §2's whole subject arriving through a `Group=` line.
    #[test]
    fn a_greeter_that_is_the_agents_group_is_caught() {
        let root = a_copy_of_the_image("greeter-is-the-agent");
        edited(
            &root,
            THE_OPENERS_UNIT,
            "Group=alo-greeter",
            "Group=alo-agent",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheGreeterIsSomebodyElse { .. })),
            "{wrong:?}"
        );
    }

    /// **A runtime directory that is not where `alo-sessiond` binds is
    /// caught.** The two are written in two files and nothing but this reads
    /// both; a door that is never opened is a machine that boots to a sign-in
    /// screen nothing answers.
    #[test]
    fn an_opener_whose_directory_is_not_where_its_door_goes_is_caught() {
        let root = a_copy_of_the_image("opener-elsewhere");
        edited(
            &root,
            THE_OPENERS_UNIT,
            "RuntimeDirectory=alo-sessiond",
            "RuntimeDirectory=alo-signing-in",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheOpenersDoorIsNotWhereItLooks { .. })),
            "{wrong:?}"
        );
    }

    /// **A sign-in door left open to everybody is caught.** The mode on the
    /// directory is the half the kernel enforces before anything in
    /// `alo-sessiond` is reached.
    #[test]
    fn an_opener_whose_directory_lets_everybody_in_is_caught() {
        let root = a_copy_of_the_image("opener-open");
        edited(
            &root,
            THE_OPENERS_UNIT,
            "RuntimeDirectoryMode=0750",
            "RuntimeDirectoryMode=0755",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheOpenersDoorIsNotShut { .. })),
            "{wrong:?}"
        );
    }

    /// **An agent service quietly given a capability is caught**, which is the
    /// exact shape ADR 0018 exists to prevent: not a privileged daemon, a
    /// directive added to an ordinary one.
    #[test]
    fn an_agent_service_given_a_capability_is_caught() {
        let root = a_copy_of_the_image("a-capability");
        edited(
            &root,
            THE_AGENTS_UNIT,
            "AmbientCapabilities=",
            "AmbientCapabilities=CAP_BPF",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **And an agent service that merely stops saying it holds nothing is
    /// caught too.** The two empty lines are the claim; deleting them leaves a
    /// service that holds nothing today and nothing saying it must.
    #[test]
    fn an_agent_service_that_stops_saying_so_is_caught() {
        let root = a_copy_of_the_image("stopped-saying");
        edited(&root, THE_AGENTS_UNIT, "CapabilityBoundingSet=", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **A description naming a person the image never creates is caught.**
    /// This is the disagreement a build cannot see and a boot turns into a
    /// daemon that stops.
    #[test]
    fn a_description_naming_a_login_nobody_makes_is_caught() {
        let root = a_copy_of_the_image("no-person");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "person = 1000",
            "person = 1001",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentIsNotThePerson { .. })),
            "{wrong:?}"
        );
    }

    /// **And a description whose agent number nothing makes is caught**, which
    /// is ADR 0001 §5: the agent is a login of its own, and SO_PEERCRED is the
    /// whole of the division between the socket's two doors.
    ///
    /// The number is one no login on this image has, and that has to be chosen
    /// rather than picked: 60991 used to be free and is the model service's, and
    /// a description naming *it* is caught by a different sentence —
    /// [`Wrong::TheServerIsSomebodyElse`], which says the truer thing, that the
    /// process holding the model would be the agent.
    #[test]
    fn a_description_whose_agent_is_no_login_is_caught() {
        let root = a_copy_of_the_image("no-agent-login");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "agent = 60989",
            "agent = 60992",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentHasNoLogin { .. })),
            "{wrong:?}"
        );
    }

    /// **A person left out of the agent's group is caught.** The socket is
    /// handed to that group, and changing a file's group is only allowed to a
    /// member of it — so this is a service that binds a door it cannot give away.
    #[test]
    fn a_person_who_cannot_reach_the_group_is_caught() {
        let root = a_copy_of_the_image("not-in-the-group");
        edited(&root, THE_AGENTS_UNIT, "SupplementaryGroups=alo-agent", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentIsNotInTheGroup { .. })),
            "{wrong:?}"
        );
    }

    /// And the same login outside the unit, which is the half a `systemd`
    /// directive does not cover.
    #[test]
    fn a_login_the_image_never_puts_in_the_group_is_caught() {
        let root = a_copy_of_the_image("no-membership");
        edited(&root, THE_SYSUSERS, "m alo alo-agent", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ThePersonIsNotInTheGroup { .. })),
            "{wrong:?}"
        );
    }

    /// **A sixth login nobody asserts is caught**, which is the mistake this
    /// check was written for: `sysusers.d` is edited, the build is green, and
    /// the machine has a login whose number is whatever was free on the base
    /// that day. Nothing about either file on its own is wrong.
    #[test]
    fn a_login_the_build_never_holds_to_its_number_is_caught() {
        let root = a_copy_of_the_image("unasserted-login");
        edited(
            &root,
            THE_SYSUSERS,
            "g alo-model 60991",
            "g alo-model 60991\nu alo-printing 60992 \"alo OS printing\" /var/lib/alo-printing /usr/sbin/nologin",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::ALoginTheBuildDoesNotAssert { login, .. } if login == "alo-printing"));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("nothing"), "{said}");
    }

    /// **A number the two files no longer agree on is caught.** The declaration
    /// moves, the assertion stays, and what fails is a build nobody can read —
    /// or, if the assertion is the one that moved, a machine whose description
    /// names a number the login does not have.
    #[test]
    fn a_number_the_build_asserts_and_the_image_does_not_declare_is_caught() {
        let root = a_copy_of_the_image("drifted-number");
        edited(
            &root,
            THE_SYSUSERS,
            "g alo-model 60991",
            "g alo-model 60891",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::AGroupTheBuildDoesNotAssert { group, .. } if group == "alo-model"));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("60991"), "{said}");
    }

    /// **A membership the build stopped checking is caught**, and this is the
    /// one that really happened: the first build of this image put alo OS's
    /// agent into `systemd-resolve`'s group, and the line that would have found
    /// it is the one deleted here.
    #[test]
    fn a_membership_the_build_stopped_checking_is_caught() {
        let root = a_copy_of_the_image("unasserted-membership");
        edited(&root, THE_CONTAINERFILE, "| grep -qx alo-agent", "| cat");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::AMembershipTheBuildDoesNotAssert { login, group }
                    if login == "alo" && group == "alo-agent"
            )),
            "{wrong:?}"
        );
    }

    /// **A door nothing makes is caught**, which is ADR 0017's whole
    /// consequence: `alo-agentd` refuses to make `/run/alo` and names it, and an
    /// image that forgot is a machine that boots to that sentence.
    #[test]
    fn a_door_nothing_makes_is_caught() {
        let root = a_copy_of_the_image("no-door");
        edited(&root, THE_TMPFILES, "d /run/alo 0755 root root -", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::NotMadeAtBoot { at } if at == Path::new(THE_DOOR))),
            "{wrong:?}"
        );
    }

    /// **A door left open is caught.** Every person's door goes in it, so
    /// anybody who could write it could replace anybody's socket.
    #[test]
    fn a_door_anybody_could_write_is_caught() {
        let root = a_copy_of_the_image("open-door");
        edited(
            &root,
            THE_TMPFILES,
            "d /run/alo 0755 root root -",
            "d /run/alo 0777 root root -",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDoorIsNotWhatWasDecided { mode: 0o777, .. })),
            "{wrong:?}"
        );
    }

    /// **An agent service that cannot make a turn's control group is caught.**
    /// This is half of what the first booted image failed on: without
    /// `Delegate=` the unit's cgroup belongs to root, the daemon's `mkdir`
    /// answers `EACCES`, and ADR 0015 stops the service rather than serving a
    /// turn nothing bounds.
    #[test]
    fn an_agent_that_cannot_make_a_control_group_is_caught() {
        let root = a_copy_of_the_image("no-delegation");
        edited(&root, THE_AGENTS_UNIT, "Delegate=yes", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentCannotMakeATurnsControlGroup { .. })),
            "{wrong:?}"
        );
    }

    /// **And an agent service whose person has no door is caught**, which is
    /// the other half: `/run/alo` is 0755 root:root on purpose, so the service
    /// cannot make its own name in it and something that is root must.
    #[test]
    fn an_agent_whose_person_has_no_door_is_caught() {
        let root = a_copy_of_the_image("no-persons-door");
        edited(&root, THE_AGENTS_UNIT, "RuntimeDirectory=alo/1000", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ThePersonsDoorIsNotMade { .. })),
            "{wrong:?}"
        );
    }

    /// **A door made for a different person is caught too**, and it is the
    /// mistake that looks right: a unit naming a number, and a description
    /// naming another one, with nothing between them but this.
    #[test]
    fn a_door_made_for_somebody_else_is_caught() {
        let root = a_copy_of_the_image("someone-elses-door");
        edited(
            &root,
            THE_AGENTS_UNIT,
            "RuntimeDirectory=alo/1000",
            "RuntimeDirectory=alo/1001",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ThePersonsDoorIsNotMade { person: 1000, .. })),
            "{wrong:?}"
        );
    }

    /// **A door left at systemd's default is caught.** 0755 is what a unit that
    /// says nothing gets, and the difference between that and 0750 is whether
    /// everybody on the machine can see whose agent is listening.
    #[test]
    fn a_persons_door_left_open_is_caught() {
        let root = a_copy_of_the_image("open-persons-door");
        edited(&root, THE_AGENTS_UNIT, "RuntimeDirectoryMode=0750", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ThePersonsDoorIsNotShut { .. })),
            "{wrong:?}"
        );
    }

    /// **A record folder nothing makes is caught**, which is the machine
    /// description's rule: the file is made on first start and the folder above
    /// it is not, so a typo there would become a second record nobody reads.
    #[test]
    fn a_record_folder_nothing_makes_is_caught() {
        let root = a_copy_of_the_image("no-record-folder");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "/var/lib/alo/record.jsonl",
            "/var/lib/somewhere-else/record.jsonl",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::NotMadeAtBoot { .. })),
            "{wrong:?}"
        );
    }

    /// **A record folder belonging to somebody else is caught.** What an agent
    /// did on somebody's machine is theirs, and a folder the service cannot
    /// write is a service that will not start.
    #[test]
    fn a_record_folder_that_is_not_the_persons_is_caught() {
        let root = a_copy_of_the_image("someone-elses-record");
        edited(
            &root,
            THE_TMPFILES,
            "d /var/lib/alo 0700 alo alo -",
            "d /var/lib/alo 0700 root root -",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheRecordFolderIsNotThePersons { .. })),
            "{wrong:?}"
        );
    }

    /// **A retention alo OS chose for somebody is caught.** ADR 0004 gives it to
    /// the organisation, and a number of days that sounded reasonable is exactly
    /// what `CLAUDE.md` refuses to ship.
    #[test]
    fn a_number_of_days_shipped_in_the_image_is_caught() {
        let root = a_copy_of_the_image("a-retention");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "keeping = \"forever\"",
            "keeping = { for-days = 90 }",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::ARetentionNobodyChose { days: 90 })),
            "{wrong:?}"
        );
    }

    /// **A unit nothing pulls in is caught**, which is the failure that looks
    /// most like success: the file is in the image, `systemctl cat` shows it,
    /// and nothing ever starts it.
    #[test]
    fn a_unit_nothing_pulls_in_is_caught() {
        let root = a_copy_of_the_image("never-started");
        edited(&root, THE_LOADERS_UNIT, "WantedBy=multi-user.target", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::NothingPullsItIn { .. })),
            "{wrong:?}"
        );
    }

    /// **An agent service started by booting rather than by signing in is
    /// caught.** This is the state the image really shipped in until the session
    /// was wired: a daemon pulled up by `multi-user.target`, running as the
    /// person before anybody had signed in as them, with no session and no bus.
    #[test]
    fn an_agent_started_before_anybody_signs_in_is_caught() {
        let root = a_copy_of_the_image("started-at-boot");
        edited(
            &root,
            THE_AGENTS_UNIT,
            "WantedBy=user@1000.service",
            "WantedBy=multi-user.target",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentIsNotStartedBySigningIn { .. })),
            "{wrong:?}"
        );
    }

    /// **And one that would outlive the session is caught.** `Requires=` is not
    /// `BindsTo=`: without the second, signing out leaves the agent service
    /// holding the person's door with nobody signed in.
    #[test]
    fn an_agent_that_would_outlive_the_session_is_caught() {
        let root = a_copy_of_the_image("outlives-the-session");
        edited(&root, THE_AGENTS_UNIT, "BindsTo=user@1000.service", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentDoesNotStopWithTheSession { .. })),
            "{wrong:?}"
        );
    }

    /// **And one started before the person's session exists is caught too**,
    /// which is the other half of the same line: `/run/user/1000` and the bus in
    /// it are made by the manager this ordering waits for.
    #[test]
    fn an_agent_started_before_the_session_exists_is_caught() {
        let root = a_copy_of_the_image("before-the-session");
        edited(&root, THE_AGENTS_UNIT, "After=user@1000.service", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentDoesNotStopWithTheSession { .. })),
            "{wrong:?}"
        );
    }

    /// **An agent service told nothing about a session is caught.** A system
    /// unit inherits none of the person's environment, so a missing line is a
    /// service running beside their session rather than inside it.
    #[test]
    fn an_agent_told_nothing_about_the_session_is_caught() {
        let root = a_copy_of_the_image("no-session-environment");
        edited(
            &root,
            THE_AGENTS_UNIT,
            "Environment=XDG_RUNTIME_DIR=/run/user/1000",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheAgentsEnvironmentIsNotTheSessions { variable, named, .. }
                    if variable == "XDG_RUNTIME_DIR" && named == NOTHING
            )),
            "{wrong:?}"
        );
    }

    /// **And one pointed at somebody else's session is caught**, which is the
    /// mistake that looks right: a number changed in one line of a unit file,
    /// and a daemon that runs as the person on root's bus.
    #[test]
    fn an_agent_pointed_at_another_logins_session_is_caught() {
        let root = a_copy_of_the_image("another-logins-session");
        edited(
            &root,
            THE_AGENTS_UNIT,
            "Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus",
            "Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/0/bus",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheAgentsEnvironmentIsNotTheSessions { variable, theirs, .. }
                    if variable == "DBUS_SESSION_BUS_ADDRESS"
                        && theirs == "unix:path=/run/user/1000/bus"
            )),
            "{wrong:?}"
        );
    }

    /// **And a description that moves the person leaves the unit behind**, which
    /// is what holds the two files to one number: every string here is derived
    /// from `[logins].person`, so changing it alone is an image whose session
    /// wiring names somebody who does not sign in.
    #[test]
    fn a_session_wired_for_a_person_the_description_no_longer_names_is_caught() {
        let root = a_copy_of_the_image("moved-the-person");
        edited(
            &root,
            THE_DESCRIPTION_FILE,
            "person = 1000",
            "person = 1001",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentIsNotStartedBySigningIn { .. })),
            "{wrong:?}"
        );
        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheAgentsEnvironmentIsNotTheSessions { .. })),
            "{wrong:?}"
        );
    }

    /// **An image carrying the accounts a person signs in with is caught.**
    /// The store goes in `/etc/alo` beside the machine description, so a store
    /// committed into `image/` would look exactly like the file that belongs
    /// there, would be copied by the same `COPY` line, and would hand every
    /// holder of the image a login on every machine built from it.
    #[test]
    fn an_image_that_ships_an_account_is_caught() {
        let root = a_copy_of_the_image("an-account");
        let written = std::fs::write(
            root.join(the_store_file()),
            "format = 1\n\n[[account]]\nname = \"alo\"\n",
        );
        assert!(
            written.is_ok(),
            "the folder the description ships in is there: {written:?}"
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::AnAccountShippedWithTheImage { .. })),
            "{wrong:?}"
        );
    }

    /// **An image that dropped the model runtime is caught.** The line is one
    /// `COPY` in a recipe nobody reviews twice, the build stays green, and the
    /// machine that ships is one whose sovereignty is an option to find — the
    /// exact sentence ADR 0025 was accepted to make false.
    #[test]
    fn an_image_that_dropped_the_runtime_is_caught() {
        let root = a_copy_of_the_image("no-runtime");
        edited(
            &root,
            THE_CONTAINERFILE,
            "COPY --from=runtime /runtime/bin/ollama /usr/bin/ollama",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheRuntimeIsNotOnTheImage { at }
                    if at == Path::new(crate::THE_RUNTIMES_BINARY)
            )),
            "{wrong:?}"
        );
    }

    /// **And one that dropped only the runtime's libraries is caught too**,
    /// which is the mistake that looks harmless: the binary is there, and it
    /// loads nothing.
    #[test]
    fn an_image_that_dropped_the_runtimes_libraries_is_caught() {
        let root = a_copy_of_the_image("no-runtime-libraries");
        edited(
            &root,
            THE_CONTAINERFILE,
            "COPY --from=runtime /runtime/lib/ollama/ /usr/lib/ollama/",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheRuntimeIsNotOnTheImage { at }
                    if at == Path::new(crate::THE_RUNTIMES_LIBRARIES)
            )),
            "{wrong:?}"
        );
    }

    /// **A runtime left floating is caught, in words that say why.** `latest`
    /// is the version somebody writes to get a build going, and it is a
    /// different runtime on two builds of one image.
    #[test]
    fn a_runtime_left_floating_is_caught() {
        let root = a_copy_of_the_image("floating-runtime");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_RUNTIME=0.34.0",
            "ARG THE_RUNTIME=latest",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::TheRuntimesVersionIsNotPinned { .. }));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("latest"), "{said}");
        assert!(said.contains("ADR 0006"), "{said}");
    }

    /// **A digest the build stopped checking is caught.** The `ARG` is still
    /// there, every grep for the pin still finds it, and it verifies nothing.
    #[test]
    fn a_runtime_arriving_unchecked_is_caught() {
        let root = a_copy_of_the_image("unchecked-runtime");
        edited(&root, THE_CONTAINERFILE, "sha256sum --check -", "true");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheRuntimeArrivesUnverified { .. })),
            "{wrong:?}"
        );
    }

    /// **An image that carries no weights is caught.** This is the state the
    /// image really shipped in until this check existed: the runtime aboard,
    /// pinned and verified, and nothing at all for it to load — which reads
    /// like a machine ready to run a model and is a machine that has to fetch
    /// one first.
    #[test]
    fn an_image_that_carries_no_weights_is_caught() {
        let root = a_copy_of_the_image("no-weights");
        edited(
            &root,
            THE_CONTAINERFILE,
            "COPY --from=weights /models/ /usr/share/alo/models/",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheWeightsAreNotOnTheImage { at } if at == Path::new(crate::THE_WEIGHTS)
            )),
            "{wrong:?}"
        );
    }

    /// **Weights fetched from a branch are caught.** A revision is one file
    /// forever and a branch is whatever the publisher pushed this morning; with
    /// a digest beside it, that is a release that stopped building rather than
    /// a mistake anybody noticed.
    #[test]
    fn weights_fetched_from_a_name_that_moves_are_caught() {
        let root = a_copy_of_the_image("moving-weights");
        edited(
            &root,
            THE_CONTAINERFILE,
            "resolve/a64113399c2f6b8ad3e11c394733a2ddadaa7f33/",
            "resolve/main/",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::TheWeightsAreNotPinned { .. }));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("moving name"), "{said}");
    }

    /// **Weights nothing checks are caught.** The `ARG` is still there and
    /// every grep for the pin still finds it — and what the image would carry
    /// is whatever the network handed the build machine.
    #[test]
    fn weights_arriving_unchecked_are_caught() {
        let root = a_copy_of_the_image("unchecked-weights");
        edited(
            &root,
            THE_CONTAINERFILE,
            "echo \"${THE_MODELS_SHA256}  /weights.gguf\" | sha256sum --check -",
            "true",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheWeightsArriveUnverified { .. })),
            "{wrong:?}"
        );
    }

    /// **Weights carried twice are caught.** This is what the first build of
    /// the real recipe shipped: the runtime's copy of the checked file left in
    /// the store beside the blob the manifest names, 2.23 GiB referenced by
    /// nothing on a read-only `/usr`. The edit that brings it back is the
    /// removal turned into a no-op, which is what a tidy-up of a shell line
    /// looks like.
    #[test]
    fn weights_carried_twice_are_caught() {
        let root = a_copy_of_the_image("weights-twice");
        edited(
            &root,
            THE_CONTAINERFILE,
            "rm -f \"/models/blobs/sha256-${THE_MODELS_SHA256}\"",
            "true",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheWeightsAreCarriedTwice)),
            "{wrong:?}"
        );
        assert!(
            !wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheStoreIsHeldToNothing)),
            "the store is still held to its manifest, and that is a separate finding: {wrong:?}"
        );
    }

    /// **A store held to nothing is caught**, separately: the removal above
    /// catches today's second copy, and only the walk over the store catches
    /// the one a runtime update leaves under a name nobody wrote down.
    #[test]
    fn a_store_held_to_nothing_is_caught() {
        let root = a_copy_of_the_image("store-unheld");
        edited(
            &root,
            THE_CONTAINERFILE,
            "for blob in /models/blobs/*",
            "for blob in /nowhere/*",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheStoreIsHeldToNothing)),
            "{wrong:?}"
        );
        assert!(
            !wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheWeightsAreCarriedTwice)),
            "the source is still dropped, and that is a separate finding: {wrong:?}"
        );
    }

    /// **A model the catalogue does not have is caught.** Weights nothing can
    /// look up are weights nobody can be told the licence, the cost or the
    /// measurement of, and the catalogue is where all three live.
    #[test]
    fn weights_naming_a_model_the_catalogue_never_heard_of_are_caught() {
        let root = a_copy_of_the_image("uncatalogued-weights");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_MODEL=phi-3-mini-instruct",
            "ARG THE_MODEL=something-somebody-liked",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheWeightsNameAModelTheCatalogueDoesNotHave { model }
                    if model == "something-somebody-liked"
            )),
            "{wrong:?}"
        );
    }

    /// **And a recipe that carries weights without saying which model they are
    /// is caught too**, which is the same mistake with the argument left blank
    /// rather than filled in wrongly.
    #[test]
    fn weights_that_do_not_say_which_model_they_are_are_caught() {
        let root = a_copy_of_the_image("unnamed-weights");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_MODEL=phi-3-mini-instruct",
            "ARG THE_MODEL=",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheImageDoesNotSayWhichModelItCarries)),
            "{wrong:?}"
        );
    }

    /// **A model nobody measured is caught.** `docs/features.md` promises a
    /// catalogue measured by us rather than claimed by the publisher, and the
    /// one model every machine arrives with is the last place to take a
    /// publisher's word for it — `mistral-7b-instruct` is catalogued, runs on
    /// the certified laptop, and nobody has put it to `alo-driving`.
    #[test]
    fn weights_naming_a_model_nobody_measured_are_caught() {
        let root = a_copy_of_the_image("unmeasured-weights");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_MODEL=phi-3-mini-instruct",
            "ARG THE_MODEL=mistral-7b-instruct",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::TheWeightsWereNeverMeasured { .. }));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("alo-driving"), "{said}");
    }

    /// **A quantisation the catalogue does not state is caught.** The entry a
    /// person reads and the name their machine answers to are one string or
    /// they are two different models, and nothing but this reads both.
    #[test]
    fn weights_that_are_not_the_artefact_the_catalogue_names_are_caught() {
        let root = a_copy_of_the_image("another-quantisation");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_MODELS_QUANTISATION=Q4_K_M",
            "ARG THE_MODELS_QUANTISATION=Q8_0",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheWeightsAreNotTheArtefactTheCatalogueNames { said, .. }
                    if said.contains("Q8_0")
            )),
            "{wrong:?}"
        );
    }

    /// **A model larger than the machine it ships on is caught.** One image is
    /// built and `docs/hardware.md` says which machine decides whether this
    /// product has a market: 16 GB and no card. `mixtral-8x7b-instruct` needs
    /// 48 GB and is graded `slow` without one, which is a machine that arrives
    /// ready to wait.
    #[test]
    fn weights_larger_than_the_certified_laptop_are_caught() {
        let root = a_copy_of_the_image("weights-too-large");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_MODEL=phi-3-mini-instruct",
            "ARG THE_MODEL=mixtral-8x7b-instruct",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::TheWeightsAreMoreThanTheMachineCanDrive { .. }));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("48"), "{said}");
    }

    /// **A licence that was not ours to hand on is caught**, and it is the
    /// refusal that looks most like success: `llama-3.2-3b-instruct` is
    /// catalogued, measured, small enough and comfortable on a laptop — and its
    /// licence attaches conditions to everybody the weights are passed on to.
    /// Carrying weights in an image *is* passing them on, so the catalogue may
    /// offer that model and the machine may not arrive with it.
    #[test]
    fn weights_under_a_licence_that_was_not_ours_to_hand_on_are_caught() {
        let root = a_copy_of_the_image("a-licence-with-conditions");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_MODEL=phi-3-mini-instruct",
            "ARG THE_MODEL=llama-3.2-3b-instruct",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong.iter().find(|it| {
            matches!(
                it,
                Wrong::TheWeightsCarryALicenceThatWasNotOursToHandOn { .. }
            )
        });
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("redistribution"), "{said}");
    }

    /// **A model service running as the person is caught.** It is up before
    /// anybody signs in and stays up after they sign out, so a process in their
    /// name outlives the session they ended — and on the machine that ships,
    /// `User=alo` is the one edit that would look like tidying up.
    #[test]
    fn a_model_service_running_as_the_person_is_caught() {
        let root = a_copy_of_the_image("server-is-the-person");
        edited(&root, THE_SERVERS_UNIT, "User=alo-model", "User=alo");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheServerIsSomebodyElse { whose, .. } if whose == "the person"
            )),
            "{wrong:?}"
        );
    }

    /// **And one running as a login this image never makes is caught**, which
    /// is the likelier half: a name somebody typed, a service that will not
    /// start, and a machine that otherwise boots perfectly.
    #[test]
    fn a_model_service_running_as_nobody_this_image_makes_is_caught() {
        let root = a_copy_of_the_image("server-is-nobody");
        edited(&root, THE_SERVERS_UNIT, "User=alo-model", "User=ollama");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServerIsNotALoginThisImageMakes { .. })),
            "{wrong:?}"
        );
    }

    /// **A model service in the agent's group is caught.** ADR 0001 §5 draws a
    /// division around that login, and a `Group=` line is all it takes to put
    /// the process holding the model inside it.
    #[test]
    fn a_model_service_in_the_agents_group_is_caught() {
        let root = a_copy_of_the_image("server-in-the-agents-group");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "Group=alo-model",
            "Group=alo-agent",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheServerSharesItsGroup { whose, .. } if whose == "the agent's"
            )),
            "{wrong:?}"
        );
    }

    /// **And a group this image does not make is caught too**, for the reason
    /// the opener's own group is: a service whose group does not exist is one
    /// that never starts.
    #[test]
    fn a_model_service_in_a_group_this_image_does_not_make_is_caught() {
        let root = a_copy_of_the_image("server-no-group");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "Group=alo-model",
            "Group=alo-somebody",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServerIsNotInAGroupOfItsOwn { .. })),
            "{wrong:?}"
        );
    }

    /// **A model service given a capability is caught**, and it is ADR 0018's
    /// shape a third time: not a privileged daemon, a directive added to an
    /// ordinary one to make something work.
    #[test]
    fn a_model_service_given_a_capability_is_caught() {
        let root = a_copy_of_the_image("server-a-capability");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "AmbientCapabilities=",
            "AmbientCapabilities=CAP_SYS_NICE",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServerHoldsSomething { .. })),
            "{wrong:?}"
        );
    }

    /// **A model service told nothing about where the weights are is caught.**
    /// Without the line the runtime uses its own default — a home directory
    /// this image does not make — and a machine carrying 2.23 GiB of model
    /// reports nothing installed.
    #[test]
    fn a_model_service_pointed_at_no_store_is_caught() {
        let root = a_copy_of_the_image("server-no-store");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "Environment=OLLAMA_MODELS=/usr/share/alo/models",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheServersStoreIsNotTheWeights { store, .. } if store == NOTHING
            )),
            "{wrong:?}"
        );
    }

    /// **And one pointed somewhere the weights did not land is caught too**,
    /// which is the same mistake with the path changed rather than deleted.
    #[test]
    fn a_model_service_pointed_at_the_wrong_store_is_caught() {
        let root = a_copy_of_the_image("server-elsewhere");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "OLLAMA_MODELS=/usr/share/alo/models",
            "OLLAMA_MODELS=/var/lib/alo-model/models",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServersStoreIsNotTheWeights { .. })),
            "{wrong:?}"
        );
    }

    /// **A model service bound to every interface is caught, and it is the most
    /// expensive line in the file.** One word: the machine's own model, offered
    /// to whatever network it is plugged into, to anybody, with no grant, no
    /// record and a dark egress indicator — because nothing left.
    #[test]
    fn a_model_service_bound_to_every_interface_is_caught() {
        let root = a_copy_of_the_image("server-on-the-network");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "OLLAMA_HOST=http://127.0.0.1:11434",
            "OLLAMA_HOST=http://0.0.0.0:11434",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        let found = wrong
            .iter()
            .find(|it| matches!(it, Wrong::TheServerIsNotWhereTheMachineLooks { .. }));
        assert!(found.is_some(), "{wrong:?}");
        let said = found.map(ToString::to_string).unwrap_or_default();
        assert!(said.contains("0.0.0.0"), "{said}");
    }

    /// **And a model service at an address this machine does not knock at is
    /// caught**, which is the quieter half of the same line: the runtime is
    /// serving, `alo-models` finds nothing, and the machine says no model is
    /// installed on a machine that shipped with one.
    #[test]
    fn a_model_service_at_a_port_nothing_knocks_at_is_caught() {
        let root = a_copy_of_the_image("server-another-port");
        edited(&root, THE_SERVERS_UNIT, ":11434", ":11435");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServerIsNotWhereTheMachineLooks { .. })),
            "{wrong:?}"
        );
    }

    /// **A model service that may reach the network is caught.** Law 1: a
    /// working day with a local model produces zero inference egress, and the
    /// one process holding the model is the one that has to be silent for that
    /// sentence to be true. Deleting the deny leaves the allow list filtering
    /// nothing at all, which is the mistake that looks like a tidy-up.
    #[test]
    fn a_model_service_that_may_reach_the_network_is_caught() {
        let root = a_copy_of_the_image("server-may-leave");
        edited(&root, THE_SERVERS_UNIT, "IPAddressDeny=any", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServerMayReachTheNetwork { .. })),
            "{wrong:?}"
        );
    }

    /// **And an allow list widened past this machine is caught too**, which is
    /// how an update check really arrives: not by deleting a line, by adding a
    /// name to one.
    #[test]
    fn a_model_service_allowed_somewhere_beyond_this_machine_is_caught() {
        let root = a_copy_of_the_image("server-allowed-out");
        edited(
            &root,
            THE_SERVERS_UNIT,
            "IPAddressAllow=localhost",
            "IPAddressAllow=localhost\nIPAddressAllow=any",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheServerMayReachTheNetwork { .. })),
            "{wrong:?}"
        );
    }

    /// **A model service that still asks its publisher is caught.** The
    /// filter keeps the two start-up requests from leaving; the runtime's own
    /// switch keeps them from being made, and a unit that dropped it would be
    /// a machine whose journal names the publisher every five minutes for as
    /// long as it is up. Deleted, and set to the wrong value: both are the
    /// same finding.
    #[test]
    fn a_model_service_that_still_asks_its_publisher_is_caught() {
        for (what, to) in [
            ("deleted", ""),
            ("left on", "Environment=OLLAMA_NO_CLOUD=0"),
        ] {
            let root = a_copy_of_the_image(&format!("server-asks-{}", what.replace(' ', "-")));
            edited(&root, THE_SERVERS_UNIT, "Environment=OLLAMA_NO_CLOUD=1", to);

            let wrong = everything_wrong_with(&image_at(&root));

            assert!(
                wrong.iter().any(|it| matches!(
                    it,
                    Wrong::TheServerStillAsksItsPublisher { server, .. }
                        if server == crate::THE_SERVER
                )),
                "the switch {what}: {wrong:?}"
            );
        }
    }

    /// **A model service nothing pulls in is caught**, which is this task's own
    /// failure mode: the unit is in the image, `systemctl cat` shows it, and a
    /// machine boots with 2.23 GiB of model and nothing serving it — the exact
    /// state the image really shipped in until this unit existed.
    #[test]
    fn a_model_service_nothing_starts_is_caught() {
        let root = a_copy_of_the_image("server-never-started");
        edited(&root, THE_SERVERS_UNIT, "WantedBy=multi-user.target", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(
                |it| matches!(it, Wrong::NothingPullsItIn { unit } if unit == crate::THE_SERVER)
            ),
            "{wrong:?}"
        );
    }

    /// **An image that stopped saying what disk it becomes is caught.** The
    /// label is one line in a recipe nobody reviews twice, and without it
    /// `docs/booting.md` is a document held to nothing.
    #[test]
    fn an_image_that_declares_no_disk_is_caught() {
        let root = a_copy_of_the_image("no-disk-declared");
        edited(
            &root,
            THE_CONTAINERFILE,
            "LABEL alo.disk.file=\"alo-os.raw\"",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheImageDoesNotSayWhatDiskItBecomes { label }
                    if label == crate::disk::THE_FILE
            )),
            "{wrong:?}"
        );
    }

    /// **A disk written by something other than the base's own tool is caught**,
    /// and that is the promise rather than a preference: a partitioner of ours
    /// lays out a disk nobody upstream ever tested, on the one part of the
    /// system whose mistakes appear only on somebody else's machine.
    #[test]
    fn a_disk_written_by_a_partitioner_of_ours_is_caught() {
        let root = a_copy_of_the_image("a-partitioner");
        edited(
            &root,
            THE_CONTAINERFILE,
            "LABEL alo.disk.tool=\"bootc install to-disk\"",
            "LABEL alo.disk.tool=\"sfdisk\"",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDiskIsNotWrittenByThePinnedBase { .. })),
            "{wrong:?}"
        );
    }

    /// **A base that stopped being pinned is caught, as the disk's problem.**
    /// The tool comes out of the base, so a tag that moved is a partitioner
    /// nobody chose writing the disk a machine boots from.
    #[test]
    fn a_base_on_a_tag_that_moves_is_caught_as_an_unpinned_tool() {
        let root = a_copy_of_the_image("unpinned-base");
        edited(&root, THE_CONTAINERFILE, "@sha256:", "@nothing:");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDiskIsNotWrittenByThePinnedBase { pinned: false, .. }
            )),
            "{wrong:?}"
        );
    }

    /// **A partitioner that arrives in the document is caught too.** It is the
    /// likelier half: not a second recipe, one helpful extra step added to the
    /// page somebody follows.
    #[test]
    fn a_partitioner_in_the_document_is_caught() {
        let root = a_copy_of_the_image("a-documented-partitioner");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "## What you need installed",
            "## What you need installed\n\nThen run `mkfs.ext4` on the second partition.\n",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(
                |it| matches!(it, Wrong::TheDiskWouldBeLaidOutByHand { by, .. } if by == "mkfs")
            ),
            "{wrong:?}"
        );
    }

    /// **A document that names a different firmware from the recipe is caught.**
    /// The person follows the document; nothing else in this repository reads
    /// the two together.
    #[test]
    fn a_document_naming_a_firmware_the_image_is_not_installed_for_is_caught() {
        let root = a_copy_of_the_image("another-firmware");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "firmware: uefi",
            "firmware: bios",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentDoesNotSayWhatTheRecipeDoes { fact, said, .. }
                    if fact == "firmware" && said == "bios"
            )),
            "{wrong:?}"
        );
    }

    /// **And a document telling somebody to select the other generation is
    /// caught**, which is the mistake that is one digit wide: a generation 1
    /// machine is the BIOS one, and pointed at this disk it finds nothing to
    /// boot at all.
    #[test]
    fn a_document_telling_somebody_the_wrong_generation_is_caught() {
        let root = a_copy_of_the_image("another-generation");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "generation: 2",
            "generation: 1",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheFirmwareIsNotWhatAPersonIsToldToSelect { generation, other, .. }
                    if generation == "1" && other == "bios"
            )),
            "{wrong:?}"
        );
    }

    /// **A document that names the tool and never runs it is caught.** One
    /// documented command turning the image into a disk is the whole of what
    /// that document is for.
    #[test]
    fn a_document_that_gives_no_command_is_caught() {
        let root = a_copy_of_the_image("no-command");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "      bootc install to-disk --via-loopback --wipe \\",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheDocumentDoesNotGiveTheCommand { .. })),
            "{wrong:?}"
        );
    }

    /// **A document missing one of its four answers is caught.** Somebody does
    /// this once, on a machine that has never run alo OS, and the document is
    /// the whole of what they have.
    #[test]
    fn a_document_missing_a_section_is_caught() {
        let root = a_copy_of_the_image("no-section");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "## What you need installed",
            "",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentIsMissingASection { heading }
                    if heading == "What you need installed"
            )),
            "{wrong:?}"
        );
    }

    /// **A document that stopped saying what a virtual machine cannot show is
    /// caught.** That paragraph is what stands between a virtual machine and
    /// somebody quoting it as the hardware acceptance in phase 8 — and the way
    /// it goes is one bullet at a time.
    #[test]
    fn a_document_that_stopped_naming_what_a_disk_cannot_show_is_caught() {
        let root = a_copy_of_the_image("no-limits");
        edited(
            &root,
            THE_BOOTING_DOCUMENT,
            "- **The GPU.** A virtual display adapter is not *the GPU works on first boot*.",
            "- **The display.** It is a synthetic adapter.",
        );

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheDocumentDoesNotSayWhatADiskCannotShow { about, .. } if about == "GPU"
            )),
            "{wrong:?}"
        );
    }

    /// **Everything wrong comes back at once**, rather than the first of it.
    /// An image is edited in one sitting and has five files in it.
    #[test]
    fn two_things_wrong_are_two_answers() {
        let root = a_copy_of_the_image("two-things");
        edited(&root, THE_LOADERS_UNIT, "WantedBy=multi-user.target", "");
        edited(&root, THE_TMPFILES, "d /run/alo 0755 root root -", "");

        let wrong = everything_wrong_with(&image_at(&root));

        assert!(wrong.len() >= 2, "{wrong:?}");
    }
}
