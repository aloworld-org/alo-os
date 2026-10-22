# A service that starts, serves its name, and holds nothing

The keyring the owner's provider key goes in. Found 2026-10-22, by running it,
in a change that was gated green and about to land.

## What was concluded

`image/usr/lib/systemd/system/alo-keyring.service`, in its own words:

> **`--unlock` reads a password from standard input**, and `StandardInput=null`
> below is what makes that password empty

And the recipe's build-time proof, which was written to be better than
`test -x` and is:

```
&& /usr/bin/gnome-keyring-daemon --version | grep -q 'gnome-keyring-daemon' \
&& grep -rqs 'org.freedesktop.secrets' /usr/share/dbus-1/services/ \
&& echo "a keyring: a Secret Service daemon that runs, and a bus name it can be reached by"
```

Both passed. Every test of the provider road passed. The branch was green.

## What was true

`StandardInput=null` is **end of file, which is not an empty password.**

Measured under systemd 255, with this unit, a real `gnome-keyring-daemon` and
`secret-tool` as the client — two units differing in one setting and nothing
else:

| standard input | systemd says | `default` alias | store a key |
| --- | --- | --- | --- |
| `StandardInput=null` | `active` | `/` | **fails:** *object does not exist at path …/collection/login* |
| `StandardInput=data`, `StandardInputData=Cg==` | `active` | `…/collection/login` | **stored, and read back by another process** |

`Cg==` is base64 for one newline. That byte is the difference between a person
being able to keep their OpenAI key and not.

**Both units report `active`.** The service starts, stays up, and takes
`org.freedesktop.secrets` on the bus in both cases. What it does not do in the
first case is create any collection, so there is nowhere for a key to go — and
`alo-secrets` asks for `get_default_collection`, which is the alias that reads
`/`.

## The mechanism

**Three checks, each true, none of them the thing.**

`--version` proves the binary loads and resolves its libraries. The `grep`
proves a D-Bus activation file names the service. `systemctl is-active` proves
the process is up. A reader can hold all three and still be wrong, because the
product's question is *can a key be stored*, and none of the three touches it.

This is the family already in the gate section — `test -x` standing in for a
converter starting, `gst-inspect-1.0` standing in for a file decoding — with
one turn of the screw: those checks were **weaker than** the thing. These are
**adjacent to** it. `--version` really does run the binary. The service really
is active. Each check is sound and measures something one step to the side of
what matters, which is why upgrading `test -x` to `--version` felt like
finishing the job.

**And the comment was an assertion about somebody else's software, written in
the voice of a measurement.** *`StandardInput=null` is what makes that password
empty* reads like a fact about systemd. Nothing had run it. A sentence
describing what a flag does is a hypothesis until a process has been started
with it, and this one was wrong in the way that is hardest to see: the plausible
direction. `null` sounds like nothing, and an empty password *is* nothing — the
gap between *no password* and *no input* is one byte and does not survive being
paraphrased.

**The gate could not have caught it.** No test in this repository starts a
systemd unit. The end-to-end test that exists uses `alo_keyring_fixture::
AKeyringOfOurOwn`, which starts a daemon the test itself unlocks, so it proves
every join in the Rust and cannot see the unit at all. It passed throughout.

## The cure

**Ask the product's question, not a question beside it.** For anything that
serves, the measurement is a **round trip**: write through one process, read
back in another, assert the bytes. For this unit:

```bash
# in a container from the recipe's own pinned base, with systemd as pid 1
systemctl start alo-keyring.service
dbus-send --session --print-reply --dest=org.freedesktop.secrets \
  /org/freedesktop/secrets org.freedesktop.Secret.Service.ReadAlias string:default
# "/" means no collection: the service is up and useless
secret-tool store --label=k provider OpenAI <<< 'a-key'
secret-tool lookup provider OpenAI    # must give it back
```

**Two variants, not one run.** The reading above is only a finding because the
same probe with one setting changed passed. A single failing run is a broken
probe until something adjacent to it succeeds — and the first version of this
probe *was* broken twice: `$?` read after a pipe through `sed` reported `EXIT=0`
for commands that had failed, and a nested heredoc produced no output at all,
which reads exactly like a test with nothing to report.

**And where a comment states what another system's flag does, say whether it
was run.** This one now carries the measurement and the date.
