"""Additional read-only card checks from Standing/Wording's typed wire contract."""

import json
import socket
import threading
import time
import unittest

from pending_microphone import read_pending


def reply():
    return {'format': 1, 'tells': {'waiting': {'changes': [{
        'number': 0,
        'sentence': {'text': 'enable the microphone for this alo Studio recording',
                     'came_from': 'the-source'},
        'lapses_in': 1, 'touches': [{'application': 'org.aloworld.Studio'}],
    }]}}}


class PendingMicrophoneEdges(unittest.TestCase):
    def exchange(self, value):
        client, server = socket.socketpair()
        self.addCleanup(client.close)
        self.addCleanup(server.close)

        def answer():
            try:
                data = b''
                while not data.endswith(b'\n'):
                    part = server.recv(1024)
                    if not part:
                        return
                    data += part
                server.sendall((json.dumps(value) + '\n').encode())
            except OSError:
                pass

        worker = threading.Thread(target=answer)
        worker.start()
        try:
            return read_pending(client, enabled=True, timeout=.2)
        finally:
            client.close()
            worker.join(1)
            self.assertFalse(worker.is_alive())

    def test_lifetime_has_the_same_u64_wire_type_as_its_number(self):
        for invalid in (-1, 1.5, 2**64, float('inf')):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                value = reply()
                value['tells']['waiting']['changes'][0]['lapses_in'] = invalid
                self.exchange(value)

    def test_standing_and_wording_refuse_unknown_fields_as_the_protocol_does(self):
        for where in ('standing', 'wording'):
            with self.subTest(where=where), self.assertRaises(ValueError):
                value = reply()
                change = value['tells']['waiting']['changes'][0]
                target = change if where == 'standing' else change['sentence']
                target['unexpected'] = 'not part of this typed proposal'
                self.exchange(value)

    def test_falsy_values_cannot_disguise_a_malformed_text_vector(self):
        for invalid in (None, False, 0, '', {}):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                value = reply()
                value['tells']['waiting']['changes'][0]['texts'] = invalid
                self.exchange(value)

    def test_translation_and_an_empty_text_vector_are_valid_typed_fields(self):
        value = reply()
        change = value['tells']['waiting']['changes'][0]
        change['sentence']['came_from'] = 'translation'
        change['texts'] = []
        self.assertEqual(self.exchange(value)['number'], 0)

    def test_invalid_options_refuse_before_sending_any_request(self):
        for options in ({'enabled': 1}, {'enabled': None}, {'timeout': True},
                        {'timeout': 0}, {'timeout': -1}, {'timeout': 3.01},
                        {'timeout': float('nan')}, {'timeout': float('inf')}):
            with self.subTest(options=options):
                client, server = socket.socketpair()
                try:
                    server.settimeout(.02)
                    arguments = dict(enabled=True, timeout=.1)
                    arguments.update(options)
                    with self.assertRaises(ValueError):
                        read_pending(client, **arguments)
                    with self.assertRaises(TimeoutError):
                        server.recv(1024)
                finally:
                    client.close()
                    server.close()

    def test_partial_reply_trickle_does_not_extend_the_absolute_deadline(self):
        client, server = socket.socketpair()
        stopped = threading.Event()

        def trickle():
            try:
                server.recv(1024)
                while not stopped.wait(.02):
                    server.sendall(b' ')
            except OSError:
                pass

        worker = threading.Thread(target=trickle)
        worker.start()
        started = time.monotonic()
        try:
            with self.assertRaises((ValueError, TimeoutError)):
                read_pending(client, enabled=True, timeout=.12)
            self.assertLess(time.monotonic() - started, .8)
        finally:
            stopped.set()
            client.close()
            server.close()
            worker.join(1)
            self.assertFalse(worker.is_alive())


if __name__ == '__main__':
    unittest.main()
