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
//!
//! # At full access there is nothing to grant, and that is a sentence too
//!
//! **Measured on a booted machine on 2026-11-11** (the walk of `57b98c15`). The
//! person chose full access, asked *what folders are in this window* about
//! Dolphin showing their home, and was told *nothing has been granted, a person
//! grants a folder by picking it in Settings* — the sentence above, said where
//! it was false. Full access reaches everything the person's account can, with
//! no folder grants at all (ADR 0149), so the list was empty *because* the
//! person had given more, and an empty list was read as having given nothing.
//!
//! So whether the person chose full access is told as well
//! (`crate::at_full_access`), and where they did, the model is told it may
//! name any path their account can open — **and no folder is named**: the
//! sentence says what kind of reach it is, never what is on the disk.

use std::path::Path;

use crate::the_applications_installed::TheApplicationsInstalled;
use crate::the_persons_folders::ThePersonsFolders;

/// The folders a model is told it may reach.
///
/// Sealed: built from what a machine really holds, so nothing downstream can
/// put a folder in front of a model that nobody granted.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WhatItMayReach {
    /// The folders, sorted and without repeats — [`None`] where whoever built
    /// this does not know what is granted and must not pretend to.
    folders: Option<Vec<String>>,
    /// **Where the person's home and standard folders are**, when their shell
    /// said (`crate::the_persons_folders`). Where things are, never what may
    /// be reached: it changes nothing above, and nothing here grants.
    home: Option<ThePersonsFolders>,
    /// **Whether the person chose full access** (ADR 0149), as their shell
    /// said (`crate::at_full_access`). A hint, as the folders are: the daemon
    /// reads the level itself before anything is reached.
    at_full_access: bool,
    /// **The applications installed on this computer, by name**, when the
    /// person's shell said (`crate::the_applications_installed`; the owner,
    /// 2026-11-14: *"Yes, send app names"*). What is here, never what may be
    /// reached: the daemon reads the grants and the installed list itself.
    applications: Option<TheApplicationsInstalled>,
}

impl WhatItMayReach {
    /// **Nobody said**, so a model is told nothing about what it may reach.
    ///
    /// Not the same as nothing being granted, and the difference is a sentence
    /// that would be false. `alo-agentd` wraps an agent's next request for the
    /// pinned runtime and **does not know this person's grants** — `alo-turn`
    /// holds the rule that a question asks the grants nothing. A daemon that
    /// said *you have been granted no folder* there would contradict the
    /// agent's own correct list, inside the same text.
    ///
    /// So this says nothing at all, and the side that knows is the side that
    /// speaks.
    #[must_use]
    pub fn not_said() -> Self {
        Self {
            folders: None,
            home: None,
            at_full_access: false,
            applications: None,
        }
    }

    /// A machine where nothing has been granted, and this is known.
    #[must_use]
    pub fn nothing() -> Self {
        Self {
            folders: Some(Vec::new()),
            home: None,
            at_full_access: false,
            applications: None,
        }
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
        Self {
            folders: Some(folders),
            home: None,
            at_full_access: false,
            applications: None,
        }
    }

    /// **The same, with where the person's own folders are** on this
    /// computer, or nothing where their shell did not say.
    ///
    /// Told before what may be reached, so a model learns where things are
    /// and then which of them it may touch (`crate::the_persons_folders`).
    #[must_use]
    pub fn in_this_home(self, home: Option<ThePersonsFolders>) -> Self {
        Self { home, ..self }
    }

    /// **The same, at full access or not**, as the person's shell said.
    ///
    /// At full access what may be reached is told as everything the person's
    /// account can open, whatever folders were also granted — a grant inside
    /// full access adds nothing a model needs to know.
    #[must_use]
    pub fn at_full_access(self, at_full_access: bool) -> Self {
        Self {
            at_full_access,
            ..self
        }
    }

    /// **The same, with the applications installed on this computer**, or
    /// nothing where the person's shell did not say.
    ///
    /// Told first, before where the folders are, so a model learns what is
    /// here before what it may touch.
    #[must_use]
    pub fn with_these_applications(self, applications: Option<TheApplicationsInstalled>) -> Self {
        Self {
            applications,
            ..self
        }
    }

    /// The applications installed, when the person's shell said.
    #[must_use]
    pub const fn applications(&self) -> Option<&TheApplicationsInstalled> {
        self.applications.as_ref()
    }

    /// Whether the person's shell said they chose full access.
    #[must_use]
    pub const fn is_at_full_access(&self) -> bool {
        self.at_full_access
    }

    /// Where the person's own folders are, when their shell said.
    #[must_use]
    pub const fn home(&self) -> Option<&ThePersonsFolders> {
        self.home.as_ref()
    }

    /// Whether this says nothing at all, because nobody knew.
    #[must_use]
    pub const fn was_not_said(&self) -> bool {
        self.folders.is_none()
    }

