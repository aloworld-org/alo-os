# A keeper handed the folder was handed the file

**Found 2026-11-07, the glide lane, while building the Reduced motion switch.**

## What was concluded

#1110 read a person's reduced-motion setting at sign-in with
`alo_access::keeping::at_sign_in(&folder.join(alo_access::keeping::THE_FILE))`,
the way `alo-desktop` reads `appearance.toml` two lines above it.

## What was true

`alo_access::keeping` is handed the **folder** and joins `access.toml` itself.
`alo_appearance::keeping`, `alo_dock::keeping` and the rest are handed the
**file**. So the desktop looked for `access.toml/access.toml`, found nothing,
and read *nothing turned on* — silently, because a missing file is not an
error there. Nothing reached a person, because no surface could turn reduced
motion on until the switch that found it.

## The mechanism

Two crates with the same function names (`at_sign_in`, `keep`) and the same
parameter name (`at: &Path`) meant different things by it. Copying the call
from its neighbour was copying a convention the neighbour did not share, and a
read that answers *nothing* on a wrong path is indistinguishable from a person
who has not chosen. No test covered the desktop's sign-in read.

## The cure

Read the rustdoc of the function being called, not the call beside it. A test
that writes the setting through the owning crate's `keep` and reads it back
through the road under test would have failed: the Settings test that found
this does exactly that.
