//! One drag, from the moment it is picked up to the moment it is let go.
//!
//! # A drag is a selection that a person is aiming
//!
//! It carries an `alo_clipboard::Offer` — the forms the source can produce, and
//! whether letting go copies or moves — and the way back to the window that has
//! the data. Nothing is read from that window until somebody lets go, which is
//! why dragging a hundred megabytes of image across four windows costs nothing.
//!
//! The plan's acceptance is that *a drop carries what copy and paste carries,
//! through `alo-clipboard`'s payload types rather than a second set*, and the
//! way that is kept true is that there is no type here to carry a payload in.
//! Text, images, files and the fourth thing nobody has written yet are one
//! mechanism, exactly as they are on the clipboard.
//!
//! # It happens once, and the compiler is what says so
//!
//! [`Drag::let_go_on`] takes `self`. A drag that could be let go twice would be
//! a move that happened twice — the failure `alo_clipboard::Clipboard` retires a
//! cut offer to prevent — and here it is not a rule that is checked but a
//! program that does not compile. Abandoning a drag is dropping it, and a
//! dropped drag has told nobody anything.
//!
//! # Nothing here draws, and nothing here is the agent's
//!
//! There is no pointer in this crate, no surface and no cursor. What a person
//! sees while they drag is [`crate::Over`], which is a sentence and a form; the
//! compositor draws it.
//!
//! And no agent starts a drag. Every drag begins with a hand — ADR 0009's rule
//! is that a person can do anything a verb can, and it is not the other way
//! round. There is no verb here, no `alo-capability` dependency to declare one
//! with, and a file that reaches the agent reaches it because a person aimed at
//! its surface and let go.

use alo_clipboard::{Form, Gives, Kind, Offer, Taking};

use crate::delivery::Delivery;
use crate::for_one_turn::ForOneTurn;
use crate::handed::Handed;
use crate::over::Over;
use crate::refusing::NotHanded;
use crate::target::Target;
use crate::uri_list;

/// Something a person has picked up and is carrying across the screen.
pub struct Drag {
    /// What the window it came from says it can give, and how it is being
    /// taken.
    offer: Offer,
    /// The way back to that window. Asked once, when the person lets go.
    source: Box<dyn Gives>,
}

impl std::fmt::Debug for Drag {
    /// What is being carried, without the window it came from — which has no
    /// `Debug` — and **without anything that was dragged**, which is not here.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Drag")
            .field("offer", &self.offer)
            .finish_non_exhaustive()
    }
}

impl Drag {
    /// A person picks something up: these forms, and this way back to the
    /// window that has it.
    #[must_use]
    pub fn begun(offer: Offer, source: Box<dyn Gives>) -> Self {
        Self { offer, source }
    }

    /// Every form what is being dragged can be given in, in the order the
    /// window that has it preferred them.
    #[must_use]
    pub fn forms(&self) -> &[Kind] {
        self.offer.forms()
    }

    /// Whether letting go copies what is being dragged or moves it.
    #[must_use]
    pub fn taking(&self) -> Taking {
        self.offer.taking()
    }

    /// What letting go here would do — the answer the pointer shows, with
    /// nothing read and nothing changed.
    #[must_use]
    pub fn over(&self, target: &Target) -> Over {
        match target {
            Target::Nothing => Over::NothingHere,
            // An `Offer` cannot be empty — `alo_clipboard::OfferError::NothingOffered`
            // is what an empty one is — so the first form is always there, and
            // the other arm is what this crate does instead of an `unwrap`.
            Target::TheAgentsSurface { .. } => match self.offer.forms().first() {
                Some(form) => Over::WouldOfferItForThisQuestion { form: form.clone() },
                None => Over::WouldNotTakeIt,
            },
            Target::AWindow(application) => match self.form_for(application.accepts()) {
                None => Over::WouldNotTakeIt,
                Some(form) => Over::WouldHandItOver {
                    form: form.clone(),
                    taking: self.taking(),
                },
            },
        }
    }

