# Power test fixtures cannot remove each other's files

Battery and settings-persistence fixtures in alo-power had separate zero-based
counters but both used alo-power-<process>-<counter> temporary directory names.
In the same test process, they could select the same directory; either fixture's
cleanup then removed files the other was still using.

Workspace publication gates reproduced both failure paths: battery fixture
writes failed with NotFound, and a persistence write failed with
NotWritable(NotFound). This prevented publication of unrelated approved artwork
and Settings work.

The battery fixture now uses alo-power-battery-<process>-<counter>. The module's
counter still isolates its own fixtures, and the distinct prefix separates it
from persistence fixtures. Cleanup, assertions and parallel execution remain
unchanged. No production power behavior changes.

The owner explicitly authorized this existing repair on this PC to unblock
publication, together with the approved wallpaper. This is a narrow integration
repair, not a transfer of the devices/media workstream or a new plan task. The
repair was already prepared in lane B; this change reuses its prefix isolation.

Validation requests: repeatedly run all alo-power library tests with normal test
parallelism to exercise both fixture lifetimes, then run all nine required gates
on the integrated tree. Actual results are recorded by the publication operator;
no unrun check or certified-hardware acceptance is claimed here.

Proposed change description: isolate power test fixtures so concurrent battery
and settings-persistence tests cannot delete each other's temporary data.
