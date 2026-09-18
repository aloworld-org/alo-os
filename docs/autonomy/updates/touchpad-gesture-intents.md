# Touchpad gesture intents

Date: 2026-09-18. Workstream: hands on the desktop.
Responsible contributor: Codex development worker, lane B.
Assigned task: Task 5 — Gestures.

Status: implementation prepared; supervisor validation pending. This is not a
claim that the task has gated or is ready to publish. The operator reconstructed handoff.toml to request serialized supervisor validation.

Touchpad scroll, two-finger pinch zoom, and three- or four-finger desktop swipes
now have a closed decision API in `alo-desktops`. Each gesture can be disabled
independently. Natural and traditional scrolling, and every enable switch, are
kept in the person's `gestures.toml`. A gesture cannot request an agent invocation.

## Implementation and decisions

- `gesture_events.rs` holds owned input fields and the closed `Intent` enum:
  scroll, zoom, or the existing desktop `Switch`. The same extraction helpers
  used by the Linux adapter are exercised with supported and refused finger
  counts and both upstream scroll-direction configurations.
- `gestures.rs` holds a bounded recognizer for one touchpad. The session must
  retain one per device and reset it on focus change, device removal or seat
  pause. No device handle, target window, grant or context enters this state.
- `libinput_gestures.rs` borrows the existing unmodified `input` 0.9.1 library's
  events on Linux. It reads modern finger scroll, pinch and swipe events;
  deprecated axis events, wheels and unrelated input are not interpreted as
  touchpad gestures. Unsupported gestures reset recognition. The dependency
  enables the upstream modern-event feature without adding logging or a new
  engine. No unsafe block or driver has been added.
- Swipes accumulate unaccelerated normalized motion and commit once at normal
  end. Left goes next, right goes previous. Minimum travel is 100 normalized
  units; horizontal travel must exceed twice vertical travel. This rejects
  short, vertical and ambiguous diagonal motion. Swipe direction is independent
  of scroll direction. Desktop edges use the existing no-wrap refusal.
- Pinches use the last absolute scale since begin, rather than multiplying
  absolute updates together. Zoom commits at normal end, allowing cancellation
  to leave no zoom effect. This API does not provide a live zoom preview.
  Positive finite scales are accepted; zero, negative and nonfinite values retire
  the sequence. An unchanged pinch produces no intent.
- Scroll is continuous, preserving absent axes separately from zero-valued stop
  events. Upstream natural-scroll inversion is undone before applying the
  person's setting, so direction is applied exactly once.
- Overlapping starts, wrong-family updates or ends, cancellation, invalid
  arithmetic, reset and preference changes retire pending recognition. Orphan
  updates and repeated ends have no effect. Preferences cannot revive a gesture
  that began before they changed.
- `gesture_settings.rs` owns the settings shape, `gesture_files.rs` owns keeping
  it through `alo-kept`, and `gesture_refusals.rs` owns localized file refusals.
  Only differences from enabled gestures and natural scrolling are written,
  under format 1. The session supplies the path. Missing files mean shipped
  settings; malformed files refuse whole and cannot be silently overwritten.
  The existing `alo_kept::put_back_as_shipped::<Preferences>` is the explicit
  reset route. No new store, watcher, environment read or agent verb is added.
- File refusal sentences and translator notes are registered in the existing
  desktop vocabulary, already collected by `alo-saying`. No new vocabulary
  crate or image component requires registration.

These choices follow accepted ADRs
[0011](../../decisions/0011-the-base-is-rented-and-the-image-is-a-container.md),
[0009](../../decisions/0009-a-good-computer-without-the-agent.md), and
[0038](../../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md).
The upstream event API was inspected directly at
`https://raw.githubusercontent.com/Smithay/input.rs/v0.9.1/src/event/gesture.rs`.

## Acceptance evidence awaiting execution

Every row below names a newly added test file. All workspaces are `.` and all
crates are `alo-desktops`. These are proposed evidence, not passing results.

