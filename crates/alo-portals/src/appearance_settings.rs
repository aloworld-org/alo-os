//! The `org.freedesktop.appearance` namespace, answered from what the person set.
//!
//! `org.freedesktop.portal.Settings` is how an application that follows light
//! and dark, and the accent colour, asks what the desktop looks like. This
//! module is what it is answered with, and who may be answered — without the
//! bus, so both are decided and tested on any machine. `crate::settings_portal`
//! speaks it.
//!
//! # One namespace, and what it holds
//!
//! [`THE_NAMESPACE`] is the only one answered. Every other namespace a Linux
//! desktop keeps settings under — GNOME's, KDE's — is answered as unknown and
//! never with a value: those are that desktop's own settings, and this machine
//! is not that desktop.
//!
//! | Key | Type | From `alo-appearance` |
//! |---|---|---|
//! | `color-scheme` | `u` | [`Appearance::scheme_at`]: `1` dark, `2` light |
//! | `accent-color` | `(ddd)` | [`Appearance::accent_at`], each channel from 0 to 1 |
//!
//! Both are resolved **at the moment of asking**, at the time of day the
//! machine gives, so a person who turns dark at six is answered dark at six.
//!
//! **`contrast` is not answered.** The specification's third key is a person's
//! preference for higher contrast, and `alo-appearance` keeps no such
//! preference: its `contrast` module measures two colours against the standard,
//! and says nothing about what a person asked for. Answering `0`, *no
//! preference*, would be a value the person never set, so the key is left out
//! — [`Setting`] has no variant for it — until that crate keeps one.
//!
//! # Who is answered
//!
//! [`allowed`] judges the caller: a sandboxed application, named by its
//! identifier, holding a grant of the `appearance settings` facility at this
//! moment ([`crate::Request::judged`] for [`Portal::Settings`]). The person's
//! settings are read only after that, by [`read`], so a refused application is
//! never answered from them.

use std::time::SystemTime;

use alo_appearance::{Appearance, Colour, Scheme};

use crate::answered::{Outcome, Unanswered};
use crate::judging::Allowed;
use crate::portal::Portal;
use crate::request::Request;
use crate::the_machine::TheMachine;

/// The one namespace the Settings portal answers.
pub const THE_NAMESPACE: &str = "org.freedesktop.appearance";

/// One key of [`THE_NAMESPACE`] that is answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Setting {
    /// `color-scheme`: light or dark.
    ColorScheme,
    /// `accent-color`: the accent, for the scheme showing.
    AccentColor,
}

impl Setting {
    /// Every key answered, in the order the specification lists them.
    pub const EVERY: [Self; 2] = [Self::ColorScheme, Self::AccentColor];

    /// The key, as the specification names it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::ColorScheme => "color-scheme",
            Self::AccentColor => "accent-color",
        }
    }

    /// The setting `namespace` and `key` name, or [`None`] for anything this
    /// machine does not answer — another namespace, or a key it does not hold.
    #[must_use]
    pub fn named(namespace: &str, key: &str) -> Option<Self> {
        if namespace != THE_NAMESPACE {
            return None;
        }
        Self::EVERY.into_iter().find(|setting| setting.key() == key)
    }

    /// This setting's value, from `appearance` at `now`.
    #[must_use]
    pub fn of(self, appearance: &Appearance, now: alo_appearance::TimeOfDay) -> Value {
        match self {
            Self::ColorScheme => Value::ColorScheme(appearance.scheme_at(now)),
            Self::AccentColor => Value::AccentColor(appearance.accent_at(now)),
        }
    }
}

/// One setting's value, as `alo-appearance` resolved it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// Light or dark.
    ColorScheme(Scheme),
    /// The accent's colour.
    AccentColor(Colour),
}

