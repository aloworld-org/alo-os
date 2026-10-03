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

**So the remedy is one remedy.** Put anything carrying a `$`, a backtick or a background
`&` into a **script file**, and run the file. Then the quoting is bash's problem, the exit
code can be written somewhere it survives, and there is no login shell waiting to take the
work down with it. Reaching for a per-symptom fix — escaping the `$`, capturing the code
in a variable, adding `nohup` — fixes the one in front of you and leaves the others, which
is how this boundary gets rediscovered three times.

## The script-file remedy runs into a second boundary, which is not WSL's

Taking the advice above fails on its first use from Git Bash:

```
wsl -d Ubuntu -- bash /mnt/c/Users/…/ask-next.sh
  bash: C:/Program Files/Git/mnt/c/Users/…/ask-next.sh: No such file or directory
```

**Git Bash rewrote the argument before `wsl` ever saw it.** MSYS path conversion treats a
leading `/` as a Unix path and maps it into the Git installation, so `wsl` is asked for a
script that was never written. Nothing in the error names the conversion: it names a file
whose path contains `C:/Program Files/Git`, which reads like somebody's typo rather than
like a translation.

```
MSYS_NO_PATHCONV=1 wsl -d Ubuntu -- bash /mnt/c/dev/thing.sh
```

**Every absolute-path argument is rewritten, not just the script**, and you are shown
whichever one is reached first. Measured on two machines with different habits:

```text
wsl … -- bash /mnt/c/…/x.sh        `bash` is unqualified and found on PATH, so the
                                   SCRIPT is the first absolute path:
                                   bash: C:/Program Files/Git/mnt/c/…/x.sh: not found

wsl … --exec /bin/bash -c … /root/x.sh
                                   the INTERPRETER is the first absolute path:
                                   ERROR: execvpe(C:/Program Files/Git/usr/bin/bash) failed
```

So the rule is not *a `/mnt/...` argument is rewritten*. It is **every absolute-path
argument is rewritten, and the one you are shown is the one that was reached first** —
so a reader who checks only their script path can leave `/bin/bash` exposed and meet a
different error with the same cause. `MSYS_NO_PATHCONV=1` sufficed on both machines;
`MSYS2_ARG_CONV_EXCL='*'` may be belt and braces rather than necessary.

**That is the second correction this page has needed, and both were found by following
it rather than by reading it.** The `pgrep` paragraph is the first: *ask the process, not
the filesystem* is right, and without the `-f` it answers `0` about a healthy run. A
remedy stated without the flag that makes it work is a new instance of the fault.

## This page was corrupted by a patch script, and every cheap check passed it

The two sentences above that quote `$` as a character to watch for were each
replaced by **2,984 characters of this document's own text** the moment they were
written. The page reached `main` that way and stayed through nine gates and CI, because
nothing in the workspace reads it.

**The cause is not this page's subject, and the first diagnosis blamed it wrongly.** It
looked like a backtick opening command substitution — the hazard these notes are full
of. It was JavaScript:

```text
haystack.replace(anchor, "a literal " + DOLLAR + " here")
  -> "…-a literal `START-XXXXXXXXXXXXXXXXXXXX- here-END"
haystack.replace(anchor, () => "a literal " + DOLLAR + " here")
  -> "…-a literal `$` here-END"
```

In `String.prototype.replace`, a replacement **string** treats `$&`, `$$`, and the
dollar-backtick and dollar-apostrophe pairs as patterns. Dollar-backtick means *insert
everything before the match*. Any prose quoting a dollar immediately before a backtick
therefore inserts the whole preceding document. **A replacement function is taken
literally**, so `() => text` is the form to use for anything but a known-safe literal.

**Python has the same trap and it fails loudly**, which is worth knowing before porting a
patch script either way. Measured on the development PC:

```text
str.replace          literal — a dollar-backtick in the replacement stays put
re.sub               raises `bad escape \d` on a Windows path in the replacement
```

So the silent-corruption form of this is JavaScript's alone: `str.replace` does not
interpret the replacement at all, and `re.sub` refuses rather than guessing. **A tool that
errors is safe; a tool that substitutes is not**, and the two are one keyword apart in
either language.

**And it was found by trying to edit the file again, not by any check run on it.** Zero
control bytes, balanced emphasis, nine green gates: every cheap measurement passed over
127 lines of duplicated prose sitting in the middle of a sentence. What finds this class
of fault is reading the paragraph — and what would have found it mechanically is asking
whether a patch that should add 40 lines added 40 lines.
