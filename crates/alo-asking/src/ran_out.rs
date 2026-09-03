//! The identifiers a service says an account has run out with, and the ones
//! that only look like it.
//!
//! [`openai`](crate::openai) knows the convention: an address, a bearer token,
//! one message in and one message out. This file knows the one thing that
//! convention never agreed on — **how a service says the money is gone.** It is
//! a second reason to change and therefore a second file (law 4), and it is the
//! only place in this crate that reads the body of a refusal at all.
//!
//! # There is no status that means it, so there is a list
//!
//! `402 Payment Required` is in the HTTP standard and almost nobody sends it;
//! the services that do are gateways and resellers. What the large providers do
//! instead is answer an ordinary status with a machine-readable identifier
//! inside — `429` with `insufficient_quota`, `403` with a billing name — and the
//! statuses on their own mean *slow down* and *not allowed*, which are two
//! different things to tell somebody and neither of them is this.
//! `docs/quirks.md` records it.
//!
//! # Nothing anybody else wrote is kept
//!
//! An identifier is compared against the list below and dropped. Not the
//! message beside it, not the provider's name for itself, not the amount
//! outstanding — `alo_answering::WentWrong` holds no text anybody outside alo OS
//! wrote, and a file that read one out of a refusal would be the road it
//! arrived by. What crosses out of here is a `bool`.
//!
//! # And when in doubt, it has not run out
//!
//! A wrong *the account has run out* sends somebody to pay for something that
//! was never the problem, which is worse than the number they would have been
//! shown instead. So the list holds only identifiers that mean an account has
//! nothing left and mean nothing else: Google's `RESOURCE_EXHAUSTED` and
//! everybody's `rate_limit_exceeded` are deliberately absent, because both are
//! also what a service says to somebody asking too fast — and telling that
//! person to pay would be this crate inventing a bill.

use serde_json::Value;

/// The identifiers that mean an account has nothing left to spend.
///
/// Flattened as [`plainly`] writes them, so `insufficient_quota`,
/// `insufficient-quota` and `InsufficientQuota` are one entry rather than
/// three. Each is a name a service publishes rather than a sentence it composed.
const NOTHING_LEFT: [&str; 6] = [
    // OpenAI, and everything that copied its error shape.
    "insufficientquota",
    "billinghardlimitreached",
    "billingnotactive",
    // Gateways and resellers, which mostly name the status.
    "paymentrequired",
    "quotaexceeded",
    "creditsexhausted",
];

/// The longest an identifier can be and still be one.
///
/// A refusal that answers with a paragraph where a name belongs has not named
/// anything, and comparing a kilobyte against six short strings is work done for
/// a body that was never going to match.
const AS_LONG_AS_A_NAME_GETS: usize = 64;

/// Whether this refusal is a service saying the account it answers on has run
/// out.
///
/// Reads four places, because the convention has two shapes and services put
/// the name in either: `error.code`, `error.type`, and both of those at the top
/// level. A body that is not JSON, is JSON of another shape, or names something
/// that is not in the list above answers `false`, which leaves the refusal
/// exactly as it was.
pub(crate) fn said_in(body: &str) -> bool {
    let Ok(refusal) = serde_json::from_str::<Value>(body) else {
        return false;
    };
    named_in(&refusal).any(|name| NOTHING_LEFT.contains(&plainly(name).as_str()))
}

/// Every place in a refusal a service might have put the name of what went
/// wrong.
fn named_in(refusal: &Value) -> impl Iterator<Item = &str> {
    let inside = refusal.get("error");
    [
        inside.and_then(|error| error.get("code")),
        inside.and_then(|error| error.get("type")),
        refusal.get("code"),
        refusal.get("type"),
    ]
    .into_iter()
    .flatten()
    .filter_map(Value::as_str)
    .filter(|name| name.len() <= AS_LONG_AS_A_NAME_GETS)
}

