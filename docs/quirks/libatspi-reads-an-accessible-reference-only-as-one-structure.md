# libatspi reads an accessible reference only as one `(so)` structure, and reads the cache without checking it

- **Version:** at-spi2-core 2.60.7 and Orca 50.3 on the walk machine (image from
  `main` bb645d7b plus the Fedora 44 pin fix). The source was read at at-spi2-core
  tag `2.60.0`.
- **Whose:** ours. libatspi does what at-spi2-core's own XML says. The surprise
  is how zbus shapes a reply, which is a platform fact about the library we use.
- **Date:** 2026-11-13.

## Behaviour

With Settings → Accessibility → Screen reader on, Orca stayed up and logged this
about 3.5 minutes in, while Settings was open:

```
orca[2874]: AT-SPI: Error in GetItems, sender=:1.0, error=Unknown object '/org/a11y/atspi/cache'
orca[2874]: AT-SPI: Called _atspi_dbus_return_accessible_from_message with strange signature so   (x10+ in half a second)
```

`:1.0` was alo's shell. There are three facts behind it, each read in the at-spi2-core source:

1. **A reference is one argument of type `(so)`.** `Accessible.xml` gives
   `GetChildAtIndex` and `GetApplication` a single out argument, `(so)`.
   `_atspi_dbus_return_accessible_from_message` (`atspi/atspi-misc.c`) compares
   the reply's signature to the string `"(so)"`. On anything else it prints the
   warning above and returns nothing, so the child Orca asked for does not exist
   as far as Orca knows.
2. **zbus sends a returned tuple as loose arguments.** A `#[zbus::interface]`
   method returning `(String, OwnedObjectPath)` produces a reply whose signature
   is `so`, not `(so)`, because zbus uses a body tuple as the message's argument
   list. To send one structure, wrap it once more: `((name, path),)`. A
   *property* is different. It is a single variant, so a property returning the
   same tuple goes out correctly as `v` holding `(so)`.
3. **libatspi asks every application it meets for `GetItems`** on
   `org.a11y.atspi.Cache` at `/org/a11y/atspi/cache`, right after
   `Application.GetApplicationBusAddress`. **`handle_get_items` does not check the
   reply's signature.** It walks each entry field by field as
   `((so)(so)(so)iiassusau)`, so an entry in the wrong shape is misread rather than
   refused. Once the cache is loaded, libatspi follows the `AddAccessible` and
   `RemoveAccessible` signals and does not ask again. An application that changes
   its tree without sending them leaves a reader on the old one.

libatspi only uses its cache when a main loop runs (`atspi_event_main`, which Orca
calls) or after `atspi_accessible_set_cache_mask`. A client with neither of those
asks every question over the bus. That is how one process can read a tree both
ways.

## Our response

- `crates/alo-shell/src/access_answers.rs` wraps every reference reply once more,
  and its module documentation explains why.
- `crates/alo-shell/src/access_cache.rs` serves `GetItems` in the spec's shape and
  sends `AddAccessible` and `RemoveAccessible` for what changed whenever the tree
  is replaced.
- What holds it, on every gate:
  - `crates/alo-libatspi-walk/tests/libatspi_reads_alos_tree.rs` reads
    the tree with libatspi itself, both ways, and fails on any line libatspi
    prints;
  - `access_bus_tests` reads each reply's signature off a real bus.
- Not yet checked: that Orca on a booted machine stops logging these lines.
  That needs the next walk.
