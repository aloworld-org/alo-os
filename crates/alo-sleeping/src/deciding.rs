//! Whether the machine sleeps, decided before anything reaches `logind`.
//!
//! [`asked`] is the one question, and [`Why`] is the closed list of reasons it
//! is asked. The answer is a [`Decided`]: a [`Going`] — which can only be
//! carried out by locking the seat first (`crate::going`) — or a
//! [`StaysAwake`], which says why in a sentence per reason.
//!
//! # Who may hold a machine awake against whom
//!
//! | Asked because | The lid rule | A keeper |
//! |---|---|---|
//! | [`Why::LidClosed`] | decides | named, and does not hold the lid open |
//! | [`Why::YouAsked`] | — | named, and does not hold the person's sleep off |
//! | [`Why::NobodyIsUsingIt`] | — | **holds the machine awake**, named |
//!
//! A keeper stops the machine sleeping **on its own**, and never stops a sleep
//! the person asked for by closing the lid or choosing Sleep: an application
//! that could refuse a person's own act would be the application deciding, and
//! a laptop that stays awake in a bag because a video was paused is the failure
//! everybody knows. What a person's own sleep overrode is still named
//! ([`Going::overriding`]), so a turn that was running is not a surprise.

use std::time::SystemTime;

use alo_capability::Grants;
use alo_strings::{Filling, Said, Strings};

use crate::changes::Settings;
use crate::holding::Holding;
use crate::keeper::Keeper;
use crate::lid::{Displays, LidClosed};
use crate::words;

/// Why the machine is being asked to sleep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// The lid was closed.
    LidClosed,
    /// The person chose Sleep, or pressed the key that sleeps.
    YouAsked,
    /// Nobody has used the machine for as long as it waits.
    NobodyIsUsingIt,
}

/// What was decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decided {
    /// The machine sleeps — once the seat is locked.
    Sleeps(Going),
    /// The machine stays awake, and why.
    StaysAwake(StaysAwake),
}

/// Why the machine is staying awake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaysAwake {
    /// The lid is closed, another display is attached, and the person chose
    /// that.
    WithTheLidClosed,
    /// Nobody is using the machine, and these are holding it awake. Never
    /// empty.
    KeptBy(Vec<Keeper>),
}

impl StaysAwake {
    /// Why, one sentence per reason, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::WithTheLidClosed => {
                vec![strings.say(&words::AWAKE_WITH_THE_LID_CLOSED.key(), &Filling::nothing())]
            }
            Self::KeptBy(keepers) => keepers.iter().map(|kept| kept.said(strings)).collect(),
        }
    }
}

/// A sleep that was decided, and is not yet carried out.
///
/// Carried out only by `Going::carried_out`, which locks the seat first.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a sleep that was decided and never carried out has not happened"]
pub struct Going {
    /// Why.
    why: Why,
    /// What was holding the machine awake, and was overridden by the person's
    /// own act.
    overriding: Vec<Keeper>,
}

impl Going {
    /// Why the machine is going to sleep.
    #[must_use]
    pub const fn why(&self) -> Why {
        self.why
    }

    /// What was holding the machine awake and did not hold off this sleep,
    /// because the person closed the lid or chose Sleep. Empty for a machine
    /// that sleeps on its own, which nothing was holding.
    #[must_use]
    pub fn overriding(&self) -> &[Keeper] {
        &self.overriding
    }
}

