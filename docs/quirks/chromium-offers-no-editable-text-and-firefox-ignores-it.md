# Chromium offers no way to put text in through the tree, and Firefox ignores it

- **Version:** VSCodium 1.143.17232 (Flathub), Firefox 160.0 (Mozilla's own
  build), beside GNOME Text Editor 46.3 on GTK 4.14.5 as the control.
- **Whose:** Chromium's and Gecko's. alo OS configures both and patches neither.
- **Date:** 2026-11-13, for ADR 0143 Q2 (*how does alo type*).

## Behaviour

AT-SPI's `EditableText` interface is how an assistive technology puts text
into a field — insert at a position, or replace the whole text.

- **Chromium does not offer it on any node.** VSCodium's editor (an `entry`
  named after the open file, in either of its two input modes), its terminal,
  and every other editable node had no `EditableText` interface in five runs.
- **Firefox offers it on every node, and a call changes nothing.** On a page's
  `<input>`, `<textarea>` and `contenteditable`, `InsertText` and
  `SetTextContents` answered *true*; the text read back unchanged and the
  page's own `input` handler never ran. Two runs, one after focusing the field.
- **GTK 4 does what the interface says.** Text Editor's document took an
  insert and a whole replacement exactly, two runs of two.

Measured once, by hand, not on an alo machine and not as Flatpaks behind the
accessibility proxy; why each of the first two behaves so was not read from
their source. The full measurement and what follows from it are in
[`docs/autonomy/updates/how-alo-can-type-into-an-application.md`](../autonomy/updates/how-alo-can-type-into-an-application.md).

## What alo OS does about it

Nothing yet. **A call's answer is not evidence that text arrived**: whatever
alo builds reads the text back before it tells the person anything was typed,
because Firefox answers *true* for text it never put anywhere.
