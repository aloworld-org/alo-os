# Record a demonstration in alo Studio

This is the acceptance walk for integrating Studio with the current OS. The
November 9 recordings belong to the old branch and image. They do not attest
this integration. A passing unit suite alone does not pass this walk.

Use a private branch walk image under the existing KVM workflow. Install the
normal Studio Flatpak from the matching source commit, with verification
features disabled. Preserve the source revision, package checksum, runtime
revision, report, screenshots and decoded media measurements. Publish no image,
test account, signing key or disk. Leave unrelated VMs running.

## The recording flow

Run once with a screen and microphone disabled, and once with one window and
microphone enabled. Use a changing picture and irregular audio/visual markers
whose own continuity and timing are checked independently of the recorder.

1. Open the installed application. The microphone starts off.
2. Choose the source in the actual system picker. Cancelling grants nothing.
   Choosing a window never includes another application's pixels.
3. Select the explicit lightweight 360p, up-to-5-fps profile, with no countdown.
   Accept the performance warning explicitly if it appears.
4. Start and wait for actual recording. Microphone input exists only in the
   microphone-enabled case, and only while recording.
5. Record changing content, pause for at least five seconds, and compare two
   independently timed status snapshots. Saved elapsed time stays unchanged
   and microphone capture is inactive throughout the pause.
6. Resume, verify recording and microphone state, and record more changing
   content. Finish and wait for the completed Review state.
7. Rename the result, find it in the library, and export through alo's current
   SaveFile sheet and Documents portal. The exported video is byte-identical
   to the private library original. Cancelling export grants no destination.
8. Decode the saved container to EOS. Require exactly one video track and the
   expected number of audio tracks. Reject empty or unfinished recordings.

The selected 5 fps profile must deliver at least 4.5 fps, with strictly
increasing video timestamps and no uncovered gap longer than 400 ms. Do not
duplicate frames, synthesize audio, shift timestamps or weaken these limits to
obtain a pass. The motion source must visibly advance in the decoded frames.

Microphone PCM must have no gap or overlap longer than 40 ms. Audio and video
track endings must differ by at most 400 ms. Synchronization requires at least
ten independently matched irregular markers, at least 80 percent coverage of
each marker sequence, a median absolute offset at most 200 ms and a maximum
absolute offset at most 400 ms. A failed independent reference invalidates the
sync measurement; continuous recorded PCM alone does not establish sync.

## Agent control and permissions

With the normal approval policy, discover no Studio actions before a grant.
Create the grant in native Settings, read status, propose one typed action and
approve that exact action. Verify its actual application state and execution
record. Take access back and verify discovery and execution refuse immediately.
Repeat the boundary under the current full-access policy, including withdrawal.
Source selection and export retain their native consent surfaces at every level.
Exercise the real model repeatedly when changing what actions it is offered,
using the existing every-try infrastructure; a direct protocol call is not that
model check.

## Restore and report

Stop all recording inputs and fixtures, restore any test-only accessibility or
audio settings, remove private verification access, and resume any compiler work
paused with the owner's existing authorization. Report each uncompleted check
as failed or not reached, with its reason. Hardware microphone quality remains
not reached on a virtual audio fixture.

Studio's existing GTK interface is a development surface. The shared alo UI kit
currently supplies drawing and text; its application window, input, controls
and accessibility layers are separate unfinished prerequisites. This walk does
not claim that the UI migration or public market publication is complete.
