//! **Where the person's own folders are on this computer**, as one line between
//! their shell and the agent, and as the words a model is told.
//!
//! Measured on a booted machine on 2026-11-10 (the walk of `f028d5ec`): asked
//! *make a folder called demo in my Documents*, the model ran
//! `mkdir /Users/username/Documents/demo` — a macOS home with a placeholder
//! name — and it failed, because the person's home is `/var/home/alo` and
//! nothing had ever told the model so. The terminal's own header showed it.
//!
//! So a model is told the home folder and the person's standard folders,
//! **under the names they really have** — `Dokumente` for a German person —
//! which are the names `alo-their-folders` gave them and recorded in
//! `~/.config/user-dirs.dirs` when the account was made:
//!
//! ```text
//! alo-their-folders:{"home":"/var/home/anna","folders":{"Documents":"/var/home/anna/Dokumente"}}
//! ```
//!
//! It is the fifth line on the asking socket, after the window offered
//! (`docs/contracts/asking-socket.md`), for the reason every line is after the
//! last one an older agent reads: an agent written before it never reads it.
//!
//! # The machine's layout, not the person's content
//!
//! Where a folder is, and never what is in it. It is read on the person's side
//! **at the moment they send a request** and travels inside that request only —
//! *context is offered, never watched* (`CLAUDE.md`). It leaves the machine only
//! inside the question, as the folders a person granted already do.
//!
//! # A hint, never a capability
//!
//! **Knowing where a folder is allows nothing.** What a verb may reach is still
//! decided by `alo-agentd` against the grants it reads itself, at the person's
//! own level; a model told about a folder it was not granted is refused there
//! exactly as it would be for any other path, and the model is told so.

use std::path::Path;

use serde_json::{Map, Value, json};

/// The marker the line begins with.
pub const THE_PERSONS_FOLDERS: &str = "alo-their-folders:";

/// **The most standard folders a line names.** `alo-their-folders` makes three;
/// the bound leaves room for the others `xdg-user-dirs` knows and refuses a
/// line no shell would write.
pub const THE_MOST_FOLDERS: usize = 16;

/// The longest path a line names, in bytes: Linux's own `PATH_MAX`.
pub const THE_LONGEST_PATH: usize = 4096;

/// The longest name a folder is given in the line, in characters
/// (`Documents`, `Downloads`, `Pictures`).
pub const THE_LONGEST_ROLE: usize = 64;

/// The most a line may be before it is read at all, in bytes: every path at
/// its longest, escaped, and the envelope.
pub const THE_LONGEST_FOLDERS_LINE: usize =
    2 * (THE_MOST_FOLDERS + 1) * (THE_LONGEST_PATH + 4 * THE_LONGEST_ROLE) + 256;

/// The person's home folder, and where their standard folders are in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThePersonsFolders {
    /// The home folder, an absolute path.
    home: String,
    /// Each standard folder by what it is for — `Documents` — and where it
    /// is, in the order given.
    folders: Vec<(String, String)>,
}

/// Why a line could not be read as the person's folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTheirFolders {
    /// The line does not begin with [`THE_PERSONS_FOLDERS`].
    NotMarked,
    /// Longer than a shell writes.
    TooLong(usize),
    /// Not the JSON a shell writes.
    NotReadable(String),
}

impl std::fmt::Display for NotTheirFolders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMarked => write!(f, "the line after the window is not the person's folders"),
            Self::TooLong(bytes) => write!(
                f,
                "the person's folders are described in {bytes} bytes, more than a shell writes, \
                 so they were not read"
            ),
            Self::NotReadable(why) => write!(f, "the person's folders could not be read: {why}"),
        }
    }
}

impl std::error::Error for NotTheirFolders {}

/// Whether `path` is one a line may name: absolute, one line, within the bound.
fn a_path_a_line_may_name(path: &str) -> bool {
    path.starts_with('/')
        && path.len() <= THE_LONGEST_PATH
        && !path.contains(['\n', '\r', '\0'])
}

/// Whether `role` is a name a line may give a folder.
fn a_role_a_line_may_name(role: &str) -> bool {
    !role.trim().is_empty()
        && role.chars().count() <= THE_LONGEST_ROLE
        && !role.contains(['\n', '\r', '\0'])
}

