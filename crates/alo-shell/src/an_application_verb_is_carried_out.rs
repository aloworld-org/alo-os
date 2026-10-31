//! The compositor's hand: three acts against the windows it really has.
//!
//! [`crate::the_application_door`] decides who is asking and reads the line.
//! This decides nothing at all — it carries out what it is handed and says
//! which of four things happened. ADR 0108: *the compositor is the hand rather
//! than a second judge*, and *may this run* was settled by the turn before the
//! line was written.
//!
//! # Asked, never completed — and that is the published promise
//!
//! Every answer here is about what this machine **asked for**, never about what
//! an application then did. That is not a weakening; it is what
//! `docs/contracts/agent-verbs.md` already promises a person:
//! *`close_application` asks; nothing kills anything … the application is
//! asked, it may put up its own save your changes?, and the person answers
//! that.* `Server::request_window_close`'s own documentation says the same
//! thing one layer down: *success means one XDG close event was queued … it
//! does not mean the application received it or closed.*
//!
//! So `acted` means *this machine asked*, for all three. An application that
//! declines is `it-did-not` where the compositor can tell, and where it cannot
//! the record is the person's own: a window that is still there.
//!
//! # `open` is recorded rather than sent, and that is deliberate
//!
//! `alo_portals::activated` sends `Activate` to the application's bus name and
//! the bus starts it if it is not running — the correct mechanism, and not a
//! fast one. Sent from here it would hold the frame loop until the application
//! answered, so every window on the machine would stop drawing while one
//! started. [`crate::an_application_is_started`] carries that whole argument,
//! and `open` therefore records the ask, in `Server::its_agent_asked_to_start`,
//! where the loop sends it off the input path. It is told apart from a
//! person's press so the journal can say who asked; since the owner's decision
//! of 2026-10-31 (ADR 0108) both take the same road — the bus where the entry
//! asks for it, the entry's own `Exec` line otherwise.
//!
//! **`not-installed` is a word this compositor never says**, and that is
//! honest rather than a gap. Which applications exist is
//! `alo_applications::Installed`'s answer and the compositor has no business
//! holding it (ADR 0108's *three answers, three owners*). An `open` for
//! something that is not installed is asked for, the bus finds no name to
//! start, and the person sees nothing open — which is the same outcome by a
//! road that keeps one question in one place. `alo_acting::answer::could_answer`
//! permits the word rather than requiring it, so a compositor that does know
//! may say it.
//!
//! # Several windows: the most recent, not a choice
//!
//! A person pressing a Dock icon for an application with three windows is
//! offered a list, because they are standing there to pick from it. **An agent
//! is not**, so `focus` brings forward the window the person used most recently
//! — the one they last looked at — and `close` asks **every** window of that
//! application, because *close Geary* means Geary and not one of its windows.
//! `docs/contracts/application-door.md` says both.

use alo_acting::{Acting, Answer, NotActed};
use alo_applications::Application;
use alo_dock::{AppId, WindowId};

use crate::Server;
use crate::the_application_door::CarriesOutAnApplicationVerb;
use crate::what_an_application_act_asks_for::{WhatToDo, what_an_application_act_asks_for};

impl CarriesOutAnApplicationVerb for Server {
    fn carrying_out(&mut self, acting: &Acting) -> Answer {
        let Some(app) = as_the_dock_names_it(acting.application()) else {
            // `Application` already refuses an empty identifier, so this is
            // unreachable from the door. It is an answer rather than an
            // assumption because the two types are different crates' and a
            // third one may widen one of them.
            return Answer::NotActed(NotActed::CouldNotAsk);
        };
        let asked_for =
            what_an_application_act_asks_for(&self.the_windows_the_dock_sees(), &app, acting);
        self.carry_this_out(asked_for, &app)
    }
}

