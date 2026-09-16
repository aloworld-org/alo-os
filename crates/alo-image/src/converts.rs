//! Every promise ADR 0039 makes about the converting service, checked against
//! the files the image ships.
//!
//! `crate::converter` reads them; this says what is wrong, one [`Wrong`] per
//! broken promise:
//!
//! - **the engine is one exact release, checked by digest**, and the service
//!   starts it from where that release installs;
//! - **the service is aboard** and runs as a login of its own that holds
//!   nothing;
//! - **it reaches nothing and can name no person's file** — no network, no
//!   address family but Unix, no view of any home folder;
//! - **it is reached at the one path the verb knocks at**, with its listening
//!   socket on standard input.

use crate::converter::{THE_CONVERTER, THE_CONVERTERS_BINARY};
use crate::image::Image;
use crate::wrong::Wrong;

/// The login the service runs as.
const ITS_LOGIN: &str = "alo-convert";

/// A section of a unit.
const SERVICE: &str = "Service";

/// Everything the converting service's files disagree with.
pub(crate) fn everything_wrong_with_the_converter(image: &Image, wrong: &mut Vec<Wrong>) {
    the_engine_is_pinned_and_checked(image, wrong);
    the_service_is_aboard_as_a_login_of_its_own(image, wrong);
    the_service_reaches_nothing(image, wrong);
    the_service_is_reached_where_the_verb_knocks(image, wrong);
}

/// **The engine is one exact release, checked before it is trusted, and is
/// where the service starts it.**
fn the_engine_is_pinned_and_checked(image: &Image, wrong: &mut Vec<Wrong>) {
    let converter = image.converter();
    let version = converter.version().unwrap_or("-");
    let pinned = is_a_release(version)
        && converter
            .digest()
            .is_some_and(crate::recipe::is_a_whole_digest)
        && converter.is_checked();
    if !pinned {
        wrong.push(Wrong::TheConverterArrivesUnverified {
            version: version.to_owned(),
        });
    }
    let installs_to = version
        .rsplit_once('.')
        .map(|(major_minor, _)| format!("/opt/libreoffice{major_minor}/"));
    let started_from_it =
        installs_to.is_some_and(|folder| alo_converting::engine::THE_ENGINE.starts_with(&folder));
    if !converter.tests_its_engine() || !started_from_it {
        wrong.push(Wrong::TheConverterIsNotAboard {
            what: format!("the engine at {}", alo_converting::engine::THE_ENGINE),
        });
    }
}

/// **The service is on the image and runs as a login of its own that holds
/// nothing.**
fn the_service_is_aboard_as_a_login_of_its_own(image: &Image, wrong: &mut Vec<Wrong>) {
    let converter = image.converter();
    let service = converter.service();
    if !converter.lands_its_binary()
        || service.one(SERVICE, "ExecStart") != Some(THE_CONVERTERS_BINARY)
    {
        wrong.push(Wrong::TheConverterIsNotAboard {
            what: THE_CONVERTERS_BINARY.to_owned(),
        });
    }
    let own_login = service.one(SERVICE, "User") == Some(ITS_LOGIN)
        && service.one(SERVICE, "Group") == Some(ITS_LOGIN)
        && image.login_called(ITS_LOGIN).is_some();
    let holds_nothing = service.says(SERVICE, "CapabilityBoundingSet")
        && service.listed(SERVICE, "CapabilityBoundingSet").is_empty()
        && service.listed(SERVICE, "AmbientCapabilities").is_empty()
        && service.one(SERVICE, "NoNewPrivileges") == Some("yes");
    if !own_login {
        wrong.push(Wrong::TheConverterMayReach {
            converter: THE_CONVERTER.to_owned(),
            setting: "User".to_owned(),
            says: service.one(SERVICE, "User").unwrap_or("-").to_owned(),
        });
    }
    if !holds_nothing {
        wrong.push(Wrong::TheConverterMayReach {
            converter: THE_CONVERTER.to_owned(),
            setting: "CapabilityBoundingSet".to_owned(),
            says: service.listed(SERVICE, "CapabilityBoundingSet").join(" "),
        });
    }
}

/// **The service reaches nothing off this machine, and no person's files.**
fn the_service_reaches_nothing(image: &Image, wrong: &mut Vec<Wrong>) {
    let service = image.converter().service();
    for (setting, wanted) in [
        ("PrivateNetwork", "yes"),
        ("ProtectHome", "yes"),
        ("PrivateTmp", "yes"),
    ] {
        let says = service.one(SERVICE, setting);
        if says != Some(wanted) {
            wrong.push(Wrong::TheConverterMayReach {
                converter: THE_CONVERTER.to_owned(),
                setting: setting.to_owned(),
                says: says.unwrap_or("-").to_owned(),
            });
        }
    }
    let families = service.listed(SERVICE, "RestrictAddressFamilies");
    if families != ["AF_UNIX"] {
        wrong.push(Wrong::TheConverterMayReach {
            converter: THE_CONVERTER.to_owned(),
            setting: "RestrictAddressFamilies".to_owned(),
            says: families.join(" "),
        });
    }
}

/// **The service is reached at the path the verb knocks at, and nowhere
/// else.**
fn the_service_is_reached_where_the_verb_knocks(image: &Image, wrong: &mut Vec<Wrong>) {
    let converter = image.converter();
    let listens = converter.socket().listed("Socket", "ListenStream");
    let handed_on_standard_input = converter.service().one(SERVICE, "StandardInput")
        == Some("socket")
        && converter.socket().one("Socket", "Accept") == Some("no");
    if listens != [alo_converting::THE_SOCKET] || !handed_on_standard_input {
        wrong.push(Wrong::TheConverterListensElsewhere {
            listens: listens.join(" "),
        });
    }
}

