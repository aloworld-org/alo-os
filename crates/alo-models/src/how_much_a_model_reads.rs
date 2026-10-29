//! **How much a model on this machine is asked to read at once**, and how alo
//! OS makes sure a question is never answered from part of itself.
//!
//! [ADR 0136](../../../docs/decisions/0136-a-follow-up-is-sent-with-the-conversation-so-far.md),
//! as the owner amended it on 2026-10-28. Until then alo OS set no size at all,
//! and the pinned runtime chose one itself: on a machine with no graphics card
//! it chose **4096 tokens** and started the model with context shifting on —
//! measured in that runtime's own log on this repository's measuring machine
//! on 2026-10-11 (`-c 4096 --context-shift`). Context shifting means a
//! question longer than the window has its **beginning dropped without a
//! word**, and the model answers from what is left. A follow-up carries the
//! whole conversation, so that would have been the oldest part of what a
//! person said, gone.
//!
//! # What alo asks for
//!
//! **The smaller of the model's own maximum and [`THE_MOST_ASKED_FOR`]**, on
//! every question. The model's own maximum is what the runtime lists for the
//! weights it holds; a model that lists none is asked for
//! [`THE_MOST_ASKED_FOR`]. The figure is 8192 because the owner named it and
//! nothing documented argued for another: it is twice what the runtime chose
//! on its own, and what it costs is memory, stated in the models contract.
//!
//! # And a question too long for it is refused, twice over
//!
//! **Before it is sent**, when even at [`BYTES_A_TOKEN_TAKES_AT_MOST`] bytes a
//! token the question and room for an answer cannot fit. The runtime has no
//! request that counts tokens without answering, so this is an estimate — and
//! it is pitched to refuse only what certainly does not fit, never what might.
//!
//! **After it was answered**, when the runtime says the window was full: the
//! tokens it read and the tokens it wrote reach the size it was given. Then
//! something may have been shifted out, and an answer to part of a question is
//! not shown as an answer to the whole of it. This is the half that holds
//! whatever the estimate missed. Both are refused with the sentence a provider's
//! refusal for the same reason is (`alo_answering::WentWrong::TooLongForTheModel`).
//!
//! The runtime's context shifting itself cannot be turned off from here: the
//! pinned runtime's server configuration, as it prints it, has no setting for
//! it, and its chat request has no option for it. The two checks are what make
//! it unreachable instead.

/// The most a model on this machine is asked to read at once, in tokens.
pub const THE_MOST_ASKED_FOR: u32 = 8192;

/// Room left for the answer, in tokens, when deciding before sending.
///
/// A question that fills the window leaves the model nothing to answer in, and
/// the runtime shifts the start of the question out to make room.
pub const ROOM_FOR_AN_ANSWER: u32 = 512;

/// The most bytes one token can stand for, as this estimate assumes.
///
/// English runs near four bytes a token and code and JSON nearer three; six is
/// generous on purpose, so that a question refused before sending is one that
/// could not have fitted. Whatever slips past is caught after the answer.
pub const BYTES_A_TOKEN_TAKES_AT_MOST: usize = 6;

/// **The size a model is asked to read**: the smaller of its own maximum and
/// [`THE_MOST_ASKED_FOR`].
#[must_use]
pub fn the_window(its_own_maximum: Option<u32>) -> u32 {
    its_own_maximum.map_or(THE_MOST_ASKED_FOR, |most| most.min(THE_MOST_ASKED_FOR))
}

/// **Whether this question certainly does not fit**, before it is sent.
#[must_use]
pub fn certainly_does_not_fit(question: &str, window: u32) -> bool {
    let at_the_least = question.len().div_ceil(BYTES_A_TOKEN_TAKES_AT_MOST);
    let room = usize::try_from(window.saturating_sub(ROOM_FOR_AN_ANSWER)).unwrap_or(usize::MAX);
    at_the_least > room
}

/// **Whether the window was full**, after the answer: what the runtime read and
/// wrote reached the size it was given, so the start of the question may have
/// been shifted out.
#[must_use]
pub fn the_window_was_full(read: u64, wrote: u64, window: u32) -> bool {
    read.saturating_add(wrote) >= u64::from(window)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The window is the smaller of the two**, and a model that lists no
    /// maximum is asked for the stated figure.
    #[test]
    fn the_window_is_the_smaller_of_the_models_own_and_the_figure() {
        assert_eq!(the_window(Some(32_768)), 8192);
        assert_eq!(the_window(Some(2048)), 2048);
        assert_eq!(the_window(None), 8192);
    }

    /// **Only what certainly cannot fit is refused before sending**: a question
    /// that might fit at four bytes a token goes, and one too long even at six
    /// does not.
    #[test]
    fn only_a_question_that_certainly_cannot_fit_is_refused_before_sending() {
        let window = 2048;
        let might_fit = "a".repeat(4 * 1500);
        assert!(!certainly_does_not_fit(&might_fit, window));
        let cannot = "a".repeat(BYTES_A_TOKEN_TAKES_AT_MOST * 2048);
        assert!(certainly_does_not_fit(&cannot, window));
    }

    /// **A full window is caught after the answer**, and one with room is not.
    #[test]
    fn a_window_that_was_full_is_said_to_be() {
        assert!(the_window_was_full(2000, 48, 2048));
        assert!(the_window_was_full(2048, 0, 2048));
        assert!(!the_window_was_full(1500, 200, 2048));
    }
}
