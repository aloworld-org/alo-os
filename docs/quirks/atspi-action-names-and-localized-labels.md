# AT-SPI action identifiers differ from translated labels

Observed 2026-11-18 on alo OS source
`ab49f785268934879e41a3dac1d2f906ef003146`: the installed Studio walk read
Settings successfully, then `alo-app alo --in settings press tab "Running code"`
refused before recording began.

The shell publishes the visible label through `GetActions` and the machine
identifier `click` through `GetName`. This matches the
[AT-SPI Action contract](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Action.html).
The adapter and private verification reader incorrectly treated the localized
tuple as an identifier. Both now read the signed `NActions` property, require
a count from zero through 64, and fetch each machine name by its original
index. Malformed counts refuse before invoking an action. Visible labels and
the shell's protocol remain unchanged.

Independent private D-Bus regressions reproduced the refusal and the inverse
error: a localized label `click` could select an unknown machine action.
They check translated labels, exact invocation index, zero actions, the upper
bound, and negative or wrongly typed counts. These run in the adapter suite
and Studio's verification-instruments workflow. Fixture success establishes
the protocol correction; installed recording and consent acceptance must still
come from the image walk.