/// One identifier with the punctuation and the capitals taken out of it.
///
/// Services write the same name four ways — `insufficient_quota`,
/// `INSUFFICIENT_QUOTA`, `insufficient-quota`, `InsufficientQuota` — and which
/// one a provider chose is not a thing this repository should have an opinion
/// about. Matching on the letters is the opinion it has instead.
fn plainly(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|letter| letter.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three replies that mean this in practice**, in the shapes the
    /// services that send them actually send. `402` is not here because it needs
    /// no body at all — `openai.rs` answers it on the status.
    #[test]
    fn the_refusals_that_really_mean_the_money_is_gone_are_read_as_that() {
        for body in [
            // OpenAI: a rate-limit status carrying a name that is not about rate.
            r#"{"error":{"message":"You exceeded your current quota","type":"insufficient_quota","param":null,"code":"insufficient_quota"}}"#,
            // The same, said the other way round by something that copied it.
            r#"{"error":{"type":"billing_hard_limit_reached"}}"#,
            // A gateway naming the status it would have sent.
            r#"{"code":"PAYMENT_REQUIRED","message":"add credit to continue"}"#,
        ] {
            assert!(said_in(body), "{body}");
        }
    }

    /// **Asking too fast is not running out**, and this is the test that keeps
    /// the two apart. Both arrive as `429`, and one of them is fixed by waiting
    /// a moment — a person told to pay for that would be paying for nothing.
    #[test]
    fn a_service_saying_slow_down_is_never_read_as_a_bill() {
        for body in [
            r#"{"error":{"type":"rate_limit_exceeded","message":"slow down"}}"#,
            r#"{"error":{"status":"RESOURCE_EXHAUSTED","code":429}}"#,
            r#"{"error":{"code":"requests_per_minute"}}"#,
        ] {
            assert!(!said_in(body), "{body}");
        }
    }

    /// A refused key stays a refused key. It is the other sentence this variant
    /// was invented to stop being confused with, so the confusion must not run
    /// the other way either.
    #[test]
    fn a_key_that_was_refused_is_not_read_as_a_bill_either() {
        for body in [
            r#"{"error":{"code":"invalid_api_key","message":"Incorrect API key provided: sk-live"}}"#,
            r#"{"error":{"type":"authentication_error"}}"#,
        ] {
            assert!(!said_in(body), "{body}");
        }
    }

    /// **A body this file cannot read changes nothing.** A refusal answered as
    /// HTML by something in front of the service, an empty body, or JSON of a
    /// shape nobody here has seen: each leaves the status to say what it says.
    #[test]
    fn nothing_that_cannot_be_read_is_read_as_a_bill() {
        for body in [
            "",
            "<html><title>403 Forbidden</title></html>",
            "null",
            "[]",
            r#"{"error":"insufficient_quota"}"#,
            r#"{"detail":"insufficient_quota"}"#,
        ] {
            assert!(!said_in(body), "{body:?}");
        }
    }

    /// The same name written four ways is one name, because which spelling a
    /// provider chose is not something this repository should have to track.
    #[test]
    fn one_name_spelled_four_ways_is_one_name() {
        for spelling in [
            "insufficient_quota",
            "INSUFFICIENT_QUOTA",
            "insufficient-quota",
            "InsufficientQuota",
        ] {
            let body = format!(r#"{{"error":{{"code":"{spelling}"}}}}"#);
            assert!(said_in(&body), "{spelling}");
        }
    }

    /// **A paragraph where a name belongs is not a name**, and the bound says so
    /// before the letters are counted. The body here would match on its letters
    /// alone, which is what makes it a test of the bound rather than of the list.
    #[test]
    fn a_message_standing_where_a_name_belongs_names_nothing() {
        let padded = format!("insufficient_quota{}", "_".repeat(60));
        assert!(padded.len() > AS_LONG_AS_A_NAME_GETS);
        assert!(NOTHING_LEFT.contains(&plainly(&padded).as_str()));
        assert!(!said_in(&format!(r#"{{"error":{{"code":"{padded}"}}}}"#)));
    }

    /// Nothing anybody else wrote comes out of this file: the answer is a
    /// `bool`, and the message beside the name is never looked at.
    #[test]
    fn what_leaves_this_file_is_a_yes_or_a_no() {
        let said: bool = said_in(r#"{"error":{"code":"insufficient_quota","message":"pay us"}}"#);
        assert!(said);
    }
}
