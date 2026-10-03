# A backgrounded process does not survive the WSL call that started it

**What happened.** The nine gates were started like this, so the loop could do other work
while they ran:

```
wsl -d Ubuntu -- bash -lc 'nohup bash /c/dev/setup/nine-gates.sh REPO BUILD > /root/out 2>&1 & echo started'
```

It printed `started`. Four minutes later:

```
-rw-r--r-- 1 root root 0 Oct  3 07:42 /root/t15-gates.out
pgrep -af "nine-gates|cargo"    (nothing)
```

Zero bytes, no process. The run never produced a line, and `nohup` did not save it.

**Why.** `wsl -- bash -lc` is one Windows process wrapping a Linux session. When the login
shell exits, the session goes, and the `&`-ed child goes with it — `nohup` protects a
process from `SIGHUP`, not from the session's container being torn down. So the gate run was
killed between the `echo started` and the first thing `cargo` would have written.

**What makes this worth a quirk rather than a footnote.** It fails *silently and
successfully*. The command printed `started`, exited 0, and left a file. Every signal
available at the moment of the call said the run was under way, and the only thing that
disagreed was the file still being empty some minutes later. A loop that read `started` as
the answer and came back for a verdict would wait for a verdict that nothing is computing —
and an empty output file reads the same as a run that has not got to its first line yet.

This is the same boundary that eats `$?` and `$(...)` in a `wsl -- bash -lc` string, seen
from the other side: there, a result cannot get out; here, a process cannot stay in.

**What to do instead.** Keep the Windows process alive for as long as the work. Run the
command in the foreground of its own `wsl` invocation and background *that* — in this loop,
the Bash tool's `run_in_background`, which holds a live Windows process for the call. The
gate run then survives, and its output file fills.

If something must genuinely outlive the call, it needs its own service or a scheduled task
on the Windows side, not `nohup` on the Linux side.

**How to tell which one you have.** Before believing a background run exists, ask the
machine for the process rather than for the file:

```
pgrep -af nine-gates
```

A file that is empty is ambiguous. A `pgrep` that finds nothing is not.

**The `-f` is not optional, and leaving it off inverts the answer.** Within the hour of
writing the paragraph above, this check was run as `pgrep -c nine-gates` on a run that was
alive and building. It returned `0`:

```
pgrep -af nine-gates   ->  4137566 bash /c/dev/setup/nine-gates.sh REPO BUILD
pgrep -c  nine-gates   ->  0
```

Both are the documented tool and both exit cleanly. `pgrep NAME` matches the *process
name*, which here is `bash` — the script is an argument to it, not the name of it. Only
`-f` matches the whole command line. So the form without `-f` answers a different question
from the one being asked, and answers it with a plausible number rather than an error.

This is worth the extra paragraph because the remedy above is otherwise a new trap: a
reader who takes away *ask pgrep* and not *ask pgrep with -f* gets a confident `0` for a
run that is healthy, and will go looking for a dead process that is compiling.

## It is one quirk with three symptoms, not three quirks

This page is named after the symptom that cost the time, but the cause is shared, and a
fix aimed at any one symptom leaves the other two standing:

```
a backgrounded process dies      the session is torn down when the login shell exits
$? is eaten                      the exit code never gets back out
$(...) substitutes to empty      the substitution never gets in
```

**The string handed to `wsl -- bash -lc '…'` does not reach bash intact, and the process
it starts does not outlive the call.** Everything above is one of those two sentences.

The third symptom is the nastiest, because it produces no error and no wrong value — it
produces *no value*, and the empty string lands in whatever was being built around it.
Reporting a gate run's progress with

```
wsl -d Ubuntu -- bash -lc 'echo "lines: $(grep -c "" /root/gates.out)"'
```

printed `lines: 0` over a file of 371 KB. Not a wrong count: no count, wearing the one
value that is indistinguishable from a run that has not written its first line yet.

