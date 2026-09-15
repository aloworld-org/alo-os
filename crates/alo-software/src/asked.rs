//! What the rented tool is asked, argument by argument.
//!
//! Every question and every act [`crate::Tool`] names, as the exact argument
//! list the tool is started with — built here from checked values, and never
//! from text. There is no shell between this list and the tool, so nothing in
//! it is interpreted: an identifier is one argument, a place's name is one
//! argument, and every one of them comes after `--`, which ends the tool's
//! options: whatever an identifier looks like, it is read as an identifier. A
//! place's name cannot begin with `-` either (`SourceName` refuses it), so it
//! is safe twice.
//!
//! **Machine-wide, and never asking.** Every call names the machine's own
//! installation (`--system`), where the places an organisation sets up live,
//! and every act is non-interactive: the tool is never left waiting on a
//! question in a terminal nobody is looking at. The person's question was the
//! approval, or their own choice in Settings, and it was asked before any of
//! this.
//!
//! Kept apart from starting the process ([`crate::rented`]) so the lists are
//! testable as lists on any machine — including one where the tool is not
//! installed.

use alo_applications::Application;

use crate::source::SourceName;

/// Where the rented tool is, on an alo OS machine.
pub const THE_TOOL: &str = "/usr/bin/flatpak";

/// Every place the machine's installation is set up with, one per line: name,
/// address, and the options the place was set up with.
#[must_use]
pub fn sources() -> Vec<String> {
    owned(&[
        "remotes",
        "--system",
        "--show-disabled",
        "--columns=name,url,options",
    ])
}

/// Every installed application, one identifier per line.
#[must_use]
pub fn installed() -> Vec<String> {
    owned(&["list", "--system", "--app", "--columns=application"])
}

/// Every application running, one identifier per line.
#[must_use]
pub fn open() -> Vec<String> {
    owned(&["ps", "--columns=application"])
}

/// Install this application from this place.
#[must_use]
pub fn install(source: &SourceName, application: &Application) -> Vec<String> {
    let mut asked = owned(&["install", "--system", "--noninteractive", "--app", "--"]);
    asked.push(source.as_str().to_owned());
    asked.push(application.identifier().to_owned());
    asked
}

/// Which installed applications this place has newer versions of.
#[must_use]
pub fn updates(source: &SourceName) -> Vec<String> {
    let mut asked = owned(&[
        "remote-ls",
        "--system",
        "--updates",
        "--app",
        "--columns=application",
        "--",
    ]);
    asked.push(source.as_str().to_owned());
    asked
}

/// Update this one application, and nothing else.
#[must_use]
pub fn update(application: &Application) -> Vec<String> {
    let mut asked = owned(&["update", "--system", "--noninteractive", "--app", "--"]);
    asked.push(application.identifier().to_owned());
    asked
}

/// Remove this one application.
#[must_use]
pub fn remove(application: &Application) -> Vec<String> {
    let mut asked = owned(&["uninstall", "--system", "--noninteractive", "--app", "--"]);
    asked.push(application.identifier().to_owned());
    asked
}

/// The same arguments, owned.
fn owned(arguments: &[&str]) -> Vec<String> {
    arguments
        .iter()
        .map(|&argument| argument.to_owned())
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn editor() -> Application {
        Application::identified("org.gnome.TextEditor").unwrap()
    }

    fn flathub() -> SourceName {
        SourceName::checked("flathub").unwrap()
    }

    #[test]
    fn each_act_names_one_application_and_one_place_as_single_arguments() {
        assert_eq!(
            install(&flathub(), &editor()),
            [
                "install",
                "--system",
                "--noninteractive",
                "--app",
                "--",
                "flathub",
                "org.gnome.TextEditor"
            ]
        );
        assert_eq!(
            update(&editor()),
            [
                "update",
                "--system",
                "--noninteractive",
                "--app",
                "--",
                "org.gnome.TextEditor"
            ]
        );
        assert_eq!(
            remove(&editor()),
            [
                "uninstall",
                "--system",
                "--noninteractive",
                "--app",
                "--",
                "org.gnome.TextEditor"
            ]
        );
        assert_eq!(
            updates(&flathub()),
            [
                "remote-ls",
                "--system",
                "--updates",
                "--app",
                "--columns=application",
                "--",
                "flathub"
            ]
        );
    }

    /// **Nothing the tool is asked waits on a terminal, restarts anything, or
    /// widens what the tool does**: every act is non-interactive and names one
    /// application, and no list holds an option that reinstalls, repairs,
    /// deletes data or skips a check.
    #[test]
    fn nothing_asked_waits_restarts_or_skips_a_check() {
        let every = [
            sources(),
            installed(),
            open(),
            install(&flathub(), &editor()),
            updates(&flathub()),
            update(&editor()),
            remove(&editor()),
        ];
        for asked in &every {
            for argument in asked {
                for forbidden in [
                    "reboot",
                    "restart",
                    "--reinstall",
                    "--delete-data",
                    "--no-deploy",
                    "--no-related",
                    "--or-update",
                    "--assumeyes",
                    "repair",
                    "--gpg",
                    "--no-gpg-verify",
                    "--user",
                    "kill",
                ] {
                    assert!(!argument.contains(forbidden), "{asked:?} holds {forbidden}");
                }
            }
        }
        for act in [
            install(&flathub(), &editor()),
            update(&editor()),
            remove(&editor()),
        ] {
            assert!(
                act.iter().any(|argument| argument == "--noninteractive"),
                "{act:?}"
            );
        }
    }

    /// **Nothing that reaches the tool can be read as an option.** A place's
    /// name that looks like one cannot be made, and an identifier that looks
    /// like one — which `alo-applications` has no reason to refuse — arrives
    /// after `--`, where the tool reads it as an identifier and finds nothing.
    #[test]
    fn nothing_that_could_be_read_as_an_option_is_asked_as_one() {
        assert!(SourceName::checked("--no-gpg-verify").is_none());
        let sneaky = Application::identified("--delete-data").unwrap();
        for act in [
            install(&flathub(), &sneaky),
            update(&sneaky),
            remove(&sneaky),
        ] {
            let ends = act.iter().position(|argument| argument == "--").unwrap();
            let at = act
                .iter()
                .position(|argument| argument == "--delete-data")
                .unwrap();
            assert!(ends < at, "{act:?}");
        }
    }
}
