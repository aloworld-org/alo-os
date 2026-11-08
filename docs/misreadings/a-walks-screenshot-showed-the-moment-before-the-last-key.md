# A walk's screenshot showed the moment before its last key

**Found 2026-11-08, by reading the machine's own files after the walk. Nothing
caught it automatically; the screenshots looked like a product fault.**

## What was concluded

From the walk of image `f7cea72c` in qemu (`w17-38` to `w17-46` in
`C:\dev\boot-evidence\the-walk\`), two faults in Settings → Accessibility:

1. *Tab jumped from Pointer size straight to Reduced motion, and Right and Left
   did nothing*, so the pointer rows were out of the keyboard's reach.
2. *A left click on **Larger** did not select Larger; it toggled Reduced motion,
   the row holding keyboard focus.*

## What was true

The person's files on the machine's disk, read with the disk attached read-only
on the host (`losetup -r`, `mount -o ro,rescue=all`):

```
access.toml       written 05:26:20   format = 1 / on = []
appearance.toml   written 05:30:58   format = 1 / pointer_size = "Larger"
```

The click, at about 05:31, wrote **Larger** and did not touch `access.toml`.
`access.toml` was last written by **Space**, a few seconds before `w17-43`.
The pointer in `w17-46` is drawn larger. One click acted on one row, and it was
the right row.

Every shot from `w17-39` to `w17-46` shows **the state before the last input
the script sent**, however long the script waited afterwards:

| shot | script's last input | what the shot shows |
|---|---|---|
| `w17-39` | Tab, Tab | the keyboard still in the side column |
| `w17-40` | Right, Right | the effect of the two Tabs: *Default* |
| `w17-41` | Tab | unchanged, which Right is meant to leave it |
| `w17-42` (15 s later) | Enter | the effect of Tab: *Reduced motion*, off |
| `w17-43` | Space | the effect of Enter: *Reduced motion* **on** |
| `w17-44` | pointer moves | the effect of Space: off |
| `w17-45` | the click | *Default* still drawn as chosen |
| `w17-46` (40 s later) | Enter in the terminal | the command typed, not run |

And with that one-step lag, the keyboard behaved exactly as
`docs/design/the-settings-window.md` says: Tab goes to the next section, Right
and Left change only the text size, and Down reaches every choice.

## The mechanism

**A shot taken after one input was read as the result of that input**, and the
whole series agreed with itself, because every shot was one step behind in the
same way. Read one at a time, each shot told a plausible story; the story only
broke when the series was laid out against the script that made it, or when the
machine's own files were read.

**Why the screen trails is not known.** It may be qemu's `screendump` on
`virtio-vga` with `-display none` returning the surface before the update it
asked for, or the desktop not presenting a frame until the next input arrives.
These two are different faults with different owners, and nothing here tells
them apart. It has not been looked at on real hardware.

## The cure

- **Read what the input should have written before believing the picture.** A
  setting is a file; its time and its contents answer *did the press happen,
  and to which row* without a screenshot.
- **Lay a walk's shots against the script that made them**, input by input, and
  look for the shot that shows the input *before*.
- **Send one harmless input before each shot** — a pointer nudge of one pixel —
  until the trailing is explained, so the shot shows the input that matters.
