#!/usr/bin/env bash
set -euo pipefail
# Run under the machine's Cargo lock when this shares an alo OS build machine.
source_dir="$(cd -- "$(dirname -- "$0")/.." && pwd)"
work_dir="${ALO_STUDIO_BUILD_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/alo-studio-flatpak}"
mkdir -p "$work_dir/source" "$work_dir/source/.cargo" "$work_dir/source/packaging"
cd "$source_dir"
tar --exclude=./target --exclude=./vendor --exclude=./repo --exclude=./build --exclude=./.flatpak-builder -cf - . | tar -xf - -C "$work_dir/source"
cd "$work_dir/source"
cargo vendor --locked vendor > .cargo/config.toml
flatpak-builder --user --disable-rofiles-fuse --install-deps-from=flathub --repo="$work_dir/repo" --force-clean "$work_dir/build" packaging/org.aloworld.Studio.json
flatpak build-bundle "$work_dir/repo" "$work_dir/alo-studio-0.1.0.flatpak" org.aloworld.Studio --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo
printf 'Installable bundle: %s\n' "$work_dir/alo-studio-0.1.0.flatpak"
