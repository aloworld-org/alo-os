//! **A machine that boots alo OS says it is alo OS**, and stops promising
//! Fedora's support.
//!
//! # What was measured, on a written disk, 2026-10-05
//!
//! The base's own `/usr/lib/os-release` calls the machine *Fedora Linux 42
//! (Adams)*, and a person who installed alo OS did not install Fedora. GRUB
//! offers it, systemd greets with it, and the login prompt reads `fedora
//! login:`.
//!
//! It is not only a name. That file carries `SUPPORT_END=2026-05-13`, so systemd
//! prints, in red, on every boot:
//!
//! ```text
//! !! This OS version (Fedora Linux 42 (Adams)) is past its end-of-support
//!    date (2026-05-13)
//! ```
//!
//! **That is a support promise this project never made and cannot expire**, told
//! to our own customer about our own product. And it is a trademark matter
//! rather than a preference: the Fedora Project asks that a modified remix not
//! carry its marks.
//!
//! # Why a test and not a careful change
//!
//! Three of the facts here are the kind that go stale silently.
//!
//! - **The copy has to come after the last `dnf`.** Package resolution reads
//!   `VERSION_ID` and `PLATFORM_ID` out of this file, so a copy that landed
//!   earlier would have `dnf` reasoning about a release called `alo`. Nothing
//!   about the recipe makes that ordering visible, and somebody tidying the
//!   `COPY` lines together would undo it without noticing.
//! - **The absence of `SUPPORT_END` is the whole point**, and an absence is what
//!   no reader of a diff ever checks. Re-adding it would restore the red line.
//! - **The release is stated in two files.** The recipe labels the image
//!   `0.0.6` and this file says so four more times. A version that drifts
//!   between them is a machine that disagrees with its own registry page.
//!
//! # It needs no image build
//!
//! Both files are read as text, so this runs in the ordinary gates — which is
//! where it has to run, because the image build is the thing nobody runs until a
//! release.
//!
//! # What this does not check
//!
//! That the base really is Fedora 42, or that `dnf` is happy. Those are answered
//! by the build. This answers *does the recipe say what we decided it says*.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_appearance::Role;

/// The recipe, read rather than retyped.
const THE_RECIPE: &str = "image/Containerfile";

/// What the machine says it is, as this repository writes it.
const WHAT_IT_SAYS: &str = "image/usr/lib/os-release";

/// Where it lands on the machine.
const WHERE_IT_LANDS: &str = "/usr/lib/os-release";

/// The field a base states its own end of support in.
///
/// Named here so that the test for its absence says what is absent.
const A_SUPPORT_PROMISE: &str = "SUPPORT_END";

/// One of the two files, as text.
fn read(what: &str) -> String {
    let at = Path::new(alo_image::THE_IMAGE)
        .parent()
        .expect("the image directory has a parent")
        .join(what);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} does not read: {why}", at.display()))
}

/// Every field the file states, in the order it states them, with the quotes
/// taken off.
///
/// Comments and blank lines are dropped. A value is unquoted because
/// `os-release` permits either spelling and nothing here should care which was
/// used.
fn every_field(text: &str) -> Vec<(String, String)> {
    let mut stated = Vec::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((field, value)) = line.split_once('=') else {
            panic!("a line of {WHAT_IT_SAYS} states no field: {line}");
        };
        let value = value.trim().trim_matches('"').to_owned();
        stated.push((field.trim().to_owned(), value));
    }
    stated
}

/// What the file says one field is, where it says it once.
fn says(stated: &[(String, String)], field: &str) -> String {
    let mut found = stated.iter().filter(|(name, _)| name == field);
    let value = found
        .next()
        .unwrap_or_else(|| panic!("{WHAT_IT_SAYS} states no {field}"));
    assert!(
        found.next().is_none(),
        "{WHAT_IT_SAYS} states {field} more than once, so it states nothing"
    );
    value.1.clone()
}

/// **The machine says it is alo OS, by name and by identifier.**
#[test]
fn the_machine_says_it_is_alo_os() {
    let stated = every_field(&read(WHAT_IT_SAYS));
    assert_eq!(says(&stated, "NAME"), "alo OS");
    assert_eq!(says(&stated, "ID"), "alo");
    assert_eq!(says(&stated, "DEFAULT_HOSTNAME"), "alo");
}

/// **Nothing a person reads says Fedora**, in any spelling.
///
/// `ID_LIKE`, `VERSION_ID` and `PLATFORM_ID` are exempt and are checked
/// separately: those are what the machine is built on, stated plainly, and a
/// derivative that hides its base makes every later question about it harder to
/// answer.
#[test]
fn nothing_a_person_reads_says_fedora() {
    let stated = every_field(&read(WHAT_IT_SAYS));
    let the_base_is_named_in = ["ID_LIKE", "VERSION_ID", "PLATFORM_ID"];
    for (field, value) in &stated {
        if the_base_is_named_in.contains(&field.as_str()) {
            continue;
        }
        assert!(
            !value.to_ascii_lowercase().contains("fedora"),
            "{field} says Fedora to a person: {value}"
        );
    }
}

/// **It makes no support promise**, which is the red line this change exists to
/// stop.
///
/// An absence, asserted, because an absence is what no reader of a diff checks.
#[test]
fn it_makes_no_support_promise() {
    let stated = every_field(&read(WHAT_IT_SAYS));
    assert!(
        !stated.iter().any(|(field, _)| field == A_SUPPORT_PROMISE),
        "{WHAT_IT_SAYS} states {A_SUPPORT_PROMISE}, so systemd tells the owner \
         in red on every boot that alo OS is out of support"
    );
    // And the base's own support fields are gone with it, rather than left
    // pointing at somebody else's bug tracker.
    for field in ["REDHAT_BUGZILLA_PRODUCT", "REDHAT_SUPPORT_PRODUCT"] {
        assert!(
            !stated.iter().any(|(stated, _)| stated == field),
            "{WHAT_IT_SAYS} still states {field}"
        );
    }
}