| Acceptance | Test target | Full test name |
|---|---|---|
| Scroll intent and invalid fields | gestures_from_a_touchpad | scroll_is_an_intent_and_nonfinite_input_is_refused |
| Pinch zoom intent and refusal | gestures_from_a_touchpad | pinch_zoom_uses_the_last_scale_and_cancelled_pinches_do_nothing |
| Three/four-finger desktop intents and refusal | gestures_from_a_touchpad | three_and_four_finger_swipes_switch_once_and_refuse_ambiguous_motion |
| Each gesture can be disabled | gestures_from_a_touchpad | each_gesture_can_be_disabled_without_disabling_the_others |
| Natural/traditional preference | gestures_from_a_touchpad | natural_and_traditional_scrolling_are_the_persons_setting |
| No gesture invokes the agent | gestures_from_a_touchpad | no_gesture_can_invoke_the_agent_and_broken_sequences_are_retired |
| Input-library field normalization and refused counts | gestures_from_a_touchpad | library_fields_refuse_other_finger_counts_and_do_not_double_invert_scroll |
| Preferences are kept by their owner | gesture_preferences_are_kept | each_preference_is_kept_and_missing_settings_are_shipped |
| Bad settings are refused without replacement | gesture_preferences_are_kept | malformed_settings_are_preserved_and_refusals_have_words |
| Swipes use desktop switching and refuse wrapping | swipes_use_the_desktop_switch | swipes_reach_neighbours_and_refuse_to_wrap |
| Overflow and separate-device state refuse completion | gestures_from_a_touchpad | overflowing_motion_and_another_device_cannot_finish_a_swipe |

For each row, the exact standalone evidence command is
`cargo test -p alo-desktops --test <test-target> <full-test-name> -- --exact`.
The supervisor should also run `cargo fmt --all`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test -p alo-desktops`,
within its nine serialized gates, including the Linux adapter compilation.

No cargo, rustc, formatter, test, build, Linux-tree sync or hardware check was
run by this worker. No build directory was created. Local validation is deferred
to the supervisor by the development-worker instructions, which take precedence
over the task prompt's request to run local gates. Source/API inspection only
was performed on Windows. No test is claimed to have passed.

## Limits and integration handover

The plan expressly forbids editing `alo-shell`. Accordingly, this change supplies
decisions for later shell routing and does not enable gestures on a running
compositor or claim physical touchpad acceptance. No physical events were
replayed; deterministic tests start at extracted library fields. Smooth preview
and application routing belong to the shell that consumes these decisions.

The proposed plan marks implementation complete with supervisor validation pending.
The completion mark and this code may publish only after all nine gates and
acceptance evidence pass. The handoff requests those checks; it does not certify them. Tasks 6 and 7 already follow it; no task was appended.
The four protected shared progress files were not edited. No commit or push was
performed. There is no legal decoder decision or signing-key work here.

Suggested conventional subject:
`feat(desktops): decide touchpad gestures and keep their preferences`

Suggested body: Decide touchpad scroll, pinch zoom and desktop swipes through a
closed intent type backed by unmodified libinput events. Persist independent
gesture toggles and scrolling direction through the owning crate. Cover
cancellation, malformed input, disabled gestures, corrupt preferences and
desktop-edge refusals; keep agent invocation outside the gesture type.

Every file touched:

```text
Cargo.lock
crates/alo-desktops/Cargo.toml
crates/alo-desktops/src/lib.rs
crates/alo-desktops/src/words.rs
crates/alo-desktops/src/gesture_events.rs
crates/alo-desktops/src/gesture_files.rs
crates/alo-desktops/src/gesture_refusals.rs
crates/alo-desktops/src/gesture_settings.rs
crates/alo-desktops/src/gestures.rs
crates/alo-desktops/src/libinput_gestures.rs
crates/alo-desktops/tests/gesture_preferences_are_kept.rs
crates/alo-desktops/tests/gestures_from_a_touchpad.rs
crates/alo-desktops/tests/swipes_use_the_desktop_switch.rs
docs/autonomy/v0-5-hands-on-the-desktop-plan.md
docs/autonomy/updates/touchpad-gesture-intents.md
```

Proposed changelog: Touchpad gesture decisions now include scroll, zoom and
desktop swipes, with separate switches and remembered scrolling direction.
Cancelled and malformed gestures do not switch desktops or zoom, and gestures
cannot invoke the agent. The shell integration and physical verification remain
with their owning workstream. No roadmap or release gate should be ticked on
the strength of this unexecuted evidence.
