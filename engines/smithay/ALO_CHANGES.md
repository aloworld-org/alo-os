# Owned software image mappings

Upstream: Smithay 0.7.0, the published crates.io package.
Package SHA-256: `740cea6927892bc182d5bf70c8f79806c8bc9f68f2fb96e55a30be171b63af98`.

ADR 0149 authorizes only initialized offscreen allocation, render-target binding, and checked byte
borrowing on the existing PixmanMapping. The renderer and every image operation
are unchanged. Upstream source files and license are preserved; the only source
addition is recorded in `owned-pixman-mapping.patch`. Remove the local binding
when a pinned upstream release supplies these ownership interfaces.