/// Whether a version is three numbers, and so one release rather than a name
/// for whichever is newest.
fn is_a_release(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{THE_CONTAINERFILE, a_copy_of_the_image, edited, image_at};

    /// The converter's unit, beneath the image's root.
    const THE_UNIT: &str = "usr/lib/systemd/system/alo-convertd.service";

    /// The converter's socket, beneath the image's root.
    const THE_SOCKET_UNIT: &str = "usr/lib/systemd/system/alo-convertd.socket";

    /// What this crate finds wrong with an image, about converting only.
    fn wrong_with_the_converter(root: &std::path::Path) -> Vec<Wrong> {
        let mut wrong = Vec::new();
        everything_wrong_with_the_converter(&image_at(root), &mut wrong);
        wrong
    }

    /// **The shipped image keeps every promise ADR 0039 makes about converting.**
    #[test]
    fn the_shipped_converter_keeps_every_promise() {
        assert_eq!(
            wrong_with_the_converter(std::path::Path::new(crate::THE_IMAGE)),
            Vec::new()
        );
    }

    /// **An engine left floating, or whose digest nobody checks, is caught.**
    #[test]
    fn an_engine_floating_or_unchecked_is_caught() {
        let root = a_copy_of_the_image("converter-floating");
        edited(
            &root,
            THE_CONTAINERFILE,
            "ARG THE_CONVERTER=26.2.6",
            "ARG THE_CONVERTER=latest",
        );
        let wrong = wrong_with_the_converter(&root);
        assert!(
            wrong
                .iter()
                .any(|one| matches!(one, Wrong::TheConverterArrivesUnverified { version } if version == "latest")),
            "{wrong:?}"
        );

        let root = a_copy_of_the_image("converter-unchecked");
        edited(
            &root,
            THE_CONTAINERFILE,
            "echo \"${THE_CONVERTER_SHA256}  /converter.tar.gz\" | sha256sum --check -",
            "true",
        );
        assert!(matches!(
            wrong_with_the_converter(&root).as_slice(),
            [Wrong::TheConverterArrivesUnverified { .. }]
        ));
    }

    /// **A service given the network, or a view of home folders, is caught**,
    /// naming the setting.
    #[test]
    fn a_service_that_could_reach_something_is_caught() {
        let root = a_copy_of_the_image("converter-network");
        edited(&root, THE_UNIT, "PrivateNetwork=yes", "PrivateNetwork=no");
        edited(
            &root,
            THE_UNIT,
            "RestrictAddressFamilies=AF_UNIX",
            "RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6",
        );
        edited(&root, THE_UNIT, "ProtectHome=yes", "ProtectHome=read-only");
        let settings: Vec<String> = wrong_with_the_converter(&root)
            .into_iter()
            .filter_map(|one| match one {
                Wrong::TheConverterMayReach { setting, .. } => Some(setting),
                _ => None,
            })
            .collect();
        assert_eq!(
            settings,
            ["PrivateNetwork", "ProtectHome", "RestrictAddressFamilies"]
        );
    }

    /// **A service run as the person, or holding a capability, is caught.**
    #[test]
    fn a_service_run_as_someone_else_or_holding_something_is_caught() {
        let root = a_copy_of_the_image("converter-login");
        edited(&root, THE_UNIT, "User=alo-convert", "User=alo");
        edited(
            &root,
            THE_UNIT,
            "CapabilityBoundingSet=\n",
            "CapabilityBoundingSet=CAP_DAC_READ_SEARCH\n",
        );
        let wrong = wrong_with_the_converter(&root);
        assert_eq!(wrong.len(), 2, "{wrong:?}");
    }

    /// **A socket somewhere the verb does not knock is caught**, and so is a
    /// service that would not be handed it.
    #[test]
    fn a_socket_elsewhere_is_caught() {
        let root = a_copy_of_the_image("converter-socket");
        edited(
            &root,
            THE_SOCKET_UNIT,
            "ListenStream=/run/alo-convertd/socket",
            "ListenStream=127.0.0.1:9000",
        );
        assert!(matches!(
            wrong_with_the_converter(&root).as_slice(),
            [Wrong::TheConverterListensElsewhere { listens }] if listens == "127.0.0.1:9000"
        ));

        let root = a_copy_of_the_image("converter-stdin");
        edited(
            &root,
            THE_UNIT,
            "StandardInput=socket",
            "StandardInput=null",
        );
        assert!(matches!(
            wrong_with_the_converter(&root).as_slice(),
            [Wrong::TheConverterListensElsewhere { .. }]
        ));
    }

    /// **An image that dropped the service or the engine is caught.**
    #[test]
    fn an_image_without_the_service_or_the_engine_is_caught() {
        let root = a_copy_of_the_image("converter-dropped");
        edited(
            &root,
            THE_CONTAINERFILE,
            "COPY --from=built /alo-os/target/${THE_TARGET}/release/alo-convertd /usr/libexec/alo-convertd",
            "",
        );
        edited(
            &root,
            THE_CONTAINERFILE,
            " && test -x /opt/libreoffice26.2/program/soffice",
            "",
        );
        let wrong = wrong_with_the_converter(&root);
        assert_eq!(wrong.len(), 2, "{wrong:?}");
        assert!(
            wrong
                .iter()
                .all(|one| matches!(one, Wrong::TheConverterIsNotAboard { .. })),
            "{wrong:?}"
        );
    }

    /// Only three numbers are a release.
    #[test]
    fn only_three_numbers_are_a_release() {
        assert!(is_a_release("26.2.6"));
        for not in ["latest", "26.2", "26.2.x", "26..6", "26.2.6.3"] {
            assert!(!is_a_release(not), "{not}");
        }
    }
}
