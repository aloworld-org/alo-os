//! Refuse a production capture if the required private engine was substituted.

use gst::prelude::*;
use std::{io, path::Path};

/// The immutable private plugin installed by the OS image.
const FILE: &str = "/usr/lib/alo-capture-provider/gstreamer-1.0/libgstpipewire.so";
/// The upstream buffer-ownership revision and separately pinned race fix.
const VERSION: &str = "1.4.90+ee2c6eb4.2770143f";

/// Check the engine selected by an installed portal, without a system fallback.
pub(super) fn verify() -> io::Result<()> {
    // Standalone development fixtures may explicitly use the host engine.
    // The installed portal unit always requires the private pinned engine.
    if std::env::var_os("ALO_CAPTURE_REQUIRE_PRIVATE_PLUGIN").is_none() {
        return Ok(());
    }
    let plugin = gst::ElementFactory::find("pipewiresink")
        .and_then(|factory| factory.plugin())
        .ok_or_else(|| io::Error::other("the required capture engine is unavailable"))?;
    validate(plugin.filename().as_deref(), plugin.version().as_str())
}

/// Missing metadata, other search paths, and other versions are refusals.
fn validate(filename: Option<&Path>, version: &str) -> io::Result<()> {
    if filename != Some(Path::new(FILE)) || version != VERSION {
        return Err(io::Error::other(
            "the required private capture engine is missing or has an unexpected version",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_distribution_plugin_is_not_a_private_engine_fallback() {
        assert!(
            validate(
                Some(Path::new("/usr/lib64/gstreamer-1.0/libgstpipewire.so")),
                VERSION
            )
            .is_err()
        );
        assert!(validate(None, VERSION).is_err());
    }

    #[test]
    fn an_unexpected_private_engine_version_is_refused() {
        assert!(validate(Some(Path::new(FILE)), "1.4.11").is_err());
        assert!(validate(Some(Path::new(FILE)), VERSION).is_ok());
    }
}
