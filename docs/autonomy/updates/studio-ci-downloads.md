# Studio verification selects Ubuntu's HTTPS archive

Hosted native setup repeatedly reached its 15-minute limit while downloading
media libraries from the regional HTTP Ubuntu archive, before tests began.
The Studio and installed-walk workflows now select the central HTTPS archive
before updating package indexes. Setup retains bounded network retries and
timeouts; emulator installation now has the same 15-minute step limit.

The helper changes only exact Ubuntu archive URI tokens in legacy source
lines and deb822 URI fields. Suites, components, signature configuration,
security endpoints, comments and unrelated repositories retain their bytes.
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

This fixes a measured setup blocker, not recording acceptance. Hosted download
timing remains to be measured. Installed run 44428146796 continues against its
unchanged application source and matching signed package; its video, audio and
native agent permissions still need acceptance. No application or model-facing
behavior changes here, and no public package or image is published.
