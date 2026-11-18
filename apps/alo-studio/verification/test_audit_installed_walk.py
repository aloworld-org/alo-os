"""Test-first orchestration/report contract, never installed acceptance evidence.

API: audit_installed_walk.audit(evidence_dir, *, expected_source, regions) -> dict.
CLI: audit_installed_walk.py EVIDENCE --source SHA --regions REGIONS_JSON.
The CLI prints one JSON object, exits 0 only for passed media scope, and exits
1 for failed/unreached required checks. Neither API repairs evidence nor writes
to the supplied directory. Expected source is an explicit lowercase 40-char SHA.

Regions are explicit per-recording observations, not guessed defaults:
{'screen': {'marker_region': [x,y,w,h], 'motion_region': [x,y,w,h]},
 'window': {'marker_region': [x,y,w,h], 'motion_region': [x,y,w,h]}}.
Call decode_media.decode for each actual path with those exact regions; feed
its observations unchanged to recording_health.inspect(microphone=False/True).
Use reference_health.inspect on reference.json before certifying window sync.
Module-qualified calls keep these real public seams independently observable.

Report v1: scope='decoded-recording-media', status='passed'/'failed',
installed_acceptance='not_reached', checks indexed by the names below. Every
check has status passed/failed/not_reached; failed/unreached checks have a
nonempty reason. Reference and recording checks retain inspector results under
'measurements'. No prose success substitutes for an unattempted check.
Provenance retains the observed fields under 'identity'; top-level 'regions'
retains the explicit boxes and 'inputs' hashes the four consumed canonical files.

session-report.json must contain exactly one normal installed-provenance event
with source, verification=False, studio_commit, reference_commit, runtime and
runtime_commit; missing/ambiguous/bad/mismatched provenance blocks media checks.
This validates an observation's identity, not authenticity or the native flow.
Other person-flow, grant/model, hardware and full-walk evidence stays separate.

All fixtures below are synthetic and visibly test-only. Patched decoding in
unit cases establishes orchestration only. The CLI case uses closed FFmpeg-made
WebMs and real inspectors; it still does not establish installed OS behavior.
"""

import copy
import hashlib
import json
import math
import pathlib
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
import wave
from unittest.mock import patch

import audit_installed_walk
import recording_health
import reference_health
from test_recording_health import observations
from test_reference_health import healthy


SOURCE = 'a' * 40
REGIONS = {
    'screen': {'marker_region': [0, 0, 8, 16], 'motion_region': [8, 0, 8, 16]},
    'window': {'marker_region': [0, 0, 8, 16], 'motion_region': [8, 0, 8, 16]},
}
CORE = ['provenance', 'reference', 'screen_decode', 'screen_recording',
        'window_decode', 'window_recording']


def write_inputs(folder):
    """Inert test-only inputs; unit tests replace the decoder, never the files."""
    provenance = dict(kind='installed-provenance', observed_ns=100,
                      source=SOURCE, verification=False,
                      studio_commit='1' * 64, reference_commit='2' * 64,
                      runtime='org.gnome.Platform/x86_64/49', runtime_commit='3' * 64)
    session = {'version': 1, 'events': [provenance]}
    (folder / 'session-report.json').write_text(json.dumps(session))
    (folder / 'reference.json').write_text(json.dumps(healthy()))
    for source in ['screen', 'window']:
        (folder / f'{source}.webm').write_bytes(b'INERT TEST-ONLY DECODER INPUT ' + source.encode())
    return session


def bounded_claims(test, report):
    test.assertEqual(report['version'], 1)
    test.assertEqual(report['scope'], 'decoded-recording-media')
    test.assertEqual(report['installed_acceptance'], 'not_reached')
    for name in ['agent_permissions', 'hardware_microphone']:
        check = report['checks'][name]
        test.assertEqual(check['status'], 'not_reached')
        test.assertTrue(check['reason'])
    for check in report['checks'].values():
        test.assertIn(check['status'], ['passed', 'failed', 'not_reached'])
        if check['status'] != 'passed':
            test.assertIsInstance(check['reason'], str)
            test.assertTrue(check['reason'].strip())
    # Reject NaN/infinity and objects that only happen to print like a report.
    json.dumps(report, allow_nan=False)