impl Value {
    /// `color-scheme` as the specification writes it: `1` for dark, `2` for
    /// light.
    ///
    /// Never `0`, *no preference*: this machine always shows one or the other,
    /// and which is the person's setting or the release's default, resolved.
    #[must_use]
    pub const fn color_scheme(scheme: Scheme) -> u32 {
        match scheme {
            Scheme::Dark => 1,
            Scheme::Light => 2,
        }
    }

    /// `accent-color` as the specification writes it: red, green and blue, each
    /// from 0 to 1.
    #[must_use]
    pub fn accent_color(colour: Colour) -> (f64, f64, f64) {
        let part = |channel: u8| f64::from(channel) / 255.0;
        (
            part(colour.red()),
            part(colour.green()),
            part(colour.blue()),
        )
    }
}

/// Every answered setting and its value, in [`Setting::EVERY`]'s order.
pub type Values = Vec<(Setting, Value)>;

/// Whether a `ReadAll` namespace pattern reaches [`THE_NAMESPACE`].
///
/// The specification's patterns: the empty string is every namespace, and one
/// ending in `*` is every namespace beginning with what is before it. Anything
/// else is one namespace, named exactly.
#[must_use]
pub fn reaches(pattern: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => THE_NAMESPACE.starts_with(prefix),
        None => pattern.is_empty() || pattern == THE_NAMESPACE,
    }
}

/// Whether a `ReadAll` of `patterns` asks for [`THE_NAMESPACE`] — an empty list
/// being every namespace.
#[must_use]
pub fn asked_of(patterns: &[String]) -> bool {
    patterns.is_empty() || patterns.iter().any(|pattern| reaches(pattern))
}

/// Whether `application` may read the appearance settings at `at`, judged
/// against the grants as they are now.
///
/// # Errors
/// The [`Outcome`] the record keeps for a caller that may not, boxed because an
/// outcome can carry a whole opened file's answer:
/// [`Unanswered::NotIdentified`] for a program with no sandbox,
/// [`Outcome::NotARequest`] for a sandbox naming something that is not an
/// identifier, [`Unanswered::GrantsUnread`], and [`Outcome::Refused`] with the
/// grants' own refusal.
pub fn allowed(
    application: Option<&str>,
    machine: &dyn TheMachine,
    at: SystemTime,
) -> Result<Allowed, Box<Outcome>> {
    let unanswered = |why| Box::new(Outcome::Unanswered(why));
    let application = application.ok_or_else(|| unanswered(Unanswered::NotIdentified))?;
    let request = Request::of(application, Portal::Settings)
        .map_err(|not| Box::new(Outcome::NotARequest(not)))?;
    let grants = machine
        .grants()
        .ok_or_else(|| unanswered(Unanswered::GrantsUnread))?;
    request
        .judged(&grants, at)
        .map_err(|refused| Box::new(Outcome::Refused(refused)))
}

/// Every answered setting's value, as the person set it and at the machine's
/// time of day.
///
/// # Errors
/// [`Unanswered::AppearanceUnread`], boxed as [`allowed`]'s refusals are, when
/// the person's settings or the time of day cannot be read — never the release's
/// defaults in their place.
pub fn read(machine: &dyn TheMachine) -> Result<Values, Box<Outcome>> {
    let unread = || Box::new(Outcome::Unanswered(Unanswered::AppearanceUnread));
    let appearance = machine.appearance().ok_or_else(unread)?;
    let now = machine.time_of_day().ok_or_else(unread)?;
    Ok(Setting::EVERY
        .into_iter()
        .map(|setting| (setting, setting.of(&appearance, now)))
        .collect())
}

