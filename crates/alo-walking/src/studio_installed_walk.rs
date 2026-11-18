//! Person-side installation and recording on an explicitly requested private VM.
//!
//! This command is typed into the person's terminal. It is not a broker verb,
//! uses native Settings for private agent grants, and does not claim decoded
//! media acceptance.

use std::time::Duration;

use crate::{Chord, Settling, Since, Walk};

/// Opt-in supplied only for a manually requested private package walk.
pub const PRIVATE_STUDIO: &str = "ALO_WALK_PRIVATE_STUDIO";

/// Extend the fresh-account walk after it opens the person's terminal.
#[must_use]
pub fn then_installed_studio(walk: Walk) -> Walk {
    walk.section_only_with(PRIVATE_STUDIO, |walk| {
        // Maximising returns focus to Window actions on the terminal's band.
        // Leave that band and prove the shell accepts input before installing.
        walk.press(Chord::key("esc"))
            .mark()
            .type_text("printf 'Studio terminal command accepted\\n' | systemd-cat -t studio-walk")
            .press(Chord::key("ret"))
            .expect_line(
                "the terminal accepts the private Studio command",
                "Studio terminal command accepted",
                Since::Mark,
                Duration::from_secs(30),
            )
            .mark()
            .type_text(concat!(
                "(set -eu; d=$(mktemp -d); cd \"$d\"; ",
                "for f in studio.flatpak fixture.flatpak studio-session studio-agent-probe source.txt features.txt sdk-versions.txt checksums.txt; ",
                "do curl -fsS --max-time 120 http://10.0.2.2:8767/$f -o $f; done; ",
                "sha256sum -c checksums.txt; ",
                "flatpak install --user -y --noninteractive studio.flatpak; ",
                "flatpak install --user -y --noninteractive fixture.flatpak; ",
                "STUDIO_PRIVATE_INSTALLED_WALK=1 python3 studio-session); ",
                "r=$?; if [ \"$r\" -ne 0 ]; then ",
                "printf 'Studio native permission settings failed\\n' | systemd-cat -t studio-walk; fi; ",
                "printf 'Studio installed flow exit %s\\n' \"$r\" | systemd-cat -t studio-walk"
            ))
            .press(Chord::key("ret"))
            .expect_line_saying(
                "the private controller requests native Settings",
                "Studio native permission settings",
                "requested",
                Since::Mark,
                Duration::from_secs(20 * 60),
            )
            .press(Chord::with("meta_l", "i"))
            .expect_line_saying(
                "the installed Studio person flow completes",
                "Studio installed flow exit",
                "exit 0",
                Since::Mark,
                Duration::from_secs(20 * 60),
            )
            .look("studio-installed-flow-completed", Settling::UNDER_KVM)
    })
}
