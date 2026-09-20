//! The password a person hands the broker beside the proxy it signs in to.
//!
//! `proxy_file.rs` is the shape of the two files the broker touches for the
//! proxy **setting**; this is the third file, and it is the one that carries a
//! credential. [ADR 0060](../../../docs/decisions/0060-a-persons-own-proxy-password-is-set-with-the-proxy-in-one-act.md)
//! is the decision, and `alo_proxy::provisioning` is where a password's bytes
//! are made and read — this file says only **where** they are handed over and
//! **how** they and the proxy become one thing a person approves.
//!
//! | file | written by | holds |
//! |---|---|---|
//! | [`THE_WANTED_PASSWORD`] | the person, in the broker's own directory for it, `0600` | `alo_proxy::provisioning::handed_over` bytes |
//!
//! # One act, so one identity
//!
//! ADR 0060 §1: the proxy and the password it asks for are set together, by the
//! one verb `network.set-proxy`, under one approval — because two verbs cannot
//! promise that a machine is never left with a proxy set that it cannot sign in
//! to, and an approval is never a session (ADR 0001 §5).
//!
//! So what crosses the door is the digest of **both**, and [`handed_over_together`]
//! is the one place that says how. For a proxy handed over with no password it
//! is the proxy's bytes exactly, which is what `docs/contracts/machine-proxy-file.md`
//! already publishes and what this change may not break.
//!
//! # The password is not in the proxy
//!
//! It is a second file on purpose. `alo_proxy::TheProxy` has no field a
//! password could arrive in (ADR 0022) and a test in `alo-proxy` holds it
//! there; putting one into the bytes that file is written as would make that
//! property a matter of care rather than of shape.

/// Where a person puts the password their proxy asks for, for the broker to
/// write.
///
/// Beside [`crate::proxy_file::THE_WANTED_PROXY`], in the same runtime folder
/// the broker makes `0770` in the person's group at start-up. The file itself
/// is the person's, `0600`, and the broker removes it whichever way the act
/// goes — including every refusal, because bytes left in `/run` are a
/// credential waiting for somebody to read them.
pub const THE_WANTED_PASSWORD: &str = "/run/alo-broker/wanted/proxy-password";

/// The bytes a person's proxy and its password are approved as, together.
///
/// The proxy's bytes, then the password's. A proxy handed over with **no**
/// password — `password` empty — digests to the proxy's own bytes and nothing
/// else, so the sentence the contract already publishes, *the SHA-256 of
/// exactly those bytes*, stays literally true for every machine that does not
/// use this.
///
/// The concatenation needs no framing between the two: the proxy's half is the
/// JSON `alo_proxy` writes and the password's half begins with
/// `alo_proxy::NONCE_BYTES` bytes of the kernel's randomness, and both halves
/// are written by the same person in the same act. What the digest is for is
/// binding an approval to what was handed over, not telling two strangers
/// apart.
#[must_use]
pub fn handed_over_together(proxy: &[u8], password: &[u8]) -> Vec<u8> {
    let mut both = Vec::with_capacity(proxy.len().saturating_add(password.len()));
    both.extend_from_slice(proxy);
    both.extend_from_slice(password);
    both
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::proxy_file::{THE_WANTED_PROXY, wanted};
    use alo_proxy::{
        Exceptions, Password, ProxyAddress, SpokenTo, TheProxy, WhereThePasswordIs, handed_over,
    };

    /// A person's own machine's proxy, signing in under the one name ADR 0060
    /// §2 reserves for it.
    fn signing_in_on_this_machine() -> TheProxy {
        TheProxy::one(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128)
                .unwrap()
                .signing_in("anna", WhereThePasswordIs::on_this_machine())
                .unwrap(),
        )
        .excepting(Exceptions::of(["intranet.example.com"]).unwrap())
    }

    /// **A proxy with no password digests to exactly the proxy's own bytes**,
    /// which is the sentence the contract already publishes and which this
    /// change does not break.
    #[test]
    fn a_proxy_with_no_password_is_still_exactly_its_own_bytes() {
        let bytes = wanted(&TheProxy::None).unwrap();
        assert_eq!(handed_over_together(&bytes, &[]), bytes);
    }

    /// **A password handed over changes what is approved**, so an approval for
    /// one password is not an approval for another — and neither is an approval
    /// for the proxy alone.
    #[test]
    fn a_password_handed_over_is_part_of_what_was_approved() {
        let proxy = wanted(&signing_in_on_this_machine()).unwrap();
        let one = handed_over(&Password::typed("hunter2").unwrap()).unwrap();
        let another = handed_over(&Password::typed("something else").unwrap()).unwrap();

        assert_ne!(handed_over_together(&proxy, &one), proxy);
        assert_ne!(
            handed_over_together(&proxy, &one),
            handed_over_together(&proxy, &another)
        );
        assert_eq!(
            handed_over_together(&proxy, &one),
            handed_over_together(&proxy, &one),
            "the same two files are the same approval"
        );
    }

    /// **A settings panel can say *a password is set*, and can never read it.**
    ///
    /// ADR 0060 §1 and §6: the broker writes the machine's proxy file only
    /// after the credential, so the file naming where the password is kept is
    /// the machine's statement that one is there. What a panel reads is that
    /// file — `0644`, everybody's — and the whole of what it holds is the name;
    /// there is nothing anywhere that answers with the password itself.
    #[test]
    fn a_panel_reads_that_a_password_is_set_and_never_reads_the_password() {
        use crate::proxy_file::{kept_on_this_machine, machines};
        use alo_proxy::{Kept, Scheme};

        let written = machines(&Kept::by_this_person(signing_in_on_this_machine())).unwrap();
        let kept = kept_on_this_machine(&written).unwrap();
        let signing_in = kept.proxy().for_(Scheme::Http).unwrap();

        assert_eq!(signing_in.name(), Some("anna"));
        assert!(
            signing_in
                .password()
                .is_some_and(alo_proxy::WhereThePasswordIs::is_this_machines_own),
            "a panel cannot tell that a password is set"
        );

        // And the file everybody may read holds the name and no credential.
        let read = String::from_utf8(written).unwrap();
        assert!(read.contains("the-proxy-on-this-machine"), "{read}");
        assert!(!read.contains("hunter2"), "{read}");

        // A proxy that signs in nowhere says so, which is the other half of
        // what the panel draws: an offer to set one.
        let none = machines(&Kept::by_this_person(TheProxy::None)).unwrap();
        assert!(
            kept_on_this_machine(&none)
                .unwrap()
                .proxy()
                .for_(Scheme::Http)
                .is_none()
        );
    }

    /// **The password is handed over beside the proxy, in the broker's own
    /// folder** — and the proxy's own bytes never carry it.
    #[test]
    fn the_password_is_handed_over_beside_the_proxy_and_never_inside_it() {
        assert!(THE_WANTED_PASSWORD.starts_with("/run/alo-broker/wanted/"));
        assert_ne!(THE_WANTED_PASSWORD, THE_WANTED_PROXY);
        assert_eq!(
            std::path::Path::new(THE_WANTED_PASSWORD).parent(),
            std::path::Path::new(THE_WANTED_PROXY).parent()
        );

        let bytes = wanted(&signing_in_on_this_machine()).unwrap();
        let written = String::from_utf8(bytes).unwrap();
        assert!(written.contains("the-proxy-on-this-machine"), "{written}");
        assert!(!written.contains("hunter2"), "{written}");
    }
}
