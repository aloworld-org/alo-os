//! Actual worker process refusals, without opening a compositor or media node.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "a failed fixture operation is a test failure"
)]

use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

/// Make parallel registry fixtures unique inside this test process.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// An exclusive temporary folder holding only a test GStreamer registry cache.
struct Registry(
    /// Created exclusively here, removed only after the owned worker exits.
    PathBuf,
);

impl Registry {
    /// Refuse to reuse any existing folder instead of sharing plugin state.
    fn fresh() -> Self {
        let path = std::env::temp_dir().join(format!(
            "studio-capture-plugin-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("create an exclusive test registry folder");
        Self(path)
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0)
            .expect("remove only this test's exclusive registry folder");
    }
}

/// The actual package binary, with no alternate executable or test-only path.
fn worker() -> Command {
    Command::new(env!("CARGO_BIN_EXE_alo-capture-provider"))
}

/// Reach the required-engine refusal if, and only if, argument validation passed.
fn worker_without_plugins(registry: &Registry) -> Command {
    let mut command = worker();
    command
        .env("ALO_CAPTURE_REQUIRE_PRIVATE_PLUGIN", "1")
        .env("GST_PLUGIN_PATH", "")
        .env("GST_PLUGIN_PATH_1_0", "")
        .env("GST_PLUGIN_SYSTEM_PATH", "")
        .env("GST_PLUGIN_SYSTEM_PATH_1_0", "")
        .env("GST_REGISTRY", registry.0.join("registry.bin"))
        .env("GST_REGISTRY_1_0", registry.0.join("registry.bin"));
    command
}

/// Argument errors must exit unsuccessfully before announcing a capture node.
#[test]
fn missing_extra_and_invalid_fixed_arguments_refuse_in_the_actual_entry() {
    let cases: &[&[&str]] = &[
        &[],
        &["1"],
        &["1", "1"],
        &["1", "1", "capture"],
        &["1", "1", "capture", "org.aloworld.Studio", "extra"],
        &["0", "1", "capture", "org.aloworld.Studio"],
        &["1", "0", "capture", "org.aloworld.Studio"],
        &["16384", "16384", "capture", "org.aloworld.Studio"],
        &["invalid", "1", "capture", "org.aloworld.Studio"],
        &["1", "1", "", "org.aloworld.Studio"],
        &["1", "1", "capture", "other/application"],
    ];
    for arguments in cases {
        let output = worker()
            .args(*arguments)
            .output()
            .expect("run the fixed capture worker");
        assert!(
            !output.status.success(),
            "worker accepted arguments {arguments:?}"
        );
        assert!(
            output.stdout.is_empty(),
            "a refusal must not announce a media node"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("alo-capture-provider:"));
    }
}

/// Each invalid identity and surplus argument must stop before inspecting media.
#[test]
fn invalid_node_application_and_extra_argument_stop_at_the_argument_gate() {
    let registry = Registry::fresh();
    let cases: &[&[&str]] = &[
        &["1", "1", "capture/foreign", "org.aloworld.Studio"],
        &["1", "1", "capture", "other/application"],
        &["1", "1", "capture", "org.aloworld.Studio", "extra"],
    ];
    for arguments in cases {
        let output = worker_without_plugins(&registry)
            .args(*arguments)
            .output()
            .expect("run an invalid call with a distinct later engine refusal");
        assert_eq!(output.status.code(), Some(1), "arguments: {arguments:?}");
        assert!(output.stdout.is_empty(), "arguments: {arguments:?}");
        assert_eq!(
            String::from_utf8_lossy(&output.stderr).trim(),
            "alo-capture-provider: invalid capture worker arguments",
            "must refuse these arguments before the later engine check: {arguments:?}"
        );
    }
}

/// Empty plugin search paths must fail specifically at the required-engine gate.
#[test]
fn a_required_private_engine_is_checked_before_any_media_graph_or_fallback() {
    let registry = Registry::fresh();
    let output = worker_without_plugins(&registry)
        .args(["1", "1", "capture", "org.aloworld.Studio"])
        .output()
        .expect("run with no discoverable media-engine plugins");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "alo-capture-provider: the required capture engine is unavailable",
        "valid arguments must reach the engine check before constructing a graph"
    );
}
