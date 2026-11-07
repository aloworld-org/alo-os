//! Hold the rented renderer to its pinned upstream files and reviewed overlay.

#![cfg(target_os = "linux")]

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

#[test]
fn renderer_sources_match_the_pinned_package_and_documented_mapping_patch()
-> Result<(), Box<dyn std::error::Error>> {
    let engine = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../engines/smithay");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(engine.join("upstream-files.sha256.json"))?)?;
    assert_eq!(
        manifest["package"],
        "740cea6927892bc182d5bf70c8f79806c8bc9f68f2fb96e55a30be171b63af98"
    );
    let files = manifest["files"]
        .as_object()
        .ok_or("missing upstream file hashes")?;
    let overlay = manifest["overlay"]
        .as_object()
        .ok_or("missing reviewed overlay hashes")?;
    assert_eq!(overlay.len(), 2);
    let source = "src/backend/renderer/pixman/mod.rs";
    assert!(overlay.contains_key(source));
    assert!(overlay.contains_key("owned-pixman-mapping.patch"));
    let mut checks = String::new();
    for (name, original) in files.iter().chain(
        overlay
            .iter()
            .filter(|(name, _)| !files.contains_key(*name)),
    ) {
        assert!(
            Path::new(name)
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_)))
        );
        let digest = overlay
            .get(name)
            .unwrap_or(original)
            .as_str()
            .ok_or("invalid file hash")?;
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
        checks.push_str(&format!("{digest}  {name}\n"));
    }
    let mut child = Command::new("sha256sum")
        .args(["--strict", "--check", "-"])
        .current_dir(engine)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing checksum input")?
        .write_all(checks.as_bytes())?;
    assert!(
        child.wait()?.success(),
        "renderer source differs from its pinned package and reviewed patch"
    );
    Ok(())
}
