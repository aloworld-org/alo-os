//! **Starting one of the media server's own tools**, with the environment it
//! needs and nothing else.
//!
//! Four crates start these tools — `alo-in-use` reads the graph, `alo-sound`
//! reads and sets devices, `alo-cameras` lists what can see, and
//! `alo-capturing` opens a stream — and every one of them had its own copy of
//! these fifteen lines. The copies drifted, which is how `alo-in-use` spent a
//! day telling a machine with a working server that its server would not answer.
//!
//! # The rules, in one place
//!
//! - **The program is started directly**, so no argument is interpreted by
//!   anything but the tool: no shell, ever.
//! - **The environment is cleared**, and given back exactly what the tool needs:
//!   where this machine's own programs are, and the C locale — what these tools
//!   print is read to decide things, and a translated field name is a record
//!   nothing recognises.
//! - **And where the media server is listening** ([`WHERE_THE_SERVER_IS`]),
//!   passed through when the caller has one. This is the line that was missing
//!   in one copy: the tools find the server by that variable and fall back to a
//!   directory named after the user's number, which is the same place on an
//!   ordinary login and a different place on every test session and every
//!   machine running more than one.
//! - **Nothing else of the caller's environment**, and the variable only when it
//!   is set: a service started without one is a service asking this machine's
//!   own default.
//!
//! A caller that needs one more variable — `alo-capturing` tells the server what
//! the client calls itself — adds it with [`ATool::also`], where it is visible.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where a machine's own programs are, for a cleared environment.
pub const WHERE_ITS_PROGRAMS_ARE: &str = "/usr/bin:/bin";

/// Where the media server is listening, which a session sets.
pub const WHERE_THE_SERVER_IS: &str = "XDG_RUNTIME_DIR";

/// **One of the media server's tools, ready to start.**
#[derive(Debug)]
pub struct ATool {
    /// What is started.
    program: String,
    /// What it is given.
    arguments: Vec<String>,
    /// Anything else of the environment this caller needs, by name.
    also: Vec<(String, std::ffi::OsString)>,
}

impl ATool {
    /// The tool of this name.
    #[must_use]
    pub fn named(program: &str) -> Self {
        Self {
            program: program.to_owned(),
            arguments: Vec::new(),
            also: Vec::new(),
        }
    }

    /// Asked for this.
    #[must_use]
    pub fn asking(mut self, arguments: &[String]) -> Self {
        self.arguments = arguments.to_vec();
        self
    }

    /// With one more thing from the environment, where the caller has it.
    ///
    /// Everything a tool is given is visible at the call, which is the point:
    /// the day one of these matters, it is in the caller's own file rather than
    /// inherited from whatever started the process.
    #[must_use]
    pub fn also(mut self, named: &str, value: Option<impl AsRef<OsStr>>) -> Self {
        if let Some(value) = value {
            self.also
                .push((named.to_owned(), value.as_ref().to_os_string()));
        }
        self
    }

    /// What this tool is called.
    #[must_use]
    pub fn program(&self) -> &str {
        &self.program
    }

    /// **The command, with the environment these rules give it** — with where
    /// the server is listening handed in.
    ///
    /// The variable arrives as an argument, as `alo-choosing` takes the ones it
    /// reads, and for the same reason: a machine without one is then a test
    /// rather than something somebody has to arrange on a real login.
    /// [`ATool::started_here`] is this with the one this process was started
    /// with.
    #[must_use]
    pub fn started_where(&self, listening: Option<impl AsRef<OsStr>>) -> Command {
        // A changed PATH with a bare executable forces Rust's fork/exec path.
        // Resolve only within the same fixed system PATH before spawning, so a
        // graph request need not write-protect the compositor's frame buffers.
        let search_path = self
            .also
            .iter()
            .rev()
            .find(|(name, _)| name == "PATH")
            .map_or(OsStr::new(WHERE_ITS_PROGRAMS_ARE), |(_, value)| {
                value.as_os_str()
            });
        let mut command = Command::new(system_program(&self.program, search_path));
        command
            .args(&self.arguments)
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("PATH", WHERE_ITS_PROGRAMS_ARE)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(listening) = listening {
            command.env(WHERE_THE_SERVER_IS, listening.as_ref());
        }
        for (named, value) in &self.also {
            command.env(named, value);
        }
        command
    }

