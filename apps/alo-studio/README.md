# alo Studio

A separate native application for recording a demo: pick a screen or window,
optionally add your microphone, record, pause, preview, and save a copy.

The microphone starts **off for every recording**. Videos stay on this device;
the application has no network permission. VP9 and Opus are encoded into WebM,
following alo OS's encoding decision. Your original stays in the app's private
library even after you export a copy.

## Run from source

Requires Rust 1.97+, GTK 4.14+, GStreamer 1.22+, and the PipeWire, VP9, Opus,
PulseAudio and GTK media plugins. On Ubuntu:

```sh
sudo apt install libgtk-4-dev libgtk-4-media-gstreamer \
  libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
  gstreamer1.0-pipewire gstreamer1.0-pulseaudio
cargo run --manifest-path apps/alo-studio/Cargo.toml
```

Screen selection requires a running `org.freedesktop.portal.ScreenCast`
service provided by the desktop. The app never replaces that permission with
unrestricted screen access. The system's picker may offer only the source
types supported by that desktop. Saving a copy uses the system file chooser.

## Application-market packaging

The desktop ID, Flatpak ID and AppStream ID are all `org.aloworld.Studio`.
`packaging/org.aloworld.Studio.json` is the source-build manifest. It has no
network permission, broad home-folder access, or camera permission. The audio
socket permits the optional microphone; the app opens it only when requested.

Use `packaging/build-flatpak.sh` to prepare vendored sources, build the Flatpak,
and export an installable bundle. A market publisher must publish this build to
a signed Flatpak remote before adding its identifier and source to alo OS's
catalogue. **It is not on Flathub merely because it has a manifest.**

## Checks

```sh
cargo fmt --manifest-path apps/alo-studio/Cargo.toml --check
cargo clippy --manifest-path apps/alo-studio/Cargo.toml --all-targets --features verification -- -D warnings
cargo test --manifest-path apps/alo-studio/Cargo.toml
```

The lifecycle and storage tests cover refusal paths, pause timing, microphone
reset, path traversal, empty recordings, and overwrite prevention. The optional
`verify-recording` example records a real X11 display through the production
encoder, pauses and resumes, finalizes the container, and decodes it to EOS.
The X11 diagnostic source is absent from ordinary installed builds.

User-facing copy goes through the gettext domain `alo-studio`. Translators can
provide standard gettext catalogues without changing the recording code.
