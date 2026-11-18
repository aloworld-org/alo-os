//! Genuine Ask alo input and native consent for a private installed Studio.
//!
//! The person-side controller observes independent state, record, delivery and
//! process facts. This harness supplies only keyboard input and retained native
//! card pictures; it never sends an approval on the daemon protocol.

use std::time::Duration;

use alo_acting::said_on_the_log::A_CARD_IS_UP;

use crate::{Chord, Settling, Since, Walk};

/// One bounded wait for a genuine model response or native controller phase.
const WAIT: Duration = Duration::from_secs(300);

/// Run after provider setup and before any other model request on the image.
#[must_use]
pub fn after_provider(walk: Walk) -> Walk {
    walk.section_only_with(crate::studio_installed_walk::PRIVATE_STUDIO, |walk| {
        let walk = walk
            .press(Chord::key("esc"))
            .press(Chord::holding(&["ctrl", "alt"], "t"))
            .look("studio-approval-terminal", Settling::UNDER_KVM)
            .type_text(concat!(
                "(set -eu; d=$(mktemp -d); cd \"$d\"; ",
                "for f in studio.flatpak fixture.flatpak studio-session studio-agent-probe source.txt features.txt sdk-versions.txt checksums.txt; ",
                "do curl -fsS --max-time 120 http://10.0.2.2:8767/$f -o $f; done; ",
                "sha256sum -c checksums.txt; ",
                "STUDIO_PRIVATE_INSTALLED_WALK=1 python3 studio-session --approvals); ",
                "r=$?; printf 'Studio approved flow exit %s\\n' \"$r\" | systemd-cat -t studio-walk"
            ))
            .mark()
            .press(Chord::key("ret"));
        let walk = settings(walk, "initial");
        let walk = ask(walk, "enable", true, true);
        let walk = card(walk, "enable")
            .mark()
            .press(Chord::key("tab"))
            .press(Chord::key("ret"));
        let walk = observed(walk, "enable");
        let walk = ask(walk, "withdraw", false, false);
        let walk = card(walk, "withdraw")
            .press(Chord::with("meta_l", "i"))
            .expect_line(
                "the person revoked Studio while the same card was pending",
                "Studio approval withdraw grant revoked",
                Since::Mark,
                WAIT,
            )
            .press(Chord::key("esc"))
            .look("studio-withdraw-card-after-revocation", Settling::UNDER_KVM)
            .mark()
            .press(Chord::key("tab"))
            .press(Chord::key("ret"));
        let walk = settings(observed(walk, "withdraw"), "full");
        observed(ask(walk, "full", false, true), "full")
    })
}

/// Restore native settings even when a preceding approval phase failed.
#[must_use]
pub fn cleanup(walk: Walk) -> Walk {
    walk.section_only_with(crate::studio_installed_walk::PRIVATE_STUDIO, |walk| {
        settings(walk, "restore")
            .expect_line_saying(
                "native Studio approval, withdrawal and Full control complete",
                "Studio approved flow exit",
                "exit 0",
                Since::Mark,
                WAIT,
            )
            .look("studio-approval-completed", Settling::UNDER_KVM)
    })
}

/// Open the person's Settings only after the controller asks for this phase.
fn settings(walk: Walk, phase: &str) -> Walk {
    walk.expect_line(
        "the controller needs the person's native permission settings",
        &format!("Studio approval {phase} settings requested"),
        Since::Mark,
        WAIT,
    )
    .press(Chord::with("meta_l", "i"))
}

/// Submit a fixed microphone request after its independent observers are armed.
fn ask(walk: Walk, phase: &str, enabled: bool, close_settings: bool) -> Walk {
    let walk = walk.expect_line(
        "the independent observers are armed before the real request",
        &format!("Studio approval {phase} request ready"),
        Since::Mark,
        WAIT,
    );
    let walk = if close_settings {
        walk.press(Chord::key("esc"))
    } else {
        walk
    };
    walk.press(Chord::with("meta_l", "a"))
        .type_text(if enabled {
            "Enable the microphone in alo Studio."
        } else {
            "Disable the microphone in alo Studio."
        })
        .mark()
        .press(Chord::key("ret"))
}

/// Require a fresh native card and matching pending query, then retain its view.
fn card(walk: Walk, phase: &str) -> Walk {
    walk.expect_line_saying(
        "a fresh native card asks for the microphone change",
        A_CARD_IS_UP,
        "0 characters of text to type, whole",
        Since::Mark,
        WAIT,
    )
    .expect_line(
        "the pending number and exact microphone sentence were read back",
        &format!("Studio approval {phase} card verified"),
        Since::Mark,
        WAIT,
    )
    .look(
        &format!("studio-{phase}-approval-card"),
        Settling::UNDER_KVM,
    )
}

/// Wait for the controller's joined process, delivery, record and state evidence.
fn observed(walk: Walk, phase: &str) -> Walk {
    walk.expect_line(
        "actual process exit, recorder delivery, state and record agree",
        &format!("Studio approval {phase} observed"),
        Since::Mark,
        WAIT,
    )
}
