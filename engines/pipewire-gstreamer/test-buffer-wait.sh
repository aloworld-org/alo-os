#!/usr/bin/env bash
set -euo pipefail
here=$(cd -- "$(dirname -- "$0")" && pwd)
built=${1:?Pass the plugin build directory containing its patched pool and config}
scratch=$(mktemp -d /tmp/studio-pool-wait.XXXXXX)
trap 'rm -rf -- "$scratch"' EXIT
sources=()
for source in "$here"/src/gst/*.c; do
  [[ "$source" == "$here/src/gst/gstpipewirepool.c" ]] || sources+=("$source")
done
compile() {
  "${CC:-cc}" -std=gnu11 -O2 -fno-strict-aliasing -D_GNU_SOURCE \
    -Wall -Wextra -Wno-unused-parameter -I"$1" -I"$built" -I"$here/src" -I"$here/src/gst" \
    "$here/tests/buffer-wait.c" "${sources[@]}" -o "$2" \
    $(pkg-config --cflags --libs libpipewire-0.3 gstreamer-1.0 gstreamer-base-1.0 \
      gstreamer-net-1.0 gstreamer-video-1.0 gstreamer-audio-1.0 gstreamer-allocators-1.0) -lm
}
compile "$built" "$scratch/recheck"
"$scratch/recheck"
mkdir "$scratch/baseline"
cp "$here/src/gst/gstpipewirepool.c" "$scratch/baseline/gstpipewirepool.c"
patch --batch --fuzz=0 "$scratch/baseline/gstpipewirepool.c" "$here/patches/buffer-release-race.patch"
compile "$scratch/baseline" "$scratch/baseline-wait"
set +e
"$scratch/baseline-wait"
baseline=$?
set -e
[[ "$baseline" == 73 ]]
printf 'BASELINE_MISSED_NOTIFICATION_REPRODUCED\n'
