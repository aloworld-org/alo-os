//! **The refusal path is tested as carefully as the happy path**, which
//! `CLAUDE.md` asks for and which matters more here than usual: everything this
//! module reads arrives from a program nobody vetted, and the interesting
//! question is always *and it was stopped*.

use super::*;

/// A reader, for a test that only needs one.
fn reading(bytes: &[u8]) -> Vec<Result<AShellSaid, NotRead>> {
    WhatTheShellSays::new().reading(bytes)
}

/// `OSC 133 ; <what> ST`, built the way a shell writes it.
fn a_mark(what: &str) -> Vec<u8> {
    format!("\x1b]133;{what}\x1b\\").into_bytes()
}

/// An `alo-typed` sequence carrying this command, with an honest length.
fn typed(command: &str) -> Vec<u8> {
    let encoded = STANDARD.encode(command.as_bytes());
    a_mark(&format!("alo-typed;{encoded};{}", command.len()))
}

/// **The four cycle marks are read**, which is the whole industry format.
#[test]
fn every_mark_in_the_cycle_is_read() {
    assert_eq!(
        reading(&a_mark("A")),
        vec![Ok(AShellSaid::APromptIsAboutToBeDrawn)]
    );
    assert_eq!(
        reading(&a_mark("B")),
        vec![Ok(AShellSaid::ThePromptHasEnded)]
    );
    assert_eq!(
        reading(&a_mark("C")),
        vec![Ok(AShellSaid::TheCommandIsRunning)]
    );
    assert_eq!(
        reading(&a_mark("D;0")),
        vec![Ok(AShellSaid::TheCommandFinished { status: Some(0) })]
    );
}

/// **A finished command with no status is still a finished command.**
///
/// Conforming shells send bare `D`, and reading it as *nothing finished* would
/// lose the end of every cycle on those shells.
#[test]
fn a_finish_without_a_status_is_still_a_finish() {
    assert_eq!(
        reading(&a_mark("D")),
        vec![Ok(AShellSaid::TheCommandFinished { status: None })]
    );
}

/// **`BEL` terminates as well as `ST`**, because shells use both.
#[test]
fn a_mark_ended_with_bel_reads_the_same_as_one_ended_with_st() {
    let with_bel = b"\x1b]133;A\x07";
    assert_eq!(
        reading(with_bel),
        vec![Ok(AShellSaid::APromptIsAboutToBeDrawn)]
    );
}

/// **The typed line comes back as the person typed it.**
#[test]
fn the_typed_line_is_read_back_whole() {
    assert_eq!(
        reading(&typed("git push")),
        vec![Ok(AShellSaid::TheyTyped("git push".to_owned()))]
    );
}

/// **A command containing `;` survives**, which is why it is encoded at all.
///
/// Unencoded, this would arrive as three parameters and be read as a command
/// ending at the first semicolon — a different command from the one typed, and
/// one that reads perfectly plausibly.
#[test]
fn a_command_with_semicolons_in_it_is_not_cut_at_them() {
    let awkward = "echo one; rm -rf two; echo three";
    assert_eq!(
        reading(&typed(awkward)),
        vec![Ok(AShellSaid::TheyTyped(awkward.to_owned()))]
    );
}

/// **A truncated payload is refused**, and this is the refusal that matters.
///
/// `vte` caps an OSC payload at 1024 bytes and dispatches what it kept rather
/// than dropping it, with no flag to say so. A long command therefore arrives
/// as a fragment that is still valid base64 and still decodes to valid text.
/// Without the declared length alo would explain half a command as though it
/// were whole.
///
/// Built by declaring a length the payload does not have, which is exactly the
/// shape truncation produces.
#[test]
fn a_payload_that_does_not_match_its_declared_length_is_refused() {
    let command = "rm -rf /important";
    let encoded = STANDARD.encode(command.as_bytes());
    let lying = a_mark(&format!("alo-typed;{encoded};{}", command.len() + 40));

    assert_eq!(
        reading(&lying),
        vec![Err(NotRead::TooLongToRead {
            declared: command.len() + 40,
            arrived: command.len(),
        })]
    );
}