    /// The person lets go here.
    ///
    /// Takes `self`: one drag is one drop.
    ///
    /// # Errors
    /// [`NotHanded`] — there is nothing here that takes a drop, this window
    /// takes none of the forms on offer, the window that had it could not hand
    /// it over, or what it handed over as a list of files is not one. **Every
    /// one of them hands nothing to anybody**, and the window it came from
    /// keeps what it has, move or not.
    pub fn let_go_on(mut self, target: &Target) -> Result<LetGo, NotHanded> {
        match target {
            Target::Nothing => Err(NotHanded::NothingHere),
            Target::TheAgentsSurface { the_question_ends } => Ok(LetGo::OfferedForThisQuestion(
                ForOneTurn::of(self.offer.forms().to_vec(), self.source, *the_question_ends),
            )),
            Target::AWindow(application) => {
                let Some(form) = self.form_for(application.accepts()).cloned() else {
                    return Err(NotHanded::WillNotTakeIt);
                };
                let bytes = self
                    .source
                    .give(&form)
                    .map_err(NotHanded::DidNotHandItOver)?;
                let delivery = delivery_of(bytes, &form, application.is_sandboxed())?;
                Ok(LetGo::Handed(Handed::of(
                    application.id(),
                    form,
                    self.offer.taking(),
                    delivery,
                )))
            }
        }
    }

    /// The form to hand over, given what the target takes: the **source's**
    /// order of preference, for `alo_clipboard::Offered::best_of`'s reason —
    /// the window that has the data is the one that knows what each form loses.
    fn form_for<'a>(&'a self, accepted: &[Kind]) -> Option<&'a Kind> {
        self.offer
            .forms()
            .iter()
            .find(|form| accepted.contains(form))
    }
}

/// How what was handed over reaches the window: straight over, or through the
/// documents portal.
///
/// Files to a **sandboxed** window are the one case that is not straight over,
/// and `delivery.rs` is why: a path means nothing inside a sandbox, so what
/// crosses is an export of each file to that application and never a location
/// on the person's disk. A list with no file in it — a link dragged out of a
/// browser — has nothing to export and goes straight over.
fn delivery_of(bytes: Vec<u8>, form: &Kind, sandboxed: bool) -> Result<Delivery, NotHanded> {
    if !sandboxed || form.form() != Form::Files {
        return Ok(Delivery::OnTheSpot(bytes));
    }
    let files = uri_list::files_in(&bytes).map_err(|why| NotHanded::NotAFileList { why })?;
    if files.is_empty() {
        return Ok(Delivery::OnTheSpot(bytes));
    }
    Ok(Delivery::ThroughTheDocuments { files })
}

/// What letting go did.
#[derive(Debug)]
pub enum LetGo {
    /// A window received it.
    Handed(Handed),
    /// The agent's surface was offered it, for this question only, and **was
    /// granted nothing**.
    OfferedForThisQuestion(ForOneTurn),
}

impl LetGo {
    /// The drop, when a window received it.
    #[must_use]
    pub fn handed(&self) -> Option<&Handed> {
        match self {
            Self::Handed(handed) => Some(handed),
            Self::OfferedForThisQuestion(_) => None,
        }
    }

