#!/usr/bin/env bash
set -euo pipefail
here=$(cd -- "$(dirname -- "$0")" && pwd)
output=${1:?Pass an output directory}
mkdir -p "$output"
output=$(cd -- "$output" && pwd)
cd "$here"
sha256sum --check --quiet SHA256SUMS
pkg-config --atleast-version=1.4 libpipewire-0.3
pkg-config --atleast-version=1.24 gstreamer-allocators-1.0
cat > "$output/config.h" <<'CONFIG'
#define PACKAGE "pipewire"
#define PACKAGE_VERSION "1.4.90+ee2c6eb4.2770143f"
#define HAVE_GSTREAMER_SHM_ALLOCATOR 1
CONFIG
cp src/gst/gstpipewirepool.c "$output/gstpipewirepool.c"
patch --batch --fuzz=0 "$output/gstpipewirepool.c" patches/buffer-release-race.patch
sources=()
for source in src/gst/*.c; do
  if [[ "$source" == src/gst/gstpipewirepool.c ]]; then
    sources+=("$output/gstpipewirepool.c")
  else
    sources+=("$source")
  fi
done
"${CC:-cc}" -std=gnu11 -O2 -fPIC -shared -fvisibility=hidden \
  -fno-strict-aliasing -fno-strict-overflow -D_GNU_SOURCE \
  -fstack-protector-strong -D_FORTIFY_SOURCE=2 \
  -Wall -Wextra -Wno-unused-parameter -Wl,-z,relro,-z,now \
  -I"$output" -Isrc -Isrc/gst -o "$output/libgstpipewire.so" "${sources[@]}" \
  $(pkg-config --cflags --libs libpipewire-0.3 gstreamer-1.0 gstreamer-base-1.0 \
      gstreamer-net-1.0 gstreamer-video-1.0 gstreamer-audio-1.0 gstreamer-allocators-1.0) -lm
