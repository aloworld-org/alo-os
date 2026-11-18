"""Test-first fresh-profile restoration contract for the private walk only.

require_automatic_source(nodes, *, state='') observes an actual pw-dump list
and optional raw WirePlumber state text. It returns None or raises ValueError;
it must not change observations. The metadata object's props and metadata list
are ROOT fields, as observed in an isolated PipeWire/WirePlumber session.
clear_reference_source(command) executes exactly wpctl clear-default 1 and
propagates failure. Session separately owns deciding whether it made an override.

Synthetic dictionaries/text here are test-only observations, not OS evidence.
Both default-nodes and default-streams persisted state files must be checked by
the caller; deleting the virtual node alone does not undo saved preferences.
"""

import copy
import unittest

from audio_selection import clear_reference_source, require_automatic_source


def automatic():
    return [dict(id=34, type='PipeWire:Interface:Metadata',
                 props={'metadata.name': 'default'}, metadata=[
                     dict(subject=0, key='default.audio.source',
                          type='Spa:String:JSON', value={'name': 'automatic-source'}),
                     dict(subject=0, key='default.configured.audio.sink',
                          type='Spa:String:JSON', value={'name': 'chosen-speakers'}),
                     dict(subject=0, key='default.configured.video.source',
                          type='Spa:String:JSON', value={'name': 'chosen-camera'}),
                 ])]


class AudioSelection(unittest.TestCase):
    def test_present_empty_default_store_is_a_valid_automatic_profile(self):
        nodes = automatic()
        nodes[0]['metadata'] = []
        self.assertIsNone(require_automatic_source(nodes))

    def test_automatic_source_allows_unrelated_configured_sink_and_video(self):
        nodes = automatic()
        original = copy.deepcopy(nodes)
        state = ('[default-nodes]\n'
                 'default.configured.audio.sink=chosen-speakers\n'
                 'default.configured.audio.sink.0=old-speakers\n'
                 'default.configured.video.source=chosen-camera\n')
        self.assertIsNone(require_automatic_source(nodes, state=state))
        self.assertEqual(nodes, original)

    def test_existing_configured_source_refuses_even_when_current_source_is_different(self):
        for value in [{'name': 'old-unplugged-source'}, None, '', {}]:
            with self.subTest(value=value):
                nodes = automatic()
                nodes[0]['metadata'].append(dict(subject=0,
                    key='default.configured.audio.source', type='Spa:String:JSON', value=value))
                with self.assertRaises(ValueError):
                    require_automatic_source(nodes)

    def test_persisted_source_or_history_refuses_without_live_configured_entry(self):
        for key in ['default.configured.audio.source', 'default.configured.audio.source.0',
                    'default.configured.audio.source.8']:
            for section in ['default-nodes', 'default-streams']:
                with self.subTest(key=key, section=section):
                    with self.assertRaises(ValueError):
                        require_automatic_source(automatic(), state=f'[{section}]\n{key}=old-source\n')

    def test_missing_duplicate_or_malformed_default_metadata_is_not_certified_empty(self):
        cases = [None, {}, [], [{}],
                 [dict(type='PipeWire:Interface:Metadata', props={'metadata.name': 'settings'}, metadata=[])],
                 automatic() + automatic(),
                 [dict(type='PipeWire:Interface:Metadata', info={'props': {'metadata.name': 'default'}}, metadata=[])],
                 [dict(type='PipeWire:Interface:Metadata', props={'metadata.name': 'default'})],
                 [dict(type='PipeWire:Interface:Metadata', props={'metadata.name': 'default'}, metadata=None)],
                 [dict(type='PipeWire:Interface:Metadata', props={'metadata.name': 'default'}, metadata=[None])]]
        for nodes in cases:
            with self.subTest(nodes=nodes), self.assertRaises(ValueError):
                require_automatic_source(nodes)

    def test_source_clear_is_scoped_and_propagates_refusal(self):
        calls = []
        clear_reference_source(lambda *args: calls.append(args))
        self.assertEqual(calls, [('wpctl', 'clear-default', '1')])

        def fail(*args):
            raise RuntimeError('test-only clear refusal')

        with self.assertRaisesRegex(RuntimeError, 'clear refusal'):
            clear_reference_source(fail)


if __name__ == '__main__':
    unittest.main()