    /// Whether it is known that nothing has been granted.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.folders.as_ref().is_some_and(Vec::is_empty)
    }

    /// The folders, as they are shown — empty when nothing was said as well as
    /// when nothing was granted, which [`Self::was_not_said`] tells apart.
    #[must_use]
    pub fn folders(&self) -> &[String] {
        self.folders.as_deref().unwrap_or_default()
    }

    /// What a model is told: a list, a sentence, or — where nobody knew —
    /// nothing at all.
    #[must_use]
    pub fn told(&self) -> String {
        let mut text = self
            .applications
            .as_ref()
            .map_or_else(String::new, TheApplicationsInstalled::told);
        if let Some(home) = &self.home {
            text.push_str(&home.told());
        }
        text.push_str(&self.what_may_be_reached());
        text
    }

    /// What a model is told about the folders granted, or nothing where nobody
    /// knew.
    fn what_may_be_reached(&self) -> String {
        if self.at_full_access {
            return "\n\nThe person chose full access: you may reach every file and folder their \
                    own account can open on this computer, and no folder needs to be granted \
                    first. Name the full path of what you need, in their home folder or wherever \
                    else their account reaches. A file their account cannot open is refused, and \
                    the machine says so."
                .to_owned();
        }
        let Some(folders) = &self.folders else {
            return String::new();
        };
        if folders.is_empty() {
            return "\n\nYou have not been granted any folder on this computer. Every verb that \
                    reads a file will be refused whatever path you name, so do not guess one. Say \
                    that nothing has been granted, and that a person grants a folder by picking \
                    it in Settings."
                .to_owned();
        }
        let mut text =
            String::from("\n\nThe folders you may reach, and nothing else on this computer:\n");
        for folder in folders {
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

    /// **Nobody said is silence, not a claim.** The daemon wraps an agent's
    /// next request and does not know this person's grants; a sentence there
    /// saying nothing was granted would contradict the agent's own correct
    /// list inside the same text.
    #[test]
    fn nobody_having_said_tells_a_model_nothing_at_all() {
        let it = WhatItMayReach::not_said();
        assert!(it.was_not_said());
        assert!(!it.is_nothing(), "not said is not the same as nothing");
        assert_eq!(it.told(), "");
    }

    /// And the two are never confused: nothing granted still speaks.
    #[test]
    fn nothing_granted_and_nobody_saying_are_different_things() {
        assert_ne!(
            WhatItMayReach::not_said().told(),
            WhatItMayReach::nothing().told()
        );
        assert!(WhatItMayReach::nothing().is_nothing());
        assert!(!WhatItMayReach::nothing().was_not_said());
    }

    /// **The case the walk of 2026-11-11 found.** At full access nothing is
    /// granted folder by folder, and the model is told it may reach what the
    /// person's account can — never that nothing was granted.
    #[test]
    fn full_access_with_nothing_granted_is_told_as_everything_the_account_can_open() {
        let told = WhatItMayReach::nothing().at_full_access(true).told();
        assert!(told.contains("chose full access"), "{told}");
        assert!(told.contains("every file and folder"), "{told}");
        for never in [
            "not been granted",
            "picking it in Settings",
            "will be refused whatever",
        ] {
            assert!(!told.contains(never), "{never} at full access: {told}");
        }
    }

    /// **And at every other level nothing changes**: the sentence that sent the
    /// person to Settings is still said where it is true.
    #[test]
    fn below_full_access_nothing_granted_is_still_nothing() {
        let it = WhatItMayReach::nothing().at_full_access(false);
        assert!(!it.is_at_full_access());
        assert_eq!(it.told(), WhatItMayReach::nothing().told());
        assert!(it.told().contains("picking it in Settings"));
    }

    /// **At full access no folder is named**, granted or not: the sentence says
    /// what kind of reach it is and nothing about what is on the disk.
    #[test]
    fn full_access_names_no_folder() {
        let told = WhatItMayReach::these([Path::new("/home/alo/Documents")])
            .at_full_access(true)
            .told();
        assert!(!told.contains("/home/alo/Documents"), "{told}");
        assert!(!told.contains("nothing else on this computer"), "{told}");
    }

    /// Where the person's home is, is still told at full access, before it.
    #[test]
    fn full_access_still_tells_where_the_home_is() {
        let home = ThePersonsFolders::of(Path::new("/var/home/anna"), []);
        let told = WhatItMayReach::nothing()
            .in_this_home(home)
            .at_full_access(true)
            .told();
        let home = told.find("/var/home/anna");
        let full = told.find("chose full access");
        assert!(home.is_some() && full.is_some(), "{told}");
        assert!(home < full, "the home is told before full access: {told}");
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
        for never in [
            "/home/alo/Pictures",
            "/home/alo/Downloads",
            "/etc",
            "/home/alo\n",
        ] {
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
