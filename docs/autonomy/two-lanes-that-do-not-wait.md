# Two lanes that do not wait for each other

The owner asked for a roadmap where neither lane blocks the other, so each can
be driven as its own loop. This is that split.

**The rule this is built on:** a lane that is waiting is a lane that has run
out of work it was allowed to start. Every item below is startable the moment
the one above it is finished **by the same lane** — never by the other one.

## The line between the lanes is the restart

It is already how these two have been working, and it falls on file boundaries
with no overlap.

| | owns |
|---|---|
| **The Mac lane** | `image/installing/`, `crates/alo-installing`, `crates/alo-image`, `.github/workflows`, declaring versions, every dispatch |
| **The Windows lane** | `crates/alo-installer`, `crates/alo-shell`, `image/Containerfile` |
| **The owner** | signing, pinning, scope rulings |
| **The testing machine** | every walk on real hardware |

## Two decisions taken now, so that nothing waits on one later

A lane blocked on a ruling is blocked just as hard as one blocked on a branch.
Both of these were open and both are answerable without new information.

**1. The boot environment ASKS for the network. It does not carry it from
Windows.** Debian's installer asks; so do Ubuntu's, Fedora's and Windows Setup.
It is the ordinary answer, it works on a machine with no Windows to carry
anything from — which the removable-media road needs — and it means **no Wi-Fi
password is ever written to a FAT partition**, so the ADR that question needed
is not needed at all. The Windows lane has no work in it, so the Mac lane is
never waiting on this lane to start.

**2. The staging area goes AFTER alo OS's root**, not before it. Today
`BesideWindows` lays out `Windows | area | alo OS`, and nothing chose that — it
falls out of one line inherited from the empty-disk road, which has no alo OS
partition to order against. Reversed, `tidying` removing the area leaves
contiguous free space immediately after the btrfs root, and one
`btrfs filesystem resize` hands it to the person instead of leaving a hole. The
geometry is the Windows lane's to change; the resize is the Mac lane's. Neither
waits, because the shape is agreed here.

## The spine: the only thing that is serialised

Four handovers, and they are genuine — an image cannot be booted before it is
built. **Nobody idles during them**, because each lane has its own queue below.

    Mac: dispatch image.yml
      -> Windows: boot 0.0.8 in qemu, press the Applications entry
        -> Owner: sign both commands, verify, pin
          -> Mac: dispatch installer-candidate.yml
            -> Testing machine: the walk

The second step gates the third **deliberately**: nobody has ever watched the
market open on a running machine, and a signature on something unseen is the
thing this project exists not to do.

## The Mac lane's loop

In order. Each is startable without this lane, except where the spine says
otherwise — and when the spine is waiting, the next item here is.

1. **#843** — declare 0.0.8. In the queue.
2. **Dispatch `image.yml`** once #843 and #844 are in. Then go to 3 rather
   than watch it.
3. **Wi-Fi in the boot environment.** `image/installing/alo-installing.conf`
   gains the supplicant, NetworkManager's Wi-Fi plugin and the Intel firmware —
   about 75 MB installed, against a 1 GiB area holding a few hundred
   mebibytes. Measured: `wpa_supplicant` 6.6 MB, `NetworkManager-wifi` 134 KB,
   `iwlwifi-mvm-firmware` 66 MB, `iwlwifi-dvm-firmware` 2 MB.
4. **The network question in `alo-installing`.** Which network, and the
   password. The environment already asks for a disk name and takes a typed
   answer, so this is the same shape rather than a new one.
5. **`--source-imgref` reading a local copy.** Settled from the engine itself:
   `bootc 1.15.1` says *See skopeo(1) for accepted formats*, and
   `--target-transport` names `oci`, `oci-archive`, `containers-storage`. So
   this is configuring the flag, not patching the engine.
6. **Verification before the shrink.** `crate::verifying` runs after the
   restart today. With a local copy it can run while the disk is untouched,
   which is where it is worth the most.
7. **`THE_AREA`**, and the refusal around it. Theirs, because it is where both
   halves meet and it decides whether a machine can install at all.
8. **The `btrfs filesystem resize`** after `tidying`, once the geometry above
   lands.

## The Windows lane's loop

1. **#844** — wireless in the image, and ADR 0009's missing test. In its gate.
2. **#845** — the Windows-only test that could not compile. In its gate.
3. **Boot 0.0.8 in qemu and press the Applications entry.** The spine's second
   step. Twenty minutes, and it gates the owner's signature.
4. **The road question is offered before its own guard.** `sequence.rs:127`
   asks `which_road`; `may_replace` — the disk-count guard — is the first line
   of `walk`, which only runs after someone has answered *replace*. A one-disk
   machine is shown a choice it is not eligible for.
5. **Two comments that state the opposite of what the code does.** `walk`'s
   header says *refuse before asking*; `sequence.rs` says the question is
   *asked only where replacing can be walked*. Neither is true on a one-disk
   machine, and a stale comment that contradicts its own function is how the
   next person builds the wrong thing confidently.
6. **Three sentences that are wrong or missing.** *Already holds files or a
   system* said of an empty stick; three unrecognised answers defaulting to
   *keep* without saying so; *this is what happens, in this order:* followed by
   a question rather than the list it announces.
7. **The Windows half of download-first.** Fetch 3.55 GiB with resume and
   retry, stage it where the Mac lane's item 5 reads it, and grow the area to
   hold it. The refusal for *not enough space* must come **before** the shrink:
   discovering it afterwards is this whole approach's own failure mode wearing
   different clothes.
8. **The layout change** — `Windows | alo OS | area`, for the Mac lane's
   item 8.
9. **Fast Startup read differently by two builds on identical input.** v0.0.6
   said *could not be found out* where the 2026-10-08 candidate says *on*, with
   the same registry state. Unexplained, and a behaviour change nobody intended
   is worth one look.

## The interfaces, agreed before either lane needs them

**The staged image**, for download-first: a path on the staging volume that the
Mac lane's item 5 points `--source-imgref` at, in a transport skopeo accepts.
The Windows lane writes it; the Mac lane reads it; neither needs the other's
internals.

**The network**: nothing crosses. The environment asks, so there is no file and
no credential to hand over. That is the decision's second benefit after the
missing ADR.

**The geometry**: `Windows | alo OS | area`, offsets computed by the Windows
lane in `windows_volume.rs`, consumed by the Mac lane's resize.

## Not scheduled, and said so rather than left implied

- **Printing.** The image carries `cups-libs` and no daemon. ADR 0078 says what
  printing owes is a printer rather than a release, and `docs/features.md` says
  plainly that no paper has come out of anything. Tracked, not hidden.
- **Nobody has watched an application install from the market.**
  `alo_software::install` is unit-tested against a fake `Tool`, and
  `docs/quirks.md` records no real one. The market opening is item 3 above; the
  installing is a walk nobody has done.
- **The removable-media road.** Named as a road that must exist by
  `docs/features.md` and ADR 0023's amendment, deliberately not specified. It
  wants a design before it wants a lane.
