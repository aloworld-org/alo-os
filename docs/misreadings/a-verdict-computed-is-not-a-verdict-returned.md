# A verdict computed is not a verdict returned

**Concluded, every time a gate was read:** *the gate exited 0, so it ran and
passed.*

**True:** `gate3.sh` captured `verdict=${PIPESTATUS[0]}` at line 135, printed
`GATES=$verdict`, and then **ended on an `echo`** — so it exited with that
echo's status, which is always 0. The verdict was computed, printed and
dropped. Its own final line was a probe proving exit codes propagate.

**And the fix had the same fault.** `gate-and-save.sh`, written the same day to
stop a gate reporting a refusal as a success, **ended on a `sed`**.

**The mechanism.** A script's exit status is its last command's, and a script
that ends on output ends on a success. The discipline that covered it — *the
verdict is the `GATES=` line, never the exit code* — worked, which is why the
tool was never fixed.

**The cure.** A script that decides something ends on `exit "$verdict"`, and a
missing verdict line is a failure rather than a pass. **And a fix tells you
about one file and nothing about its siblings** — audit the rest the same hour.
