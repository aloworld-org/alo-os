//! What in an adapter's declaration would let a model's words become code.
//!
//! The contract's one rule: **the adapter exposes typed verbs and generates any
//! script internally, from validated arguments; the model never authors code
//! that executes.** `alo-capability` already guarantees no argument *carries*
//! free text. What an adapter adds is the other half — where a validated value
//! **lands** in the application — and there are three ways that goes wrong, each
//! refused here:
//!
//! 1. **an argument declared as a script, a command or free text** —
//!    [`crate::Kind::is_code`];
//! 2. **a parameter the application interprets** — [`crate::Part::Evaluated`];
//! 3. **a method that runs things, or an argument that picks which action
//!    runs.** An application's `ActivateAction` with the action's name taken
//!    from an argument is one verb whose meaning a model chooses among
//!    everything the application registers, including *quit*. So the action's
//!    name must be a literal the adapter's author wrote.
//!
//! **A tripwire, like `alo_capability::verb`'s, and it says so.** Somebody
//! determined to reach an interpreter through a method called `Refresh` is not
//! stopped by a word list. What stops them is review of the declaration — which
//! is data, short, and readable by somebody who did not write it — and the
//! adapter allowlist (v1). What this catches is the author who did not know.

/// The words that mean a method runs something, matched word by word.
const RUNS: [&str; 20] = [
    "eval",
    "evaluate",
    "exec",
    "execute",
    "run",
    "script",
    "scripts",
    "scripting",
    "command",
    "commands",
    "shell",
    "spawn",
    "interpret",
    "interpreter",
    "expression",
    "python",
    "javascript",
    "lua",
    "console",
    "launch",
];

/// The methods whose first parameter chooses which of the application's
/// actions runs: `(interface, method)`.
pub const CHOOSES_AN_ACTION: [(&str, &str); 3] = [
    ("org.freedesktop.Application", "ActivateAction"),
    ("org.gtk.Actions", "Activate"),
    ("org.gtk.Actions", "SetState"),
];

/// The words of a name: split at dots, underscores and a lower-case letter
/// followed by a capital, lower-cased.
fn words_of(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut previous_lower = false;
    for c in name.chars() {
        if c == '.' || c == '_' || c == '-' {
            words.push(std::mem::take(&mut word));
            previous_lower = false;
            continue;
        }
        if c.is_ascii_uppercase() && previous_lower {
            words.push(std::mem::take(&mut word));
        }
        previous_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        word.push(c.to_ascii_lowercase());
    }
    words.push(word);
    words.retain(|word| !word.is_empty());
    words
}

/// Whether an interface or a method is named for running something.
#[must_use]
pub fn runs_something(interface: &str, method: &str) -> bool {
    words_of(interface)
        .iter()
        .chain(words_of(method).iter())
        .any(|word| RUNS.contains(&word.as_str()))
}

/// Whether this method's first parameter chooses which action runs.
#[must_use]
pub fn chooses_an_action(interface: &str, method: &str) -> bool {
    CHOOSES_AN_ACTION.contains(&(interface, method))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methods_named_for_running_something_are_caught_word_by_word() {
        for (interface, method) in [
            ("org.gnome.Shell", "Eval"),
            ("org.blender.Scripting", "Open"),
            ("org.example.App", "RunCommand"),
            ("org.example.App", "ExecuteScript"),
            ("org.example.Python", "Call"),
            ("org.example.App", "run_python"),
        ] {
            assert!(runs_something(interface, method), "{interface} {method}");
        }
        for (interface, method) in [
            ("org.freedesktop.Application", "Open"),
            ("org.freedesktop.Application", "ActivateAction"),
            ("org.example.App", "Running"),
            ("org.example.Evaluation", "Show"),
        ] {
            assert!(!runs_something(interface, method), "{interface} {method}");
        }
    }

    #[test]
    fn an_actions_name_is_the_first_parameter_of_three_methods() {
        assert!(chooses_an_action(
            "org.freedesktop.Application",
            "ActivateAction"
        ));
        assert!(chooses_an_action("org.gtk.Actions", "Activate"));
        assert!(!chooses_an_action("org.freedesktop.Application", "Open"));
    }
}