/// **A long command arrives whole, and the 1024-byte cap is a `no_std` cap.**
///
/// This test was written to prove that `vte` truncates at `MAX_OSC_RAW` and
/// dispatches the fragment — the reason ADR 0123 and
/// `docs/contracts/shell-integration-marks.md` both gave for declaring a
/// length. **It failed, and the documents were wrong.** Measured 2026-10-18
/// against `vte` 0.15.0: a 2005-byte command inside a 2699-byte payload came
/// back **whole**, both in one read and in 64-byte reads.
///
/// The reason is two lines of `vte`'s own `Parser`:
///
/// ```text
/// #[cfg(not(feature = "std"))]
/// osc_raw: ArrayVec<u8, OSC_RAW_BUF_SIZE>,
/// #[cfg(feature = "std")]
/// osc_raw: Vec<u8>,
/// ```
///
/// **The cap is real and it is one cargo feature away**, which is why the
/// declared length stays: a `no_std` build truncates silently, with no flag to
/// say so, and alo would explain half a command as though it were whole. The
/// check guards the configuration rather than today's behaviour — and that is a
/// weaker claim than the one the documents made, so they now say this instead.
#[test]
fn a_long_command_arrives_whole_because_the_cap_is_a_no_std_cap() {
    let long = "echo ".to_owned() + &"x".repeat(2000);
    let sent = typed(&long);
    assert!(
        sent.len() > 1024,
        "the payload must be past MAX_OSC_RAW for this to measure anything, and it is {}",
        sent.len()
    );

    assert_eq!(
        reading(&sent),
        vec![Ok(AShellSaid::TheyTyped(long.clone()))],
        "a std build of vte grows its OSC buffer, so nothing is cut"
    );

    let mut reader = WhatTheShellSays::new();
    let mut in_pieces = Vec::new();
    for piece in sent.chunks(64) {
        in_pieces.extend(reader.reading(piece));
    }
    assert_eq!(
        in_pieces,
        vec![Ok(AShellSaid::TheyTyped(long))],
        "and splitting it across reads does not cut it either"
    );
}

/// **A payload with no declared length is refused rather than trusted.**
#[test]
fn a_typed_line_with_no_length_is_refused() {
    let encoded = STANDARD.encode(b"git push");
    assert_eq!(
        reading(&a_mark(&format!("alo-typed;{encoded}"))),
        vec![Err(NotRead::NoLengthDeclared)]
    );
}

/// **A payload that is not base64 is refused.**
#[test]
fn a_typed_line_that_is_not_base64_is_refused() {
    assert_eq!(
        reading(&a_mark("alo-typed;not base64 at all;8")),
        vec![Err(NotRead::NotBase64)]
    );
}

/// **Bytes that are not text are refused rather than lossily repaired.**
///
/// A repaired command is not the command they typed, and explaining it would be
/// explaining something nobody ran.
#[test]
fn a_typed_line_that_is_not_text_is_refused() {
    let not_text = [0xff_u8, 0xfe, 0xfd];
    let encoded = STANDARD.encode(not_text);
    assert_eq!(
        reading(&a_mark(&format!("alo-typed;{encoded};3"))),
        vec![Err(NotRead::NotText)]
    );
}

/// **A key alo does not know is dropped and changes nothing.**
///
/// That is how other terminals already extend these marks. A refusal for every
/// unknown key would make the format unextendable and would fill a person's
/// screen with complaints about somebody else's shell integration.
#[test]
fn a_mark_alo_does_not_know_is_dropped_in_silence() {
    assert_eq!(reading(&a_mark("P;shell=zsh")), vec![]);
    assert_eq!(reading(&a_mark("Z")), vec![]);
}

/// **An OSC that is not 133 is not ours and is not read.**
///
/// Setting a window title is the common one, and it arrives on every prompt in
/// a great many configurations.
#[test]
fn an_osc_that_is_not_the_semantic_prompt_is_ignored() {
    let a_title = b"\x1b]0;a window title\x07";
    assert_eq!(reading(a_title), vec![]);
}

/// **A mark split across two reads is still read**, which is why the parser's
/// state outlives a call.
///
/// A pseudoterminal hands over whatever happened to be in the buffer, so a
/// sequence straddling a read boundary is ordinary rather than exotic. A reader
/// built fresh per read would lose every one of them, and would do so only
/// under load — which is the worst way for anything to fail.
#[test]
fn a_mark_split_across_two_reads_is_still_read() {
    let whole = typed("git push");
    let (first, second) = whole.split_at(whole.len() / 2);

    let mut reader = WhatTheShellSays::new();
    assert_eq!(reader.reading(first), vec![], "nothing is complete yet");
    assert_eq!(
        reader.reading(second),
        vec![Ok(AShellSaid::TheyTyped("git push".to_owned()))]
    );
}

/// **Ordinary output carries no marks**, which is the overwhelming majority of
/// everything a terminal reads.
#[test]
fn output_with_no_marks_in_it_hands_back_nothing() {
    assert_eq!(
        reading(b"total 48\ndrwxr-xr-x  7 someone staff  224 Oct 18 10:11 .\n"),
        vec![]
    );
}

/// **Several marks in one read come back in the order they arrived.**
///
/// A prompt drawn and a command started land in the same read constantly, and a
/// reader that kept only the last would lose the cycle.
#[test]
fn several_marks_in_one_read_come_back_in_order() {
    let mut bytes = a_mark("A");
    bytes.extend_from_slice(&a_mark("B"));
    bytes.extend_from_slice(&typed("ls"));

    assert_eq!(
        reading(&bytes),
        vec![
            Ok(AShellSaid::APromptIsAboutToBeDrawn),
            Ok(AShellSaid::ThePromptHasEnded),
            Ok(AShellSaid::TheyTyped("ls".to_owned())),
        ]
    );
}