class AuditInstalledWalk(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='studio-audit-test-only-')
        self.addCleanup(self.temporary.cleanup)
        self.folder = pathlib.Path(self.temporary.name)
        self.session = write_inputs(self.folder)
        self.decoded = {source: observations(microphone=source == 'window')
                        for source in ['screen', 'window']}

    def save_session(self):
        (self.folder / 'session-report.json').write_text(json.dumps(self.session))

    def inspect(self, *, decode_error=None, regions=None, expected_source=SOURCE):
        """Wrap actual health inspectors; only decoded transport is replaced."""
        calls = []

        def decode(path, *, marker_region, motion_region):
            source = pathlib.Path(path).stem
            self.assertEqual(pathlib.Path(path), self.folder / f'{source}.webm')
            self.assertEqual(list(marker_region), REGIONS[source]['marker_region'])
            self.assertEqual(list(motion_region), REGIONS[source]['motion_region'])
            calls.append(source)
            if source == decode_error:
                raise ValueError('test-only incomplete decode')
            return self.decoded[source]

        with patch('audit_installed_walk.decode_media.decode', side_effect=decode) as decoded, \
                patch('audit_installed_walk.reference_health.inspect', wraps=reference_health.inspect) as reference, \
                patch('audit_installed_walk.recording_health.inspect', wraps=recording_health.inspect) as recording:
            result = audit_installed_walk.audit(
                self.folder, expected_source=expected_source,
                regions=copy.deepcopy(REGIONS if regions is None else regions))
        bounded_claims(self, result)
        return result, decoded, reference, recording

    def test_actual_inspectors_receive_unmodified_observations_and_explicit_regions(self):
        before = copy.deepcopy(self.decoded)
        files = {p.name: hashlib.sha256(p.read_bytes()).digest() for p in self.folder.iterdir()}
        result, decoded, reference, recording = self.inspect()
        self.assertEqual(result['status'], 'passed')
        self.assertTrue(all(result['checks'][name]['status'] == 'passed' for name in CORE))
        self.assertEqual(decoded.call_count, 2)
        self.assertEqual(reference.call_count, 1)
        self.assertEqual(recording.call_count, 2)
        self.assertEqual(result['regions'], REGIONS)
        for field in ['source', 'verification', 'studio_commit', 'reference_commit', 'runtime', 'runtime_commit']:
            self.assertEqual(result['checks']['provenance']['identity'][field], self.session['events'][0][field])
        self.assertEqual(set(result['inputs']), {'screen.webm', 'window.webm', 'reference.json', 'session-report.json'})
        for name, facts in result['inputs'].items():
            self.assertEqual(facts['sha256'], hashlib.sha256((self.folder / name).read_bytes()).hexdigest())
        for call in recording.call_args_list:
            source = 'window' if call.kwargs['microphone'] else 'screen'
            self.assertIs(call.args[0], self.decoded[source][0])
            self.assertIs(call.args[1], self.decoded[source][1])
            self.assertIs(type(call.kwargs['microphone']), bool)
            self.assertEqual(result['checks'][source + '_recording']['measurements'],
                             recording_health.inspect(*self.decoded[source], microphone=source == 'window'))
        self.assertEqual(result['checks']['reference']['measurements'], reference_health.inspect(healthy()))
        self.assertEqual(self.decoded, before)
        self.assertEqual({p.name: hashlib.sha256(p.read_bytes()).digest() for p in self.folder.iterdir()}, files)

    def test_source_features_or_deployed_identity_refusal_blocks_downstream_checks(self):
        changes = [('source', 'b' * 40), ('source', 'A' * 40), ('source', ''),
                   ('verification', True), ('verification', 0),
                   ('studio_commit', '1' * 63), ('reference_commit', ''),
                   ('runtime_commit', '3' * 63), ('runtime', ''), ('runtime', 'two\nruntimes')]
        original = copy.deepcopy(self.session)
        for field, value in changes:
            with self.subTest(field=field, value=value):
                self.session = copy.deepcopy(original)
                self.session['events'][0][field] = value
                self.save_session()
                report, decoded, reference, recording = self.inspect()
                self.assertEqual(report['status'], 'failed')
                self.assertEqual(report['checks']['provenance']['status'], 'failed')
                for name in CORE[1:]:
                    self.assertEqual(report['checks'][name]['status'], 'not_reached')
                decoded.assert_not_called()
                reference.assert_not_called()
                recording.assert_not_called()

    def test_missing_duplicate_or_malformed_session_provenance_is_not_success(self):
        original = copy.deepcopy(self.session)
        cases = [None, {}, {'version': 2, 'events': original['events']},
                 {'version': 1, 'events': []}, {'version': 1, 'events': [None]},
                 {'version': 1, 'events': original['events'] * 2}]
        for session in cases:
            with self.subTest(session=session):
                (self.folder / 'session-report.json').write_text(json.dumps(session))
                report, decoded, _, _ = self.inspect()
                self.assertEqual(report['checks']['provenance']['status'], 'failed')
                self.assertEqual(report['status'], 'failed')
                decoded.assert_not_called()

    def test_missing_or_unparseable_session_file_has_a_structured_failed_reason(self):
        for contents in [None, '{incomplete JSON']:
            with self.subTest(contents=contents):
                path = self.folder / 'session-report.json'
                path.unlink(missing_ok=True)
                if contents is not None:
                    path.write_text(contents)
                report, decoded, _, _ = self.inspect()
                self.assertEqual(report['checks']['provenance']['status'], 'failed')
                self.assertEqual(report['status'], 'failed')
                decoded.assert_not_called()

    def test_failed_live_reference_cannot_be_replaced_by_continuous_recorded_pcm(self):
        trace = healthy()
        trace['errors'] = ['test-only stopped live reference']
        (self.folder / 'reference.json').write_text(json.dumps(trace))
        report, _, reference, recording = self.inspect()
        self.assertEqual(report['status'], 'failed')
        self.assertEqual(report['checks']['reference']['status'], 'failed')
        self.assertEqual(report['checks']['window_recording']['status'], 'not_reached')
        reference.assert_called_once()
        self.assertFalse(any(call.kwargs['microphone'] for call in recording.call_args_list))

    def test_missing_or_bad_reference_is_failed_not_silently_unreached_or_success(self):
        for contents in [None, '{}', '{truncated']:
            with self.subTest(contents=contents):
                path = self.folder / 'reference.json'
                path.unlink(missing_ok=True)
                if contents is not None:
                    path.write_text(contents)
                report, _, _, recording = self.inspect()
                self.assertEqual(report['checks']['reference']['status'], 'failed')
                self.assertEqual(report['checks']['window_recording']['status'], 'not_reached')
                self.assertEqual(report['status'], 'failed')
                self.assertFalse(any(call.kwargs['microphone'] for call in recording.call_args_list))

    def test_each_decode_failure_blocks_its_health_but_not_the_other_recording(self):
        for source in ['screen', 'window']:
            with self.subTest(source=source):
                report, _, _, _ = self.inspect(decode_error=source)
                other = 'window' if source == 'screen' else 'screen'
                self.assertEqual(report['status'], 'failed')
                self.assertEqual(report['checks'][source + '_decode']['status'], 'failed')
                self.assertIn('incomplete decode', report['checks'][source + '_decode']['reason'])
                self.assertEqual(report['checks'][source + '_recording']['status'], 'not_reached')
                self.assertEqual(report['checks'][other + '_recording']['status'], 'passed')

    def test_actual_numeric_recording_refusals_are_not_swallowed_by_orchestration(self):
        for source in ['screen', 'window']:
            with self.subTest(source=source):
                self.decoded = {s: observations(microphone=s == 'window') for s in ['screen', 'window']}
                # Two separately covered promises: frozen motion and a 50 ms PCM gap.
                if source == 'screen':
                    self.decoded[source][0]['frames'][1]['fingerprint'] = self.decoded[source][0]['frames'][0]['fingerprint']
                else:
                    del self.decoded[source][1]['chunks'][500:505]
                report, _, _, _ = self.inspect()
                self.assertEqual(report['status'], 'failed')
                self.assertEqual(report['checks'][source + '_decode']['status'], 'passed')
                self.assertEqual(report['checks'][source + '_recording']['status'], 'failed')

    def test_silent_and_microphone_expectations_cannot_be_inferred_from_available_tracks(self):
        for source in ['screen', 'window']:
            with self.subTest(source=source):
                self.decoded = {s: observations(microphone=s == 'window') for s in ['screen', 'window']}
                self.decoded[source] = observations(microphone=source == 'screen')
                report, _, _, _ = self.inspect()
                self.assertEqual(report['checks'][source + '_recording']['status'], 'failed')
                self.assertEqual(report['status'], 'failed')

    def test_missing_regions_are_not_guessed_from_defaults_or_other_recording(self):
        for source in ['screen', 'window']:
            with self.subTest(source=source):
                regions = copy.deepcopy(REGIONS)
                del regions[source]
                report, _, _, _ = self.inspect(regions=regions)
                self.assertEqual(report['checks'][source + '_decode']['status'], 'failed')
                self.assertEqual(report['checks'][source + '_recording']['status'], 'not_reached')
                self.assertEqual(report['status'], 'failed')

    def test_expected_source_is_required_to_be_a_lowercase_commit_identity(self):
        for expected in ['', 'A' * 40, 'a' * 39, None]:
            with self.subTest(expected=expected):
                report, decoded, _, _ = self.inspect(expected_source=expected)
                self.assertEqual(report['checks']['provenance']['status'], 'failed')
                self.assertEqual(report['status'], 'failed')
                decoded.assert_not_called()

    def test_nonfinite_or_non_json_region_inputs_return_safe_failed_reports(self):
        invalid = [float('nan'), float('inf'), float('-inf'), b'not JSON', object(),
                   {'screen': {'marker_region': [float('nan'), 0, 8, 16],
                               'motion_region': [8, 0, 8, 16]}},
                   {'screen': {'marker_region': [0, 0, 8, 16],
                               'motion_region': [8, 0, 8, 16], 'unexpected': object()}}]
        for regions in invalid:
            with self.subTest(kind=type(regions).__name__):
                # A decoder refusal must never be hidden by a second exception
                # while serializing the invalid coordinates into its report.
                with patch('audit_installed_walk.decode_media.decode',
                           side_effect=ValueError('test-only invalid regions')):
                    report = audit_installed_walk.audit(self.folder, expected_source=SOURCE, regions=regions)
                bounded_claims(self, report)
                self.assertEqual(report['status'], 'failed')
                self.assertEqual(report['checks']['screen_decode']['status'], 'failed')

    @unittest.skipUnless(shutil.which('ffmpeg') and shutil.which('ffprobe'), 'requires real FFmpeg tools')
    def test_bad_coordinate_types_and_extra_region_values_cannot_be_success(self):
        subprocess.run(['ffmpeg', '-v', 'error', '-y', '-f', 'lavfi',
                        '-i', 'testsrc2=size=16x16:rate=5:duration=1', '-c:v', 'libvpx',
                        str(self.folder / 'screen.webm')], check=True, capture_output=True, timeout=30)
        baseline = audit_installed_walk.audit(self.folder, expected_source=SOURCE, regions=copy.deepcopy(REGIONS))
        self.assertEqual(baseline['checks']['screen_decode']['status'], 'passed', baseline)
        invalid = [None, '0,0,8,16', [0, 0, 8], [False, 0, 8, 16],
                   [0.5, 0, 8, 16], [0, 0, 8, float('inf')], [0, 0, 8, 16, 1]]
        for rectangle in invalid:
            with self.subTest(rectangle=rectangle):
                regions = copy.deepcopy(REGIONS)
                regions['screen']['marker_region'] = rectangle
                # Use the real decoder here: it rejects malformed rectangle
                # fields without the wrapper manufacturing a successful check.
                report = audit_installed_walk.audit(self.folder, expected_source=SOURCE, regions=regions)
                bounded_claims(self, report)
                self.assertEqual(report['checks']['screen_decode']['status'], 'failed')
                self.assertEqual(report['checks']['screen_recording']['status'], 'not_reached')
                self.assertEqual(report['status'], 'failed')


