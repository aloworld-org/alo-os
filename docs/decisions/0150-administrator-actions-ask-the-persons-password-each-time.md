# ADR 0150 — Administrator actions ask the person's password each time

**Status:** **accepted, 2026-11-09, by the owner.** It builds decision 2 of
[ADR 0149](0149-full-access-is-everything-the-person-can-do.md) and settles
what that ADR left open under *Not decided here* (how the password is asked
for). It **partly amends
[ADR 0024](0024-what-a-person-signs-in-at.md)**: the person's system password
is now kept the same as their alo password (below, *What it reverses in ADR
0024*).
**Date:** 2026-11-09

## The owner's words

On 2026-11-08 (ADR 0149), asked whether full access should include
administrator actions: **"Ask my password each time."** Offered how it is asked
for: **"The system's own window (Recommended)."**

On 2026-11-09, told that the password a person signs in with is kept only in
alo's own account file, that the system checks administrator rights against its
own list (`/etc/shadow`), where the person's password is locked, and that the
person's account is not an administrator, the owner answered three questions:

- *Where the password is checked*: **"Keep both in step (Recommended)"**. alo
  also writes the person's password into the system's password list whenever
  it is set; one password everywhere.
- *Whether the person is an administrator*: **"New and existing
  (Recommended)"**. The person's account is an administrator on every machine,
  including ones already in use once they update. alo's own agent account is
  always refused.
- *Which window asks*: **"Yes, alo's desktop draws it (Recommended)"**. The
  password sheet is drawn by alo's own desktop, never by the assistant or an
  application, and alo never sees the password.

## What it changes for the person

**When alo needs to do something only an administrator can do**, such as
installing system software or changing a system setting, a password sheet
appears on their screen. It shows the exact command and asks for the password
they sign in with. Typing it and pressing Enter runs that one command. Escape
cancels it. The next command asks again: a password is never remembered.

- It is the **same password, or PIN, they sign in with**. Every sign-in hands
  it to the system's list, so it stays the same everywhere, including for
  `sudo` in their own terminal. **That last clause was false when it was
  written**: the desktop's unit refused every `sudo` in the person's terminal
  until 2026-11-09 (below, *The person's terminal*). That change takes the
  refusal away; the next boot walk is where `sudo` working is measured.
- A **wrong password** leaves the sheet up and says so. A password that
  **cannot be checked here** says why instead of calling it wrong.
- If they **cancel**, or nothing could ask them (nobody signed in at the
  screen), alo says so in one plain sentence. **Nothing is ever run another
  way instead.**
- The **record** says which commands were run as administrator, and whether
  the person gave their password, cancelled, or was not accepted.

## The decision

1. **The person's account is an administrator** (`wheel`) on every machine
   (`image/usr/lib/sysusers.d/alo.conf`: `m alo wheel`). `systemd-sysusers`
   applies it at every start, so a machine already in use becomes one at the
   first start after it updates. The agent's own login, `alo-agent`, is
   refused every polkit action in every session
   (`image/usr/share/polkit-1/rules.d/49-alo-administrator.rules`).
2. **The system's password is kept the same as the person's, at every
   sign-in.** After a sign-in succeeds, the sign-in screen sends the person's
   number and the password just verified to `alo-keeping-the-system-password`
   (a root-only socket), which writes it with `chpasswd -c YESCRYPT`. It holds
   `CAP_DAC_OVERRIDE` and nothing else, and it writes whether it did to
   `/run/alo-the-system-password/<number>`. Every sign-in, not only when an
   account is made or a password changes, because that is what keeps them the
   same through every road a password can change (*Machines already in use*,
   below). A keeper that fails never stops a sign-in.
3. **The password sheet is polkit's authentication agent, drawn by alo's
   desktop.** `alo-desktop` registers `org.freedesktop.PolicyKit1.AuthenticationAgent`
   for its own session. When polkit needs an administrator's password from
   anything on the machine, the desktop draws the sheet
   (`alo_shell::the_password_sheet`) in the approval surface's layer, above
   every window and below the egress indicator. While it is up, **every key is
   the sheet's**. It asks only for its own person's password. What they type
   goes to polkit's own helper, `polkit-agent-helper-1`, through
   `alo-checking-a-password`, which systemd starts as root for one connection
   and which runs the helper **for the login the kernel says is calling**. The
   helper asks PAM and tells polkit. The desktop hears only *SUCCESS* or
   *FAILURE*.
4. **A command alo writes as `sudo …` or `pkexec …` runs as administrator
   through `alo-running-as-administrator`.** The agent service holds no
   capability and no new privileges, so neither word can do anything inside
   it, and that stays so. Such a line goes to the runner, which checks that the
   caller is the agent service, that the line is one command, and that the
   person is an administrator and signed in at a screen. It then asks polkit
   `CheckAuthorization` for `org.aloworld.alo.run-as-administrator` **about
   the person's own session**, so the sheet appears in front of them. That
   action is `auth_admin` at every setting and never `auth_admin_keep`, so
   every command asks again. The runner runs the command, as root and bounded
   like every command alo runs, only on polkit's yes.
5. **Levels are unchanged.** At full access the password is the only question.
   At *ask me each time* the OK comes first and the password after it. At the
   sealed box nothing runs.
6. **Every refusal is said, and none is a fallback.** Cancelled, not accepted,
   nothing could ask, not an administrator, no answer in five minutes, could
   not ask: each is a sentence the person reads and a value in the record
   (`as_an_administrator`, `docs/contracts/record-file.md`).

## The person's terminal

*Added 2026-11-09, after a boot walk of an image built from `main` at
`90babddc`.*

**`sudo` in the person's own terminal was refused**: *The "no new privileges"
flag is set, which prevents sudo from running as root.* The terminal is drawn
inside alo's desktop, and the person's shell is started by the desktop's own
process (`alo-shell`, through `alo_terminal::AShellOnAPseudoterminal`), so it
carries every limit the desktop's unit sets. `alo-desktop@.service` set
`NoNewPrivileges=yes` and an empty `CapabilityBoundingSet=`. Measured the same
day in `localhost/alo-os:dev`, a process of the person's under each line:
`NoNewPrivileges=yes` gives the walk's message, the empty bounding set gives
*unable to change to root gid*, and with neither `sudo` answered `root`.

**Decided: the desktop's unit sets neither.** CLAUDE.md, law 2: *the law binds
the agent, never the person: alo OS ships a terminal, because a system that
does not trust its owner with a shell is a toy.* A limit on the desktop is a
limit on the owner's shell, and the owner's shell may do what the owner's
account may do. The desktop is still handed no capability
(`AmbientCapabilities=` stays empty) and runs as the person.

**The agent's boundary does not move.** The agent is not in the desktop: it
runs in `alo-asking@` and asks `alo-agentd`, which runs every command alo
runs, and both keep `NoNewPrivileges=yes`. So nothing alo runs can use `sudo`
or another setuid program, and what alo runs as administrator still goes only
through `alo-running-as-administrator` and the person's password, every time.
`sudo`'s own memory of a password is per terminal on this base, so a password
the person gave in their terminal is not one alo's commands can borrow.

**Considered and not chosen: starting the shell outside the desktop**, as a
unit in the person's own manager with the terminal handed to it. It would keep
the desktop locked down, but the terminal reads the shell's process for its
folder, the command in front and why it is waiting, and every one of those is
built on the shell being the desktop's child. It is a rebuild of the terminal,
not a fix, and the locked-down desktop it would keep protects nothing the
person's own account could not already do.

**How this is checked, on every build**:
`crates/alo-sessiond/tests/the_units_this_starts_are_on_the_image.rs`,
`the_persons_terminal_may_use_sudo_and_the_agent_may_not`, fails if the
desktop's unit sets either line again, or if `alo-agentd.service` or
`alo-asking@.service` stops setting `NoNewPrivileges=yes`. That `sudo` works in
the terminal on a booted machine is **not** checked by anything here; it is a
step on the next boot walk.

## Machines already in use

Before the update, such a machine's system password is locked (`!*`, what
`sysusers` leaves) and the person is not in `wheel`. After it:

- **The first start after the update** puts them in `wheel`.
- **The first sign-in after that** writes their password into the system's
  list. Updates apply at the next start (ADR 0011), so a person cannot reach
  their desktop on the new image without signing in first.
- **Between the two, nothing can ask for the password.** The sheet exists only
  inside a signed-in desktop, so there is no moment when a locked system
  password meets a password sheet. A `sudo` from a console nobody signed into
  is refused, which is what it did before.
- **If the keeper fails at that sign-in**, the state file says so, and a
  password typed into the sheet that polkit then refuses is reported as *could
  not be given to the system when you signed in*, never as *wrong*.

**Changing a password, and setting a forgotten one again**, have no screen on
this image yet (`Accounts::password_set_again` has no caller). Whichever change
builds them keeps the two the same by the same road. Until then, the next
sign-in does it anyway.

**A password the person changes with `passwd` in their own terminal** is
replaced by their alo password at the next sign-in: one password everywhere is
what was chosen, and alo's sign-in is the one this machine checks first.

## What it reverses in ADR 0024

ADR 0024 said: *"The person's system password is not the alo account. `alo` is
created by `sysusers` with no password set; if a design lets a Unix password
sign somebody in, then `alo-accounts` is a decoration over `/etc/shadow`."*

**The first sentence is reversed**: the system password is now the person's
alo password, written there by alo. **The rest stands.** Nobody is signed in
by the system password. `alo-accounts` remains the only authenticator at the
sign-in screen (Option B), and `/etc/shadow` is written from it, never read by
it. The PAM-module road ADR 0024 weighed and set aside, with its `unsafe` on
the authentication path, is still not taken.

## Considered and not chosen

- **An upstream polkit agent** (a GTK one) in the desktop. It would bring a
  second toolkit and a password prompt in somebody else's words and
  vocabulary, which ADR 0002 and ADR 0024 already refused for the sign-in
  screen. The owner chose alo's desktop.
