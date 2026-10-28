//! **A unit this image ships is a unit this image starts.**
//!
//! # The releases this is written for
//!
//! Two units reached `main` that nothing ever started, each from a change that
//! landed the half it was about:
//!
//! - `alo-compositor.service` says `WantedBy=multi-user.target` and was not in
//!   the recipe's `systemctl enable` list. Nothing enables it, no preset file
//!   exists in `image/`, and so systemd writes no `multi-user.target.wants`
//!   symlink — an installed machine has the compositor on disk and boots to no
//!   screen. The change that added it had been reported as *a machine boots to a
//!   sign-in screen*, which was inferred from the binary being present.
//! - `alo-portald.service` says `WantedBy=user@1000.service`, was not enabled,
//!   and `alo-portald` appeared nowhere in `image/` at all — not built, not
//!   copied, not made executable. Its unit started `/usr/libexec/alo-portald`,
//!   a file the image never created.
//!
//! Five other units were enabled explicitly, and `alo-convertd.service` is
//! socket-activated and correctly declares no `[Install]`. So the convention was
//! already right and these two were simply missed, which is exactly the kind of
//! thing a person does not notice by reading a recipe.
//!
//! # A third one, and it was in the blind spot of both checks above
//!
//! `alo-giving-a-folder@.service` shipped on 2026-10-17 and **nothing started
//! it**. The unit was right, the binary was on the image, the capability was
//! right, and the folder it exists to make was absent on every machine.
//!
//! It got past both tests here, and the reason is worth reading twice: it has
//! **no `[Install]` section at all**, so
//! `every_unit_that_asks_to_be_wanted_is_enabled` skips it — it is not asking to
//! be wanted. And its `ExecStart` program *is* put there by the recipe, so
//! `every_unit_starts_a_file_this_image_puts_there` passes. A unit asking for
//! nothing and starting a file that exists is a unit both of these call correct.
//!
//! It was found by a person thinking about it, which is not a mechanism.
//! `every_unit_with_no_install_section_is_started_by_something` is, and it knows
//! the three ways such a unit can be reachable: another unit names it, an
//! activator of the same stem exists (`alo-asking.socket` →
//! `alo-asking@.service`, by systemd's convention and not by being written
//! down), or a program builds the name — which is [`STARTED_BY_A_PROGRAM`],
//! because no reading of unit files can see it.
//!
//! # It needs no image build
//!
//! Both questions are answered by reading the unit files and the recipe as text,
//! so this runs in the ordinary gates. The image build is the thing nobody runs
//! until a release, which is how both of these survived.
//!
//! # What this does not check
//!
//! That the file an `ExecStart` names is *correct* — only that the recipe puts
//! something at that path. A recipe that copied the wrong binary there would
//! pass. The failure this exists for is the file being absent entirely.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The recipe, read rather than retyped.
const THE_RECIPE: &str = "image/Containerfile";

/// Where the units this image installs live.
const THE_UNITS: &str = "image/usr/lib/systemd/system";

/// Where the PAM stacks this image installs live.
///
/// `/usr/lib` and not `/etc`, because the base keeps its own `systemd-run0`
/// and `systemd-user` stacks there: a stack is what the image ships, and
/// `/etc` is what a person may edit.
const THE_STACKS: &str = "image/usr/lib/pam.d";

/// The workspace root, from this crate's own manifest.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// The recipe's text.
fn the_recipe() -> String {
    std::fs::read_to_string(the_repository().join(THE_RECIPE))
        .unwrap_or_else(|why| panic!("{THE_RECIPE} could not be read: {why}"))
}

/// Every unit the image installs, by name, with its text.
fn the_units() -> Vec<(String, String)> {
    let at = the_repository().join(THE_UNITS);
    let mut units: Vec<(String, String)> = std::fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{THE_UNITS} could not be read: {why}"))
        .map(|entry| {
            let entry = entry.expect("a readable directory entry");
            let name = entry.file_name().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(entry.path())
                .unwrap_or_else(|why| panic!("{name} could not be read: {why}"));
            (name, text)
        })
        .collect();
    units.sort();
    assert!(
        !units.is_empty(),
        "{THE_UNITS} holds no unit at all, which cannot be right"
    );
    units
}

