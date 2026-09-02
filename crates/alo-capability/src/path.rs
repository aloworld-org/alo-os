//! Whether one path lies inside another, decided without touching the disk.
//!
//! This is the smallest file in the crate and the one most worth reading
//! carefully: it is where a grant to `/home/anna/Invoices` either does or does
//! not cover `/home/anna/Invoices2`. String prefixes get that wrong, so nothing
//! here compares strings — paths are compared **component by component**, which
//! is the same comparison the kernel makes.
//!
//! Two deliberate choices, both of which look like bugs until the reason is
//! read:
//!
//! **Nothing here touches the filesystem.** A grant check must give the same
//! answer whether or not the file exists, whether or not the disk is mounted,
//! and without a syscall per question. It is therefore a *lexical* decision.
//! The consequence is honest and belongs in the daemon rather than hidden here:
//! a symbolic link inside a granted folder can point outside it, so whatever
//! executes a verb resolves the real path first and asks about *that*. Deciding
//! reach on an unresolved path is a bug, and it is the one to look for first if
//! this crate is ever blamed for letting something through.
//!
//! **A path is judged by [`Path::has_root`], not `Path::is_absolute`.**
//! `is_absolute` asks the host platform, so `/home/anna` is absolute when this
//! code is compiled for Linux and not absolute when the same test runs on a
//! developer's Windows machine. A grant means the same thing wherever the check
//! was compiled, so the question asked is about the path and not about the
//! host.

use std::path::{Component, Path};

/// Whether a path is one this crate will reason about at all.
///
/// Rooted, and with no `..` in it. A relative path means something different
/// depending on where it is read from, and a path that steps upwards can leave
/// the folder it appears to be in — neither can be compared honestly, so both
/// are refused rather than normalised. Normalising would mean this crate and
/// the kernel disagreeing about what a path means, which is exactly how a
/// containment check gets defeated.
#[must_use]
pub fn is_usable(path: &Path) -> bool {
    path.has_root() && !steps_upwards(path)
}

/// Whether a path names a filesystem root — `/`, and `C:\` where somebody is
/// running the tests.
///
/// A root has no named component: everything in it is the prefix or the root
/// itself. That is the definition rather than a comparison against `"/"`,
/// because `//`, `/.` and `C:\` are the same grant to the whole machine wearing
/// different clothes.
#[must_use]
pub fn is_a_root(path: &Path) -> bool {
    path.has_root() && !path.components().any(|c| matches!(c, Component::Normal(_)))
}

/// Whether a path contains a `..` component.
#[must_use]
pub fn steps_upwards(path: &Path) -> bool {
    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// Whether `candidate` is `folder` itself or something inside it.
///
/// Component-wise, so `/home/anna/Invoices` does **not** cover
/// `/home/anna/Invoices2`. An unusable path on either side is inside nothing.
#[must_use]
pub fn is_inside(folder: &Path, candidate: &Path) -> bool {
    if !is_usable(folder) || !is_usable(candidate) {
        return false;
    }
    let mut within = candidate.components();
    folder
        .components()
        .all(|wanted| within.next() == Some(wanted))
}

/// Whether two paths name exactly the same thing.
///
/// Used by a grant to a single file — the document offered at invocation — for
/// which "inside" is not the question.
#[must_use]
pub fn is_exactly(one: &Path, other: &Path) -> bool {
    is_usable(one) && is_usable(other) && one.components().eq(other.components())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The bug this file exists to prevent. A string prefix says yes here, and
    /// a person who granted their invoices folder has just granted a second
    /// folder they have never seen.
    #[test]
    fn a_folder_does_not_cover_a_sibling_whose_name_starts_the_same() {
        let granted = Path::new("/home/anna/Invoices");
        assert!(!is_inside(granted, Path::new("/home/anna/Invoices2")));
        assert!(!is_inside(
            granted,
            Path::new("/home/anna/Invoices.old/x.pdf")
        ));
        assert!(is_inside(granted, Path::new("/home/anna/Invoices/x.pdf")));
        assert!(is_inside(granted, Path::new("/home/anna/Invoices")));
    }

    /// A path that steps upwards is refused rather than normalised: this crate
    /// and the kernel must agree about what a path means.
    #[test]
    fn a_path_that_steps_upwards_is_inside_nothing() {
        let granted = Path::new("/home/anna/Invoices");
        assert!(!is_inside(
            granted,
            Path::new("/home/anna/Invoices/../Taxes/2024.pdf")
        ));
        assert!(!is_inside(granted, Path::new("/home/anna/Invoices/../..")));
        assert!(steps_upwards(Path::new("/home/anna/../root")));
        assert!(!is_usable(Path::new("/home/anna/../root")));
    }

    /// A relative path means something different depending on where it is read
    /// from, so it is never inside anything.
    #[test]
    fn a_relative_path_is_inside_nothing() {
        assert!(!is_inside(Path::new("/home/anna"), Path::new("Invoices")));
        assert!(!is_inside(Path::new("home/anna"), Path::new("home/anna/x")));
        assert!(!is_usable(Path::new("Invoices/x.pdf")));
    }

    /// `/`, `//` and `/.` are the same grant to the whole machine, and so is
    /// `C:\` on the machine somebody is running the tests from.
    #[test]
    fn every_spelling_of_the_root_is_a_root() {
        assert!(is_a_root(Path::new("/")));
        assert!(is_a_root(Path::new("//")));
        assert!(is_a_root(Path::new("/.")));
        assert!(!is_a_root(Path::new("/home")));
        // Only where the host parses drive letters: on Linux this is a
        // perfectly ordinary relative filename with a colon in it.
        #[cfg(windows)]
        assert!(is_a_root(Path::new("C:\\")));
    }

    #[test]
    fn exactly_is_the_same_file_and_nothing_under_it() {
        let file = Path::new("/home/anna/Invoices/march.pdf");
        assert!(is_exactly(file, Path::new("/home/anna/Invoices/march.pdf")));
        assert!(is_exactly(
            file,
            Path::new("/home/anna/Invoices/./march.pdf")
        ));
        assert!(!is_exactly(file, Path::new("/home/anna/Invoices")));
        assert!(!is_exactly(
            file,
            Path::new("/home/anna/Invoices/march.pdf/x")
        ));
    }
}
