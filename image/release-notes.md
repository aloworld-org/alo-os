# alo OS 0.0.7 — a terminal you can type in, and a command that asks alo

The notes the GitHub Release carries, committed here so that nothing about the
release is typed twice. `crates/alo-image` holds every fact below to
`image/pinned.toml` and to what `crates/alo-installer` actually accepts, and the
workflow in `.github/workflows/release.yml` publishes this file as the Release's
body rather than a sentence somebody wrote in the web form.

Download `alo-installer.zip`, unpack it, and run `alo-installer.exe`. It reads
this computer, says what it found, says exactly what it will do, and changes
nothing until you type the name of a disk.

**This release has a terminal, and it is the first one that does.** Every image
before it could sign you in and draw a desktop, and gave you nowhere to type.
This one opens a terminal of its own, takes the keyboard, draws what a program
prints — colours, the cursor, a file diff — and scrolls. alo OS ships a terminal
because a system that does not trust its owner with a shell is a toy.

**And the agent can be asked about a command before you run it.** The terminal
reads the marks a shell sends, so it knows where one command ended and the next
began, and alo can say what a command will do, or why one that looks stuck is
waiting. Every run leaves a record, and so does every refusal.

**`alo-ask` is here: the first image where you can type a question to alo.**

    $ alo-ask may the tenant sublet?

It takes the question and no options — which model answers is a setting, not a
flag.

**It still carries no model, so out of the box nothing answers.** That is the
decision 0.0.6 made and this release keeps: a model we chose is a model chosen
for you. Point alo OS at weights you already have, use a provider you have an
account with, or use it with no model at all. Until you choose one, `alo-ask`
will tell you it has nothing to think with rather than quietly failing.

## One fault you will meet, and how to get out of it

**Do not press the Applications icon on the Dock — the "A", second from the
left.** It opens a window that is never drawn, and from that moment the machine
appears to stop taking the keyboard, because the invisible window is taking it.

**Press Escape and the keyboard comes back.** The window closes, even though you
cannot see it. Everything else on the Dock works, Settings included.

This is written down rather than left for you to find because a person whose
keyboard stops working has no way to tell a broken machine from a broken
program. Measured on a machine booted from this image's own commit on
2026-10-19, with a control each way: the press registered, the screen did not
change by a single byte, and the next key was not counted until Escape was
pressed.

The cause is one line in the shell, found the same day and fixed after this
image was built, so the fix is not in it. It will be in the next release.

**Windows does not know who made this program, and it will say so.** When you
run it, Windows shows a blue box saying *Windows protected your PC*, and the
only button offered is *Do not run*. To go on, choose **More info**, then **Run
anyway**. We are telling you here, before you download it, because a program
that repartitions your only computer should never be something you were
surprised by.

**You can check for yourself that what you downloaded is what we published.**
`SHA256SUMS`, beside the download, is a short code worked out from every byte of
the file. Work out the same code on your own machine:

    Get-FileHash -Algorithm SHA256 alo-installer.zip

If it matches the line in `SHA256SUMS`, the file on your disk is the file we
put there, byte for byte.

**Windows stays.** alo OS is installed onto an empty disk beside it, and
Windows starts exactly as it did before. Nothing on the Windows volume is
touched except the 1 GB the installer asks Windows itself to free.

**Secure Boot.** This release installs only on a computer where Secure Boot is
already off. Where it is on, or where Windows will not say, the installer stops,
says why, and changes nothing — and it never asks anybody to alter a firmware
setting.

What this release installs, from the registry, by content:

    registry: ghcr.io/aloworld-org/alo-os
    tag: 0.0.7
    digest: sha256:0daaa69bc1fbbbb8e228f9b9d75b52a3fcad2b5a8d6667e6e51d127766d50c80
    secure boot: off

Nothing but that digest is pulled, whatever a tag in the registry says later.

**What changed since 0.0.6.** 278 commits, and the terminal is most of them: a
person types into it, a shell receives the bytes, programs draw in their own
colours, and alo reads the same transcript the person does. `alo-ask` arrives
with it — the one command a person types to put a question to the machine, and
absent from every image before this one. Underneath, a command runs in a sealed
box and cannot run at all without leaving a record, a person chooses at which of
three levels code is allowed to run, and the record can be asked which command
touched a given file. The thing that opens a session now makes the person's
folder, which is why a fresh account has somewhere to put anything.

`SHA256SUMS` is published beside the download: it is the checksum of every asset
in this Release, so a download can be compared with what was published.

What this computer needs is in `docs/hardware.md`, and the README's *Try it*
says it in short.
