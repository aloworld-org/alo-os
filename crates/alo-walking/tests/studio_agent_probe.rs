//! Contract for private installed-agent probe checks, written before the probe.
//!
//! Input is an observed /proc/self/status or an actual typed daemon reply.
//! Synthetic observations here establish validator behavior only. They do not
//! establish a guest launch, kernel separation, a held connection, native
//! Settings approval, actual Studio execution, or real-model acceptance.

#![expect(
    clippy::expect_used,
    reason = "an invalid literal protocol fixture must fail this independent specification"
)]

use std::collections::BTreeSet;

use alo_adapters::studio::STUDIO;
use alo_protocol::{Done, ToAnAgent};
use alo_walking::studio_agent_probe::{
    Stage, require_confinement, require_discovery, require_refused,
};

// Deliberately independent of STUDIO, ACTION_NAMES, and the future validator.
const NAMES: [&str; 19] = [
    "studio.status",
    "studio.new_recording",
    "studio.choose_screen",
    "studio.choose_window",
    "studio.start",
    "studio.pause",
    "studio.resume",
    "studio.finish",
    "studio.enable_microphone",
    "studio.disable_microphone",
    "studio.set_quality",
    "studio.countdown_off",
    "studio.countdown_three",
    "studio.countdown_five",
    "studio.cancel_countdown",
    "studio.show_library",
    "studio.review",
    "studio.rename",
    "studio.export",
];

// Groups and unrelated kernel fields are present deliberately: credentials are
// the four real/effective/saved/filesystem UID/GID columns, not an occurrence
// of 60989 somewhere in the text. No secret or real guest status is copied.
const CONFINED: &str = concat!(
    "Name:\tstudio-probe\nState:\tR (running)\n",
    "Uid:\t60989\t60989\t60989\t60989\n",
    "Gid:\t60989\t60989\t60989\t60989\nGroups:\t\n",
    "CapInh:\t0000000000000000\nCapPrm:\t0000000000000000\n",
    "CapEff:\t0000000000000000\nCapBnd:\t0000000000000000\n",
    "CapAmb:\t0000000000000000\nNoNewPrivs:\t1\nSeccomp:\t2\n",
);

fn discovery(names: impl IntoIterator<Item = impl AsRef<str>>) -> ToAnAgent {
    ToAnAgent::Capabilities {
        verbs: names
            .into_iter()
            .map(|name| name.as_ref().to_owned())
            .collect(),
    }
}

fn zero_stages() -> [Stage; 3] {
    [Stage::AskBeforeGrant, Stage::Revoked, Stage::AskAgain]
}

fn all_stages() -> [Stage; 5] {
    [
        Stage::AskBeforeGrant,
        Stage::Granted,
        Stage::Revoked,
        Stage::Full,
        Stage::AskAgain,
    ]
}

#[test]
fn only_the_fully_dropped_agent_identity_is_confined() {
    assert!(require_confinement(CONFINED).is_ok());
    for field in ["Uid", "Gid"] {
        for column in 0..4 {
            for foreign in ["0", "1000", "60990"] {
                let mut ids = ["60989"; 4];
                if let Some(id) = ids.get_mut(column) {
                    *id = foreign;
                }
                let changed = CONFINED.replace(
                    &format!("{field}:\t60989\t60989\t60989\t60989"),
                    &format!("{field}:\t{}", ids.join("\t")),
                );
                assert!(
                    require_confinement(&changed).is_err(),
                    "{field} column {column}={foreign}"
                );
            }
        }
    }
    for field in ["CapInh", "CapEff", "CapPrm", "CapAmb", "CapBnd"] {
        for nonzero in ["0000000000000001", "0000000080000000", "ffffffffffffffff"] {
            let changed = CONFINED.replace(
                &format!("{field}:\t0000000000000000"),
                &format!("{field}:\t{nonzero}"),
            );
            assert!(require_confinement(&changed).is_err(), "{field}={nonzero}");
        }
    }
    assert!(require_confinement(&CONFINED.replace("NoNewPrivs:\t1", "NoNewPrivs:\t0")).is_err());
    for malformed in ["2", "1 1", "true"] {
        assert!(
            require_confinement(
                &CONFINED.replace("NoNewPrivs:\t1", &format!("NoNewPrivs:\t{malformed}"))
            )
            .is_err()
        );
    }
}

