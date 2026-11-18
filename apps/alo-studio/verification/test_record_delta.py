"""Independent test-first checks of person-side recording-permission evidence.

Public shapes: docs/contracts/record-file.md; alo-record::{Entry,Happened,What}.
Literal synthetic JSONL fixtures below are test inputs, never installed evidence.

record_delta.snapshot(path) returns an opaque checkpoint of an existing regular,
non-symlink record: version-1 head, device/inode, complete byte offset and retained
prefix identity. verify_microphone_change(path, checkpoint, *, enabled,
from_approval, full_access) reads only a bounded append and returns the seven
summary fields in expected_summary(). Both functions refuse invalid observations
with ValueError or OSError, without changing the file. Bounds: 4 MiB whole file,
1 MiB append, 64 KiB per JSONL line (including newline). Reads must not block on
a FIFO. JSON must be UTF-8, finite, unambiguous, and complete through the last LF.

The only selected actions are enabled=True -> studio.enable_microphone and
False -> studio.disable_microphone. enabled/full_access are strict booleans.
Ask requires a u64 approval (including zero, never bool), nonempty u64 grant numbers,
and false access flags; false flags may be omitted by the real serializer.
Full requires no approval, at_full_access/by_full_access true, and permits an
empty grant list. Other well-formed read/inference entries are permitted noise;
additional Studio change executions cannot be filtered out as nonmatches.

This verifies ONE RECORDED RUN. It establishes neither actual application state,
native approval input, single D-Bus delivery, nor installed-agent acceptance.
The eventual controller must obtain those observations independently.
"""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from record_delta import snapshot, verify_microphone_change


HEADER = b'{"format":1}\n'
SENTENCE = 'enable the microphone for this alo Studio recording'
APPROVAL = 73
MIB = 1024 * 1024


def line(value):
    return (json.dumps(value, ensure_ascii=False, separators=(',', ':')) + '\n').encode('utf-8')


def ran(*, enabled=True, full=False, approval=APPROVAL):
    """Literal record shape, independent of any production record constructor."""
    entry = {
        'at': {'secs_since_epoch': 1760000001, 'nanos_since_epoch': 12},
        'happened': {'ran': {
            'agent': 'alo',
            'what': {
                'verb': 'studio.enable_microphone' if enabled else 'studio.disable_microphone',
                'effect': 'change',
                'sentence': SENTENCE if enabled else 'disable the microphone for this alo Studio recording',
                'arguments': {},
            },
            'from_approval': None if full else approval,
            'against': [] if full else [19],
        }},
    }
    if full:
        entry['at_full_access'] = True
        entry['happened']['ran']['by_full_access'] = True
    return entry


def read_noise():
    entry = ran(approval=None)
    entry['happened']['ran']['what'] = {
        'verb': 'studio.status', 'effect': 'read',
        'sentence': "read alo Studio's recording status", 'arguments': {},
    }
    return entry


def inference_noise():
    return {'at': {'secs_since_epoch': 1760000001, 'nanos_since_epoch': 12},
            'happened': {'answered-here': {'agent': 'alo'}}}


def expected_summary(*, enabled=True, full=False):
    return {
        'agent': 'alo',
        'verb': 'studio.enable_microphone' if enabled else 'studio.disable_microphone',
        'recorded_runs': 1,
        'from_approval': None if full else APPROVAL,
        'against': [] if full else [19],
        'at_full_access': full,
        'by_full_access': full,
    }


