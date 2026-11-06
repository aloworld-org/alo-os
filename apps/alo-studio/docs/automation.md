# Controlling alo Studio

Studio exposes `org.aloworld.Studio.Recorder1` at
`/org/aloworld/Studio/Recorder` on the session-bus name `org.aloworld.Studio`.
The interface is independent of translated labels and keyboard focus. Its
introspection declaration is `data/org.aloworld.Studio.Recorder1.xml`.

Every command uses the same lifecycle and visible settings as the native UI.
An invalid state returns `org.aloworld.Studio.Error.InvalidState`, never a
successful no-op. Invalid settings return the standard D-Bus `InvalidArgs`
error. Pause and Resume are separate operations; repeating one cannot toggle
the recording in the opposite direction. A completed or absent recording cannot
be finished again. No command takes code, a source identifier or an export path.

| Method | Meaning |
|---|---|
| `GetStatus() -> s` | Read the bounded JSON session status without starting a device. |
| `NewRecording()` | Return to the workspace, release the previous source and turn microphone off. |
| `ChooseScreen()`, `ChooseWindow()` | Open the system picker for the person to choose and approve a source. |
| `SetQuality(s)` | Choose `full_hd`, `hd`, `original`, or `lightweight`. |
| `SetCountdown(x)` | Choose 0, 3, or 5 seconds. |
| `EnableMicrophone()`, `DisableMicrophone()` | Set the per-recording microphone option, while configuration is allowed. |
| `Start()` | Begin the selected source's performance check and countdown. |
| `Pause()`, `Resume()` | Pause or resume the actual recording and microphone together. |
| `Finish()` | Stop capture and begin draining and validating the saved media. |
| `CancelCountdown()` | Cancel a countdown before devices start recording. |
| `ShowLibrary()` | Open the private recording library. |
| `ReviewRecording(s)` | Open the completed recording with this library identifier. |
| `RenameRecording(s)` | Persist a name for the recording being reviewed. |
| `ExportRecording()` | Open the native Save a copy dialog for the reviewed recording. |

Choosing, starting, finishing and exporting are asynchronous. A successful
method reply means the request was accepted, not that consent or media validation
has completed. Poll `GetStatus` for the next step. Source consent still requires
the system picker; export still requires the system file chooser. A cancelled
picker grants nothing. Start refuses until the selected source has a real preview.
The performance warning still requires an explicit choice on a slow device.

Status version 1 includes `phase`, `source_kind`, `quality`, `countdown_seconds`,
`microphone_enabled`, `microphone_active`, `elapsed_ms`, `recording_id`,
`preview_ready`, `export_pending` and the `can_start`, `can_pause`, `can_resume`
and `can_finish` flags. `recording_id` may identify an unfinished recording;
only `phase: review` establishes a completed, validated library entry. The phases
are `ready`, `choosing`, `selected`, `checking`, `countdown`, `preparing`,
`recording`, `paused`, `finishing` and `review`. Changes to this version are
additive. Microphone enabled is a setting; microphone active is actual recording
state and becomes false during Pause.

A typical agent flow opens Studio, asks for a source, waits for the person's
picker answer and `can_start`, sets quality/countdown/microphone, calls Start,
waits for `recording`, then uses Pause, Resume and Finish. It waits for Review
before renaming, reviewing or requesting export. Opening the picker for a
different source kind resets the microphone choice, just as the UI does.

ask-alo and other agents must reach these methods through the OS's application
adapter and the person's application grant. A read still needs that grant;
changes follow the person's approval policy. The application interface is not
a second consent mechanism and cannot select a hidden source or silently save
outside the private library.