#[test]
fn missing_duplicate_and_malformed_credential_fields_fail_closed() {
    for field in [
        "Uid",
        "Gid",
        "CapInh",
        "CapEff",
        "CapPrm",
        "CapAmb",
        "CapBnd",
        "NoNewPrivs",
    ] {
        let prefix = format!("{field}:");
        let original = CONFINED
            .lines()
            .find(|line| line.starts_with(&prefix))
            .expect("fixture field");
        let removed = CONFINED.replace(&format!("{original}\n"), "");
        assert!(require_confinement(&removed).is_err(), "missing {field}");
        for repeated in [original.to_owned(), format!("{field}:\t0")] {
            assert!(
                require_confinement(&format!("{CONFINED}{repeated}\n")).is_err(),
                "duplicate {field} cannot be resolved by choosing a value"
            );
        }
        for malformed in ["", "-1", "not-a-number", "1844674407370955161600000"] {
            let changed = CONFINED.replace(original, &format!("{field}:\t{malformed}"));
            assert!(
                require_confinement(&changed).is_err(),
                "{field}={malformed:?}"
            );
        }
    }
    for field in ["Uid", "Gid"] {
        let original = format!("{field}:\t60989\t60989\t60989\t60989");
        for malformed in [
            "60989 60989 60989",
            "60989 60989 60989 60989 60989",
            "60989 60989 60989 nope",
        ] {
            assert!(
                require_confinement(
                    &CONFINED.replace(&original, &format!("{field}:\t{malformed}"))
                )
                .is_err()
            );
        }
    }
    assert!(require_confinement("").is_err());
}

#[test]
fn proc_status_has_a_hard_sixty_four_kibibyte_bound() {
    let prefix = format!("{CONFINED}Padding:\t");
    let at_limit = format!("{prefix}{}\n", "x".repeat(65_536 - prefix.len() - 1));
    assert_eq!(at_limit.len(), 65_536);
    assert!(require_confinement(&at_limit).is_ok());
    assert!(
        require_confinement(&format!("{at_limit}\n")).is_err(),
        "valid credentials cannot excuse an oversized observation"
    );
}

#[test]
fn supplementary_groups_must_be_observed_empty_not_absent_or_retained() {
    assert!(require_confinement(&CONFINED.replace("Groups:\t\n", "")).is_err());
    assert!(require_confinement(&format!("{CONFINED}Groups:\t\n")).is_err());
    for groups in ["0", "1000", "60989", "0 60989", "1000 60989", "invalid"] {
        let changed = CONFINED.replace("Groups:\t\n", &format!("Groups:\t{groups}\n"));
        assert!(
            require_confinement(&changed).is_err(),
            "retained groups {groups:?}"
        );
    }
    assert!(require_confinement(&format!("{CONFINED}Groups:\t1000\n")).is_err());
}

#[test]
fn the_registry_and_all_five_discovery_stages_have_the_exact_independent_nineteen_actions() {
    let declared: Vec<_> = STUDIO
        .verbs
        .iter()
        .map(|verb| format!("studio.{}", verb.name))
        .collect();
    assert_eq!(declared.len(), 19);
    assert_eq!(
        declared.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        NAMES.into_iter().collect()
    );
    for stage in zero_stages() {
        assert!(require_discovery(stage, &discovery(["list_folder", "vscodium.open"])).is_ok());
        assert!(require_discovery(stage, &discovery(NAMES)).is_err());
        // Even one leaked action violates the ungranted/revoked boundary.
        for leaked in NAMES {
            assert!(
                require_discovery(stage, &discovery([leaked])).is_err(),
                "leaked {leaked}"
            );
        }
    }
    for stage in [Stage::Granted, Stage::Full] {
        let mut shuffled = declared.clone();
        shuffled.reverse();
        shuffled.push("list_folder".to_owned());
        assert!(
            require_discovery(stage, &discovery(shuffled)).is_ok(),
            "order and unrelated families do not matter"
        );
        assert!(require_discovery(stage, &discovery(Vec::<String>::new())).is_err());
        for missing in NAMES {
            assert!(
                require_discovery(
                    stage,
                    &discovery(NAMES.into_iter().filter(|name| *name != missing))
                )
                .is_err(),
                "missing {missing}"
            );
        }
    }
}

