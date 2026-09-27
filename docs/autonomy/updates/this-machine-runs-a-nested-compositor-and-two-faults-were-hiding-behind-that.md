# This machine runs a nested compositor, and two faults were hiding behind that

**2026-09-27.** The question was whether the Windows Server machine that gates
this repository can run a nested compositor — `weston --backend=headless` with
llvmpipe, software rendering, no GPU, the way the Mac's Lima VM does. **It can,
and every sub-mode of `the_nested_fixtures` now passes on it.** Nine steps of the
shell were read back off the frames they drew.

Getting there was not installing weston. It was finding two faults that had been
making this machine *fail* those fixtures rather than skip them — so `main` has
been red here, and the failure looked like a machine that could not do it.

## The answer, plainly

Yes. weston 13.0.0, headless backend, Mesa 25.2.8,
`GL_RENDERER: llvmpipe (LLVM 20.1.2, 256 bits)`, 19 Wayland globals including
`wl_compositor` v5, `xdg_wm_base` v5, `wl_seat` and `wl_output` v4 reporting
1280x800 at 60 Hz. GLES and SHM clients both connect and hold.

And the fixtures that matter:

```
offscreen: passed
9 steps walked on a nested parent, each read back off the frame it drew
walk: passed
grabs: passed
pointer-release-grabs: passed
keyboard-grabs: passed
keyboard-release-grabs: passed
client: passed
client-everything: passed
```

| step the walk drew | pixels painted at 1366x768 |
|---|---|
| the sign-in screen, before any account is chosen | 1,049,088 |
| a name typed at the sign-in screen | 1,049,088 |
| the screen divided between two windows | 512 |
| the same two windows, zoomed out to 40 per cent | 84 |
| a second display docked, and the desktop after it | 127,038 |
| a capture of the screen, with a blur marked on it | 241,662 |
| a notification arrives on the desktop | 173,118 |
| the screen is locked | 1,049,088 |
| a key is pressed at the locked screen | 1,049,088 |

So compositor work can be taken on this machine, and the canvas stops being one
machine's bottleneck. What it still is not is a photograph: a headless parent
composites to nothing, so every *physical display unverified* in
`docs/autonomy/COMPOSITOR.md` stands and no owed screenshot of a real display is
discharged.

## Fault one: a variable is a claim, a socket is a fact

`a_parent` in `crates/alo-shell/tests/the_nested_fixtures.rs` asked whether
`WAYLAND_DISPLAY` was set and non-empty, and took that as a parent. On this
machine that is always true and always wrong:

- WSL puts `WAYLAND_DISPLAY=wayland-0` and `XDG_RUNTIME_DIR=/run/user/0` into
  every shell, whoever it is;
- WSLg's socket is really at `/mnt/wslg/runtime-dir/wayland-0`, and WSL links it
  into `/run/user/1000` for the default user — **never into `/run/user/0`**;
- the gate runs as root.

So the name pointed at nothing, `a_parent` returned *already there*, weston was
never looked for even once it was installed, and all eight sub-modes failed with
`nested graphics initialization: No such file or directory (os error 2)`.

Proved before it was fixed: the same command with `env -u WAYLAND_DISPLAY` started
weston and seven of the eight passed immediately.

It now resolves the name the way a Wayland client does — an absolute
`WAYLAND_DISPLAY` is the path, anything else is relative to `XDG_RUNTIME_DIR` —
and counts it only if there is really a socket there. A name that reaches nothing
falls through to starting weston, and the refusal left over says which of the two
was missing rather than naming only one.

**This is the third time today the same shape has come up.** A plan's status word
against its own `**Done,` marker; a converted document's declared fonts against
the text actually set in them; an environment variable against the socket it
names. Each time, something that *says* a thing was being trusted over something
that *is* it.

## Fault two: a deadline stops a hang, it does not measure performance

With a parent found, seven of eight passed and `client-everything` failed:

```
thread '<unnamed>' panicked at nested_client_check.rs:145:
no callback after GLES submission
```

Three runs out of three, deterministically, alone, with a weston of its own and
nothing else competing — so not contention. Then both deadlines were raised out
of the way and the run measured:

| run | two frame callbacks arrived after |
|---|---|
| `client` | **52.7 ms** |
| `client-everything` | **5.66 s** |

Nothing was broken. `nested_client_check`'s loop does the reader's twelve EGL page
submissions and the control strip's eight **in the same loop iteration**, as soon
as the client's surface is mapped, and only then calls `Server::render`, which is
what produces the client's frame callbacks. The client's wait was five seconds. On
a GPU that fits inside it. On llvmpipe it misses by about thirteen per cent.

The fix is `Flags::patience()` — five seconds for the plain run, **unchanged**, so
nothing about the fast path is loosened, and sixty for a run that asks the
compositor for the reader or the strip, which is an order of magnitude above the
measurement. `Flags::whole_run()` is raised in step, because the client's wait
sits inside it and one raised without the other would fail in the loop and say
something less true about why. Both carry the measurement in their doc comments.

The price is that a real regression there reports in a minute rather than in five
seconds. That is the right way round: a number tight enough to measure performance
turns a software rasteriser into a failing gate, and every machine this project
has is a software rasteriser until the certified one exists.

## What was tried and did not work, so nobody repeats it

`weston-screenshooter` does not work here. Under the GL renderer it aborts on
`width > 0` in `screenshot_create_shm_buffer`; under `--renderer=pixman` it exits
cleanly and writes no file. It does not matter — the shell reads its own
framebuffer back, which is stronger evidence than a screenshot tool's — but an
hour can go into it.

Installing weston pulled in ffmpeg and gstreamer **libraries** as dependencies of
its RDP and remoting backends (`libavcodec`, `libswscale`, `libpocketsphinx`).
Nothing was started and no media stack runs; this is recorded because *do not
install a media stack in WSL* is a standing instruction and a reader of the
package list should know why those names are there.

## The gate

`crates/alo-shell`: `cargo fmt --check`, `clippy -D warnings`, and its tests
including all eight nested sub-modes against a real weston — `GATES=0` on this
machine. Then the workspace.

`crates/alo-shell` is the shell lane's and `the_nested_fixtures` is its file. Both
changes were taken because `main` was red on the machine that gates, which blocks
every lane on it, and because the fixtures' own header states the rule these
faults broke: *a gate only one machine can pass is how these fixtures ended up
unrun in the first place.*
