"""Keyboard/screen-reader playback against the actual normal Studio Review.

Run on an owned Xvfb with STUDIO_PRIVATE_REVIEW_SESSION=1 and an explicit normal
STUDIO_REVIEW_PLAYBACK_APP binary. Media is generated test-only VP8+Opus, opened
through the existing versioned ReviewRecording API. Play/Pause uses Tab+Space,
never mocked state or a fabricated decoder. AT-SPI Value is seconds (double).

Position proves MediaStream progress, not audible output. Optional retained
window PNGs let a reviewer separately inspect rendered frames. This is local
normal-app evidence; it does not certify installed OS recording or consent.
"""

import json
import math
import os
from pathlib import Path
import re
import shutil
import tempfile
import time
import unittest

from fixtures.review_process import ReviewProcess, media, provenance


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_REVIEW_SESSION') == '1',
                     'requires an explicit actual Studio binary and owned Xvfb')
class ActualReviewPlayback(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.binary = Path(os.environ['STUDIO_REVIEW_PLAYBACK_APP']).resolve(strict=True)
        if os.environ.get('STUDIO_REVIEW_PLAYBACK_FEATURE_SCOPE') != 'normal':
            raise AssertionError('this acceptance check requires an explicitly verified normal build')
        for command in ('ffmpeg', 'ffprobe', 'dbus-daemon', 'xdotool', 'pipewire',
                        'wireplumber', 'pipewire-pulse', 'pactl'):
            if not shutil.which(command):
                raise AssertionError('required playback fixture dependency missing: ' + command)
        if os.environ.get('STUDIO_REVIEW_PLAYBACK_EVIDENCE') and not shutil.which('import'):
            raise AssertionError('window evidence requested but ImageMagick import is missing')
        cls.media_held = tempfile.TemporaryDirectory(prefix='studio-playback-media-')
        cls.addClassCleanup(cls.media_held.cleanup)
        folder = Path(cls.media_held.name)
        cls.clips = {name: media(folder, name, duration)
                     for name, duration in [('long', 12), ('short', 1.8), ('other', 5)]}
        bad = folder / 'broken.webm'
        bad.write_bytes(b'This is deliberately not a media container.\n')
        cls.clips['broken'] = (bad, 3)
        print('ACTUAL_NORMAL_APP', json.dumps(provenance(cls.binary)), flush=True)

    def setUp(self):
        held = tempfile.TemporaryDirectory(prefix='studio-actual-review-')
        self.addCleanup(held.cleanup)
        self.app = ReviewProcess(Path(held.name), self.binary, self.clips)
        self.addCleanup(self.app.close)

    def controls(self, clip='long'):
        self.app.open(clip)
        self.capture('selected-' + clip)
        def prepared_controls():
            found = self.app.named('Play recording', 43)
            position = self.app.values()
            if (found and {24, 25, 30} <= found['states'] and position
                    and {24, 25, 30} <= position[0]['states'] and position[1][1] > 0):
                return found, position
            return None
        play, (position, values) = self.app.until(prepared_controls,
                              'actual Review has no usable named exterior Play recording button', 8)
        self.assertLessEqual({24, 25, 30}, play['states'], 'Play must be usable and visible without pointer motion')
        self.assertLessEqual({24, 25, 30}, position['states'])
        low, high, current = values
        self.assertTrue(all(math.isfinite(value) for value in values))
        self.assertEqual(low, 0)
        self.assertAlmostEqual(high, self.clips[clip][1], delta=.3,
                               msg='range must describe actual selected media in seconds')
        self.assertGreaterEqual(current, 0)
        self.assertLessEqual(current, .15, 'selecting a saved recording must not autoplay')
        self.readable_time(current, high)
        return current

    def readable_time(self, current, duration):
        pattern = r'([0-9]+(?::[0-9]{2}){1,2}) / ([0-9]+(?::[0-9]{2}){1,2})'
        # GTK's label accessible exposes its actual text as Name, even when a
        # Label property is supplied. Require that readable role29 text itself,
        # rather than a synthetic name GTK does not publish.
        labels = [item for item in self.app.nodes() if item['role'] == 29
                  and {25, 30} <= item['states'] and re.fullmatch(pattern, item['name'])]
        self.assertEqual(len(labels), 1, 'elapsed and duration must have one visible readable label')
        label = labels[0]
        text = str(self.app.interface(label['node'], 'org.a11y.atspi.Text').GetText(0, -1, timeout=3))
        match = re.fullmatch(pattern, text)
        self.assertIsNotNone(match, 'actual elapsed and duration text must be readable: ' + repr(text))
        def seconds(clock):
            total = 0
            for part in clock.split(':'):
                total = total * 60 + int(part)
            return total
        self.assertAlmostEqual(seconds(match[1]), current, delta=1.1,
                               msg='readable elapsed label must track real playback position')
        self.assertAlmostEqual(seconds(match[2]), duration, delta=1.1,
                               msg='readable duration must describe selected media')

    def current(self):
        result = self.app.values()
        self.assertIsNotNone(result, 'selected recording lost its position control')
        return result[1][2]

    def advanced(self, start, amount=.6):
        return self.app.until(lambda: self.current() if self.current() >= start + amount else None,
                              'actual MediaStream position did not advance after keyboard Play', 6)

    def frozen(self):
        # Let the already-dispatched last timestamp notification settle, then
        # sample for a full second; a button label alone cannot prove pause.
        time.sleep(.15)
        start = self.current()
        values = []
        deadline = time.monotonic() + 1
        while time.monotonic() < deadline:
            values.append(self.current())
            time.sleep(.1)
        self.assertTrue(all(abs(value - start) <= .08 for value in values),
                        f'position kept advancing after pause/no-autoplay: {start}, {values}')
        return start

    def capture(self, suffix):
        if os.environ.get('STUDIO_REVIEW_PLAYBACK_EVIDENCE'):
            self.app.capture(self._testMethodName + '-' + suffix)

    def test_keyboard_play_advances_real_media_pause_freezes_and_resume_continues(self):
        start = self.controls()
        self.frozen()
        self.capture('ready')
        self.app.keyboard('Play recording')
        self.app.until(lambda: self.app.named('Pause playback', 43), 'playing state did not expose Pause playback')
        first = self.advanced(start, 1.1)
        self.capture('playing-first')
        self.advanced(first, 1.1)
        self.readable_time(self.current(), self.clips['long'][1])
        self.capture('playing-later')
        self.app.keyboard('Pause playback')
        self.app.until(lambda: self.app.named('Play recording', 43), 'paused state did not expose Play recording')
        paused = self.frozen()
        self.capture('paused')
        self.app.keyboard('Play recording')
        self.advanced(paused)
        self.app.keyboard('Pause playback')
        self.frozen()

    def test_ended_recording_offers_replay_and_keyboard_restarts_real_media(self):
        self.controls('short')
        self.app.keyboard('Play recording')
        first = self.advanced(0, .4)
        observations = []
        furthest = [first]
        def ended():
            values = self.app.values()[1]
            buttons = [item['name'] for item in self.app.nodes() if item['role'] == 43
                       and item['name'] in ('Play recording', 'Pause playback')]
            observations.append(dict(values=values, buttons=buttons))
            furthest.append(values[2])
            duration = self.clips['short'][1]
            # GTK media backends may return timestamp zero at EOS. Require
            # actual progression near the end, then stable replay state; do
            # not manufacture a terminal timestamp the backend did not emit.
            return ('Play recording' in buttons and max(furthest) >= duration - .45
                    and (values[2] <= .15 or values[2] >= duration - .25))
        try:
            self.app.until(ended, 'end of real media did not offer replay', 7)
        except AssertionError as error:
            self.capture('eof-failure')
            raise AssertionError(f'{error}; actual EOF observations={observations[-12:]}') from error
        self.frozen()
        self.app.keyboard('Play recording')
        self.app.until(lambda: self.app.named('Pause playback', 43)
                       and self.current() < self.clips['short'][1] - .3,
                       'replaying an ended recording did not seek to its start', 3)
        self.advanced(0, .35)

    def test_keyboard_seek_moves_real_stream_while_paused_and_home_returns_to_start(self):
        self.controls()
        self.app.key_control('Playback position', 51, 'Right')
        self.app.until(lambda: self.current() >= .5,
                       'keyboard Right did not seek the actual MediaStream', 4)
        self.assertIsNotNone(self.app.named('Play recording', 43), 'seeking a paused clip must not start it')
        sought = self.frozen()
        self.readable_time(sought, self.clips['long'][1])
        self.capture('seeked-paused')
        self.app.key_control('Playback position', 51, 'Home')
        self.app.until(lambda: self.current() <= .15,
                       'keyboard Home did not seek the actual MediaStream to its start', 4)
        self.frozen()

    def test_switching_files_resets_position_and_old_stream_cannot_drive_new_controls(self):
        self.controls()
        self.app.keyboard('Play recording')
        self.advanced(0, 1)
        self.controls('other')
        self.frozen()
        self.app.keyboard('Play recording')
        self.advanced(0)
        self.app.keyboard('Pause playback')
        self.frozen()
        self.app.control.NewRecording(timeout=3)
        self.app.until(lambda: not any(item['name'] in ('Play recording', 'Pause playback', 'Playback position')
                                      and 25 in item['states'] for item in self.app.nodes()),
                       'leaving Review kept visible controls for the old recording')
        self.controls('long')
        self.frozen()

    def test_unreadable_recording_has_readable_error_disables_play_and_can_recover(self):
        self.app.open('broken')
        self.app.until(lambda: self.app.named('This recording could not be played.'),
                       'unreadable media did not present the concise playback error', 10)
        play = self.app.named('Play recording', 43)
        position = self.app.named('Playback position', 51)
        self.assertIsNotNone(play, 'error must leave a clearly named disabled Play control')
        self.assertIsNotNone(position)
        self.assertNotIn(24, play['states'])
        self.assertNotIn(24, position['states'])
        self.assertIsNone(self.app.named('Pause playback', 43))
        self.controls('other')
        self.assertFalse(any(item['name'] == 'This recording could not be played.' and 25 in item['states']
                             for item in self.app.nodes()), 'old media error remained visible after changing files')
        self.app.keyboard('Play recording')
        self.advanced(0)


if __name__ == '__main__':
    unittest.main(verbosity=2)