impl Server {
    /// Carry out what the decision asked for.
    ///
    /// **The decision is [`crate::what_an_application_act_asks_for`]'s and the
    /// doing is here**, which is why there are two files: which window of three
    /// is arithmetic a unit test settles, and reaching it needs a real
    /// compositor and a real client.
    fn carry_this_out(&mut self, asked_for: WhatToDo, app: &AppId) -> Answer {
        match asked_for {
            // Recorded rather than sent, for the reason this file's own
            // documentation gives: `alo_portals::activated` waits for the bus
            // and this is the frame loop. **On the agent's own list**, so the
            // sender knows the agent asked and keeps it to the bus — see
            // `crate::an_application_is_started`.
            WhatToDo::StartIt => {
                self.its_agent_asked_to_start.push(app.clone());
                Answer::Acted
            }
            WhatToDo::BringThisForward(id) => self.bring_this_window_forward(id),
            WhatToDo::AskTheseToClose(these) => self.ask_these_to_close(these),
            WhatToDo::ThereIsNoWindow => Answer::NotActed(NotActed::NoWindow),
        }
    }

    /// Ask each of these windows to close.
    ///
    /// A window that cannot be asked does not stop the others: `acted` if any
    /// was asked, and `it-did-not` if none could be — which is the honest
    /// split, because there *were* windows when the list was made and *no
    /// window* would say there were not. The usual cause is a window that
    /// closed between the list and the ask, which is an ordinary race.
    fn ask_these_to_close(&mut self, these: Vec<WindowId>) -> Answer {
        let mut asked = 0_usize;
        for id in these {
            // **Looked up one at a time, not kept from the list.** Ids are the
            // window list's own indices and a window closing renumbers the
            // rest, so a surface held since the list could be a different
            // window by the time it is asked — which is `a_press_on_the_dock`'s
            // own reasoning about the same race.
            let Some(surface) = self.the_window_numbered(id).cloned() else {
                continue;
            };
            if self.request_window_close(&surface).is_ok() {
                asked += 1;
            }
        }
        if asked == 0 {
            return Answer::NotActed(NotActed::ItDidNot);
        }
        Answer::Acted
    }

    /// Reveal this window if it was put aside, then give it the keyboard.
    ///
    /// The same two steps, in the same order and for the same reason, as a
    /// press on a Dock icon: **a window that will not come back is not then
    /// focused**, because focusing a hidden window leaves a person with the
    /// keyboard somewhere they cannot see.
    fn bring_this_window_forward(&mut self, id: WindowId) -> Answer {
        let Some(surface) = self.the_window_numbered(id).cloned() else {
            // It closed between the decision and now. An ordinary race, and
            // from the caller's side there is no window to focus.
            return Answer::NotActed(NotActed::NoWindow);
        };
        if self
            .the_windows_the_dock_sees()
            .window(id)
            .is_some_and(|window| window.sits().was_put_aside())
            && self.set_window_minimized(&surface, false).is_err()
        {
            return Answer::NotActed(NotActed::ItDidNot);
        }
        if self.activate_window(&surface).is_err() {
            return Answer::NotActed(NotActed::ItDidNot);
        }
        Answer::Acted
    }
}

/// The capability model's identifier, as the Dock's own type.
///
/// **Two types for one thing, converted at this seam and nowhere else.**
/// `alo_applications::Application` is what a verb is granted and approved
/// against; `alo_dock::AppId` is what this machine's windows are labelled with.
/// They are different crates' answers to different questions and neither should
/// become the other's — so the one place a line from the door meets a window on
/// a screen does the conversion, in one function, where it can be read.
///
/// It cannot fail in practice: `Application::identified` already refuses an
/// empty or blank identifier, which is the only thing `AppId::named` refuses.
/// The [`Option`] is carried rather than assumed away because that is a fact
/// about two crates agreeing today, not a guarantee either of them makes.
fn as_the_dock_names_it(application: &Application) -> Option<AppId> {
    AppId::named(application.identifier()).ok()
}
