//! **The shell integration alo OS ships**, which a person installs and alo
//! never installs for them.
//!
//! [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
//! term 6, accepted by the owner on 2026-10-18: *alo OS ships the snippet, the
//! sentence saying plainly what it reveals — every command you run, to alo, on
//! this machine — and a one-press way to add it. alo does not write to
//! anybody's shell startup unprompted.*
//!
//! This is the snippet and the sentence. **The one-press way is not built**, and
//! that is said here rather than discovered: what surface offers it is a
//! decision nobody has made, and a file that installed itself would break the
//! term it was written for.
//!
//! # Why it is `include_str!` and not a path
//!
//! A path is a promise about where a file will be on a machine, and this one
//! has to survive a container image, a developer's checkout and a test. The
//! bytes are the thing; where they end up is the installer's business, and
//! there is no installer yet.
//!
//! # What it is held to
//!
//! `the_snippet_a_person_installs_tests.rs` runs **a real interactive `bash`**
//! on a real pseudoterminal with this sourced, presses the key, and asserts
//! that [`crate::what_the_shell_says_about_the_prompt`] reads back what was
//! typed. A snippet tested by reading it would be a test of the reading.

/// The integration for `bash`, as a person would paste it into `.bashrc`.
///
/// Shipped as bytes rather than a path: see this module's header.
const FOR_BASH: &str = include_str!("../shell-integration/alo.bash");

/// What a person adds to keep a place to return to before each command, for
/// `bash`.
///
/// **A second fragment and not part of the first**, because they are two
/// choices. The integration above tells alo where a person's prompt is and what
/// they typed; this one asks alo to snapshot their files before each command
/// they run, which costs disk and spends
/// [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s undo
/// window. Somebody who wanted the first did not thereby ask for the second,
/// and folding them together would turn installing shell integration into
/// turning on snapshots without being asked.
const SNAPSHOTS_FOR_BASH: &str = include_str!("../shell-integration/alo-before.bash");

/// The key the snippet binds, as a person would describe it.
///
/// `Ctrl`+`X` `Ctrl`+`A`. The owner chose the `Ctrl`+`X` family on 2026-10-18
/// and this is the free pair in it — measured the same day, `Ctrl`+`X`
/// `Ctrl`+`E` is `edit-and-execute-command` in the `bash` a person actually
/// gets, and `Ctrl`+`X` `Ctrl`+`A` is unbound.
///
/// **Not a user-facing string**: a person reads a key in their own language's
/// words for it, which is `alo-strings`' business and not this crate's. This is
/// the readline binding, which is the same in every language.
pub const THE_KEY_IT_BINDS: &str = r#""\C-x\C-a""#;

/// The snippet for a shell, or nothing for one alo does not ship integration
/// for.
///
/// **`zsh` is not here yet**, and answering `None` is how that is said. A
/// snippet that handed a `zsh` user the `bash` one would be worse than nothing:
/// `bind -x` is not a `zsh` builtin, and the error would arrive on every new
/// terminal they opened.
#[must_use]
pub fn for_the_shell_called(name: &str) -> Option<&'static str> {
    // The name a person's `SHELL` holds is a path on most machines, so the last
    // component is what identifies it. `/bin/bash`, `/usr/bin/bash` and `bash`
    // are the same shell and all three reach a person.
    match name.rsplit('/').next().unwrap_or(name) {
        "bash" => Some(FOR_BASH),
        _ => None,
    }
}

/// The startup file a shell reads, by the shell's name.
///
/// **Here rather than in the Settings surface**, for the reason the snippet
/// itself is here: a crate that knows `bash` is the shell it ships a snippet for
/// is the crate that knows `bash` reads `.bashrc`. A surface that spelled the
/// filename would be a second place to change when `zsh` arrives, and the two
/// would disagree.
///
/// `None` for a shell alo ships nothing for, which is the same answer
/// [`for_the_shell_called`] gives and for the same reason.
///
/// **Relative to the person's folder**, never absolute: the folder is the
/// session's answer and this crate does not know it.
#[must_use]
pub fn the_startup_file_of(shell: &str) -> Option<&'static str> {
    match shell.rsplit('/').next().unwrap_or(shell) {
        // `.bashrc` and not `.bash_profile`: the snippet binds a key and sets
        // `PS1`, which an interactive shell needs on every start, and a login
        // shell reads `.bash_profile` once. A person opening a second terminal
        // would have integration in one and not the other.
        "bash" => Some(".bashrc"),
        _ => None,
    }
}

/// What adding the snippet to a shell's startup file did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Added {
    /// It was appended, and the next shell a person opens reads it.
    Appended,
    /// A copy was already there, so **nothing was written**.
    ///
    /// Told apart from [`Added::Appended`] because a person who presses twice
    /// has not broken anything and should not be told they have, and because a
    /// second copy of this snippet in one startup file would bind the key twice
    /// and prepend the prompt marks twice.
    AlreadyThere,
}

/// Why the snippet was not added.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAdded {
    /// alo OS ships no integration for this shell.
    ///
    /// Named rather than silently doing nothing: a `zsh` user pressing the row
    /// is owed *we do not have one for your shell yet*, which is true, and not
    /// *it did not work*, which sounds like a fault.
    #[error("alo OS ships no shell integration for {0}")]
    NoIntegrationFor(String),
    /// The startup file is there and would not be read.
    #[error("{at} is there and would not be read: {why}")]
    WouldNotRead {
        /// The file that would not be read.
        at: String,
        /// What the machine said.
        why: String,
    },
    /// The startup file would not take a write.
    #[error("{at} would not take a write: {why}")]
    WouldNotWrite {
        /// The file that would not take a write.
        at: String,
        /// What the machine said.
        why: String,
    },
}

