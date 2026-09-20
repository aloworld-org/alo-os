//! `alo-looking-once`: the one look this machine takes on its way up.
//!
//! Started once by `alo-looking-once.service` at `multi-user.target`, as the
//! person, holding nothing. It asks, it writes two files, and it stops. See
//! [`alo_looking_once`] for where it runs and under whose privilege, and why.
//!
//! # What it prints, and to whom
//!
//! One line, to the journal, for whoever administers the machine — the shape
//! `alo-boundaryd` prints. **None of it is a sentence a person reads**: what a
//! person reads is drawn by a surface from the answer this kept, in their own
//! language, which is why nothing here is translated and nothing here takes a
//! vocabulary.
//!
//! # What it exits with
//!
//! `0` when the answer was kept, and `1` when it was not. A machine that could
//! not find out whether there is an update is a machine somebody should be able
//! to notice in `systemctl status`, and a unit that is always green about a
//! question it never answered is a unit nobody would look at twice.
//!
//! **A refused check is a failure of this unit and not of the machine.** The
//! machine is exactly as it was; the next start asks again; nothing is
//! restarted and no timer tries in a minute.

use std::process::ExitCode;
use std::time::SystemTime;

use alo_looking::{Place, SaidOnce, TheRegistry};
use alo_looking_once::{AtAStart, DidNotLook, the_road_out};
use alo_updating::TheBase;

/// What this program is called where somebody reads its lines.
const WHAT_IT_IS_CALLED: &str = "alo-looking-once";

fn main() -> ExitCode {
    match looked() {
        Ok(said) => {
            println!("{WHAT_IT_IS_CALLED}: {said}");
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!(
                "{WHAT_IT_IS_CALLED}: this machine did not find out whether there is an update: \
                 {why}{}",
                if why.nothing_left_this_machine() {
                    " — nothing left this machine"
                } else {
                    ""
                }
            );
            ExitCode::FAILURE
        }
    }
}

/// The whole of it, so that every road out is one `match` above.
fn looked() -> Result<String, DidNotLook> {
    let place = Place::on_this_machine().map_err(DidNotLook::NoPlace)?;
    let asking = TheRegistry::at(&place).taking(the_road_out(&place)?);
    let at_a_start = AtAStart::on_this_machine();
    // Made here and held for one check, which is the whole life of this
    // process: `alo_looking::SaidOnce` is deliberately never written to a disk.
    let mut said = SaidOnce::new();

    let found = at_a_start.look_once(
        &TheBase::on_this_machine(),
        &place,
        &asking,
        &mut said,
        SystemTime::now(),
    )?;

    Ok(format!(
        "asked {} and kept the answer at {}: {}",
        place.host(),
        at_a_start.answer().path().display(),
        if found.is_ready() {
            "a newer version of this machine's system is available"
        } else {
            "this machine is up to date"
        }
    ))
}
