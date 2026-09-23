//! The menu a machine with two systems on it starts at, generated as
//! configuration for the base's own loader.
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! decided that the menu is alo OS's and that Windows stands behind it. This is
//! that menu: an entry that hands the firmware's own signed Windows program
//! over, a short countdown, and the last choice taken from — and written back
//! to — the loader's environment block (`crate::saved`).
//!
//! # Configured, never patched
//!
//! [ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md):
//! the base is rented, and its loader is configured rather than changed. What
//! is written is a **whole file of ours**, beside the base's own generated
//! configuration, which the base's configuration already reads if it is there.
//! Nothing here edits a line the base wrote, and a base update that replaces
//! the loader leaves this file where it is.
//!
//! # Windows is found by its own program, not by a disk this file names
//!
//! The entry searches for the file Windows starts from and hands that over.
//! Naming a partition here would mean keeping an identifier that the installer
//! learned once and that this file could not check afterwards — a second copy
//! of a fact about somebody's disk, of exactly the kind
//! [`crate::TheStartingChoice`] exists to refuse. Searching for the program
//! itself is the same answer computed each time the machine starts.
//!
//! # The title is a sentence somebody translates
//!
//! A menu title is the first thing a person reads on a machine that has just
//! been turned on, so it is filled from the machine's own vocabulary
//! (`crate::words::THE_WINDOWS_ENTRY_TITLE`) rather than written here in
//! English. [`Menu::offering`] refuses a title carrying anything the
//! configuration format would read as punctuation of its own, because a
//! translation is a file somebody else wrote and a menu that will not parse is
//! a machine that starts at a loader's prompt.
//!
//! # What has not been watched
//!
//! Nothing here has run on a machine. `docs/booting.md` says what is owed and
//! who owes it: the walk belongs to the virtual machine on the development PC,
//! and what the base's loader does with a saved default across an update is
//! measured there rather than assumed here.

use std::fmt::Write as _;

use crate::chosen::{SAVED_ENTRY, THE_BLOCK_ON_THE_ESP};
use crate::systems::{THE_WINDOWS_ENTRY, THE_WINDOWS_LOADER};

/// What this configuration calls the EFI system partition while it is using it.
///
/// A loader variable, not a name anything outside this file knows: the
/// partition is found at every start by the one file on it this configuration
/// wants, so nothing here keeps a disk, a number or a label of somebody's
/// machine ([`crate::THE_BLOCK_ON_THE_ESP`]).
const PARTITION: &str = "esp";

/// The one file this menu is written to, on a machine.
///
/// The base's own generated configuration reads it if it is there, which is
/// what makes this configuration rather than a change to the base.
pub const THE_MENU: &str = "/boot/grub2/custom.cfg";

/// How long the menu waits before starting the last-chosen system, in seconds.
///
/// ADR 0062 asks for *a short countdown*. Five seconds is long enough to reach
/// for a key from across a desk and short enough that a machine somebody
/// restarted and walked away from comes back on its own.
pub const THE_COUNTDOWN: u8 = 5;

/// The longest a title may be, in characters.
///
/// A loader draws a menu in a fixed-width console of eighty columns, and a
/// title longer than this has nowhere to go on the narrowest screen alo OS
/// starts on.
pub const LONGEST_TITLE: usize = 48;

/// Why a menu was not generated.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAMenu {
    /// A title that is empty, too long, or carrying something the
    /// configuration format reads as punctuation of its own.
    #[error(
        "{title:?} cannot be a title in a start-up menu: a title is one line of at most \
         {LONGEST_TITLE} characters, and holds none of a quote, a backslash, a brace or a dollar"
    )]
    NotATitle {
        /// The title that was offered.
        title: String,
    },

    /// A countdown of nothing, or one nobody would wait through.
    #[error(
        "{seconds} seconds is not a countdown a person can use: a menu nobody can see is not a \
         menu, and a machine that waits longer than {LONGEST_COUNTDOWN} seconds has not started"
    )]
    NotACountdown {
        /// The countdown that was offered.
        seconds: u8,
    },
}

