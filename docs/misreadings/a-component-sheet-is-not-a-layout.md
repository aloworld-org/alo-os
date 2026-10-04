# A component sheet is not a layout

**Concluded:** *the design places a Settings button on the Dock, drawn once for
each edge.*

**True:** node `348:23618` sits in a frame named `Dock edges · Components` and
is one of **fifteen 44×44 buttons in an evenly spaced row** — a component
palette. The three other hits were the overflow list on three specification
sheets. The design puts Settings **under a heading reading *More open apps***,
beside Mail, Calendar, Notes and Terminal: it is an application, not a control
beside the applications.

**The mechanism.** A definition and a placement look identical in a node tree:
both are frames with coordinates, names and sizes. The row of fifteen even had
plausible geometry — 44×44 matches the owner's click-target ruling exactly.

**The cure.** Before reading a frame as a layout, walk up to its named ancestor.
`Dock edges · Components` and `Dock Left / Overflow` answer the question that
coordinates cannot. A top-level page listing is the same trap one level up: it
showed one page named `00 — Cover` while the file held 20,105 nodes.
