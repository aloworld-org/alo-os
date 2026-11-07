# Narrow safe-wrapper additions

ADR 0149 authorises these additions before the source patch. Everything else is
the published MIT-licensed `pipewire` 0.10.0 crate, including its license,
copyright notices, and registry metadata. Its original crate archive SHA-256:
`8585aba8a52ad74ccc633b8e293c1dc4277976bd5d510b925533f34fd6685f38`.

Two generic wrappers are added, with no alo policy or protocol changes:

- `Core::update_self_permissions` checks the count, borrows the core's client,
  invokes its existing update-permissions method, and returns its error.
- `CoreBox::into_fd` consumes unique ownership, steals the descriptor, gives its
  ownership to `OwnedFd`, then runs the normal detached-core destruction. It is
  available only with the minimum PipeWire feature gate required by this use.

These wrap the two operations used by upstream xdg-desktop-portal when preparing
and handing off a restricted media connection. The source of the upstream
reference implementation is xdg-desktop-portal 1.18.4, `src/screen-cast.c` and
`src/pipewire.c`. The Rust binding's otherwise unchanged API and license remain
upstream's. Remove these changes when upstream supplies safe equivalent APIs.

The workspace still forbids unsafe code. This rented binding is excluded from
the workspace's own crate set. Real-server handoff tests must demonstrate both
the selected node's availability and unrelated nodes' invisibility; the wrappers
alone do not establish that permission policy or successful capture.