impl ThePersonsFolders {
    /// **The home at `home`, with these standard folders in it** — each by
    /// what it is for and where it is.
    ///
    /// [`None`] for a home that is not an absolute path a line may carry,
    /// because a model told a relative home would guess again. A folder that
    /// could not travel is left out, and the others still go.
    #[must_use]
    pub fn of<'a>(home: &Path, folders: impl IntoIterator<Item = (&'a str, &'a Path)>) -> Option<Self> {
        let home = home.to_str()?.to_owned();
        if !a_path_a_line_may_name(&home) {
            return None;
        }
        let folders = folders
            .into_iter()
            .filter_map(|(role, at)| Some((role.trim().to_owned(), at.to_str()?.to_owned())))
            .filter(|(role, at)| a_role_a_line_may_name(role) && a_path_a_line_may_name(at))
            .take(THE_MOST_FOLDERS)
            .collect();
        Some(Self { home, folders })
    }

    /// The home folder.
    #[must_use]
    pub fn home(&self) -> &str {
        &self.home
    }

    /// Each standard folder, by what it is for, and where it is.
    #[must_use]
    pub fn folders(&self) -> &[(String, String)] {
        &self.folders
    }

    /// **The line a shell writes**, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        let folders: Map<String, Value> = self
            .folders
            .iter()
            .map(|(role, at)| (role.clone(), Value::String(at.clone())))
            .collect();
        format!(
            "{THE_PERSONS_FOLDERS}{}",
            json!({ "home": self.home, "folders": folders })
        )
    }

    /// **The line read back**, with or without its newline.
    ///
    /// # Errors
    /// [`NotTheirFolders`], for anything a current shell would not write: an
    /// object with any other field, a home or a folder that is not an absolute
    /// path, or more folders than [`THE_MOST_FOLDERS`].
    pub fn read(line: &str) -> Result<Self, NotTheirFolders> {
        let line = line.trim_end_matches(['\n', '\r']);
        if line.len() > THE_LONGEST_FOLDERS_LINE {
            return Err(NotTheirFolders::TooLong(line.len()));
        }
        let payload = line
            .strip_prefix(THE_PERSONS_FOLDERS)
            .ok_or(NotTheirFolders::NotMarked)?;
        let not = |why: &str| NotTheirFolders::NotReadable(why.to_owned());
        let value: Value = serde_json::from_str(payload).map_err(|why| not(&why.to_string()))?;
        let whole = value
            .as_object()
            .filter(|whole| whole.len() == 2)
            .ok_or_else(|| not("it is not an object holding a home and its folders"))?;
        let home = whole
            .get("home")
            .and_then(Value::as_str)
            .filter(|home| a_path_a_line_may_name(home))
            .ok_or_else(|| not("its home is not an absolute path"))?;
        let named = whole
            .get("folders")
            .and_then(Value::as_object)
            .ok_or_else(|| not("its folders are not an object"))?;
        if named.len() > THE_MOST_FOLDERS {
            return Err(not("it names more folders than a shell writes"));
        }
        let mut folders = Vec::with_capacity(named.len());
        for (role, at) in named {
            let at = at
                .as_str()
                .filter(|at| a_path_a_line_may_name(at))
                .ok_or_else(|| not("a folder in it is not an absolute path"))?;
            if !a_role_a_line_may_name(role) {
                return Err(not("a folder in it is named by nothing a shell writes"));
            }
            folders.push((role.clone(), at.to_owned()));
        }
        Ok(Self {
            home: home.to_owned(),
            folders,
        })
    }

    /// **The line a shell writes for this request**, without its newline: the
    /// folders, or the marker alone where the shell knows no home.
    ///
    /// **Always written**, as the window's marker is, so an agent that reads
    /// the line finds one where a current shell sent it and never waits for a
    /// line that is not coming.
    #[must_use]
    pub fn the_line_for(folders: Option<&Self>) -> String {
        folders.map_or_else(|| THE_PERSONS_FOLDERS.to_owned(), Self::written)
    }

    /// **The line read back**: the folders, or [`None`] for the marker alone.
    ///
    /// # Errors
    /// [`NotTheirFolders`], as [`Self::read`].
    pub fn read_the_line(line: &str) -> Result<Option<Self>, NotTheirFolders> {
        let bare = line.trim_end_matches(['\n', '\r']);
        if bare == THE_PERSONS_FOLDERS {
            return Ok(None);
        }
        Self::read(bare).map(Some)
    }

    /// **The words a model is told**: where the home and the standard folders
    /// are, that these are the paths to name, and that knowing them allows
    /// nothing.
    #[must_use]
    pub fn told(&self) -> String {
        let mut text = String::from(
            "\n\nThis computer's own layout, so that a path you name is one that exists here. \
             The person's home folder is ",
        );
        text.push_str(&self.home);
        text.push_str(", and ~ means that folder.");
        if !self.folders.is_empty() {
            text.push_str(" Their own folders, under the names they have on this computer:\n");
            for (role, at) in &self.folders {
                text.push_str("  ");
                text.push_str(role);
                text.push_str(": ");
                text.push_str(at);
                text.push('\n');
            }
            text.push_str(
                "When the person names one of these, such as their documents, use the path \
                 above, even where it is named in another language.",
            );
        }
        text.push_str(
            " Never name a path from another computer's layout, such as /Users/name or \
             /home/user. Knowing where a folder is does not allow reaching it: what you may \
             reach is decided exactly as for any other path.",
        );
        text
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A German person's home, with their documents in `Dokumente`.
    fn annas() -> ThePersonsFolders {
        ThePersonsFolders::of(
            Path::new("/var/home/anna"),
            [
                ("Documents", Path::new("/var/home/anna/Dokumente")),
                ("Downloads", Path::new("/var/home/anna/Downloads")),
            ],
        )
        .unwrap()
    }

    /// **What a shell writes is what an agent reads**, whole.
    #[test]
    fn the_line_reads_back_as_it_was_written() {
        let written = annas().written();
        assert!(written.starts_with(THE_PERSONS_FOLDERS), "{written}");
        assert_eq!(ThePersonsFolders::read(&written).unwrap(), annas());
        assert_eq!(
            ThePersonsFolders::read_the_line(&format!("{written}\n")).unwrap(),
            Some(annas())
        );
        assert_eq!(
            ThePersonsFolders::read_the_line(&format!("{THE_PERSONS_FOLDERS}\n")).unwrap(),
            None,
            "the marker alone is a shell that knows no home"
        );
    }

    /// **The model is told the home and the real names**, and is told not to
    /// guess another computer's layout — the walk's `/Users/username`.
    #[test]
    fn the_model_is_told_the_home_and_the_names_the_folders_really_have() {
        let told = annas().told();
        assert!(told.contains("home folder is /var/home/anna"), "{told}");
        assert!(told.contains("Documents: /var/home/anna/Dokumente"), "{told}");
        assert!(told.contains("/Users/name"), "{told}");
        assert!(told.contains("does not allow reaching it"), "{told}");
    }

    /// **The whole text a turn shows carries the home**, before what may be
    /// reached and before the request — and a turn whose shell said nothing
    /// about a home is shown exactly what it was shown before.
    #[test]
    fn the_text_a_turn_shows_carries_the_home_before_the_request() {
        let verbs = crate::testing::the_verbs();
        let request = "make a folder called demo in my Documents";
        let granted = crate::WhatItMayReach::these([Path::new("/var/home/anna/Dokumente")]);
        let shown = crate::shown_to_a_turn(
            &verbs,
            &granted.clone().in_this_home(Some(annas())),
            request,
        );
        let home = shown.find("home folder is /var/home/anna").unwrap();
        let reach = shown.find("The folders you may reach").unwrap();
        let asked = shown.find(request).unwrap();
        assert!(home < reach && reach < asked, "{shown}");
        assert_eq!(
            crate::shown_to_a_turn(&verbs, &granted.clone().in_this_home(None), request),
            crate::shown_to_a_turn(&verbs, &granted, request),
            "a turn told no home was shown something new"
        );
    }

    /// **A home that is not an absolute path does not travel**, and a folder
    /// that cannot does not cost the others.
    #[test]
    fn a_home_that_is_not_absolute_is_not_sent_and_a_bad_folder_is_left_out() {
        assert_eq!(ThePersonsFolders::of(Path::new("home/anna"), []), None);
        let some = ThePersonsFolders::of(
            Path::new("/var/home/anna"),
            [
                ("Documents", Path::new("Dokumente")),
                ("Pictures", Path::new("/var/home/anna/Bilder")),
            ],
        )
        .unwrap();
        assert_eq!(
            some.folders(),
            [("Pictures".to_owned(), "/var/home/anna/Bilder".to_owned())]
        );
    }

    /// **Refused, in words, rather than half read**: a line a shell would not
    /// write is not shown to a model as though it were the person's.
    #[test]
    fn a_line_no_shell_writes_is_refused() {
        for line in [
            "alo-this-window:{}",
            "alo-their-folders:not json",
            r#"alo-their-folders:{"home":"/var/home/anna"}"#,
            r#"alo-their-folders:{"home":"var/home","folders":{}}"#,
            r#"alo-their-folders:{"home":"/var/home/anna","folders":{"Documents":"Dokumente"}}"#,
            r#"alo-their-folders:{"home":"/var/home/anna","folders":{},"more":1}"#,
        ] {
            assert!(ThePersonsFolders::read(line).is_err(), "{line}");
        }
        let long = format!(
            "{THE_PERSONS_FOLDERS}{}",
            "x".repeat(THE_LONGEST_FOLDERS_LINE)
        );
        assert!(matches!(
            ThePersonsFolders::read(&long),
            Err(NotTheirFolders::TooLong(_))
        ));
    }
}
