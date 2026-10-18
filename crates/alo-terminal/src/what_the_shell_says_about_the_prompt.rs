//! **What a shell says about where it is in a command cycle**, read off its own
//! output.
//!
//! [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
//! terms 1 to 7, and `docs/contracts/shell-integration-marks.md` is the public
//! form of what this reads. That document was published on 2026-10-18 saying
//! **nothing in alo OS reads these yet**. This is the thing that does.
//!
//! # Why a shell has to say it, in one paragraph
//!
//! alo cannot know what a person typed by looking. Reading it off the screen
//! cannot tell what the **person** typed from what a **program** printed, and
//! keeping a copy of the bytes alo forwards diverges the first time somebody
//! presses `Up`, because the shell owns the line editor. The only component
//! that knows what is in the line editor is the one that owns it.
//!
//! # A second parser, and the engine is not patched
//!
//! `CLAUDE.md`: *engines are configured, never patched*. The grid's
//! `Processor` is `alacritty_terminal`'s full state machine and it drops OSC
//! 133 on the floor — `osc_dispatch` hands it to `unhandled()`. Rather than
//! reach into that, this runs **a second `vte::Parser` over the same bytes**,
//! implementing [`Perform`] for nothing but `osc_dispatch`. The grid is
//! unaffected and `alacritty_terminal` is untouched.
//!
//! Two parsers over one stream cost one extra pass over bytes that are already
//! in cache, and buy a reader nobody has to maintain inside somebody else's
//! state machine.
//!
//! # The declared length is the whole check, not belt and braces
//!
//! **Measured 2026-10-18, and it corrects what ADR 0123 and the contract both
//! said.** They said `vte` caps an OSC payload at `MAX_OSC_RAW` = 1024 bytes and
//! dispatches the fragment. A 2005-byte command inside a 2699-byte payload came
//! back **whole**, in one read and in 64-byte reads alike. The reason is two
//! lines of `vte`'s own `Parser`:
//!
//! ```text
//! #[cfg(not(feature = "std"))]
//! osc_raw: ArrayVec<u8, OSC_RAW_BUF_SIZE>,
//! #[cfg(feature = "std")]
//! osc_raw: Vec<u8>,
//! ```
//!
//! **The cap is a `no_std` cap**, and alo builds with `std`, so the buffer grows
//! and nothing is cut.
//!
//! The length stays, for a weaker and honest reason: **the cap is one cargo
//! feature away.** A `no_std` build truncates silently, with no flag saying so,
//! and alo would explain half a command as though it were whole. The check
//! guards the configuration rather than today's behaviour, and it also catches a
//! shell that mis-encodes.
//!
//! So a payload that does not decode to **exactly** the declared length is
//! refused as [`NotRead::TooLongToRead`], and the person is told the command was
//! too long to read rather than shown an explanation of its first half.
//!
//! *A single number cannot be caught being wrong. Two that must agree can.*
//!
//! # A mark is a claim, and it buys nothing
//!
//! Any program in the session can print these sequences. That is survivable
//! because the sequence carries **no authority**: it cannot run anything, hold
//! or change a grant, or approve a change. A forged mark can make alo explain a
//! command nobody typed; it cannot make alo *do* anything.
//!
//! And it never touches the gutter.
//! [ADR 0113](../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
//! stands: who is speaking is decided by where bytes came from, never by what
//! they say.

use alacritty_terminal::vte::{Parser, Perform};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// The OSC number the semantic prompt marks use, as every other terminal emits
/// it.
///
/// Not ours. kitty, iTerm2, WezTerm and VS Code already write these, which is
/// why somebody who has configured a terminal before keeps their configuration.
const THE_SEMANTIC_PROMPT: &[u8] = b"133";

/// The namespaced key for the line a person has typed but not run.
///
/// A token where a conforming reader expects `A` to `D`, so no reader can
/// mistake it for a cycle mark, and the `alo-` prefix cannot collide with a key
/// another terminal defines later. Additive to OSC 133, not a number taken from
/// somebody else's range.
const THE_TYPED_LINE: &[u8] = b"alo-typed";

/// What a shell told alo about where it is.
///
/// Each is a **claim by the shell**, not a fact alo established. Anything built
/// on one carries that standing to the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AShellSaid {
    /// `A` — a prompt is about to be drawn.
    APromptIsAboutToBeDrawn,
    /// `B` — the prompt has ended; what follows is what the person types.
    ThePromptHasEnded,
    /// `C` — the command is running; what follows is its output.
    TheCommandIsRunning,
    /// `D` — the command has finished, with the status it reported.
    ///
    /// `None` when the shell sent no status, which conforming shells do.
    TheCommandFinished {
        /// What the shell said the command exited with.
        status: Option<i64>,
    },
    /// `alo-typed` — the line as it stands in the editor, before it is run.
    ///
    /// **The only thing here that is not already an industry sequence**, and the
    /// only one that carries a person's own words.
    TheyTyped(String),
}