/// The line every copy of a snippet begins with, which is how one already
/// there is recognised.
///
/// **Taken from the snippet rather than written out here.** A marker spelled in
/// this file *and* in `alo.bash` is two sources of truth, and the day they
/// differ is the day a second copy is appended to somebody's startup file —
/// which binds the key twice and prepends the prompt marks twice.
fn the_line_it_begins_with(snippet: &str) -> &str {
    snippet.lines().next().unwrap_or(snippet)
}

/// Whether a startup file already holds alo's snippet for a shell.
///
/// `None` where alo ships nothing for that shell, because then there is nothing
/// to look for — which is a different answer from *it is not there*, and
/// collapsing the two is how a `zsh` user gets offered a `bash` snippet.
///
/// **Takes the file's text rather than its path**, so a test chooses what it
/// holds instead of discovering it.
#[must_use]
pub fn already_in(startup: &str, shell: &str) -> Option<bool> {
    let snippet = for_the_shell_called(shell)?;
    Some(startup.contains(the_line_it_begins_with(snippet)))
}

/// What a person adds to be able to go back, for this shell, or [`None`] where
/// alo ships nothing for it.
#[must_use]
pub fn for_snapshots_in_the_shell_called(name: &str) -> Option<&'static str> {
    match name.rsplit('/').next().unwrap_or(name) {
        "bash" => Some(SNAPSHOTS_FOR_BASH),
        _ => None,
    }
}

/// Whether a startup file already asks for a place to return to.
///
/// [`None`] where alo ships nothing for that shell, for [`already_in`]'s
/// reason: *there is nothing to look for* and *it is not there* are different
/// answers.
#[must_use]
pub fn snapshots_already_in(startup: &str, shell: &str) -> Option<bool> {
    let snippet = for_snapshots_in_the_shell_called(shell)?;
    Some(startup.contains(the_line_it_begins_with(snippet)))
}

/// Add the place-keeping fragment to a shell's startup file, once.
///
/// The same rules as [`add_to`] and for the same reasons: a press is the
/// prompt, it appends and never rewrites, and a file that is not there yet is
/// not an error.
///
/// # Errors
/// As [`add_to`].
pub fn add_snapshots_to(startup: &std::path::Path, shell: &str) -> Result<Added, NotAdded> {
    let snippet = for_snapshots_in_the_shell_called(shell)
        .ok_or_else(|| NotAdded::NoIntegrationFor(shell.to_owned()))?;
    appended(startup, snippet)
}

/// Add the snippet to a shell's startup file, once, and never twice.
///
/// **This is the one-press way [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
/// term 6 promises**, and it is called from a row a person pressed. Nothing in
/// this crate calls it on its own: *alo does not write to anybody's shell
/// startup unprompted* is the term, and a press is the prompt.
///
/// **It appends and never rewrites.** A whole-file write that failed halfway
/// would lose a startup file a person has had for years, and nothing in this
/// change is worth that; an append cannot lose what was already there.
///
/// # Errors
///
/// [`NotAdded::NoIntegrationFor`] for a shell alo ships nothing for,
/// [`NotAdded::WouldNotRead`] where the file is there and will not be read, and
/// [`NotAdded::WouldNotWrite`] where it will not take the append. A startup
/// file that does not exist yet is **not** an error: a person who has never
/// edited one still gets the integration.
pub fn add_to(startup: &std::path::Path, shell: &str) -> Result<Added, NotAdded> {
    let snippet =
        for_the_shell_called(shell).ok_or_else(|| NotAdded::NoIntegrationFor(shell.to_owned()))?;
    appended(startup, snippet)
}

/// Append one snippet to a startup file, once, whichever snippet it is.
///
/// **One appending rule rather than two**, because the rule is the delicate
/// part — a newline where the file did not end with one, a blank line between
/// what was there and what is added, and never a rewrite — and two copies of it
/// is the fault `docs/misreadings/a-check-fixed-in-one-place-leaves-its-copy.md`
/// is named after.
fn appended(startup: &std::path::Path, snippet: &str) -> Result<Added, NotAdded> {
    use std::io::Write as _;

    let at = || startup.display().to_string();

    let held = match std::fs::read_to_string(startup) {
        Ok(held) => held,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(why) => {
            return Err(NotAdded::WouldNotRead {
                at: at(),
                why: why.to_string(),
            });
        }
    };
    if held.contains(the_line_it_begins_with(snippet)) {
        return Ok(Added::AlreadyThere);
    }

    // **A blank line between what was there and what is added**, and a newline
    // first where the file did not end with one — a snippet appended onto the
    // tail of somebody's last line is a syntax error in their shell.
    let mut adding = String::new();
    if !held.is_empty() {
        if !held.ends_with('\n') {
            adding.push('\n');
        }
        adding.push('\n');
    }
    adding.push_str(snippet);
    if !adding.ends_with('\n') {
        adding.push('\n');
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(startup)
        .map_err(|why| NotAdded::WouldNotWrite {
            at: at(),
            why: why.to_string(),
        })?;
    file.write_all(adding.as_bytes())
        .map_err(|why| NotAdded::WouldNotWrite {
            at: at(),
            why: why.to_string(),
        })?;
    Ok(Added::Appended)
}

#[cfg(all(test, target_os = "linux"))]
#[path = "the_snippet_a_person_installs_tests.rs"]
mod tests;
