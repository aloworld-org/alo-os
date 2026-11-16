# What alo's copy of VSCodium sends, and to where

- **Version:** VSCodium 1.143.17232 from Flathub (`com.vscodium.codium`).
- **Whose:** VSCodium's and Electron's; alo OS configures them and patches
  neither.
- **Date:** 2026-11-15, after the VM walk on `main` (run 43751841598) showed
  *VSCodium (com.vscodium.codium), in the copy alo is working in, is sending*.
- **Measured once by hand**, in WSL, not on an alo machine: the editor started
  with a fresh profile set as alo's copy is, every connection it made sent
  through a logging proxy (`ss` showed no other peer), for 60 seconds, then
  started again on the same profile. Nothing runs this again.
- **Where that line is shown now.** Since the owner's ruling of 2026-11-15 —
  *"We remove this and the sending, the border is enough"*, *"Border only, it
  thickens"* — the corner no longer shows this line while VSCodium's window
  wears alo's border; the border thickens instead. Where no border is drawn,
  the corner still shows it (`docs/design/the-window-controls.md`).

## What it sends

| where to | when | why | checked or fits |
|---|---|---|---|
| `redirector.gvt1.com`, then a `*.gvt1.com` mirror (Google) | the **first** start of a profile only | Electron's spell checker fetches its English dictionary, `en-US-10-1.bdic`, 452 KB, kept in the profile's `Dictionaries` folder | **checked**: the file appeared, and the second start sent nothing there |
| `raw.githubusercontent.com` (GitHub) | **every** start | the list of malicious and deprecated extensions, `extensionsGallery.controlUrl` in `product.json` | **checked**: gone with `extensions.excludeUnsafes: false`, which the code reads before fetching it |

**Nothing else.** `product.json` names no update address and no telemetry
address (read, not inferred), and the Open VSX gallery is not asked anything at
start with no extensions installed.

## What alo's copy sets

`update.mode: "none"` and `telemetry.telemetryLevel: "off"`
(`alo_launching::ASKS_FOR_NOTHING_IT_NEED_NOT`): neither sent anything here,
and these keep it so if a later build adds an address. Measured with them on,
the two rows above were unchanged.

**The two rows above are left on**, because turning either off costs the
person something:

- the extension list: VSCodium would stop warning about a known-malicious
  extension installed in alo's copy;
- the dictionary: there is no setting for it; it would need a dictionary
  shipped with alo OS or the spell checker's address changed.

## Decided

**The Open VSX safety list stays on** — the owner, 2026-11-15. alo's copy
leaves `extensions.excludeUnsafes` unset, so VSCodium keeps fetching the list
of malicious and deprecated extensions from GitHub on every start, and keeps
warning about a known-bad extension installed in alo's copy. The indicator
therefore shows alo's copy sending on every start, which is correct: it is.

The dictionary row has no decision yet.
