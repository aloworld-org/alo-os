#!/usr/bin/env bash
set -euo pipefail
# Private walk input, never a market entry or a public release. Run after the
# normal package build, under the same machine-wide Cargo lock if applicable.
app_dir="$(cd -- "$(dirname -- "$0")/.." && pwd)"
work_dir="${ALO_STUDIO_BUILD_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/alo-studio-flatpak}"
test -f "$work_dir/source/Cargo.lock"
test -f "$work_dir/source/.cargo/config.toml"
test -d "$work_dir/source/vendor"
# The prepared source and vendor tree are the normal package's exact inputs.
# Refuse to combine that source with a reference from a different revision.
cmp "$app_dir/Cargo.lock" "$work_dir/source/Cargo.lock"
cmp "$app_dir/examples/studio-reference-window.rs" "$work_dir/source/examples/studio-reference-window.rs"
cmp "$app_dir/verification/reference_signal.rs" "$work_dir/source/verification/reference_signal.rs"
python3 -I "$app_dir/verification/make_reference_manifest.py" \
  "$work_dir/source/packaging/org.aloworld.Studio.json" \
  > "$work_dir/source/packaging/org.aloworld.Studio.Verification.json"
cd "$work_dir/source"
flatpak-builder --user --disable-rofiles-fuse --repo="$work_dir/reference-repo" \
  --force-clean "$work_dir/reference-build" \
  "$work_dir/source/packaging/org.aloworld.Studio.Verification.json"
flatpak build-bundle "$work_dir/reference-repo" "$work_dir/studio-reference.flatpak" \
  org.aloworld.Studio.Verification \
  --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo
printf 'Private reference bundle: %s\n' "$work_dir/studio-reference.flatpak"