@unittest.skipUnless(shutil.which('ffmpeg') and shutil.which('ffprobe'), 'requires real FFmpeg tools')
class AuditCli(unittest.TestCase):
    def test_cli_nonfinite_or_malformed_regions_emit_failed_json_without_a_traceback(self):
        with tempfile.TemporaryDirectory(prefix='studio-audit-invalid-region-test-') as directory:
            folder = pathlib.Path(directory)
            write_inputs(folder)
            regions_file = folder / 'regions.json'
            script = pathlib.Path(audit_installed_walk.__file__).resolve()
            command = [sys.executable, '-I', str(script), str(folder), '--source', SOURCE,
                       '--regions', str(regions_file)]
            cases = ['NaN', 'Infinity', '-Infinity', '{truncated']
            for value in ['NaN', 'Infinity', '-Infinity']:
                cases.append('{"screen":{"marker_region":[' + value + ',0,8,16],'
                             '"motion_region":[8,0,8,16]},"window":'
                             '{"marker_region":[0,0,8,16],"motion_region":[8,0,8,16]}}')
            for contents in cases:
                with self.subTest(contents=contents):
                    regions_file.write_text(contents)
                    process = subprocess.run(command, capture_output=True, text=True, timeout=30)
                    self.assertEqual(process.returncode, 1, process.stderr + process.stdout)
                    self.assertNotIn('Traceback', process.stderr)
                    report = json.loads(process.stdout)
                    bounded_claims(self, report)
                    self.assertEqual(report['status'], 'failed')
                    self.assertEqual(report['checks']['screen_decode']['status'], 'failed')

    def test_cli_decodes_closed_test_webms_and_prints_one_bounded_result_on_success_and_refusal(self):
        with tempfile.TemporaryDirectory(prefix='studio-audit-ffmpeg-test-only-') as directory:
            folder = pathlib.Path(directory)
            write_inputs(folder)
            video, audio = observations()
            raw = folder / 'test-only.rgb'
            pixels = bytearray()
            for index, frame in enumerate(video['frames']):
                for _y in range(16):
                    for x in range(16):
                        value = (255 if frame['lit'] else 0) if x < 8 else (255 if index & (1 << (x - 8)) else 0)
                        pixels.extend([value] * 3)
            raw.write_bytes(pixels)
            sound = folder / 'test-only.wav'
            with wave.open(str(sound), 'wb') as wav:
                wav.setnchannels(1)
                wav.setsampwidth(2)
                wav.setframerate(16000)
                wav.writeframes(b''.join(struct.pack('<h', round(8000 * math.sin(2 * math.pi * 880 * n / 16000))
                                                     if audio['chunks'][n // 160]['tone'] else 0)
                                         for n in range(len(audio['chunks']) * 160)))
            for source in ['screen', 'window']:
                args = ['ffmpeg', '-v', 'error', '-y', '-f', 'rawvideo', '-pixel_format', 'rgb24',
                        '-video_size', '16x16', '-framerate', '5', '-i', str(raw)]
                if source == 'window':
                    args += ['-i', str(sound), '-c:a', 'libopus', '-b:a', '24k', '-frame_duration', '60']
                args += ['-c:v', 'libvpx', '-crf', '4', '-b:v', '1M', str(folder / f'{source}.webm')]
                subprocess.run(args, check=True, capture_output=True, timeout=60)
            regions = folder / 'regions.json'
            regions.write_text(json.dumps(REGIONS))
            script = pathlib.Path(audit_installed_walk.__file__).resolve()
            command = [sys.executable, '-I', str(script), str(folder), '--source', SOURCE, '--regions', str(regions)]
            for reference_ok in [True, False]:
                if not reference_ok:
                    trace = healthy()
                    trace['errors'] = ['test-only reference failure']
                    (folder / 'reference.json').write_text(json.dumps(trace))
                process = subprocess.run(command, capture_output=True, text=True, timeout=60)
                self.assertEqual(process.returncode, 0 if reference_ok else 1, process.stderr + process.stdout)
                report = json.loads(process.stdout)
                bounded_claims(self, report)
                self.assertEqual(report['status'], 'passed' if reference_ok else 'failed')
                if reference_ok:
                    self.assertTrue(all(report['checks'][name]['status'] == 'passed' for name in CORE))
                else:
                    self.assertEqual(report['checks']['reference']['status'], 'failed')
                    self.assertEqual(report['checks']['window_recording']['status'], 'not_reached')
            (folder / 'reference.json').write_text(json.dumps(healthy()))
            (folder / 'window.webm').unlink()
            process = subprocess.run(command, capture_output=True, text=True, timeout=60)
            self.assertEqual(process.returncode, 1, process.stderr + process.stdout)
            report = json.loads(process.stdout)
            bounded_claims(self, report)
            self.assertEqual(report['checks']['window_decode']['status'], 'failed')
            self.assertEqual(report['checks']['window_recording']['status'], 'not_reached')


if __name__ == '__main__':
    unittest.main()