- **Lifting `NoNewPrivileges` from the agent service so `sudo` and `pkexec`
  work inside it.** It would give every command alo runs every setuid program
  on the machine, and the record could only guess from a command's own
  messages whether a password was given. The runner knows, from polkit's own
  answer.
- **A PAM module that checks alo's own account file.** It is the road ADR 0024
  set aside (a C ABI, so `unsafe`, on the authentication path). The owner chose
  to keep the two the same instead. Recorded for whoever reopens this: PAM's
  `pam_exec` with `expose_authtok` could hand the password to a Rust program
  without `unsafe`. That was not put to the owner, and it is a new fact if
  anybody does.

## What is not decided here

- **The sheet's design.** The design file has no frame for it. Until it does,
  the sheet is the approval surface's panel, with the same measures, palette
  and layer (`crates/alo-shell/src/the_password_sheet_drawn.rs` records the
  departure).
- **More than one person on a machine** (ADR 0118). Every piece here is
  written per person by number, but the image is still wired to one.

## How this is checked

These run on every build, as tests:

- **An administrator action without the password is refused.** The agent
  service, given a `sudo` line at full access with a runner that answers
  *cancelled*, says so, runs nothing as the person (the folder it would have
  made does not exist), and writes an entry stamped `they-cancelled`. With no
  runner at all it says *could not ask* and runs nothing
  (`crates/alo-agentd/src/running_a_command_tests.rs`).
- **One yes runs it once.** In the runner's order, every refusal (not the
  agent service, not one command, not an administrator, nobody at a screen,
  and each of polkit's noes) runs nothing, and a yes runs exactly once
  (`crates/alo-administrator/src/running_tests.rs`). One OK at *ask me each
  time* is one run, and the number spent (`crates/alo-turn/src/a_command_as_administrator_tests.rs`).
- **polkit's answers are read as the person's**: authorised, no agent to ask
  (a challenge), dismissed and not accepted
  (`crates/alo-administrator/src/what_polkit_said.rs`).
- **The sheet**: one Enter sends once, Escape cancels, a wrong password leaves
  it up, a password that cannot be checked says why, a question polkit takes
  back takes it down, and a sheet that cannot be drawn cancels rather than
  hold every key invisibly (`crates/alo-shell/src/the_password_sheet_tests.rs`).
  Keys at the seat reach it and no client (`a_key_reaches_the_password_sheet.rs`).
  How long the password is, is never drawn (`the_password_sheet_drawn_tests.rs`).
- **The desktop's conversation with the helper**: a wrong password then the
  right one, a password the system's list was not given, no checker, and a
  cancel (`crates/alo-desktop/src/asking_for_the_persons_password_tests.rs`).
  Only one hidden question is ever answered with the password
  (`crates/alo-administrator/src/the_helpers_conversation.rs`).
- **The sign-in screen hands over only a password it just verified**, with the
  person's number. A wrong one, an unknown name, or a number that is not this
  machine's person hands over nothing
  (`crates/alo-greeting/tests/the_system_password_is_kept_in_step.rs`).
- **The image holds the pieces to each other**: the rule refuses the agent,
  the action asks every time, the person is in `wheel`, and the three sockets
  are enabled (`crates/alo-image/tests/administrator_actions_ask_every_time.rs`).

**Measured once, by hand, on 2026-11-09, in `localhost/alo-os:dev`.** These will
not run again unless somebody repeats them:

- polkit is 126-3.fc42.
- `/usr/lib/polkit-1/polkit-agent-helper-1` is setuid root, and
  `/usr/lib/pam.d/polkit-1` exists.
- `wheel` is in `/etc/group` with no members, and after `systemd-sysusers`
  read `m alo wheel`, `id alo` listed it.
- The person's shadow entry is `!*`.
- `chpasswd -c YESCRYPT` wrote a `$y$` hash for `1234` holding only
  `CAP_DAC_OVERRIDE`, and failed holding nothing.

The helper's conversation and polkit's three answers were read from polkit
126's source the same day.

**Not checked by anything yet, and not walked on a booted machine:**

- the sheet appearing over a real desktop;
- the helper accepting the right password through `alo-checking-a-password`
  under the base's SELinux policy;
- the keeper writing `/etc/shadow` under SELinux.

Each is a step on the next boot walk.

**What the 2026-11-09 boot walk found, and what was fixed.** polkit took the
desktop's registration (*the password sheet is polkit's agent for session 3*).
Asking polkit about the person's **session** then killed polkitd 126, whose
rules engine aborts on any subject but a process or a bus name; the runner now
asks about the session's leader
(`docs/quirks/polkit-aborts-on-a-session-subject.md`). Behind that was a
second fault no walk had reached yet: polkit refused a right password as *No
session for cookie*, because the helper ran with a real number of 0
(`docs/quirks/polkit-takes-the-helpers-answer-only-from-the-agents-owner.md`).
**Measured once, by hand, the same day**, in `localhost/alo-os:dev` run as a
container with polkitd 126, a session agent and the runner's own code: a right
password was accepted, a wrong one was not, a session that is not the person's
or that logind does not know asks nobody, and polkitd stayed up throughout.
