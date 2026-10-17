//! That a person's ask reaches the machine, shows while it does, and comes
//! back to the window.
//!
//! **These drive the real road.** The tool is a list in memory — whether
//! `/usr/bin/flatpak` can be driven is `alo-software`'s question and is tested
//! there — and everything between the ask and the answer is production code,
//! including the thread. A test that called `alo_software::install` itself
//! would pass with the hand-over deleted, which is the fault
//! `crate::a_key_reaches_settings` records about its own road.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use alo_applications::Application;
use alo_software::{Catalogue, Configured, Failed, SourceName, Tool};

use super::*;
use crate::MarketWindow;

/// How long a test will wait for the installing thread.
///
/// A thread that posts nothing is the failure being reported, so this is a
/// deadline rather than a sleep: **a test that hangs is not a test that
/// fails**, which is what the terminal's own `Drop` cost this repository on
/// 2026-10-16.
const PATIENCE: Duration = Duration::from_secs(20);

/// A fixed moment, so nothing here depends on when it runs.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A machine with one place set up, which remembers what it installed and can
/// be told to refuse.
struct OnePlace {
    /// Every application it was asked to install, in order.
    installed: Mutex<Vec<String>>,
    /// Told when `install` is called, so a test can wait for the thread to
    /// have reached the tool rather than guessing.
    reached: Mutex<Option<Sender<String>>>,
    /// What `install` answers with.
    answers: Mutex<Result<(), String>>,
    /// Whether the place checks signatures, which decides whether it is usable.
    checks_signatures: bool,
}

impl OnePlace {
    /// A place that works, with nothing installed on it.
    fn working() -> Self {
        Self {
            installed: Mutex::new(Vec::new()),
            reached: Mutex::new(None),
            answers: Mutex::new(Ok(())),
            checks_signatures: true,
        }
    }

    /// A place set up without signature checking, which `alo-software` refuses
    /// before anything leaves.
    fn unverified() -> Self {
        Self {
            checks_signatures: false,
            ..Self::working()
        }
    }

    /// A place whose tool refuses the install itself, after the line is up.
    fn refusing() -> Self {
        Self {
            answers: Mutex::new(Err("the tool would not".to_owned())),
            ..Self::working()
        }
    }

    /// Tell this tool where to say it was reached.
    fn telling(self, where_to: Sender<String>) -> Self {
        *self
            .reached
            .lock()
            .expect("nothing panics holding this lock") = Some(where_to);
        self
    }
}

impl Tool for OnePlace {
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Ok(vec![Configured {
            name: "flathub".to_owned(),
            address: "https://dl.flathub.org/repo/".to_owned(),
            checks_signatures: self.checks_signatures,
            switched_off: false,
        }])
    }

    fn installed(&self) -> Result<Vec<String>, Failed> {
        Ok(self
            .installed
            .lock()
            .expect("nothing panics holding this lock")
            .clone())
    }

    fn open(&self) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
    }

    fn install(&self, _: &SourceName, application: &Application) -> Result<(), Failed> {
        if let Some(telling) = self
            .reached
            .lock()
            .expect("nothing panics holding this lock")
            .as_ref()
        {
            drop(telling.send(application.identifier().to_owned()));
        }
        let answer = self
            .answers
            .lock()
            .expect("nothing panics holding this lock")
            .clone();
        match answer {
            Ok(()) => {
                self.installed
                    .lock()
                    .expect("nothing panics holding this lock")
                    .push(application.identifier().to_owned());
                Ok(())
            }
            Err(said) => Err(Failed::DidNotAnswer { said }),
        }
    }

    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
    }

    fn update(&self, _: &Application) -> Result<(), Failed> {
        Ok(())
    }

    fn remove(&self, _: &Application) -> Result<(), Failed> {
        Ok(())
    }
}

/// The first application in the decided catalogue, which is what a person's
/// window opens on.
fn the_first_one(catalogue: &Catalogue) -> Listed {
    catalogue
        .every()
        .first()
        .expect("the catalogue holds something")
        .clone()
}

