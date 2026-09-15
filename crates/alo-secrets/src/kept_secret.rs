//! What an application gets back from the keyring: its own bytes, and nothing
//! that would render them.
//!
//! A provider's key comes back as `alo_models::Secret`, which cannot be read at
//! all, because nothing in alo OS should ever look at one. An application's
//! secret is the opposite case: the whole point is to hand the bytes to the
//! application that kept them — the portal writes them into the descriptor it
//! was given. So this can be read, and only by asking for [`KeptSecret::bytes`]
//! by name; it has no `Display`, and its `Debug` says what it is without
//! saying what is in it.

/// Bytes an application kept in the keyring, handed back to that application.
pub struct KeptSecret(Vec<u8>);

impl KeptSecret {
    /// Hold these bytes as a secret.
    pub(crate) const fn holding(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// The bytes, for the one place they go: back to the application that kept
    /// them.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for KeptSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KeptSecret(..)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A kept secret does not render itself**, whoever formats it.
    #[test]
    fn a_kept_secret_does_not_render_itself() {
        let kept = KeptSecret::holding(b"hunter2-FIXTURE".to_vec());
        let shown = format!("{kept:?}");
        assert!(!shown.contains("hunter2"), "{shown}");
        assert_eq!(kept.bytes(), b"hunter2-FIXTURE");
    }
}
