//! Which processes there are: the numbered directories under `/proc`.
//!
//! Every process is a directory named by its pid, and everything else under
//! `/proc` — `stat`, `meminfo`, `self`, `sys` — is not a process. The list is
//! sorted, so two readings of the same machine list its processes in the same
//! order and a window does not shuffle.

use std::path::Path;

use crate::kernel::Kernel;
use crate::refusing::NotMeasured;

/// Every pid under `proc`, ascending.
///
/// # Errors
///
/// [`NotMeasured::Unreadable`] naming `proc` if it cannot be listed. There is
/// no list without it.
pub(crate) fn pids(kernel: &dyn Kernel, proc: &Path) -> Result<Vec<u32>, NotMeasured> {
    let names = kernel.list(proc).map_err(|why| NotMeasured::Unreadable {
        at: proc.to_path_buf(),
        why: why.to_string(),
    })?;
    let mut pids: Vec<u32> = names
        .iter()
        .filter_map(|name| name.parse::<u32>().ok())
        .collect();
    pids.sort_unstable();
    Ok(pids)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::pids;
    use crate::kernel::Kernel;
    use std::io;
    use std::path::{Path, PathBuf};

    /// A kernel that lists what it is told to.
    struct Listing(io::Result<Vec<&'static str>>);

    impl Kernel for Listing {
        fn list(&self, _: &Path) -> io::Result<Vec<String>> {
            match &self.0 {
                Ok(names) => Ok(names.iter().map(|name| (*name).to_owned()).collect()),
                Err(why) => Err(io::Error::new(why.kind(), why.to_string())),
            }
        }
        fn read(&self, _: &Path) -> io::Result<String> {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
        fn link(&self, _: &Path) -> io::Result<PathBuf> {
            Err(io::Error::from(io::ErrorKind::Unsupported))
        }
    }

    /// The numbers are the processes; `self`, `stat` and the rest are not.
    #[test]
    fn the_numbered_directories_are_the_processes_in_order() {
        let kernel = Listing(Ok(vec![
            "stat",
            "self",
            "42",
            "1",
            "meminfo",
            "sys",
            "7",
            "thread-self",
        ]));
        assert_eq!(pids(&kernel, Path::new("/proc")).unwrap(), [1, 7, 42]);
    }

    /// A `/proc` that cannot be listed is a refusal naming it — there is
    /// nothing to put in a list.
    #[test]
    fn a_proc_that_cannot_be_listed_is_refused_by_name() {
        let kernel = Listing(Err(io::Error::from(io::ErrorKind::PermissionDenied)));
        let refused = pids(&kernel, Path::new("/proc")).unwrap_err();
        assert!(
            matches!(&refused, crate::NotMeasured::Unreadable { at, .. } if at == Path::new("/proc")),
            "{refused}"
        );
    }
}