/// **What the machine is built on is stated plainly**, rather than hidden.
///
/// Engines are configured, never patched: this image *is* built on Fedora 42,
/// package tooling reasons about `$releasever` and the platform identifier, and
/// removing these would be a decision somebody has to make on purpose.
#[test]
fn what_it_is_built_on_is_stated_plainly() {
    let stated = every_field(&read(WHAT_IT_SAYS));
    assert_eq!(says(&stated, "ID_LIKE"), "fedora");
    assert_eq!(says(&stated, "VERSION_ID"), "42");
    assert_eq!(says(&stated, "PLATFORM_ID"), "platform:f42");
}

/// **The release it says it is, is the release the recipe builds** — in all four
/// places it is written.
#[test]
fn the_release_it_says_it_is_matches_the_recipe() {
    let recipe = read(THE_RECIPE);
    let label = format!("{}=", alo_image::THE_VERSION_LABEL);
    let stated_by_the_recipe: Vec<&str> = recipe
        .lines()
        .filter_map(|line| line.trim().strip_prefix("LABEL "))
        .filter_map(|rest| rest.trim().strip_prefix(label.as_str()))
        .map(|value| value.trim().trim_matches('"'))
        .collect();
    // One, matched as one: two releases stated are two answers to one question,
    // which read as none.
    let [release] = stated_by_the_recipe.as_slice() else {
        panic!(
            "the recipe states its release {} times, and it must state it once",
            stated_by_the_recipe.len()
        )
    };

    let stated = every_field(&read(WHAT_IT_SAYS));
    assert_eq!(says(&stated, "VERSION"), *release);
    assert_eq!(says(&stated, "PRETTY_NAME"), format!("alo OS {release}"));
    assert_eq!(
        says(&stated, "CPE_NAME"),
        format!("cpe:/o:aloworld:alo_os:{release}")
    );
}

/// **The recipe copies it, and after every `dnf`.**
///
/// Asked of the whole recipe rather than of the last stage, which is the
/// stricter question and the one that keeps being true: a `dnf` added below this
/// line in any stage, later, fails here.
#[test]
fn the_recipe_copies_it_after_every_dnf() {
    let recipe = read(THE_RECIPE);
    let mut copies_at = None;
    let mut last_dnf_at = None;
    for (at, raw) in recipe.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with("COPY")
            && line.contains(WHAT_IT_SAYS)
            && line.split_whitespace().next_back() == Some(WHERE_IT_LANDS)
        {
            assert!(
                copies_at.is_none(),
                "the recipe copies {WHAT_IT_SAYS} more than once"
            );
            copies_at = Some(at);
        }
        if line.contains("dnf ") {
            last_dnf_at = Some(at);
        }
    }
    let copies_at = copies_at.unwrap_or_else(|| {
        panic!("the recipe never copies {WHAT_IT_SAYS} to {WHERE_IT_LANDS}, so the machine keeps the base's name")
    });
    let last_dnf_at =
        last_dnf_at.expect("the recipe installs no packages at all, which cannot be right");
    assert!(
        copies_at > last_dnf_at,
        "the recipe copies {WHAT_IT_SAYS} at line {} and still runs dnf at line {} — package \
         resolution would read VERSION_ID and PLATFORM_ID out of our file",
        copies_at + 1,
        last_dnf_at + 1
    );
}

/// **It is made readable**, like every other file the recipe places.
///
/// A file only root can read is a file `systemd` greets nobody with.
#[test]
fn it_is_made_readable() {
    let recipe = read(THE_RECIPE);
    assert!(
        recipe
            .lines()
            .any(|line| line.trim().starts_with(WHERE_IT_LANDS)
                || line.trim().starts_with(&format!("{WHERE_IT_LANDS} \\"))),
        "the recipe never makes {WHERE_IT_LANDS} readable"
    );
}

/// **The colour it tells a terminal to use is the one the palette reserves for
/// the agent**, rather than a second spelling of it.
///
/// `ANSI_COLOR` is a 24-bit SGR sequence — `0;38;2;R;G;B` — and the three
/// numbers are read back and compared with [`Role::AccentDefault`], so the
/// palette moving is the palette moving in one place.
#[test]
fn the_colour_it_names_is_the_palettes_own() {
    let stated = every_field(&read(WHAT_IT_SAYS));
    let ansi = says(&stated, "ANSI_COLOR");
    let channels: Vec<&str> = ansi.split(';').collect();
    // Matched as a shape rather than counted and then indexed: the pattern is
    // the assertion that it is `0;38;2;R;G;B` and nothing else.
    let [_reset, kind, bits, red, green, blue] = channels.as_slice() else {
        panic!("ANSI_COLOR is not a 24-bit sequence: {ansi}")
    };
    assert_eq!(
        (*kind, *bits),
        ("38", "2"),
        "ANSI_COLOR does not set a foreground colour in 24 bits: {ansi}"
    );
    let accent = Role::AccentDefault.colour();
    assert_eq!(
        (*red, *green, *blue),
        (
            accent.red().to_string().as_str(),
            accent.green().to_string().as_str(),
            accent.blue().to_string().as_str(),
        ),
        "ANSI_COLOR is not the accent the palette reserves for the agent"
    );
}
