//! The folders a model is told it may reach, and the sentence when it has
//! none.
//!
//! **Measured on a booted machine on 2026-10-24.** A person granted
//! `/home/alo/Documents` through Settings, asked *what is in my Documents
//! folder*, and the turn spent all eight of its rounds like this:
//!
//! ```text
//! alo has not been granted /home/user/Documents
//! alo has not been granted /home/user/Videos
//! alo has not been granted /home/user/Downloads
//! alo has not been granted /home/user/Pictures
//! ```
//!
//! Every refusal was correct. The grant was real, the daemon enforced it
//! exactly, and the model never once named the folder it had — because
//! `crate::shown` showed it *how to answer, the verbs, and the request*, and
//! **nothing had ever told it what it may reach.** It guessed a conventional
//! home directory and was refused eight times.
//!
//! So this is the third thing a model is shown. It is information rather than
//! instruction: it changes no verb and no shape of reply.
//!
//! # Only what a person granted, and that is the whole privacy argument
//!
//! These folder names leave the machine with every question a provider
//! answers. What is listed here is exactly what a person chose, deliberately,
//! to let the agent reach — by standing in a folder and picking it. A person
//! who has granted a folder to an agent they have also pointed at a provider
//! has already decided that. Nothing else on the disk is named, no folder is
//! listed that was not granted, and an expired grant is not listed because it
//! is not held.
//!
//! # Nothing granted is a sentence, not an empty list
//!
//! A model shown an empty list will guess, which is what the eight rounds
//! above were. Told plainly that it has nothing and that guessing is refused,
//! it can say so in one round and give the person the sentence that actually
//! helps: *a folder is granted by picking it*.

use std::path::Path;

/// The folders a model is told it may reach.
///
/// Sealed: built from what a machine really holds, so nothing downstream can
/// put a folder in front of a model that nobody granted.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WhatItMayReach {
    /// The folders, sorted and without repeats.
    folders: Vec<String>,
}

impl WhatItMayReach {
    /// A machine where nothing has been granted.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// These folders, sorted and deduplicated.
    ///
    /// **Sorted so that two machines holding the same grants show the same
    /// text**, whatever order the grants file happens to list them in — which
    /// is what keeps one model's answer comparable with another's, the reason
    /// `crate::shown` gives for the verbs.
    #[must_use]
    pub fn these<'a>(folders: impl IntoIterator<Item = &'a Path>) -> Self {
        let mut folders: Vec<String> = folders
            .into_iter()
            .map(|folder| folder.display().to_string())
            .filter(|folder| !folder.trim().is_empty())
            .collect();
        folders.sort_unstable();
        folders.dedup();
        Self { folders }
    }

    /// Whether anything has been granted.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.folders.is_empty()
    }

    /// The folders, as they are shown.
    #[must_use]
    pub fn folders(&self) -> &[String] {
        &self.folders
    }

    /// What a model is told, which is a list or a sentence and never an empty
    /// list.
    #[must_use]
    pub fn told(&self) -> String {
        if self.folders.is_empty() {
            return "\n\nYou have not been granted any folder on this computer. Every verb that \
                    reads a file will be refused whatever path you name, so do not guess one. Say \
                    that nothing has been granted, and that a person grants a folder by picking \
                    it in Settings."
                .to_owned();
        }
        let mut text = String::from(
            "\n\nThe folders you may reach, and nothing else on this computer:\n",
        );
        for folder in &self.folders {
            text.push_str("  ");
            text.push_str(folder);
            text.push('\n');
        }
        text.push_str(
            "Name a path inside one of these. A path that is not inside one of them is refused, \
             and guessing one spends a round for nothing.",
        );
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// **The case that cost eight rounds on a real machine.** A model with
    /// nothing granted is told so, rather than shown an empty list to guess
    /// against.
    #[test]
    fn nothing_granted_is_a_sentence_that_tells_it_not_to_guess() {
        let told = WhatItMayReach::nothing().told();
        assert!(told.contains("not been granted any folder"), "{told}");
        assert!(told.contains("do not guess"), "{told}");
        assert!(told.contains("picking it in Settings"), "{told}");
    }

    /// The folder a person granted is named, in full.
    #[test]
    fn a_granted_folder_is_named_so_it_need_not_be_guessed() {
        let folder = PathBuf::from("/home/alo/Documents");
        let told = WhatItMayReach::these([folder.as_path()]).told();
        assert!(told.contains("/home/alo/Documents"), "{told}");
        assert!(told.contains("nothing else on this computer"), "{told}");
    }

    /// **Nothing but what was granted.** The test that would catch a folder
    /// reaching a provider because somebody widened this.
    #[test]
    fn no_folder_appears_that_was_not_granted() {
        let told = WhatItMayReach::these([Path::new("/home/alo/Documents")]).told();
        for never in ["/home/alo/Pictures", "/home/alo/Downloads", "/etc", "/home/alo\n"] {
            assert!(!told.contains(never), "{never} reached a model: {told}");
        }
    }

    /// Two machines holding the same grants show the same text, whatever order
    /// the file listed them in.
    #[test]
    fn the_same_grants_are_shown_the_same_way_whatever_their_order() {
        let one = WhatItMayReach::these([Path::new("/b"), Path::new("/a"), Path::new("/c")]);
        let other = WhatItMayReach::these([Path::new("/c"), Path::new("/a"), Path::new("/b")]);
        assert_eq!(one, other);
        assert_eq!(one.told(), other.told());
    }

    /// One folder granted twice is one folder, not two lines.
    #[test]
    fn a_folder_granted_twice_is_shown_once() {
        let it = WhatItMayReach::these([Path::new("/home/alo/Documents"); 2]);
        assert_eq!(it.folders().len(), 1);
    }

    /// An empty path is not a folder and is not shown as one.
    #[test]
    fn an_empty_path_is_not_shown() {
        assert!(WhatItMayReach::these([Path::new("")]).is_nothing());
    }
}