class RecordDelta(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.path = self.directory / 'record.jsonl'
        self.path.write_bytes(HEADER)

    def append(self, *entries):
        with self.path.open('ab') as output:
            for entry in entries:
                output.write(line(entry))

    def verify(self, checkpoint, *, enabled=True, approval=APPROVAL, full=False):
        return verify_microphone_change(self.path, checkpoint, enabled=enabled,
                                        from_approval=approval, full_access=full)

    def refuses(self, operation):
        with self.assertRaises((ValueError, OSError)):
            operation()

    def assert_summary(self, summary, expected):
        # JSON distinguishes false/0 and true/1, which Python equality does not.
        self.assertEqual(json.dumps(summary, sort_keys=True, allow_nan=False),
                         json.dumps(expected, sort_keys=True, allow_nan=False))

    def test_native_approval_run_is_selected_only_after_the_checkpoint(self):
        self.append(ran())
        before = self.path.read_bytes()
        checkpoint = snapshot(self.path)
        self.assertEqual(self.path.read_bytes(), before)
        self.append(read_noise(), inference_noise(), ran(), read_noise())
        content = self.path.read_bytes()
        self.assert_summary(self.verify(checkpoint), expected_summary())
        self.assertEqual(self.path.read_bytes(), content)

    def test_full_access_disable_has_no_approval_and_may_have_no_grants(self):
        checkpoint = snapshot(self.path)
        self.append(ran(enabled=False, full=True))
        summary = self.verify(checkpoint, enabled=False, approval=None, full=True)
        self.assert_summary(summary, expected_summary(enabled=False, full=True))
        # The allowlist contains a recorded count, never a delivery or acceptance claim.
        self.assertLess(len(json.dumps(summary, allow_nan=False)), 1024)

    def test_explicit_false_flags_are_equivalent_to_real_omitted_false_flags(self):
        checkpoint = snapshot(self.path)
        entry = ran()
        entry['at_full_access'] = False
        entry['happened']['ran']['by_full_access'] = False
        self.append(entry)
        self.assert_summary(self.verify(checkpoint), expected_summary())

    def test_zero_is_a_real_approval_and_grant_number_not_missing_evidence(self):
        checkpoint = snapshot(self.path)
        entry = ran(approval=0)
        entry['happened']['ran']['against'] = [0]
        self.append(entry)
        self.assert_summary(self.verify(checkpoint, approval=0),
                            expected_summary() | {'from_approval': 0, 'against': [0]})

    def test_boolean_approvals_cannot_equal_the_real_numbers_zero_or_one(self):
        for boolean, number in [(False, 0), (True, 1)]:
            for invalid_api in [False, True]:
                with self.subTest(boolean=boolean, invalid_api=invalid_api):
                    self.path.write_bytes(HEADER)
                    checkpoint = snapshot(self.path)
                    self.append(ran(approval=number if invalid_api else boolean))
                    self.refuses(lambda: self.verify(checkpoint,
                                 approval=boolean if invalid_api else number))

    def test_read_and_inference_noise_do_not_supply_a_missing_change(self):
        checkpoint = snapshot(self.path)
        self.append(read_noise(), inference_noise())
        self.refuses(lambda: self.verify(checkpoint))

    def test_two_matching_runs_are_not_one_execution_record(self):
        checkpoint = snapshot(self.path)
        self.append(ran(), ran())
        self.refuses(lambda: self.verify(checkpoint))

    def test_wrong_studio_changes_cannot_hide_beside_one_matching_run(self):
        alternatives = []
        for field, value in [('from_approval', APPROVAL + 1), ('agent', 'someone-else')]:
            bad = ran()
            bad['happened']['ran'][field] = value
            alternatives.append(bad)
        alternatives.append(ran(enabled=False))
        bad = ran()
        bad['happened']['ran']['what']['arguments'] = {'invented': {'count': 1}}
        alternatives.append(bad)
        for bad in alternatives:
            with self.subTest(bad=bad):
                self.path.write_bytes(HEADER)
                checkpoint = snapshot(self.path)
                self.append(ran(), bad)
                self.refuses(lambda: self.verify(checkpoint))

    def test_matching_identity_action_effect_and_empty_arguments_are_required(self):
        cases = [('agent', 'other'), ('verb', 'studio.disable_microphone'),
                 ('verb', 'studio.enable_microphone.extra'), ('verb', 'Studio.enable_microphone'),
                 ('effect', 'read'), ('effect', 'Change'), ('arguments', []),
                 ('arguments', {'application': {'application': 'org.mozilla.firefox'}}),
                 ('sentence', ''), ('sentence', None)]
        for field, value in cases:
            with self.subTest(field=field, value=value):
                self.path.write_bytes(HEADER)
                checkpoint = snapshot(self.path)
                entry = ran()
                target = entry['happened']['ran'] if field == 'agent' else entry['happened']['ran']['what']
                target[field] = value
                self.append(entry)
                self.refuses(lambda: self.verify(checkpoint))

    def test_approval_and_grants_are_typed_and_bound_to_this_approval(self):
        cases = [('from_approval', None), ('from_approval', APPROVAL + 1),
                 ('from_approval', True), ('from_approval', float(APPROVAL)),
                 ('from_approval', str(APPROVAL)), ('from_approval', -1),
                 ('against', []), ('against', None), ('against', '19'),
                 ('against', [True]), ('against', [-1]), ('against', [1.0]),
                 ('against', [2 ** 64])]
        for field, value in cases:
            with self.subTest(field=field, value=value):
                self.path.write_bytes(HEADER)
                checkpoint = snapshot(self.path)
                entry = ran()
                entry['happened']['ran'][field] = value
                self.append(entry)
                self.refuses(lambda: self.verify(checkpoint))

    def test_wrong_policy_flags_are_not_truthy_or_defaulted_into_success(self):
        for full in [False, True]:
            for field in ['at_full_access', 'by_full_access']:
                for value in [not full, None, 0, 1, 'true']:
                    with self.subTest(full=full, field=field, value=value):
                        self.path.write_bytes(HEADER)
                        checkpoint = snapshot(self.path)
                        entry = ran(enabled=not full, full=full)
                        target = entry if field == 'at_full_access' else entry['happened']['ran']
                        target[field] = value
                        self.append(entry)
                        self.refuses(lambda: self.verify(checkpoint, enabled=not full,
                                                         approval=None if full else APPROVAL, full=full))
        for field in ['at_full_access', 'by_full_access']:
            self.path.write_bytes(HEADER)
            checkpoint = snapshot(self.path)
            entry = ran(enabled=False, full=True)
            target = entry if field == 'at_full_access' else entry['happened']['ran']
            del target[field]
            self.append(entry)
            self.refuses(lambda: self.verify(checkpoint, enabled=False, approval=None, full=True))

    def test_full_access_must_not_borrow_an_approval(self):
        checkpoint = snapshot(self.path)
        entry = ran(enabled=False, full=True)
        entry['happened']['ran']['from_approval'] = APPROVAL
        self.append(entry)
        self.refuses(lambda: self.verify(checkpoint, enabled=False, approval=None, full=True))

    def test_full_access_grant_list_remains_typed_even_when_it_may_be_empty(self):
        for against in [None, '19', [True], [-1], [1.0], [2 ** 64]]:
            with self.subTest(against=against):
                self.path.write_bytes(HEADER)
                checkpoint = snapshot(self.path)
                entry = ran(enabled=False, full=True)
                entry['happened']['ran']['against'] = against
                self.append(entry)
                self.refuses(lambda: self.verify(checkpoint, enabled=False, approval=None, full=True))

    def test_stopped_entry_is_not_a_run(self):
        checkpoint = snapshot(self.path)
        executed = ran()['happened']['ran']
        entry = {'at': {'secs_since_epoch': 1760000001, 'nanos_since_epoch': 12},
                 'happened': {'stopped': {'agent': 'alo', 'what': executed['what'],
                                          'how': {'at-the-moment': 'The grant was withdrawn'}}}}
        self.append(entry)
        self.refuses(lambda: self.verify(checkpoint))

    def test_missing_required_fields_cannot_be_inferred_from_the_expected_result(self):
        for where, fields in [('entry', ['at', 'happened']),
                              ('ran', ['agent', 'what', 'from_approval', 'against']),
                              ('what', ['verb', 'effect', 'sentence', 'arguments'])]:
            for field in fields:
                with self.subTest(where=where, field=field):
                    self.path.write_bytes(HEADER)
                    checkpoint = snapshot(self.path)
                    entry = ran()
                    targets = {'entry': entry, 'ran': entry['happened']['ran'],
                               'what': entry['happened']['ran']['what']}
                    del targets[where][field]
                    self.append(entry)
                    self.refuses(lambda: self.verify(checkpoint))

    def test_record_header_must_be_present_complete_and_version_one(self):
        for content in [b'', b'{}\n', b'{"format":2}\n', b'{"format":true}\n',
                        b'{"format":"1"}\n', b'{"format":1}', line(ran()),
                        b'{"format":2,"format":1}\n']:
            with self.subTest(content=content):
                self.path.write_bytes(content)
                self.refuses(lambda: snapshot(self.path))

    def test_additive_fields_and_a_valid_retention_header_remain_readable(self):
        self.path.write_bytes(line({'format': 1,
                                    'since': {'secs_since_epoch': 1760000000, 'nanos_since_epoch': 0},
                                    'under': 'forever'}))
        checkpoint = snapshot(self.path)
        entry = ran()
        entry['future_observation'] = {'opaque': 'must not be copied to the summary'}
        entry['happened']['ran']['future_field'] = 'additive'
        self.append(entry)
        self.assert_summary(self.verify(checkpoint), expected_summary())

    def test_malformed_partial_duplicate_and_nonfinite_json_lines_refuse(self):
        valid = line(ran())
        bad_lines = [b'not JSON\n', b'{}\n', b'[]\n', b'null\n', b'\n',
                     valid[:-1], valid[:-3] + b'\n', b'\xff\n',
                     valid.replace(b'"agent":"alo"', b'"agent":"foreign","agent":"alo"'),
                     valid.replace(b'"nanos_since_epoch":12', b'"nanos_since_epoch":NaN'),
                     valid.replace(b'"nanos_since_epoch":12', b'"nanos_since_epoch":Infinity')]
        for bad in bad_lines:
            for baseline in [False, True]:
                with self.subTest(bad=bad[:90], baseline=baseline):
                    self.path.write_bytes(HEADER)
                    if baseline:
                        with self.path.open('ab') as output:
                            output.write(bad)
                        self.refuses(lambda: snapshot(self.path))
                    else:
                        checkpoint = snapshot(self.path)
                        with self.path.open('ab') as output:
                            output.write(valid + bad)
                        self.refuses(lambda: self.verify(checkpoint))

    def test_invalid_timestamps_and_ambiguous_event_tags_refuse(self):
        cases = []
        for at in [None, {}, {'secs_since_epoch': -1, 'nanos_since_epoch': 0},
                   {'secs_since_epoch': True, 'nanos_since_epoch': 0},
                   {'secs_since_epoch': 1760000001, 'nanos_since_epoch': 1000000000}]:
            entry = ran()
            entry['at'] = at
            cases.append(entry)
        entry = ran()
        entry['happened']['answered-here'] = {'agent': 'alo'}
        cases.append(entry)
        for entry in cases:
            with self.subTest(entry=entry):
                self.path.write_bytes(HEADER)
                checkpoint = snapshot(self.path)
                self.append(entry)
                self.refuses(lambda: self.verify(checkpoint))

    def test_no_append_rotation_truncation_and_prefix_rewrite_refuse(self):
        self.append(inference_noise())
        checkpoint = snapshot(self.path)
        self.refuses(lambda: self.verify(checkpoint))
        original = self.path.read_bytes()
        self.path.rename(self.directory / 'old.jsonl')
        self.path.write_bytes(original + line(ran()))
        self.refuses(lambda: self.verify(checkpoint))
        checkpoint = snapshot(self.path)
        self.path.write_bytes(HEADER)
        self.refuses(lambda: self.verify(checkpoint))
        self.path.write_bytes(original)
        checkpoint = snapshot(self.path)
        # Same inode and size prefix, but rewritten history: cannot rely on size alone.
        self.path.write_bytes(original.replace(b'1760000001', b'1760000000') + line(ran()))
        self.refuses(lambda: self.verify(checkpoint))

    def test_changed_header_cannot_be_hidden_by_an_otherwise_valid_append(self):
        checkpoint = snapshot(self.path)
        self.path.write_bytes(b'{"format":2}\n' + line(ran()))
        self.refuses(lambda: self.verify(checkpoint))

    def test_missing_files_directories_and_symlinks_are_not_records(self):
        self.refuses(lambda: snapshot(self.directory / 'missing.jsonl'))
        self.refuses(lambda: snapshot(self.directory))
        link = self.directory / 'link.jsonl'
        link.symlink_to(self.path)
        self.refuses(lambda: snapshot(link))
        checkpoint = snapshot(self.path)
        self.append(ran())
        target = self.directory / 'target.jsonl'
        self.path.rename(target)
        self.path.symlink_to(target)
        self.refuses(lambda: self.verify(checkpoint))

    def test_fifo_refuses_promptly_instead_of_waiting_for_a_writer(self):
        fifo = self.directory / 'record.fifo'
        os.mkfifo(fifo)
        code = ('import sys; from pathlib import Path; '
                'sys.path.insert(0, sys.argv[1]); from record_delta import snapshot; '
                '\ntry: snapshot(Path(sys.argv[2]))'
                '\nexcept (ValueError, OSError): print("refused")'
                '\nelse: raise SystemExit("FIFO accepted")')
        completed = subprocess.run([sys.executable, '-I', '-B', '-c', code,
                                    str(Path(__file__).parent), str(fifo)],
                                   capture_output=True, text=True, timeout=2, check=False)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(completed.stdout, 'refused\n')

    def test_file_delta_and_line_limits_are_independent_refusals(self):
        # Whole-file refusal also bounds baseline hashing/validation.
        noise = line(inference_noise())
        self.path.write_bytes(HEADER + noise * (4 * MIB // len(noise) + 1))
        self.refuses(lambda: snapshot(self.path))
        self.path.write_bytes(HEADER)
        checkpoint = snapshot(self.path)
        # Every line is small/valid and total stays below the whole-file bound.
        with self.path.open('ab') as output:
            output.write(noise * (MIB // len(noise) + 1))
            output.write(line(ran()))
        self.refuses(lambda: self.verify(checkpoint))
        self.path.write_bytes(HEADER)
        checkpoint = snapshot(self.path)
        entry = ran()
        entry['oversize_additive_field'] = 'x' * (64 * 1024)
        self.append(entry)
        self.refuses(lambda: self.verify(checkpoint))
        self.refuses(lambda: snapshot(self.path))

    def test_a_small_append_cannot_push_a_large_baseline_past_the_file_bound(self):
        noise = line(inference_noise())
        self.path.write_bytes(HEADER + noise * ((4 * MIB - 128 * 1024) // len(noise)))
        checkpoint = snapshot(self.path)
        with self.path.open('ab') as output:
            output.write(noise * (256 * 1024 // len(noise)))
            output.write(line(ran()))
        self.refuses(lambda: self.verify(checkpoint))

    def test_api_inputs_cannot_coerce_another_action_or_policy(self):
        checkpoint = snapshot(self.path)
        self.append(ran())
        for enabled in [None, 0, 1, 'false', 'studio.finish']:
            with self.subTest(enabled=enabled):
                self.refuses(lambda: self.verify(checkpoint, enabled=enabled))
        for full in [None, 0, 1, 'false']:
            with self.subTest(full=full):
                self.refuses(lambda: self.verify(checkpoint, full=full))
        for approval in [None, True, -1, float(APPROVAL), str(APPROVAL), 2 ** 64]:
            with self.subTest(approval=approval):
                self.refuses(lambda: self.verify(checkpoint, approval=approval))
        self.refuses(lambda: self.verify(checkpoint, approval=APPROVAL, full=True))


if __name__ == '__main__':
    unittest.main()