/// The longest countdown a menu may have, in seconds.
pub const LONGEST_COUNTDOWN: u8 = 60;

/// The menu this machine starts at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    /// What the Windows entry is called, in the reader's own language.
    windows_titled: String,
    /// How long it waits.
    countdown: u8,
}

impl Menu {
    /// A menu offering alo OS and the Windows beside it, with this title on the
    /// Windows entry and this countdown.
    ///
    /// alo OS's own entries are the base's, made from the kernels on the
    /// machine, and are not written here: what this adds to them is the second
    /// system, the countdown and the saved default.
    ///
    /// # Errors
    /// [`NotAMenu::NotATitle`] for a title that is empty, longer than
    /// [`LONGEST_TITLE`], or holding a character the configuration format would
    /// read as its own; [`NotAMenu::NotACountdown`] for a countdown of nothing
    /// or one longer than [`LONGEST_COUNTDOWN`].
    pub fn offering(windows_titled: &str, countdown: u8) -> Result<Self, NotAMenu> {
        let title = windows_titled.trim();
        let holds_its_own = title.chars().any(|each| {
            each.is_control() || matches!(each, '\'' | '"' | '\\' | '{' | '}' | '$' | '`')
        });
        if title.is_empty() || title.chars().count() > LONGEST_TITLE || holds_its_own {
            return Err(NotAMenu::NotATitle {
                title: windows_titled.to_owned(),
            });
        }
        if countdown == 0 || countdown > LONGEST_COUNTDOWN {
            return Err(NotAMenu::NotACountdown { seconds: countdown });
        }
        Ok(Self {
            windows_titled: title.to_owned(),
            countdown,
        })
    }

    /// What the Windows entry is called.
    #[must_use]
    pub fn windows_titled(&self) -> &str {
        &self.windows_titled
    }

    /// How long the menu waits, in seconds.
    #[must_use]
    pub const fn countdown(&self) -> u8 {
        self.countdown
    }

    /// Whether this configuration offers Windows.
    ///
    /// The one question anything asks of a menu after it is written, and it is
    /// asked of the **file** rather than of a setting beside it: a machine alo
    /// OS replaced Windows on has no Windows entry, and a record of that kept
    /// anywhere else would be a second copy of a fact about somebody's disk
    /// ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
    /// term 3).
    ///
    /// What is looked for is the identifier [`Menu::written`] gives the entry,
    /// never its title: the title is a sentence somebody translated, and a
    /// machine whose menu is in Greek offers exactly the same Windows.
    #[must_use]
    pub fn offers_windows(configuration: &str) -> bool {
        configuration.contains(&format!("--id '{THE_WINDOWS_ENTRY}'"))
    }