/// Whether the machine sleeps now, for this reason.
///
/// `holding` is asked [`Holding::as_it_stands`] at `now`, so a revoked grant or
/// an ended turn is not holding anything by the time it is asked.
#[must_use]
pub fn asked<H>(
    why: Why,
    settings: &Settings,
    displays: Displays,
    holding: &mut Holding<H>,
    grants: &Grants,
    now: SystemTime,
) -> Decided {
    let keepers = holding.as_it_stands(grants, now);
    match why {
        Why::LidClosed => match settings.lid.closed(displays) {
            LidClosed::StaysAwake => Decided::StaysAwake(StaysAwake::WithTheLidClosed),
            LidClosed::Sleeps => Decided::Sleeps(Going {
                why,
                overriding: keepers,
            }),
        },
        Why::YouAsked => Decided::Sleeps(Going {
            why,
            overriding: keepers,
        }),
        Why::NobodyIsUsingIt if keepers.is_empty() => Decided::Sleeps(Going {
            why,
            overriding: Vec::new(),
        }),
        Why::NobodyIsUsingIt => Decided::StaysAwake(StaysAwake::KeptBy(keepers)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_capability::Applicant;
    use alo_portals::{Portal, Request};

    use super::*;
    use crate::lid::Lid;
    use crate::testing::{
        Holds, TheMachine, allowed_to_stay_awake, in_english, noon, on_a_machine,
    };

    /// **Every keeper holds off a machine sleeping on its own, and each is
    /// named** — the person's setting, an application and a turn, in that
    /// order, one sentence each.
    #[test]
    fn every_keeper_holds_off_idle_sleep_and_is_named() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let (grants, _) = allowed_to_stay_awake("org.libreoffice.Impress");
        let allowed = Request::of("org.libreoffice.Impress", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();
        let settings = Settings::shipped();
        let mut holding = Holding::none();

        assert!(matches!(
            asked(
                Why::NobodyIsUsingIt,
                &settings,
                Displays::OnlyItsOwn,
                &mut holding,
                &grants,
                noon()
            ),
            Decided::Sleeps(_)
        ));

        holding.your_setting(true, &mut logind, &strings).unwrap();
        holding
            .an_application(&allowed, &grants, noon(), &mut logind, &strings)
            .unwrap();
        on_a_machine(|turning| {
            holding
                .a_turn(turning, noon(), &mut logind, &strings)
                .unwrap();
            let decided = asked(
                Why::NobodyIsUsingIt,
                &settings,
                Displays::OnlyItsOwn,
                &mut holding,
                &grants,
                noon(),
            );
            let Decided::StaysAwake(awake) = decided else {
                unreachable!("three keepers hold a machine nobody is using awake");
            };
            assert!(matches!(
                &awake,
                StaysAwake::KeptBy(keepers) if matches!(
                    keepers.as_slice(),
                    [Keeper::YourSetting, Keeper::AnApplication(_), Keeper::TheAgent(_)]
                )
            ));
            let said = awake.said(&strings);
            assert_eq!(said.len(), 3);
            assert!(said.iter().all(|one| !one.is_a_bug()));
            assert!(
                said.get(1)
                    .is_some_and(|one| one.text().contains("org.libreoffice.Impress"))
            );
        });
    }

    /// **A keeper never holds off the person's own sleep**: closing the lid and
    /// choosing Sleep both sleep, and name what they overrode.
    #[test]
    fn a_keeper_does_not_hold_off_a_sleep_the_person_asked_for() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let grants = Grants::default();
        let mut holding = Holding::none();
        holding.your_setting(true, &mut logind, &strings).unwrap();

        for why in [Why::LidClosed, Why::YouAsked] {
            let decided = asked(
                why,
                &Settings::shipped(),
                Displays::OnlyItsOwn,
                &mut holding,
                &grants,
                noon(),
            );
            let Decided::Sleeps(going) = decided else {
                unreachable!("{why:?} is the person's own act");
            };
            assert_eq!(going.why(), why);
            assert_eq!(going.overriding(), [Keeper::YourSetting]);
        }
    }

    /// **A closed lid with another display attached stays awake only if the
    /// person chose it**, and says so.
    #[test]
    fn a_closed_lid_follows_the_persons_choice() {
        let strings = in_english();
        let grants = Grants::default();
        let mut holding: Holding<()> = Holding::none();
        let chose = Settings {
            lid: Lid::StaysAwakeWithADisplay,
            keep_awake: false,
        };

        let decided = asked(
            Why::LidClosed,
            &chose,
            Displays::AnotherAttached,
            &mut holding,
            &grants,
            noon(),
        );
        assert_eq!(decided, Decided::StaysAwake(StaysAwake::WithTheLidClosed));
        if let Decided::StaysAwake(awake) = decided {
            assert!(awake.said(&strings).iter().all(|one| !one.is_a_bug()));
        }

        for (settings, displays) in [
            (Settings::shipped(), Displays::AnotherAttached),
            (chose, Displays::OnlyItsOwn),
        ] {
            assert!(matches!(
                asked(
                    Why::LidClosed,
                    &settings,
                    displays,
                    &mut holding,
                    &grants,
                    noon()
                ),
                Decided::Sleeps(_)
            ));
        }
    }

    /// **A grant revoked a moment ago holds nothing** when the machine next
    /// asks whether to sleep.
    #[test]
    fn a_revoked_application_does_not_hold_the_machine_awake() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let (mut grants, id) = allowed_to_stay_awake("org.videolan.VLC");
        let allowed = Request::of("org.videolan.VLC", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();
        let mut holding = Holding::none();
        holding
            .an_application(&allowed, &grants, noon(), &mut logind, &strings)
            .unwrap();
        assert_eq!(
            asked(
                Why::NobodyIsUsingIt,
                &Settings::shipped(),
                Displays::OnlyItsOwn,
                &mut holding,
                &grants,
                noon()
            ),
            Decided::StaysAwake(StaysAwake::KeptBy(vec![Keeper::AnApplication(
                Applicant::named("org.videolan.VLC")
            )]))
        );

        grants.revoke(id);
        assert!(matches!(
            asked(
                Why::NobodyIsUsingIt,
                &Settings::shipped(),
                Displays::OnlyItsOwn,
                &mut holding,
                &grants,
                noon()
            ),
            Decided::Sleeps(_)
        ));
    }
}
