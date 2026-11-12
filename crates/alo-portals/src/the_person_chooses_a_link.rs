//! **The person chooses what opens a link**, when an application set `ask` on
//! `OpenURI` — the *Open with* sheet the owner approved on 2026-11-12 (frame
//! `25:52`, `docs/design/default-apps-and-open-with.md`).
//!
//! # The handle now, the response when the person has answered
//!
//! Every other answer here is decided while the call is handled, so its
//! response goes out before the reply (`crate::asked`). This one waits on a
//! person, who may take a minute, so the handle is returned at once and the
//! `Response` follows on it — the specification's own shape for a dialogue.
//! The waiting is off the bus's thread ([`blocking::unblock`]), so one person
//! looking at a sheet stops no other application's request.
//!
//! # In this order
//!
//! 1. The desktop is asked to show the sheet, offering what
//!    [`LinkOpened::offered`] holds — the default first.
//! 2. A choice is held to that offer ([`LinkOpened::chosen_on_the_sheet`]); an
//!    answer naming anything else is not believed.
//! 3. **Recorded, then started**: the choice is kept naming who asked, the
//!    address and what the person chose, and only then is the desktop asked to
//!    start it, through the same `open-link` act every link takes.
//! 4. Nothing chosen is kept as [`LinkUnchosen`] with its reason, and nothing
//!    opens — **never the default instead**. Only the person's own cancel is
//!    response `1`.

use alo_applications::Link;
use zbus::zvariant::OwnedObjectPath;

use crate::answered::{Outcome, Unanswered};
use crate::asked::{Asked, REFUSED};
use crate::link_answered::LinkOpened;
use crate::link_not_chosen::{LinkUnchosen, Unchosen};
use crate::opening_links::{SheetAnswer, LinkStarted};
use crate::serving::Backend;

/// Ask the person, and answer `asked` when they have: the handle now, the
/// response later.
pub(crate) fn the_person_chooses(
    connection: &zbus::Connection,
    backend: &Backend,
    asked: Asked,
    opened: LinkOpened,
) -> OwnedObjectPath {
    let handle = asked.handle().clone();
    let backend = backend.clone();
    let bus = connection.clone();
    connection
        .executor()
        .spawn(
            async move {
                let deciding = backend.clone();
                let (asked, response) = blocking::unblock(move || {
                    let response = chosen_and_opened(&deciding, &asked, &opened);
                    (asked, response)
                })
                .await;
                // Whatever the bus says when the response cannot be sent, the
                // answer is already in the record.
                let _ = asked.responded(&bus, response).await;
            },
            "the person chooses what opens a link",
        )
        .detach();
    handle
}

/// Ask the desktop, record what the person did, start what they chose — and
/// the response it all comes to.
pub(crate) fn chosen_and_opened(backend: &Backend, asked: &Asked, opened: &LinkOpened) -> u32 {
    let not_chosen = |why| {
        let not = LinkUnchosen::of(opened.from().clone(), opened.link().clone(), why);
        asked
            .recorded(backend, Outcome::LinkUnchosen(not))
            .unwrap_or(REFUSED)
    };
    let asking = opened.from().as_str().to_owned();
    let chose = backend
        .links()
        .chosen(&asking, opened.link(), opened.offered());
    let picked = match chose {
        SheetAnswer::Picked(identifier) => match opened.chosen_on_the_sheet(&identifier) {
            Some(picked) => picked,
            // **A choice that was not offered is not believed**: the desktop
            // draws what it is sent, so this is a desktop with a fault.
            None => return not_chosen(Unchosen::CouldNotAsk),
        },
        SheetAnswer::Cancelled => return not_chosen(Unchosen::Cancelled),
        SheetAnswer::Unanswered => return not_chosen(Unchosen::Unanswered),
        SheetAnswer::CannotShow => return not_chosen(Unchosen::CannotShow),
        SheetAnswer::NoDesktop => return not_chosen(Unchosen::NoDesktop),
        SheetAnswer::CouldNotAsk => return not_chosen(Unchosen::CouldNotAsk),
    };
    let opener = picked.opens_in().to_owned();
    let link: Link = picked.link().clone();
    let Ok(response) = asked.recorded(backend, Outcome::LinkOpened(picked)) else {
        return REFUSED;
    };
    let after = match backend.links().opened(&opener, &link) {
        LinkStarted::Asked => return response,
        LinkStarted::NoDesktop => Unanswered::NoDesktop { opener },
        LinkStarted::NotInstalled | LinkStarted::CouldNotAsk => {
            Unanswered::NotOpenedALink { opener }
        }
    };
    let _ = asked.recorded(backend, Outcome::Unanswered(after));
    REFUSED
}
