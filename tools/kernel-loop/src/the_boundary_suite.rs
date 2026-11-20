//! The tests that attach a BPF programme to this machine's kernel, and the one
//! list of them.
//!
//! Those tests are what shows the kernel *imposes* the boundary rather than the
//! daemon following a rule (ADR 0013), and they cannot run on a pull request's
//! hosted runner: its kernel was started without `bpf` among its security
//! modules, so an LSM programme has nothing to attach to. They run after each
//! merge instead, in `.github/workflows/boundary.yml`, inside a virtual machine
//! started with `bpf` added, and the merge gate runs everything else.
//!
//! **Two runs that split one suite between them have one way to fail
//! silently**: a test in neither. Until 2026-11-17 that was 539 of
//! `alo-agentd`'s tests, because the gate left the package out whole and no
//! workflow ran the boundary profile at all. So the split is written once, here,
//! and the tests below hold the gate's filter, `.config/nextest.toml` and the
//! source of `alo-agentd` to it.

/// Every test the boundary suite runs, as a nextest filter.
///
/// The gate runs exactly the rest: its filter in [`crate::gates::EVERY_GATE`] is
/// `not (` this `)` and one test that walks the machine's own root. Printed by
/// `alo-kernel-loop boundary-filter`, which is what `boundary.yml` runs, so the
/// workflow does not carry a copy.
pub const THE_BOUNDARY: &str = "package(alo-bounding) + package(alo-boundaryd) \
     + binary(the_kernel_refuses_what_a_run_was_not_given) \
     + binary_id(=alo-agentd::a_question_is_bounded_by_the_kernel) \
     + binary_id(=alo-agentd::a_turn_is_bounded_by_the_kernel) \
     + binary_id(=alo-agentd::a_turn_is_refused_when_the_boundary_is_gone) \
     + binary_id(=alo-agentd::the_agent_asks_a_real_provider) \
     + binary_id(=alo-agentd::the_agent_really_does_the_work) \
     + (binary_id(=alo-agentd) and test(/^a_paired_machine_on_two_networks_with_one_address::/)) \
     + (binary_id(=alo-agentd) and test(/^a_paired_machine_over_link_local::/))";