    /// The configuration, as the file holds it.
    #[must_use]
    pub fn written(&self) -> String {
        let mut out = String::new();
        // Writing into a `String` cannot fail. Every result is taken rather
        // than discarded so that nothing here is an ignored error.
        let written = write!(
            out,
            "# alo OS writes this file, whole, every time it writes it. It is generated, and\n\
             # never edited by hand: anything put here is gone the next time it is written.\n\
             #\n\
             # The loader itself is the base's, configured and never changed (ADR 0011). This\n\
             # is the configuration: the menu a machine with two systems on it starts at,\n\
             # which ADR 0062 decided is alo OS's, with Windows standing behind it.\n\
             #\n\
             # The last choice is on the EFI system partition (ADR 0066 term 1), which is\n\
             # found here by the block itself rather than by an identifier of somebody's\n\
             # disk written down once — the same answer computed at every start. A machine\n\
             # whose block is not there sets nothing, and starts the first entry: alo OS.\n\
             \n\
             search --no-floppy --set={PARTITION} --file {block}\n\
             load_env -f (${{{PARTITION}}}){block} {SAVED_ENTRY}\n\
             set default=\"${{{SAVED_ENTRY}}}\"\n\
             set timeout_style=menu\n\
             set timeout={countdown}\n\
             \n\
             menuentry '{title}' --class windows --id '{entry}' {{\n\
             \tinsmod part_gpt\n\
             \tinsmod fat\n\
             \tinsmod chain\n\
             \tsearch --no-floppy --set=root --file {loader}\n\
             \tchainloader {loader}\n\
             \tset {SAVED_ENTRY}='{entry}'\n\
             \tsave_env -f (${{{PARTITION}}}){block} {SAVED_ENTRY}\n\
             }}\n",
            countdown = self.countdown,
            title = self.windows_titled,
            entry = THE_WINDOWS_ENTRY,
            loader = THE_WINDOWS_LOADER,
            block = THE_BLOCK_ON_THE_ESP,
        );
        debug_assert!(written.is_ok());
        out
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A menu as a machine would generate one.
    fn a_menu() -> Menu {
        Menu::offering("Windows", THE_COUNTDOWN).unwrap()
    }

    /// **The menu offers Windows by handing its own program over**, counts
    /// down, and takes its default from the loader's saved choice.
    #[test]
    fn the_menu_offers_windows_counts_down_and_reads_the_saved_choice() {
        let written = a_menu().written();
        assert!(written.contains("menuentry 'Windows'"), "{written}");
        assert!(
            written.contains(&format!("chainloader {THE_WINDOWS_LOADER}")),
            "{written}"
        );
        assert!(written.contains("set timeout=5"), "{written}");
        assert!(written.contains("set timeout_style=menu"), "{written}");
        assert!(
            written.contains(&format!("load_env -f (${{esp}}){THE_BLOCK_ON_THE_ESP}")),
            "{written}"
        );
        assert!(
            written.contains("set default=\"${saved_entry}\""),
            "{written}"
        );
        assert!(
            written.contains(&format!(
                "save_env -f (${{esp}}){THE_BLOCK_ON_THE_ESP} saved_entry"
            )),
            "{written}"
        );
        assert!(
            written.contains(&format!("set saved_entry='{THE_WINDOWS_ENTRY}'")),
            "{written}"
        );
    }

    /// **Neither environment line goes anywhere but the EFI system partition.**
    ///
    /// A bare `load_env` or `save_env` reads and writes `$prefix/grubenv`,
    /// which is the loader's directory under `/boot` — alo OS's own filesystem,
    /// which Windows cannot write. That is where this file kept the last choice
    /// before ADR 0066, and the way back to it is a line somebody shortens.
    #[test]
    fn no_environment_line_falls_back_to_the_loaders_own_directory() {
        for line in a_menu().written().lines().map(str::trim) {
            if line.starts_with("load_env") || line.starts_with("save_env") {
                assert!(
                    line.contains(&format!("(${{esp}}){THE_BLOCK_ON_THE_ESP}")),
                    "{line:?} keeps the last choice somewhere Windows cannot reach"
                );
            }
        }
    }

    /// **The partition is found by the file on it**, never by a disk, a
    /// partition number or a label written down when the machine was installed.
    #[test]
    fn the_partition_is_found_by_the_block_and_not_by_an_identifier() {
        let written = a_menu().written();
        assert!(
            written.contains(&format!(
                "search --no-floppy --set=esp --file {THE_BLOCK_ON_THE_ESP}"
            )),
            "{written}"
        );
        assert!(!written.contains("--fs-uuid"), "{written}");
        assert!(!written.contains("hd0"), "{written}");
    }

    /// **A generated menu offers Windows, and anything else does not.** The
    /// question is asked of the file, in the identifier the menu itself writes,
    /// so a menu whose title was translated still answers yes and a machine alo
    /// OS replaced Windows on answers no.
    #[test]
    fn a_generated_menu_offers_windows_and_nothing_else_does() {
        assert!(Menu::offers_windows(&a_menu().written()));
        for title in ["Παράθυρα", "ウィンドウズ"] {
            let menu = Menu::offering(title, THE_COUNTDOWN).unwrap();
            assert!(Menu::offers_windows(&menu.written()), "{title}");
        }
        for not_a_menu in [
            "",
            "# nothing to choose between\n",
            "menuentry 'Windows' {\n\tchainloader /somewhere/else.efi\n}\n",
            &format!("# {THE_WINDOWS_ENTRY}\n"),
        ] {
            assert!(
                !Menu::offers_windows(not_a_menu),
                "{not_a_menu:?} was read as offering Windows"
            );
        }
    }

    /// **The file says it is generated, in its first two lines**, so whoever
    /// opens it on a machine is told before they edit it that an edit will not
    /// last.
    #[test]
    fn the_file_says_it_is_generated_before_anything_else() {
        let written = a_menu().written();
        let first = written.lines().take(2).collect::<Vec<&str>>().join(" ");
        assert!(first.contains("generated"), "{first}");
        assert!(first.starts_with('#'), "{first}");
    }

    /// **Nothing in it mounts anything.** Windows is reached by handing its own
    /// program over, which is what ADR 0062 decided; a menu that mounted the
    /// Windows volume to find it would be exactly the thing alo OS promises
    /// never to do.
    #[test]
    fn nothing_in_the_menu_mounts_anything() {
        let written = a_menu().written().to_lowercase();
        for never in ["mount", "ntfs", "rw ", "rw,", "fuse", "linux ", "initrd "] {
            assert!(!written.contains(never), "the menu says {never:?}");
        }
    }

    /// **A title the format would read as punctuation of its own is refused**,
    /// not escaped. A translation is a file somebody else wrote, and the
    /// failure it would cause is a machine that starts at a prompt instead of a
    /// menu — on the one screen nobody can reach a tool from.
    #[test]
    fn a_title_that_would_break_the_menu_is_refused() {
        for title in [
            "",
            "   ",
            "Wind'ows",
            "Wind\"ows",
            "Wind\\ows",
            "Windows {",
            "Windows }",
            "Windows $1",
            "Windows `x`",
            "Wind\nows",
            "Wind\u{7}ows",
            &"W".repeat(LONGEST_TITLE + 1),
        ] {
            assert_eq!(
                Menu::offering(title, THE_COUNTDOWN),
                Err(NotAMenu::NotATitle {
                    title: title.to_owned()
                }),
                "{title:?} was taken as a title"
            );
        }
    }

    /// **A countdown nobody can use is refused.** Zero is a menu that is never
    /// seen, which is the requirement ADR 0062 states undone in one number.
    #[test]
    fn a_countdown_nobody_can_use_is_refused() {
        for seconds in [0, LONGEST_COUNTDOWN + 1, u8::MAX] {
            assert_eq!(
                Menu::offering("Windows", seconds),
                Err(NotAMenu::NotACountdown { seconds }),
                "{seconds} was taken as a countdown"
            );
        }
        assert!(Menu::offering("Windows", 1).is_ok());
        assert!(Menu::offering("Windows", LONGEST_COUNTDOWN).is_ok());
    }

    /// **A title in another language is a title.** The refusal above is about
    /// the format's own punctuation and nothing else — a check that refused
    /// anything but ASCII would be this crate deciding which languages a menu
    /// may be read in.
    #[test]
    fn a_title_in_another_language_is_a_title() {
        for title in ["Fenster", "Παράθυρα", "Windows®", "Виндоус", "ウィンドウズ"]
        {
            let menu = Menu::offering(title, THE_COUNTDOWN).unwrap();
            assert!(
                menu.written().contains(&format!("menuentry '{title}'")),
                "{title}"
            );
        }
    }

    /// **The title is taken as it was meant, with the space around it gone**,
    /// because a title that begins with a space reads as a menu drawn wrongly.
    #[test]
    fn a_title_is_taken_without_the_space_around_it() {
        let menu = Menu::offering("  Windows  ", THE_COUNTDOWN).unwrap();
        assert_eq!(menu.windows_titled(), "Windows");
        assert_eq!(menu.countdown(), THE_COUNTDOWN);
    }
}
