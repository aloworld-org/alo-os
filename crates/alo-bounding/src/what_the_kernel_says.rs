//! Which files describe this kernel's structures, and reading them together.
//!
//! A kernel built to say so publishes its own type information at
//! `/sys/kernel/btf/vmlinux`, and **one file for each loaded module** beside it.
//! A structure a module defines is described only in the module's file: a
//! kernel that builds btrfs in describes `btrfs_inode` in `vmlinux`, and one
//! that builds it as a module describes it in `/sys/kernel/btf/btrfs`.
//!
//! Until 2026-11-20 this crate read `vmlinux` alone. On a kernel with btrfs as
//! a module it therefore found no btrfs, and the boundary refused every file on
//! every btrfs disk. That was safe, and it was wrong: a granted file did not
//! open. Measured in `boundary.yml`'s machine, whose kernel is
//! `CONFIG_BTRFS_FS=m`, where `the_kernel_refuses::on_btrfs_a_turn_granted_a_
//! folder_opens_a_file_inside_it` was the one boundary test that failed.
//!
//! # Which modules, and what their absence means
//!
//! Only the modules whose structures the program reads are read, and today that
//! is btrfs. A module that is **not there** is a kernel that does not describe
//! it, which `crate::fields` already answers safely: every file on that
//! filesystem is refused. A module file that **is** there and cannot be read is
//! refused like an unreadable `vmlinux`, because that kernel is wrong rather
//! than silent.

use std::{fs, io, path::Path};

use crate::{btf::Types, failing::NotBounded};

/// Where a kernel that was built to say so keeps its own type information,
/// and its modules'.
const WHERE_THE_KERNEL_SAYS: &str = "/sys/kernel/btf";

/// The kernel's own file, in that directory.
const THE_KERNELS_OWN: &str = "vmlinux";

/// The modules whose structures the program reads, by the name of their file.
const THE_MODULES_READ: [&str; 1] = ["btrfs"];

impl Types {
    /// What the kernel this daemon is running on says about itself, with what
    /// the modules the program reads say.
    ///
    /// A kernel that says nothing is a kernel this boundary will not be imposed
    /// on, and the refusal names the file rather than the feature — because
    /// what is missing is `CONFIG_DEBUG_INFO_BTF`, and somebody has to be able
    /// to work that out from a machine that will not start.
    ///
    /// # Errors
    /// [`NotBounded::NoTypeInformation`] when the kernel's own file cannot be
    /// read, and [`NotBounded::TypesAreNotReadable`] when it or a module's file
    /// is not type information.
    pub fn of_this_kernel() -> Result<Self, NotBounded> {
        Self::as_described_in(Path::new(WHERE_THE_KERNEL_SAYS))
    }

    /// The same, from a directory laid out as `/sys/kernel/btf` is.
    pub(crate) fn as_described_in(directory: &Path) -> Result<Self, NotBounded> {
        let own = directory.join(THE_KERNELS_OWN);
        let file = fs::read(&own).map_err(|why| NotBounded::NoTypeInformation {
            path: own.display().to_string(),
            why,
        })?;
        let mut types = Self::read(file)?;
        for module in THE_MODULES_READ {
            match fs::read(directory.join(module)) {
                Ok(file) => types = types.with_module(file)?,
                Err(why) if why.kind() == io::ErrorKind::NotFound => {}
                Err(why) => {
                    return Err(NotBounded::NoTypeInformation {
                        path: directory.join(module).display().to_string(),
                        why,
                    });
                }
            }
        }
        Ok(types)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::testing;

    /// A directory of this test's own, laid out as `/sys/kernel/btf` is.
    fn a_directory(named: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let at =
            std::env::temp_dir().join(format!("alo-bounding-btf-{named}-{}", std::process::id()));
        drop(fs::remove_dir_all(&at));
        fs::create_dir_all(&at).expect("a directory can be made");
        for (name, bytes) in files {
            fs::write(at.join(name), bytes).expect("a file can be written");
        }
        at
    }

    /// **A kernel that builds btrfs as a module is read with it**, so btrfs's
    /// structures are found where the module describes them.
    #[test]
    fn a_kernel_is_read_with_the_module_beside_it() {
        let (kernel, module) = testing::btrfs_as_a_module();
        let at = a_directory("with-btrfs", &[("vmlinux", &kernel), ("btrfs", &module)]);
        let types = Types::as_described_in(&at).expect("both read");
        assert_eq!(
            types.member("btrfs_root", "anon_dev").map(|m| m.offset),
            Some(120)
        );
        drop(fs::remove_dir_all(&at));
    }

    /// **A kernel that describes btrfs nowhere is still read**, and describes
    /// no btrfs: a module that is not loaded is not an error, and what follows
    /// is `crate::fields`' safe answer, every btrfs file refused.
    #[test]
    fn a_module_that_is_not_there_describes_nothing() {
        let (kernel, _) = testing::btrfs_as_a_module();
        let at = a_directory("without-btrfs", &[("vmlinux", &kernel)]);
        let types = Types::as_described_in(&at).expect("the kernel reads");
        assert_eq!(types.member("btrfs_root", "anon_dev"), None);
        assert!(types.member("super_block", "s_dev").is_some());
        drop(fs::remove_dir_all(&at));
    }

    /// **A module's file that is there and is not type information is
    /// refused**, as the kernel's own would be.
    #[test]
    fn a_module_file_that_is_not_type_information_is_refused() {
        let (kernel, _) = testing::btrfs_as_a_module();
        let at = a_directory(
            "broken-btrfs",
            &[("vmlinux", &kernel), ("btrfs", b"not type information")],
        );
        assert!(matches!(
            Types::as_described_in(&at),
            Err(NotBounded::TypesAreNotReadable { .. })
        ));
        drop(fs::remove_dir_all(&at));
    }

    /// **A kernel that publishes nothing is refused by the file's name**, as it
    /// was before modules were read.
    #[test]
    fn a_kernel_with_no_type_information_is_refused_by_name() {
        let at = a_directory("nothing", &[]);
        let refused = Types::as_described_in(&at).expect_err("a kernel with nothing was read");
        assert!(
            matches!(&refused, NotBounded::NoTypeInformation { path, .. } if path.ends_with("vmlinux")),
            "{refused:?}"
        );
        drop(fs::remove_dir_all(&at));
    }
}