/// The first value a unit gives a key, if it gives one.
fn value_of<'a>(unit: &'a str, key: &str) -> Option<&'a str> {
    unit.lines()
        .filter_map(|line| line.trim().strip_prefix(key))
        .map(str::trim)
        .next()
}

/// The one `systemctl enable` invocation, as a single line.
fn what_is_enabled(recipe: &str) -> String {
    let enabling: Vec<&str> = recipe
        .lines()
        .skip_while(|line| !line.contains("systemctl enable"))
        .take_while(|line| !line.contains("bootc container lint"))
        .collect();
    assert!(
        !enabling.is_empty(),
        "{THE_RECIPE} enables nothing, which cannot be right"
    );
    enabling.join(" ")
}

/// Units with no `[Install]` section that a **program** starts, and the file
/// that does it.
///
/// The third of the three ways a unit without an `[Install]` section can be
/// reachable, and the only one no amount of reading unit files can establish:
/// something in this workspace builds the name and asks systemd to start it.
///
/// **It is a list because the alternative was a weak grep.** Searching Rust for
/// a unit's stem finds doc comments, crate names and task reports —
/// `alo-giving-a-folder` appears as a *crate* name in a test that has nothing to
/// do with starting it. A list names the file, and
/// `every_unit_said_to_be_started_by_a_program_is_named_in_that_program` reads
/// that file for the name, so the claim is checked rather than taken.
const STARTED_BY_A_PROGRAM: [(&str, &str); 4] = [
    (
        "alo-applying-an-update.service",
        "crates/alo-brokerd/src/units.rs",
    ),
    ("alo-going-back.service", "crates/alo-brokerd/src/units.rs"),
    (
        "alo-giving-a-folder@.service",
        "crates/alo-sessiond/src/starting_their_units.rs",
    ),
    (
        "alo-giving-a-name@.service",
        "crates/alo-sessiond/src/starting_their_units.rs",
    ),
];

/// **A unit that asks to be wanted is a unit the recipe enables.**
///
/// `WantedBy=` is a request, not an effect: it only does anything once something
/// enables the unit, and in this image that something is the recipe. A unit with
/// an `[Install]` section that nothing enables is a unit that never runs, and it
/// looks exactly like one that does.
#[test]
fn every_unit_that_asks_to_be_wanted_is_enabled() {
    let recipe = the_recipe();
    let enabled = what_is_enabled(&recipe);
    let mut asked = 0_usize;
    for (name, text) in the_units() {
        if value_of(&text, "WantedBy=").is_none() {
            continue;
        }
        asked += 1;
        assert!(
            enabled.contains(&name),
            "{name} says WantedBy= and {THE_RECIPE} never enables it, \
             so the image installs a unit that never starts"
        );
    }
    assert!(
        asked >= 6,
        "only {asked} units asked to be wanted, which is fewer than this image has"
    );
}

/// **A unit starts a file this image puts there.**
///
/// An `ExecStart` naming a path the recipe never creates is a service that fails
/// at boot with `203/EXEC`, on a machine where every gate was green.
#[test]
fn every_unit_starts_a_file_this_image_puts_there() {
    let recipe = the_recipe();
    let mut started = 0_usize;
    for (name, text) in the_units() {
        let Some(command) = value_of(&text, "ExecStart=") else {
            continue;
        };
        let program = command
            .split_whitespace()
            .next()
            .unwrap_or_else(|| panic!("{name} has an ExecStart= with no program"));
        started += 1;
        assert!(
            recipe.contains(program),
            "{name} starts {program}, and {THE_RECIPE} never puts a file there"
        );
    }
    assert!(
        started >= 7,
        "only {started} units started anything, which is fewer than this image has"
    );
}