/// The one test the gate leaves out that is not a boundary test: it walks
/// whatever `/` it lands on, and `gates.rs` says why at length.
#[cfg(test)]
const THE_WALK_OF_THE_ROOT: &str =
    "test(=naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so)";

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "an unexpected reading is a test failure"
)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use super::{THE_BOUNDARY, THE_WALK_OF_THE_ROOT};
    use crate::gates::EVERY_GATE;

    /// What a line of code says when it attaches: `Imposed::once` is the only way
    /// `alo-bounding` loads a programme, and `Waited::on_this_kernel` is the lock
    /// every test that does so takes first.
    const ATTACHES: [&str; 2] = ["Imposed::once(", "Waited::on_this_kernel("];

    fn the_repository() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn every_rust_file(under: &Path, into: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(under) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                every_rust_file(&path, into);
            } else if path.extension().is_some_and(|it| it == "rs") {
                into.push(path);
            }
        }
    }

    /// Whether a file has a line of code, not of commentary, that attaches.
    fn attaches(text: &str) -> bool {
        text.lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .any(|line| ATTACHES.iter().any(|it| line.contains(it)))
    }

    /// How the boundary filter names a file of `alo-agentd`: an integration test
    /// by its binary, a module of the library by the test path its tests share.
    fn named_as(crate_root: &Path, file: &Path) -> Option<String> {
        let relative = file.strip_prefix(crate_root).ok()?;
        let stem = relative.file_stem()?.to_str()?;
        match relative.components().next()?.as_os_str().to_str()? {
            "tests" => Some(format!("binary_id(=alo-agentd::{stem})")),
            "src" => Some(format!("(binary_id(=alo-agentd) and test(/^{stem}::/))")),
            _ => None,
        }
    }

    /// Every file of `alo-agentd` that attaches, as the filter names it.
    fn what_attaches_in_the_agent_service() -> BTreeSet<String> {
        let root = the_repository().join("crates/alo-agentd");
        let mut files = Vec::new();
        every_rust_file(&root.join("src"), &mut files);
        every_rust_file(&root.join("tests"), &mut files);
        assert!(
            files.len() > 50,
            "only {} Rust files were found under {}, so this read nothing",
            files.len(),
            root.display()
        );
        files
            .iter()
            .filter(|file| std::fs::read_to_string(file).is_ok_and(|text| attaches(&text)))
            .filter_map(|file| named_as(&root, file))
            .collect()
    }

    /// The `alo-agentd` terms of the boundary filter, one per `+`.
    fn what_the_boundary_names_in_the_agent_service() -> BTreeSet<String> {
        THE_BOUNDARY
            .split(" + ")
            .map(str::trim)
            .filter(|it| it.contains("alo-agentd"))
            .map(str::to_owned)
            .collect()
    }

    /// **Every test in `alo-agentd` that attaches is in the boundary suite, and
    /// nothing is there that does not.** The first half is the one that matters:
    /// a new kernel test nobody listed would run in neither place. The second
    /// keeps a test that stopped attaching from staying off the merge path.
    #[test]
    fn the_boundary_names_exactly_what_attaches_in_the_agent_service() {
        let attaching = what_attaches_in_the_agent_service();
        let named = what_the_boundary_names_in_the_agent_service();
        assert!(
            attaching.len() >= 7,
            "only {attaching:?} were found attaching, so the reading is wrong"
        );
        let unlisted: Vec<_> = attaching.difference(&named).collect();
        let stale: Vec<_> = named.difference(&attaching).collect();
        assert!(
            unlisted.is_empty(),
            "these attach a BPF programme and are in neither the gate nor the boundary \
             suite; add them to THE_BOUNDARY and to the gate's filter: {unlisted:?}"
        );
        assert!(
            stale.is_empty(),
            "these are kept off the merge path for attaching and no longer do: {stale:?}"
        );
    }

    /// **The gate runs exactly what the boundary suite does not.**
    #[test]
    fn the_gate_is_the_complement_of_the_boundary() {
        let gate = EVERY_GATE
            .iter()
            .find(|it| it.named == "the workspace's tests")
            .expect("the gate that runs the workspace's tests is defined");
        let filter = gate
            .args
            .iter()
            .skip_while(|it| **it != "-E")
            .nth(1)
            .expect("the tests gate passes a filter");
        assert_eq!(
            *filter,
            format!("not ({THE_BOUNDARY}) and not {THE_WALK_OF_THE_ROOT}")
        );
    }

    /// **The test group that serialises attaching names the same tests**, in
    /// every profile, so the boundary suite runs one attach at a time and the
    /// rest of `alo-agentd` is not queued behind a group it does not need.
    #[test]
    fn the_test_runner_groups_the_same_tests() {
        let config = std::fs::read_to_string(the_repository().join(".config/nextest.toml"))
            .expect(".config/nextest.toml can be read");
        let expected = format!("filter = '{THE_BOUNDARY}'");
        let grouped = config
            .lines()
            .filter(|line| line.trim() == expected)
            .count();
        assert_eq!(
            grouped, 3,
            "the default, gates and boundary profiles each group the boundary suite"
        );
        assert!(
            !config.contains("package(alo-agentd)"),
            "the whole of alo-agentd is still grouped as though it attached"
        );
    }

    /// **The reading refuses what it should**: a commented-out attach is not an
    /// attach, and code that attaches is.
    #[test]
    fn a_comment_is_not_an_attach() {
        assert!(!attaches(
            "//! takes `alo_bounding::Waited::on_this_kernel()`"
        ));
        assert!(!attaches("    // Imposed::once(&pinned)"));
        assert!(attaches(
            "        let loaded = Imposed::once(&pinned).expect("
        ));
    }
}
