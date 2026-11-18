//! **The person's windows, as a turn reaches them** — the one implementation of
//! `alo_turn::TheWindowsOnTheScreen` that opens a real accessibility tree.
//!
//! This daemon runs as the person (`User=` in its unit) with their session bus
//! in its environment, so the tree it opens is the one their applications
//! publish and Orca reads. **One tree per read or press**: [`ThePersonsWindows`]
//! holds nothing but the place to look, and the connection an opening makes
//! closes when the turn's executor drops it — so nothing stays attached to a
//! person's applications between acts, and with no invocation nothing is
//! asked at all.
//!
//! # Which switch would help, read at each act
//!
//! *Let alo work fully inside apps like VSCodium* and *Apps describe themselves*
//! are read from the person's settings file at the moment of each act, the same
//! file and the same way `alo-app` and the desktop read it, so what a refusal
//! says, what `alo-app` would say, and how the application was started agree —
//! and a switch turned on a moment ago counts.

use std::ffi::OsString;

use alo_adapters::{AccessibilityTree, AccessibleSession, TreeFault, WorkingFully};
use alo_turn::TheWindowsOnTheScreen;

/// The person's accessibility tree, opened for one act at a time.
///
/// **Where the person's settings are is handed in, not read here.** The two
/// values are this session's `XDG_CONFIG_HOME` and `HOME`, which
/// `crate::starting` reads once, with its reason written down beside it
/// (`tests/the_person_is_read_in_named_places.rs`). Only where to look is
/// held; the file itself is still read at each act.
#[derive(Debug, Clone, Default)]
pub struct ThePersonsWindows {
    /// `XDG_CONFIG_HOME`, as the session has it.
    config_home: Option<OsString>,
    /// `HOME`, as the session has it.
    home: Option<OsString>,
}

impl ThePersonsWindows {
    /// The person's windows, with their settings found from these two.
    #[must_use]
    pub const fn for_a_session(config_home: Option<OsString>, home: Option<OsString>) -> Self {
        Self { config_home, home }
    }
}

impl TheWindowsOnTheScreen for ThePersonsWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        AccessibleSession::of_this_person()
            .map(|session| Box::new(session) as Box<dyn AccessibilityTree>)
    }

    fn working_fully(&self, application: &str) -> WorkingFully {
        let access =
            alo_choosing::where_the_folder_is(self.config_home.as_deref(), self.home.as_deref())
                .map_or_else(alo_access::TurnedOn::nothing, |folder| {
                    alo_access::keeping::at_sign_in(&folder).0
                });
        WorkingFully::for_one(
            alo_installed::is_drawn_by_chromium(application) && access.apps_describe_themselves(),
            access.alo_works_fully_inside_apps(),
        )
    }
}