**So the remedy is one remedy.** Put anything carrying a `# A backgrounded process does not survive the WSL call that started it

**What happened.** The nine gates were started like this, so the loop could do other work
while they ran:

```
wsl -d Ubuntu -- bash -lc 'nohup bash /c/dev/setup/nine-gates.sh REPO BUILD > /root/out 2>&1 & echo started'
```

It printed `started`. Four minutes later:

```
-rw-r--r-- 1 root root 0 Oct  3 07:42 /root/t15-gates.out
pgrep -af "nine-gates|cargo"    (nothing)
```

Zero bytes, no process. The run never produced a line, and `nohup` did not save it.

**Why.** `wsl -- bash -lc` is one Windows process wrapping a Linux session. When the login
shell exits, the session goes, and the `&`-ed child goes with it — `nohup` protects a
process from `SIGHUP`, not from the session's container being torn down. So the gate run was
killed between the `echo started` and the first thing `cargo` would have written.

**What makes this worth a quirk rather than a footnote.** It fails *silently and
successfully*. The command printed `started`, exited 0, and left a file. Every signal
available at the moment of the call said the run was under way, and the only thing that
disagreed was the file still being empty some minutes later. A loop that read `started` as
the answer and came back for a verdict would wait for a verdict that nothing is computing —
and an empty output file reads the same as a run that has not got to its first line yet.

This is the same boundary that eats `$?` and `$(...)` in a `wsl -- bash -lc` string, seen
from the other side: there, a result cannot get out; here, a process cannot stay in.

**What to do instead.** Keep the Windows process alive for as long as the work. Run the
command in the foreground of its own `wsl` invocation and background *that* — in this loop,
the Bash tool's `run_in_background`, which holds a live Windows process for the call. The
gate run then survives, and its output file fills.

If something must genuinely outlive the call, it needs its own service or a scheduled task
on the Windows side, not `nohup` on the Linux side.

**How to tell which one you have.** Before believing a background run exists, ask the
machine for the process rather than for the file:

```
pgrep -af nine-gates
```

A file that is empty is ambiguous. A `pgrep` that finds nothing is not.

**The `-f` is not optional, and leaving it off inverts the answer.** Within the hour of
writing the paragraph above, this check was run as `pgrep -c nine-gates` on a run that was
alive and building. It returned `0`:

```
pgrep -af nine-gates   ->  4137566 bash /c/dev/setup/nine-gates.sh REPO BUILD
pgrep -c  nine-gates   ->  0
```

Both are the documented tool and both exit cleanly. `pgrep NAME` matches the *process
name*, which here is `bash` — the script is an argument to it, not the name of it. Only
`-f` matches the whole command line. So the form without `-f` answers a different question
from the one being asked, and answers it with a plausible number rather than an error.

, a backtick or a background
`&` into a **script file**, and run the file. Then the quoting is bash's problem, the exit
code can be written somewhere it survives, and there is no login shell waiting to take the
work down with it. Reaching for a per-symptom fix — escaping the `# A backgrounded process does not survive the WSL call that started it

**What happened.** The nine gates were started like this, so the loop could do other work
while they ran:

```
wsl -d Ubuntu -- bash -lc 'nohup bash /c/dev/setup/nine-gates.sh REPO BUILD > /root/out 2>&1 & echo started'
```

It printed `started`. Four minutes later:

```
-rw-r--r-- 1 root root 0 Oct  3 07:42 /root/t15-gates.out
pgrep -af "nine-gates|cargo"    (nothing)
```

Zero bytes, no process. The run never produced a line, and `nohup` did not save it.

**Why.** `wsl -- bash -lc` is one Windows process wrapping a Linux session. When the login
shell exits, the session goes, and the `&`-ed child goes with it — `nohup` protects a
process from `SIGHUP`, not from the session's container being torn down. So the gate run was
killed between the `echo started` and the first thing `cargo` would have written.

**What makes this worth a quirk rather than a footnote.** It fails *silently and
successfully*. The command printed `started`, exited 0, and left a file. Every signal
available at the moment of the call said the run was under way, and the only thing that
disagreed was the file still being empty some minutes later. A loop that read `started` as
the answer and came back for a verdict would wait for a verdict that nothing is computing —
and an empty output file reads the same as a run that has not got to its first line yet.

This is the same boundary that eats `$?` and `$(...)` in a `wsl -- bash -lc` string, seen
from the other side: there, a result cannot get out; here, a process cannot stay in.

**What to do instead.** Keep the Windows process alive for as long as the work. Run the
command in the foreground of its own `wsl` invocation and background *that* — in this loop,
the Bash tool's `run_in_background`, which holds a live Windows process for the call. The
gate run then survives, and its output file fills.

If something must genuinely outlive the call, it needs its own service or a scheduled task
on the Windows side, not `nohup` on the Linux side.

**How to tell which one you have.** Before believing a background run exists, ask the
machine for the process rather than for the file:

```
pgrep -af nine-gates
```

A file that is empty is ambiguous. A `pgrep` that finds nothing is not.

**The `-f` is not optional, and leaving it off inverts the answer.** Within the hour of
writing the paragraph above, this check was run as `pgrep -c nine-gates` on a run that was
alive and building. It returned `0`:

```
pgrep -af nine-gates   ->  4137566 bash /c/dev/setup/nine-gates.sh REPO BUILD
pgrep -c  nine-gates   ->  0
```

Both are the documented tool and both exit cleanly. `pgrep NAME` matches the *process
name*, which here is `bash` — the script is an argument to it, not the name of it. Only
`-f` matches the whole command line. So the form without `-f` answers a different question
from the one being asked, and answers it with a plausible number rather than an error.

, capturing the code
in a variable, adding `nohup` — fixes the one in front of you and leaves the others, which
is how this boundary gets rediscovered three times.
