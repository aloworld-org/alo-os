//! What a bridged process printed, read in the encoding it was written in.
//!
//! The Linux side of the bridge writes UTF-8: cargo, rustc, the tests. The
//! bridge itself does not. When `wsl.exe` has something of its own to say — a
//! distribution that did not answer, a service error, a warning about the
//! host's proxy — it writes it to the same pipe in **UTF-16LE**, because that
//! is what a Windows console program writes. Read as UTF-8, `Wsl/Service/`
//! comes out as `W s l / S e r v i c e /` with a NUL between every letter, and
//! nothing that looks for the machine's own words finds them.
//!
//! On 2026-09-14 that turned a machine's fault into a verdict on the work: the
//! evidence for task 11 of the local-network plan had stood up twice, WSL
//! timed out under the third run, and the classifier in `crate::gates` that
//! exists for exactly that sentence did not see it in the bytes — so the loop
//! sent its one repair worker at a test that had passed ninety seconds earlier.
//! The classifier's own test had the sentence in UTF-8, which is not how the
//! bridge ever prints it.
//!
//! So every byte a bridged process printed comes through here before anything
//! reads it. A run of UTF-16LE is told apart from UTF-8 by the one thing UTF-8
//! output never contains: a NUL byte after every character. Runs of each can
//! follow one another in one stream — the bridge's warning, then cargo's
//! output — and each is decoded as what it is.

use std::process::Output;

/// Everything a bridged process printed, standard output before standard
/// error, as text.
#[must_use]
pub(crate) fn everything(said: &Output) -> String {
    format!("{}{}", as_text(&said.stdout), as_text(&said.stderr))
}

/// Bytes a bridged process printed, as text, whichever encoding each run of
/// them was written in.
///
/// A run of UTF-16LE is one whose units have a zero high byte — every
/// character the bridge's own English or Latin-1 messages contain. A unit
/// outside that range ends the run and is read as UTF-8, lossily, so a message
/// in a script this cannot name is degraded rather than lost: the ASCII in it,
/// which is where an error code lives, still comes out as itself. A byte-order
/// mark is dropped, because it is not part of what was said.
#[must_use]
pub(crate) fn as_text(printed: &[u8]) -> String {
    let mut text = String::new();
    let mut rest = printed;
    while !rest.is_empty() {
        let wide = wide_run(rest);
        if wide == 0 {
            let until = (1..rest.len())
                .find(|&at| rest.get(at..).is_some_and(begins_wide))
                .unwrap_or(rest.len());
            let (narrow, after) = rest.split_at(until);
            text.push_str(&String::from_utf8_lossy(narrow));
            rest = after;
        } else {
            let (run, after) = rest.split_at(wide);
            let units = run
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&pair| u16::from_le_bytes(pair));
            text.extend(
                char::decode_utf16(units)
                    .map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER))
                    .filter(|&letter| letter != '\u{FEFF}'),
            );
            rest = after;
        }
    }
    text
}

/// Whether these bytes begin with one UTF-16LE unit this reads: a byte-order
/// mark, or a character whose high byte is zero.
fn begins_wide(bytes: &[u8]) -> bool {
    matches!(bytes, [0xFF, 0xFE, ..] | [1..=0xFF, 0, ..])
}

/// How many bytes at the start are one run of UTF-16LE, in whole units.
fn wide_run(bytes: &[u8]) -> usize {
    let mut taken = 0;
    while bytes.get(taken..).is_some_and(begins_wide) {
        taken += 2;
    }
    taken
}

#[cfg(test)]
mod tests {
    use super::as_text;

    /// The bytes as `wsl.exe` writes them: every character followed by a NUL.
    fn as_wsl_writes(said: &str) -> Vec<u8> {
        said.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    /// **What the bridge said on 2026-09-14, in the bytes it said it in**, is
    /// read as the sentence — so the classifier in `gates` can see the error
    /// code it was written to see.
    #[test]
    fn what_wsl_prints_in_utf16_is_read_as_the_words() {
        let said = "A connection attempt failed because the connected party did not properly \
                    respond after a period of time, or established connection failed because \
                    connected host has failed to respond. \r\nError code: \
                    Wsl/Service/0x8007274c\r\n";
        assert_eq!(as_text(&as_wsl_writes(said)), said);
        assert!(as_text(&as_wsl_writes(said)).contains("Wsl/Service/0x8007274c"));
    }

    /// **Cargo's own output is untouched**, non-ASCII included: a test name
    /// with an accent in it and a rustc arrow are UTF-8 and stay so.
    #[test]
    fn what_the_linux_side_prints_in_utf8_is_left_as_it_is() {
        let cargo = "running 1 test\ntest café::répond ... ok\n  --> src/lib.rs:4:1\n\ntest \
                     result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out\n";
        assert_eq!(as_text(cargo.as_bytes()), cargo);
    }

    /// **One stream can hold both**: the bridge's warning first, then what the
    /// Linux side printed, and each is read as what it is — so a test result
    /// behind a warning is still the one result line.
    #[test]
    fn a_warning_from_the_bridge_before_cargos_output_reads_as_both() {
        let mut printed = as_wsl_writes("wsl: Detected localhost proxy configuration.\r\n");
        printed.extend_from_slice(b"test result: ok. 1 passed; 0 failed\n");
        assert_eq!(
            as_text(&printed),
            "wsl: Detected localhost proxy configuration.\r\ntest result: ok. 1 passed; 0 \
             failed\n"
        );

        let mut printed = b"warning: unused import\n".to_vec();
        printed.extend(as_wsl_writes("Error code: Wsl/Service/0x8007274c\r\n"));
        assert_eq!(
            as_text(&printed),
            "warning: unused import\nError code: Wsl/Service/0x8007274c\r\n"
        );
    }

    /// **A byte-order mark is not part of what was said**, and a character
    /// this reader cannot name degrades to a replacement rather than taking
    /// the error code beside it with it.
    #[test]
    fn a_byte_order_mark_is_dropped_and_a_wide_character_does_not_lose_the_code() {
        let mut printed = vec![0xFF, 0xFE];
        printed.extend(as_wsl_writes("Error code: Wsl/Service/0x8007274c"));
        assert_eq!(as_text(&printed), "Error code: Wsl/Service/0x8007274c");

        let read = as_text(&as_wsl_writes("错误 Wsl/Service/0x8007274c"));
        assert!(read.contains("Wsl/Service/0x8007274c"), "{read}");
        assert_eq!(as_text(&[]), "");
    }
}