#[test]
fn duplicate_extra_and_count_preserving_wrong_studio_actions_are_refused() {
    for stage in [Stage::Granted, Stage::Full] {
        for duplicate in NAMES {
            let mut doubled = NAMES.to_vec();
            doubled.push(duplicate);
            assert!(
                require_discovery(stage, &discovery(doubled)).is_err(),
                "duplicate {duplicate}"
            );
        }
        for extra in ["studio.exec", "studio.status.extra", "studio."] {
            assert!(
                require_discovery(stage, &discovery(NAMES.into_iter().chain([extra]))).is_err()
            );
            // Merely counting19 would let an omitted control be replaced.
            assert!(
                require_discovery(stage, &discovery(NAMES.into_iter().skip(1).chain([extra])))
                    .is_err()
            );
        }
    }
    for stage in zero_stages() {
        assert!(require_discovery(stage, &discovery(["studio.unknown"])).is_err());
    }
}

fn reply(line: &str) -> ToAnAgent {
    ToAnAgent::read(line).expect("fixture is a real typed protocol reply")
}

fn non_refusals() -> Vec<ToAnAgent> {
    vec![
        discovery(Vec::<String>::new()),
        discovery(NAMES),
        ToAnAgent::did(Done::ApplicationControlled {
            application: "org.aloworld.Studio".to_owned(),
            action: "studio.status".to_owned(),
            reply: Some(r#"{"version":1,"phase":"selected"}"#.to_owned()),
            unanswered: false,
        }),
        ToAnAgent::did(Done::ApplicationControlled {
            application: "org.aloworld.Studio".to_owned(),
            action: "studio.enable_microphone".to_owned(),
            reply: None,
            unanswered: false,
        }),
        ToAnAgent::did(Done::ApplicationControlled {
            application: "org.aloworld.Studio".to_owned(),
            action: "studio.enable_microphone".to_owned(),
            reply: None,
            unanswered: true,
        }),
        reply(
            r#"{"format":1,"tells":{"proposed":{"number":7,"sentence":{"text":"enable the microphone","came_from":"the-source"},"lapses_in":300}}}"#,
        ),
        reply(
            r#"{"format":1,"tells":{"done-without-asking":{"did":{"application-controlled":{"application":"org.aloworld.Studio","action":"studio.enable_microphone","reply":null,"unanswered":false}},"said":{"text":"done","came_from":"the-source"}}}}"#,
        ),
        reply(
            r#"{"format":1,"tells":{"answered":{"text":"Access was refused.","came_from":{"text":"this machine","came_from":"the-source"},"model":"test-only"}}}"#,
        ),
    ]
}

#[test]
fn discovery_requires_a_capability_reply_and_withdrawal_requires_an_actual_refusal() {
    let refused = reply(
        r#"{"format":1,"tells":{"refused":{"text":"Access is not permitted.","came_from":"the-source"}}}"#,
    );
    assert!(require_refused(&refused).is_ok());
    for stage in all_stages() {
        assert!(
            require_discovery(stage, &refused).is_err(),
            "a refusal is not empty discovery"
        );
        for wrong in non_refusals()
            .into_iter()
            .filter(|answer| !matches!(answer, ToAnAgent::Capabilities { .. }))
        {
            assert!(
                require_discovery(stage, &wrong).is_err(),
                "unexpected reply is not discovery"
            );
        }
    }
    for wrong in non_refusals() {
        assert!(
            require_refused(&wrong).is_err(),
            "zero actions, a proposal, success, uncertainty or model prose is not an execution refusal"
        );
    }
    // Silence/malformed input never becomes a typed Refused value.
    for invalid in [
        "",
        "Access is not permitted.",
        "{}",
        r#"{"format":1,"tells":{}}"#,
    ] {
        assert!(ToAnAgent::read(invalid).is_err());
    }
}