/// The settings in `now` whose value differs from `before`.
#[must_use]
pub fn changed(before: &Values, now: &Values) -> Values {
    now.iter()
        .filter(|(setting, value)| {
            before
                .iter()
                .find(|(was, _)| was == setting)
                .is_none_or(|(_, was)| was != value)
        })
        .copied()
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::refused::Refused;
    use crate::the_machine::Applications;
    use alo_appearance::{Accent, Following, TimeOfDay};
    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    const NEWSFLASH: &str = "io.gitlab.news_flash.NewsFlash";

    /// A machine whose parts a test decides, and which says whether the
    /// person's appearance was read.
    #[derive(Default)]
    struct AMachine {
        grants: Option<Grants>,
        appearance: Option<Appearance>,
        time: Option<TimeOfDay>,
        appearance_read: AtomicBool,
    }

    impl TheMachine for AMachine {
        fn grants(&self) -> Option<Grants> {
            self.grants.clone()
        }
        fn applications(&self) -> Option<Applications> {
            None
        }
        fn appearance(&self) -> Option<Appearance> {
            self.appearance_read.store(true, Ordering::SeqCst);
            self.appearance.clone()
        }
        fn time_of_day(&self) -> Option<TimeOfDay> {
            self.time
        }
    }

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn granted(reach: Reach) -> Grants {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked_for(
                &Applicant::named(NEWSFLASH).grantee(),
                reach,
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        grants
    }

    /// **Two keys, each named as the specification names it**, and every
    /// other key or namespace — `contrast` among them — is none of them.
    #[test]
    fn only_the_appearance_namespace_and_its_answered_keys_are_named() {
        assert_eq!(
            Setting::named(THE_NAMESPACE, "color-scheme"),
            Some(Setting::ColorScheme)
        );
        assert_eq!(
            Setting::named(THE_NAMESPACE, "accent-color"),
            Some(Setting::AccentColor)
        );
        for (namespace, key) in [
            (THE_NAMESPACE, "contrast"),
            (THE_NAMESPACE, ""),
            ("org.gnome.desktop.interface", "color-scheme"),
            ("org.freedesktop.appearance.extra", "color-scheme"),
            ("", "color-scheme"),
        ] {
            assert_eq!(Setting::named(namespace, key), None, "{namespace} {key}");
        }
    }

    /// **The patterns `ReadAll` takes reach the namespace the way the
    /// specification says**, and no further.
    #[test]
    fn a_read_all_pattern_reaches_the_namespace_as_the_specification_says() {
        for reaching in ["", "*", "org.*", "org.freedesktop.*", THE_NAMESPACE] {
            assert!(reaches(reaching), "{reaching}");
        }
        for not in [
            "org.gnome.*",
            "org.freedesktop.appearance.",
            "org.freedesktop.appearance.*",
            "org.freedesktop.Appearance",
            "*.appearance",
        ] {
            assert!(!reaches(not), "{not}");
        }
        assert!(asked_of(&[]));
        assert!(!asked_of(&["org.gnome.desktop.interface".to_owned()]));
        assert!(asked_of(&[
            "org.gnome.desktop.interface".to_owned(),
            "org.freedesktop.*".to_owned()
        ]));
    }

    /// **The values are what `alo-appearance` resolves, at the time given**,
    /// written the way the specification writes them.
    #[test]
    fn the_values_are_the_persons_resolved_at_the_time_given() {
        let mut appearance = Appearance::shipped();
        appearance.set_accent(Accent::Rose);
        appearance.follow(Following::from(Scheme::Dark));
        let machine = AMachine {
            appearance: Some(appearance.clone()),
            time: Some(TimeOfDay::checked(9, 0).unwrap()),
            ..AMachine::default()
        };
        let values = read(&machine).unwrap();
        assert_eq!(
            values,
            [
                (Setting::ColorScheme, Value::ColorScheme(Scheme::Dark)),
                (
                    Setting::AccentColor,
                    Value::AccentColor(Accent::Rose.on(Scheme::Dark))
                ),
            ]
        );
        assert_eq!(Value::color_scheme(Scheme::Dark), 1);
        assert_eq!(Value::color_scheme(Scheme::Light), 2);
        let (red, green, blue) = Value::accent_color(Colour::of(255, 0, 51));
        assert!((red - 1.0).abs() < f64::EPSILON);
        assert!(green.abs() < f64::EPSILON);
        assert!((blue - 0.2).abs() < f64::EPSILON);
    }

    /// **Settings that cannot be read are refused, never replaced by the
    /// release's defaults** — and a time of day that cannot be told is the
    /// same refusal.
    #[test]
    fn unread_settings_are_refused_rather_than_defaulted() {
        let unread = Outcome::Unanswered(Unanswered::AppearanceUnread);
        let no_file = AMachine {
            time: Some(TimeOfDay::checked(9, 0).unwrap()),
            ..AMachine::default()
        };
        assert_eq!(*read(&no_file).unwrap_err(), unread);
        let no_clock = AMachine {
            appearance: Some(Appearance::shipped()),
            ..AMachine::default()
        };
        assert_eq!(*read(&no_clock).unwrap_err(), unread);
    }

    /// **Only a sandboxed application holding the facility is allowed**, and
    /// each refusal is the one the record keeps — none of them reading the
    /// person's settings.
    #[test]
    fn only_an_application_granted_appearance_settings_is_allowed() {
        let machine = AMachine {
            grants: Some(granted(Reach::Facility(Facility::AppearanceSettings))),
            appearance: Some(Appearance::shipped()),
            time: Some(TimeOfDay::checked(9, 0).unwrap()),
            ..AMachine::default()
        };
        let allowed = allowed(Some(NEWSFLASH), &machine, noon()).unwrap();
        assert_eq!(allowed.portal(), Portal::Settings);
        assert_eq!(allowed.against().len(), 1);

        assert_eq!(
            *super::allowed(None, &machine, noon()).unwrap_err(),
            Outcome::Unanswered(Unanswered::NotIdentified)
        );
        assert!(matches!(
            *super::allowed(Some("not an identifier"), &machine, noon()).unwrap_err(),
            Outcome::NotARequest(_)
        ));
        assert_eq!(
            *super::allowed(Some("org.example.Stranger"), &machine, noon()).unwrap_err(),
            Outcome::Refused(Refused::NothingGranted {
                application: Applicant::named("org.example.Stranger"),
                portal: Portal::Settings,
            })
        );
        assert!(matches!(
            *super::allowed(Some(NEWSFLASH), &machine, noon() + Duration::from_secs(61))
                .unwrap_err(),
            Outcome::Refused(_)
        ));

        let camera_only = AMachine {
            grants: Some(granted(Reach::Facility(Facility::Camera))),
            ..AMachine::default()
        };
        assert!(matches!(
            *super::allowed(Some(NEWSFLASH), &camera_only, noon()).unwrap_err(),
            Outcome::Refused(Refused::NotAllowed { .. })
        ));
        let no_grants = AMachine::default();
        assert_eq!(
            *super::allowed(Some(NEWSFLASH), &no_grants, noon()).unwrap_err(),
            Outcome::Unanswered(Unanswered::GrantsUnread)
        );
        for machine in [&machine, &camera_only, &no_grants] {
            assert!(
                !machine.appearance_read.load(Ordering::SeqCst),
                "judging read the person's settings"
            );
        }
    }

    /// **Only what moved is a change**, including a setting that was not there
    /// before.
    #[test]
    fn only_what_moved_is_changed() {
        let light = vec![
            (Setting::ColorScheme, Value::ColorScheme(Scheme::Light)),
            (
                Setting::AccentColor,
                Value::AccentColor(Accent::Verdigris.on(Scheme::Light)),
            ),
        ];
        let dark_scheme = vec![
            (Setting::ColorScheme, Value::ColorScheme(Scheme::Dark)),
            (
                Setting::AccentColor,
                Value::AccentColor(Accent::Verdigris.on(Scheme::Light)),
            ),
        ];
        assert!(changed(&light, &light).is_empty());
        assert_eq!(
            changed(&light, &dark_scheme),
            [(Setting::ColorScheme, Value::ColorScheme(Scheme::Dark))]
        );
        assert_eq!(changed(&Vec::new(), &light), light);
    }
}
