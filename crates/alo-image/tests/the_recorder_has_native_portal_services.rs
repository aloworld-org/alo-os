//! The image must carry capture workers and native selection without a second frontend.

#[test]
fn capture_workers_and_native_services_are_in_the_recipe() {
    let recipe = include_str!("../../../image/Containerfile");
    for requirement in [
        "--package alo-capture-provider",
        "--package alo-capture-remote",
        "/usr/libexec/alo-capture-provider",
        "/usr/libexec/alo-capture-remote",
        "engines/pipewire-gstreamer/build.sh",
        "/usr/lib/alo-capture-provider/gstreamer-1.0/libgstpipewire.so",
        "xdg-desktop-portal-gtk",
        "pipewire-pulseaudio",
        "wireplumber rtkit",
        "systemctl --global mask xdg-desktop-portal.service",
        "pipewire-pulse.socket",
        "test -x /usr/bin/setpriv",
    ] {
        assert!(
            recipe.contains(requirement),
            "missing recorder prerequisite: {requirement}"
        );
    }
    let activation = include_str!(
        "../../../image/usr/share/dbus-1/services/org.freedesktop.portal.Desktop.service"
    );
    assert!(activation.contains("Name=org.freedesktop.portal.Desktop"));
    assert!(activation.contains("Exec=/usr/bin/false"));
    assert!(activation.contains("SystemdService=xdg-desktop-portal.service"));
    let portal_unit = include_str!("../../../image/usr/lib/systemd/system/alo-portald.service");
    assert!(portal_unit.contains("CacheDirectory=alo-capture-provider"));
    assert!(portal_unit.contains("CacheDirectoryMode=0700"));
    assert!(
        recipe.contains(
            "COPY image/usr/share/dbus-1/services/org.freedesktop.portal.Desktop.service"
        )
    );
}

#[test]
fn native_audio_scheduling_does_not_depend_on_an_unsupported_portal_interface() {
    let recipe = include_str!("../../../image/Containerfile");
    for (directory, config) in [
        (
            "pipewire.conf.d",
            include_str!(
                "../../../image/usr/share/pipewire/pipewire.conf.d/20-alo-native-realtime.conf"
            ),
        ),
        (
            "pipewire-pulse.conf.d",
            include_str!(
                "../../../image/usr/share/pipewire/pipewire-pulse.conf.d/20-alo-native-realtime.conf"
            ),
        ),
    ] {
        assert!(recipe.contains(&format!(
            "COPY image/usr/share/pipewire/{directory}/ /usr/share/pipewire/{directory}/"
        )));
        assert!(config.contains("module.rt.args"));
        assert!(config.contains("rtportal.enabled = false"));
        assert!(config.contains("rtkit.enabled = true"));
        assert!(!config.contains("context.modules"), "keep upstream modules");
        assert!(
            !config.contains("rt.prio"),
            "keep the broker's priority bound"
        );
    }
}