/// **A unit that names a PAM stack is a unit whose stack this image ships.**
///
/// `PAMName=` is the same shape of promise as `ExecStart=`: it names a file by
/// convention, and systemd resolves it at *run* time against the running
/// machine. A unit naming a stack the image never installs starts a service
/// that cannot open a session, and the failure arrives two layers down as an
/// errno from whatever wanted the session.
///
/// # The release this is written for
///
/// `alo-compositor.service` takes its card from the login seat through
/// `libseat`, which asks `logind` what session the calling process is in. It
/// had `After=systemd-logind.service` and a comment reasoning that logind
/// answering was enough. **Ordering is not membership**: a system service is
/// in no session, so logind answered and there was still nothing to ask
/// about. Measured on a written disk on 2026-10-05 -- *display session
/// connect failed: Function not implemented*, four restarts out of four, on a
/// machine whose every other unit had started.
///
/// The cure is `PAMName=`, and the fault this test exists for is the one that
/// would follow it: a unit naming a stack no recipe copies in.
#[test]
fn a_unit_that_names_a_pam_stack_ships_that_stack() {
    let at = the_repository().join(THE_STACKS);
    let recipe = the_recipe();
    let mut named = Vec::new();

    for (unit, text) in the_units() {
        let Some(stack) = value_of(&text, "PAMName=") else {
            continue;
        };
        named.push(stack.to_owned());
        assert!(
            at.join(stack).is_file(),
            "{unit} names the PAM stack `{stack}` and {THE_STACKS}/{stack} is not a file; systemd resolves PAMName= on the running machine, so a stack the image never installs is a session the service cannot open"
        );
        assert!(
            recipe.contains("COPY image/usr/lib/pam.d/"),
            "{unit} names a PAM stack and {THE_RECIPE} never copies {THE_STACKS} into the image, so the file exists in this repository and nowhere a machine can read it"
        );
        assert!(
            recipe.contains(&format!("/usr/lib/pam.d/{stack}")),
            "{THE_RECIPE} copies the stacks but never names {stack} in its chmod list, and every other file this image installs is given its mode explicitly"
        );
    }

    // The other direction: a stack nothing asks for is a stack nobody reads.
    let shipped = std::fs::read_dir(&at)
        .unwrap_or_else(|why| panic!("{THE_STACKS} could not be read: {why}"));
    for entry in shipped {
        let entry = entry.expect("a readable directory entry");
        let file = entry.file_name().to_string_lossy().into_owned();
        assert!(
            named.contains(&file),
            "{THE_STACKS}/{file} is shipped and no unit names it with PAMName=, so nothing on the machine will ever read it"
        );
    }
}

