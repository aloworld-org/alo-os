//! Private launcher prerequisites, not evidence of an installed agent launch.
//!
//! The config is public image data; metadata and stream peers are test-only.
//! Actual fixed-path, no-follow/fstat wiring and one held daemon connection
//! require a later review and real guest run. No credentials are read here.

#![expect(
    clippy::expect_used,
    reason = "invalid test setup or a missing public fixture must fail the specification"
)]

use std::time::{Duration, Instant};

use alo_walking::studio_agent_launch::{
    remaining_budget, require_description, require_description_metadata, stage_token,
};
use alo_walking::studio_agent_probe::Stage;

const DESCRIPTION: &str = "format = 1\n[logins]\nperson = 1000\nagent = 60989\ngroup = 60989\n";
const TOKENS: [(Stage, &str); 5] = [
    (Stage::AskBeforeGrant, "ask-before-grant"),
    (Stage::Granted, "granted"),
    (Stage::Revoked, "revoked"),
    (Stage::Full, "full"),
    (Stage::AskAgain, "ask-again"),
];

#[test]
fn description_accepts_the_actual_image_and_supported_formats_with_exact_logins() {
    assert!(require_description(include_str!("../../../image/etc/alo/agentd.toml")).is_ok());
    for format in 1..=4 {
        assert!(
            require_description(&DESCRIPTION.replace("format = 1", &format!("format = {format}")))
                .is_ok()
        );
    }
    assert!(require_description(&format!("# public image metadata\n{DESCRIPTION}\n[agent]\nname = 'alo'\n[record]\npath = '/var/lib/alo/record.jsonl'\n")).is_ok(),
        "other legitimate daemon sections cannot invalidate the fixed login observation");
}

#[test]
fn description_refuses_missing_duplicate_malformed_or_foreign_identity() {
    for (field, valid) in [
        ("format", "1"),
        ("person", "1000"),
        ("agent", "60989"),
        ("group", "60989"),
    ] {
        let original = format!("{field} = {valid}\n");
        assert!(
            require_description(&DESCRIPTION.replace(&original, "")).is_err(),
            "missing {field}"
        );
        assert!(
            require_description(&DESCRIPTION.replace(&original, &format!("{original}{original}")))
                .is_err(),
            "duplicate {field}"
        );
        for bad in [
            "-1",
            "0",
            "true",
            "'1'",
            "1.0",
            "[]",
            "1844674407370955161600000",
        ] {
            assert!(
                require_description(&DESCRIPTION.replace(&original, &format!("{field} = {bad}\n")))
                    .is_err(),
                "{field}={bad}"
            );
        }
    }
    for bad in ["5", "99"] {
        assert!(
            require_description(&DESCRIPTION.replace("format = 1", &format!("format = {bad}")))
                .is_err()
        );
    }
    for (field, valid) in [("person", "1000"), ("agent", "60989"), ("group", "60989")] {
        for foreign in ["1000", "60989", "60990"] {
            if foreign != valid {
                assert!(
                    require_description(&DESCRIPTION.replace(
                        &format!("{field} = {valid}"),
                        &format!("{field} = {foreign}")
                    ))
                    .is_err(),
                    "foreign {field}={foreign}"
                );
            }
        }
    }
    for malformed in [
        "",
        "[logins]\n",
        "format=1\nlogins='person1000agent60989group60989'\n",
        "format=1\n[logins]\nperson=1000\nagent=60989\ngroup=60989\n[logins]\n",
    ] {
        assert!(require_description(malformed).is_err());
    }
}

#[test]
fn description_input_is_bounded_even_if_its_credentials_are_valid() {
    let prefix = format!("{DESCRIPTION}#");
    let at_limit = format!("{prefix}{}\n", "x".repeat(65_536 - prefix.len() - 1));
    assert_eq!(at_limit.len(), 65_536);
    assert!(require_description(&at_limit).is_ok());
    assert!(require_description(&format!("{at_limit}\n")).is_err());
}

#[test]
fn description_must_be_a_root_owned_regular_file_not_writable_by_group_or_others() {
    for mode in [0o444, 0o644, 0o100444, 0o100644] {
        assert!(require_description_metadata(0, mode, true, false).is_ok());
    }
    for owner in [1000, 60989, 60990] {
        assert!(require_description_metadata(owner, 0o644, true, false).is_err());
    }
    for mode in [0o664, 0o646, 0o666, 0o622, 0o100666] {
        assert!(
            require_description_metadata(0, mode, true, false).is_err(),
            "writable mode {mode:o}"
        );
    }
    for (regular, symlink) in [(false, false), (false, true), (true, true)] {
        assert!(require_description_metadata(0, 0o644, regular, symlink).is_err());
    }
}

#[test]
fn one_three_hundred_second_budget_never_restarts_at_a_stage_boundary() {
    let start = Instant::now() + Duration::from_secs(1);
    assert_eq!(remaining_budget(start, start), Ok(Duration::from_secs(300)));
    assert_eq!(
        remaining_budget(start, start + Duration::from_secs(250)),
        Ok(Duration::from_secs(50))
    );
    assert_eq!(
        remaining_budget(start, start + Duration::from_secs(299)),
        Ok(Duration::from_secs(1))
    );
    assert_eq!(
        remaining_budget(start, start + Duration::from_millis(299_999)),
        Ok(Duration::from_millis(1))
    );
    assert!(remaining_budget(start, start + Duration::from_secs(300)).is_err());
    assert!(remaining_budget(start, start + Duration::from_secs(301)).is_err());
    assert!(
        remaining_budget(start, start - Duration::from_secs(1)).is_err(),
        "a backward observation cannot extend the budget"
    );
}

