"""The live reference must remain a separate, privately built application."""

import copy
import json
import pathlib
import unittest

from make_reference_manifest import manifest


class ReferenceManifest(unittest.TestCase):
    def normal(self):
        source = pathlib.Path(__file__).resolve().parents[1] / 'packaging' / 'org.aloworld.Studio.json'
        return json.loads(source.read_text())

    def test_normal_installable_application_is_not_changed(self):
        normal = self.normal()
        before = copy.deepcopy(normal)
        fixture = manifest(normal)
        self.assertEqual(normal, before)
        self.assertEqual(normal['id'], 'org.aloworld.Studio')
        self.assertNotEqual(fixture['id'], normal['id'])
        self.assertNotIn('--features', ' '.join(normal['modules'][0]['build-commands']))

    def test_reference_needs_no_extra_filesystem_network_or_bus_grants(self):
        normal = self.normal()
        fixture = manifest(normal)
        self.assertEqual(fixture['finish-args'], normal['finish-args'])
        self.assertEqual(fixture['runtime'], normal['runtime'])
        self.assertEqual(fixture['runtime-version'], normal['runtime-version'])
        self.assertEqual(fixture['sdk'], normal['sdk'])
        self.assertEqual(fixture['sdk-extensions'], normal['sdk-extensions'])

    def test_only_the_private_reference_example_is_built_and_launched(self):
        fixture = manifest(self.normal())
        self.assertEqual(fixture['id'], 'org.aloworld.Studio.Verification')
        self.assertEqual(fixture['command'], 'studio-reference-window')
        commands = fixture['modules'][0]['build-commands']
        self.assertIn('cargo build --release --locked --offline --features verification --example studio-reference-window', commands)
        self.assertNotIn('install -Dm755 target/release/alo-studio /app/bin/alo-studio', commands)
        self.assertIn('install -Dm755 target/release/examples/studio-reference-window /app/bin/studio-reference-window', commands)


if __name__ == '__main__':
    unittest.main()
