//! Which deployment this machine booted, out of the words the kernel was
//! started with.
//!
//! The base keeps every build it has on the disk under `/ostree/deploy`, and
//! more than one of them at a time is the whole point of it — so *which folder
//! is this machine running out of* is a real question with a real answer, and
//! the answer is in the one place nothing can disagree with: **the command line
//! the kernel was actually started with**, which the base's own bootloader
//! entry wrote and which every booted Linux machine publishes, world-readable,
//! at [`THE_KERNELS_WORDS`].
//!
//! # Why this and not the base's command
//!
//! `bootc status` answers the same question and **refuses an unprivileged
//! caller** — measured on a real machine on 2026-09-20,
//! `docs/autonomy/updates/the-base-answers-only-root.md`: *This command must be
//! executed as the root user*, on every spelling of the question, taken on the
//! write path before it reads anything. The one act on an alo OS machine that
//! has to ask which build is running and must **not** be root is the check at a
//! start (`alo-looking-once`, ADR 0018's argument and ADR 0001 §2's), so that
//! act reads what the base has already written down instead of asking it
//! anything.
//!
//! Nothing here is privileged, nothing here is new on the machine, and **no
//! program is run**: it is one file read and one path resolved. ADR 0011's *the
//! base is spoken to through its own command* is untouched for every act that
//! **changes** the machine — `upgrade`, `switch`, `rollback` — which is where
//! being root is correct and is the caller's business rather than this
//! component's.
//!
//! # What the word looks like
//!
//! `ostree=/ostree/boot.1/default/<boot checksum>/0`, which is a symlink the
//! base maintains onto the deployment's own folder — so the path is resolved
//! rather than taken apart. The checksum in it is the **boot** checksum and not
//! the deployment's, which is exactly why this file resolves the link instead
//! of building a path out of its pieces.

use std::path::{Path, PathBuf};

/// Where the words the kernel was started with are, under the machine's own
/// root — `/proc/cmdline` on a machine that is running.
///
/// Written without its leading separator so that it joins onto a root rather
/// than replacing one, which is what lets a test stand a whole machine up in a
/// folder of its own and read it by exactly this code.
pub const THE_KERNELS_WORDS: &str = "proc/cmdline";

/// The word in them that names the deployment this machine booted.
const THE_WORD: &str = "ostree=";

/// Why the deployment this machine booted could not be named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotBooted {
    /// The words the kernel was started with could not be read.
    NotRead {
        /// Where they are.
        path: String,
        /// What the machine said.
        why: String,
    },
    /// They name no deployment: this machine did not boot from one.
    ///
    /// An ordinary Linux machine that is not an alo OS machine reads exactly
    /// like this, and so does a developer's checkout, which is why it is a
    /// refusal of its own rather than the one above.
    NamesNoDeployment {
        /// Where the words are.
        path: String,
    },
    /// They name a deployment that is not on the disk.
    NotThere {
        /// The folder they named.
        path: String,
        /// What the machine said.
        why: String,
    },
}

/// The folder the deployment this machine booted is in.
///
/// `under` is the machine's own root on a real machine and a folder of its own
/// in a test, so that every path below it is read the same way in both.
pub(crate) fn booted(under: &Path) -> Result<PathBuf, NotBooted> {
    let at = under.join(THE_KERNELS_WORDS);
    let words = std::fs::read_to_string(&at).map_err(|why| NotBooted::NotRead {
        path: at.display().to_string(),
        why: why.to_string(),
    })?;
    let named = named_in(&words).ok_or_else(|| NotBooted::NamesNoDeployment {
        path: at.display().to_string(),
    })?;
    let folder = under.join(named);
    folder.canonicalize().map_err(|why| NotBooted::NotThere {
        path: folder.display().to_string(),
        why: why.to_string(),
    })
}

/// The deployment a kernel command line names, without the leading separator
/// so that it joins onto a root rather than replacing one.
///
/// Only an absolute path is read: a relative one is not something the base
/// writes, and joining it onto a root would name a folder decided by wherever
/// this process happened to be.
fn named_in(words: &str) -> Option<&str> {
    words
        .split_whitespace()
        .find_map(|word| word.strip_prefix(THE_WORD))
        .and_then(|path| path.strip_prefix('/'))
        .filter(|path| !path.is_empty())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A machine of this test's own, with nothing in it.
    fn a_machine(named: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-updating-booted-{named}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(folder.join("proc")).unwrap();
        folder
    }

    /// The words the kernel of a real alo OS machine was started with, as one
    /// was measured on 2026-09-21, with the deployment it names made.
    fn a_machine_that_booted(named: &str, deployment: &str) -> PathBuf {
        let folder = a_machine(named);
        let deployed = folder.join("ostree/deploy/default/deploy").join(deployment);
        std::fs::create_dir_all(&deployed).unwrap();
        std::fs::write(
            folder.join(THE_KERNELS_WORDS),
            format!(
                "BOOT_IMAGE=(hd0,gpt3)/boot/ostree/default-abc/vmlinuz root=UUID=1234 \
                 rw ostree=/ostree/deploy/default/deploy/{deployment}\n"
            ),
        )
        .unwrap();
        folder
    }

    /// **The deployment the kernel names is the one that is read**, out of a
    /// command line with everything else on it that a real one carries.
    #[test]
    fn the_deployment_the_kernel_names_is_the_one_read() {
        let machine = a_machine_that_booted("named", "f9ce6166.0");

        let folder = booted(&machine).unwrap();

        assert!(folder.ends_with("f9ce6166.0"), "{}", folder.display());
    }

    /// **A machine that did not boot from a deployment says so**, rather than
    /// being read as one with nothing running: every developer's machine in
    /// this repository is one of these.
    #[test]
    fn a_machine_that_booted_no_deployment_says_so() {
        let machine = a_machine("no-deployment");
        std::fs::write(
            machine.join(THE_KERNELS_WORDS),
            "BOOT_IMAGE=/vmlinuz root=UUID=1234 rw quiet\n",
        )
        .unwrap();

        assert!(matches!(
            booted(&machine),
            Err(NotBooted::NamesNoDeployment { .. })
        ));
    }

    /// **Words that are not there are refused**, and told apart from words that
    /// name nothing: one is a machine this cannot ask, the other is a machine
    /// that answered.
    #[test]
    fn words_that_are_not_there_are_told_apart_from_words_that_name_nothing() {
        let machine = a_machine("no-words");

        assert!(matches!(booted(&machine), Err(NotBooted::NotRead { .. })));
    }

    /// **A deployment named and not on the disk is refused**, never answered
    /// with the folder that is not there: the machine is then not one this can
    /// say anything about.
    #[test]
    fn a_deployment_that_is_not_on_the_disk_is_refused() {
        let machine = a_machine("gone");
        std::fs::write(
            machine.join(THE_KERNELS_WORDS),
            "rw ostree=/ostree/deploy/default/deploy/f9ce6166.0\n",
        )
        .unwrap();

        assert!(matches!(booted(&machine), Err(NotBooted::NotThere { .. })));
    }

    /// **A relative path is not a deployment.** The base never writes one, and
    /// joining one onto a root would name a folder decided by wherever this
    /// process was started.
    #[test]
    fn a_relative_path_is_not_read_as_a_deployment() {
        assert_eq!(named_in("rw ostree=ostree/deploy/default/deploy/a.0"), None);
        assert_eq!(named_in("rw ostree=/"), None);
        assert_eq!(
            named_in("rw ostree=/ostree/boot.1/default/abc/0"),
            Some("ostree/boot.1/default/abc/0")
        );
    }
}
