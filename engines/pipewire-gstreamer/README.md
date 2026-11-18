# Capture worker's PipeWire GStreamer plugin

This is the unchanged GStreamer plugin source from PipeWire commit
[`ee2c6eb41ebab02c91faeb567cba86ece44dd996`](https://github.com/PipeWire/pipewire/commit/ee2c6eb41ebab02c91faeb567cba86ece44dd996),
licensed under MIT/X11. `SHA256SUMS` covers every upstream file and `COPYING`.
The source files retain their upstream paths and notices. The separately pinned
upstream [buffer-release race fix](https://github.com/PipeWire/pipewire/commit/2770143f506a92e242f8a5b8b870929b515b48c0)
is applied to a build copy, with exact context and no fuzz. `SHA256SUMS` also
covers that patch. No daemon, SPA plugin,
codec, or renderer is vendored here.

The separately checksummed `buffer-wait-recheck.patch` bounds the publisher
pool's condition wait to 100 ms. The sink's process callback signals without
that pool mutex; a notification can arrive between dequeue and wait. A timed
predicate recheck avoids an indefinite wait without reversing the loop/pool
lock order. Queue bounds, buffer ownership, flushing and pause checks remain
the engine's own. ADR 0150 records the trace, private trial and removal rule.

PipeWire 1.4.11's distribution plugin terminates a provided video source when a
consumer removes its buffers. The pinned upstream implementation owns its
buffer memory and keeps a provided source alive across that link transition.
This is required when Studio pauses and resumes. ADR 0150 records the boundary.

`build.sh OUTPUT_DIRECTORY` builds against the OS's own PipeWire 1.4+ and
GStreamer 1.24+ development libraries. It generates only build configuration;
the imported C source files remain unchanged. Device discovery and DMA-DRM-specific
extensions are omitted. The native frame bridge uses shared-memory BGRx.

The image installs the plugin privately in
`/usr/lib/alo-capture-provider/gstreamer-1.0`. The publisher launcher explicitly
gives its capture worker that search path, a separate private registry cache,
and the requirement to use the pinned engine. These settings survive its clean
process environment; other tools inherit none of them. The portal unit creates
the cache directory for its own user with mode 0700. The
worker checks the selected plugin's filename and pinned version before opening
a stream. A missing or substituted private plugin is a refusal. Other programs
continue to use the distribution plugin.

The Studio CI builds this plugin on the image's pinned base and exercises
recording, decoded audio/video, and frames delivered after Resume. A software VM
trial is supporting evidence; it does not establish hardware frame rates.

Remove this copy when the OS's packaged GStreamer plugin supplies the required
provider buffer ownership and passes the same lifecycle checks. Upgrades must
replace the source and license together, refresh the manifest, and rerun those
checks. The Rust PipeWire binding is a separate rental in `../pipewire`.