/// Wait for the answer, or say the thread never posted one.
fn whatever_came(
    road: &TheInstallingThatIsAsked,
    indicator: &mut Indicator,
) -> Vec<WhatBecameOfIt> {
    let until = Instant::now() + PATIENCE;
    loop {
        let came = road.whatever_became_of_them(indicator);
        if !came.is_empty() {
            return came;
        }
        assert!(
            Instant::now() < until,
            "the installing thread posted nothing in {PATIENCE:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Wait for a message, or say it never came.
fn heard(reached: &Receiver<String>) -> String {
    reached
        .recv_timeout(PATIENCE)
        .expect("the installing thread never reached the tool")
}

/// **The whole road, in the order a person experiences it**: they ask, the line
/// goes up, the machine fetches, the line comes down, the window is told.
///
/// This is the test `docs/autonomy`' market plan asks for by name — *one test
/// drives ask → errand shown → installed → arrival* — and every step between
/// the first line and the last is production code.
#[test]
fn an_ask_is_shown_while_it_happens_and_the_window_is_told_it_arrived() {
    let (telling, reached) = channel();
    let tool = Arc::new(OnePlace::working().telling(telling));
    let road = TheInstallingThatIsAsked::with(Arc::clone(&tool) as Arc<_>, Bound::Nobodys);
    let catalogue = Catalogue::decided().expect("the catalogue this machine ships");
    let one = the_first_one(&catalogue);
    let mut indicator = Indicator::default();

    // Nothing is leaving before anybody asks for anything.
    assert!(indicator.is_quiet());

    road.asked_for(&one, &mut indicator, noon())
        .expect("a working place installs");

    // **The line is up, and it is up before the tool is reached.** The thread
    // waits for the authority, so by the time `install` is called the
    // indicator already says what is happening — and this asserts the order
    // rather than the end state: the indicator is read first, then the tool's
    // own report of having been reached is waited for.
    assert!(
        !indicator.is_quiet(),
        "the machine is fetching an application and the indicator says nothing is leaving"
    );
    assert_eq!(indicator.showing().len(), 1);
    assert_eq!(heard(&reached), one.application().identifier());

    let came = whatever_came(&road, &mut indicator);

    // **And the line comes down.** A line left lit would tell a person
    // something is leaving for the rest of their session.
    assert!(
        indicator.is_quiet(),
        "the installation finished and the indicator is still lit"
    );
    assert_eq!(came.len(), 1);
    let what = came.first().expect("one answer");
    assert_eq!(what.identifier, one.application().identifier());
    let installed = what.outcome.as_ref().expect("it arrived");
    assert_eq!(
        installed.application().identifier(),
        one.application().identifier()
    );
    assert_eq!(
        *tool.installed.lock().expect("no panic"),
        [one.application().identifier()],
        "the tool was reached once, with what was asked for"
    );
}

/// **The window learns it arrived**, which is the half a person sees.
///
/// Driven through `MarketWindow` rather than asserted on the answer alone,
/// because the window is what records the ask and what draws the row — and
/// `it_arrived` had no production caller until this road existed.
#[test]
fn the_window_that_asked_is_the_window_that_is_told() {
    let tool = Arc::new(OnePlace::working());
    let road = TheInstallingThatIsAsked::with(Arc::clone(&tool) as Arc<_>, Bound::Nobodys);
    let catalogue = Catalogue::decided().expect("the catalogue this machine ships");
    let mut window = MarketWindow::closed();
    window.opened(
        Catalogue::decided().expect("the catalogue this machine ships"),
        Vec::new(),
    );
    let mut indicator = Indicator::default();

    // The person presses Install on the row they are on.
    assert_eq!(
        window.pressed(crate::MarketKey::Install),
        crate::MarketPressed::AskedToInstall
    );
    let asked = window
        .what_they_asked_for()
        .expect("the window recorded what was asked for")
        .clone();
    assert_eq!(
        asked.application().identifier(),
        the_first_one(&catalogue).application().identifier(),
        "the window asked for something other than the row it was on"
    );

    road.asked_for(&asked, &mut indicator, noon())
        .expect("a working place installs");
    for what in whatever_came(&road, &mut indicator) {
        match what.outcome {
            Ok(installed) => window.it_arrived(installed.application().identifier()),
            Err(why) => window.it_was_refused(&what.identifier, why),
        }
    }

    // **The row says it is installed, and nothing says it is still going.**
    let crate::MarketShows::Browsing(browsing) = window.shows() else {
        unreachable!("an open window is browsing")
    };
    assert!(
        matches!(browsing.how_it_is_going(&asked), crate::HowItIsGoing::Here),
        "the window was told it arrived and does not say so"
    );
    assert!(
        window.what_they_asked_for().is_none(),
        "the window still has an ask standing after it was answered"
    );
}

/// **A refusal before anything leaves never lights the indicator**, which is
/// `alo-software`'s own promise and the half of law 1 that is easy to get
/// wrong: putting the line up first and taking it down on a refusal would be
/// an indicator that flickers for things that never happened.
#[test]
fn a_refusal_before_anything_leaves_leaves_the_indicator_quiet() {
    let tool = Arc::new(OnePlace::unverified());
    let road = TheInstallingThatIsAsked::with(tool as Arc<_>, Bound::Nobodys);
    let catalogue = Catalogue::decided().expect("the catalogue this machine ships");
    let one = the_first_one(&catalogue);
    let mut indicator = Indicator::default();

    let refused = road
        .asked_for(&one, &mut indicator, noon())
        .expect_err("a place with signature checking off is refused");

    assert!(
        indicator.is_quiet(),
        "a refusal that never reached the network lit the indicator: {refused:?}"
    );
    assert!(
        matches!(refused, Stopped::Refused(NotDone::NotVerified { .. })),
        "{refused:?}"
    );
    assert!(
        road.whatever_became_of_them(&mut indicator).is_empty(),
        "a refused ask posted an answer"
    );
}

/// **A refusal from the tool itself still takes the line down**, because by
/// then something really was leaving and the person watched it.
#[test]
fn a_refusal_from_the_tool_still_takes_the_line_down() {
    let tool = Arc::new(OnePlace::refusing());
    let road = TheInstallingThatIsAsked::with(tool as Arc<_>, Bound::Nobodys);
    let catalogue = Catalogue::decided().expect("the catalogue this machine ships");
    let one = the_first_one(&catalogue);
    let mut indicator = Indicator::default();

    road.asked_for(&one, &mut indicator, noon())
        .expect("the ask reached the tool");
    assert!(!indicator.is_quiet());

    let came = whatever_came(&road, &mut indicator);

    assert!(
        indicator.is_quiet(),
        "the install was refused and the indicator is still lit"
    );
    assert_eq!(came.len(), 1);
    let what = came.first().expect("one answer");
    assert!(
        what.outcome.is_err(),
        "the tool refused and the answer says it arrived"
    );
    assert_eq!(what.identifier, one.application().identifier());
}

/// **Something already here is refused before anything leaves**, so a person
/// who asks for what they have is told rather than made to wait for a download
/// of it.
///
/// This is also the answer to the one thing `what_can_be_installed` cannot tell
/// an agent — see `alo_software::what_the_catalogue_holds` — so it is asserted
/// here rather than left as prose there.
#[test]
fn something_already_installed_is_refused_with_nothing_leaving() {
    let tool = Arc::new(OnePlace::working());
    let catalogue = Catalogue::decided().expect("the catalogue this machine ships");
    let one = the_first_one(&catalogue);
    tool.installed
        .lock()
        .expect("no panic")
        .push(one.application().identifier().to_owned());
    let road = TheInstallingThatIsAsked::with(tool as Arc<_>, Bound::Nobodys);
    let mut indicator = Indicator::default();

    let refused = road
        .asked_for(&one, &mut indicator, noon())
        .expect_err("an application already here is refused");

    assert!(indicator.is_quiet(), "{refused:?}");
    assert!(
        matches!(refused, Stopped::Refused(NotDone::AlreadyInstalled { .. })),
        "{refused:?}"
    );
}

/// **Two asks are two lines and two answers**, each taking off its own.
///
/// The window refuses a second ask while one is going, so this is not a state a
/// person can reach — it is asserted because the mechanism must not depend on
/// that rule being kept somewhere else. An answer that took off the wrong line
/// would leave an indicator lit with nothing leaving, and a map from identifier
/// to line is exactly how that happens.
#[test]
fn two_installations_each_take_off_their_own_line() {
    let tool = Arc::new(OnePlace::working());
    let road = TheInstallingThatIsAsked::with(tool as Arc<_>, Bound::Nobodys);
    let catalogue = Catalogue::decided().expect("the catalogue this machine ships");
    let mut every = catalogue.every().iter();
    let first = every.next().expect("one").clone();
    let second = every.next().expect("two").clone();
    let mut indicator = Indicator::default();

    road.asked_for(&first, &mut indicator, noon())
        .expect("the first");
    road.asked_for(&second, &mut indicator, noon())
        .expect("the second");
    assert_eq!(indicator.showing().len(), 2);

    let until = Instant::now() + PATIENCE;
    let mut answered = Vec::new();
    while answered.len() < 2 {
        answered.extend(road.whatever_became_of_them(&mut indicator));
        assert!(
            Instant::now() < until,
            "only {} of two installations answered",
            answered.len()
        );
        std::thread::sleep(Duration::from_millis(10));
    }

    assert!(
        indicator.is_quiet(),
        "both installations answered and {} lines are still lit",
        indicator.showing().len()
    );
    let mut named: Vec<&str> = answered
        .iter()
        .map(|what| what.identifier.as_str())
        .collect();
    named.sort_unstable();
    let mut wanted = [
        first.application().identifier(),
        second.application().identifier(),
    ];
    wanted.sort_unstable();
    assert_eq!(named, wanted);
}
