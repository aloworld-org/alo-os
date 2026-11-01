//! Folders put back as they were, at the person's request — the one road for
//! both of the person's ways of asking: *Put back* in the record window, and
//! `alo-back` in their terminal.
//!
//! # Why here
//!
//! The owner's answer of 2026-11-01: putting back **is written in this list
//! too**, and it is written by the service that writes the record — the
//! record has one writer (`docs/contracts/record-file.md`), and the window
//! that shows it writes nothing. This service runs as the person
//! (`image/usr/lib/systemd/system/alo-agentd.service`, `User=alo`), so it does
//! to their folders exactly what `alo-back` does from their own shell, with no
//! privilege either has not got.
//!
//! # What it does
//!
//! Exactly what `alo-back` does, for every folder asked for, **together or
//! not at all** (the owner's second answer): each folder is planned with
//! `alo_as_it_was::what_would_go_back` against the named place, and the plans
//! are carried out by `alo_as_it_was::go_back_together`. What is in each
//! folder now is renamed beside it, never deleted.
//!
//! # What it writes
//!
//! One `put-back` entry, whether it worked or not, naming the folders, the
//! moment the place was kept and the sentence it was kept for — copied off
//! the place's note. A place that is not among this person's is refused
//! without an entry: there is no moment and no sentence to write, and nothing
//! was touched.
//!
//! # Not an agent's
//!
//! There is no verb that puts back (ADR 0045's second term), and this request
//! is refused on an agent's door by `alo-protocol` before it reaches here.

use std::path::Path;
use std::time::SystemTime;

use alo_as_it_was::{Places, words};
use alo_keeping::NotKept;
use alo_protocol::{ToAPerson, Wording};
use alo_strings::{Filling, Strings};

use crate::holding::Holding;
use crate::the_bracket_this_turn_takes::WhereBracketsGo;

/// Put `folders` back from the place named `place`, write it down, and say
/// what happened.
///
/// # Errors
/// [`NotKept`] when the record could not be written — the service stops, as
/// it does for every entry it cannot keep. The folders were put back by then;
/// what failed is the record of it.
pub fn put_back(
    place: &str,
    folders: &[String],
    holding: &mut Holding<'_, '_, '_>,
    strings: &Strings,
    now: SystemTime,
) -> Result<ToAPerson, NotKept> {
    let said = |word: alo_strings::Word, filling: &Filling| strings.say(&word.key(), filling);
    let no_such_place = || {
        ToAPerson::refused(&said(
            words::NO_SUCH_PLACE,
            &Filling::of("named", place.to_owned()),
        ))
    };
    let Some(goes) = WhereBracketsGo::for_this_session() else {
        return Ok(no_such_place());
    };
    let places = Places::under(goes.whose());
    let Some(kept) = places.named(place) else {
        return Ok(no_such_place());
    };
    let as_kept = (kept.taken(), kept.did());
    match the_act(
        &kept.before(),
        folders,
        goes.home(),
        (kept.taken(), now),
        strings,
    ) {
        Ok(went) => {
            holding.folders_were_put_back(as_kept, &went.folders, None, now)?;
            // **Every folder went back**; where one's old contents could not be
            // given their final name, that is what the person is told, since
            // it is where to find them.
            let told = match went.half {
                Some(half) => vec![Wording::of(&half)],
                None => went
                    .asides
                    .iter()
                    .zip(&went.folders)
                    .map(|(aside, folder)| {
                        Wording::of(&said(
                            words::IT_IS_AS_IT_WAS,
                            &Filling::of("folder", folder.clone()).and("aside", aside.clone()),
                        ))
                    })
                    .collect(),
            };
            Ok(ToAPerson::PutBack { said: told })
        }
        Err(why) => {
            let refusal = why.said(strings);
            holding.folders_were_put_back(as_kept, folders, Some(refusal.text()), now)?;
            Ok(ToAPerson::refused(&refusal))
        }
    }
}

/// What went back: each folder, and where what was in it now is.
struct WentBack {
    /// The folders, as they resolve.
    folders: Vec<String>,
    /// Where what was in each is now, in the same order.
    asides: Vec<String>,
    /// Every folder went back and one's old contents kept their working
    /// name: the sentence saying where they are.
    half: Option<alo_strings::Said>,
}

/// Plan every folder, then put them all back together.
fn the_act(
    before: &Path,
    folders: &[String],
    home: &Path,
    (taken, now): (SystemTime, SystemTime),
    strings: &Strings,
) -> Result<WentBack, alo_as_it_was::NotGoingBack> {
    let mut plans = Vec::new();
    for folder in folders {
        plans.push(alo_as_it_was::what_would_go_back(
            before,
            Path::new(folder),
            home,
            taken,
            now,
        )?);
    }
    let half = match alo_as_it_was::go_back_together(&plans) {
        Ok(()) => None,
        Err(none) if none.why.the_folder_went_back() => Some(none.why.said(strings)),
        Err(none) => return Err(none.why),
    };
    Ok(WentBack {
        folders: plans
            .iter()
            .map(|plan| plan.folder().display().to_string())
            .collect(),
        asides: plans
            .iter()
            .map(|plan| plan.what_was_here().display().to_string())
            .collect(),
        half,
    })
}
