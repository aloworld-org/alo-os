//! How a path becomes the thing this machine would really open.
//!
//! One question, one implementation, and no way to add a second. [`Resolving`]
//! is public because it appears in [`crate::Touching::of`]'s signature and
//! because naming the question is worth more than hiding it — but [`Real`] has
//! no public constructor, so the only type that can answer it is
//! [`OnThisMachine`]. Reach is decided on the answer; a second answerer would
//! be a second opinion about what a grant covers.
//!
//! It exists as a trait rather than as a bare function for one reason that
//! earns it: **the refusal has to be testable everywhere the tests run.** The
//! escape this whole crate exists to stop is a link inside a granted folder
//! pointing outside it, and making one takes a privilege that a developer's
//! machine may not have. So [`crate::Touching`]'s decision is tested against a
//! filesystem written down in the test, on every platform, and this file's one
//! implementation is tested against a real one — with the link followed for
//! real where the platform allows it.

use std::io::ErrorKind;
use std::path::Path;

use crate::real::{Real, RealError};

/// How a path is made real.
///
/// Sealed: implementing it requires making a [`Real`], and nothing outside this
/// crate can.
///
/// ```compile_fail
/// use std::path::{Path, PathBuf};
/// use alo_files::{Real, RealError, Resolving};
///
/// struct Wherever;
/// impl Resolving for Wherever {
///     fn real(&self, path: &Path) -> Result<Real, RealError> {
///         Ok(Real::new(PathBuf::from("/home/anna/Invoices/march.pdf")))
///     }
/// }
/// ```
///
/// The twin that passes, so the pair cannot rot into a test that a typo fails
/// to compile:
///
/// ```
/// use alo_files::{OnThisMachine, Resolving};
///
/// let nowhere = OnThisMachine.real(std::path::Path::new("/no/such/folder/here"));
/// assert!(nowhere.is_err());
/// ```
pub trait Resolving {
    /// The real path this one names, with every link followed.
    ///
    /// # Errors
    /// [`RealError`] — nothing there, or a path this machine would not follow.
    fn real(&self, path: &Path) -> Result<Real, RealError>;
}

/// The filesystem of the machine this is running on.
///
/// The only thing in `alo-files` that touches a disk, which is why it is a type
/// of its own with nothing else in it: everything that decides is somewhere a
/// test can reach without a filesystem.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnThisMachine;

impl Resolving for OnThisMachine {
    fn real(&self, path: &Path) -> Result<Real, RealError> {
        match std::fs::canonicalize(path) {
            Ok(real) => Ok(Real::new(real)),
            Err(why) if why.kind() == ErrorKind::NotFound => Err(RealError::Nothing {
                path: path.display().to_string(),
            }),
            Err(why) => Err(RealError::Unreadable {
                path: path.display().to_string(),
                why: why.to_string(),
            }),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_folder_of_our_own;
    use std::fs;

    /// The ordinary case, against a real filesystem: a file that is there
    /// resolves to what this machine would open.
    #[test]
    fn a_path_that_is_there_resolves_to_what_this_machine_would_open() {
        let folder = a_folder_of_our_own("there");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();

        let real = OnThisMachine.real(&file).unwrap();
        assert_eq!(real.as_path(), fs::canonicalize(&file).unwrap());
        assert!(real.describe().contains("march.pdf"), "{}", real.describe());

        let _ = fs::remove_dir_all(&folder);
    }

    /// Nothing there is a refusal and not an empty answer, because a path that
    /// does not exist cannot be compared against a grant at all.
    #[test]
    fn a_path_that_is_not_there_is_refused_and_says_so() {
        let folder = a_folder_of_our_own("missing");
        let err = OnThisMachine.real(&folder.join("april.pdf")).unwrap_err();
        assert!(matches!(err, RealError::Nothing { .. }), "{err}");
        assert!(err.to_string().contains("there is nothing at"), "{err}");
        let _ = fs::remove_dir_all(&folder);
    }

    /// **The reason this file exists**, against a real filesystem: a link
    /// resolves to where it points, not to where it sits.
    ///
    /// Unix only, because creating a symbolic link on Windows needs a
    /// privilege a developer's account may not have, and a test that quietly
    /// skips itself is a test that stops being run. The decision this feeds —
    /// that such a path is refused — is tested on every platform in
    /// [`crate::touching`].
    #[cfg(unix)]
    #[test]
    fn a_link_resolves_to_where_it_points_and_not_to_where_it_sits() {
        let folder = a_folder_of_our_own("link");
        let elsewhere = folder.join("Elsewhere");
        let invoices = folder.join("Invoices");
        fs::create_dir_all(&elsewhere).unwrap();
        fs::create_dir_all(&invoices).unwrap();
        let secret = elsewhere.join("secret.txt");
        fs::write(&secret, b"not an invoice").unwrap();
        std::os::unix::fs::symlink(&secret, invoices.join("march.pdf")).unwrap();

        let real = OnThisMachine.real(&invoices.join("march.pdf")).unwrap();
        assert_eq!(real.as_path(), fs::canonicalize(&secret).unwrap());
        assert!(
            !real
                .as_path()
                .starts_with(fs::canonicalize(&invoices).unwrap())
        );

        let _ = fs::remove_dir_all(&folder);
    }
}
