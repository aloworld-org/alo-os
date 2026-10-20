# The installer's screens, as rules rather than pixels

**Read off section `530:30834` on 2026-10-20** — the six frames the owner
adopted that day — and off `03 · Keep Windows or replace it` (`370:30118`),
which all six are cloned from and which the Windows road already draws.

Every frame is 1440 × 960. **No machine is that.** The environment draws on
`/dev/fb0` at whatever mode the firmware left, with no compositor to normalise
it and no mode setting of its own (`crate::a_surface_to_draw_on`). So each
number below appears twice: as the frame draws it, and as the rule a layout
follows — `CLAUDE.md`: *a figure taken from a frame becomes a proportion or a
named rule, never a constant that holds only at the size somebody drew.*

## The screens

| | what it asks |
|---|---|
| `530:30835` S1 | Start this PC from a stick — **drawn by the Windows program**, not here |
| `530:30856` S2 | Try alo OS, or install it |
| `530:30877` S3 | How should this computer connect? |
| `530:30898` S4 | Which disk should alo OS use? |
| `530:30919` S5 | Review · this erases *disk* |
| `530:30940` S6 | No disk here is big enough |

S1 belongs to `alo-installer` and runs on Windows — *we restart this PC* and
*Windows starts as usual the next time you switch this PC on* can be said
nowhere else. The other five are this environment's.

## One layout, six screens

Every frame is the same shape with different words, which is the point of
cloning them: a person meets one page and reads a different question on it.

| | at 1440 × 960 | the rule |
|---|---|---|
| the block | at (160, 164), 600 wide | left at **11.1%** of width, top at **17.1%** of height, **41.7%** wide |
| eyebrow | 11 / 16 | **1.15%** of height |
| title | 36 / 44, 32 below the eyebrow | **3.75%** of height |
| introduction | 16 / 25, 104 below the title's top | **1.67%** of height |
| a card | 600 × 86, radius 12 | the block's width; **8.96%** of height |
| card to card | 10 | **1.04%** of height |
| the consequence line | 14 / 22, at +375 | **1.46%** of height |
| the actions | at +413, 48 tall | **5%** of height |
| a button | 160 × 48, radius 12, 12 apart | **11.1%** of width |
| *alo OS*, top right | 13 / 18 at (950, 52) | **1.35%** of height |
| accessibility | 160 × 48 at (40, 876) | a button, **2.8%** in from the left and the bottom |
| the help line | 11 / 16, right-aligned at y 892 | **1.15%** of height |

## Why everything is a proportion here, and nothing is a scale

The desktop has `alo_displays::Scale`, which works a scale out from pixels and
the glass's physical size. **The environment does not use it**, and that is a
property of these screens rather than an exception granted to the installer.

Each of these is **full screen, single purpose, with nothing else on it to
agree with**. There is no window, no chrome, no second surface, and nothing a
person points at with a pointer. A scale exists so that surfaces drawn from
different sources agree about how large a thing is; here there is one source
and one thing.

And the case that decides it: **`Scale::worked_out_for` answers 100% when the
display reports no physical size** — the only honest answer when the machine is
told nothing. On the desktop a person opens Settings and changes it. **Here
they cannot**: there is no Settings, no session, and no next boot to fix it in,
and the screen they are reading asks them to confirm erasing a disk. A panel
that lies about its size is not rare — virtual outputs do it. So the layout is
built not to need the answer.

## The one measure that is not a proportion: the legibility floor

A proportion keeps the *shape* of a screen and not the *size* of its letters.
Measured against the design's own physical intent — 1440 × 960 read at 96 dpi,
cap height about 0.72 em:

| | title (36) | body (16) |
|---|---|---|
| the design, at 96 dpi | 6.86 mm | **3.05 mm** |
| 15-inch 4K, by proportion | 5.04 mm | **2.24 mm** |
| 32-inch 4K, by proportion | 10.76 mm | 4.78 mm |

**The title is never the problem.** Five millimetres of capital is a large
heading anywhere. The number that matters is the **body text at 2.24 mm on a
laptop panel** — three-quarters the physical size the design draws it, and the
smallest thing a person has to read on a screen asking them to confirm erasing
a disk.

**So the floor is on body text's cap height, not on the title's type size.** A
floor set on the title would let the body fall as far as it liked; set on the
body, it raises everything above it.

**The floor is the design's own physical intent** — the body's cap height at
96 dpi — rather than a number from outside. When the display reports its glass
and proportion would draw body text smaller than that, the type is raised until
it is not. When the display reports nothing, the proportion stands: no scale is
guessed from a pixel count, because **a pixel count cannot tell a 13-inch 2560
laptop from a 27-inch 2560 monitor**, and those want opposite answers.

## What this file does not decide

**The colours.** `crate::a_surface_to_draw_on` cannot know the framebuffer's
channel order without a struct it exists to avoid, which is why the first thing
it draws is grey — *a colour whose channels are equal cannot be wrong about
their order*. Until a coloured screen is confirmed on a machine, these screens
are laid out here and coloured there.

**Which rows a disk list shows, and in what words.** That is
`crate::which_disks_may_be_offered`, which decides and says and has no geometry
in it at all.
