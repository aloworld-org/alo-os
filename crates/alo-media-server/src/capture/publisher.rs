//! Give only the video publisher its pinned engine and private registry.

use crate::ATool;

/// Fixed launch settings survive the rented-tool runner's clean environment.
pub(super) fn tool(arguments: &[String]) -> ATool {
    ATool::named("/usr/bin/setpriv")
        .asking(arguments)
        .also(
            "GST_PLUGIN_PATH_1_0",
            Some("/usr/lib/alo-capture-provider/gstreamer-1.0"),
        )
        .also(
            "GST_REGISTRY",
            Some("/var/cache/alo-capture-provider/registry.bin"),
        )
        .also("ALO_CAPTURE_REQUIRE_PRIVATE_PLUGIN", Some("1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    /// A sanitized process must still require and select the installed engine.
    #[test]
    fn publisher_keeps_its_engine_identity_in_a_clean_environment() {
        let command = tool(&[]).started_where(None::<&str>);
        for (name, value) in [
            (
                "GST_PLUGIN_PATH_1_0",
                "/usr/lib/alo-capture-provider/gstreamer-1.0",
            ),
            (
                "GST_REGISTRY",
                "/var/cache/alo-capture-provider/registry.bin",
            ),
            ("ALO_CAPTURE_REQUIRE_PRIVATE_PLUGIN", "1"),
        ] {
            assert!(command.get_envs().any(|(key, given)| {
                key == OsStr::new(name) && given == Some(OsStr::new(value))
            }));
        }
    }
}