    /// **The same, with the runtime directory this process was started with.**
    ///
    /// The one place in this crate that reads an environment, so that a caller
    /// which has been handed one somewhere else can say so instead.
    #[must_use]
    pub fn started_here(&self) -> Command {
        self.started_where(std::env::var_os(WHERE_THE_SERVER_IS))
    }
}

/// Preserve explicit paths and search only the environment this tool receives.
/// Missing programs keep the ordinary command's existing not-found behavior.
fn system_program(program: &str, search_path: &OsStr) -> PathBuf {
    let path = Path::new(program);
    if path.components().count() > 1 || path.is_absolute() {
        return path.to_owned();
    }
    std::env::split_paths(search_path)
        .map(|directory| directory.join(path))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The environment is what these rules say and nothing else**, including
    /// where the server is listening.
    #[test]
    fn a_tool_is_given_the_machines_programs_the_c_locale_and_where_the_server_is() {
        let started = ATool::named("pw-dump")
            .asking(&["--help".to_owned()])
            .also("PIPEWIRE_PROPS", Some("a-client"))
            .started_where(Some("/run/somewhere"));
        let given: Vec<(String, Option<String>)> = started
            .get_envs()
            .map(|(named, value)| {
                (
                    named.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect();

        for (named, value) in [
            ("PATH", WHERE_ITS_PROGRAMS_ARE),
            ("LC_ALL", "C"),
            ("LANG", "C"),
            (WHERE_THE_SERVER_IS, "/run/somewhere"),
            ("PIPEWIRE_PROPS", "a-client"),
        ] {
            assert!(
                given.contains(&(named.to_owned(), Some(value.to_owned()))),
                "{named} was not given to the tool: {given:?}"
            );
        }
        assert_eq!(
            given.len(),
            5,
            "the tool was given something these rules do not name: {given:?}"
        );
        assert_eq!(
            Path::new(started.get_program()).file_name(),
            Some(OsStr::new("pw-dump"))
        );
    }

    /// **Something the caller does not have is not passed as nothing.**
    #[test]
    fn a_variable_the_caller_has_not_got_is_simply_not_passed() {
        let started = ATool::named("pw-dump")
            .also("PIPEWIRE_PROPS", None::<&str>)
            .started_where(None::<&str>);
        assert!(
            !started
                .get_envs()
                .any(|(named, _)| named == OsStr::new("PIPEWIRE_PROPS"))
        );
    }

    /// A system tool remains reachable after PATH is cleared for its child.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_known_system_tool_is_started_by_its_absolute_path() {
        let mut command = ATool::named("true").started_where(None::<&str>);
        let program = Path::new(command.get_program());
        assert!(program.is_absolute());
        assert!(matches!(
            program.to_str(),
            Some("/usr/bin/true" | "/bin/true")
        ));
        command.env("PATH", "/missing-system-programs");
        assert!(command.status().is_ok_and(|status| status.success()));
    }

    /// Explicit tools and a missing program preserve their existing outcomes.
    #[test]
    fn explicit_and_missing_tool_names_are_preserved() {
        for program in ["/missing-tools/probe", "./probe", "alo-missing-media-tool"] {
            let command = ATool::named(program).started_where(None::<&str>);
            assert_eq!(command.get_program(), OsStr::new(program));
        }
        let mut command = ATool::named("true")
            .also("PATH", Some("/missing-system-programs"))
            .started_where(None::<&str>);
        assert_eq!(command.get_program(), OsStr::new("true"));
        assert!(
            command
                .status()
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        );
    }
}