    /// What the agent was offered, when it was the agent's surface.
    #[must_use]
    pub fn offered(self) -> Option<ForOneTurn> {
        match self {
            Self::Handed(_) => None,
            Self::OfferedForThisQuestion(offered) => Some(offered),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::target::the_agents_surface;
    use crate::testing::{Asked, an_application, at, copied, cut, nothing_to_give, takes};

    /// The pointer answers before anything is read: what letting go would do,
    /// over four different things, with the window that has the data never
    /// woken up.
    #[test]
    fn what_letting_go_would_do_is_answered_without_reading_anything() {
        let asked = Asked::nothing_yet();
        let drag = Drag::begun(
            copied(&[Kind::files(), Kind::text()]),
            an_application(&asked),
        );

        assert_eq!(
            drag.over(&takes("org.alo.Notes", &[Kind::text()])),
            Over::WouldHandItOver {
                form: Kind::text(),
                taking: Taking::Copied,
            }
        );
        assert_eq!(
            drag.over(&takes("org.alo.Paint", &[Kind::image_png()])),
            Over::WouldNotTakeIt
        );
        assert_eq!(
            drag.over(&the_agents_surface(at(200))),
            Over::WouldOfferItForThisQuestion {
                form: Kind::files()
            }
        );
        assert_eq!(drag.over(&Target::Nothing), Over::NothingHere);
        assert_eq!(asked.how_many(), 0);
    }

    /// The form handed over is the **source's** first that the target takes,
    /// not the target's — the window that has the data knows what each form
    /// loses.
    #[test]
    fn the_form_is_the_sources_preference_among_what_the_target_takes() {
        let html = Kind::named("text/html").expect("a media type");
        let asked = Asked::nothing_yet();
        let drag = Drag::begun(
            copied(&[html.clone(), Kind::text()]),
            an_application(&asked),
        );

        let handed = drag
            .let_go_on(&takes("org.alo.Notes", &[Kind::text(), html.clone()]))
            .expect("a window that takes both");
        let handed = handed.handed().expect("a window received it").clone();
        assert_eq!(handed.form(), &html);
        assert_eq!(asked.forms(), [html]);
    }

    /// Letting go over nothing, and over a window that takes none of the forms:
    /// **nothing is handed to anybody and nobody is asked for a byte.**
    #[test]
    fn a_drop_that_does_not_happen_asks_the_window_for_nothing() {
        let asked = Asked::nothing_yet();
        let drag = Drag::begun(copied(&[Kind::text()]), an_application(&asked));
        assert_eq!(
            drag.let_go_on(&Target::Nothing).unwrap_err(),
            NotHanded::NothingHere
        );
        assert_eq!(asked.how_many(), 0);

        let asked = Asked::nothing_yet();
        let drag = Drag::begun(copied(&[Kind::text()]), an_application(&asked));
        assert_eq!(
            drag.let_go_on(&takes("org.alo.Paint", &[Kind::image_png()]))
                .unwrap_err(),
            NotHanded::WillNotTakeIt
        );
        assert_eq!(asked.how_many(), 0);
    }

    /// The window that had it quit between the drag and the drop.
    #[test]
    fn a_window_that_cannot_hand_it_over_hands_nothing_over() {
        let drag = Drag::begun(copied(&[Kind::text()]), nothing_to_give());
        assert!(matches!(
            drag.let_go_on(&takes("org.alo.Notes", &[Kind::text()])),
            Err(NotHanded::DidNotHandItOver(_))
        ));
    }

    /// A move is a move: what the drag says about copying or cutting is what
    /// the drop says, so the window it came from knows whether to let go of its
    /// own copy.
    #[test]
    fn a_dragged_move_arrives_as_a_move() {
        let asked = Asked::nothing_yet();
        let drag = Drag::begun(cut(&[Kind::text()]), an_application(&asked));
        assert_eq!(drag.taking(), Taking::Cut);
        assert_eq!(drag.forms(), [Kind::text()]);

        let let_go = drag
            .let_go_on(&takes("org.alo.Notes", &[Kind::text()]))
            .expect("a window that takes text");
        assert_eq!(
            let_go.handed().expect("a window received it").taking(),
            Taking::Cut
        );
    }

    /// What is printed about a drag is the forms, and never what is being
    /// dragged — which is not held here to print.
    #[test]
    fn nothing_printed_about_a_drag_is_the_persons_own() {
        let asked = Asked::nothing_yet();
        let drag = Drag::begun(copied(&[Kind::text()]), an_application(&asked));
        let printed = format!("{drag:?}");
        assert!(printed.contains("text/plain"), "{printed}");
        assert!(!printed.contains("what was dragged"), "{printed}");
    }
}
