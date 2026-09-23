# The session this machine cannot make

**What was asked.** Kernel-enforcement task 10 says its first step is to
establish *whether a session of the kind a person really has can be started and
ended here at all*, and only then whether its three cases can be observed. The
task had sat **scheduled** on that question since it was written. It was put to
the machine on 2026-09-22 rather than reasoned about further.

**How.** `C:\dev\setup\can-a-real-session-end-here.sh` on the third PC
(`AGAI01`, Ubuntu 24.04.5 in WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`):
make a throwaway person, open a session as them through `pam_systemd` with
`su -`, ask `logind` what it made while the session is open, ask again after it
ends, and remove everything it created on every exit path.

**What `logind` answered.**

| | |
|---|---|
| the session already present | `class=user` — but it is WSL's own, not a login |
| the session `su -` opened | registered as `c2`, **`class=background`** |
| its state while open | `active` |
| its runtime directory | present |
| its session bus | **present** |
| after it ended | no sessions for that person, `linger=no` |

**The answer is no, and two parts of it are worth separating.**

`su` **does** reach `logind` here. A session is registered, it is active, and a
session bus exists — which the plan's earlier text doubted, and which matters
because the held-handle half needs a Secret Service on the person's session bus.

But what `su` registers is **`background`**, not the **`user`** class a person's
login has. The three cases are about what a *person's* logout does to a
credential, and a background session ending is not that. Nothing on this machine
can be measured into that answer.

**INCONCLUSIVE.** `/run/user/1001` was still present immediately after the
session ended and gone when asked again later. The probe had removed the
throwaway person in between, so whether the logout took the directory or
`userdel` did is **not distinguished**. It is evidence for neither reading, and
it is recorded here so nobody cites it as though it were.

**A correction to this measurement, made against its own output.** The first
version of the probe asked `loginctl list-sessions` *after* `su` had exited and
concluded that no session had ever been registered. It was looking after the
thing it was measuring had ended — the captured output from inside the session
already showed `XDG_SESSION_ID=c2` and a runtime directory. The script says so
in its own header now, because a probe that reports the opposite of what it
captured is worse than no probe.

**What would answer it, and who can.** A machine that registers a `user`-class
login — a real greeter or a `getty`. The **development PC** inside a KVM guest
with a real login, its hardware virtualisation measured on 2026-09-20; or the
owner's **certified laptop** once alo OS is installed on it. Not this machine,
and **not by installing `sshd`**, which was the earlier guess: the class of the
session is what matters, and a listener does not change it.

**What this change does not do.** It does not tick anything, does not weaken the
task, and does not touch `crates/alo-secrets`. The three cases measured on
2026-09-10 stand exactly as they were. The status moves from *scheduled* to
*blocked* so that the plan says what is true — a task nobody can run here reads
as waiting for a machine, not as waiting for a turn.
