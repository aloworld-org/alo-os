//! Which installed application fills each of the Dock's default roles.
//!
//! `alo_dock::pinning::Discovery` is the shape of this answer and has been
//! since the roles were written down. **Nothing produced one.** Its own header
//! says why it could not: *whoever reads desktop entries off a Linux machine
//! fills it; this crate could not and should not.* `alo-software` learned to
//! read the machine, and `alo-dock` holds the roles, and **only a binary sees
//! both** — `alo-saying` is the one other crate that does and it is the
//! translation collector. So the join lives here, beside the process it is
//! for, exactly as the launch sender does and for the same reason.
//!
//! # Two of three roles, and the third is named rather than faked
//!
//! `ARole::Settings` opens the compositor's own surface. The owner's ruling of
//! 2026-10-12 forbids inventing an identifier or a bus endpoint for it, and
//! `Discovery::Completed` carries `(ARole, AppId)` — an identifier. So Settings
//! is **deliberately absent from what this produces** and reaches the Dock as a
//! native-action entry, which is separate work. A reader who adds
//! `(ARole::Settings, AppId::named("…"))` here would be inventing the thing the
//! ruling forbids, which is why this paragraph is longer than the code.
//!
//! # A failure is reported as a failure
//!
//! Two things can go wrong and neither yields an empty list.
//! `alo_software::what_this_machine_has` returns no list at all when the tool
//! would not answer, and `Shipped::decided` can fail to parse the built-in
//! list. Both become [`Discovery::Failed`], because
//! `alo_dock::pinning::ThePins::what_to_pin` then answers `NotYet` and **writes
//! nothing down** — which is the owner's fourth clause: *an unfinished scan or
//! discovery failure is not proof that an application is absent.*
//!
//! A scan that succeeded and found no browser is a different fact: that is
//! `Completed` with the browser missing, a Dock with the roles it can fill, and
//! it is written down.
//!
//! # `StillLooking` is not produced here
//!
//! This runs once, before the desktop stands up, and answers. There is no
//! partial state to report. If the scan ever becomes asynchronous, that variant
//! is waiting for it.

use alo_dock::pinning::{ARole, Discovery};
use alo_dock::window::AppId;
use alo_software::rented::TheRentedTool;
use alo_software::{Role, Shipped};

/// The Dock roles this machine can fill from what it has installed, in
/// [`ARole::EVERY`]'s order.
///
/// Reads the machine once, through the rented tool, and matches what it found
/// against the applications this system ships for each role.
pub fn what_this_machine_offers() -> Discovery {
    let tool = TheRentedTool::on_this_machine();
    let Ok(has) = alo_software::what_this_machine_has::what_this_machine_has(&tool) else {
        return Discovery::Failed;
    };
    let Ok(ships) = Shipped::decided() else {
        return Discovery::Failed;
    };
    Discovery::Completed(filled_by(&has, &ships))
}

/// The pairs, from one reading and one shipped list.
///
/// **Separated from the two readings so it can be tested at all.** The function
/// above asks a real machine for a real tool's output; this one is given both
/// and is where every rule lives — the order, the two roles, and the omission
/// of anything not installed.
fn filled_by(
    has: &alo_software::what_this_machine_has::WhatThisMachineHas,
    ships: &Shipped,
) -> Vec<(ARole, AppId)> {
    let mut filled = Vec::new();
    // **In `ARole::EVERY`'s order**, skipping the role no scan can answer.
    // Written as a match over every role rather than a list of the two, so that
    // a role added to `ARole` is a compile error here rather than a Dock slot
    // that silently never fills.
    for role in ARole::EVERY {
        let wanted = match role {
            ARole::Files => Role::FileManager,
            ARole::Browser => Role::WebBrowser,
            // Not an application. See this module's header.
            ARole::Settings => continue,
        };
        let identifier = ships.the(wanted).application().identifier();
        if !has.installed().has(identifier) {
            // **Omitted and not placeholdered**, which is the owner's words:
            // *omit unavailable applications; never show placeholders or
            // non-working buttons.*
            continue;
        }
        let Ok(app) = AppId::named(identifier) else {
            // An identifier this machine shipped that the Dock will not take is
            // a packaging fault, not a reason to pin something else.
            continue;
        };
        filled.push((role, app));
    }
    filled
}

#[cfg(test)]
#[path = "what_this_machine_offers_tests.rs"]
mod tests;
