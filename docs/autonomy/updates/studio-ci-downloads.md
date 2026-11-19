# Studio verification selects Ubuntu's HTTPS archive

Hosted native setup repeatedly reached its 15-minute limit while downloading
media libraries from the regional HTTP Ubuntu archive, before tests began.
The Studio and installed-walk workflows now select the central HTTPS archive
before updating package indexes. Setup retains bounded network retries and
timeouts; emulator installation now has the same 15-minute step limit.

The helper changes only exact Ubuntu archive URI tokens and GitHub's standard
`mirror+file:/etc/apt/apt-mirrors.txt` token in legacy source lines and deb822
URI fields. Suites, components, signature configuration, direct security
endpoints, comments and unrelated repositories retain their bytes.
Source files already using the selected URI are not rewritten.

Nine independently authored tests first failed because the helper did not
exist. Two workflow tests then failed because source selection, the walk's
step bound and change triggers were absent. All eleven pass after the change.
Independent review added a twelfth regression for valid `URIs:value` fields
without intervening whitespace. It failed before the field-prefix correction;
all twelve now pass. Fourteen deliberate faults were caught in temporary
copies, followed by a restored passing suite. These fault experiments were
one-time measurements. The existing seven Rust checks for walk publication,
source selection and package provenance also pass against the changed YAML.
They recur in Studio's verification-instruments job before package installation;
both pull-request and main-push filters include the helper directory.

Hosted run 44430834993 exposed a missing format: the initial helper changed
zero source files because GitHub's runners use the mirror-list token. Its native
job passed with 32-second setup on the existing Azure HTTP mirror; that result
does not show an effect from this helper. The verification job continued to use
the slow EC2 HTTP mirror. The runner's [source configuration](https://github.com/actions/runner-images/blob/main/images/ubuntu/scripts/build/configure-apt-sources.sh)
confirms the exact standard mirror-list URI.

Three additional independent tests failed before mirror-list support was added.
All fifteen CI tests now pass. Five new targeted mirror faults were caught in
temporary copies and the restored suite passed. The existing workflow mutation
tests also exposed assumptions about path ordering and a formerly unique test
command; mutations now target the intended PR filter and live suite, retaining
all fifteen refusals. All 393 private instrument tests pass after correction.
These recurring tests do not replace a hosted measurement of the corrected URI.

This addresses a measured setup blocker, not recording acceptance. Hosted
download timing after the mirror-list correction remains to be measured.
Installed run 44428146796 failed before recording at an independent application
permission boundary; its video, audio and native agent permissions still need
acceptance. No application or model-facing
behavior changes here, and no public package or image is published.
