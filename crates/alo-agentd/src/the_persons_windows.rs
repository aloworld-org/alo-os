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

use alo_adapters::{AccessibilityTree, AccessibleSession, TreeFault, WorkingFully};
use alo_turn::TheWindowsOnTheScreen;

/// The person's accessibility tree, opened for one act at a time.
#[derive(Debug, Clone, Copy, Default)]
pub struct ThePersonsWindows;

impl TheWindowsOnTheScreen for ThePersonsWindows {
    fn opening(&mut self) -> Result<Box<dyn AccessibilityTree>, TreeFault> {
        AccessibleSession::of_this_person()
            .map(|session| Box::new(session) as Box<dyn AccessibilityTree>)
    }

    fn working_fully(&self, application: &str) -> WorkingFully {
        let access = alo_choosing::where_the_folder_is(
            std::env::var_os(alo_choosing::CONFIG_HOME).as_deref(),
            std::env::var_os(alo_choosing::HOME).as_deref(),
        )
        .map_or_else(alo_access::TurnedOn::nothing, |folder| {
            alo_access::keeping::at_sign_in(&folder).0
        });
        WorkingFully::for_one(
            alo_launching::is_drawn_by_chromium(application) && access.apps_describe_themselves(),
            access.alo_works_fully_inside_apps(),
        )
    }
}