#[test]
fn native_boundary_tokens_are_fixed_and_distinct() {
    for (stage, expected) in TOKENS {
        assert_eq!(stage_token(stage), expected);
    }
}

#[cfg(unix)]
mod native_io {
    use super::*;
    use alo_walking::studio_agent_launch::exchange_stage;
    use std::{
        io::{self, Write},
        net::Shutdown,
        os::unix::net::UnixStream,
    };

    #[derive(Default)]
    struct Output {
        bytes: Vec<u8>,
        flushes: usize,
        fail_write: bool,
        fail_flush: bool,
    }

    impl Write for Output {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail_write {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "test write refusal",
                ));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            if self.fail_flush {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "test flush refusal",
                ))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn stage_writes_and_flushes_its_token_then_accepts_the_same_complete_token() {
        for (stage, literal) in TOKENS {
            let (input, mut person) = UnixStream::pair().expect("test stream pair");
            person
                .write_all(format!("{literal}\n").as_bytes())
                .expect("person ack");
            // Keep the writer open: success requires a newline, not EOF.
            let mut output = Output::default();
            assert!(
                exchange_stage(
                    stage,
                    &input,
                    &mut output,
                    Instant::now() + Duration::from_secs(2)
                )
                .is_ok()
            );
            assert_eq!(output.bytes, format!("{literal}\n").into_bytes());
            assert_eq!(
                output.flushes, 1,
                "controller must see the token before the probe waits"
            );
        }
    }

    #[test]
    fn wrong_stage_whitespace_extra_values_and_unfinished_or_empty_input_refuse() {
        for (stage, literal) in TOKENS {
            for invalid in [
                String::new(),
                literal.to_owned(),
                format!(" {literal}\n"),
                format!("{literal} \n"),
                format!("{literal} extra\n"),
                format!("{literal}\r\n"),
                "wrong-stage\n".to_owned(),
                "x".repeat(4097),
            ] {
                let (input, mut person) = UnixStream::pair().expect("test stream pair");
                person
                    .write_all(invalid.as_bytes())
                    .expect("bad person ack");
                person.shutdown(Shutdown::Write).expect("EOF");
                assert!(
                    exchange_stage(
                        stage,
                        &input,
                        &mut Output::default(),
                        Instant::now() + Duration::from_secs(2)
                    )
                    .is_err(),
                    "{literal}: unfinished or non-exact acknowledgement {invalid:?}"
                );
            }
            for (_, other) in TOKENS {
                if other == literal {
                    continue;
                }
                let (input, mut person) = UnixStream::pair().expect("test stream pair");
                person
                    .write_all(format!("{other}\n").as_bytes())
                    .expect("out of order ack");
                assert!(
                    exchange_stage(
                        stage,
                        &input,
                        &mut Output::default(),
                        Instant::now() + Duration::from_secs(2)
                    )
                    .is_err()
                );
            }
        }
    }

    #[test]
    fn expired_budget_refuses_before_stage_progress_even_with_an_ack_waiting() {
        let (input, mut person) = UnixStream::pair().expect("test stream pair");
        person.write_all(b"granted\n").expect("ack");
        let mut output = Output::default();
        assert!(exchange_stage(Stage::Granted, &input, &mut output, Instant::now()).is_err());
        assert!(
            output.bytes.is_empty(),
            "no stage token may announce progress after the shared deadline"
        );
        assert_eq!(output.flushes, 0);
    }

    #[test]
    fn partial_line_with_writer_still_open_cannot_block_past_the_deadline() {
        let (input, mut person) = UnixStream::pair().expect("test stream pair");
        person
            .write_all(b"grant")
            .expect("unfinished acknowledgement");
        let began = Instant::now();
        assert!(
            exchange_stage(
                Stage::Granted,
                &input,
                &mut Output::default(),
                began + Duration::from_millis(100)
            )
            .is_err()
        );
        assert!(
            began.elapsed() < Duration::from_secs(5),
            "read_line after initial readability must not wait indefinitely for newline"
        );
    }

    #[test]
    fn output_write_or_flush_failure_never_becomes_a_completed_boundary() {
        for (fail_write, fail_flush) in [(true, false), (false, true)] {
            let (input, mut person) = UnixStream::pair().expect("test stream pair");
            person
                .write_all(b"granted\n")
                .expect("ack already available");
            let mut output = Output {
                fail_write,
                fail_flush,
                ..Output::default()
            };
            assert!(
                exchange_stage(
                    Stage::Granted,
                    &input,
                    &mut output,
                    Instant::now() + Duration::from_secs(2)
                )
                .is_err()
            );
        }
    }

    #[test]
    fn unreadable_input_descriptor_is_an_error_not_an_acknowledgement() {
        // A write-only descriptor is valid to poll, but cannot supply input.
        // No file contents are changed; /dev/null receives no writes.
        let input = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/null")
            .expect("local write-only test descriptor");
        assert!(
            exchange_stage(
                Stage::Granted,
                &input,
                &mut Output::default(),
                Instant::now() + Duration::from_secs(2)
            )
            .is_err()
        );
    }
}
