# Private recording verification

These instruments are person-side tests, not agent tools. The normal Studio
package has verification disabled. The separate reference package and disposable
KVM walk retain only their bounded reports, pictures and recording files.

After collecting `screen.webm`, `window.webm`, `reference.json` and
`session-report.json`, inspect decoded frames to choose explicit rectangles in
each recording. Store them in a JSON file:

```json
{
  "screen": {"marker_region": [20, 30, 40, 40], "motion_region": [50, 80, 100, 30]},
  "window": {"marker_region": [20, 30, 40, 40], "motion_region": [50, 80, 100, 30]}
}
```

The values above describe the format only. Replace them with observed pixel
coordinates `[x, y, width, height]`; the marker box must avoid text and borders,
and the motion box must include independently changing content. Do not infer a
pass from a flashing marker alone. Both rectangles are bounded by the real
decoded frame dimensions. No default or automatic rectangle selection is used.

On a host with FFmpeg and ffprobe installed, run:

```sh
python3 -I apps/alo-studio/verification/audit_installed_walk.py EVIDENCE \
  --source EXACT_SOURCE_COMMIT --regions regions.json > media-report.json
```

Exit zero means the report's `decoded-recording-media` scope passed. A failed
provenance check blocks decoding; a failed live reference prevents window
synchronization certification. Other failures remain visible independently.
The report retains consumed input hashes, deployment identities, regions and
the health inspectors' measurements. It changes no input files, samples,
pictures or timestamps.

This report cannot certify native consent, export, source isolation, microphone
capture stopping during pause, cleanup, installed agent grants or a hardware
microphone. Full installed acceptance remains `not_reached`, even when the
decoded media scope passes. Complete the separate acceptance walk in
`docs/walks/recording-a-demo-in-studio.md` before describing Studio as ready.

The `verification-instruments` job in `.github/workflows/studio.yml` reruns
these tests on pull requests, including real decoder and isolated GTK/AT-SPI
fixtures. Synthetic test media and patched decoder observations in unit tests
are never installed acceptance evidence.
