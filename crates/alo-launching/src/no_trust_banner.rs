//! **alo's own copy of a code editor shows no Restricted Mode banner, and keeps
//! Restricted Mode** — the owner's of 2026-11-15, after the walk on `main`
//! (run 43751841598) showed the banner across the top of the editor alo was
//! working in.
//!
//! **Only the banner.** Workspace trust is left as VSCodium has it: a folder
//! nobody trusted still runs none of its tasks, debuggers or extension code,
//! which is law 2's concern, and the status bar still says *Restricted Mode*
//! and opens the trust settings when pressed. What the person no longer sees
//! is the line under the title saying so, with its *Manage* and *Learn More*.
//!
//! `"never"` is honoured by VSCodium 1.143.17232: read in its workbench, which
//! returns no banner when the setting is `"never"`; not seen on a booted
//! machine.

/// The setting, by VSCodium's own name, and its value written as JSON.
pub const NO_TRUST_BANNER: [(&str, &str); 1] = [("security.workspace.trust.banner", r#""never""#)];
