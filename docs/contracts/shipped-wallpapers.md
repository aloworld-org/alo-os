# Shipped wallpaper lookup

A wallpaper selected as `alo_appearance::Of::Shipped(name)` resolves to
`/usr/share/alo/wallpapers/<name>.png`. The name is the single ordinary filename
validated by `alo_appearance::Picture::shipped`; it is not an arbitrary path.
Only an explicitly selected personal-file background uses a personal path.

The default name is `alo_appearance::THE_WALLPAPER`. For the current `alo`
default, the image installs the approved bytes from
`docs/artwork/wallpapers/alo-quiet-horizon.png` at
`/usr/share/alo/wallpapers/alo.png`, owned by root and readable as mode 0644.
The source artwork approval and SHA-256 are recorded beside that source asset.

Missing, unreadable or malformed shipped artwork is an image/integration failure.
The renderer must report it and must not silently replace the selected picture
with an unchosen colour or display a client surface through the lock screen.
Image decoding and output allocation must be bounded before allocation.

Installing this asset does not install a compositor or certify a working lock
screen. The shell owns rendering, fit/crop behavior, privacy and readability
acceptance; the image recipe and its tests own inclusion and the installed name.
