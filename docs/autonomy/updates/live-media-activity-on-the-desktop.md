# Live media activity on the desktop

The owner authorised completing alo Studio's desktop integration and a recording
walk on a booted alo system on 2026-11-04. Market publication is outside this step.

This makes an existing promise true: the visible indicator for every use of the
screen, camera, or microphone needs the desktop to read the real media graph.
The drawing and policy existed, but `alo-desktop` always supplied an empty list.

The desktop now asks `alo-in-use` for a complete reading off the frame thread.
It starts at most one outstanding reader, keeps the handoff bounded, and shows
an explicit unavailable reading while starting or when the server fails or is
slow. A failed reading cannot become an empty, apparently quiet indicator.
The new failure surface uses the existing translated `NotHeard` vocabulary.

The complete capture and save integration remains in progress. No booted alo
recording, permission revocation, or microphone hardware walk is claimed by
this prerequisite change. Validation results will be recorded after the scoped
checks run; existing app checks are not evidence for these OS changes.
