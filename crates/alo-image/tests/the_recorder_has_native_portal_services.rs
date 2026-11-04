//! The image must carry capture workers and native selection without a second frontend.

#[test]
fn capture_workers_and_native_services_are_in_the_recipe() {
    let recipe = include_str!("../../../image/Containerfile");
    for requirement in [
        "--package alo-capture-provider",
        "--package alo-capture-remote",
        "/usr/libexec/alo-capture-provider",
        "/usr/libexec/alo-capture-remote",
        "xdg-desktop-portal-gtk",
        "pipewire-pulseaudio",
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
    assert!(
        recipe.contains(
            "COPY image/usr/share/dbus-1/services/org.freedesktop.portal.Desktop.service"
        )
    );
}
