# Application permissions and Settings share one current list

The owner authorised completing alo Studio's capture and save integration on
2026-11-04. Those dialogs make application grants while Settings can revoke
another grant, so an old Settings view must not replace a newer permission list.

The grants store now offers a locked read, edit, and replace operation. Settings
applies its specific grant or revocation to that latest list. A failed edit
leaves the earlier file intact. The existing whole-file writer also takes the
same lock, so staging files cannot collide.

Permanent regression checks exercise simultaneous writers and a refused edit.
Validation is pending; this prerequisite does not establish working capture,
sandboxed export, or a booted recording.