/// Why a mark was not read, when the shell sent one alo recognised the shape of.
///
/// A sequence alo does not recognise at all is dropped silently and is not one
/// of these — that is how other terminals already extend these marks, and a
/// refusal for every unknown key would make this unextendable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotRead {
    /// The payload did not decode to the length the shell declared.
    ///
    /// Almost always `vte`'s 1024-byte cap, which truncates and dispatches
    /// anyway. **The person is told the command was too long to read**, and is
    /// never shown an explanation of its first half.
    #[error("the typed line declared {declared} bytes and {arrived} arrived")]
    TooLongToRead {
        /// What the shell said the decoded command weighs, in bytes.
        declared: usize,
        /// What actually decoded.
        arrived: usize,
    },
    /// The second parameter was not base64.
    #[error("the typed line was not base64")]
    NotBase64,
    /// No decoded length was declared, so truncation could not be caught.
    ///
    /// Refused rather than read on trust: without the second number the first
    /// cannot be caught being wrong, which is the whole reason the format
    /// carries one.
    #[error("the typed line declared no length")]
    NoLengthDeclared,
    /// The decoded bytes were not text.
    ///
    /// A command line is a person's words and alo explains words. Bytes that
    /// are not text are refused rather than lossily repaired, because a
    /// repaired command is not the command they typed.
    #[error("the typed line was not text")]
    NotText,
}

/// A second parser over a shell's own output, reading only the prompt marks.
///
/// Holds `vte`'s state across reads, so a sequence split across two reads of
/// the pseudoterminal is still read whole. A caller that built a new one per
/// read would lose every mark that straddled a boundary.
#[derive(Default)]
pub struct WhatTheShellSays {
    /// The state machine, which is `vte`'s and not ours.
    parser: Parser,
}

impl WhatTheShellSays {
    /// A reader with nothing heard yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read these bytes and hand back every mark they completed.
    ///
    /// Empty for the overwhelming majority of reads, which carry no marks at
    /// all. **The bytes are not consumed or changed** — the caller passes the
    /// same slice to the grid, and this only looks.
    pub fn reading(&mut self, bytes: &[u8]) -> Vec<Result<AShellSaid, NotRead>> {
        let mut heard = Heard::default();
        self.parser.advance(&mut heard, bytes);
        heard.marks
    }
}

/// What one call to [`WhatTheShellSays::reading`] collected.
///
/// Separate from the parser because `vte` hands the performer out mutably for
/// the length of a call, and the parser's own state has to outlive it.
#[derive(Default)]
struct Heard {
    /// Every mark completed during this call, in the order they arrived.
    marks: Vec<Result<AShellSaid, NotRead>>,
}

impl Heard {
    /// Read an `alo-typed` payload, or say why it could not be.
    ///
    /// The length is checked **before** the bytes are read as text, because a
    /// truncated payload is the thing this exists to catch and a fragment may
    /// well still be valid text.
    fn the_typed_line(params: &[&[u8]]) -> Result<AShellSaid, NotRead> {
        let encoded = params.get(2).ok_or(NotRead::NotBase64)?;
        let declared = params
            .get(3)
            .and_then(|it| std::str::from_utf8(it).ok())
            .and_then(|it| it.parse::<usize>().ok())
            .ok_or(NotRead::NoLengthDeclared)?;

        let decoded = STANDARD.decode(encoded).map_err(|_| NotRead::NotBase64)?;
        if decoded.len() != declared {
            return Err(NotRead::TooLongToRead {
                declared,
                arrived: decoded.len(),
            });
        }
        String::from_utf8(decoded)
            .map(AShellSaid::TheyTyped)
            .map_err(|_| NotRead::NotText)
    }
}

impl Perform for Heard {
    /// Every other callback is deliberately absent.
    ///
    /// This parser exists to read OSC 133 and nothing else. Printing,
    /// execution, CSI and ESC all belong to the grid's own `Processor`, which
    /// sees the same bytes.
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.first() != Some(&THE_SEMANTIC_PROMPT) {
            return;
        }
        let Some(&which) = params.get(1) else {
            return;
        };
        let mark = match which {
            b"A" => AShellSaid::APromptIsAboutToBeDrawn,
            b"B" => AShellSaid::ThePromptHasEnded,
            b"C" => AShellSaid::TheCommandIsRunning,
            b"D" => AShellSaid::TheCommandFinished {
                status: params
                    .get(2)
                    .and_then(|it| std::str::from_utf8(it).ok())
                    .and_then(|it| it.parse::<i64>().ok()),
            },
            THE_TYPED_LINE => {
                self.marks.push(Self::the_typed_line(params));
                return;
            }
            // **Dropped, and that is the extension point.** Unknown parameters
            // after the letter are how other terminals already extend these
            // marks; a refusal here would make the format unextendable and
            // would fill a person's screen with complaints about somebody
            // else's shell integration.
            _ => return,
        };
        self.marks.push(Ok(mark));
    }
}

#[cfg(test)]
#[path = "what_the_shell_says_about_the_prompt_tests.rs"]
mod tests;
