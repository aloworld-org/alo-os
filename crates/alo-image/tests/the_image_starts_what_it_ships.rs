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
