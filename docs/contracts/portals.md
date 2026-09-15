# Portals — which this machine answers, and which it does not yet

**Status:** v0.5, additive. **Owner:** `crates/alo-portals` (`serving`), with the
Secret portal's keyring in `crates/alo-secrets` and the appearance it reads in
`crates/alo-appearance`. **Decisions:**
[ADR 0005](../decisions/0005-applications-are-sandboxed-and-ask.md) (applications
are sandboxed and ask),
[ADR 0040](../decisions/0040-what-an-applications-grant-is-over.md) (what an
application's grant is over). **Held by:**
`crates/alo-portals/tests/the_portal_backend_answers_on_a_real_bus.rs`, which
fails when a portal is answered on the bus and not listed here as answered, or
listed here as answered and not answered; and
`crates/alo-portals/tests/the_settings_portal_answers_appearance.rs`, which
fails when the settings row stops naming what it answers.

An application on alo OS reaches the system through the XDG Desktop Portal
interfaces, `org.freedesktop.portal.*`, on the person's session bus. alo OS
answers them itself: the backend owns `org.freedesktop.portal.Desktop` and
serves `/org/freedesktop/portal/desktop`. An application written for any Linux
desktop needs no change to be answered, and knows nothing about alo OS.

**Every request is a grant.** A request is answered from the grants the person
made, on the same list an agent's verbs are judged against. No portal on this
machine answers *yes* because nobody said no.

## Who is asking

The application is named by its **sandbox**, never by the request. The backend
asks the bus which process sent the request (`GetConnectionCredentials`) and reads
`/proc/<pid>/root/.flatpak-info`, the file Flatpak writes at the root of every
sandbox, for `name` under `[Application]`. A program outside a sandbox has no
such file, is answered with the refusal response, and is recorded as a caller
that could not be named.

The process is **held by a process descriptor (a pidfd), never by its number**,
because a number is given to another process once the first has ended:

- When the bus daemon's `GetConnectionCredentials` answer carries `ProcessFD`,
  that descriptor is the process held. It was taken from the socket, so it is
  the connection's. A `ProcessID` beside it that names a different process is
  refused.
- When it carries only `ProcessID`, a descriptor is opened for that number
  (`pidfd_open`) before anything is read. After the sandbox has been read, the
  bus is asked again and must still give the connection the same number.
- Either way, `.flatpak-info` counts only if the process held is still alive
  once the file has been read.

A caller whose process cannot be held, ends while its sandbox is read, or is no
longer the connection's is **not identified**: it gets the same refusal as a
program with no sandbox, and the record names nobody. The same rule decides who
is sent `SettingChanged`: a connection whose process was gone is sent nothing,
and that is recorded as not identified. `docs/quirks.md` records which bus
daemons give `ProcessFD`.

## How an answer arrives

The Settings portal asks nobody anything, so it answers in the method's reply
and has no handle; its row below says what it replies. Every other answered
portal answers as the specification says: the method returns a handle,
`/org/freedesktop/portal/desktop/request/SENDER/TOKEN`, and the answer is the
`Response(u response, a{sv} results)` signal of `org.freedesktop.portal.Request`
on that path, sent to the caller.

- **`TOKEN`** is the `handle_token` option. It must be ASCII letters, digits
  and underscores, 1 to 64 of them. Any other token is refused on the call with
  `org.freedesktop.DBus.Error.InvalidArgs`, because no handle can be made from
  it. With no token, the backend makes one.
- **The response is sent before the method returns**, because every answer here
  is decided while the call is handled. Listen on the predicted handle before
  calling, as the specification advises and GTK, libportal and libsecret do.
- **`response` is `0`** when what was asked was done, and **`2`** for every
  refusal and every request that could not be done. It is never `1`, *cancelled
  by the user*, because no person is asked anything here.
- **`results` is empty** in every response.

Every request that reaches the backend, including every refusal, is recorded
before its response is sent. The record names the application whenever the
sandbox named one. On a machine it is kept in
`/var/lib/alo/portal-answers.jsonl`, and stays after the backend stops;
`docs/contracts/portal-answers-file.md` is its shape.

**An answer that cannot be recorded is not sent.** When the record cannot keep
an answer, the application receives the refusal instead: response `2`, or
`org.freedesktop.portal.Error.Failed` on the Settings portal. No secret is
written, no file is opened, and no `SettingChanged` is sent. A secret is written,
and a file handed to its opener, only after the answer has been recorded.

## Answered

| Portal | Interface | Version | What is answered |
|---|---|---|---|
| secret storage | `org.freedesktop.portal.Secret` | 1 | `RetrieveSecret(h fd, a{sv} options)`: the application's own secret, from the person's one keyring, written to `fd`, then response `0`. Needs a grant of the `secrets` facility to that application. On a refusal, or when the keyring is locked or absent, nothing is written and the response is `2`. The keyring is never unlocked for an application. |
| settings | `org.freedesktop.portal.Settings` | 2 | The `org.freedesktop.appearance` namespace only, from the person's appearance settings over what the release ships, resolved at the moment of asking at the machine's time of day. `color-scheme` (`u`): `1` dark, `2` light, never `0`. `accent-color` (`(ddd)`): the person's accent for the scheme showing, red, green and blue from 0 to 1. `contrast` is **not answered**: nothing on this machine keeps a person's contrast preference yet, so no value is invented for it. `ReadAll(as namespaces)` returns both keys when a pattern reaches the namespace (the empty list, `""`, a `*` prefix, or its exact name), and an empty dictionary when none does. `ReadOne(s namespace, s key)` returns one value, and the deprecated `Read` returns the same value inside one more variant. Needs a grant of the `appearance-settings` facility to that application. Without one — or from a program with no sandbox — every method replies `org.freedesktop.portal.Error.NotFound`, the same error, in the same words, as a namespace or key that is not answered, and nothing else. Any other namespace or key is `org.freedesktop.portal.Error.NotFound` to every caller, never a value. Grants or settings that could not be read are `org.freedesktop.portal.Error.Failed`. `SettingChanged(s namespace, s key, v value)` is sent, with a destination and never broadcast, to each connection whose application may read the value at the moment it changed — by the person or by their schedule's clock — and to nobody else; the backend looks every second. Every read and every refusal is recorded with the application named, and so is every `SettingChanged` sent. |
| open-with and default applications | `org.freedesktop.portal.OpenURI` | 3 | `OpenFile(s parent_window, h fd, a{sv} options)`: the file behind `fd` is opened in the application that opens its kind, which is the person's choice or the first installed application declaring it. The file's kind is read from its bytes. Needs a grant over the file **and** a grant over that application, and the file is not read until the first is found. The application is asked to open the file through D-Bus activation (`org.freedesktop.Application.Open`), and the response is `0` only if it answered. `OpenURI` (a web link) and `OpenDirectory` (a file's folder) are answered `2`, because nothing on this machine decides yet what opens either. `writable`, `ask` and `activation_token` are accepted and not read. |

## Not answered yet

These are **not registered** on the bus. An application that calls one gets
the bus's own `org.freedesktop.DBus.Error.UnknownInterface` or `UnknownMethod`.
None of them is answered *yes* to stop an application asking. Each is either
unanswerable without a dialog, or unanswerable until the part of the machine it
reaches exists.

| Portal | Why not yet |
|---|---|
| file chooser and documents | Choosing a file is a dialog, and the dialog belongs to the desktop lane. |
| notifications | A notification is something the desktop draws, and nothing is wired from this backend to the desktop yet. |
| print | Printing asks the person about the printer and pages in a dialog. |
| screenshot | Taking a picture of the screen shows the person what is being taken before the application gets it, which is a dialog. |
| screen capture | Choosing what to share is a dialog, and the capture plan puts an indicator beside it. |
| camera | The camera, and the indicator shown while it is in use, belong to the devices plan. |
| microphone | The microphone has no portal interface of its own; it is reached through PipeWire, which belongs to the devices plan. |
| clipboard | The Clipboard portal works inside a remote-desktop session, and this backend serves no such session yet. |
| trash | Answering needs the person's trash to exist as a surface they can take a file back from. |
| wallpaper | Setting the background asks the person to confirm, which is a dialog. |
| inhibit | Keeping the machine awake needs the session plan's sleep, which does not exist yet. |
| network monitor | This backend has no honest source for the network's state yet. |
| power-profile monitor | The power profile belongs to the devices plan. |

## What changes additively

A portal moves from *not answered yet* to *answered* as its decision and its
surface arrive, and the move is recorded here in the same change. The interface
versions above only increase. The meaning of a `0` does not change: something
that was asked was done.