/// **A unit with no `[Install]` section is a unit something else starts.**
///
/// The direction the two tests above cannot see. `WantedBy=` is a request the
/// recipe has to grant, and that is checked; **a unit with no `[Install]` at all
/// is not making a request**, so it is skipped by that test entirely — and its
/// `ExecStart` program is on the image, so it is skipped by the other one too.
/// It ships, it loads, `systemctl status` shows it as a perfectly ordinary
/// inactive unit, and nothing will ever start it.
///
/// This is not hypothetical. `alo-giving-a-folder@.service` shipped in that
/// state: the unit, its binary and its capability were all correct, and nothing
/// on the machine asked for it, so the folder it exists to make was absent on
/// every machine. It was found by a person thinking about it rather than by any
/// check, which is why this one exists.
///
/// # The three ways a unit without an `[Install]` can be reachable
///
/// 1. **Another unit names it** — `Unit=`, `Wants=`, `Requires=`, `Also=`. A
///    timer's or a path unit's service is this.
/// 2. **A socket, path or timer of the same stem activates it by convention.**
///    `alo-asking.socket` starts `alo-asking@.service` without naming it
///    anywhere: systemd's rule is the socket's name plus `@`. A check that
///    demanded the name be written down would fail on the one unit in this image
///    that is correct by convention.
/// 3. **A program builds the name and asks systemd**, which is
///    [`STARTED_BY_A_PROGRAM`] because no reading of unit files can see it.
///
/// A unit covered by none of the three fails here, and the message says what to
/// do: start it, or add it to the list with the program that does.
#[test]
fn every_unit_with_no_install_section_is_started_by_something() {
    let units = the_units();
    let shipped: Vec<&String> = units.iter().map(|(name, _)| name).collect();

    let mut without = 0_usize;
    let mut unreachable = Vec::new();
    for (name, text) in &units {
        if text.contains("[Install]") {
            continue;
        }
        without += 1;

        // 1. another unit names it exactly.
        if units
            .iter()
            .any(|(other, body)| other != name && body.contains(name.as_str()))
        {
            continue;
        }

        // 2. an activator of the same stem ships beside it.
        let stem = name
            .strip_suffix(".service")
            .unwrap_or(name)
            .trim_end_matches('@');
        if [".socket", ".path", ".timer"]
            .iter()
            .any(|kind| shipped.iter().any(|it| *it == &format!("{stem}{kind}")))
        {
            continue;
        }

        // 3. a program starts it, which is written down.
        if STARTED_BY_A_PROGRAM
            .iter()
            .any(|(listed, _)| listed == name)
        {
            continue;
        }

        unreachable.push(name.clone());
    }

    assert!(
        without >= 3,
        "only {without} units ship without an `[Install]` section, and this image had four on \
         2026-10-17 — so this test is reading less than it thinks"
    );
    assert!(
        unreachable.is_empty(),
        "these units ship with no `[Install]` section and nothing starts them, so they load on \
         a machine and never run:\n  {}\n\n\
         A unit like this is invisible to `every_unit_that_asks_to_be_wanted_is_enabled` — it \
         asks for nothing — and to `every_unit_starts_a_file_this_image_puts_there`, whose \
         program is present. Either have something start it (another unit, a socket of the same \
         stem, or a program) or add it to `STARTED_BY_A_PROGRAM` in this file with the file that \
         does.",
        unreachable.join("\n  ")
    );
}

/// **Every unit said to be started by a program is named in that program.**
///
/// What makes [`STARTED_BY_A_PROGRAM`] a check rather than an assertion. A line
/// in that list is a claim about a file, and a claim nobody reads is how the
/// not-yet-shipped lists elsewhere in this repository would rot if they were not
/// each held to the disk.
///
/// A template is matched on its stem, because the program builds the instance —
/// `format!("{THE_UNIT}@{person}.service")` — and never writes `@.service`.
#[test]
fn every_unit_said_to_be_started_by_a_program_is_named_in_that_program() {
    let root = the_repository();
    for (unit, by) in STARTED_BY_A_PROGRAM {
        let at = root.join(by);
        let said = std::fs::read_to_string(&at)
            .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));

        let looked_for = unit.strip_suffix("@.service").map_or(unit, |stem| stem);
        assert!(
            said.contains(looked_for),
            "{by} is named as what starts {unit}, and it does not mention {looked_for:?} at all"
        );
    }
}

/// **And nothing in that list has grown an `[Install]` section.**
///
/// The other half, and the same rule the unshipped lists follow: an entry that
/// has stopped being true is an entry somebody will read as current. A unit that
/// gained `WantedBy=` is enabled by the recipe and checked by the first test in
/// this file, so its line here is then about nothing.
#[test]
fn nothing_started_by_a_program_has_grown_an_install_section() {
    let units = the_units();
    let mut asking = Vec::new();
    for (unit, _) in STARTED_BY_A_PROGRAM {
        let Some((_, text)) = units.iter().find(|(name, _)| name == unit) else {
            panic!("{unit} is named in `STARTED_BY_A_PROGRAM` and the image does not ship it");
        };
        if text.contains("[Install]") {
            asking.push(unit);
        }
    }
    assert!(
        asking.is_empty(),
        "these are listed as started by a program and now have an `[Install]` section, so the \
         recipe is what must enable them and the lines here are about nothing: {asking:?}"
    );
}
